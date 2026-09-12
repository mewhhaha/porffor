//! Shared boundaries for Uint8Array text codecs.

use super::super::*;
use super::binary_data::{TypedArrayViewLocals, TypedArrayWitnessUse};
use crate::functions::RealmFunctionMaterializationContext;

pub(super) struct Uint8ArrayCodecOptions {
    payload_local: u32,
    tag_local: u32,
}

pub(super) enum Uint8ArrayCodecOption {
    Alphabet,
    LastChunkHandling,
    OmitPadding,
}

impl Uint8ArrayCodecOption {
    const fn name(&self) -> &'static str {
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
    pub(super) const fn code(self) -> i64 {
        match self {
            Self::Base64 => 0,
            Self::Base64Url => 1,
        }
    }
}

pub(super) enum Uint8ArrayCodecAccess {
    Read,
    Write,
}

enum Uint8ArrayCodecPrototype {
    Uint8Array,
    ArrayBuffer,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_uint8_array_codec_receiver(
        &mut self,
        receiver_local: u32,
        access: Uint8ArrayCodecAccess,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported("Uint8Array codec requires a receiver payload")
        })?;
        let receiver_tag = self
            .this_tag_local
            .ok_or_else(|| EmitError::unsupported("Uint8Array codec requires a receiver tag"))?;
        function.instruction(&Instruction::LocalGet(receiver_payload));
        function.instruction(&Instruction::LocalSet(receiver_local));
        self.emit_is_typed_array_i32(receiver_local, receiver_tag, function);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        self.load_i64_from_offset(
            receiver_local,
            HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
            function,
        );
        function.instruction(&Instruction::I64Const(typed_array_element_kind(
            StandardBuiltinId::Uint8ArrayConstructor,
        ) as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Uint8Array codec requires a Uint8Array receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        match access {
            Uint8ArrayCodecAccess::Read => {}
            Uint8ArrayCodecAccess::Write => {
                let buffer_local = self.reserve_temp_local();
                let flags_local = self.reserve_temp_local();
                self.load_i64_to_local_from_offset(
                    receiver_local,
                    HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
                    buffer_local,
                    function,
                );
                self.emit_load_array_buffer_flags(buffer_local, flags_local, function);
                function.instruction(&Instruction::LocalGet(flags_local));
                function.instruction(&Instruction::I64Const(
                    ArrayBufferFlag::Immutable.word() as i64
                ));
                function.instruction(&Instruction::I64And);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Uint8Array codec backing buffer is immutable",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.release_temp_local(flags_local);
                self.release_temp_local(buffer_local);
            }
        }
        Ok(())
    }

    pub(super) fn emit_uint8_array_codec_string(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        pointer_local: u32,
        byte_length_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Uint8Array codec input must be a string",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::LocalSet(pointer_local));
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::I64Const(u32::MAX as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(byte_length_local));
        Ok(())
    }

    pub(super) fn emit_uint8_array_codec_options(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<Uint8ArrayCodecOptions, EmitError> {
        self.emit_is_heap_object_like_tag_i32(tag_local, function);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Uint8Array codec options must be an object or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        Ok(Uint8ArrayCodecOptions {
            payload_local,
            tag_local,
        })
    }

    pub(super) fn emit_uint8_array_codec_option(
        &mut self,
        options: &Uint8ArrayCodecOptions,
        property: Uint8ArrayCodecOption,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(options.tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(
            self.strings.payload(property.name()),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            options.payload_local,
            options.tag_local,
            options.payload_local,
            options.tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(super) fn emit_uint8_array_base64_alphabet(
        &mut self,
        options: &Uint8ArrayCodecOptions,
        alphabet_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let expected_local = self.reserve_temp_local();
        self.emit_uint8_array_codec_option(
            options,
            Uint8ArrayCodecOption::Alphabet,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(alphabet_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            Uint8ArrayBase64Alphabet::Base64.code(),
        ));
        function.instruction(&Instruction::LocalSet(alphabet_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for (name, alphabet) in [
            ("base64", Uint8ArrayBase64Alphabet::Base64),
            ("base64url", Uint8ArrayBase64Alphabet::Base64Url),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(expected_local));
            self.emit_string_payload_equality_i32(payload_local, expected_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(alphabet.code()));
            function.instruction(&Instruction::LocalSet(alphabet_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(alphabet_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Uint8Array base64 alphabet must be base64 or base64url",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.release_temp_local(expected_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(super) fn emit_uint8_array_codec_bytes(
        &mut self,
        receiver_local: u32,
        pointer_local: u32,
        length_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let offset_local = self.reserve_temp_local();
        let stored_length_local = self.reserve_temp_local();
        let element_size_local = self.reserve_temp_local();
        self.emit_load_typed_array_private_state(
            receiver_local,
            buffer_local,
            offset_local,
            stored_length_local,
            element_size_local,
            function,
        );
        let view = TypedArrayViewLocals::new(
            receiver_local,
            buffer_local,
            offset_local,
            stored_length_local,
            element_size_local,
        );
        self.emit_typed_array_witness(
            &view,
            TypedArrayWitnessUse::ValidatedMethodEntry { length_local },
            function,
        )?;
        self.emit_load_array_buffer_data(buffer_local, pointer_local, function);
        function.instruction(&Instruction::LocalGet(pointer_local));
        function.instruction(&Instruction::LocalGet(offset_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(pointer_local));
        self.release_temp_local(element_size_local);
        self.release_temp_local(stored_length_local);
        self.release_temp_local(offset_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_uint8_array_codec_prototype(
        &mut self,
        prototype: Uint8ArrayCodecPrototype,
        result_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        match prototype {
            Uint8ArrayCodecPrototype::Uint8Array => {
                function.instruction(&Instruction::GlobalGet(
                    UINT8_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(result_local));
                self.load_i64_to_local_from_offset(
                    result_local,
                    HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                    result_local,
                    function,
                );
            }
            Uint8ArrayCodecPrototype::ArrayBuffer => {
                function.instruction(&Instruction::GlobalGet(ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(result_local));
            }
        }
        function.instruction(&Instruction::Else);
        let offset = match prototype {
            Uint8ArrayCodecPrototype::Uint8Array => {
                HEAP_FUNCTION_REALM_UINT8_ARRAY_PROTOTYPE_OFFSET
            }
            Uint8ArrayCodecPrototype::ArrayBuffer => {
                HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET
            }
        };
        self.load_i64_to_local_from_offset(self.current_env_local, offset, result_local, function);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_uint8_array_codec_allocation(
        &mut self,
        source_pointer_local: u32,
        byte_length_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let pointer_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let zero_local = self.reserve_temp_local();
        self.emit_array_buffer_backing_store_alloc(byte_length_local, pointer_local, function)?;
        function.instruction(&Instruction::LocalGet(pointer_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(source_pointer_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_length_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::MemoryCopy {
            src_mem: 0,
            dst_mem: self.buffer_memory_index(),
        });
        self.emit_uint8_array_codec_prototype(
            Uint8ArrayCodecPrototype::ArrayBuffer,
            prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_const_at_offset(
            buffer_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ARRAY_BUFFER,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(zero_local));
        self.emit_initialize_array_buffer_private_state(
            buffer_local,
            pointer_local,
            byte_length_local,
            byte_length_local,
            zero_local,
            function,
        );
        self.emit_uint8_array_codec_prototype(
            Uint8ArrayCodecPrototype::Uint8Array,
            prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        for (offset, value) in [
            (
                HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                OBJECT_INTERNAL_BRAND_TYPED_ARRAY,
            ),
            (HEAP_TYPED_ARRAY_BYTE_OFFSET, 0),
            (HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET, 1),
            (
                HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
                typed_array_element_kind(StandardBuiltinId::Uint8ArrayConstructor),
            ),
            (
                HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET,
                TypedArrayLengthMode::Fixed.word(),
            ),
        ] {
            self.store_i64_const_at_offset(self.result_local, offset, value, function);
        }
        self.store_i64_local_at_offset(
            self.result_local,
            HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET,
            buffer_local,
            function,
        );
        self.store_i64_local_at_offset(
            self.result_local,
            HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET,
            byte_length_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(zero_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(pointer_local);
        Ok(())
    }

    pub(super) fn emit_uint8_array_codec_result(
        &mut self,
        read_local: u32,
        written_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let number_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::Else);
        self.emit_load_function_defining_realm_object_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        for (name, value) in [("read", read_local), ("written", written_local)] {
            function.instruction(&Instruction::LocalGet(value));
            function.instruction(&Instruction::F64ConvertI64U);
            function.instruction(&Instruction::I64ReinterpretF64);
            function.instruction(&Instruction::LocalSet(number_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                name,
                number_local,
                tag_local,
                true,
                true,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(tag_local);
        self.release_temp_local(number_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(object_local);
        Ok(())
    }
    pub(super) fn emit_initialize_uint8_array_codec_methods(
        &mut self,
        constructor_local: u32,
        prototype_local: u32,
        buffer_prototype_local: u32,
        realm_functions: Option<&RealmFunctionMaterializationContext>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let method_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        for (members, target_local) in [
            (
                lila_ir::UINT8_ARRAY_CODEC_STATIC_MEMBERS.as_slice(),
                constructor_local,
            ),
            (
                lila_ir::UINT8_ARRAY_CODEC_PROTOTYPE_MEMBERS.as_slice(),
                prototype_local,
            ),
        ] {
            for &(name, builtin) in members {
                let meta = self
                    .functions
                    .get(&builtin.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "Uint8Array codec builtin is not rooted: {}",
                            builtin.debug_name()
                        ))
                    })?;
                match realm_functions {
                    Some(context) => self.emit_function_value_payload_in_realm(
                        &meta,
                        context,
                        method_local,
                        function,
                    )?,
                    None => {
                        self.emit_function_value_payload(&meta, function)?;
                        function.instruction(&Instruction::LocalSet(method_local));
                    }
                }
                self.store_i64_local_at_offset(
                    method_local,
                    HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                    method_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    method_local,
                    HEAP_FUNCTION_REALM_UINT8_ARRAY_PROTOTYPE_OFFSET,
                    prototype_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    method_local,
                    HEAP_FUNCTION_REALM_ARRAY_BUFFER_PROTOTYPE_OFFSET,
                    buffer_prototype_local,
                    function,
                );
                self.emit_object_append_local_data_property_with_flags(
                    target_local,
                    name,
                    method_local,
                    tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
            }
        }
        self.release_temp_local(tag_local);
        self.release_temp_local(method_local);
        Ok(())
    }
}
