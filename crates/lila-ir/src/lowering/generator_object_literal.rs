//! Incremental object construction through the ordinary property semantics.

use super::resumable_operand::ResumableOperandProtocol;
use super::*;

fn retain_operand(
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
    pub(super) fn lower_staged_generator_object_literal(
        &mut self,
        object: &ObjectLiteral,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_staged_object_literal(object, ResumableOperandProtocol::current(self)?)
    }

    fn lower_staged_object_literal(
        &mut self,
        object: &ObjectLiteral,
        protocol: ResumableOperandProtocol,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        if self.object_literal_has_duplicate_proto_setter(object) {
            self.diagnostics.push(IrDiagnostic::rejected(
                EarlyErrorCode::ObjectDuplicateProto,
                "early error: duplicate __proto__ prototype setter in object literal",
                None,
            ));
            return None;
        }
        // Its properties and prototype change during construction and caller
        // code can run between phases. Retain the object, without a shape proof.
        let object_info = ValueInfo::new(ValueKind::Object);
        let object_name =
            self.alloc_suspension_owned_binding("generator.object.literal.", object_info.clone());
        let mut prefix = vec![StatementIr::Lexical {
            mode: BindingMode::Let,
            name: object_name.clone(),
            init: TypedExpr::from_info(object_info, ExprIr::ObjectLiteral(Vec::new())),
        }];

        for property in object.properties() {
            let (key_source, value_source) = generator_object_property_operands(property)?;
            let mut operands = Vec::with_capacity(2);
            let mut normalized_key = None;
            let mut prepared_class = None;
            if let Some(source) = key_source {
                let (key_prefix, key) = protocol.lower(self, source)?;
                prefix.extend(key_prefix);
                self.record_possible_to_primitive_effects(&key.value_info());
                // Evaluation of PropertyName includes ToPropertyKey. Its effects
                // and any abrupt completion precede this property's value.
                let key = retain_operand(
                    self,
                    &mut prefix,
                    TypedExpr::spec_to_property_key(key),
                    "generator.object.key.",
                );
                normalized_key = Some(key.clone());
                operands.push((source, self.lower_identifier_name(key, false)));
            }
            if let Some(source) = value_source {
                match (
                    normalized_key.as_ref(),
                    source.is_anonymous_function_definition(),
                    Self::unwrap_parenthesized_expr(source),
                ) {
                    (Some(key_binding), true, Expression::ClassExpression(_)) => {
                        prepared_class = Some(self.prepare_computed_object_class_property(
                            property,
                            key_binding,
                            &mut prefix,
                        )?);
                    }
                    _ => {
                        let (value_prefix, value) = protocol.lower(self, source)?;
                        prefix.extend(value_prefix);
                        let value =
                            retain_operand(self, &mut prefix, value, "generator.object.value.");
                        operands.push((source, self.lower_identifier_name(value, false)));
                    }
                }
            }

            // Substitute completed raw operands into the one ordinary semantic
            // owner. Original AST nodes preserve method/function identities.
            let mut restored = Vec::with_capacity(operands.len());
            for (source, value) in operands {
                let key = std::ptr::from_ref(source) as usize;
                restored.push((key, self.pinned_async_operands.insert(key, value)));
            }
            let lowered = match prepared_class {
                Some(prepared) => self.lower_prepared_object_class_property(prepared),
                None => self.lower_object_literal_property(property),
            };
            for (key, previous) in restored.into_iter().rev() {
                match previous {
                    Some(value) => {
                        self.pinned_async_operands.insert(key, value);
                    }
                    None => {
                        self.pinned_async_operands.remove(&key);
                    }
                }
            }
            let target = self.lower_identifier_name(object_name.clone(), false);
            let definition = ObjectPropertyDefinitionIr::new(target, lowered?).ok()?;
            prefix.push(StatementIr::Expression(TypedExpr::from_info(
                ValueInfo::new(ValueKind::Object),
                ExprIr::ObjectPropertyDefinition(Box::new(definition)),
            )));
        }
        Some((prefix, self.lower_identifier_name(object_name, false)))
    }
}
