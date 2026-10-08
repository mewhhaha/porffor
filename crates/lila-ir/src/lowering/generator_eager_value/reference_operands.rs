use super::*;

impl ScriptLowerer<'_> {
    pub(in crate::lowering) fn lower_resumable_import_call(
        &mut self,
        call: &boa_ast::expression::ImportCall,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let protocol = ResumableOperandProtocol::current(self)?;
        let (mut prefix, specifier) = protocol.lower(self, call.argument())?;
        let name = retain_value(self, &mut prefix, specifier, "generator.import.specifier.");
        let specifier = self.lower_identifier_name(name, false);
        let options = if let Some(source) = call.options() {
            let (statements, value) = protocol.lower(self, source)?;
            prefix.extend(statements);
            let name = retain_value(self, &mut prefix, value, "generator.import.options.");
            Some(self.lower_identifier_name(name, false))
        } else {
            None
        };
        let value = match modules::lower_import_call(call, specifier, options, None) {
            Ok(value) => value,
            Err(error) => {
                self.unsupported_with_message(error);
                TypedExpr::undefined()
            }
        };
        Some((prefix, value))
    }

    pub(in crate::lowering) fn lower_resumable_update(
        &mut self,
        update: &boa_ast::expression::operator::Update,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let protocol = ResumableOperandProtocol::current(self)?;
        let mut operands = Vec::new();
        let mut prefix = Vec::new();
        let mut super_receiver = None;
        match update.target() {
            UpdateTarget::Identifier(_) => {
                return Some((Vec::new(), self.lower_update(update.op(), update.target())))
            }
            UpdateTarget::WebCompatCall(call) => {
                return self.lower_resumable_web_compat_call_target(call)
            }
            UpdateTarget::PropertyAccess(PropertyAccess::Simple(access)) => {
                operands.push(access.target());
                if let PropertyAccessField::Expr(key) = access.field() {
                    operands.push(key.as_ref());
                }
            }
            UpdateTarget::PropertyAccess(PropertyAccess::Private(access)) => {
                operands.push(access.target())
            }
            UpdateTarget::PropertyAccess(PropertyAccess::Super(access)) => {
                let receiver = self.lower_super_property_receiver()?;
                let name = retain_value(self, &mut prefix, receiver, "generator.super.receiver.");
                super_receiver = Some(self.lower_identifier_name(name, false));
                if let PropertyAccessField::Expr(key) = access.field() {
                    operands.push(key.as_ref());
                }
            }
        }
        let mut completed = Vec::with_capacity(operands.len());
        for operand in operands {
            let (statements, value) = protocol.lower(self, operand)?;
            prefix.extend(statements);
            let name = retain_value(self, &mut prefix, value, "generator.update.operand.");
            let value = self.lower_identifier_name(name, false);
            completed.push((std::ptr::from_ref(operand) as usize, value));
        }
        let mut restored = Vec::with_capacity(completed.len());
        for (key, value) in completed {
            restored.push((key, self.pinned_async_operands.insert(key, value)));
        }
        let value = match (update.target(), super_receiver) {
            (UpdateTarget::PropertyAccess(PropertyAccess::Super(access)), Some(receiver)) => {
                self.record_caller_flow_invalidation();
                self.lower_super_property_reference_parts(access)
                    .map(|(key, _, info)| {
                        self.lower_super_property_numeric_update_from_parts(
                            update.op(),
                            key,
                            Box::new(receiver),
                            info,
                        )
                    })
            }
            _ => Some(self.lower_update(update.op(), update.target())),
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
        Some((prefix, value?))
    }

    pub(in crate::lowering) fn lower_staged_generator_special_read(
        &mut self,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let protocol = ResumableOperandProtocol::current(self)?;
        let operand = match source {
            Expression::PropertyAccess(PropertyAccess::Super(access)) => match access.field() {
                PropertyAccessField::Expr(source) => source.as_ref(),
                PropertyAccessField::Const(_) => {
                    return Some((Vec::new(), self.lower_super_property_access(access)))
                }
            },
            Expression::BinaryInPrivate(source) => source.rhs(),
            _ => return None,
        };
        let mut prefix = Vec::new();
        let receiver = if matches!(source, Expression::PropertyAccess(PropertyAccess::Super(_))) {
            let receiver = self.lower_super_property_receiver()?;
            let name = retain_value(self, &mut prefix, receiver, "generator.super.receiver.");
            Some(self.lower_identifier_name(name, false))
        } else {
            None
        };
        let (statements, value) = protocol.lower(self, operand)?;
        prefix.extend(statements);
        let name = retain_value(self, &mut prefix, value, "generator.reference.operand.");
        let key = std::ptr::from_ref(operand) as usize;
        let completed = self.lower_identifier_name(name, false);
        let previous = self.pinned_async_operands.insert(key, completed);
        let mut value = match source {
            Expression::PropertyAccess(PropertyAccess::Super(access)) => {
                self.lower_super_property_access(access)
            }
            Expression::BinaryInPrivate(source) => self.lower_private_in(source),
            _ => unreachable!("the original special Reference read selected its operand"),
        };
        match previous {
            Some(value) => {
                self.pinned_async_operands.insert(key, value);
            }
            None => {
                self.pinned_async_operands.remove(&key);
            }
        }
        if let Some(held) = receiver {
            if let ExprIr::SuperPropertyRead { receiver, .. } = &mut value.expr {
                *receiver = Box::new(held);
            }
        }
        Some((prefix, value))
    }

    pub(in crate::lowering) fn lower_generator_delete_reference(
        &mut self,
        source: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let protocol = ResumableOperandProtocol::current(self)?;
        let checked = match protocol {
            ResumableOperandProtocol::Generator => CheckedGeneratorDeleteSource::new(source)?,
            ResumableOperandProtocol::Mixed => CheckedGeneratorDeleteSource::new_mixed(source)?,
            ResumableOperandProtocol::Async => CheckedGeneratorDeleteSource::new_async(source)?,
        };
        match checked.into_operand() {
            GeneratorDeleteOperand::Super(access) => {
                let mut prefix = Vec::new();
                let receiver = self.lower_super_property_receiver()?;
                let name = retain_value(
                    self,
                    &mut prefix,
                    receiver,
                    "generator.delete.super.receiver.",
                );
                let receiver = self.lower_identifier_name(name, false);
                let key = match access.field() {
                    PropertyAccessField::Const(name) => PropertyKeyIr::StaticString(
                        self.interner.resolve_expect(name.sym()).to_string(),
                    ),
                    PropertyAccessField::Expr(source) => {
                        let (statements, value) = protocol.lower(self, source)?;
                        prefix.extend(statements);
                        let name =
                            retain_value(self, &mut prefix, value, "generator.delete.super.key.");
                        PropertyKeyIr::StringExpr(Box::new(self.lower_identifier_name(name, false)))
                    }
                };
                Some((
                    prefix,
                    DeleteSuperReferencePlan::from_evaluated_this(receiver, key)
                        .into_reference_error(),
                ))
            }
            GeneratorDeleteOperand::Property { source, access } => {
                let (mut prefix, base) = protocol.lower(self, access.target())?;
                let base = retain_value(self, &mut prefix, base, "generator.delete.base.");
                let mut operands = vec![(access.target(), base)];
                if let PropertyAccessField::Expr(key) = access.field() {
                    let (key_prefix, value) = protocol.lower(self, key)?;
                    prefix.extend(key_prefix);
                    let value = retain_value(self, &mut prefix, value, "generator.delete.key.");
                    operands.push((key.as_ref(), value));
                }
                // Reuse the real Delete Reference consumer with only its raw
                // operands substituted. Acquiring a PropertyRead would invoke Get.
                let mut restored = Vec::with_capacity(operands.len());
                for (source, name) in operands {
                    let key = std::ptr::from_ref(source) as usize;
                    let value = self.lower_identifier_name(name, false);
                    let previous = self.pinned_async_operands.insert(key, value);
                    restored.push((key, previous));
                }
                let value = self.lower_delete(source);
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
                Some((prefix, value))
            }
            GeneratorDeleteOperand::Optional(source) => {
                self.lower_generator_optional_delete(source)
            }
            GeneratorDeleteOperand::AwaitedOptional(source) => {
                let enclosing = self.async_expression_prefix.replace(Vec::new());
                let value = self.lower_optional_delete_property(source);
                let prefix = std::mem::replace(&mut self.async_expression_prefix, enclosing)
                    .expect("async optional Delete owns its complete prefix");
                Some((prefix, value))
            }
            GeneratorDeleteOperand::Value(source) => {
                let (prefix, value) = protocol.lower(self, source)?;
                Some((
                    prefix,
                    TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::DeleteValue {
                            expr: Box::new(value),
                        },
                    ),
                ))
            }
        }
    }
}
