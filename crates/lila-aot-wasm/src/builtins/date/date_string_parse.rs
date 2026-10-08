use super::*;

mod components;
mod cursor;
mod display;

use components::{DateParseComponents, DateParseForm};
use cursor::DateParseCursor;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_date_parse_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let bits = schema.reserve_i64_local(function);
        self.emit_builtin_arg_to_value(0, &argument, function);
        self.emit_value_to_string_payload(&argument, &pending, function)?;
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let string = schema.reserve_gc_local(function).initialize(
            pending
                .value()
                .cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_date_parse_string(&string, bits, function)?;
        argument.set_number(bits, function);
        pending.set_normal(&argument, function);
        string.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&pending, function);
        schema.release_i64_local(bits, function);
        pending.clear(function);
        argument.clear(function);
        Ok(())
    }
    pub(crate) fn emit_date_parse_iso_string(
        &mut self,
        string_payload_local: &GcLocal<StringValue>,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let cursor = DateParseCursor::new(self, string_payload_local, function);
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

        // A time suffix may follow any of YYYY, YYYY-MM, or YYYY-MM-DD.
        // Do not consume a missing month/day merely because more input exists.
        cursor.at(self, b'-', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 2, parts.month, function);
        parts.month.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::I64ReinterpretF64);
        parts.month.store(function);
        cursor.at(self, b'-', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        cursor.advance(function);
        cursor.decimal(self, 2, parts.date, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        cursor.at(self, b'T', function);
        function.instruction(&Instruction::If(BlockType::Empty));
        parts.use_local_time(function);
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
        cursor.expect(self, b":", function);
        cursor.decimal(self, 2, offset_minute, function);
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
        function.instruction(&Instruction::End);

        self.runtime_schema().release_i64_local(negative, function);
        parts.finish(
            self,
            &cursor,
            DateParseForm::Iso,
            dest_payload_local,
            function,
        )?;
        cursor.release(self, function);
        Ok(())
    }

    pub(crate) fn emit_date_parse_string(
        &mut self,
        string_payload_local: &GcLocal<StringValue>,
        dest_payload_local: I64Local,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // The original immutable GC String remains rooted across both parse attempts.
        let source = string_payload_local;
        self.emit_date_parse_iso_string(source, dest_payload_local, function)?;
        dest_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        dest_payload_local.load(function);
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Display syntax starts with a weekday, so it cannot reinterpret an
        // out-of-range ISO expanded year through a permissive fallback.
        self.emit_date_parse_display_string(source, dest_payload_local, function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }
}
