use super::*;
use lila_intl::{COLLATOR_WIRE_HEADER_BYTES, COLLATOR_WIRE_VERSION};

#[derive(Clone, Copy)]
pub(super) enum CollatorOperation {
    ResolveLocale,
    SupportedLocales,
    Compare,
}

impl CollatorOperation {
    fn host(self) -> IntlHostOp {
        match self {
            Self::ResolveLocale | Self::SupportedLocales => IntlHostOp::ResolveCollatorLocale,
            Self::Compare => IntlHostOp::CompareCollator,
        }
    }
    fn wire_tag(self) -> u64 {
        self.host().code() as u64 * 2
    }
}

#[derive(Clone, Copy)]
pub(super) enum CollatorWireWord {
    Constant(u64),
    Local(u32),
}

pub(super) enum CollatorWireField {
    Word(CollatorWireWord),
    Bytes(u32),
    CanonicalLocales(u32),
    Configuration(u32),
    Utf16(u32),
}

#[derive(Clone, Copy)]
enum WirePass {
    Measure,
    Write,
}

pub(super) struct CollatorResponseReader {
    cursor: u32,
    end: u32,
}

impl CollatorResponseReader {
    pub(super) fn new(
        builder: &mut FunctionBuilder<'_>,
        response: u32,
        operation: CollatorOperation,
        function: &mut Function,
    ) -> Result<Self, EmitError> {
        let cursor = builder.reserve_temp_local();
        let end = builder.reserve_temp_local();
        builder.emit_unpack_string_payload(response, cursor, end, function);
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end));
        let reader = Self { cursor, end };
        let version = builder.reserve_temp_local();
        let tag = builder.reserve_temp_local();
        reader.word(builder, version, function);
        reader.word(builder, tag, function);
        for (local, expected) in [
            (version, COLLATOR_WIRE_VERSION),
            (tag, operation.wire_tag() + 1),
        ] {
            function.instruction(&Instruction::LocalGet(local));
            function.instruction(&Instruction::I64Const(expected as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        builder.release_temp_local(tag);
        builder.release_temp_local(version);
        Ok(reader)
    }

    fn require_bytes(&self, builder: &FunctionBuilder<'_>, bytes: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(bytes as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let _ = builder;
    }

    pub(super) fn require_records(
        &self,
        builder: &FunctionBuilder<'_>,
        count: u32,
        minimum_bytes: u64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(minimum_bytes as i64));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        let _ = builder;
    }

    pub(super) fn word(
        &self,
        builder: &FunctionBuilder<'_>,
        destination: u32,
        function: &mut Function,
    ) {
        self.require_bytes(builder, 8, function);
        builder.load_i64_to_local_from_offset(self.cursor, 0, destination, function);
        builder.emit_col_advance_wire_cursor(self.cursor, 8, function);
    }

    pub(super) fn bytes(
        &self,
        builder: &mut FunctionBuilder<'_>,
        destination: u32,
        function: &mut Function,
    ) {
        let length = builder.reserve_temp_local();
        self.word(builder, length, function);
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        builder.emit_pack_string_payload(self.cursor, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(self.cursor));
        builder.release_temp_local(length);
    }

    pub(super) fn finish(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        builder.release_temp_local(self.end);
        builder.release_temp_local(self.cursor);
    }
}

