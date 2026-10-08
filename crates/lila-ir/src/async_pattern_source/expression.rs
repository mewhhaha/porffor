//! Actual expression traversal owns async branch ranges, never a flat Await count.

use super::*;

pub(crate) fn append(
    source: &Expression,
    cursor: &mut u32,
    awaits: &mut Vec<(u32, u32)>,
) -> Option<()> {
    if contains(source, ContainsSymbol::YieldExpression)
        || await_requires_branch_owner(source, AwaitBranchOwner::PlainAsyncBranch)
    {
        return None;
    }
    let mut plan = ExpressionStates {
        cursor: *cursor,
        awaits,
    };
    plan.visit_expression(source).is_continue().then_some(())?;
    *cursor = plan.cursor;
    Some(())
}

struct ExpressionStates<'a> {
    cursor: u32,
    awaits: &'a mut Vec<(u32, u32)>,
}

impl ExpressionStates<'_> {
    fn next(&mut self) -> ControlFlow<()> {
        let Some(next) = self.cursor.checked_add(1) else {
            return ControlFlow::Break(());
        };
        self.cursor = next;
        ControlFlow::Continue(())
    }
    fn branches(
        &mut self,
        then_source: Option<&Expression>,
        else_source: Option<&Expression>,
    ) -> ControlFlow<()> {
        self.next()?;
        if let Some(source) = then_source {
            self.visit_expression(source)?;
        }
        self.next()?;
        if let Some(source) = else_source {
            self.visit_expression(source)?;
        }
        self.next()
    }
    fn optional_tail(
        &mut self,
        links: &[boa_ast::expression::OptionalOperation],
    ) -> ControlFlow<()> {
        let Some((first, rest)) = links.split_first() else {
            return ControlFlow::Continue(());
        };
        // The shared tail keeps eager calls guarded with an ordinary If. Only
        // a remaining Await publishes its disjoint continuation ranges.
        let owns_branch = links
            .iter()
            .any(|link| contains(link, ContainsSymbol::AwaitExpression));
        if first.shorted() && owns_branch {
            self.next()?; // skipped then arm
            self.next()?; // selected else arm
        }
        match first.kind() {
            OptionalOperationKind::SimplePropertyAccess { field } => {
                if let PropertyAccessField::Expr(source) = field {
                    self.visit_expression(source)?;
                }
            }
            OptionalOperationKind::Call { args } => {
                for source in args.iter() {
                    self.visit_expression(source)?;
                }
            }
            OptionalOperationKind::PrivatePropertyAccess { .. } => {}
        }
        self.optional_tail(rest)?;
        if first.shorted() && owns_branch {
            self.next()?;
        }
        ControlFlow::Continue(())
    }
}

impl<'ast> Visitor<'ast> for ExpressionStates<'_> {
    type BreakTy = ();
    fn visit_expression(&mut self, source: &'ast Expression) -> ControlFlow<()> {
        if !contains(source, ContainsSymbol::AwaitExpression) {
            return ControlFlow::Continue(());
        }
        match source {
            Expression::Await(source) => {
                self.visit_expression(source.target())?;
                let suspend = self.cursor;
                self.next()?;
                self.awaits.push((suspend, self.cursor));
                ControlFlow::Continue(())
            }
            Expression::Conditional(source) => {
                self.visit_expression(source.condition())?;
                if contains(source.if_true(), ContainsSymbol::AwaitExpression)
                    || contains(source.if_false(), ContainsSymbol::AwaitExpression)
                {
                    self.branches(Some(source.if_true()), Some(source.if_false()))
                } else {
                    ControlFlow::Continue(())
                }
            }
            Expression::Binary(source) => {
                self.visit_expression(source.lhs())?;
                if let BinaryOp::Logical(op) = source.op() {
                    if contains(source.rhs(), ContainsSymbol::AwaitExpression) {
                        return match op {
                            LogicalOp::And | LogicalOp::Coalesce => {
                                self.branches(Some(source.rhs()), None)
                            }
                            LogicalOp::Or => self.branches(None, Some(source.rhs())),
                        };
                    }
                }
                self.visit_expression(source.rhs())
            }
            Expression::Assign(source)
                if matches!(
                    source.op(),
                    AssignOp::BoolAnd | AssignOp::BoolOr | AssignOp::Coalesce
                ) =>
            {
                source.lhs().visit_with(self)?;
                if contains(source.rhs(), ContainsSymbol::AwaitExpression) {
                    if source.op() == AssignOp::BoolOr {
                        self.branches(None, Some(source.rhs()))
                    } else {
                        self.branches(Some(source.rhs()), None)
                    }
                } else {
                    ControlFlow::Continue(())
                }
            }
            Expression::Assign(source) if source.op() == AssignOp::Assign => {
                if let AssignTarget::Pattern(pattern) = source.lhs() {
                    if contains(pattern, ContainsSymbol::AwaitExpression) {
                        self.visit_expression(source.rhs())?;
                        let Some(source) = AsyncPatternSource::new(pattern) else {
                            return ControlFlow::Break(());
                        };
                        if source.append(&mut self.cursor, self.awaits).is_none() {
                            return ControlFlow::Break(());
                        }
                        return ControlFlow::Continue(());
                    }
                }
                source.visit_with(self)
            }
            Expression::Optional(source) => {
                self.visit_expression(source.target())?;
                if source
                    .chain()
                    .iter()
                    .any(|link| contains(link, ContainsSymbol::AwaitExpression))
                {
                    self.optional_tail(source.chain())
                } else {
                    ControlFlow::Continue(())
                }
            }
            Expression::ClassExpression(source) => {
                for operand in class_evaluation_expressions(source.super_ref(), source.elements()) {
                    self.visit_expression(operand)?;
                }
                ControlFlow::Continue(())
            }
            Expression::Yield(_) => ControlFlow::Break(()),
            _ => source.visit_with(self),
        }
    }

    fn visit_function_body(&mut self, _: &'ast FunctionBody) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn visit_formal_parameter_list(&mut self, _: &'ast FormalParameterList) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}
