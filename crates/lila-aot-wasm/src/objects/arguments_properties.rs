use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_arguments_delete_index(
        &self,
        object: &ValueLocals,
        index: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let arguments = schema.reserve_gc_local(function).initialize(
            object.cast_reference::<crate::gc_types::ArgumentsObject>(schema, function),
            function,
        );
        let indexed = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<crate::gc_types::ArgumentsObject>()
                .field(crate::gc_types::ArgumentsObjectSchema::INDEXED)
                .read(&arguments, schema, function)
                .reference(),
            function,
        );
        let length = schema.reserve_i32_local(function);
        schema
            .array_type::<crate::gc_types::IndexedTable>()
            .length(&indexed, schema, function);
        length.store(function);
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        schema.array_type::<crate::gc_types::IndexedTable>().write(
            &indexed,
            index,
            GcOperand::null(schema),
            schema,
            function,
        );
        function.instruction(&Instruction::End);
        let map = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<crate::gc_types::ArgumentsObject>()
                .field(crate::gc_types::ArgumentsObjectSchema::PARAMETER_MAP)
                .read(&arguments, schema, function)
                .reference(),
            function,
        );
        map.load(schema, function);
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let present = schema.reserve_gc_local(function).initialize(
            map.load(schema, function).require_non_null(function),
            function,
        );
        schema
            .array_type::<crate::gc_types::ArgumentsParameterMap>()
            .length(&present, schema, function);
        length.store(function);
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        schema
            .array_type::<crate::gc_types::ArgumentsParameterMap>()
            .write(&present, index, GcOperand::null(schema), schema, function);
        function.instruction(&Instruction::End);
        present.clear(function);
        function.instruction(&Instruction::End);
        map.clear(function);
        schema.release_i32_local(length, function);
        indexed.clear(function);
        arguments.clear(function);
    }
}
