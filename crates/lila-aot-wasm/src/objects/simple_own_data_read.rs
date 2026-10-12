//! Effect-free own data reads avoid public descriptor snapshots and primitive
//! boxing. Keys have already completed ToPropertyKey; no prototype or accessor
//! lookup can be bypassed by these concrete representation guards.
use super::*;
use crate::gc_types::{GcNullability, StringValue};

impl FunctionBuilder<'_> {
    pub(super) fn emit_primitive_string_length_read(
        &mut self,
        target: &ValueLocals,
        key: &PropertyKeyLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<I32Local, EmitError> {
        let schema = self.runtime_schema();
        let handled = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        handled.store(function);
        target.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let name = schema.reserve_gc_local(function).initialize(
            self.emit_interned_string_reference("length", function)?,
            function,
        );
        let length_key = PropertyKeyLocals::from_string(schema, &name, function);
        self.emit_tagged_payload_same_value_i32(key.value(), length_key.value(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        let string = schema.reserve_gc_local(function).initialize(
            target.cast_reference::<StringValue>(schema, function),
            function,
        );
        let length = schema.reserve_i64_local(function);
        self.emit_native_gc_string_length(&string, length, function);
        length.load(function);
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        length.store(function);
        let value = schema.reserve_value_local(function);
        value.set_number(length, function);
        result.set_normal(&value, function);
        function.instruction(&Instruction::I32Const(1));
        handled.store(function);
        value.clear(function);
        schema.release_i64_local(length, function);
        string.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        length_key.clear(function);
        name.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(handled)
    }

    pub(super) fn emit_ordinary_own_data_read(
        &mut self,
        target: &ValueLocals,
        key: &PropertyKeyLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<I32Local, EmitError> {
        let schema = self.runtime_schema();
        let handled = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        handled.store(function);
        target.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<OrdinaryObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        // Every exotic stores a separate OrdinaryObject header; its outer GC
        // reference is a distinct type and cannot enter this concrete arm.
        let object = schema.reserve_gc_local(function).initialize(
            target.cast_reference::<OrdinaryObject>(schema, function),
            function,
        );
        let descriptor = self.emit_ordinary_own_descriptor_reference(&object, key, function)?;
        descriptor.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let present = schema.reserve_gc_local(function).initialize(
            descriptor.load(schema, function).require_non_null(function),
            function,
        );
        let flags = schema.reserve_i64_local(function);
        schema
            .struct_type::<PropertyDescriptor>()
            .field(PropertyDescriptorSchema::FLAGS)
            .read(&present, schema, function)
            .store_i64(flags, function);
        flags.load(function);
        function.instruction(&Instruction::I64Const(DescriptorMask::ACCESSOR.as_i64()));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<PropertyDescriptor>()
                .field(PropertyDescriptorSchema::VALUE)
                .read(&present, schema, function)
                .reference(),
            function,
        );
        let value = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &value, schema, function);
        result.set_normal(&value, function);
        function.instruction(&Instruction::I32Const(1));
        handled.store(function);
        value.clear(function);
        stored.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i64_local(flags, function);
        present.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        descriptor.clear(function);
        object.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(handled)
    }
}
