mod head;
pub(super) use head::ForInIterationHead;

use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_for_in_loop(
        &mut self,
        for_in: &boa_ast::statement::iteration::ForInLoop,
    ) -> (StatementIr, ValueKind) {
        let owner = self.analysis.for_in_continuation_owners
            [&(for_in as *const boa_ast::statement::iteration::ForInLoop as usize)];
        if let crate::analysis::ForInContinuationOwner::CompleteWhole(owner) = owner {
            return self.lower_async_generator_for_in(for_in, owner);
        }
        if self.plain_async_entry_state().is_some()
            && contains(for_in.body(), ContainsSymbol::AwaitExpression)
        {
            self.unsupported("await inside a for-in loop");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        // This Annex B invalid-Reference facility is an eager throwing head,
        // not a key/body continuation. Preserve its original evaluation path.
        if let IterableLoopInitializer::WebCompatCall(call) = for_in.initializer() {
            let prefix = self.lower_for_in_initializer_prefix(for_in.initializer());
            return Self::prepend_statement(
                prefix,
                StatementIr::Expression(
                    self.lower_web_compat_loop_assignment_target(call, for_in.target()),
                ),
                ValueKind::Undefined,
            );
        }
        match owner {
            crate::analysis::ForInContinuationOwner::ImmediateOrLinear => {}
            crate::analysis::ForInContinuationOwner::CompleteWhole(_) => {
                unreachable!(
                    "the checked complete mixed owner was dispatched before the linear refusal"
                )
            }
        }
        let initializer_prefix = self.lower_for_in_initializer_prefix(for_in.initializer());
        let Some(head) = self.prepare_for_in_iteration_head(for_in) else {
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let lexical_environment = self.lower_for_in_of_environment(
            for_in as *const boa_ast::statement::iteration::ForInLoop as usize,
        );
        let mut target = self.lower_for_in_head_target(&head, for_in.target(), |this, source| {
            this.lower_expression(source)
        });
        self.widen_for_in_parameter_target(&mut target, for_in.target());
        let is_dynamic_target = target.kind == ValueKind::Dynamic;
        let is_array_target = target.possible_kinds.contains(ValueKind::Array)
            || target.possible_kinds.contains(ValueKind::Arguments);
        let is_string_target = target.possible_kinds.contains(ValueKind::String)
            || Self::value_info_is_boxed_string(&target.value_info());
        // 14.7.5.6 ForIn/OfHeadEvaluation step 3.a: when `exprValue` is
        // `undefined` or `null`, return a **break completion**. The head is
        // evaluated, the loop body runs zero times, and nothing throws — a
        // well-formed statement, not a compiler gap. `for (key in undefined)`
        // and `for (var x in null) ;` depend on exactly this, and refusing them
        // was the opposite of the spec rather than an unimplemented corner.
        //
        // Only a *statically* nullish head moves here. A `Dynamic` target that
        // happens to be nullish at run time already takes the ForInObject path,
        // which performs the same test there — which is why this is a handful
        // of cases and not a broad class.
        //
        // Steps 1-2 still evaluate the head for its effects, so this returns the
        // lowered target rather than `StatementIr::Empty`; the `Comma` restores
        // the `undefined` completion the break completion carries through
        // UpdateEmpty, which a bare expression statement would replace with the
        // head's own value.
        //
        // The `matches!` is not redundant with the subset test: an empty
        // `possible_kinds` is a subset of everything, and a vacuous hit here
        // would silently turn a loop that must iterate into one that cannot.
        //
        // The tradeoff to know about: this returns **before the body is lowered
        // at all**, so an unsupported construct inside the body of a statically
        // nullish `for-in` is now accepted rather than refused. That is
        // spec-correct — the body never runs — but it means a test262 case can
        // move to green because its body was skipped rather than because the
        // body compiles. `language/statements/for-in/let-block-with-newline.js`
        // and `let-identifier-with-newline.js` are exactly that: both bodies
        // read the undeclared identifier `let`, and neither is evidence that
        // `let`-as-identifier lowering works. `S12.6.4_A1/A2` are *not* — they
        // depend on `var` hoisting out of the skipped body, which survives,
        // because `hoist_statement`'s `ForInLoop` arm recurses into
        // `for_in.body()` in a pass that runs before this one.
        let is_nullish_target = matches!(target.kind, ValueKind::Undefined | ValueKind::Null)
            && target.possible_kinds.is_subset_of(
                KindSet::from_kind(ValueKind::Undefined).union(KindSet::from_kind(ValueKind::Null)),
            );
        if !is_dynamic_target && is_nullish_target {
            let head_effects_only = TypedExpr::from_info(
                ValueInfo::undefined(),
                ExprIr::Comma {
                    lhs: Box::new(target),
                    rhs: Box::new(TypedExpr::undefined()),
                },
            );
            return Self::prepend_statement(
                initializer_prefix,
                StatementIr::Expression(head_effects_only),
                ValueKind::Undefined,
            );
        }
        // The same eager head body is consumed by the whole continuation owner.
        // Unknown enumeration traps invalidate source facts before either path.
        self.invalidate_unknown_user_code_effects();
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.push_scope();
        let Some(mut initialization) = self.lower_for_in_iteration_initialization(&head, None)
        else {
            self.pop_scope();
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let enclosing_mixed_domain = self.async_generator_source_domain;
        self.async_generator_source_domain = AsyncGeneratorSourceDomain::ForeignIteratorBody;
        let (mut body, body_kind) = self.lower_loop_body(for_in.body());
        self.async_generator_source_domain = enclosing_mixed_domain;
        if !initialization.is_empty() {
            initialization.push(body);
            body = StatementIr::Block(BlockIr {
                result_kind: body_kind,
                statements: initialization,
                lexical_environment: None,
            });
        }
        self.pop_scope();
        let after_vars = self.var_bindings.clone();
        let after_globals = self.global_properties.clone();
        self.var_bindings = self.merge_var_bindings(&before_vars, &after_vars);
        self.global_properties = self.merge_global_properties(&before_globals, &after_globals);
        let mode = head.mode();
        let name = head.storage_name().to_string();
        let statement = if is_dynamic_target {
            StatementIr::ForInObject {
                mode,
                name,
                target,
                body: Box::new(body),
                lexical_environment,
            }
        } else if is_array_target {
            StatementIr::ForInArray {
                mode,
                name,
                target,
                body: Box::new(body),
                lexical_environment,
            }
        } else if is_string_target {
            StatementIr::ForInString {
                mode,
                name,
                target,
                body: Box::new(body),
                lexical_environment,
            }
        } else {
            StatementIr::ForInObject {
                mode,
                name,
                target,
                body: Box::new(body),
                lexical_environment,
            }
        };
        Self::prepend_statement(initializer_prefix, statement, body_kind)
    }

    pub(super) fn widen_for_in_parameter_target(
        &self,
        target: &mut TypedExpr,
        source: &Expression,
    ) {
        if target.kind == ValueKind::Undefined && self.is_current_param_expr(source) {
            target.kind = ValueKind::Dynamic;
            target.possible_kinds = KindSet::all_runtime_tags();
            target.heap_shape = None;
            target.function_targets.widen_for_possible_replacement();
        }
    }

    pub(super) fn lower_for_in_initializer_prefix(
        &mut self,
        initializer: &IterableLoopInitializer,
    ) -> Option<StatementIr> {
        let IterableLoopInitializer::Var(variable) = initializer else {
            return None;
        };
        variable.init()?;
        let declarator = self.lower_var_declarator(variable)?;
        if self.borrows_direct_eval_variable_environment() {
            declarator.init.map(|value| {
                StatementIr::DeclarationEvaluation(self.environment_identifier(
                    declarator.name,
                    EnvironmentIdentifierOperationIr::Assign {
                        value: Box::new(value),
                    },
                ))
            })
        } else {
            Some(StatementIr::Var(vec![declarator]))
        }
    }

    pub(super) fn prepend_statement(
        prefix: Option<StatementIr>,
        statement: StatementIr,
        kind: ValueKind,
    ) -> (StatementIr, ValueKind) {
        let Some(prefix) = prefix else {
            return (statement, kind);
        };
        (
            StatementIr::Block(BlockIr {
                result_kind: kind,
                statements: vec![prefix, statement],
                lexical_environment: None,
            }),
            kind,
        )
    }
}
