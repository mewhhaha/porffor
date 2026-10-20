use super::*;

impl<'a> FunctionBuilder<'a> {
    /// A deliberately separate implementation-defined grammar. Only the ASCII
    /// space separator admits short fields and compact/hour-only offsets;
    /// malformed T syntax never acquires these permissions on fallback.
    pub(super) fn emit_date_parse_legacy_space_string(
        &mut self,
        source: &GcLocal<StringValue>,
        dest: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let cursor = DateParseCursor::new(self, source, function);
        let parts = DateParseComponents::new(self, function);
        let negative = self.runtime_schema().reserve_i64_local(function);
        cursor.at(self, b'-', function);
        function.instruction(&Instruction::I64ExtendI32U);
        negative.store(function);
        cursor.at(self, b'+', function);
        negative.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 6, parts.year, function);
        negative.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        parts.year.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ne);
        cursor.require(function);
        parts.year.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        parts.year.store(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        cursor.decimal(self, 4, parts.year, function);
        function.instruction(&Instruction::End);

        cursor.expect(self, b"-", function);
        cursor.decimal_one_or_two(self, parts.month, function);
        parts.month.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        parts.month.store(function);
        cursor.expect(self, b"-", function);
        cursor.decimal_one_or_two(self, parts.date, function);
        cursor.expect(self, b" ", function);
        parts.use_local_time(function);
        cursor.decimal_one_or_two(self, parts.hour, function);
        cursor.expect(self, b":", function);
        cursor.decimal_one_or_two(self, parts.minute, function);
        cursor.at(self, b':', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal_one_or_two(self, parts.second, function);
        cursor.at(self, b'.', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        // Keep the existing three-complete-millisecond-digit policy. This
        // fallback does not broaden either route's fractional-second grammar.
        cursor.decimal(self, 3, parts.millisecond, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        cursor.at(self, b'Z', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        parts.use_utc_time(function);
        cursor.advance(function);
        function.instruction(&Instruction::Else);
        cursor.at(self, b'-', function);
        function.instruction(&Instruction::I64ExtendI32U);
        negative.store(function);
        cursor.at(self, b'+', function);
        negative.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        parts.use_utc_time(function);
        cursor.advance(function);
        let offset_hour = self.runtime_schema().reserve_i64_local(function);
        let offset_minute = self.runtime_schema().reserve_i64_local(function);
        cursor.decimal(self, 2, offset_hour, function);
        function.instruction(&Instruction::I64Const(0));
        offset_minute.store(function);
        cursor.at(self, b':', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 2, offset_minute, function);
        function.instruction(&Instruction::Else);
        cursor.at_decimal_digit(self, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.decimal(self, 2, offset_minute, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for (local, upper_bound) in [(offset_hour, 23.0), (offset_minute, 59.0)] {
            local.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(upper_bound)));
            function.instruction(&Instruction::F64Le);
            cursor.require(function);
        }
        offset_hour.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(60.0)));
        function.instruction(&Instruction::F64Mul);
        offset_minute.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::F64Const(Ieee64::from(60_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        parts.offset.store(function);
        negative.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        parts.offset.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        parts.offset.store(function);
        function.instruction(&Instruction::End);
        self.runtime_schema()
            .release_i64_local(offset_minute, function);
        self.runtime_schema()
            .release_i64_local(offset_hour, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.runtime_schema().release_i64_local(negative, function);
        parts.finish(
            self,
            &cursor,
            DateParseForm::LegacySpaceDateTime,
            dest,
            function,
        )?;
        cursor.release(self, function);
        Ok(())
    }
}
