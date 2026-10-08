use super::async_generator_assignment::RetainedAssignmentReference;
use super::*;
use crate::async_generator_loop_control::AsyncGeneratorControlError;

impl ScriptLowerer<'_> {
    pub(super) fn lower_async_generator_if(
        &mut self,
        source: AsyncGeneratorIfSource<'_>,
    ) -> (StatementIr, ValueKind) {
        let Some(states) = self
            .async_generator_entry_state()
            .and_then(|entry| source.states(entry))
        else {
            self.unsupported("async-generator If requires complete mixed source ranges");
            return (StatementIr::Empty, ValueKind::Undefined);
        };
        let ast = source.source();
        let result = (|| -> Result<_, AsyncGeneratorControlError> {
            let condition = self.lower_async_generator_expression_region(
                Some(ast.cond()),
                states.condition(),
                true,
            )?;
            let before = self.capture_conditional_flow_facts();
            let then_branch =
                self.lower_async_generator_body_region(ast.body(), states.then_branch())?;
            let then_kind = then_branch.block().result_kind;
            let then_facts = self.capture_conditional_flow_facts();
            self.install_conditional_flow_facts(before);
            let else_branch = if let Some(source) = ast.else_node() {
                self.lower_async_generator_body_region(source, states.else_branch())?
            } else {
                self.set_async_generator_phase(states.else_branch().entry());
                self.finish_async_generator_region(
                    BlockIr {
                        statements: Vec::new(),
                        result_kind: ValueKind::Undefined,
                        lexical_environment: None,
                    },
                    states.else_branch(),
                )?
            };
            let else_facts = self.capture_conditional_flow_facts();
            self.merge_conditional_flow_facts(then_facts, else_facts);
            let kind = self.merge_value_kinds(then_kind, else_branch.block().result_kind);
            let plan = AsyncGeneratorIfIr::new(states, condition, then_branch, else_branch)?;
            self.set_async_generator_phase(plan.exit_state());
            Ok((StatementIr::AsyncGeneratorIf(Box::new(plan)), kind))
        })();
        match result {
            Ok(result) => result,
            Err(error) => {
                self.unsupported_with_message(format!(
                    "unsupported in lila wasm-aot: invalid mixed async-generator If: {error:?}"
                ));
                (StatementIr::Empty, ValueKind::Undefined)
            }
        }
    }

    /// Conditional values and short-circuit assignment share the actual full
    /// mixed If carrier. The unselected arm never evaluates its source.
    pub(super) fn lower_async_generator_value_branch(
        &mut self,
        source: AsyncGeneratorExpressionSource<'_>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let states = source.branch_states(self.async_generator_entry_state()?)?;
        self.set_async_generator_phase(states.condition().entry());
        let mut reference = None;
        let (mut prefix, condition, then_source, else_source, initial) = match source.expression() {
            Expression::Conditional(source) => {
                let (prefix, condition) = self.lower_mixed_generator_value(source.condition())?;
                (
                    prefix,
                    condition,
                    Some(source.if_true()),
                    Some(source.if_false()),
                    TypedExpr::undefined(),
                )
            }
            Expression::Binary(source) => {
                let (mut prefix, lhs) = self.lower_mixed_generator_value(source.lhs())?;
                let mut info = lhs.value_info();
                info.heap_shape = None;
                let name =
                    self.alloc_suspension_owned_binding("async.generator.logical.lhs.", info);
                prefix.push(StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: name.clone(),
                    init: lhs,
                });
                let value = self.lower_identifier_name(name, false);
                match source.op() {
                    BinaryOp::Logical(LogicalOp::And) => {
                        (prefix, value.clone(), Some(source.rhs()), None, value)
                    }
                    BinaryOp::Logical(LogicalOp::Or) => {
                        (prefix, value.clone(), None, Some(source.rhs()), value)
                    }
                    BinaryOp::Logical(LogicalOp::Coalesce) => (
                        prefix,
                        generator_value_branch::nullish_condition(value.clone()),
                        Some(source.rhs()),
                        None,
                        value,
                    ),
                    BinaryOp::Arithmetic(_)
                    | BinaryOp::Relational(_)
                    | BinaryOp::Bitwise(_)
                    | BinaryOp::Comma => return None,
                }
            }
            Expression::Assign(source) => {
                let mut prefix = Vec::new();
                let (captured, value) =
                    self.capture_resumable_assignment_reference(source.lhs(), &mut prefix)?;
                reference = Some(captured);
                match source.op() {
                    AssignOp::BoolAnd => (prefix, value.clone(), Some(source.rhs()), None, value),
                    AssignOp::BoolOr => (prefix, value.clone(), None, Some(source.rhs()), value),
                    AssignOp::Coalesce => (
                        prefix,
                        generator_value_branch::nullish_condition(value.clone()),
                        Some(source.rhs()),
                        None,
                        value,
                    ),
                    _ => return None,
                }
            }
            _ => return None,
        };
        let result_name = self.alloc_suspension_owned_binding(
            "async.generator.branch.value.",
            unknown_runtime_value_info(),
        );
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: result_name.clone(),
            init: initial,
        });
        let condition = AsyncGeneratorLoopExpressionIr::new(
            self.finish_async_generator_region(
                BlockIr {
                    statements: prefix,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.condition(),
            )
            .ok()?,
            condition,
        );
        let skipped = reference
            .as_ref()
            .and_then(RetainedAssignmentReference::release_operation);
        let before = self.capture_conditional_flow_facts();
        self.set_async_generator_phase(states.then_branch().entry());
        let then_statements = self.lower_mixed_value_arm(
            then_source,
            &result_name,
            &mut reference,
            skipped.as_ref(),
        )?;
        let then_branch = self
            .finish_async_generator_region(
                BlockIr {
                    statements: then_statements,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.then_branch(),
            )
            .ok()?;
        let then_facts = self.capture_conditional_flow_facts();
        self.install_conditional_flow_facts(before);
        self.set_async_generator_phase(states.else_branch().entry());
        let else_statements = self.lower_mixed_value_arm(
            else_source,
            &result_name,
            &mut reference,
            skipped.as_ref(),
        )?;
        let else_branch = self
            .finish_async_generator_region(
                BlockIr {
                    statements: else_statements,
                    result_kind: ValueKind::Undefined,
                    lexical_environment: None,
                },
                states.else_branch(),
            )
            .ok()?;
        let else_facts = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(then_facts, else_facts);
        let branch = AsyncGeneratorIfIr::new(states, condition, then_branch, else_branch).ok()?;
        self.set_async_generator_phase(branch.exit_state());
        self.set_binding_value_info(&result_name, unknown_runtime_value_info());
        Some((
            vec![StatementIr::AsyncGeneratorIf(Box::new(branch))],
            TypedExpr::from_info(
                unknown_runtime_value_info(),
                ExprIr::Identifier(result_name),
            ),
        ))
    }

    pub(super) fn lower_mixed_value_arm(
        &mut self,
        source: Option<&Expression>,
        result_name: &str,
        reference: &mut Option<RetainedAssignmentReference>,
        skipped: Option<&(String, EnvironmentIdentifierOperationIr)>,
    ) -> Option<Vec<StatementIr>> {
        // Each selected arm can allocate a different set of private operands.
        // Publish the terminal value before leaving their lowering scope.
        self.push_scope();
        let lowered = (|| match source {
            Some(source) => {
                let (mut prefix, value) = self.lower_mixed_generator_value(source)?;
                let value = match reference.take() {
                    Some(reference) => reference.put_value(self, value),
                    None => value,
                };
                prefix.push(StatementIr::Expression(
                    self.lower_identifier_assign_value(result_name.to_string(), value),
                ));
                Some(prefix)
            }
            None => Some(
                skipped
                    .map(|(name, operation)| {
                        vec![StatementIr::Expression(
                            self.environment_identifier(name.clone(), operation.clone()),
                        )]
                    })
                    .unwrap_or_default(),
            ),
        })();
        self.pop_scope();
        lowered
    }
}
