use super::*;
use crate::gc_types::ProgramStringDescriptorTable;

impl FunctionBuilder<'_> {
    pub(super) fn initialize_program_string_roots(&self, function: &mut Function) {
        let image = self.strings.program_string_descriptor_image();
        if image.count() == 0 {
            return;
        }
        let schema = self.schema;
        let descriptors = ProgramStringDescriptorTable::from_image(schema, &image, function);
        let pool = schema
            .reserve_gc_local::<PooledStringTable, NonNullable>(function)
            .initialize(schema.pooled_string_table_reference(function), function);
        let row_index = schema.reserve_i32_local(function);
        row_index.set_constant(0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        row_index.load(function);
        function.instruction(&Instruction::I32Const(descriptors.count() as i32));
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
            row.ordinal(),
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
        pool.clear(function);
        descriptors.clear(function);
    }
}
