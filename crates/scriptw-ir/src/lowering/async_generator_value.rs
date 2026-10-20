use super::generator_identifier_reference::{
    RetainedGeneratorIdentifierReference, RetainedGeneratorIdentifierTarget,
};
use super::*;

impl ScriptLowerer<'_> {
    /// The checked source and exact current function protocol precede lowering
    /// any operand. All values use the original activation-owned cells.
    pub(super) fn lower_staged_async_generator_value(
        &mut self,
        source: AsyncGeneratorExpressionSource<'_>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.async_generator_entry_state()?;
        let expression = source.expression();
        if !contains(expression, ContainsSymbol::AwaitExpression)
            && !contains(expression, ContainsSymbol::YieldExpression)
        {
            return Some((Vec::new(), self.lower_expression(expression)));
        }
        match expression {
            Expression::Parenthesized(source) => {
                self.lower_mixed_generator_value(source.expression())
            }
            Expression::Await(source) => {
                let (mut prefix, value) = self.lower_mixed_generator_value(source.target())?;
                let name = self.alloc_suspension_owned_binding(
                    "async.generator.await.",
                    unknown_runtime_value_info(),
                );
                prefix.push(StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: name.clone(),
                    init: TypedExpr::undefined(),
                });
                let (statement, _) = self.lower_linear_async_await_value(
                    value,
                    AsyncResumeModeIr::AssignIdentifier(name.clone()),
                );
                prefix.push(statement);
                Some((prefix, self.lower_identifier_name(name, false)))
            }
            Expression::Yield(source) => {
                let (mut prefix, value) = match source.target() {
                    Some(source) => self.lower_mixed_generator_value(source)?,
                    None => (Vec::new(), TypedExpr::undefined()),
                };
                let name = self.alloc_suspension_owned_binding(
                    "async.generator.yield.",
                    unknown_runtime_value_info(),
                );
                prefix.push(StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: name.clone(),
                    init: TypedExpr::undefined(),
                });
                let (statement, _) = self.lower_linear_generator_yield_value(
                    value,
                    source.delegate(),
                    GeneratorResumeModeIr::AssignIdentifier(name.clone()),
                );
                prefix.push(statement);
                Some((prefix, self.lower_identifier_name(name, false)))
            }
            Expression::Conditional(_) => self.lower_async_generator_value_branch(source),
            Expression::Binary(binary) if matches!(binary.op(), BinaryOp::Logical(_)) => {
                self.lower_async_generator_value_branch(source)
            }
            Expression::Binary(_) | Expression::Unary(_) | Expression::TemplateLiteral(_) => {
                self.lower_staged_generator_eager_value(expression)
            }
            Expression::Assign(assignment)
                if matches!(assignment.lhs(), AssignTarget::WebCompatCall(_)) =>
            {
                let AssignTarget::WebCompatCall(call) = assignment.lhs() else {
                    unreachable!()
                };
                self.lower_resumable_web_compat_call_target(call)
            }
            Expression::Assign(assignment)
                if matches!(
                    assignment.op(),
                    AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce
                ) =>
            {
                self.lower_async_generator_value_branch(source)
            }
            Expression::Assign(assignment) => match assignment.lhs() {
                AssignTarget::Pattern(pattern) if assignment.op() == AssignOp::Assign => {
                    self.lower_async_generator_pattern_assignment(pattern, assignment.rhs())
                }
                AssignTarget::Identifier(identifier) => {
                    let name = self.interner.resolve_expect(identifier.sym()).to_string();
                    let mut prefix = Vec::new();
                    if assignment.op() == AssignOp::Assign {
                        let reference = RetainedGeneratorIdentifierTarget::capture_write_only(
                            self,
                            &mut prefix,
                            name,
                        );
                        let (rhs, value) = self.lower_mixed_generator_value(assignment.rhs())?;
                        prefix.extend(rhs);
                        Some((prefix, reference.put_value(self, value)))
                    } else {
                        let reference =
                            RetainedGeneratorIdentifierReference::capture(self, &mut prefix, name);
                        let (rhs, value) = self.lower_mixed_generator_value(assignment.rhs())?;
                        prefix.extend(rhs);
                        let old = reference.old_value();
                        let value =
                            self.combine_compound_assignment_values(assignment.op(), old, value)?;
                        Some((prefix, reference.put_value(self, value)))
                    }
                }
                AssignTarget::Access(PropertyAccess::Simple(access))
                    if assignment.op() == AssignOp::Assign =>
                {
                    self.lower_staged_generator_property_assignment(access, assignment.rhs())
                }
                AssignTarget::Access(_) => self.lower_resumable_property_assignment(assignment),
                AssignTarget::Pattern(_) | AssignTarget::WebCompatCall(_) => None,
            },
            Expression::Call(_) | Expression::New(_) | Expression::TaggedTemplate(_) => {
                self.lower_staged_async_generator_invocation(source)
            }
            Expression::PropertyAccess(PropertyAccess::Simple(access)) => {
                let (prefix, _, value) = self.lower_staged_generator_property(access)?;
                Some((prefix, value))
            }
            Expression::PropertyAccess(PropertyAccess::Super(_))
            | Expression::BinaryInPrivate(_) => {
                self.lower_staged_generator_special_read(expression)
            }
            Expression::PropertyAccess(PropertyAccess::Private(access)) => {
                self.lower_resumable_private_read(access)
            }
            Expression::ArrayLiteral(array) => self.lower_staged_generator_array_literal(array),
            Expression::ObjectLiteral(object) => self.lower_staged_generator_object_literal(object),
            Expression::ClassExpression(class) => self.lower_staged_class_expression(class),
            Expression::Optional(optional) => self
                .lower_generator_optional_chain(GeneratorOptionalChainSource::new_mixed(optional)?),
            Expression::Update(update) => self.lower_resumable_update(update),
            Expression::ImportCall(call) => self.lower_resumable_import_call(call),
            // These original owners remain consumed for a single source
            // protocol. Their opposite-protocol grammar is never widened.
            _ if !contains(expression, ContainsSymbol::YieldExpression) => {
                self.lower_async_prefixed_expression(expression)
            }
            _ if !contains(expression, ContainsSymbol::AwaitExpression) => {
                self.lower_staged_generator_expression_legacy(expression)
            }
            _ => None,
        }
    }

    pub(super) fn lower_mixed_generator_value(
        &mut self,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_staged_async_generator_value(AsyncGeneratorExpressionSource::new(source)?)
    }

    pub(super) fn combine_compound_assignment_values(
        &mut self,
        operation: AssignOp,
        lhs: TypedExpr,
        rhs: TypedExpr,
    ) -> Option<TypedExpr> {
        let value = match operation {
            AssignOp::Add => self.combine_arithmetic(ArithmeticOp::Add, lhs, rhs),
            AssignOp::Sub => self.combine_arithmetic(ArithmeticOp::Sub, lhs, rhs),
            AssignOp::Mul => self.combine_arithmetic(ArithmeticOp::Mul, lhs, rhs),
            AssignOp::Div => self.combine_arithmetic(ArithmeticOp::Div, lhs, rhs),
            AssignOp::Mod => self.combine_arithmetic(ArithmeticOp::Mod, lhs, rhs),
            AssignOp::Exp => self.combine_arithmetic(ArithmeticOp::Exp, lhs, rhs),
            AssignOp::And => self.combine_bitwise(BitwiseBinaryOp::And, lhs, rhs),
            AssignOp::Or => self.combine_bitwise(BitwiseBinaryOp::Or, lhs, rhs),
            AssignOp::Xor => self.combine_bitwise(BitwiseBinaryOp::Xor, lhs, rhs),
            AssignOp::Shl => self.combine_bitwise(BitwiseBinaryOp::Shl, lhs, rhs),
            AssignOp::Shr => self.combine_bitwise(BitwiseBinaryOp::Shr, lhs, rhs),
            AssignOp::Ushr => self.combine_bitwise(BitwiseBinaryOp::UShr, lhs, rhs),
            AssignOp::Assign | AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce => {
                return None
            }
        };
        Some(value)
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_async_generator_return_expression(
        &mut self,
        expression: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        if self.async_generator_entry_state().is_some() {
            return self.lower_mixed_generator_value(expression);
        }
        match expression {
            Expression::Parenthesized(parenthesized) => {
                self.lower_staged_async_generator_return_expression(parenthesized.expression())
            }
            Expression::Await(await_expression) => {
                let (mut statements, value) =
                    self.lower_staged_async_generator_return_expression(await_expression.target())?;
                let result_name = self.alloc_suspension_owned_binding(
                    "async.generator.return.await.",
                    ValueInfo {
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::unknown(),
                    },
                );
                statements.push(StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: result_name.clone(),
                    init: TypedExpr::undefined(),
                });
                let (await_statement, _) = self.lower_linear_async_await_value(
                    value,
                    AsyncResumeModeIr::AssignIdentifier(result_name.clone()),
                );
                statements.push(await_statement);
                Some((statements, self.lower_identifier_name(result_name, false)))
            }
            Expression::Yield(yield_expression) => {
                let (mut statements, value) = match yield_expression.target() {
                    Some(target) => self.lower_staged_async_generator_return_expression(target)?,
                    None => (Vec::new(), TypedExpr::undefined()),
                };
                let result_name = self.alloc_suspension_owned_binding(
                    "async.generator.return.yield.",
                    ValueInfo {
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: FunctionTargetKnowledge::unknown(),
                    },
                );
                statements.push(StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: result_name.clone(),
                    init: TypedExpr::undefined(),
                });
                let (yield_statement, _) = self.lower_linear_generator_yield_value(
                    value,
                    yield_expression.delegate(),
                    GeneratorResumeModeIr::AssignIdentifier(result_name.clone()),
                );
                statements.push(yield_statement);
                Some((statements, self.lower_identifier_name(result_name, false)))
            }
            // A yield-only composite uses the same activation-owned operands
            // as synchronous generators. The enclosing return still awaits its
            // final value through the async-generator return continuation.
            _ if contains(expression, ContainsSymbol::YieldExpression)
                && !contains(expression, ContainsSymbol::AwaitExpression) =>
            {
                self.lower_staged_generator_expression(expression)
            }
            _ if !contains(expression, ContainsSymbol::YieldExpression) => self
                .lower_async_prefixed_expression(expression)
                .or_else(|| {
                    (!contains(expression, ContainsSymbol::AwaitExpression))
                        .then(|| (Vec::new(), self.lower_expression(expression)))
                }),
            _ => None,
        }
    }
}
