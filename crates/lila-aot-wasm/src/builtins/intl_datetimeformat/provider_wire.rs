use super::*;
use lila_intl::{
    IntlHostCallOutcome, IntlHostOp, DATE_TIME_WIRE_HEADER_BYTES, DATE_TIME_WIRE_VERSION,
};

#[derive(Clone, Copy)]
pub(super) enum DtfWireWord {
    Constant(u64),
    Local(u32),
}

pub(super) enum DtfWireField {
    Word(DtfWireWord),
    Bytes(u32),
    CanonicalLocales(u32),
    RecordBody(u32),
    InputRecord(u32),
    TimeZone {
        identifier: u32,
        kind: u32,
        fixed_seconds: u32,
    },
}

#[derive(Clone, Copy)]
enum DtfWirePass {
    Measure,
    Write,
}

/// A response cursor is bounded by the returned span, not merely Wasm memory.
pub(super) struct DtfResponseReader {
    cursor: u32,
    end: u32,
}

impl DtfResponseReader {
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
    pub(super) fn new(
        builder: &mut FunctionBuilder<'_>,
        response: u32,
        function: &mut Function,
    ) -> Self {
        let cursor = builder.reserve_temp_local();
        let end = builder.reserve_temp_local();
        builder.emit_unpack_string_payload(response, cursor, end, function);
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end));
        builder.emit_dtf_advance_wire_cursor(cursor, DATE_TIME_WIRE_HEADER_BYTES, function);
        Self { cursor, end }
    }

    pub(super) fn word(
        &self,
        builder: &FunctionBuilder<'_>,
        destination: u32,
        function: &mut Function,
    ) {
        builder.emit_dtf_require_response_bytes(
            self.cursor,
            self.end,
            DtfWireWord::Constant(8),
            function,
        );
        builder.load_i64_to_local_from_offset(self.cursor, 0, destination, function);
        builder.emit_dtf_advance_wire_cursor(self.cursor, 8, function);
    }

    pub(super) fn bytes(
        &self,
        builder: &mut FunctionBuilder<'_>,
        destination: u32,
        function: &mut Function,
    ) {
        let length = builder.reserve_temp_local();
        self.word(builder, length, function);
        builder.emit_dtf_require_response_bytes(
            self.cursor,
            self.end,
            DtfWireWord::Local(length),
            function,
        );
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
    fn emit_dtf_wire_word(&self, word: DtfWireWord, function: &mut Function) {
        match word {
            DtfWireWord::Constant(value) => {
                function.instruction(&Instruction::I64Const(value as i64));
            }
            DtfWireWord::Local(local) => {
                function.instruction(&Instruction::LocalGet(local));
            }
        }
    }

    fn emit_dtf_advance_wire_cursor(&self, cursor: u32, count: u64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I64Const(count as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor));
    }

    fn emit_dtf_require_response_bytes(
        &self,
        cursor: u32,
        end: u32,
        count: DtfWireWord,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64GtU);
        self.emit_dtf_wire_word(count, function);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    fn emit_dtf_wire_store_word(&self, cursor: u32, word: DtfWireWord, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_dtf_wire_word(word, function);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        self.emit_dtf_advance_wire_cursor(cursor, 8, function);
    }

    fn emit_dtf_wire_copy(&self, source: u32, length: u32, cursor: u32, function: &mut Function) {
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

    fn emit_dtf_wire_bytes(
        &mut self,
        payload: u32,
        cursor: u32,
        pass: DtfWirePass,
        function: &mut Function,
    ) {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        self.emit_unpack_string_payload(payload, offset, length, function);
        match pass {
            DtfWirePass::Measure => {
                self.emit_dtf_advance_wire_cursor(cursor, 8, function);
                function.instruction(&Instruction::LocalGet(cursor));
                function.instruction(&Instruction::LocalGet(length));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(cursor));
            }
            DtfWirePass::Write => {
                self.emit_dtf_wire_store_word(cursor, DtfWireWord::Local(length), function);
                self.emit_dtf_wire_copy(offset, length, cursor, function);
            }
        }
        self.release_temp_local(length);
        self.release_temp_local(offset);
    }

    fn emit_dtf_wire_locales(
        &mut self,
        array: u32,
        cursor: u32,
        pass: DtfWirePass,
        function: &mut Function,
    ) {
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array, HEAP_LEN_OFFSET, count, function);
        match pass {
            DtfWirePass::Measure => self.emit_dtf_advance_wire_cursor(cursor, 8, function),
            DtfWirePass::Write => {
                self.emit_dtf_wire_store_word(cursor, DtfWireWord::Local(count), function)
            }
        }
        self.emit_dtf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(array, index, payload, tag, function);
        self.emit_dtf_wire_bytes(payload, cursor, pass, function);
        self.emit_dtf_advance_wire_cursor(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [tag, payload, index, count] {
            self.release_temp_local(local);
        }
    }

    fn emit_dtf_wire_fields(
        &mut self,
        fields: &[DtfWireField],
        cursor: u32,
        pass: DtfWirePass,
        function: &mut Function,
    ) {
        for field in fields {
            match *field {
                DtfWireField::Word(word) => match pass {
                    DtfWirePass::Measure => self.emit_dtf_advance_wire_cursor(cursor, 8, function),
                    DtfWirePass::Write => self.emit_dtf_wire_store_word(cursor, word, function),
                },
                DtfWireField::Bytes(payload) => {
                    self.emit_dtf_wire_bytes(payload, cursor, pass, function)
                }
                DtfWireField::CanonicalLocales(array) => {
                    self.emit_dtf_wire_locales(array, cursor, pass, function)
                }
                DtfWireField::TimeZone {
                    identifier,
                    kind,
                    fixed_seconds,
                } => {
                    match pass {
                        DtfWirePass::Measure => {
                            self.emit_dtf_advance_wire_cursor(cursor, 8, function)
                        }
                        DtfWirePass::Write => self.emit_dtf_wire_store_word(
                            cursor,
                            DtfWireWord::Local(kind),
                            function,
                        ),
                    }
                    self.emit_dtf_if_code_eq(kind, TimeZoneKind::Named.code(), function);
                    self.emit_dtf_wire_bytes(identifier, cursor, pass, function);
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::LocalGet(kind));
                    function.instruction(&Instruction::I64Const(TimeZoneKind::FixedOffset.code()));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::Unreachable);
                    function.instruction(&Instruction::End);
                    match pass {
                        DtfWirePass::Measure => {
                            self.emit_dtf_advance_wire_cursor(cursor, 8, function)
                        }
                        DtfWirePass::Write => self.emit_dtf_wire_store_word(
                            cursor,
                            DtfWireWord::Local(fixed_seconds),
                            function,
                        ),
                    }
                    function.instruction(&Instruction::End);
                }
                DtfWireField::RecordBody(payload) => {
                    let offset = self.reserve_temp_local();
                    let length = self.reserve_temp_local();
                    self.emit_unpack_string_payload(payload, offset, length, function);
                    self.emit_dtf_advance_wire_cursor(
                        offset,
                        DATE_TIME_WIRE_HEADER_BYTES,
                        function,
                    );
                    function.instruction(&Instruction::LocalGet(length));
                    function
                        .instruction(&Instruction::I64Const(DATE_TIME_WIRE_HEADER_BYTES as i64));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(length));
                    match pass {
                        DtfWirePass::Measure => {
                            function.instruction(&Instruction::LocalGet(cursor));
                            function.instruction(&Instruction::LocalGet(length));
                            function.instruction(&Instruction::I64Add);
                            function.instruction(&Instruction::LocalSet(cursor));
                        }
                        DtfWirePass::Write => {
                            self.emit_dtf_wire_copy(offset, length, cursor, function)
                        }
                    }
                    self.release_temp_local(length);
                    self.release_temp_local(offset);
                }
                DtfWireField::InputRecord(pointer) => match pass {
                    DtfWirePass::Measure => self.emit_dtf_advance_wire_cursor(
                        cursor,
                        lila_intl::DATE_TIME_INPUT_BYTES,
                        function,
                    ),
                    DtfWirePass::Write => {
                        let length = self.reserve_temp_local();
                        self.emit_dtf_set_const(
                            length,
                            lila_intl::DATE_TIME_INPUT_BYTES as i64,
                            function,
                        );
                        self.emit_dtf_wire_copy(pointer, length, cursor, function);
                        self.release_temp_local(length);
                    }
                },
            }
        }
    }

    pub(super) fn emit_dtf_provider_request(
        &mut self,
        operation: IntlHostOp,
        fields: &[DtfWireField],
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        self.emit_dtf_set_const(length, DATE_TIME_WIRE_HEADER_BYTES as i64, function);
        self.emit_dtf_wire_fields(fields, length, DtfWirePass::Measure, function);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_from_local(length, function)?;
        function.instruction(&Instruction::LocalTee(pointer));
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_dtf_wire_store_word(
            cursor,
            DtfWireWord::Constant(DATE_TIME_WIRE_VERSION),
            function,
        );
        self.emit_dtf_wire_store_word(
            cursor,
            DtfWireWord::Constant(u64::from(operation.code()) * 2),
            function,
        );
        self.emit_dtf_wire_fields(fields, cursor, DtfWirePass::Write, function);
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        for local in [cursor, pointer, length] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_dtf_provider_call(
        &mut self,
        operation: IntlHostOp,
        request: u32,
        response: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let outcome = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let header = self.reserve_temp_local();
        let import = self.intl_call_import_function_index()?;
        function.instruction(&Instruction::I64Const(operation.wire()));
        function.instruction(&Instruction::LocalGet(request));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalSet(outcome));
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(IntlHostCallOutcome::Rejected.wire()));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            INTL_DTF_EMPTY_TEMPORAL_FORMAT,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(DATE_TIME_WIRE_HEADER_BYTES as u32).wire(),
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
        function.instruction(&Instruction::I64Const(operation.wire()));
        function.instruction(&Instruction::LocalGet(request));
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::Call(import));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        for (offset, expected) in [
            (0, DATE_TIME_WIRE_VERSION),
            (8, u64::from(operation.code()) * 2 + 1),
        ] {
            self.load_i64_to_local_from_offset(pointer, offset, header, function);
            function.instruction(&Instruction::LocalGet(header));
            function.instruction(&Instruction::I64Const(expected as i64));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(response));
        for local in [header, pointer, length, outcome] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
