use super::super::intl_numberformat::NfOptionsLocals;
use super::*;
use lila_intl::{
    NUMBER_CONFIGURATION_WORDS, PLURAL_RULES_WIRE_HEADER_BYTES, PLURAL_RULES_WIRE_VERSION,
};

#[derive(Clone, Copy)]
pub(super) enum PluralRulesOperation {
    ResolveLocale,
    SelectCategory,
}

impl PluralRulesOperation {
    fn host(self) -> IntlHostOp {
        match self {
            Self::ResolveLocale => IntlHostOp::ResolvePluralRulesLocale,
            Self::SelectCategory => IntlHostOp::SelectPluralCategory,
        }
    }

    fn wire_tag(self) -> u64 {
        u64::from(self.host().code()) * 2
    }
}

#[derive(Clone, Copy)]
pub(super) enum PluralRulesWireWord {
    Constant(u64),
    Local(u32),
}

pub(super) enum PluralRulesWireField<'a> {
    Word(PluralRulesWireWord),
    CanonicalLocales(u32),
    RuleOptions {
        rule_type: u32,
        options: &'a NfOptionsLocals,
    },
    Configuration(u32),
    NumberInput {
        kind: u32,
        bytes: u32,
    },
}

#[derive(Clone, Copy)]
enum WirePass {
    Measure,
    Write,
}

pub(super) struct PluralRulesResponseReader {
    cursor: u32,
    end: u32,
}

impl PluralRulesResponseReader {
    pub(super) fn new(
        builder: &mut FunctionBuilder<'_>,
        response: u32,
        operation: PluralRulesOperation,
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
            (version, PLURAL_RULES_WIRE_VERSION),
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
        builder.emit_pr_advance_wire_cursor(self.cursor, 8, function);
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
    fn emit_pr_wire_word(&self, word: PluralRulesWireWord, function: &mut Function) {
        match word {
            PluralRulesWireWord::Constant(value) => {
                function.instruction(&Instruction::I64Const(value as i64));
            }
            PluralRulesWireWord::Local(local) => {
                function.instruction(&Instruction::LocalGet(local));
            }
        }
    }

    pub(super) fn emit_pr_advance_wire_cursor(
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

    fn emit_pr_wire_store_word(
        &self,
        cursor: u32,
        word: PluralRulesWireWord,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        self.emit_pr_wire_word(word, function);
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        self.emit_pr_advance_wire_cursor(cursor, 8, function);
    }

    fn emit_pr_wire_copy(&self, source: u32, length: u32, cursor: u32, function: &mut Function) {
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

    fn emit_pr_wire_bytes(
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
                self.emit_pr_advance_wire_cursor(cursor, 8, function);
                function.instruction(&Instruction::LocalGet(cursor));
                function.instruction(&Instruction::LocalGet(length));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(cursor));
            }
            WirePass::Write => {
                self.emit_pr_wire_store_word(cursor, PluralRulesWireWord::Local(length), function);
                self.emit_pr_wire_copy(offset, length, cursor, function);
            }
        }
        self.release_temp_local(length);
        self.release_temp_local(offset);
    }

