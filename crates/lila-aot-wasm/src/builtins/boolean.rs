//! Boolean primitives and boxes share the completed GC primitive-data owner.

use super::super::*;
use crate::functions::OrdinaryDefaultPrototype;
use crate::gc_types::{GcNullability, GcOperand, PrimitiveBox, PrimitiveBoxSchema, StoredValue};

enum BooleanPrototypeOperation {
    ToString,
    ValueOf,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_boolean_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let primitive = schema.reserve_value_local(function);
        let new_target = schema.reserve_value_local(function);
        let output = schema.reserve_completion(function);
        let truth = schema.reserve_i32_local(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_to_boolean_payload_from_tagged_locals(&argument, function)?;
        function.instruction(&Instruction::I32WrapI64);
        truth.store(function);
        primitive.set_boolean(truth, function);
        output.set_normal(&primitive, function);
        self.compile_new_target_to_locals(&new_target, function)?;
        new_target.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        let prototype = schema.reserve_completion(function);
        self.emit_get_prototype_from_constructor(
            &new_target,
            OrdinaryDefaultPrototype::Boolean,
            &prototype,
            function,
        )?;
        output.copy_from(&prototype, function);
        prototype.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(prototype.value()), function)?,
            function,
        );
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&primitive, function),
            function,
        );
        let boxed = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<PrimitiveBox>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(&stored, schema),
                ),
                function,
            ),
            function,
        );
        let value = schema.reserve_value_local(function);
        value.set_reference(&boxed, schema, function);
        output.set_normal(&value, function);
        value.clear(function);
        boxed.clear(function);
        stored.clear(function);
        header.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        prototype.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        schema.release_i32_local(truth, function);
        output.clear(function);
        new_target.clear(function);
        primitive.clear(function);
        argument.clear(function);
        Ok(())
    }

    pub(super) fn emit_boolean_prototype_to_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_boolean_prototype_builtin(BooleanPrototypeOperation::ToString, function)
    }

    pub(super) fn emit_boolean_prototype_value_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_boolean_prototype_builtin(BooleanPrototypeOperation::ValueOf, function)
    }

    fn emit_boolean_prototype_builtin(
        &mut self,
        operation: BooleanPrototypeOperation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        let primitive = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        self.compile_this_to_locals(&receiver, function)?;
        primitive.copy_from(&receiver, function);
        receiver.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<PrimitiveBox>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let boxed = schema.reserve_gc_local(function).initialize(
            receiver.cast_reference::<PrimitiveBox>(schema, function),
            function,
        );
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PrimitiveBox>()
                .field(PrimitiveBoxSchema::PRIMITIVE)
                .read(&boxed, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &primitive, schema, function);
        stored.clear(function);
        boxed.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        primitive.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Boolean.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::BOOLEAN_PROTOTYPE_METHOD_REQUIRES_A_BOOLEAN_RECEIVER,
            &pending,
            function,
        )?;
        function.instruction(&Instruction::Else);
        match operation {
            BooleanPrototypeOperation::ValueOf => pending.set_normal(&primitive, function),
            BooleanPrototypeOperation::ToString => {
                let string = schema.reserve_value_local(function);
                primitive.scalar().load(function);
                function.instruction(&Instruction::I64Eqz);
                self.open_frame(ControlFrameKind::If, function);
                let text = schema.reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference("false", function)?,
                    function,
                );
                string.set_reference(&text, schema, function);
                text.clear(function);
                function.instruction(&Instruction::Else);
                let text = schema.reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference("true", function)?,
                    function,
                );
                string.set_reference(&text, schema, function);
                text.clear(function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                pending.set_normal(&string, function);
                string.clear(function);
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&pending, function);
        pending.clear(function);
        primitive.clear(function);
        receiver.clear(function);
        Ok(())
    }
}
