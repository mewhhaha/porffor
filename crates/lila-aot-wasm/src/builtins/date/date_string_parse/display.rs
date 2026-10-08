use super::*;

const WEEKDAYS: [[u8; 3]; 7] = [
    *b"Sun", *b"Mon", *b"Tue", *b"Wed", *b"Thu", *b"Fri", *b"Sat",
];
const MONTHS: [[u8; 3]; 12] = [
    *b"Jan", *b"Feb", *b"Mar", *b"Apr", *b"May", *b"Jun", *b"Jul", *b"Aug", *b"Sep", *b"Oct",
    *b"Nov", *b"Dec",
];

#[derive(Clone, Copy)]
enum DisplayOffsetForm {
    Compact,
    ExactSeconds,
}

struct DisplayOffsetLocals {
    negative: I64Local,
    hour: I64Local,
    minute: I64Local,
    second: I64Local,
}

impl DisplayOffsetLocals {
    fn read(
        builder: &mut FunctionBuilder<'_>,
        cursor: &DateParseCursor,
        form: DisplayOffsetForm,
        function: &mut Function,
    ) -> Self {
        let offset = Self {
            negative: builder.runtime_schema().reserve_i64_local(function),
            hour: builder.runtime_schema().reserve_i64_local(function),
            minute: builder.runtime_schema().reserve_i64_local(function),
            second: builder.runtime_schema().reserve_i64_local(function),
        };
        cursor.at(builder, b'-', function);
        function.instruction(&Instruction::I64ExtendI32U);
        offset.negative.store(function);
        cursor.at(builder, b'+', function);
        offset.negative.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        cursor.require(function);
        cursor.advance(function);
        cursor.decimal(builder, 2, offset.hour, function);
        match form {
            DisplayOffsetForm::Compact => {}
            DisplayOffsetForm::ExactSeconds => cursor.expect(builder, b":", function),
        }
        cursor.decimal(builder, 2, offset.minute, function);
        match form {
            DisplayOffsetForm::Compact => {
                function.instruction(&Instruction::I64Const(0));
                offset.second.store(function);
            }
            DisplayOffsetForm::ExactSeconds => {
                cursor.expect(builder, b":", function);
                cursor.decimal(builder, 2, offset.second, function);
            }
        }
        // Admitted offsets are strictly within one day. The owned display
        // preserves whole seconds; ISO syntax separately has minute precision.
        for (local, maximum) in [
            (offset.hour, 23.0),
            (offset.minute, 59.0),
            (offset.second, 59.0),
        ] {
            local.load(function);
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(maximum)));
            function.instruction(&Instruction::F64Le);
            cursor.require(function);
        }
        offset
    }

    fn store_milliseconds(&self, dest: I64Local, function: &mut Function) {
        self.hour.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(3600.0)));
        function.instruction(&Instruction::F64Mul);
        self.minute.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(60.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::F64Add);
        self.second.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::F64Const(Ieee64::from(1000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest.store(function);
        self.negative.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        dest.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        dest.store(function);
        function.instruction(&Instruction::End);
    }

    fn require_compact_agreement(
        &self,
        compact: &Self,
        cursor: &DateParseCursor,
        function: &mut Function,
    ) {
        for (exact, shown) in [
            (self.negative, compact.negative),
            (self.hour, compact.hour),
            (self.minute, compact.minute),
        ] {
            exact.load(function);
            shown.load(function);
            function.instruction(&Instruction::I64Eq);
            cursor.require(function);
        }
    }

    fn release(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        for local in [self.second, self.minute, self.hour, self.negative] {
            builder.runtime_schema().release_i64_local(local, function);
        }
    }
}

impl<'a> FunctionBuilder<'a> {
    /// Parse Lila's UTC and configured-zone display formats across the full
    /// Date range. Exact offset seconds in the owned parenthetical suffix
    /// preserve historical offsets that the specified GMT prefix truncates.
    pub(super) fn emit_date_parse_display_string(
        &mut self,
        source: &GcLocal<StringValue>,
        dest: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let cursor = DateParseCursor::new(self, source, function);
        let weekday = self.runtime_schema().reserve_i64_local(function);
        let utc_format = self.runtime_schema().reserve_i64_local(function);
        let parts = DateParseComponents::new(self, function);
        cursor.name(self, &WEEKDAYS, weekday, function);
        cursor.at(self, b',', function);
        function.instruction(&Instruction::I64ExtendI32U);
        utc_format.store(function);
        utc_format.load(function);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.expect(self, b", ", function);
        cursor.decimal(self, 2, parts.date, function);
        cursor.expect(self, b" ", function);
        cursor.name(self, &MONTHS, parts.month, function);
        function.instruction(&Instruction::Else);
        cursor.expect(self, b" ", function);
        cursor.name(self, &MONTHS, parts.month, function);
        cursor.expect(self, b" ", function);
        cursor.decimal(self, 2, parts.date, function);
        function.instruction(&Instruction::End);
        cursor.expect(self, b" ", function);
        cursor.display_year(self, parts.year, function);
        cursor.expect(self, b" ", function);
        cursor.decimal(self, 2, parts.hour, function);
        cursor.expect(self, b":", function);
        cursor.decimal(self, 2, parts.minute, function);
        cursor.expect(self, b":", function);
        cursor.decimal(self, 2, parts.second, function);
        cursor.expect(self, b" GMT", function);
        utc_format.load(function);
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        let shown = DisplayOffsetLocals::read(self, &cursor, DisplayOffsetForm::Compact, function);
        cursor.expect(self, b" (", function);
        cursor.at_bytes(self, b"Coordinated Universal Time)", function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.expect(self, b"Coordinated Universal Time)", function);
        for local in [shown.negative, shown.hour, shown.minute] {
            local.load(function);
            function.instruction(&Instruction::I64Eqz);
            cursor.require(function);
        }
        function.instruction(&Instruction::Else);
        cursor.display_time_zone_name(self, function);
        cursor.expect(self, b", ", function);
        let exact =
            DisplayOffsetLocals::read(self, &cursor, DisplayOffsetForm::ExactSeconds, function);
        exact.require_compact_agreement(&shown, &cursor, function);
        exact.store_milliseconds(parts.offset, function);
        cursor.expect(self, b")", function);
        exact.release(self, function);
        function.instruction(&Instruction::End);
        shown.release(self, function);
        function.instruction(&Instruction::End);
        parts.finish(
            self,
            &cursor,
            DateParseForm::Display { weekday },
            dest,
            function,
        )?;
        self.runtime_schema()
            .release_i64_local(utc_format, function);
        self.runtime_schema().release_i64_local(weekday, function);
        cursor.release(self, function);
        Ok(())
    }
}
