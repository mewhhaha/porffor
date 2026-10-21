//! CreateListFromArrayLike can retain an immutable own data value directly.
//! Numeric Array indices live exclusively in the strong-GC sparse store. A
//! missing entry or accessor must still use ordinary Get with the Array receiver.
use super::*;
use crate::gc_types::{ArrayObject, GcNullability};

impl FunctionBuilder<'_> {
    /// Raw primitive numeric keys can address an actual Array own data slot
    /// without allocating its decimal property name and parsing it back. The
    /// guarded index is never published as a PropertyKey. Every miss/accessor
    /// retains the original ToPropertyKey/Get path and its Array receiver.
    pub(crate) fn emit_numeric_array_own_data_read(
        &mut self,
        target: &ValueLocals,
        raw_key: &ValueLocals,
        result: &CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        raw_key.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Number as i32));
        function.instruction(&Instruction::I32Eq);
        target.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<ArrayObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        raw_key.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ge);
        raw_key.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(u32::MAX as f64)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::I32And);
        raw_key.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        raw_key.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let index = schema.reserve_i32_local(function);
        raw_key.scalar().load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        // Dominated by finite nonnegative integer and max-index guards; -0
        // correctly becomes zero, while 2^32-1 remains a named-property miss.
        function.instruction(&Instruction::I32TruncF64U);
        index.store(function);
        let stored = self.emit_array_own_indexed_data_reference(target, index, function);
        stored.load(schema, function).is_null(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let present = schema.reserve_gc_local(function).initialize(
            stored.load(schema, function).require_non_null(function),
            function,
        );
        let value = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&present, &value, schema, function);
        result.set_normal(&value, function);
        self.emit_branch_to_target(exit, function);
        value.clear(function);
        present.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        stored.clear(function);
        schema.release_i32_local(index, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    pub(crate) fn emit_array_own_indexed_data_reference(
        &mut self,
        target: &ValueLocals,
        index: I32Local,
        function: &mut Function,
    ) -> GcLocal<StoredValue, Nullable> {
        let schema = self.runtime_schema();
        let output = schema
            .reserve_gc_local(function)
            .initialize_null(schema, function);
        target.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<ArrayObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        self.open_frame(ControlFrameKind::If, function);
        let array = schema.reserve_gc_local(function).initialize(
            target.cast_reference::<ArrayObject>(schema, function),
            function,
        );
        let index64 = schema.reserve_i64_local(function);
        index.load(function);
        function.instruction(&Instruction::I64ExtendI32U);
        index64.store(function);
        let descriptor = self.emit_array_indexed_descriptor(&array, index64, function);
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
        output.replace(
            schema
                .struct_type::<PropertyDescriptor>()
                .field(PropertyDescriptorSchema::VALUE)
                .read(&present, schema, function)
                .reference()
                .nullable(),
            function,
        );
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        schema.release_i64_local(flags, function);
        present.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        descriptor.clear(function);
        schema.release_i64_local(index64, function);
        array.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        output
    }
}
