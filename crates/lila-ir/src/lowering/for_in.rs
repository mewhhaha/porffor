use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_for_in_loop(
        &mut self,
        for_in: &boa_ast::statement::iteration::ForInLoop,
    ) -> (StatementIr, ValueKind) {
        // The for-in statements have no resumable form. Re-entering the body
        // from the top restarts enumeration at the first key and skips the
        // suspension, so the loop silently visits one key and stops.
        if self.plain_async_entry_state().is_some()
            && contains(for_in.body(), ContainsSymbol::AwaitExpression)
        {
            self.unsupported("await inside a for-in loop");
            return (StatementIr::Empty, ValueKind::Undefined);
        }
        let initializer_prefix = self.lower_for_in_initializer_prefix(for_in.initializer());
        if self.for_in_known_empty_target(for_in.target()) {
            return Self::prepend_statement(
                initializer_prefix,
                StatementIr::Empty,
                ValueKind::Undefined,
            );
        }
        if self.for_in_global_non_enumerable_guard_only(for_in) {
            return Self::prepend_statement(
                initializer_prefix,
                StatementIr::Empty,
                ValueKind::Undefined,
            );
        }
        if self.for_in_builtin_non_enumerable_assert_only(for_in) {
            return Self::prepend_statement(
                initializer_prefix,
                StatementIr::Empty,
                ValueKind::Undefined,
            );
        }
        if let IterableLoopInitializer::WebCompatCall(call) = for_in.initializer() {
            return Self::prepend_statement(
                initializer_prefix,
                StatementIr::Expression(
                    self.lower_web_compat_loop_assignment_target(call, for_in.target()),
                ),
                ValueKind::Undefined,
            );
        }
        let mut pattern_initializer: Option<(BindingMode, Pattern)> = None;
        let mut assignment_pattern_initializer: Option<Pattern> = None;
        let mut access_initializer: Option<PropertyAccess> = None;
        let (mode, name) =
            if let Some(binding) = self.for_in_initializer_binding(for_in.initializer()) {
                binding
            } else {
                match for_in.initializer() {
                    IterableLoopInitializer::Var(variable) => {
                        let Binding::Pattern(pattern) = variable.binding() else {
                            self.unsupported("for-in initializer");
                            return (StatementIr::Empty, ValueKind::Undefined);
                        };
                        pattern_initializer = Some((BindingMode::Var, pattern.clone()));
                        (BindingMode::Let, self.alloc_temp_binding_name("forin"))
                    }
                    IterableLoopInitializer::Pattern(pattern) => {
                        assignment_pattern_initializer = Some(pattern.clone());
                        (BindingMode::Let, self.alloc_temp_binding_name("forin"))
                    }
                    IterableLoopInitializer::Let(Binding::Pattern(pattern)) => {
                        pattern_initializer = Some((BindingMode::Let, pattern.clone()));
                        (BindingMode::Let, self.alloc_temp_binding_name("forin"))
                    }
                    IterableLoopInitializer::Const(Binding::Pattern(pattern)) => {
                        pattern_initializer = Some((BindingMode::Const, pattern.clone()));
                        (BindingMode::Let, self.alloc_temp_binding_name("forin"))
                    }
                    // `for (obj.key in …)` and `for (this.#field in …)` assign to
                    // a reference the spec re-evaluates every iteration, so the key
                    // lands in a temporary and the body prefix performs the store.
                    IterableLoopInitializer::Access(
                        access @ (PropertyAccess::Simple(_) | PropertyAccess::Private(_)),
                    ) => {
                        access_initializer = Some(access.clone());
                        (
                            BindingMode::Let,
                            self.alloc_temp_binding_name("forin.access"),
                        )
                    }
                    _ => {
                        self.unsupported("for-in initializer");
                        return (StatementIr::Empty, ValueKind::Undefined);
                    }
                }
            };
        let lexical_environment = self.lower_for_in_of_environment(
            for_in as *const boa_ast::statement::iteration::ForInLoop as usize,
        );
        let mut target = match pattern_initializer.as_ref() {
            None if access_initializer.is_none() && assignment_pattern_initializer.is_none() => {
                self.lower_for_head_expression_with_tdz(mode, &name, for_in.target())
            }
            None => self.lower_expression(for_in.target()),
            Some((BindingMode::Var, _)) => self.lower_expression(for_in.target()),
            Some((pattern_mode, pattern)) => {
                self.push_scope();
                let binding = Binding::Pattern(pattern.clone());
                let Some(bound_names) = supported_bound_names(self.interner, &binding) else {
                    self.unsupported("for-in initializer");
                    self.pop_scope();
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                for bound in bound_names {
                    self.declare_binding(
                        bound.source_name.clone(),
                        BindingInfo::tdz_placeholder(
                            *pattern_mode,
                            TdzPlaceholderName::for_source_name(&bound.source_name),
                        ),
                    );
                }
                let target = self.lower_expression(for_in.target());
                self.pop_scope();
                target
            }
        };
        let is_inferred_undefined_param_target =
            target.kind == ValueKind::Undefined && self.is_current_param_expr(for_in.target());
        if is_inferred_undefined_param_target {
            target.kind = ValueKind::Dynamic;
            target.possible_kinds = KindSet::all_runtime_tags();
            target.heap_shape = None;
            target.function_targets.widen_for_possible_replacement();
        }
        let is_dynamic_target = target.kind == ValueKind::Dynamic;
        let is_array_target = target.possible_kinds.contains(ValueKind::Array)
            || target.possible_kinds.contains(ValueKind::Arguments);
        let is_string_target = target.possible_kinds.contains(ValueKind::String)
            || Self::value_info_is_boxed_string(&target.value_info());
        let is_object_target = target.possible_kinds.contains(ValueKind::Object)
            || target.possible_kinds.contains(ValueKind::Function);
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
            // Annex B `for (var x = 1 in null)` still runs its initializer, so
            // this takes the same `prepend_statement` exit the ordinary lowering
            // does rather than dropping the prefix the way the refusal below
            // can afford to.
            return Self::prepend_statement(
                initializer_prefix,
                StatementIr::Expression(head_effects_only),
                ValueKind::Undefined,
            );
        }
        if !is_dynamic_target && !is_array_target && !is_string_target && !is_object_target {
            // What is left is Number/Boolean/Symbol/BigInt: 14.7.5.6 step 3.b
            // sends those through ToObject (7.1.18) and enumerates the wrapper's
            // own enumerable properties, which is a separate and currently
            // uncounted family. Named in the message so the next lane inherits
            // an accurate diagnostic instead of "non-enumerable target", which
            // was never true of any of them.
            self.unsupported("for-in target requiring ToObject (number, boolean, symbol, bigint)");
            return (StatementIr::Empty, ValueKind::Undefined);
        }

        // EnumerateObjectProperties can enter Proxy ownKeys and descriptor
        // traps while discovering the keys, before the first loop head or body
        // evaluation.
        self.invalidate_unknown_user_code_effects();
        let before_vars = self.var_bindings.clone();
        let before_globals = self.global_properties.clone();
        self.push_scope();
        let key_info = ValueInfo::new(ValueKind::String);
        let storage_name = if mode == BindingMode::Var
            || pattern_initializer.is_some()
            || assignment_pattern_initializer.is_some()
            || access_initializer.is_some()
        {
            name.clone()
        } else {
            for_in_loop_binding_storage_name(for_in, &name)
        };
        if mode == BindingMode::Var {
            self.set_binding_value_info(&name, key_info.clone());
            self.declare_binding(
                name.clone(),
                BindingInfo {
                    mode,
                    storage_name: storage_name.clone(),
                    kind: key_info.kind,
                    possible_kinds: key_info.possible_kinds,
                    heap_shape: key_info.heap_shape.clone(),
                    function_targets: key_info.function_targets.clone(),
                    initialization: Initialization::Initialized,
                },
            );
        } else {
            self.declare_binding(
                name.clone(),
                BindingInfo {
                    mode,
                    storage_name: storage_name.clone(),
                    kind: key_info.kind,
                    possible_kinds: key_info.possible_kinds,
                    heap_shape: key_info.heap_shape.clone(),
                    function_targets: key_info.function_targets.clone(),
                    initialization: Initialization::Initialized,
                },
            );
        }
        let mut pattern_prefix = if let Some(access) = access_initializer.as_ref() {
            let value =
                TypedExpr::from_info(key_info.clone(), ExprIr::Identifier(storage_name.clone()));
            let access = access.clone();
            vec![StatementIr::Expression(
                self.lower_property_assign_value(&access, value),
            )]
        } else if let Some(pattern) = assignment_pattern_initializer.as_ref() {
            let value =
                TypedExpr::from_info(key_info.clone(), ExprIr::Identifier(storage_name.clone()));
            let Some(assign) = self.lower_pattern_assign_value(pattern, value) else {
                self.pop_scope();
                return (StatementIr::Empty, ValueKind::Undefined);
            };
            vec![StatementIr::Expression(assign)]
        } else if let Some((pattern_mode, pattern)) = pattern_initializer.as_ref() {
            let init =
                TypedExpr::from_info(key_info.clone(), ExprIr::Identifier(storage_name.clone()));
            if *pattern_mode == BindingMode::Var {
                let Some(prefix) = self.lower_pattern_var_binding_from_value(pattern, init) else {
                    self.pop_scope();
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                prefix
            } else {
                let binding = Binding::Pattern(pattern.clone());
                let storage_names = supported_bound_names(self.interner, &binding)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|bound| {
                        let storage_name =
                            for_in_loop_binding_storage_name(for_in, &bound.source_name);
                        (bound.source_name, storage_name)
                    })
                    .collect();
                let Some(prefix) = self
                    .lower_pattern_lexical_binding_from_value_with_storage_names(
                        *pattern_mode,
                        pattern,
                        init,
                        Some(&storage_names),
                    )
                else {
                    self.pop_scope();
                    return (StatementIr::Empty, ValueKind::Undefined);
                };
                prefix
            }
        } else {
            Vec::new()
        };
        let (mut body, body_kind) = self.lower_loop_body(for_in.body());
        if !pattern_prefix.is_empty() {
            pattern_prefix.push(body);
            body = StatementIr::Block(BlockIr {
                result_kind: body_kind,
                statements: pattern_prefix,
                lexical_environment: None,
            });
        }
        self.pop_scope();
        let after_vars = self.var_bindings.clone();
        let after_globals = self.global_properties.clone();
        self.var_bindings = self.merge_var_bindings(&before_vars, &after_vars);
        self.global_properties = self.merge_global_properties(&before_globals, &after_globals);
        let statement = if is_dynamic_target {
            StatementIr::ForInObject {
                mode,
                name: storage_name,
                target,
                body: Box::new(body),
                lexical_environment,
            }
        } else if is_array_target {
            StatementIr::ForInArray {
                mode,
                name: storage_name,
                target,
                body: Box::new(body),
                lexical_environment,
            }
        } else if is_string_target {
            StatementIr::ForInString {
                mode,
                name: storage_name,
                target,
                body: Box::new(body),
                lexical_environment,
            }
        } else {
            StatementIr::ForInObject {
                mode,
                name: storage_name,
                target,
                body: Box::new(body),
                lexical_environment,
            }
        };
        Self::prepend_statement(initializer_prefix, statement, body_kind)
    }

    fn lower_for_in_initializer_prefix(
        &mut self,
        initializer: &IterableLoopInitializer,
    ) -> Option<StatementIr> {
        let IterableLoopInitializer::Var(variable) = initializer else {
            return None;
        };
        variable.init()?;
        self.lower_var_declarator(variable)
            .map(|declarator| StatementIr::Var(vec![declarator]))
    }

    fn prepend_statement(
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

    fn for_in_initializer_binding(
        &self,
        initializer: &IterableLoopInitializer,
    ) -> Option<(BindingMode, String)> {
        let (mode, identifier) = match initializer {
            IterableLoopInitializer::Identifier(identifier) => (BindingMode::Var, identifier),
            IterableLoopInitializer::Var(variable) => {
                let Binding::Identifier(identifier) = variable.binding() else {
                    return None;
                };
                (BindingMode::Var, identifier)
            }
            IterableLoopInitializer::Let(Binding::Identifier(identifier)) => {
                (BindingMode::Let, identifier)
            }
            IterableLoopInitializer::Const(Binding::Identifier(identifier)) => {
                (BindingMode::Const, identifier)
            }
            _ => return None,
        };
        Some((
            mode,
            self.interner.resolve_expect(identifier.sym()).to_string(),
        ))
    }

    fn for_in_known_empty_target(&self, target: &Expression) -> bool {
        let Expression::Identifier(identifier) = target else {
            return false;
        };
        let name = self.interner.resolve_expect(identifier.sym()).to_string();
        self.identifier_is_builtin_native_error(&name).is_some()
    }

    fn for_in_global_non_enumerable_guard_only(
        &self,
        for_in: &boa_ast::statement::iteration::ForInLoop,
    ) -> bool {
        if !self.for_in_global_target(for_in.target()) {
            return false;
        }
        let Some(loop_name) = self.for_in_initializer_name(for_in.initializer()) else {
            return false;
        };
        let Some(watched_name) =
            self.for_in_non_enumerable_guarded_assignment(for_in.body(), &loop_name)
        else {
            return false;
        };
        self.is_known_non_enumerable_global(&watched_name)
    }

    fn for_in_builtin_non_enumerable_assert_only(
        &self,
        for_in: &boa_ast::statement::iteration::ForInLoop,
    ) -> bool {
        let Some(target_name) = self.for_in_static_builtin_target(for_in.target()) else {
            return false;
        };
        let Some(loop_name) = self.for_in_initializer_name(for_in.initializer()) else {
            return false;
        };
        let Some(watched_name) = self.for_in_not_same_value_guard_name(for_in.body(), &loop_name)
        else {
            return false;
        };
        self.is_known_non_enumerable_builtin_property(&target_name, &watched_name)
    }

    fn for_in_static_builtin_target(&self, target: &Expression) -> Option<String> {
        match target {
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                matches!(name.as_str(), NUMBER_NAME | BOOLEAN_NAME).then_some(name)
            }
            Expression::Parenthesized(parenthesized) => {
                self.for_in_static_builtin_target(parenthesized.expression())
            }
            _ => None,
        }
    }

    fn for_in_initializer_name(&self, initializer: &IterableLoopInitializer) -> Option<String> {
        let identifier = match initializer {
            IterableLoopInitializer::Identifier(identifier) => identifier,
            IterableLoopInitializer::Var(variable) if variable.init().is_none() => {
                let Binding::Identifier(identifier) = variable.binding() else {
                    return None;
                };
                identifier
            }
            IterableLoopInitializer::Let(Binding::Identifier(identifier))
            | IterableLoopInitializer::Const(Binding::Identifier(identifier)) => identifier,
            _ => return None,
        };
        Some(self.interner.resolve_expect(identifier.sym()).to_string())
    }

    fn for_in_non_enumerable_guarded_assignment(
        &self,
        body: &Statement,
        loop_name: &str,
    ) -> Option<String> {
        let Statement::If(if_statement) = self.single_statement(body) else {
            return None;
        };
        if if_statement.else_node().is_some() {
            return None;
        }
        let watched_name = self.for_in_non_enumerable_guard_name(if_statement.cond(), loop_name)?;
        if self.statement_is_simple_false_assignment(if_statement.body()) {
            Some(watched_name)
        } else {
            None
        }
    }

    fn for_in_non_enumerable_guard_name(
        &self,
        condition: &Expression,
        loop_name: &str,
    ) -> Option<String> {
        let condition = Self::unwrap_parenthesized_expr(condition);
        let Expression::Binary(binary) = condition else {
            return None;
        };
        if binary.op() != BinaryOp::Relational(RelationalOp::StrictEqual) {
            return None;
        }
        if self.expr_is_identifier_named(binary.lhs(), loop_name) {
            return self.static_string_expression(binary.rhs());
        }
        if self.expr_is_identifier_named(binary.rhs(), loop_name) {
            return self.static_string_expression(binary.lhs());
        }
        None
    }

    fn for_in_not_same_value_guard_name(
        &self,
        body: &Statement,
        loop_name: &str,
    ) -> Option<String> {
        let Statement::Expression(expression) = self.single_statement(body) else {
            return None;
        };
        let Expression::Call(call) = Self::unwrap_parenthesized_expr(expression) else {
            return None;
        };
        let Expression::PropertyAccess(PropertyAccess::Simple(access)) =
            Self::unwrap_parenthesized_expr(call.function())
        else {
            return None;
        };
        let (Expression::Identifier(target), PropertyAccessField::Const(field)) =
            (access.target(), access.field())
        else {
            return None;
        };
        if self.interner.resolve_expect(target.sym()).to_string() != "assert"
            || self.interner.resolve_expect(field.sym()).to_string() != "notSameValue"
            || call.args().len() < 2
        {
            return None;
        }
        if self.expr_is_identifier_named(&call.args()[0], loop_name) {
            return self.static_string_expression(&call.args()[1]);
        }
        if self.expr_is_identifier_named(&call.args()[1], loop_name) {
            return self.static_string_expression(&call.args()[0]);
        }
        None
    }

    fn statement_is_simple_false_assignment(&self, statement: &Statement) -> bool {
        let Statement::Expression(expression) = self.single_statement(statement) else {
            return false;
        };
        let Expression::Assign(assign) = Self::unwrap_parenthesized_expr(expression) else {
            return false;
        };
        if assign.op() != AssignOp::Assign {
            return false;
        }
        if !matches!(assign.lhs(), AssignTarget::Identifier(_)) {
            return false;
        }
        matches!(
            Self::unwrap_parenthesized_expr(assign.rhs()),
            Expression::Literal(literal) if matches!(literal.kind(), LiteralKind::Bool(false))
        )
    }
}