impl FunctionBuilder<'_> {
    fn emit_col_wire_word(&self, word: CollatorWireWord, function: &mut Function) {
        match word {
            CollatorWireWord::Constant(value) => {
                function.instruction(&Instruction::I64Const(value as i64));
            }
            CollatorWireWord::Local(local) => {
                function.instruction(&Instruction::LocalGet(local));
            }
        }
    }

    pub(super) fn emit_col_advance_wire_cursor(
        &self,
        cursor: u32,
        count: u64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I64Const(count as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor));
    }

    fn emit_col_wire_store_word(
        &self,
        cursor: u32,
        word: CollatorWireWord,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_col_wire_word(word, function);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        self.emit_col_advance_wire_cursor(cursor, 8, function);
    }

    fn emit_col_wire_copy(&self, source: u32, length: u32, cursor: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(source));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: 0,
        });
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor));
    }

    fn emit_col_wire_bytes(
        &mut self,
        payload: u32,
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        self.emit_unpack_string_payload(payload, offset, length, function);
        match pass {
            WirePass::Measure => {
                self.emit_col_advance_wire_cursor(cursor, 8, function);
                function.instruction(&Instruction::LocalGet(cursor));
                function.instruction(&Instruction::LocalGet(length));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(cursor));
            }
            WirePass::Write => {
                self.emit_col_wire_store_word(cursor, CollatorWireWord::Local(length), function);
                self.emit_col_wire_copy(offset, length, cursor, function);
            }
        }
        self.release_temp_local(length);
        self.release_temp_local(offset);
    }

    fn emit_col_wire_utf16(
        &mut self,
        payload: u32,
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        let offset = self.reserve_temp_local();
        let byte_length = self.reserve_temp_local();
        let unit_length = self.reserve_temp_local();
        self.emit_unpack_string_payload(payload, offset, byte_length, function);
        self.emit_utf16_code_unit_len_from_utf8_locals(offset, byte_length, unit_length, function);
        match pass {
            WirePass::Measure => {
                self.emit_col_advance_wire_cursor(cursor, 8, function);
                function.instruction(&Instruction::LocalGet(cursor));
                function.instruction(&Instruction::LocalGet(unit_length));
                function.instruction(&Instruction::I64Const(2));
                function.instruction(&Instruction::I64Mul);
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(cursor));
            }
            WirePass::Write => {
                self.emit_col_wire_store_word(
                    cursor,
                    CollatorWireWord::Local(unit_length),
                    function,
                );
                let byte_index = self.reserve_temp_local();
                let scalar = self.reserve_temp_local();
                let advance = self.reserve_temp_local();
                let first = self.reserve_temp_local();
                let temporary = self.reserve_temp_local();
                self.emit_col_set_const(byte_index, 0, function);
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(byte_index));
                function.instruction(&Instruction::LocalGet(byte_length));
                function.instruction(&Instruction::I64GeU);
                function.instruction(&Instruction::BrIf(1));
                self.emit_load_string_byte(offset, byte_index, first, function);
                self.emit_decode_utf8_scalar_at_index(
                    offset,
                    byte_index,
                    byte_length,
                    first,
                    scalar,
                    advance,
                    temporary,
                    function,
                );
                function.instruction(&Instruction::LocalGet(scalar));
                function.instruction(&Instruction::I64Const(0xFFFF));
                function.instruction(&Instruction::I64GtU);
                function.instruction(&Instruction::If(BlockType::Empty));
                // A supplementary scalar is encoded high surrogate first,
                // then low surrogate. Keep the original low ten bits before
                // replacing scalar with the first code unit.
                function.instruction(&Instruction::LocalGet(scalar));
                function.instruction(&Instruction::I64Const(0x10000));
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::LocalTee(temporary));
                function.instruction(&Instruction::I64Const(10));
                function.instruction(&Instruction::I64ShrU);
                function.instruction(&Instruction::I64Const(0xD800));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(scalar));
                self.emit_col_store_u16_from_local(cursor, scalar, function);
                self.emit_col_advance_wire_cursor(cursor, 2, function);
                function.instruction(&Instruction::LocalGet(temporary));
                function.instruction(&Instruction::I64Const(0x3FF));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Const(0xDC00));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(scalar));
                function.instruction(&Instruction::End);
                self.emit_col_store_u16_from_local(cursor, scalar, function);
                self.emit_col_advance_wire_cursor(cursor, 2, function);
                function.instruction(&Instruction::LocalGet(byte_index));
                function.instruction(&Instruction::LocalGet(advance));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(byte_index));
                function.instruction(&Instruction::Br(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                for local in [temporary, first, advance, scalar, byte_index] {
                    self.release_temp_local(local);
                }
            }
        }
        self.release_temp_local(unit_length);
        self.release_temp_local(byte_length);
        self.release_temp_local(offset);
    }

    fn emit_col_store_u16_from_local(&self, cursor: u32, value: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(value));
        function.instruction(&Instruction::I64Store16(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
    }

    fn emit_col_wire_locales(
        &mut self,
        array: u32,
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array, HEAP_LEN_OFFSET, count, function);
        match pass {
            WirePass::Measure => self.emit_col_advance_wire_cursor(cursor, 8, function),
            WirePass::Write => {
                self.emit_col_wire_store_word(cursor, CollatorWireWord::Local(count), function)
            }
        }
        self.emit_col_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(array, index, payload, tag, function);
        self.emit_col_wire_bytes(payload, cursor, pass, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [tag, payload, index, count] {
            self.release_temp_local(local);
        }
    }

    fn emit_col_wire_configuration(
        &mut self,
        record: u32,
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        let value = self.reserve_temp_local();
        for offset in [
            HEAP_INTL_COL_LOCALE_OFFSET,
            HEAP_INTL_COL_DATA_LOCALE_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(record, offset, value, function);
            self.emit_col_wire_bytes(value, cursor, pass, function);
        }
        self.load_i64_to_local_from_offset(record, HEAP_INTL_COL_USAGE_OFFSET, value, function);
        match pass {
            WirePass::Measure => self.emit_col_advance_wire_cursor(cursor, 8, function),
            WirePass::Write => {
                self.emit_col_wire_store_word(cursor, CollatorWireWord::Local(value), function)
            }
        }
        self.load_i64_to_local_from_offset(record, HEAP_INTL_COL_COLLATION_OFFSET, value, function);
        self.emit_col_wire_bytes(value, cursor, pass, function);
        for offset in [
            HEAP_INTL_COL_NUMERIC_OFFSET,
            HEAP_INTL_COL_CASE_FIRST_OFFSET,
            HEAP_INTL_COL_SENSITIVITY_OFFSET,
            HEAP_INTL_COL_IGNORE_PUNCTUATION_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(record, offset, value, function);
            match pass {
                WirePass::Measure => self.emit_col_advance_wire_cursor(cursor, 8, function),
                WirePass::Write => {
                    self.emit_col_wire_store_word(cursor, CollatorWireWord::Local(value), function)
                }
            }
        }
        self.release_temp_local(value);
    }

    fn emit_col_wire_fields(
        &mut self,
        fields: &[CollatorWireField],
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        for field in fields {
            match *field {
                CollatorWireField::Word(word) => match pass {
                    WirePass::Measure => self.emit_col_advance_wire_cursor(cursor, 8, function),
                    WirePass::Write => self.emit_col_wire_store_word(cursor, word, function),
                },
                CollatorWireField::Bytes(payload) => {
                    self.emit_col_wire_bytes(payload, cursor, pass, function)
                }
                CollatorWireField::CanonicalLocales(array) => {
                    self.emit_col_wire_locales(array, cursor, pass, function)
                }
                CollatorWireField::Configuration(record) => {
                    self.emit_col_wire_configuration(record, cursor, pass, function)
                }
                CollatorWireField::Utf16(payload) => {
                    self.emit_col_wire_utf16(payload, cursor, pass, function)
                }
            }
        }
    }

    pub(super) fn emit_col_provider_request(
        &mut self,
        operation: CollatorOperation,
        fields: &[CollatorWireField],
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        self.emit_col_set_const(length, COLLATOR_WIRE_HEADER_BYTES as i64, function);
        self.emit_col_wire_fields(fields, length, WirePass::Measure, function);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_from_local(length, function)?;
        function.instruction(&Instruction::LocalTee(pointer));
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_col_wire_store_word(
            cursor,
            CollatorWireWord::Constant(COLLATOR_WIRE_VERSION),
            function,
        );
        self.emit_col_wire_store_word(
            cursor,
            CollatorWireWord::Constant(operation.wire_tag()),
            function,
        );
        self.emit_col_wire_fields(fields, cursor, WirePass::Write, function);
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        for local in [cursor, pointer, length] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_col_provider_call(
        &mut self,
        operation: CollatorOperation,
        request: u32,
        response: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let outcome = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let import = self.intl_call_import_function_index()?;
        function.instruction(&Instruction::I64Const(operation.host().wire()));
        function.instruction(&Instruction::LocalGet(request));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalSet(outcome));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(IntlHostCallOutcome::Rejected.wire()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_col_range_error(COLLATOR_INVALID_OPTION, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(COLLATOR_WIRE_HEADER_BYTES as u32).wire(),
        ));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(u32::MAX).wire(),
        ));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(-2));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(length));
        self.emit_heap_alloc_from_local(length, function)?;
        function.instruction(&Instruction::LocalSet(pointer));
        function.instruction(&Instruction::I64Const(operation.host().wire()));
        function.instruction(&Instruction::LocalGet(request));
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(response));
        for local in [pointer, length, outcome] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
