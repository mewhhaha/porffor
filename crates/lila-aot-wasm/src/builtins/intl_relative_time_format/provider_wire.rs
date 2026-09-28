use super::*;

#[derive(Clone, Copy)]
pub(super) enum RelativeTimeOperation {
    ResolveLocale,
    Format,
}

#[derive(Clone, Copy)]
pub(super) enum RelativeTimeLocaleQueryLocals {
    Supported,
    Resolve {
        numbering_system: u32,
        style: u32,
        numeric: u32,
    },
}

impl RelativeTimeOperation {
    fn host(self) -> IntlHostOp {
        match self {
            Self::ResolveLocale => IntlHostOp::ResolveRelativeTimeLocale,
            Self::Format => IntlHostOp::FormatRelativeTime,
        }
    }

    fn request_tag(self) -> u64 {
        u64::from(self.host().code()) * 2
    }
}

#[derive(Clone, Copy)]
enum WireWord {
    Constant(u64),
    Local(u32),
}

#[derive(Clone, Copy)]
enum WirePass {
    Measure,
    Write,
}

pub(super) struct RelativeTimeResponseReader {
    cursor: u32,
    end: u32,
}

impl RelativeTimeResponseReader {
    pub(super) fn new(
        builder: &mut FunctionBuilder<'_>,
        response: u32,
        operation: RelativeTimeOperation,
        function: &mut Function,
    ) -> Self {
        let cursor = builder.reserve_temp_local();
        let end = builder.reserve_temp_local();
        builder.emit_unpack_string_payload(response, cursor, end, function);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Const(
            RELATIVE_TIME_WIRE_HEADER_BYTES as i64,
        ));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end));
        let reader = Self { cursor, end };
        let version = builder.reserve_temp_local();
        let tag = builder.reserve_temp_local();
        builder.load_i64_to_local_from_offset(cursor, 0, version, function);
        builder.load_i64_to_local_from_offset(cursor, 8, tag, function);
        for (local, expected) in [
            (version, RELATIVE_TIME_WIRE_VERSION),
            (tag, operation.request_tag() + 1),
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
        builder.emit_rtf_advance_wire_cursor(cursor, RELATIVE_TIME_WIRE_HEADER_BYTES, function);
        reader
    }

    fn require_bytes(&self, bytes: u32, function: &mut Function) {
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
    }

    pub(super) fn require_records(&self, count: u32, minimum_bytes: u64, function: &mut Function) {
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
    }

    pub(super) fn word(
        &self,
        builder: &FunctionBuilder<'_>,
        destination: u32,
        function: &mut Function,
    ) {
        self.require_bytes(8, function);
        builder.load_i64_to_local_from_offset(self.cursor, 0, destination, function);
        builder.emit_rtf_advance_wire_cursor(self.cursor, 8, function);
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
    fn emit_rtf_wire_word(&self, word: WireWord, function: &mut Function) {
        match word {
            WireWord::Constant(value) => function.instruction(&Instruction::I64Const(value as i64)),
            WireWord::Local(local) => function.instruction(&Instruction::LocalGet(local)),
        };
    }

    pub(super) fn emit_rtf_advance_wire_cursor(
        &self,
        cursor: u32,
        amount: u64,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I64Const(amount as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor));
    }

    fn emit_rtf_wire_store_word(&self, cursor: u32, word: WireWord, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_rtf_wire_word(word, function);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        self.emit_rtf_advance_wire_cursor(cursor, 8, function);
    }

    fn emit_rtf_wire_copy(&self, source: u32, length: u32, cursor: u32, function: &mut Function) {
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

    fn emit_rtf_wire_bytes(
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
                self.emit_rtf_advance_wire_cursor(cursor, 8, function);
                function.instruction(&Instruction::LocalGet(cursor));
                function.instruction(&Instruction::LocalGet(length));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(cursor));
            }
            WirePass::Write => {
                self.emit_rtf_wire_store_word(cursor, WireWord::Local(length), function);
                self.emit_rtf_wire_copy(offset, length, cursor, function);
            }
        }
        self.release_temp_local(length);
        self.release_temp_local(offset);
    }

    fn emit_rtf_wire_locales(
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
            WirePass::Measure => self.emit_rtf_advance_wire_cursor(cursor, 8, function),
            WirePass::Write => {
                self.emit_rtf_wire_store_word(cursor, WireWord::Local(count), function)
            }
        }
        self.emit_rtf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(array, index, payload, tag, function);
        self.emit_rtf_wire_bytes(payload, cursor, pass, function);
        self.emit_rtf_advance_wire_cursor(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [tag, payload, index, count] {
            self.release_temp_local(local);
        }
    }

    fn emit_rtf_wire_configuration(
        &mut self,
        record: u32,
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        let value = self.reserve_temp_local();
        for offset in [
            HEAP_INTL_RTF_LOCALE_OFFSET,
            HEAP_INTL_RTF_DATA_LOCALE_OFFSET,
            HEAP_INTL_RTF_NUMBERING_SYSTEM_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(record, offset, value, function);
            self.emit_rtf_wire_bytes(value, cursor, pass, function);
        }
        for offset in [HEAP_INTL_RTF_STYLE_OFFSET, HEAP_INTL_RTF_NUMERIC_OFFSET] {
            self.load_i64_to_local_from_offset(record, offset, value, function);
            match pass {
                WirePass::Measure => self.emit_rtf_advance_wire_cursor(cursor, 8, function),
                WirePass::Write => {
                    self.emit_rtf_wire_store_word(cursor, WireWord::Local(value), function)
                }
            }
        }
        self.release_temp_local(value);
    }

    fn emit_rtf_provider_request_fields(
        &mut self,
        operation: RelativeTimeOperation,
        fields: &[WireRequestField],
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        self.emit_rtf_set_const(length, RELATIVE_TIME_WIRE_HEADER_BYTES as i64, function);
        for field in fields {
            self.emit_rtf_field(field, length, WirePass::Measure, function);
        }
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_from_local(length, function)?;
        function.instruction(&Instruction::LocalTee(pointer));
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_rtf_wire_store_word(
            cursor,
            WireWord::Constant(RELATIVE_TIME_WIRE_VERSION),
            function,
        );
        self.emit_rtf_wire_store_word(
            cursor,
            WireWord::Constant(operation.request_tag()),
            function,
        );
        for field in fields {
            self.emit_rtf_field(field, cursor, WirePass::Write, function);
        }
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        for local in [cursor, pointer, length] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    fn emit_rtf_field(
        &mut self,
        field: &WireRequestField,
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        match *field {
            WireRequestField::Word(word) => match pass {
                WirePass::Measure => self.emit_rtf_advance_wire_cursor(cursor, 8, function),
                WirePass::Write => self.emit_rtf_wire_store_word(cursor, word, function),
            },
            WireRequestField::Bytes(payload) => {
                self.emit_rtf_wire_bytes(payload, cursor, pass, function)
            }
            WireRequestField::Locales(array) => {
                self.emit_rtf_wire_locales(array, cursor, pass, function)
            }
            WireRequestField::Configuration(record) => {
                self.emit_rtf_wire_configuration(record, cursor, pass, function)
            }
        }
    }

    pub(super) fn emit_rtf_locale_request(
        &mut self,
        matcher: u32,
        locales: u32,
        query: RelativeTimeLocaleQueryLocals,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match query {
            RelativeTimeLocaleQueryLocals::Supported => self.emit_rtf_provider_request_fields(
                RelativeTimeOperation::ResolveLocale,
                &[
                    WireRequestField::Word(WireWord::Local(matcher)),
                    WireRequestField::Word(WireWord::Constant(2)),
                    WireRequestField::Locales(locales),
                ],
                destination,
                function,
            ),
            RelativeTimeLocaleQueryLocals::Resolve {
                numbering_system,
                style,
                numeric,
            } => self.emit_rtf_provider_request_fields(
                RelativeTimeOperation::ResolveLocale,
                &[
                    WireRequestField::Word(WireWord::Local(matcher)),
                    WireRequestField::Word(WireWord::Constant(1)),
                    WireRequestField::Locales(locales),
                    WireRequestField::Bytes(numbering_system),
                    WireRequestField::Word(WireWord::Local(style)),
                    WireRequestField::Word(WireWord::Local(numeric)),
                ],
                destination,
                function,
            ),
        }
    }

    pub(super) fn emit_rtf_format_request(
        &mut self,
        record: u32,
        unit: u32,
        number_kind: u32,
        number: u32,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_rtf_provider_request_fields(
            RelativeTimeOperation::Format,
            &[
                WireRequestField::Configuration(record),
                WireRequestField::Word(WireWord::Local(unit)),
                WireRequestField::Word(WireWord::Local(number_kind)),
                WireRequestField::Bytes(number),
            ],
            destination,
            function,
        )
    }

    pub(super) fn emit_rtf_provider_call(
        &mut self,
        operation: RelativeTimeOperation,
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
        self.emit_rtf_range_error(RTF_INVALID_OPTION, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(RELATIVE_TIME_WIRE_HEADER_BYTES as u32).wire(),
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
        for (offset, expected) in [
            (0, RELATIVE_TIME_WIRE_VERSION),
            (8, operation.request_tag() + 1),
        ] {
            self.load_i64_to_local_from_offset(pointer, offset, outcome, function);
            function.instruction(&Instruction::LocalGet(outcome));
            function.instruction(&Instruction::I64Const(expected as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(response));
        for local in [pointer, length, outcome] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum WireRequestField {
    Word(WireWord),
    Bytes(u32),
    Locales(u32),
    Configuration(u32),
}
