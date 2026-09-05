use super::*;

const WEEKDAYS: [[u8; 3]; 7] = [*b"Sun", *b"Mon", *b"Tue", *b"Wed", *b"Thu", *b"Fri", *b"Sat"];
const MONTHS: [[u8; 3]; 12] = [
    *b"Jan", *b"Feb", *b"Mar", *b"Apr", *b"May", *b"Jun", *b"Jul", *b"Aug", *b"Sep", *b"Oct", *b"Nov",
    *b"Dec",
];

impl<'a> FunctionBuilder<'a> {
    /// Parse the two display formats emitted by Lila's existing UTC profile,
    /// for all representable years, rather than recognizing epoch literals.
    pub(super) fn emit_date_parse_display_string(
        &mut self,
        source: u32,
        dest: u32,
        function: &mut Function,
    ) {
        let cursor = DateParseCursor::new(self, source, function);
        let weekday = self.reserve_temp_local();
        let utc_format = self.reserve_temp_local();
        let parts = DateParseComponents::new(self, function);
        cursor.name(self, &WEEKDAYS, weekday, function);
        cursor.at(self, b',', function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(utc_format));
        function.instruction(&Instruction::LocalGet(utc_format));
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
        function.instruction(&Instruction::LocalGet(utc_format));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.expect(self, b"+0000 (Coordinated Universal Time)", function);
        function.instruction(&Instruction::End);
        parts.finish(
            self,
            &cursor,
            DateParseForm::Display { weekday },
            dest,
            function,
        );
        self.release_temp_local(utc_format);
        self.release_temp_local(weekday);
        cursor.release(self);
    }
}
