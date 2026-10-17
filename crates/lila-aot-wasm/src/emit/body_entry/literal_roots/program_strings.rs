use super::*;
use crate::gc_types::ProgramStringDescriptorTable;

impl FunctionBuilder<'_> {
    pub(super) fn initialize_program_string_roots<'image, 'pool>(
        &self,
        image: &'image ProgramInstanceImage<'pool>,
        function: &mut Function,
    ) -> ProgramLiteralRoots<'image, 'pool> {
        let descriptor_image = self.strings.program_string_descriptor_image();
        assert_eq!(descriptor_image.count(), image.literal_count());
        let schema = self.schema;
        let length = schema.reserve_i32_local(function);
        function.program_literal_count(image.literal_count_operand());
        length.store(function);
        let pool = schema
            .reserve_gc_local::<PooledStringTable, NonNullable>(function)
            .initialize(
                schema.array_type::<PooledStringTable>().filled(
                    GcOperand::null(schema),
                    length,
                    function,
                ),
                function,
            );
        schema.release_i32_local(length, function);
        let descriptors =
            ProgramStringDescriptorTable::from_image(schema, &descriptor_image, function);
        let row_index = schema.reserve_i32_local(function);
        row_index.set_constant(0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        row_index.load(function);
        function.program_literal_count(image.literal_count_operand());
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        let row = descriptors.read_row(schema, row_index, function);
        let construction = StringConstruction::from_program_pooled_row(
            schema,
            schema.reserve_gc_local(function),
            &row,
            function,
        );
        let string = schema
            .reserve_gc_local::<StringValue, NonNullable>(function)
            .initialize(construction.publish(schema, function), function);
        schema.array_type::<PooledStringTable>().write(
            &pool,
            row.literal_index(),
            GcOperand::nullable_reference(&string, schema),
            schema,
            function,
        );
        string.clear(function);
        row.release(schema, function);
        row_index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        row_index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        schema.release_i32_local(row_index, function);
        descriptors.clear(function);
        ProgramLiteralRoots::admit_complete_table(schema, image, pool, function)
    }
}
