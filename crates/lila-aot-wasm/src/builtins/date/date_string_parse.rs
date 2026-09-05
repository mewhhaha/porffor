use super::*;

mod components;
mod cursor;
mod display;

use components::{DateParseComponents, DateParseForm};
use cursor::DateParseCursor;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_date_parse_iso_string(
        &mut self,
        string_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        let cursor = DateParseCursor::new(self, string_payload_local, function);
        let parts = DateParseComponents::new(self, function);
        let negative = self.reserve_temp_local();
        cursor.at(self, b'-', function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative));
        cursor.at(self, b'+', function);
        function.instruction(&Instruction::LocalGet(negative));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 6, parts.year, function);
        function.instruction(&Instruction::LocalGet(negative));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(parts.year));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ne);
        cursor.require(function);
        function.instruction(&Instruction::LocalGet(parts.year));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(parts.year));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        cursor.decimal(self, 4, parts.year, function);
        function.instruction(&Instruction::End);

        // A time suffix may follow any of YYYY, YYYY-MM, or YYYY-MM-DD.
        // Do not consume a missing month/day merely because more input exists.
        cursor.at(self, b'-', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 2, parts.month, function);
        function.instruction(&Instruction::LocalGet(parts.month));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(parts.month));
        cursor.at(self, b'-', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 2, parts.date, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        cursor.at(self, b'T', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 2, parts.hour, function);
        cursor.expect(self, b":", function);
        cursor.decimal(self, 2, parts.minute, function);
        cursor.at(self, b':', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 2, parts.second, function);
        cursor.at(self, b'.', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 3, parts.millisecond, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        cursor.at(self, b'Z', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        function.instruction(&Instruction::Else);
        cursor.at(self, b'-', function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(negative));
        cursor.at(self, b'+', function);
        function.instruction(&Instruction::LocalGet(negative));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        let offset_hour = self.reserve_temp_local();
        let offset_minute = self.reserve_temp_local();
        cursor.decimal(self, 2, offset_hour, function);
        cursor.expect(self, b":", function);
        cursor.decimal(self, 2, offset_minute, function);
        for (local, upper_bound) in [(offset_hour, 23.0), (offset_minute, 59.0)] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::F64ReinterpretI64);
            function.instruction(&Instruction::F64Const(Ieee64::from(upper_bound)));
            function.instruction(&Instruction::F64Le);
            cursor.require(function);
        }
        function.instruction(&Instruction::LocalGet(offset_hour));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(60.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::LocalGet(offset_minute));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Add);
        function.instruction(&Instruction::F64Const(Ieee64::from(60_000.0)));
        function.instruction(&Instruction::F64Mul);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(parts.offset));
        function.instruction(&Instruction::LocalGet(negative));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(parts.offset));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Neg);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(parts.offset));
        function.instruction(&Instruction::End);
        self.release_temp_local(offset_minute);
        self.release_temp_local(offset_hour);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(negative);
        parts.finish(
            self,
            &cursor,
            DateParseForm::Iso,
            dest_payload_local,
            function,
        );
        cursor.release(self);
    }

    pub(crate) fn emit_date_parse_string(
        &mut self,
        string_payload_local: u32,
        dest_payload_local: u32,
        function: &mut Function,
    ) {
        // The caller may reuse its input local as the destination. Preserve the
        // original string across the ISO attempt before trying display syntax.
        let source = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(string_payload_local));
        function.instruction(&Instruction::LocalSet(source));
        self.emit_date_parse_iso_string(source, dest_payload_local, function);
        function.instruction(&Instruction::LocalGet(dest_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(dest_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Display syntax starts with a weekday, so it cannot reinterpret an
        // out-of-range ISO expanded year through a permissive fallback.
        self.emit_date_parse_display_string(source, dest_payload_local, function);
        function.instruction(&Instruction::End);
        self.release_temp_local(source);
    }
}
