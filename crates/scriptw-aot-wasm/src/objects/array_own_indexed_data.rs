//! CreateListFromArrayLike can retain an immutable own data value directly.
//! Numeric Array indices live exclusively in the strong-GC sparse store. A
//! missing entry or accessor must still use ordinary Get with the Array receiver.
use super::*;
use crate::gc_types::{ArrayObject, GcNullability};

impl FunctionBuilder<'_> {
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
