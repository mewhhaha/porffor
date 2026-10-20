use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_private_numeric_update(
        &mut self,
        source_op: UpdateOp,
        access: &PrivatePropertyAccess,
    ) -> TypedExpr {
        let Some(private_name_id) = self.current_private_name_id(access.field()) else {
            return self.unsupported_expr("private class element");
        };
        let target = self.lower_property_target(access.target());
        // The getter, old-value coercion, and setter can each run source code.
        self.observe_all_planned_source_as_unknown_property_hooks();
        self.invalidate_unknown_user_code_effects();

        let (op, return_mode) = match source_op {
            UpdateOp::IncrementPost => (NumericUpdateOp::Increment, UpdateReturnMode::Postfix),
            UpdateOp::IncrementPre => (NumericUpdateOp::Increment, UpdateReturnMode::Prefix),
            UpdateOp::DecrementPost => (NumericUpdateOp::Decrement, UpdateReturnMode::Postfix),
            UpdateOp::DecrementPre => (NumericUpdateOp::Decrement, UpdateReturnMode::Prefix),
        };
        let target_name = self.alloc_temp_binding_name("private.update.target.");
        let old_value_name = self.alloc_temp_binding_name("private.update.old.");
        let result_name = self.alloc_temp_binding_name("private.update.result.");
        let numeric_info = ValueInfo {
            kind: ValueKind::Dynamic,
            possible_kinds: KindSet::from_kind(ValueKind::Number)
                .union(KindSet::from_kind(ValueKind::BigInt)),
            heap_shape: None,
            function_targets: FunctionTargetKnowledge::none(),
        };
        // Even an Identifier must be captured: a getter or ToNumeric hook can
        // reassign it before PutValue, which must still use the original base.
        let held_target =
            TypedExpr::from_info(target.value_info(), ExprIr::Identifier(target_name.clone()));
        let mut reference = ReferenceRecord::create(
            ReferenceBase::Private {
                target: held_target,
                private_name_id,
            },
            self.reference_strictness(),
        );
        let pins = self.pin_reference_operands(&mut reference);
        let old_value = reference.read(unknown_runtime_value_info());
        let update = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::UpdateIdentifier {
                name: old_value_name.clone(),
                op,
                return_mode,
                value_kind: NumericUpdateValueKind::Dynamic,
            },
        );
        let updated_value = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::Identifier(old_value_name.clone()),
        );
        let write = pins.materialize(reference.write(updated_value, Composition::Value));
        let result = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::Identifier(result_name.clone()),
        );
        let mut body = TypedExpr::from_info(
            numeric_info.clone(),
            ExprIr::Comma {
                lhs: Box::new(write),
                rhs: Box::new(result),
            },
        );
        for (name, value) in [
            (target_name, target),
            (old_value_name, old_value),
            (result_name, update),
        ]
        .into_iter()
        .rev()
        {
            body = TypedExpr::from_info(
                numeric_info.clone(),
                ExprIr::MaterializeBinding {
                    name,
                    value: Box::new(value),
                    body: Box::new(body),
                },
            );
        }
        body
    }
}
