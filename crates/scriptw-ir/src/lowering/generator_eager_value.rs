use super::resumable_operand::ResumableOperandProtocol;
use super::*;

mod reference_operands;

/// A completed operand survives later source evaluation without retaining a
/// mutable heap-shape proof across caller mutation or resumed user code.
fn retain_value(
    lowerer: &mut ScriptLowerer<'_>,
    prefix: &mut Vec<StatementIr>,
    value: TypedExpr,
    hint: &str,
) -> String {
    let mut info = value.value_info();
    info.heap_shape = None;
    let name = lowerer.alloc_suspension_owned_binding(hint, info);
    prefix.push(StatementIr::Lexical {
        mode: BindingMode::Let,
        name: name.clone(),
        init: value,
    });
    name
}

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_generator_eager_value(
        &mut self,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        match source {
            Expression::Binary(binary) => {
                let (mut prefix, lhs) = self.lower_staged_generator_expression(binary.lhs())?;
                let lhs = retain_value(self, &mut prefix, lhs, "generator.binary.lhs.");
                let (rhs_prefix, rhs) = self.lower_staged_generator_expression(binary.rhs())?;
                prefix.extend(rhs_prefix);
                let rhs = retain_value(self, &mut prefix, rhs, "generator.binary.rhs.");
                let lhs = self.lower_identifier_name(lhs, false);
                let rhs = self.lower_identifier_name(rhs, false);
                let value = match binary.op() {
                    BinaryOp::Arithmetic(op) => self.combine_arithmetic(op, lhs, rhs),
                    BinaryOp::Relational(op) => self.combine_relational(op, lhs, rhs),
                    BinaryOp::Bitwise(op) => {
                        let op = match op {
                            BitwiseOp::And => BitwiseBinaryOp::And,
                            BitwiseOp::Or => BitwiseBinaryOp::Or,
                            BitwiseOp::Xor => BitwiseBinaryOp::Xor,
                            BitwiseOp::Shl => BitwiseBinaryOp::Shl,
                            BitwiseOp::Shr => BitwiseBinaryOp::Shr,
                            BitwiseOp::UShr => BitwiseBinaryOp::UShr,
                        };
                        self.combine_bitwise(op, lhs, rhs)
                    }
                    BinaryOp::Comma => TypedExpr::from_info(
                        rhs.value_info(),
                        ExprIr::Comma {
                            lhs: Box::new(lhs),
                            rhs: Box::new(rhs),
                        },
                    ),
                    BinaryOp::Logical(_) => return None,
                };
                Some((prefix, value))
            }
            Expression::Unary(unary) if unary.op() == UnaryOp::Delete => {
                self.lower_generator_delete_reference(unary.target())
            }
            Expression::Unary(unary) => {
                let (prefix, operand) = self.lower_staged_generator_expression(unary.target())?;
                Some((prefix, self.combine_unary_value(unary.op(), operand)))
            }
            Expression::TemplateLiteral(template) => {
                let accumulator = self.alloc_suspension_owned_binding(
                    "generator.template.value.",
                    ValueInfo::new(ValueKind::String),
                );
                let mut prefix = vec![StatementIr::Lexical {
                    mode: BindingMode::Let,
                    name: accumulator.clone(),
                    init: TypedExpr::from_info(
                        ValueInfo::new(ValueKind::String),
                        ExprIr::String(String::new()),
                    ),
                }];
                for element in template.elements() {
                    let value = match element {
                        TemplateElement::String(text) => TypedExpr::from_info(
                            ValueInfo::new(ValueKind::String),
                            ExprIr::String(self.interner.resolve_expect(*text).join(
                                |text| text.to_string(),
                                Self::utf16_units_to_runtime_string,
                                true,
                            )),
                        ),
                        TemplateElement::Expr(source) => {
                            let (part_prefix, value) =
                                self.lower_staged_generator_expression(source)?;
                            prefix.extend(part_prefix);
                            self.record_possible_to_primitive_effects(&value.value_info());
                            TypedExpr::spec_to_string(value)
                        }
                    };
                    let lhs = self.lower_identifier_name(accumulator.clone(), false);
                    let appended = TypedExpr::from_info(
                        ValueInfo::new(ValueKind::String),
                        ExprIr::StringConcat {
                            lhs: Box::new(lhs),
                            rhs: Box::new(value),
                        },
                    );
                    // Template substitution ToString completes now, before any
                    // later substitution can suspend or observe its effects.
                    prefix.push(StatementIr::Expression(
                        self.lower_identifier_assign_value(accumulator.clone(), appended),
                    ));
                }
                Some((prefix, self.lower_identifier_name(accumulator, false)))
            }
            _ => None,
        }
    }
}
