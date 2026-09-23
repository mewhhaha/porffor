use super::numeric_input::NfNumericLocals;
use super::*;
use lila_intl::{IntlHostCallOutcome, IntlHostOp};
use lila_intl::{NUMBER_WIRE_HEADER_BYTES, NUMBER_WIRE_VERSION};

#[derive(Clone, Copy)]
pub(super) enum NfOperation {
    ResolveLocale,
    SupportedLocales,
    ScalarParts,
    RangeParts,
}
impl NfOperation {
    fn host(self) -> IntlHostOp {
        match self {
            Self::ResolveLocale => IntlHostOp::ResolveNumberLocale,
            Self::SupportedLocales => IntlHostOp::SupportedNumberLocales,
            Self::ScalarParts => IntlHostOp::FormatNumberParts,
            Self::RangeParts => IntlHostOp::FormatNumberRangeParts,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum NfWireWord {
    Constant(u64),
    Local(u32),
}

pub(super) enum NfWireField<'a> {
    Word(NfWireWord),
    Bytes(u32),
    CanonicalLocales(u32),
    Configuration(u32),
    Numeric(&'a NfNumericLocals),
}

#[derive(Clone, Copy)]
enum NfWirePass {
    Measure,
    Write,
}

/// A response cursor is bounded by the returned span, not merely Wasm memory.
pub(super) struct NfResponseReader {
    cursor: u32,
    end: u32,
}

impl NfResponseReader {
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
        builder.emit_nf_advance_wire_cursor(cursor, NUMBER_WIRE_HEADER_BYTES, function);
        Self { cursor, end }
    }

    pub(super) fn word(
        &self,
        builder: &FunctionBuilder<'_>,
        destination: u32,
        function: &mut Function,
    ) {
        builder.emit_nf_require_response_bytes(
            self.cursor,
            self.end,
            NfWireWord::Constant(8),
            function,
        );
        builder.load_i64_to_local_from_offset(self.cursor, 0, destination, function);
        builder.emit_nf_advance_wire_cursor(self.cursor, 8, function);
    }

    pub(super) fn bytes(
        &self,
        builder: &mut FunctionBuilder<'_>,
        destination: u32,
        function: &mut Function,
    ) {
        let length = builder.reserve_temp_local();
        self.word(builder, length, function);
        builder.emit_nf_require_response_bytes(
            self.cursor,
            self.end,
            NfWireWord::Local(length),
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
    fn emit_nf_wire_word(&self, word: NfWireWord, function: &mut Function) {
        match word {
            NfWireWord::Constant(value) => {
                function.instruction(&Instruction::I64Const(value as i64));
            }
            NfWireWord::Local(local) => {
                function.instruction(&Instruction::LocalGet(local));
            }
        }
    }

    pub(super) fn emit_nf_advance_wire_cursor(
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

    fn emit_nf_require_response_bytes(
        &self,
        cursor: u32,
        end: u32,
        count: NfWireWord,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64GtU);
        self.emit_nf_wire_word(count, function);
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    fn emit_nf_wire_store_word(&self, cursor: u32, word: NfWireWord, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_nf_wire_word(word, function);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        self.emit_nf_advance_wire_cursor(cursor, 8, function);
    }

    fn emit_nf_wire_copy(&self, source: u32, length: u32, cursor: u32, function: &mut Function) {
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

    fn emit_nf_wire_bytes(
        &mut self,
        payload: u32,
        cursor: u32,
        pass: NfWirePass,
        function: &mut Function,
    ) {
        let offset = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        self.emit_unpack_string_payload(payload, offset, length, function);
        match pass {
            NfWirePass::Measure => {
                self.emit_nf_advance_wire_cursor(cursor, 8, function);
                function.instruction(&Instruction::LocalGet(cursor));
                function.instruction(&Instruction::LocalGet(length));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(cursor));
            }
            NfWirePass::Write => {
                self.emit_nf_wire_store_word(cursor, NfWireWord::Local(length), function);
                self.emit_nf_wire_copy(offset, length, cursor, function);
            }
        }
        self.release_temp_local(length);
        self.release_temp_local(offset);
    }

    fn emit_nf_wire_locales(
        &mut self,
        array: u32,
        cursor: u32,
        pass: NfWirePass,
        function: &mut Function,
    ) {
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array, HEAP_LEN_OFFSET, count, function);
        match pass {
            NfWirePass::Measure => self.emit_nf_advance_wire_cursor(cursor, 8, function),
            NfWirePass::Write => {
                self.emit_nf_wire_store_word(cursor, NfWireWord::Local(count), function)
            }
        }
        self.emit_nf_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(array, index, payload, tag, function);
        self.emit_nf_wire_bytes(payload, cursor, pass, function);
        self.emit_nf_advance_wire_cursor(index, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [tag, payload, index, count] {
            self.release_temp_local(local);
        }
    }

    fn emit_nf_wire_fields(
        &mut self,
        fields: &[NfWireField<'_>],
        cursor: u32,
        pass: NfWirePass,
        function: &mut Function,
    ) {
        for field in fields {
            match *field {
                NfWireField::Word(word) => match pass {
                    NfWirePass::Measure => self.emit_nf_advance_wire_cursor(cursor, 8, function),
                    NfWirePass::Write => self.emit_nf_wire_store_word(cursor, word, function),
                },
                NfWireField::Bytes(payload) => {
                    self.emit_nf_wire_bytes(payload, cursor, pass, function)
                }
                NfWireField::CanonicalLocales(array) => {
                    self.emit_nf_wire_locales(array, cursor, pass, function)
                }
                NfWireField::Numeric(value) => {
                    match pass {
                        NfWirePass::Measure => {
                            self.emit_nf_advance_wire_cursor(cursor, 8, function)
                        }
                        NfWirePass::Write => self.emit_nf_wire_store_word(
                            cursor,
                            NfWireWord::Local(value.kind),
                            function,
                        ),
                    }
                    self.emit_nf_wire_bytes(value.bytes, cursor, pass, function);
                }
                NfWireField::Configuration(record) => {
                    let value = self.reserve_temp_local();
                    for offset in [
                        HEAP_INTL_NF_LOCALE_OFFSET,
                        HEAP_INTL_NF_DATA_LOCALE_OFFSET,
                        HEAP_INTL_NF_NUMBERING_SYSTEM_OFFSET,
                    ] {
                        self.load_i64_to_local_from_offset(record, offset, value, function);
                        self.emit_nf_wire_bytes(value, cursor, pass, function);
                    }
                    for word in NfWord::ALL {
                        self.emit_nf_load_word(record, word, value, function);
                        match pass {
                            NfWirePass::Measure => {
                                self.emit_nf_advance_wire_cursor(cursor, 8, function)
                            }
                            NfWirePass::Write => self.emit_nf_wire_store_word(
                                cursor,
                                NfWireWord::Local(value),
                                function,
                            ),
                        }
                    }
                    self.load_i64_to_local_from_offset(
                        record,
                        HEAP_INTL_NF_STYLE_TEXT_OFFSET,
                        value,
                        function,
                    );
                    self.emit_nf_wire_bytes(value, cursor, pass, function);
                    self.release_temp_local(value);
                }
            }
        }
    }

    pub(super) fn emit_nf_provider_request(
        &mut self,
        operation: NfOperation,
        fields: &[NfWireField<'_>],
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        self.emit_nf_set_const(length, NUMBER_WIRE_HEADER_BYTES as i64, function);
        self.emit_nf_wire_fields(fields, length, NfWirePass::Measure, function);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_from_local(length, function)?;
        function.instruction(&Instruction::LocalTee(pointer));
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_nf_wire_store_word(cursor, NfWireWord::Constant(NUMBER_WIRE_VERSION), function);
        self.emit_nf_wire_store_word(
            cursor,
            NfWireWord::Constant(u64::from(operation.host().code()) * 2),
            function,
        );
        self.emit_nf_wire_fields(fields, cursor, NfWirePass::Write, function);
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        for local in [cursor, pointer, length] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_nf_provider_call(
        &mut self,
        operation: NfOperation,
        request: u32,
        response: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let outcome = self.reserve_temp_local();
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let header = self.reserve_temp_local();
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
        match operation {
            NfOperation::RangeParts => self.emit_nf_range_error(NF_RANGE_NAN, function)?,
            NfOperation::ResolveLocale
            | NfOperation::SupportedLocales
            | NfOperation::ScalarParts => {
                function.instruction(&Instruction::Unreachable);
            }
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(NUMBER_WIRE_HEADER_BYTES as u32).wire(),
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
            (0, NUMBER_WIRE_VERSION),
            (8, u64::from(operation.host().code()) * 2 + 1),
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
