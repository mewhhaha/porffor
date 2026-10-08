use super::*;
use crate::ir::reference::{CapturedSuperPropertyBaseSlot, CapturedSuperReferencedNameSlot};

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_super_property_receiver(&mut self) -> Option<TypedExpr> {
        if self.class_context.is_none()
            && !matches!(
                self.direct_eval_invocation(),
                Some(
                    lila_front::EvalInvocationContext::Method
                        | lila_front::EvalInvocationContext::DerivedConstructor
                        | lila_front::EvalInvocationContext::ClassFieldInitializer
                )
            )
        {
            self.unsupported("object literal method");
            return None;
        }
        Some(self.lower_current_this())
    }

    /// Direct reads and fused mutations consume the same actual receiver gate,
    /// key evaluation and inferred GetValue result.
    pub(super) fn lower_super_property_reference_parts(
        &mut self,
        access: &SuperPropertyAccess,
    ) -> Option<(PropertyKeyIr, Box<TypedExpr>, ValueInfo)> {
        let receiver = Box::new(self.lower_super_property_receiver()?);
        let key = self.lower_super_property_key(access.field())?;
        let base = TypedExpr::from_info(
            Self::value_info_from_shape(
                self.class_context
                    .as_ref()
                    .and_then(|context| context.super_base_shape.clone()),
            ),
            ExprIr::Undefined,
        );
        let (name, property) = match &key {
            PropertyKeyIr::StaticString(name) => (
                Some(name.clone()),
                self.read_current_object_shape_property(&base, name),
            ),
            PropertyKeyIr::StringExpr(key) if matches!(&key.expr, ExprIr::WellKnownSymbol(_)) => {
                let ExprIr::WellKnownSymbol(symbol) = &key.expr else {
                    unreachable!("well-known symbol guard supplies the key")
                };
                (
                    Some(shape_namespace_key(*symbol)),
                    self.read_current_object_symbol_shape_property(&base, *symbol),
                )
            }
            PropertyKeyIr::StringExpr(_)
            | PropertyKeyIr::ArrayIndex(_)
            | PropertyKeyIr::ArrayLength => (None, None),
        };
        let info = match property {
            Some(ObjectShapeProperty::Data(info)) => info,
            Some(ObjectShapeProperty::Accessor { getter: None, .. }) => ValueInfo::undefined(),
            property => {
                let info = match property {
                    Some(ObjectShapeProperty::Accessor {
                        getter: Some(getter),
                        ..
                    }) => {
                        self.merge_function_this_info(&getter.function_id, receiver.value_info());
                        if let Some(signature) =
                            self.function_signatures.get_mut(&getter.function_id)
                        {
                            Self::merge_omitted_signature_params_as_undefined(signature, 0);
                        }
                        self.accessor_return_info(&getter.function_id)
                    }
                    None => name
                        .as_deref()
                        .map_or_else(unknown_runtime_value_info, |name| {
                            self.unproven_object_property_info(&base, name)
                        }),
                    Some(ObjectShapeProperty::Data(_))
                    | Some(ObjectShapeProperty::Accessor { getter: None, .. }) => {
                        unreachable!("effect-free descriptors were handled above")
                    }
                };
                // Super still performs Get with the actual `this` before call
                // arguments or a compound-assignment RHS are evaluated.
                self.observe_all_planned_source_as_unknown_property_hooks();
                self.invalidate_unknown_user_code_effects();
                info
            }
        };
        Some((key, receiver, info))
    }

    /// PutValue of an already-obtained `value` through a freshly evaluated
    /// SuperProperty Reference (for-in/of heads, where the Reference is
    /// evaluated after the iteration value exists).
    pub(super) fn lower_super_property_assign_value(
        &mut self,
        access: &SuperPropertyAccess,
        value: TypedExpr,
    ) -> Option<TypedExpr> {
        let (name, receiver, _) = self.lower_super_property_reference_parts(access)?;
        self.record_caller_flow_invalidation();
        Some(TypedExpr::from_info(
            value.value_info(),
            ExprIr::SuperPropertyWrite {
                key: name,
                receiver,
                value: Box::new(value),
                strictness: self.reference_strictness(),
            },
        ))
    }

    /// A SuperProperty destructuring-assignment target. The Reference is
    /// captured when the target is prepared (before the element value is
    /// obtained); the PutValue reuses the captured base, `this` and
    /// uncoerced name, never re-resolving them after user code runs.
    pub(super) fn lower_super_destructuring_target(
        &mut self,
        access: &SuperPropertyAccess,
    ) -> Option<DestructuringTargetIr> {
        let receiver = self.lower_super_property_receiver()?;
        let key = self.lower_super_property_key(access.field())?;
        let receiver_slot =
            CapturedPropertyReceiverSlot::new(self.alloc_temp_binding_name("super.target.this."));
        let base_slot =
            CapturedSuperPropertyBaseSlot::new(self.alloc_temp_binding_name("super.target.base."));
        let name_slot = CapturedSuperReferencedNameSlot::new(
            self.alloc_temp_binding_name("super.target.name."),
        );
        let value_binding = self.alloc_temp_binding_name("super.target.value.");
        let plan =
            SuperPropertyReferencePlan::new(Box::new(receiver), key, self.reference_strictness());
        let (capture, reference) = plan.capture_reference(
            receiver_slot,
            base_slot,
            name_slot,
            SuperPropertyCaptureMode::WriteOnly,
        );
        let ExprIr::SuperPropertyMutation(mutation) = &capture.expr else {
            unreachable!("Super capture is produced by its consuming Reference plan")
        };
        let SuperPropertyMutationOperationIr::Capture(slots) = mutation.operation() else {
            unreachable!("capture factory owns the exact operation")
        };
        self.pending_super_destructuring_slots.extend([
            slots.receiver_storage_name().to_string(),
            slots.base_storage_name().to_string(),
            slots.referenced_name_storage_name().to_string(),
            value_binding.clone(),
        ]);
        let value = TypedExpr::from_info(
            unknown_runtime_value_info(),
            ExprIr::Identifier(value_binding.clone()),
        );
        let put = reference.write(value);
        // PutValue can run a setter, a Proxy trap or key coercion.
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();
        self.record_caller_flow_invalidation();
        Some(DestructuringTargetIr::AssignmentSuper {
            capture: Box::new(capture),
            value_binding,
            put: Box::new(put),
        })
    }

    /// Reify the receiver/key/strictness tuple produced by evaluating one
    /// SuperProperty. The returned plan is the only producer of the fused
    /// mutation IR and cannot be cloned or decomposed into separate writes.
    fn lower_super_property_reference_plan(
        &mut self,
        access: &SuperPropertyAccess,
    ) -> Option<(SuperPropertyReferencePlan, ValueInfo)> {
        let (key, receiver, info) = self.lower_super_property_reference_parts(access)?;
        Some((
            SuperPropertyReferencePlan::new(receiver, key, self.reference_strictness()),
            info,
        ))
    }

    pub(super) fn lower_super_property_numeric_update(
        &mut self,
        source_op: UpdateOp,
        access: &SuperPropertyAccess,
    ) -> TypedExpr {
        self.record_caller_flow_invalidation();
        let Some((key, receiver, read_info)) = self.lower_super_property_reference_parts(access)
        else {
            return TypedExpr::undefined();
        };
        self.lower_super_property_numeric_update_from_parts(source_op, key, receiver, read_info)
    }

    pub(super) fn lower_super_property_numeric_update_from_parts(
        &mut self,
        source_op: UpdateOp,
        key: PropertyKeyIr,
        receiver: Box<TypedExpr>,
        read_info: ValueInfo,
    ) -> TypedExpr {
        let plan = SuperPropertyReferencePlan::new(receiver, key, self.reference_strictness());
        let (op, return_mode) = match source_op {
            UpdateOp::IncrementPost => (NumericUpdateOp::Increment, UpdateReturnMode::Postfix),
            UpdateOp::IncrementPre => (NumericUpdateOp::Increment, UpdateReturnMode::Prefix),
            UpdateOp::DecrementPost => (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix),
            UpdateOp::DecrementPre => (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix),
        };
        let value_kind = match read_info.kind {
            ValueKind::Number => NumericUpdateValueKind::Number,
            ValueKind::BigInt => NumericUpdateValueKind::BigInt,
            ValueKind::Undefined
            | ValueKind::Null
            | ValueKind::Boolean
            | ValueKind::String
            | ValueKind::Symbol
            | ValueKind::Object
            | ValueKind::Array
            | ValueKind::Function
            | ValueKind::Arguments
            | ValueKind::Dynamic => NumericUpdateValueKind::Dynamic,
        };
        plan.numeric_update(op, return_mode, value_kind)
    }

    pub(super) fn lower_super_property_eager_compound_assignment(
        &mut self,
        access: &SuperPropertyAccess,
        op: EagerCompoundAssignmentOp,
        rhs: &Expression,
    ) -> TypedExpr {
        self.record_caller_flow_invalidation();
        let Some((plan, _)) = self.lower_super_property_reference_plan(access) else {
            return TypedExpr::undefined();
        };
        let rhs = self.lower_expression(rhs);
        let old_value_binding = self.alloc_temp_binding_name("super.property.mutation.old.");
        plan.eager_compound_assignment(old_value_binding, op, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};

    fn lower_object_methods(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        crate::lower(&source)
    }

    fn returned_mutation<'a>(script: &'a ScriptIr, name: &str) -> &'a SuperPropertyMutationIr {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("missing object method {name}"));
        let StatementIr::Return(value) = function
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("method should return its mutation")
        else {
            unreachable!("selected statement is a return")
        };
        let ExprIr::SuperPropertyMutation(mutation) = &value.expr else {
            panic!("expected fused Super mutation, got {:?}", value.expr);
        };
        mutation
    }

    #[test]
    fn super_property_reference_mutation_is_one_closed_receiver_key_operation() {
        let source = r#"
            const base = { p: 1 };
            const object = {
                __proto__: base,
                compound(key, rhs) { return super[key] += rhs; },
                postIncrement(key) { return super[key]++; },
                preIncrement(key) { return ++super[key]; },
                postDecrement(key) { return super[key]--; },
                preDecrement(key) { return --super[key]; }
            };
        "#;
        let program = lower_object_methods(source);
        assert!(
            program.is_wasm_supported(),
            "Super mutations should lower: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script IR");

        let compound = returned_mutation(script, "compound");
        assert!(matches!(&compound.receiver().expr, ExprIr::This));
        assert!(matches!(
            compound.referenced_name(),
            PropertyKeyIr::StringExpr(key) if matches!(&key.expr, ExprIr::Identifier(_))
        ));
        let SuperPropertyMutationOperationIr::EagerCompound {
            old_value_binding,
            result,
        } = compound.operation()
        else {
            panic!("compound method must carry an eager operation");
        };
        assert!(old_value_binding.starts_with("$super.property.mutation.old"));
        let ExprIr::CoerciveAdd { lhs, .. } = &result.expr else {
            panic!(
                "+= must use the shared eager Add application: {:?}",
                result.expr
            );
        };
        assert!(matches!(
            &lhs.expr,
            ExprIr::Identifier(name) if name == old_value_binding
        ));

        let modes = [
            (
                "postIncrement",
                NumericUpdateOp::Increment,
                UpdateReturnMode::Postfix,
            ),
            (
                "preIncrement",
                NumericUpdateOp::Increment,
                UpdateReturnMode::Prefix,
            ),
            (
                "postDecrement",
                NumericUpdateOp::Decrement,
                UpdateReturnMode::Postfix,
            ),
            (
                "preDecrement",
                NumericUpdateOp::Decrement,
                UpdateReturnMode::Prefix,
            ),
        ];
        for (name, expected_op, expected_mode) in modes {
            let mutation = returned_mutation(script, name);
            assert!(matches!(&mutation.receiver().expr, ExprIr::This));
            assert!(matches!(
                mutation.operation(),
                SuperPropertyMutationOperationIr::NumericUpdate {
                    op,
                    return_mode,
                    value_kind:
                        NumericUpdateValueKind::Number | NumericUpdateValueKind::Dynamic,
                } if *op == expected_op && *return_mode == expected_mode
            ));
        }
    }
}
