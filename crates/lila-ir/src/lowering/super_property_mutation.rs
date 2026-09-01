use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Lower the shared operands and inferred GetValue result for a
    /// SuperProperty. Direct reads and fused mutations call this one producer,
    /// so the two paths cannot disagree about receiver or key representation.
    pub(super) fn lower_super_property_reference_parts(
        &mut self,
        access: &SuperPropertyAccess,
    ) -> Option<(PropertyKeyIr, Box<TypedExpr>, ValueInfo)> {
        if self.class_context.is_none() {
            self.unsupported("object literal method");
            return None;
        }
        let receiver = Box::new(self.lower_current_this());
        let key = self.lower_super_property_key(access.field())?;
        let info = match &key {
            PropertyKeyIr::StaticString(key_name) => self
                .class_context
                .as_ref()
                .and_then(|context| context.super_base_shape.as_deref())
                .and_then(|shape| read_heap_shape_property(shape, key_name))
                .map(|property| match property {
                    ObjectShapeProperty::Data(info) => info,
                    ObjectShapeProperty::Accessor {
                        getter: Some(getter),
                        ..
                    } => self.accessor_return_info(&getter.function_id),
                    ObjectShapeProperty::Accessor { getter: None, .. } => ValueInfo::undefined(),
                })
                .unwrap_or_else(unknown_runtime_value_info),
            PropertyKeyIr::StringExpr(_)
            | PropertyKeyIr::ArrayIndex(_)
            | PropertyKeyIr::ArrayLength => unknown_runtime_value_info(),
        };
        Some((key, receiver, info))
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
        let Some((plan, read_info)) = self.lower_super_property_reference_plan(access) else {
            return TypedExpr::undefined();
        };
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
