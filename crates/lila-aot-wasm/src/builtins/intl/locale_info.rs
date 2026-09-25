//! `Intl.Locale.prototype.get*` (ECMA-402 15.3.16-15.3.22) and
//! `Intl.supportedValuesOf` (8.3.2).
//!
//! The emitted program performs RequireInternalSlot, ToString and every
//! Array/object construction in the current function's Realm. The pinned
//! provider receives only the canonical `[[Locale]]` tag (or a closed key) and
//! answers with the primitive result shape fixed by the query.

use super::*;
use lila_intl::{
    IntlHostOp, LocaleInfoQuery, LocaleInfoResponseKind, LocaleInfoShape, SupportedValuesKey,
    TextDirection, LOCALE_INFO_WIRE_HEADER_BYTES, LOCALE_INFO_WIRE_VERSION,
};

const SUPPORTED_VALUES_KEY_ERROR: &str = "Intl.supportedValuesOf key is not a supported key";
const TEXT_INFO_DIRECTION: &str = "direction";
const WEEK_INFO_FIRST_DAY: &str = "firstDay";
const WEEK_INFO_WEEKEND: &str = "weekend";

/// Every constant string the Locale information emitters reference. The key
/// and direction spellings come from the same closed domains the emitters
/// walk, so an added member cannot miss the pool.
pub(crate) fn intl_locale_info_pool_strings() -> Vec<String> {
    [
        SUPPORTED_VALUES_KEY_ERROR,
        TEXT_INFO_DIRECTION,
        WEEK_INFO_FIRST_DAY,
        WEEK_INFO_WEEKEND,
    ]
    .into_iter()
    .chain(SupportedValuesKey::ALL.iter().map(|key| key.name()))
    .chain(TextDirection::ALL.iter().map(|direction| direction.name()))
    .map(str::to_owned)
    .collect()
}

/// One request word: a constant chosen at compile time or a runtime local.
#[derive(Clone, Copy)]
enum InfoWord {
    Constant(u64),
    Local(u32),
}

/// The two element encodings a Locale information list can carry.
#[derive(Clone, Copy)]
enum InfoElement {
    /// A length-prefixed identifier that becomes a String.
    Identifier,
    /// An ISO 8601 weekday word that becomes a Number.
    Weekday,
}

/// A response cursor bounded by the byte count the host reported, not merely
/// by Wasm memory. Malformed provider output is an ABI fault.
struct LocaleInfoResponseCursor {
    cursor: u32,
    end: u32,
}

