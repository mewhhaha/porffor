//! Shared typed admission, options and completed byte lists for text codecs.
use super::super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::gc_types::*;
use crate::module::TypedArrayElementKind;
use crate::objects::PropertyKeyLocals;

pub(super) enum Uint8ArrayCodecAccess {
    Read,
    Write,
}

pub(super) enum Uint8ArrayCodecOption {
    Alphabet,
    LastChunkHandling,
    OmitPadding,
}
impl Uint8ArrayCodecOption {
    fn name(self) -> &'static str {
        match self {
            Self::Alphabet => "alphabet",
            Self::LastChunkHandling => "lastChunkHandling",
            Self::OmitPadding => "omitPadding",
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Uint8ArrayBase64Alphabet {
    Base64,
    Base64Url,
}
impl Uint8ArrayBase64Alphabet {
    pub(super) const fn code(self) -> i32 {
        match self {
            Self::Base64 => 0,
            Self::Base64Url => 1,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum Uint8ArrayLastChunk {
    Loose,
    Strict,
    StopBeforePartial,
}
impl Uint8ArrayLastChunk {
    pub(super) const fn code(self) -> i32 {
        match self {
            Self::Loose => 0,
            Self::Strict => 1,
            Self::StopBeforePartial => 2,
        }
    }
}

#[must_use]
pub(super) struct Base64AlphabetLocal(I32Local);
impl Base64AlphabetLocal {
    pub(super) fn load(&self, f: &mut Function) {
        self.0.load(f);
    }
    pub(super) fn clear(self, s: &RuntimeSchema, f: &mut Function) {
        s.release_i32_local(self.0, f);
    }
}
#[must_use]
pub(super) struct Base64LastChunkLocal(I32Local);
impl Base64LastChunkLocal {
    pub(super) fn load(&self, f: &mut Function) {
        self.0.load(f);
    }
    pub(super) fn clear(self, s: &RuntimeSchema, f: &mut Function) {
        s.release_i32_local(self.0, f);
    }
}

#[must_use]
pub(super) struct Uint8ArrayCodecOptions(ValueLocals);
impl Uint8ArrayCodecOptions {
    pub(super) fn clear(self, f: &mut Function) {
        self.0.clear(f);
    }
}

#[must_use]
pub(super) struct Uint8ArrayCodecReceiver {
    pub(super) object: GcLocal<TypedArrayObject>,
}
impl Uint8ArrayCodecReceiver {
    pub(super) fn clear(self, f: &mut Function) {
        self.object.clear(f);
    }
}

#[must_use]
pub(super) struct Uint8ArrayCodecString {
    pub(super) units: GcLocal<CodeUnitArray>,
    pub(super) length: I64Local,
}
impl Uint8ArrayCodecString {
    pub(super) fn clear(self, s: &RuntimeSchema, f: &mut Function) {
        self.units.clear(f);
        s.release_i64_local(self.length, f);
    }
}

/// Error retains the completed prefix; in-place entries write that prefix
/// before publishing SyntaxError, while static entries throw before allocation.
#[must_use]
pub(super) struct Uint8ArrayDecodedBytes {
    pub(super) bytes: GcLocal<ByteArray>,
    pub(super) read: I64Local,
    pub(super) written: I64Local,
    pub(super) error: CompletionLocals,
}
impl Uint8ArrayDecodedBytes {
    pub(super) fn clear(self, s: &RuntimeSchema, f: &mut Function) {
        self.error.clear(f);
        self.bytes.clear(f);
        s.release_i64_local(self.written, f);
        s.release_i64_local(self.read, f);
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_uint8_codec_abrupt_exit(
        &mut self,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) {
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        output.copy_from(pending, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
    }
    pub(super) fn emit_uint8_codec_type_error_if(
        &mut self,
        message: RuntimeErrorMessage,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(message, output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_uint8_codec_syntax_error_if(
        &mut self,
        message: RuntimeErrorMessage,
        result: &Uint8ArrayDecodedBytes,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_error(
            lila_ir::NativeErrorKind::SyntaxError,
            message,
            &result.error,
            f,
        )?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_uint8_codec_receiver(
        &mut self,
        access: Uint8ArrayCodecAccess,
        value: &ValueLocals,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<Uint8ArrayCodecReceiver, EmitError> {
        let s = self.runtime_schema();
        value.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            s.reference_type::<TypedArrayObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.emit_uint8_codec_type_error_if(
            RuntimeErrorMessage::UINT8ARRAY_CODEC_REQUIRES_A_UINT8ARRAY_RECEIVER,
            output,
            exit,
            f,
        )?;
        let object = s
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<TypedArrayObject>(s, f), f);
        let kind = s.reserve_i32_local(f);
        s.field(TypedArrayObjectSchema::ELEMENT_KIND)
            .read(&object, s, f)
            .store(kind, f);
        kind.load(f);
        f.instruction(&Instruction::I32Const(
            TypedArrayElementKind::Uint8.encode(),
        ));
        f.instruction(&Instruction::I32Ne);
        self.emit_uint8_codec_type_error_if(
            RuntimeErrorMessage::UINT8ARRAY_CODEC_REQUIRES_A_UINT8ARRAY_RECEIVER,
            output,
            exit,
            f,
        )?;
        s.release_i32_local(kind, f);
        match access {
            Uint8ArrayCodecAccess::Read => {}
            Uint8ArrayCodecAccess::Write => {
                // Bounds belong after input/options. This admission checks
                // immutable backing only, before any observable option Get.
                self.emit_validate_typed_array_writable_buffer(&object, pending, f)?;
                self.emit_uint8_codec_abrupt_exit(pending, output, exit, f);
            }
        }
        Ok(Uint8ArrayCodecReceiver { object })
    }
    pub(super) fn emit_uint8_codec_string(
        &mut self,
        value: &ValueLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<Uint8ArrayCodecString, EmitError> {
        let s = self.runtime_schema();
        value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        f.instruction(&Instruction::I32Ne);
        self.emit_uint8_codec_type_error_if(
            RuntimeErrorMessage::UINT8ARRAY_CODEC_INPUT_MUST_BE_A_STRING,
            output,
            exit,
            f,
        )?;
        let text = s
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<StringValue>(s, f), f);
        let units = s.reserve_gc_local(f).initialize(
            s.field(StringValueSchema::CODE_UNITS)
                .read(&text, s, f)
                .reference(),
            f,
        );
        let length = s.reserve_i64_local(f);
        s.array_type::<CodeUnitArray>().length(&units, s, f);
        f.instruction(&Instruction::I64ExtendI32U);
        length.store(f);
        text.clear(f);
        Ok(Uint8ArrayCodecString { units, length })
    }
    pub(super) fn emit_uint8_codec_options(
        &mut self,
        value: &ValueLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<Uint8ArrayCodecOptions, EmitError> {
        let s = self.runtime_schema();
        self.emit_is_heap_object_like_tag_i32(value.tag(), f);
        value.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        f.instruction(&Instruction::I32Or);
        f.instruction(&Instruction::I32Eqz);
        self.emit_uint8_codec_type_error_if(
            RuntimeErrorMessage::UINT8ARRAY_CODEC_OPTIONS_MUST_BE_AN_OBJECT_OR_UNDEFINED,
            output,
            exit,
            f,
        )?;
        let owned = s.reserve_value_local(f);
        owned.copy_from(value, f);
        Ok(Uint8ArrayCodecOptions(owned))
    }
    pub(super) fn emit_uint8_codec_option(
        &mut self,
        options: &Uint8ArrayCodecOptions,
        property: Uint8ArrayCodecOption,
        pending: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        options.0.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        pending.set_normal(&undefined, f);
        undefined.clear(f);
        f.instruction(&Instruction::Else);
        let text = s
            .reserve_gc_local(f)
            .initialize(self.emit_interned_string_reference(property.name(), f)?, f);
        let key = PropertyKeyLocals::from_string(s, &text, f);
        self.emit_object_read(&options.0, &options.0, &key, pending, f)?;
        key.clear(f);
        text.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    pub(super) fn emit_uint8_codec_alphabet(
        &mut self,
        options: &Uint8ArrayCodecOptions,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<Base64AlphabetLocal, EmitError> {
        let s = self.runtime_schema();
        let result = s.reserve_i32_local(f);
        let equal = s.reserve_i32_local(f);
        let folding = s.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        folding.store(f);
        self.emit_uint8_codec_option(options, Uint8ArrayCodecOption::Alphabet, pending, f)?;
        self.emit_uint8_codec_abrupt_exit(pending, output, exit, f);
        f.instruction(&Instruction::I32Const(-1));
        result.store(f);
        pending.value().tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I32Const(
            Uint8ArrayBase64Alphabet::Base64.code(),
        ));
        result.store(f);
        f.instruction(&Instruction::Else);
        pending.value().tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let actual = s
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(s, f), f);
        for (name, alphabet) in [
            ("base64", Uint8ArrayBase64Alphabet::Base64),
            ("base64url", Uint8ArrayBase64Alphabet::Base64Url),
        ] {
            let expected = s
                .reserve_gc_local(f)
                .initialize(self.emit_interned_string_reference(name, f)?, f);
            self.emit_gc_string_equality(&actual, &expected, folding, equal, f);
            expected.clear(f);
            equal.load(f);
            self.open_frame(ControlFrameKind::If, f);
            f.instruction(&Instruction::I32Const(alphabet.code()));
            result.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        actual.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        result.load(f);
        f.instruction(&Instruction::I32Const(-1));
        f.instruction(&Instruction::I32Eq);
        self.emit_uint8_codec_type_error_if(
            RuntimeErrorMessage::UINT8ARRAY_BASE64_ALPHABET_MUST_BE_BASE64_OR_BASE64URL,
            output,
            exit,
            f,
        )?;
        s.release_i32_local(folding, f);
        s.release_i32_local(equal, f);
        Ok(Base64AlphabetLocal(result))
    }
    pub(super) fn emit_uint8_codec_last_chunk(
        &mut self,
        options: &Uint8ArrayCodecOptions,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<Base64LastChunkLocal, EmitError> {
        let s = self.runtime_schema();
        let result = s.reserve_i32_local(f);
        let equal = s.reserve_i32_local(f);
        let folding = s.reserve_i32_local(f);
        f.instruction(&Instruction::I32Const(0));
        folding.store(f);
        self.emit_uint8_codec_option(
            options,
            Uint8ArrayCodecOption::LastChunkHandling,
            pending,
            f,
        )?;
        self.emit_uint8_codec_abrupt_exit(pending, output, exit, f);
        f.instruction(&Instruction::I32Const(-1));
        result.store(f);
        pending.value().tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::I32Const(Uint8ArrayLastChunk::Loose.code()));
        result.store(f);
        f.instruction(&Instruction::Else);
        pending.value().tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let actual = s
            .reserve_gc_local(f)
            .initialize(pending.value().cast_reference::<StringValue>(s, f), f);
        for (name, mode) in [
            ("loose", Uint8ArrayLastChunk::Loose),
            ("strict", Uint8ArrayLastChunk::Strict),
            (
                "stop-before-partial",
                Uint8ArrayLastChunk::StopBeforePartial,
            ),
        ] {
            let expected = s
                .reserve_gc_local(f)
                .initialize(self.emit_interned_string_reference(name, f)?, f);
            self.emit_gc_string_equality(&actual, &expected, folding, equal, f);
            expected.clear(f);
            equal.load(f);
            self.open_frame(ControlFrameKind::If, f);
            f.instruction(&Instruction::I32Const(mode.code()));
            result.store(f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        actual.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        result.load(f);
        f.instruction(&Instruction::I32Const(-1));
        f.instruction(&Instruction::I32Eq);
        self.emit_uint8_codec_type_error_if(RuntimeErrorMessage::UINT8ARRAY_BASE64_LASTCHUNKHANDLING_MUST_BE_LOOSE_STRICT_OR_STOP_BEFORE_PARTIAL,output,exit,f)?;
        s.release_i32_local(folding, f);
        s.release_i32_local(equal, f);
        Ok(Base64LastChunkLocal(result))
    }
    pub(super) fn emit_uint8_codec_decoded_list(
        &self,
        source_bound: I64Local,
        max_length: I64Local,
        f: &mut Function,
    ) -> Uint8ArrayDecodedBytes {
        let s = self.runtime_schema();
        let capacity = s.reserve_i32_local(f);
        source_bound.load(f);
        max_length.load(f);
        source_bound.load(f);
        max_length.load(f);
        f.instruction(&Instruction::I64LtU);
        f.instruction(&Instruction::Select);
        f.instruction(&Instruction::I32WrapI64);
        capacity.store(f);
        let bytes = s.reserve_gc_local(f).initialize(
            s.array_type::<ByteArray>()
                .filled(GcOperand::i32(0), capacity, f),
            f,
        );
        s.release_i32_local(capacity, f);
        let read = s.reserve_i64_local(f);
        let written = s.reserve_i64_local(f);
        let error = s.reserve_completion(f);
        f.instruction(&Instruction::I64Const(0));
        read.store(f);
        f.instruction(&Instruction::I64Const(0));
        written.store(f);
        error.initialize(f);
        Uint8ArrayDecodedBytes {
            bytes,
            read,
            written,
            error,
        }
    }
    pub(super) fn emit_uint8_codec_push_byte(
        &self,
        result: &Uint8ArrayDecodedBytes,
        byte: I32Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let index = s.reserve_i32_local(f);
        result.written.load(f);
        f.instruction(&Instruction::I32WrapI64);
        index.store(f);
        s.array_type::<ByteArray>()
            .write(&result.bytes, index, GcOperand::i32_local(byte), s, f);
        self.emit_increment_local(result.written, 1, f);
        s.release_i32_local(index, f);
    }
    pub(super) fn emit_uint8_codec_string_unit(
        &self,
        text: &Uint8ArrayCodecString,
        index: I64Local,
        unit: I32Local,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let at = s.reserve_i32_local(f);
        index.load(f);
        f.instruction(&Instruction::I32WrapI64);
        at.store(f);
        s.array_type::<CodeUnitArray>()
            .read(&text.units, at, s, f)
            .store(unit, f);
        s.release_i32_local(at, f);
    }
    pub(super) fn emit_uint8_codec_copy_bytes(
        &mut self,
        receiver: &GcLocal<TypedArrayObject>,
        bytes: &GcLocal<ByteArray>,
        length: I64Local,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let i = s.reserve_i64_local(f);
        let at = s.reserve_i32_local(f);
        let byte = s.reserve_i32_local(f);
        let bits = s.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        i.store(f);
        let end = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        i.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(end, f);
        i.load(f);
        f.instruction(&Instruction::I32WrapI64);
        at.store(f);
        s.array_type::<ByteArray>()
            .read(bytes, at, s, f)
            .store(byte, f);
        byte.load(f);
        f.instruction(&Instruction::I64ExtendI32U);
        bits.store(f);
        self.emit_typed_array_element_bits_write(receiver, i, bits, pending, f)?;
        self.emit_uint8_codec_abrupt_exit(pending, output, exit, f);
        self.emit_increment_local(i, 1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i64_local(bits, f);
        s.release_i32_local(byte, f);
        s.release_i32_local(at, f);
        s.release_i64_local(i, f);
        Ok(())
    }
    pub(super) fn emit_uint8_codec_static_result(
        &mut self,
        result: &Uint8ArrayDecodedBytes,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        self.emit_uint8_codec_abrupt_exit(&result.error, output, exit, f);
        let realm = s
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let constructor = s.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::Uint8ArrayConstructor,
            &constructor,
            f,
        );
        let length = s.reserve_value_local(f);
        let number = s.reserve_i64_local(f);
        result.written.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::I64ReinterpretF64);
        number.store(f);
        length.set_number(number, f);
        let argv = self.emit_pre_evaluated_arg_vector(&[&length], f);
        self.emit_function_or_proxy_construct_with_argv(
            &constructor,
            &constructor,
            &argv,
            pending,
            f,
        )?;
        argv.clear(f);
        length.clear(f);
        s.release_i64_local(number, f);
        constructor.clear(f);
        realm.clear(f);
        self.emit_uint8_codec_abrupt_exit(pending, output, exit, f);
        let value = s.reserve_value_local(f);
        value.copy_from(pending.value(), f);
        let object = s
            .reserve_gc_local(f)
            .initialize(value.cast_reference::<TypedArrayObject>(s, f), f);
        self.emit_uint8_codec_copy_bytes(
            &object,
            &result.bytes,
            result.written,
            pending,
            output,
            exit,
            f,
        )?;
        output.set_normal(&value, f);
        object.clear(f);
        value.clear(f);
        Ok(())
    }
    pub(super) fn emit_uint8_codec_count_result(
        &mut self,
        result: &Uint8ArrayDecodedBytes,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        self.emit_uint8_codec_abrupt_exit(&result.error, output, exit, f);
        let realm = s
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let prototype = s.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ObjectPrototype,
            &prototype,
            f,
        );
        let object = s.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype), f)?,
            f,
        );
        realm.clear(f);
        prototype.clear(f);
        let target = s.reserve_value_local(f);
        target.set_reference(&object, s, f);
        let value = s.reserve_value_local(f);
        let number = s.reserve_i64_local(f);
        for (name, count) in [("read", result.read), ("written", result.written)] {
            count.load(f);
            f.instruction(&Instruction::F64ConvertI64U);
            f.instruction(&Instruction::I64ReinterpretF64);
            number.store(f);
            value.set_number(number, f);
            let text = s
                .reserve_gc_local(f)
                .initialize(self.emit_interned_string_reference(name, f)?, f);
            let key = PropertyKeyLocals::from_string(s, &text, f);
            self.emit_create_data_property_or_throw(&target, &key, &value, pending, f)?;
            key.clear(f);
            text.clear(f);
            self.emit_uint8_codec_abrupt_exit(pending, output, exit, f);
        }
        output.set_normal(&target, f);
        s.release_i64_local(number, f);
        value.clear(f);
        target.clear(f);
        object.clear(f);
        Ok(())
    }
    pub(super) fn emit_uint8_codec_snapshot(
        &mut self,
        receiver: &Uint8ArrayCodecReceiver,
        length: I64Local,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<GcLocal<ByteArray>, EmitError> {
        let s = self.runtime_schema();
        self.emit_validate_typed_array_view(&receiver.object, length, pending, f)?;
        self.emit_uint8_codec_abrupt_exit(pending, output, exit, f);
        // A byte list has a physical GC extent. Do not wrap a u64 host-backed
        // shared length into a smaller array index.
        length.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let at = s.reserve_i32_local(f);
        length.load(f);
        f.instruction(&Instruction::I32WrapI64);
        at.store(f);
        let bytes = s.reserve_gc_local(f).initialize(
            s.array_type::<ByteArray>().filled(GcOperand::i32(0), at, f),
            f,
        );
        let i = s.reserve_i64_local(f);
        let bits = s.reserve_i64_local(f);
        let valid = s.reserve_i32_local(f);
        let byte = s.reserve_i32_local(f);
        f.instruction(&Instruction::I64Const(0));
        i.store(f);
        let end = self.open_frame(ControlFrameKind::Block, f);
        let again = self.open_frame(ControlFrameKind::Loop, f);
        i.load(f);
        length.load(f);
        f.instruction(&Instruction::I64GeU);
        self.emit_branch_if_to_target(end, f);
        self.emit_typed_array_element_bits_read(&receiver.object, i, bits, valid, f);
        valid.load(f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        f.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        bits.load(f);
        f.instruction(&Instruction::I32WrapI64);
        byte.store(f);
        i.load(f);
        f.instruction(&Instruction::I32WrapI64);
        at.store(f);
        s.array_type::<ByteArray>()
            .write(&bytes, at, GcOperand::i32_local(byte), s, f);
        self.emit_increment_local(i, 1, f);
        self.emit_branch_to_target(again, f);
        self.pop_control(ControlFrameKind::Loop);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        s.release_i32_local(byte, f);
        s.release_i32_local(valid, f);
        s.release_i64_local(bits, f);
        s.release_i64_local(i, f);
        s.release_i32_local(at, f);
        Ok(bytes)
    }
}
