//! Capacity growth shared by the named and Arguments descriptor tables.
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_ordinary_append_property_entry(
        &self,
        object: &GcLocal<OrdinaryObject>,
        key: &PropertyKeyLocals,
        descriptor: &GcLocal<PropertyDescriptor>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.runtime_schema().call_helper(
            crate::runtime_helpers::OrdinaryPropertyAppendArguments::new(object, key, descriptor),
            self.runtime_helper_base()?,
            function,
        );
        Ok(())
    }

    pub(crate) fn compile_ordinary_property_append_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function =
            self.begin_helper_body(crate::runtime_helpers::RuntimeHelperId::OrdinaryPropertyAppend);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::OrdinaryPropertyAppendParameters>(
                &mut function,
            );
        self.emit_ordinary_append_property_entry_inner(
            &parameters.object,
            &parameters.key,
            &parameters.descriptor,
            &mut function,
        );
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Appends the actual GC entry and preserves existing table order and holes.
    /// Capacity doubles only when full; the storage retains a replacement
    /// before temporary roots retire. Deletion holes never become new slots.
    fn emit_ordinary_append_property_entry_inner(
        &self,
        object: &GcLocal<OrdinaryObject>,
        key: &PropertyKeyLocals,
        descriptor: &GcLocal<PropertyDescriptor>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let storage = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<OrdinaryObject>()
                .field(OrdinaryObjectSchema::PROPERTIES)
                .read(object, schema, function)
                .reference(),
            function,
        );
        let table = schema.reserve_gc_local(function).initialize(
            schema
                .field(OrdinaryPropertyStorageSchema::ENTRIES)
                .read(&storage, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        let new_length = schema.reserve_i32_local(function);
        let capacity = schema.reserve_i32_local(function);
        let new_capacity = schema.reserve_i32_local(function);
        let entries = schema.array_type::<PropertyTable>();
        schema
            .field(OrdinaryPropertyStorageSchema::LENGTH)
            .read(&storage, schema, function)
            .store(length, function);
        length.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        new_length.store(function);
        entries.length(&table, schema, function);
        capacity.store(function);
        length.load(function);
        capacity.load(function);
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_property_table_growth_capacity(capacity, new_length, new_capacity, function);
        let replacement_slot = schema.reserve_gc_local(function);
        let replacement = replacement_slot.initialize(
            entries.filled(GcOperand::null(schema), new_capacity, function),
            function,
        );
        entries.copy_prefix_from(&replacement, &table, length, schema, function);
        schema.field(OrdinaryPropertyStorageSchema::ENTRIES).write(
            &storage,
            GcOperand::reference(&replacement, schema),
            schema,
            function,
        );
        table.replace(replacement.load(schema, function), function);
        replacement.clear(function);
        function.instruction(&Instruction::End);
        let stored_key_slot = schema.reserve_gc_local(function);
        let stored_key = stored_key_slot.initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(key.value(), function),
            function,
        );
        let entry_slot = schema.reserve_gc_local(function);
        let entry = entry_slot.initialize(
            schema.struct_type::<PropertyEntry>().construct(
                (
                    GcOperand::reference(&stored_key, schema),
                    GcOperand::reference(descriptor, schema),
                ),
                function,
            ),
            function,
        );
        entries.write(
            &table,
            length,
            GcOperand::nullable_reference(&entry, schema),
            schema,
            function,
        );
        schema.field(OrdinaryPropertyStorageSchema::LENGTH).write(
            &storage,
            GcOperand::i32_local(new_length),
            schema,
            function,
        );
        entry.clear(function);
        stored_key.clear(function);
        schema.release_i32_local(new_capacity, function);
        schema.release_i32_local(capacity, function);
        schema.release_i32_local(new_length, function);
        schema.release_i32_local(length, function);
        table.clear(function);
        storage.clear(function);
    }

    /// Chooses max(4, required, 2 * capacity), without unsigned i32 wrap.
    /// An unrepresentable required extent is a private storage failure, never
    /// a fabricated JavaScript exception or silently truncated property list.
    pub(super) fn emit_property_table_growth_capacity(
        &self,
        capacity: I32Local,
        required: I32Local,
        output: I32Local,
        function: &mut Function,
    ) {
        required.load(function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        capacity.load(function);
        function.instruction(&Instruction::I32Const(i32::MAX));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        capacity.load(function);
        function.instruction(&Instruction::I32Const(2));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::Else);
        required.load(function);
        function.instruction(&Instruction::End);
        output.store(function);
        output.load(function);
        required.load(function);
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        required.load(function);
        output.store(function);
        function.instruction(&Instruction::End);
        output.load(function);
        function.instruction(&Instruction::I32Const(4));
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I32Const(4));
        output.store(function);
        function.instruction(&Instruction::End);
    }
}