impl LocaleInfoResponseCursor {
    fn new(builder: &mut FunctionBuilder<'_>, response: u32, function: &mut Function) -> Self {
        let cursor = builder.reserve_temp_local();
        let end = builder.reserve_temp_local();
        builder.emit_unpack_string_payload(response, cursor, end, function);
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::LocalGet(end));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(end));
        Self::advance(cursor, LOCALE_INFO_WIRE_HEADER_BYTES, function);
        Self { cursor, end }
    }

    fn advance(cursor: u32, count: u64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I64Const(count as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor));
    }

    /// Traps unless `count` more bytes (a constant or a local) remain.
    fn require(&self, count: InfoWord, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::I64GtU);
        match count {
            InfoWord::Constant(constant) => {
                function.instruction(&Instruction::I64Const(constant as i64))
            }
            InfoWord::Local(local) => function.instruction(&Instruction::LocalGet(local)),
        };
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    fn word(&self, builder: &FunctionBuilder<'_>, destination: u32, function: &mut Function) {
        self.require(InfoWord::Constant(8), function);
        builder.load_i64_to_local_from_offset(self.cursor, 0, destination, function);
        Self::advance(self.cursor, 8, function);
    }

    /// Traps unless every one of `count` records can own `minimum` bytes.
    fn require_records(&self, count: u32, minimum: u64, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::LocalGet(self.end));
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(minimum as i64));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    /// A length-prefixed UTF-8 identifier, exposed as a string payload that
    /// views the immutable response buffer.
    fn text(&self, builder: &mut FunctionBuilder<'_>, destination: u32, function: &mut Function) {
        let length = builder.reserve_temp_local();
        self.word(builder, length, function);
        self.require(InfoWord::Local(length), function);
        builder.emit_pack_string_payload(self.cursor, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        function.instruction(&Instruction::LocalGet(self.cursor));
        function.instruction(&Instruction::LocalGet(length));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(self.cursor));
        builder.release_temp_local(length);
    }

    fn finish(self, builder: &mut FunctionBuilder<'_>, function: &mut Function) {
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

impl<'a> FunctionBuilder<'a> {
    fn emit_intl_info_store_word(&self, cursor: u32, value: InfoWord, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(cursor));
        function.instruction(&Instruction::I32WrapI64);
        match value {
            InfoWord::Constant(constant) => {
                function.instruction(&Instruction::I64Const(constant as i64))
            }
            InfoWord::Local(local) => function.instruction(&Instruction::LocalGet(local)),
        };
        function.instruction(&Instruction::I64Store(MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        LocaleInfoResponseCursor::advance(cursor, 8, function);
    }

    /// Allocates `[version][operation request code][words...][bytes]`.
    fn emit_intl_info_request(
        &mut self,
        operation: IntlHostOp,
        words: &[InfoWord],
        text: Option<u32>,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let length = self.reserve_temp_local();
        let pointer = self.reserve_temp_local();
        let cursor = self.reserve_temp_local();
        let text_offset = self.reserve_temp_local();
        let text_length = self.reserve_temp_local();
        let fixed = LOCALE_INFO_WIRE_HEADER_BYTES + 8 * words.len() as u64;
        function.instruction(&Instruction::I64Const(fixed as i64));
        function.instruction(&Instruction::LocalSet(length));
        if let Some(text) = text {
            self.emit_unpack_string_payload(text, text_offset, text_length, function);
            function.instruction(&Instruction::LocalGet(length));
            function.instruction(&Instruction::I64Const(8));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalGet(text_length));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(length));
        }
        self.emit_heap_alloc_from_local(length, function)?;
        function.instruction(&Instruction::LocalTee(pointer));
        function.instruction(&Instruction::LocalSet(cursor));
        self.emit_intl_info_store_word(
            cursor,
            InfoWord::Constant(LOCALE_INFO_WIRE_VERSION),
            function,
        );
        self.emit_intl_info_store_word(
            cursor,
            InfoWord::Constant(u64::from(operation.code()) * 2),
            function,
        );
        for word in words {
            self.emit_intl_info_store_word(cursor, *word, function);
        }
        if text.is_some() {
            self.emit_intl_info_store_word(cursor, InfoWord::Local(text_length), function);
            function.instruction(&Instruction::LocalGet(cursor));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::LocalGet(text_offset));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::LocalGet(text_length));
            function.instruction(&Instruction::I32WrapI64);
            function.instruction(&Instruction::MemoryCopy {
                src_mem: 0,
                dst_mem: 0,
            });
        }
        self.emit_pack_string_payload(pointer, length, function);
        function.instruction(&Instruction::LocalSet(destination));
        for local in [text_length, text_offset, cursor, pointer, length] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// A pure capacity query, one exact allocation and the writing call. No
    /// JavaScript operation can run between the two host calls.
    fn emit_intl_info_host_call(
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
        // Both operations are total over their typed requests: Rejected, a
        // written empty result and any out-of-domain response are ABI faults.
        function.instruction(&Instruction::LocalGet(outcome));
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(LOCALE_INFO_WIRE_HEADER_BYTES as u32).wire(),
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
        function.instruction(&Instruction::I64Const(
            IntlHostCallOutcome::RequiredCapacity(0).wire(),
        ));
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
            (0, LOCALE_INFO_WIRE_VERSION),
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

    fn emit_intl_info_array_entry(
        &mut self,
        array: u32,
        index: u32,
        payload: u32,
        kind: ValueKind,
        function: &mut Function,
    ) {
        let entry = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array, HEAP_PTR_OFFSET, entry, function);
        function.instruction(&Instruction::LocalGet(entry));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry));
        self.store_i64_const_at_offset(entry, HEAP_ARRAY_TAG_OFFSET, kind.tag() as u64, function);
        self.store_i64_local_at_offset(entry, HEAP_ARRAY_PAYLOAD_OFFSET, payload, function);
        self.store_i64_const_at_offset(
            entry,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        self.release_temp_local(entry);
    }

    /// Reads `[count][record]*` into a new Array of the current function's
    /// Realm. Identifier records become Strings; weekday words become Numbers.
    fn emit_intl_info_array(
        &mut self,
        reader: &LocaleInfoResponseCursor,
        element: InfoElement,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let count = self.reserve_temp_local();
        let index = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        reader.word(self, count, function);
        reader.require_records(count, 8, function);
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            count,
            destination,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::LocalGet(count));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        let kind = match element {
            InfoElement::Identifier => {
                reader.text(self, value, function);
                ValueKind::String
            }
            InfoElement::Weekday => {
                reader.word(self, value, function);
                self.emit_intl_info_weekday_number(value, function);
                ValueKind::Number
            }
        };
        self.emit_intl_info_array_entry(destination, index, value, kind, function);
        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        for local in [value, index, count] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Validates an ISO weekday word (1 through 7) and converts it in place
    /// to the Number payload encoding.
    fn emit_intl_info_weekday_number(&self, local: u32, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(local));
    }

    /// OrdinaryObjectCreate(%Object.prototype%) of the current function Realm.
    fn emit_intl_info_result_object(
        &mut self,
        destination: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm = self.reserve_temp_local();
        let prototype = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(realm));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_realm_intrinsic_prototype_or_global(
            realm,
            HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
            OBJECT_PROTOTYPE_GLOBAL_INDEX,
            prototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(prototype), None, function)?;
        function.instruction(&Instruction::LocalSet(destination));
        self.release_temp_local(prototype);
        self.release_temp_local(realm);
        Ok(())
    }

    /// CreateDataPropertyOrThrow on a fresh ordinary object.
    fn emit_intl_info_data_property(
        &mut self,
        object: u32,
        name: &str,
        payload: u32,
        kind: ValueKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key));
        function.instruction(&Instruction::I64Const(kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        self.emit_object_append_data_property_with_flags(
            object, key, payload, tag, true, true, true, function,
        )?;
        self.release_temp_local(tag);
        self.release_temp_local(key);
        Ok(())
    }

    fn emit_intl_info_require_kind(
        &self,
        kind: u32,
        expected: LocaleInfoResponseKind,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(kind));
        function.instruction(&Instruction::I64Const(expected.wire_code() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    fn emit_intl_info_set_result(&self, payload: u32, kind: ValueKind, function: &mut Function) {
        function.instruction(&Instruction::LocalGet(payload));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(kind.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
    }

    pub(in crate::builtins) fn emit_intl_locale_info_builtin(
        &mut self,
        query: LocaleInfoQuery,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let record = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let kind = self.reserve_temp_local();
        let output = self.reserve_temp_local();
        let value = self.reserve_temp_local();

        self.emit_intl_locale_record_from_receiver(record, function)?;
        self.load_i64_to_local_from_offset(record, HEAP_INTL_LOCALE_TAG_OFFSET, tag, function);
        self.emit_intl_info_request(
            IntlHostOp::LocaleInfo,
            &[InfoWord::Constant(query.wire_code())],
            Some(tag),
            request,
            function,
        )?;
        self.emit_intl_info_host_call(IntlHostOp::LocaleInfo, request, response, function)?;
        let reader = LocaleInfoResponseCursor::new(self, response, function);
        reader.word(self, kind, function);
        match query.shape() {
            LocaleInfoShape::Identifiers => {
                self.emit_intl_info_require_kind(
                    kind,
                    LocaleInfoResponseKind::Identifiers,
                    function,
                );
                self.emit_intl_info_array(&reader, InfoElement::Identifier, output, function)?;
                self.emit_intl_info_set_result(output, ValueKind::Array, function);
            }
            LocaleInfoShape::OptionalIdentifiers => {
                // TimeZonesOfLocale step 2: undefined without a region subtag.
                function.instruction(&Instruction::LocalGet(kind));
                function.instruction(&Instruction::I64Const(
                    LocaleInfoResponseKind::Undefined.wire_code() as i64,
                ));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(output));
                self.emit_intl_info_set_result(output, ValueKind::Undefined, function);
                function.instruction(&Instruction::Else);
                self.emit_intl_info_require_kind(
                    kind,
                    LocaleInfoResponseKind::Identifiers,
                    function,
                );
                self.emit_intl_info_array(&reader, InfoElement::Identifier, output, function)?;
                self.emit_intl_info_set_result(output, ValueKind::Array, function);
                function.instruction(&Instruction::End);
            }
            LocaleInfoShape::Direction => {
                // getTextInfo: "direction" is defined even when undefined.
                self.emit_intl_info_result_object(output, function)?;
                let direction_kind = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(value));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(direction_kind));
                function.instruction(&Instruction::LocalGet(kind));
                function.instruction(&Instruction::I64Const(
                    LocaleInfoResponseKind::Undefined.wire_code() as i64,
                ));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_intl_info_require_kind(kind, LocaleInfoResponseKind::Direction, function);
                let code = self.reserve_temp_local();
                reader.word(self, code, function);
                for direction in TextDirection::ALL {
                    function.instruction(&Instruction::LocalGet(code));
                    function.instruction(&Instruction::I64Const(direction.wire_code() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload(direction.name()),
                    ));
                    function.instruction(&Instruction::LocalSet(value));
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(direction_kind));
                    function.instruction(&Instruction::End);
                }
                function.instruction(&Instruction::LocalGet(direction_kind));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Unreachable);
                function.instruction(&Instruction::End);
                self.release_temp_local(code);
                function.instruction(&Instruction::End);
                let key = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(
                    self.strings.payload(TEXT_INFO_DIRECTION),
                ));
                function.instruction(&Instruction::LocalSet(key));
                self.emit_object_append_data_property_with_flags(
                    output,
                    key,
                    value,
                    direction_kind,
                    true,
                    true,
                    true,
                    function,
                )?;
                self.release_temp_local(key);
                self.release_temp_local(direction_kind);
                self.emit_intl_info_set_result(output, ValueKind::Object, function);
            }
            LocaleInfoShape::Week => {
                self.emit_intl_info_require_kind(kind, LocaleInfoResponseKind::Week, function);
                self.emit_intl_info_result_object(output, function)?;
                reader.word(self, value, function);
                self.emit_intl_info_weekday_number(value, function);
                self.emit_intl_info_data_property(
                    output,
                    WEEK_INFO_FIRST_DAY,
                    value,
                    ValueKind::Number,
                    function,
                )?;
                self.emit_intl_info_array(&reader, InfoElement::Weekday, value, function)?;
                self.emit_intl_info_data_property(
                    output,
                    WEEK_INFO_WEEKEND,
                    value,
                    ValueKind::Array,
                    function,
                )?;
                self.emit_intl_info_set_result(output, ValueKind::Object, function);
            }
        }
        reader.finish(self, function);
        for local in [value, output, kind, response, request, tag, record] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    pub(in crate::builtins) fn emit_intl_supported_values_of(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let key = self.reserve_temp_local();
        let expected = self.reserve_temp_local();
        let code = self.reserve_temp_local();
        let request = self.reserve_temp_local();
        let response = self.reserve_temp_local();
        let output = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, payload, tag, function);
        self.emit_value_to_string_payload(payload, tag, function)?;
        function.instruction(&Instruction::LocalSet(key));
        self.emit_return_current_completion_if_throw(function);
        // `code` is the key's wire code plus one; zero selects no key.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(code));
        for candidate in SupportedValuesKey::ALL {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(candidate.name()),
            ));
            function.instruction(&Instruction::LocalSet(expected));
            self.emit_string_payload_equality_i32(key, expected, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(candidate.wire_code() as i64 + 1));
            function.instruction(&Instruction::LocalSet(code));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(code));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_range_error(
            SUPPORTED_VALUES_KEY_ERROR,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // The request carries the selected closed key, never the JS string.
        function.instruction(&Instruction::LocalGet(code));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(code));
        self.emit_intl_info_request(
            IntlHostOp::SupportedValues,
            &[InfoWord::Local(code)],
            None,
            request,
            function,
        )?;

        self.emit_intl_info_host_call(IntlHostOp::SupportedValues, request, response, function)?;
        let reader = LocaleInfoResponseCursor::new(self, response, function);
        self.emit_intl_info_array(&reader, InfoElement::Identifier, output, function)?;
        reader.finish(self, function);
        self.emit_intl_info_set_result(output, ValueKind::Array, function);
        for local in [output, response, request, code, expected, key, tag, payload] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
