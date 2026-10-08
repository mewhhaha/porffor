use super::*;

impl FunctionBuilder<'_> {
    pub(in crate::builtins) fn emit_intl_is_unicode_type_i32(
        &self,
        text: &GcLocal<StringValue>,
        ok: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let units = schema
            .reserve_gc_local::<CodeUnitArray, NonNullable>(function)
            .initialize(
                schema
                    .struct_type::<StringValue>()
                    .field(StringValueSchema::CODE_UNITS)
                    .read(text, schema, function)
                    .reference(),
                function,
            );
        let length = schema.reserve_i32_local(function);
        let index = schema.reserve_i32_local(function);
        let run = schema.reserve_i32_local(function);
        let unit = schema.reserve_i32_local(function);
        schema
            .array_type::<CodeUnitArray>()
            .length(&units, schema, function);
        length.store(function);
        set_i32(ok, 1, function);
        set_i32(index, 0, function);
        set_i32(run, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        index.load(function);
        length.load(function);
        function.instruction(&Instruction::I32GeU);
        function.instruction(&Instruction::BrIf(1));
        schema
            .array_type::<CodeUnitArray>()
            .read(&units, index, schema, function)
            .store(unit, function);
        unit.load(function);
        function.instruction(&Instruction::I32Const(b'-' as i32));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        run.load(function);
        function.instruction(&Instruction::I32Const(3));
        function.instruction(&Instruction::I32LtU);
        run.load(function);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_i32(ok, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        set_i32(run, 0, function);
        function.instruction(&Instruction::Else);
        for (low, high) in [(b'0', b'9'), (b'A', b'Z'), (b'a', b'z')] {
            unit.load(function);
            function.instruction(&Instruction::I32Const(low as i32));
            function.instruction(&Instruction::I32GeU);
            unit.load(function);
            function.instruction(&Instruction::I32Const(high as i32));
            function.instruction(&Instruction::I32LeU);
            function.instruction(&Instruction::I32And);
        }
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        set_i32(ok, 0, function);
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        run.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        run.store(function);
        function.instruction(&Instruction::End);
        index.load(function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::I32Add);
        index.store(function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        run.load(function);
        function.instruction(&Instruction::I32Const(3));
        function.instruction(&Instruction::I32LtU);
        run.load(function);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        ok.load(function);
        function.instruction(&Instruction::I32And);
        ok.store(function);
        for local in [unit, run, index, length] {
            schema.release_i32_local(local, function);
        }
        units.clear(function);
    }
}
