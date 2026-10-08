//! Each suspension protocol retains one actual Reference through PutValue.

use super::generator_identifier_reference::RetainedGeneratorIdentifierReference;
use super::pattern_target::retain_pattern_value;
use super::resumable_operand::ResumableOperandProtocol;
use super::*;
use crate::ir::reference::{
    CapturedOrdinaryPropertyReference, CapturedSuperPropertyBaseSlot,
    CapturedSuperPropertyReference, CapturedSuperReferencedNameSlot,
};

/// Consuming the selected arm spends the original Reference. In particular,
/// an ordinary property's canonical key cannot be reconstructed from the RHS.
pub(super) enum RetainedAssignmentReference {
    Identifier(RetainedGeneratorIdentifierReference),
    Property(CapturedOrdinaryPropertyReference),
    Super(CapturedSuperPropertyReference),
    Private {
        target: TypedExpr,
        private_name_id: PrivateNameId,
    },
}

impl RetainedAssignmentReference {
    pub(super) fn put_value(self, lowerer: &mut ScriptLowerer<'_>, value: TypedExpr) -> TypedExpr {
        let mut result = match self {
            Self::Identifier(reference) => reference.put_value(lowerer, value),
            Self::Property(reference) => {
                let (_, mut setters) = lowerer.possible_unknown_accessor_functions();
                setters.extend_known(lowerer.dynamically_installed_setters.iter().cloned());
                lowerer.observe_all_planned_source_as_unknown_property_hooks();
                lowerer.invalidate_unknown_user_code_effects();
                reference.write(value, setters)
            }
            Self::Super(reference) => {
                lowerer.observe_all_planned_source_as_unknown_property_hooks();
                lowerer.invalidate_unknown_user_code_effects();
                lowerer.record_caller_flow_invalidation();
                reference.write(value)
            }
            Self::Private {
                target,
                private_name_id,
            } => {
                lowerer.observe_all_planned_source_as_unknown_property_hooks();
                lowerer.invalidate_unknown_user_code_effects();
                lowerer.record_caller_flow_invalidation();
                TypedExpr::from_info(
                    value.value_info(),
                    ExprIr::PrivateWrite {
                        target: Box::new(target),
                        private_name_id,
                        value: Box::new(value),
                    },
                )
            }
        };
        // A setter preserves the result's value, but may mutate its object.
        result.heap_shape = None;
        result
    }

    pub(super) fn release_operation(&self) -> Option<(String, EnvironmentIdentifierOperationIr)> {
        match self {
            Self::Identifier(reference) => Some(reference.release_operation()),
            Self::Property(_) | Self::Super(_) | Self::Private { .. } => None,
        }
    }
}