    fn emit_pr_wire_locales(
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
            WirePass::Measure => self.emit_pr_advance_wire_cursor(cursor, 8, function),
            WirePass::Write => {
                self.emit_pr_wire_store_word(cursor, PluralRulesWireWord::Local(count), function)
            }
        }
        self.emit_pr_set_const(index, 0, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(array, index, payload, tag, function);
        self.emit_pr_wire_bytes(payload, cursor, pass, function);
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

    fn emit_pr_wire_configuration(
        &mut self,
        record: u32,
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        let value = self.reserve_temp_local();
        for offset in [HEAP_INTL_PR_LOCALE_OFFSET, HEAP_INTL_PR_DATA_LOCALE_OFFSET] {
            self.load_i64_to_local_from_offset(record, offset, value, function);
            self.emit_pr_wire_bytes(value, cursor, pass, function);
        }
        for offset in [
            HEAP_INTL_PR_TYPE_OFFSET,
            HEAP_INTL_PR_CATEGORY_MASK_OFFSET,
            HEAP_INTL_PR_CATEGORY_COUNT_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(record, offset, value, function);
            match pass {
                WirePass::Measure => self.emit_pr_advance_wire_cursor(cursor, 8, function),
                WirePass::Write => self.emit_pr_wire_store_word(
                    cursor,
                    PluralRulesWireWord::Local(value),
                    function,
                ),
            }
        }
        for index in 0..NUMBER_CONFIGURATION_WORDS {
            self.load_i64_to_local_from_offset(
                record,
                HEAP_INTL_PR_WORDS_OFFSET + (index as u64 * 8),
                value,
                function,
            );
            match pass {
                WirePass::Measure => self.emit_pr_advance_wire_cursor(cursor, 8, function),
                WirePass::Write => self.emit_pr_wire_store_word(
                    cursor,
                    PluralRulesWireWord::Local(value),
                    function,
                ),
            }
        }
        self.release_temp_local(value);
    }

    fn emit_pr_wire_fields(
        &mut self,
        fields: &[PluralRulesWireField<'_>],
        cursor: u32,
        pass: WirePass,
        function: &mut Function,
    ) {
        for field in fields {
            match field {
                PluralRulesWireField::Word(word) => match pass {
                    WirePass::Measure => self.emit_pr_advance_wire_cursor(cursor, 8, function),
                    WirePass::Write => self.emit_pr_wire_store_word(cursor, *word, function),
                },
                PluralRulesWireField::CanonicalLocales(array) => {
                    self.emit_pr_wire_locales(*array, cursor, pass, function)
                }
                PluralRulesWireField::RuleOptions { rule_type, options } => {
                    match pass {
                        WirePass::Measure => self.emit_pr_advance_wire_cursor(cursor, 8, function),
                        WirePass::Write => self.emit_pr_wire_store_word(
                            cursor,
                            PluralRulesWireWord::Local(*rule_type),
                            function,
                        ),
                    }
                    for word in NfWord::ALL {
                        let value = options.word(word);
                        match pass {
                            WirePass::Measure => {
                                self.emit_pr_advance_wire_cursor(cursor, 8, function)
                            }
                            WirePass::Write => self.emit_pr_wire_store_word(
                                cursor,
                                PluralRulesWireWord::Local(value),
                                function,
                            ),
                        }
                    }
                }
                PluralRulesWireField::Configuration(record) => {
                    self.emit_pr_wire_configuration(*record, cursor, pass, function)
                }
                PluralRulesWireField::NumberInput { kind, bytes } => {
                    match pass {
                        WirePass::Measure => self.emit_pr_advance_wire_cursor(cursor, 8, function),
                        WirePass::Write => self.emit_pr_wire_store_word(
                            cursor,
                            PluralRulesWireWord::Local(*kind),
                            function,
                        ),
                    }
                    self.emit_pr_wire_bytes(*bytes, cursor, pass, function);
                }
            }
        }
    }

    pub(super) fn emit_pr_provider_request(
        &mut self,
        operation: PluralRulesOperation,
        fields: &[PluralRulesWireField<'_>],
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        self.emit_pr_set_const(length, PLURAL_RULES_WIRE_HEADER_BYTES as i64, function);
        self.emit_pr_wire_fields(fields, length, WirePass::Measure, function);
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        self.emit_heap_alloc_from_local(length, function)?;
        function.instruction(&Instruction::LocalTee(pointer));
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_pr_wire_store_word(
            cursor,
            PluralRulesWireWord::Constant(PLURAL_RULES_WIRE_VERSION),
            function,
        );
        self.emit_pr_wire_store_word(
            cursor,
            PluralRulesWireWord::Constant(operation.wire_tag()),
            function,
        );
        self.emit_pr_wire_fields(fields, cursor, WirePass::Write, function);
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        for local in [cursor, pointer, length] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(super) fn emit_pr_provider_call(
        &mut self,
        operation: PluralRulesOperation,
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
        self.emit_pr_range_error(PR_INVALID_OPTION, function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(PLURAL_RULES_WIRE_HEADER_BYTES as u32).wire(),
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

    pub(super) fn emit_plural_rules_record_from_response(
        &mut self,
        reader: &PluralRulesResponseReader,
        record: u32,
        function: &mut Function,
    ) {
        let value = self.reserve_temp_local();
        for offset in [HEAP_INTL_PR_LOCALE_OFFSET, HEAP_INTL_PR_DATA_LOCALE_OFFSET] {
            reader.bytes(self, value, function);
            self.store_i64_local_at_offset(record, offset, value, function);
        }
        for offset in [
            HEAP_INTL_PR_TYPE_OFFSET,
            HEAP_INTL_PR_CATEGORY_MASK_OFFSET,
            HEAP_INTL_PR_CATEGORY_COUNT_OFFSET,
        ] {
            reader.word(self, value, function);
            self.store_i64_local_at_offset(record, offset, value, function);
        }
        for index in 0..NUMBER_CONFIGURATION_WORDS {
            reader.word(self, value, function);
            self.store_i64_local_at_offset(
                record,
                HEAP_INTL_PR_WORDS_OFFSET + (index as u64 * 8),
                value,
                function,
            );
        }
        self.release_temp_local(value);
    }
}