impl ScriptLowerer<'_> {
    pub(super) fn capture_resumable_assignment_reference(
        &mut self,
        source: &AssignTarget,
        prefix: &mut Vec<StatementIr>,
    ) -> Option<(RetainedAssignmentReference, TypedExpr)> {
        let protocol = ResumableOperandProtocol::current(self)?;
        let (reference, read) = match source {
            AssignTarget::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                let reference = RetainedGeneratorIdentifierReference::capture(self, prefix, name);
                let old = reference.old_value();
                return Some((RetainedAssignmentReference::Identifier(reference), old));
            }
            AssignTarget::Access(PropertyAccess::Simple(access)) => {
                let (before, target) = protocol.lower(self, access.target())?;
                prefix.extend(before);
                let target =
                    retain_pattern_value(self, prefix, "async.generator.reference.base.", target);
                let key = match access.field() {
                    PropertyAccessField::Const(name) => PropertyKeyIr::StaticString(
                        self.interner.resolve_expect(name.sym()).to_string(),
                    ),
                    PropertyAccessField::Expr(source) => {
                        let (before, key) = protocol.lower(self, source)?;
                        prefix.extend(before);
                        PropertyKeyIr::StringExpr(Box::new(key))
                    }
                };
                let plan = OrdinaryPropertyReferencePlan::new(
                    Box::new(target),
                    key,
                    self.reference_strictness(),
                );
                let receiver = self.alloc_captured_property_receiver();
                let target = self.alloc_captured_property_target();
                let key = self.alloc_captured_property_key();
                let (getters, _) = self.possible_unknown_accessor_functions();
                let (read, reference) = plan.capture_get(receiver, target, key, getters);
                let ExprIr::OrdinaryPropertyGetCapture(capture) = &read.expr else {
                    unreachable!("the Reference owner produces its own Get capture")
                };
                for name in [
                    capture.receiver_storage_name(),
                    capture.target_storage_name(),
                    capture.key_storage_name(),
                ] {
                    prefix.push(StatementIr::Lexical {
                        mode: BindingMode::Let,
                        name: name.to_string(),
                        init: TypedExpr::undefined(),
                    });
                }
                self.observe_all_planned_source_as_unknown_property_hooks();
                self.invalidate_unknown_user_code_effects();
                (RetainedAssignmentReference::Property(reference), read)
            }
            AssignTarget::Access(PropertyAccess::Private(access)) => {
                let (mut target, private_name_id) =
                    self.lower_resumable_private_target(access, prefix)?;
                let read = self.lower_private_get_value(&mut target, private_name_id);
                (
                    RetainedAssignmentReference::Private {
                        target,
                        private_name_id,
                    },
                    read,
                )
            }
            AssignTarget::Access(PropertyAccess::Super(access)) => {
                let (reference, read) = self.capture_resumable_super_reference(
                    access,
                    SuperPropertyCaptureMode::ReadBeforeRhs,
                    prefix,
                )?;
                (RetainedAssignmentReference::Super(reference), read)
            }
            AssignTarget::Pattern(_) | AssignTarget::WebCompatCall(_) => return None,
        };
        // Execute GetValue before entering the RHS, including its nullish,
        // private-brand, key-conversion and accessor abrupt completions.
        let old = retain_pattern_value(self, prefix, "async.generator.reference.old.", read);
        Some((reference, old))
    }

    fn lower_resumable_private_target(
        &mut self,
        access: &PrivatePropertyAccess,
        prefix: &mut Vec<StatementIr>,
    ) -> Option<(TypedExpr, PrivateNameId)> {
        let private_name_id = self.current_private_name_id(access.field())?;
        let protocol = ResumableOperandProtocol::current(self)?;
        let (before, target) = protocol.lower(self, access.target())?;
        prefix.extend(before);
        let target = retain_pattern_value(self, prefix, "async.generator.private.base.", target);
        Some((target, private_name_id))
    }

    pub(super) fn capture_resumable_super_reference(
        &mut self,
        access: &SuperPropertyAccess,
        mode: SuperPropertyCaptureMode,
        prefix: &mut Vec<StatementIr>,
    ) -> Option<(CapturedSuperPropertyReference, TypedExpr)> {
        let protocol = ResumableOperandProtocol::current(self)?;
        let receiver = self.lower_super_property_receiver()?;
        let receiver = retain_pattern_value(self, prefix, "async.generator.super.this.", receiver);
        let key = match access.field() {
            PropertyAccessField::Const(name) => {
                PropertyKeyIr::StaticString(self.interner.resolve_expect(name.sym()).to_string())
            }
            PropertyAccessField::Expr(source) => {
                let (before, key) = protocol.lower(self, source)?;
                prefix.extend(before);
                PropertyKeyIr::StringExpr(Box::new(key))
            }
        };
        let receiver_slot = self.alloc_captured_property_receiver();
        let base_slot = CapturedSuperPropertyBaseSlot::new(self.alloc_suspension_owned_binding(
            "async.generator.super.base.",
            unknown_runtime_value_info(),
        ));
        let name_slot = CapturedSuperReferencedNameSlot::new(self.alloc_suspension_owned_binding(
            "async.generator.super.name.",
            unknown_runtime_value_info(),
        ));
        let plan =
            SuperPropertyReferencePlan::new(Box::new(receiver), key, self.reference_strictness());
        let (read, reference) = plan.capture_reference(receiver_slot, base_slot, name_slot, mode);
        let ExprIr::SuperPropertyMutation(mutation) = &read.expr else {
            unreachable!("Super capture is produced by its consuming Reference plan")
        };
        let SuperPropertyMutationOperationIr::Capture(capture) = mutation.operation() else {
            unreachable!("capture factory owns the exact operation")
        };
        for name in [
            capture.receiver_storage_name(),
            capture.base_storage_name(),
            capture.referenced_name_storage_name(),
        ] {
            prefix.push(StatementIr::Lexical {
                mode: BindingMode::Let,
                name: name.to_string(),
                init: TypedExpr::undefined(),
            });
        }
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();
        Some((reference, read))
    }

    pub(super) fn lower_resumable_private_read(
        &mut self,
        access: &PrivatePropertyAccess,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let mut prefix = Vec::new();
        let (mut target, private_name_id) =
            self.lower_resumable_private_target(access, &mut prefix)?;
        let value = self.lower_private_get_value(&mut target, private_name_id);
        Some((prefix, value))
    }

    pub(super) fn lower_resumable_property_assignment(
        &mut self,
        assignment: &boa_ast::expression::operator::Assign,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_resumable_property_assignment_parts(
            assignment.op(),
            assignment.lhs(),
            assignment.rhs(),
        )
    }

    pub(super) fn lower_resumable_property_assignment_parts(
        &mut self,
        operation: AssignOp,
        target: &AssignTarget,
        rhs: &Expression,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let protocol = ResumableOperandProtocol::current(self)?;
        let mut prefix = Vec::new();
        let (reference, old) = if operation == AssignOp::Assign {
            let reference = match target {
                AssignTarget::Access(PropertyAccess::Private(access)) => {
                    // Plain assignment retains the base without performing
                    // PrivateGet or a brand check ahead of the RHS.
                    let (target, private_name_id) =
                        self.lower_resumable_private_target(access, &mut prefix)?;
                    RetainedAssignmentReference::Private {
                        target,
                        private_name_id,
                    }
                }
                AssignTarget::Access(PropertyAccess::Super(access)) => {
                    let (reference, capture) = self.capture_resumable_super_reference(
                        access,
                        SuperPropertyCaptureMode::WriteOnly,
                        &mut prefix,
                    )?;
                    prefix.push(StatementIr::Expression(capture));
                    RetainedAssignmentReference::Super(reference)
                }
                AssignTarget::Identifier(_)
                | AssignTarget::Access(PropertyAccess::Simple(_))
                | AssignTarget::Pattern(_)
                | AssignTarget::WebCompatCall(_) => return None,
            };
            (reference, None)
        } else {
            let (reference, old) =
                self.capture_resumable_assignment_reference(target, &mut prefix)?;
            (reference, Some(old))
        };
        let (before, rhs) = protocol.lower(self, rhs)?;
        prefix.extend(before);
        let value = match old {
            Some(old) => self.combine_compound_assignment_values(operation, old, rhs)?,
            None => rhs,
        };
        Some((prefix, reference.put_value(self, value)))
    }
}
