use super::super::*;
use crate::operations::PrimitiveToStringAbruptRoute;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum JsonBuiltin {
    Parse,
    Stringify,
    RawJson,
    IsRawJson,
}

const JSON_PARSE_FRAME_SIZE: u64 = 48;
const JSON_PARSE_FRAME_PAYLOAD_OFFSET: u64 = 0;
const JSON_PARSE_FRAME_TAG_OFFSET: u64 = 8;
const JSON_PARSE_FRAME_STATE_OFFSET: u64 = 16;
const JSON_PARSE_FRAME_KEY_OR_INDEX_OFFSET: u64 = 24;
const JSON_PARSE_FRAME_METADATA_PAYLOAD_OFFSET: u64 = 32;
const JSON_PARSE_FRAME_METADATA_TAG_OFFSET: u64 = 40;
const JSON_PARSE_INITIAL_FRAME_CAPACITY: u64 = 8;

const JSON_PARSE_METADATA_SIZE: u64 = 40;
const JSON_PARSE_METADATA_VALUE_PAYLOAD_OFFSET: u64 = 0;
const JSON_PARSE_METADATA_VALUE_TAG_OFFSET: u64 = 8;
const JSON_PARSE_METADATA_SOURCE_OFFSET: u64 = 16;
const JSON_PARSE_METADATA_CHILDREN_PAYLOAD_OFFSET: u64 = 24;
const JSON_PARSE_METADATA_CHILDREN_TAG_OFFSET: u64 = 32;

const JSON_REVIVER_FRAME_SIZE: u64 = 96;
const JSON_REVIVER_FRAME_HOLDER_PAYLOAD_OFFSET: u64 = 0;
const JSON_REVIVER_FRAME_HOLDER_TAG_OFFSET: u64 = 8;
const JSON_REVIVER_FRAME_KEY_PAYLOAD_OFFSET: u64 = 16;
const JSON_REVIVER_FRAME_KEY_INDEX_OFFSET: u64 = 24;
const JSON_REVIVER_FRAME_METADATA_OFFSET: u64 = 32;
const JSON_REVIVER_FRAME_VALUE_PAYLOAD_OFFSET: u64 = 40;
const JSON_REVIVER_FRAME_VALUE_TAG_OFFSET: u64 = 48;
const JSON_REVIVER_FRAME_STATE_OFFSET: u64 = 56;
const JSON_REVIVER_FRAME_CURSOR_OFFSET: u64 = 64;
const JSON_REVIVER_FRAME_LIMIT_OFFSET: u64 = 72;
const JSON_REVIVER_FRAME_KEYS_PAYLOAD_OFFSET: u64 = 80;
const JSON_REVIVER_FRAME_ROLE_OFFSET: u64 = 88;
const JSON_REVIVER_INITIAL_FRAME_CAPACITY: u64 = 8;

const JSON_STRINGIFY_SEEN_NODE_SIZE: u64 = 24;
const JSON_STRINGIFY_SEEN_VALUE_OFFSET: u64 = 0;
const JSON_STRINGIFY_SEEN_PARENT_OFFSET: u64 = 8;
const JSON_STRINGIFY_SEEN_REALM_OFFSET: u64 = 16;

macro_rules! json_wire_domain {
    ($name:ident { $($variant:ident = $word:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum $name {
            $($variant),+
        }

        impl $name {
            const ALL: &'static [Self] = &[$(Self::$variant),+];

            const fn word(self) -> u64 {
                match self {
                    $(Self::$variant => $word),+
                }
            }
        }

        const _: () = {
            let all = $name::ALL;
            let mut index = 0;
            while index < all.len() {
                assert!(all[index].word() == index as u64);
                index += 1;
            }
        };
    };
}

json_wire_domain!(JsonReviverFrameState {
    Enter = 0,
    ArrayChildren = 1,
    ObjectChildren = 2,
    Apply = 3,
});

json_wire_domain!(JsonReviverPropertyRole {
    Nested = 0,
    Root = 1,
});

const JSON_PARSE_ARRAY_FIRST_OR_END: i64 = 0;
const JSON_PARSE_ARRAY_VALUE: i64 = 1;
const JSON_PARSE_ARRAY_COMMA_OR_END: i64 = 2;
const JSON_PARSE_OBJECT_FIRST_KEY_OR_END: i64 = 3;
const JSON_PARSE_OBJECT_KEY: i64 = 4;
const JSON_PARSE_OBJECT_COLON: i64 = 5;
const JSON_PARSE_OBJECT_VALUE: i64 = 6;
const JSON_PARSE_OBJECT_COMMA_OR_END: i64 = 7;

#[derive(Debug, Clone, Copy)]
enum JsonStaticPropertyKey<'a> {
    String(&'a str),
    ArrayIndex(usize),
}

fn json_static_primitive_source(value: &JsonStaticValueIr) -> Option<&str> {
    match value {
        JsonStaticValueIr::Null { source }
        | JsonStaticValueIr::Boolean { source, .. }
        | JsonStaticValueIr::Number { source, .. }
        | JsonStaticValueIr::String { source, .. } => Some(source.as_str()),
        JsonStaticValueIr::Array(_) | JsonStaticValueIr::Object(_) => None,
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_json_builtin(
        &mut self,
        builtin: JsonBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            JsonBuiltin::Parse => {
                let text_payload_local = self.reserve_temp_local();
                let text_tag_local = self.reserve_temp_local();
                let reviver_payload_local = self.reserve_temp_local();
                let reviver_tag_local = self.reserve_temp_local();
                let value_payload_local = self.reserve_temp_local();
                let value_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let root_payload_local = self.reserve_temp_local();
                let root_tag_local = self.reserve_temp_local();
                let parsed_string_flag_local = self.reserve_temp_local();
                let reviver_callable_local = self.reserve_temp_local();
                let root_metadata_local = self.reserve_temp_local();
                let parse_float_meta = self
                    .functions
                    .get(HOST_PARSE_FLOAT_FUNCTION_ID)
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in lila wasm-aot first slice: missing builtin meta `parseFloat`",
                        )
                    })?;

                self.emit_builtin_arg_to_locals(0, text_payload_local, text_tag_local, function);
                self.emit_builtin_arg_to_locals(
                    1,
                    reviver_payload_local,
                    reviver_tag_local,
                    function,
                );
                self.emit_value_to_string_payload(text_payload_local, text_tag_local, function)?;
                function.instruction(&Instruction::LocalSet(text_payload_local));
                self.emit_return_current_completion_if_throw(function);
                self.emit_is_callable_i32(reviver_tag_local, reviver_payload_local, function)?;
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(reviver_callable_local));

                self.emit_try_parse_json_text(
                    text_payload_local,
                    value_payload_local,
                    value_tag_local,
                    parsed_string_flag_local,
                    reviver_callable_local,
                    root_metadata_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(parsed_string_flag_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_validate_json_parse_no_raw_string_controls(text_payload_local, function)?;
                self.emit_validate_json_parse_no_structural_trailing_commas(
                    text_payload_local,
                    function,
                )?;
                self.emit_try_parse_json_string_text(
                    text_payload_local,
                    value_payload_local,
                    value_tag_local,
                    parsed_string_flag_local,
                    function,
                )?;
                self.emit_try_parse_json_keyword_text(
                    text_payload_local,
                    value_payload_local,
                    value_tag_local,
                    parsed_string_flag_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(parsed_string_flag_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_validate_json_parse_number_text(text_payload_local, function)?;
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(text_tag_local));

                self.emit_direct_js_call(
                    &parse_float_meta,
                    None,
                    &[(text_payload_local, text_tag_local)],
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(reviver_callable_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(self.strings.payload("")));
                function.instruction(&Instruction::LocalSet(key_payload_local));
                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(root_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(root_tag_local));
                self.emit_object_define_enumerable_data(
                    root_payload_local,
                    key_payload_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                self.emit_json_internalize_dynamic(
                    root_payload_local,
                    root_tag_local,
                    root_metadata_local,
                    reviver_payload_local,
                    reviver_tag_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);

                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(root_metadata_local);
                self.release_temp_local(reviver_callable_local);
                self.release_temp_local(parsed_string_flag_local);
                self.release_temp_local(root_tag_local);
                self.release_temp_local(root_payload_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(value_tag_local);
                self.release_temp_local(value_payload_local);
                self.release_temp_local(reviver_tag_local);
                self.release_temp_local(reviver_payload_local);
                self.release_temp_local(text_tag_local);
                self.release_temp_local(text_payload_local);
            }
            JsonBuiltin::Stringify => {
                let value_payload_local = self.reserve_temp_local();
                let value_tag_local = self.reserve_temp_local();
                let replacer_payload_local = self.reserve_temp_local();
                let replacer_tag_local = self.reserve_temp_local();
                let key_payload_local = self.reserve_temp_local();
                let key_tag_local = self.reserve_temp_local();
                let wrapper_payload_local = self.reserve_temp_local();
                let wrapper_tag_local = self.reserve_temp_local();
                let space_payload_local = self.reserve_temp_local();
                let space_tag_local = self.reserve_temp_local();
                let gap_payload_local = self.reserve_temp_local();
                let gap_start_local = self.reserve_temp_local();
                let gap_len_local = self.reserve_temp_local();
                let space_boxed_kind_local = self.reserve_temp_local();
                let stringify_realm_local = self.reserve_temp_local();
                let root_seen_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
                self.emit_builtin_arg_to_locals(
                    1,
                    replacer_payload_local,
                    replacer_tag_local,
                    function,
                );
                self.emit_builtin_arg_to_locals(2, space_payload_local, space_tag_local, function);
                function.instruction(&Instruction::I64Const(self.strings.payload("")));
                function.instruction(&Instruction::LocalSet(gap_payload_local));
                self.emit_json_normalize_replacer_array(
                    replacer_payload_local,
                    replacer_tag_local,
                    function,
                )?;

                function.instruction(&Instruction::LocalGet(space_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    space_payload_local,
                    HEAP_OBJECT_BOXED_KIND_OFFSET,
                    space_boxed_kind_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(space_boxed_kind_local));
                function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_json_boxed_object_to_primitive_payload(
                    space_payload_local,
                    ToPrimitiveHint::String,
                    space_payload_local,
                    space_tag_local,
                    function,
                )?;
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalGet(space_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Cannot convert a Symbol value to a string",
                    space_payload_local,
                    space_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_primitive_to_string_payload(
                    space_payload_local,
                    space_tag_local,
                    PrimitiveToStringAbruptRoute::ReturnCurrentFunction,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(space_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(space_tag_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(space_boxed_kind_local));
                function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_json_boxed_object_to_primitive_payload(
                    space_payload_local,
                    ToPrimitiveHint::Number,
                    space_payload_local,
                    space_tag_local,
                    function,
                )?;
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalGet(space_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Cannot convert BigInt to number",
                    space_payload_local,
                    space_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(space_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Cannot convert Symbol to number",
                    space_payload_local,
                    space_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_primitive_to_number_payload(
                    space_tag_local,
                    space_payload_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(space_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(space_tag_local));
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(space_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(10));
                function.instruction(&Instruction::LocalSet(gap_len_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(gap_start_local));
                self.emit_utf16_code_unit_range_payload_from_locals(
                    space_payload_local,
                    gap_start_local,
                    gap_len_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(gap_payload_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalGet(space_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(gap_start_local));
                function.instruction(&Instruction::LocalGet(space_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::F64Gt);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(space_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::F64Const(Ieee64::from(10.0)));
                function.instruction(&Instruction::F64Gt);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(10));
                function.instruction(&Instruction::LocalSet(gap_len_local));
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(space_payload_local));
                function.instruction(&Instruction::F64ReinterpretI64);
                function.instruction(&Instruction::I64TruncSatF64U);
                function.instruction(&Instruction::LocalSet(gap_len_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(gap_len_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(self.strings.payload("          ")));
                function.instruction(&Instruction::LocalSet(gap_payload_local));
                self.emit_string_slice_payload_from_locals(
                    gap_payload_local,
                    gap_start_local,
                    gap_len_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(gap_payload_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(self.strings.payload("")));
                function.instruction(&Instruction::LocalSet(key_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(key_tag_local));
                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(wrapper_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(wrapper_tag_local));
                self.emit_object_define_enumerable_data(
                    wrapper_payload_local,
                    key_payload_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(stringify_realm_local));
                function.instruction(&Instruction::LocalGet(self.current_env_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Else);
                self.load_i64_to_local_from_offset(
                    self.current_env_local,
                    HEAP_FUNCTION_DEFINING_REALM_OFFSET,
                    stringify_realm_local,
                    function,
                );
                function.instruction(&Instruction::End);
                self.emit_json_create_seen_root(stringify_realm_local, root_seen_local, function)?;
                self.emit_json_apply_to_json(
                    value_payload_local,
                    value_tag_local,
                    key_payload_local,
                    key_tag_local,
                    root_seen_local,
                    function,
                )?;
                self.emit_json_apply_replacer_with_this(
                    replacer_payload_local,
                    replacer_tag_local,
                    wrapper_payload_local,
                    wrapper_tag_local,
                    key_payload_local,
                    key_tag_local,
                    value_payload_local,
                    value_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::Block(BlockType::Empty));
                self.emit_json_omits_value_i32(value_payload_local, value_tag_local, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::Br(1));
                function.instruction(&Instruction::End);
                let root_indent_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(root_indent_local));
                self.emit_json_stringify_value_call(
                    value_payload_local,
                    value_tag_local,
                    replacer_payload_local,
                    replacer_tag_local,
                    gap_payload_local,
                    self.result_local,
                    root_indent_local,
                    root_seen_local,
                    function,
                )?;
                self.release_temp_local(root_indent_local);
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                function.instruction(&Instruction::End);
                self.release_temp_local(root_seen_local);
                self.release_temp_local(stringify_realm_local);
                self.release_temp_local(space_boxed_kind_local);
                self.release_temp_local(gap_len_local);
                self.release_temp_local(gap_start_local);
                self.release_temp_local(gap_payload_local);
                self.release_temp_local(space_tag_local);
                self.release_temp_local(space_payload_local);
                self.release_temp_local(wrapper_tag_local);
                self.release_temp_local(wrapper_payload_local);
                self.release_temp_local(key_tag_local);
                self.release_temp_local(key_payload_local);
                self.release_temp_local(replacer_tag_local);
                self.release_temp_local(replacer_payload_local);
                self.release_temp_local(value_tag_local);
                self.release_temp_local(value_payload_local);
            }
            JsonBuiltin::RawJson => {
                let object_local = self.reserve_temp_local();
                let key_local = self.reserve_temp_local();
                let value_payload_local = self.reserve_temp_local();
                let value_tag_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Cannot convert a Symbol value to a string",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                self.emit_value_to_string_payload(value_payload_local, value_tag_local, function)?;
                function.instruction(&Instruction::LocalSet(value_payload_local));
                self.emit_return_current_completion_if_throw(function);
                self.emit_validate_json_raw_json_text(value_payload_local, function)?;

                self.emit_alloc_plain_object_with_prototype(None, None, function)?;
                function.instruction(&Instruction::LocalSet(object_local));
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    OBJECT_INTERNAL_BRAND_RAW_JSON,
                    function,
                );
                function.instruction(&Instruction::I64Const(self.strings.payload("rawJSON")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                self.emit_object_define_data_with_configurable(
                    object_local,
                    key_local,
                    value_payload_local,
                    value_tag_local,
                    false,
                    true,
                    false,
                    function,
                )?;
                self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, 0, function);
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(value_tag_local);
                self.release_temp_local(value_payload_local);
                self.release_temp_local(key_local);
                self.release_temp_local(object_local);
            }
            JsonBuiltin::IsRawJson => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let brand_local = self.reserve_temp_local();
                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::LocalGet(arg_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    arg_payload_local,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    brand_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(brand_local));
                function.instruction(&Instruction::I64Const(
                    OBJECT_INTERNAL_BRAND_RAW_JSON as i64,
                ));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.release_temp_local(brand_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
        }
        Ok(())
    }

    fn json_static_value_to_expr(value: &JsonStaticValueIr) -> TypedExpr {
        match value {
            JsonStaticValueIr::Null { .. } => {
                TypedExpr::from_info(ValueInfo::new(ValueKind::Null), ExprIr::Null)
            }
            JsonStaticValueIr::Boolean { value, .. } => {
                TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(*value))
            }
            JsonStaticValueIr::Number { bits, .. } => {
                TypedExpr::from_info(ValueInfo::new(ValueKind::Number), ExprIr::Number(*bits))
            }
            JsonStaticValueIr::String { value, .. } => TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String(value.clone()),
            ),
            JsonStaticValueIr::Array(values) => {
                let elements = values
                    .iter()
                    .map(Self::json_static_value_to_expr)
                    .collect::<Vec<_>>();
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Array,
                        possible_kinds: KindSet::from_kind(ValueKind::Array),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::ArrayLiteral(elements),
                )
            }
            JsonStaticValueIr::Object(properties) => {
                let properties = properties
                    .iter()
                    .map(|(key, value)| ObjectPropertyIr::Data {
                        key: key.clone(),
                        value: Self::json_static_value_to_expr(value),
                        is_shorthand: false,
                    })
                    .collect::<Vec<_>>();
                TypedExpr::from_info(
                    ValueInfo {
                        kind: ValueKind::Object,
                        possible_kinds: KindSet::from_kind(ValueKind::Object),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    },
                    ExprIr::ObjectLiteral(properties),
                )
            }
        }
    }

    pub(crate) fn compile_json_static_reviver_to_locals(
        &mut self,
        value: &JsonStaticValueIr,
        reviver: &TypedExpr,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reviver_payload_local = self.reserve_temp_local();
        let reviver_tag_local = self.reserve_temp_local();
        let root_payload_local = self.reserve_temp_local();
        let root_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let empty_key_local = self.reserve_temp_local();

        self.compile_expr_to_locals(reviver, reviver_payload_local, reviver_tag_local, function)?;
        let value_expr = Self::json_static_value_to_expr(value);
        self.compile_expr_to_locals(&value_expr, value_payload_local, value_tag_local, function)?;

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(root_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(root_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(empty_key_local));
        self.emit_object_define_enumerable_data(
            root_payload_local,
            empty_key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        self.emit_json_static_internalize_property(
            value,
            root_payload_local,
            root_tag_local,
            JsonStaticPropertyKey::String(""),
            JsonReviverPropertyRole::Root,
            reviver_payload_local,
            reviver_tag_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(empty_key_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(root_tag_local);
        self.release_temp_local(root_payload_local);
        self.release_temp_local(reviver_tag_local);
        self.release_temp_local(reviver_payload_local);
        Ok(())
    }

    fn emit_json_static_internalize_property(
        &mut self,
        value: &JsonStaticValueIr,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key: JsonStaticPropertyKey<'_>,
        role: JsonReviverPropertyRole,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let current_payload_local = self.reserve_temp_local();
        let current_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();

        self.emit_json_static_key_payload(key, key_payload_local, function);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));

        match key {
            JsonStaticPropertyKey::ArrayIndex(index) => {
                let index_local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(index as i64));
                function.instruction(&Instruction::LocalSet(index_local));
                self.emit_array_index_get_with_prototype(
                    holder_payload_local,
                    index_local,
                    holder_payload_local,
                    holder_tag_local,
                    current_payload_local,
                    current_tag_local,
                    function,
                )?;
                self.release_temp_local(index_local);
            }
            JsonStaticPropertyKey::String(_) => {
                self.emit_object_read(
                    holder_payload_local,
                    holder_tag_local,
                    holder_payload_local,
                    holder_tag_local,
                    key_payload_local,
                    current_payload_local,
                    current_tag_local,
                    function,
                )?;
            }
        }
        self.emit_propagate_throw_from_locals_if_needed(
            current_payload_local,
            current_tag_local,
            function,
        )?;

        match value {
            JsonStaticValueIr::Array(values) => {
                function.instruction(&Instruction::LocalGet(current_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                for (index, element) in values.iter().enumerate() {
                    self.emit_json_static_internalize_property(
                        element,
                        current_payload_local,
                        current_tag_local,
                        JsonStaticPropertyKey::ArrayIndex(index),
                        JsonReviverPropertyRole::Nested,
                        reviver_payload_local,
                        reviver_tag_local,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                }
                if values.is_empty() {
                    self.emit_json_static_maybe_internalize_dynamic_value(
                        current_payload_local,
                        current_tag_local,
                        reviver_payload_local,
                        reviver_tag_local,
                        function,
                    )?;
                }
                function.instruction(&Instruction::Else);
                self.emit_json_static_maybe_internalize_dynamic_value(
                    current_payload_local,
                    current_tag_local,
                    reviver_payload_local,
                    reviver_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            JsonStaticValueIr::Object(properties) => {
                function.instruction(&Instruction::LocalGet(current_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                for property_index in Self::json_static_object_property_order(properties) {
                    let (property_key, property_value) = &properties[property_index];
                    self.emit_json_static_internalize_property(
                        property_value,
                        current_payload_local,
                        current_tag_local,
                        JsonStaticPropertyKey::String(property_key),
                        JsonReviverPropertyRole::Nested,
                        reviver_payload_local,
                        reviver_tag_local,
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                }
                if properties.is_empty() {
                    self.emit_json_static_maybe_internalize_dynamic_value(
                        current_payload_local,
                        current_tag_local,
                        reviver_payload_local,
                        reviver_tag_local,
                        function,
                    )?;
                }
                function.instruction(&Instruction::Else);
                self.emit_json_static_maybe_internalize_dynamic_value(
                    current_payload_local,
                    current_tag_local,
                    reviver_payload_local,
                    reviver_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
            }
            JsonStaticValueIr::Null { .. }
            | JsonStaticValueIr::Boolean { .. }
            | JsonStaticValueIr::Number { .. }
            | JsonStaticValueIr::String { .. } => {
                self.emit_json_static_maybe_internalize_dynamic_value(
                    current_payload_local,
                    current_tag_local,
                    reviver_payload_local,
                    reviver_tag_local,
                    function,
                )?;
            }
        }

        self.emit_json_static_apply_reviver(
            value,
            holder_payload_local,
            holder_tag_local,
            key,
            role,
            key_payload_local,
            key_tag_local,
            current_payload_local,
            current_tag_local,
            reviver_payload_local,
            reviver_tag_local,
            result_payload_local,
            result_tag_local,
            function,
        )?;

        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(current_tag_local);
        self.release_temp_local(current_payload_local);
        Ok(())
    }

    fn emit_json_static_maybe_internalize_dynamic_value(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let is_array_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let keys_tag_local = self.reserve_temp_local();
        let keys_arg_payload_local = self.reserve_temp_local();
        let keys_arg_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(0));

        self.emit_json_static_is_array_like(
            value_payload_local,
            value_tag_local,
            is_array_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_payload_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_value_to_number_payload(element_tag_local, element_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::LocalGet(element_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_json_static_internalize_dynamic_property(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            Some(index_local),
            reviver_payload_local,
            reviver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(keys_arg_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(keys_arg_tag_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("ownKeys")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            self.scratch_local,
            proxy_handler_tag_local,
            self.scratch_local,
            proxy_handler_tag_local,
            key_payload_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(keys_arg_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(keys_arg_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        let keys_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.keys`",
                )
            })?;
        self.emit_direct_js_call(
            &keys_meta,
            None,
            &[(keys_arg_payload_local, keys_arg_tag_local)],
            keys_payload_local,
            keys_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            keys_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_static_internalize_dynamic_property(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            None,
            reviver_payload_local,
            reviver_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(keys_arg_tag_local);
        self.release_temp_local(keys_arg_payload_local);
        self.release_temp_local(keys_tag_local);
        self.release_temp_local(keys_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(is_array_local);
        Ok(())
    }

    fn emit_json_static_internalize_dynamic_property(
        &mut self,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        array_index_local: Option<u32>,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        if let Some(array_index_local) = array_index_local {
            function.instruction(&Instruction::LocalGet(holder_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_array_index_get_with_prototype(
                holder_payload_local,
                array_index_local,
                holder_payload_local,
                holder_tag_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_object_read(
                holder_payload_local,
                holder_tag_local,
                holder_payload_local,
                holder_tag_local,
                key_payload_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
        } else {
            self.emit_object_read(
                holder_payload_local,
                holder_tag_local,
                holder_payload_local,
                holder_tag_local,
                key_payload_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
        }
        self.emit_propagate_current_completion_if_throw(function);

        self.emit_json_apply_reviver_with_source(
            None,
            holder_payload_local,
            holder_tag_local,
            array_index_local,
            false,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            reviver_payload_local,
            reviver_tag_local,
            self.scratch_local,
            self.result_tag_local,
            function,
        )?;

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        Ok(())
    }

    fn emit_json_static_is_array_like(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        is_array_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(target_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(target_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    fn emit_json_static_apply_reviver(
        &mut self,
        value: &JsonStaticValueIr,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key: JsonStaticPropertyKey<'_>,
        role: JsonReviverPropertyRole,
        key_payload_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let array_index_local = match key {
            JsonStaticPropertyKey::ArrayIndex(index) => {
                let local = self.reserve_temp_local();
                function.instruction(&Instruction::I64Const(index as i64));
                function.instruction(&Instruction::LocalSet(local));
                Some(local)
            }
            JsonStaticPropertyKey::String(_) => None,
        };

        let source = json_static_primitive_source(value);
        let result = if source.is_some() {
            self.emit_json_static_current_matches_value_i32(
                value,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_json_apply_reviver_with_source(
                source,
                holder_payload_local,
                holder_tag_local,
                array_index_local,
                role,
                key_payload_local,
                key_tag_local,
                value_payload_local,
                value_tag_local,
                reviver_payload_local,
                reviver_tag_local,
                result_payload_local,
                result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::Else);
            self.emit_json_apply_reviver_with_source(
                None,
                holder_payload_local,
                holder_tag_local,
                array_index_local,
                role,
                key_payload_local,
                key_tag_local,
                value_payload_local,
                value_tag_local,
                reviver_payload_local,
                reviver_tag_local,
                result_payload_local,
                result_tag_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            Ok(())
        } else {
            self.emit_json_apply_reviver_with_source(
                None,
                holder_payload_local,
                holder_tag_local,
                array_index_local,
                role,
                key_payload_local,
                key_tag_local,
                value_payload_local,
                value_tag_local,
                reviver_payload_local,
                reviver_tag_local,
                result_payload_local,
                result_tag_local,
                function,
            )
        };

        if let Some(array_index_local) = array_index_local {
            self.release_temp_local(array_index_local);
        }
        result
    }

    fn emit_json_static_current_matches_value_i32(
        &mut self,
        value: &JsonStaticValueIr,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match value {
            JsonStaticValueIr::Null { .. } => {
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
            }
            JsonStaticValueIr::Boolean { value, .. } => {
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::I64Const(i64::from(*value)));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
            }
            JsonStaticValueIr::Number { bits, .. } => {
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(value_payload_local));
                function.instruction(&Instruction::I64Const(*bits as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
            }
            JsonStaticValueIr::String { value, .. } => {
                let expected_payload_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(value_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64Const(self.strings.payload(value)));
                function.instruction(&Instruction::LocalSet(expected_payload_local));
                self.emit_string_payload_equality_i32(
                    value_payload_local,
                    expected_payload_local,
                    function,
                );
                function.instruction(&Instruction::I32And);
                self.release_temp_local(expected_payload_local);
            }
            JsonStaticValueIr::Array(_) | JsonStaticValueIr::Object(_) => {
                function.instruction(&Instruction::I32Const(0));
            }
        }
        Ok(())
    }

    fn emit_json_apply_reviver_with_source(
        &mut self,
        source: Option<&str>,
        holder_payload_local: u32,
        holder_tag_local: u32,
        array_index_local: Option<u32>,
        role: JsonReviverPropertyRole,
        key_payload_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let context_payload_local = self.reserve_temp_local();
        let context_tag_local = self.reserve_temp_local();
        let source_key_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(context_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(context_tag_local));

        if let Some(source) = source {
            function.instruction(&Instruction::I64Const(self.strings.payload("source")));
            function.instruction(&Instruction::LocalSet(source_key_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(source)));
            function.instruction(&Instruction::LocalSet(source_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(source_tag_local));
            self.emit_object_define_enumerable_data(
                context_payload_local,
                source_key_local,
                source_payload_local,
                source_tag_local,
                function,
            )?;
        }

        self.emit_indirect_call_from_locals(
            reviver_payload_local,
            reviver_tag_local,
            Some((holder_payload_local, holder_tag_local)),
            &[
                (key_payload_local, key_tag_local),
                (value_payload_local, value_tag_local),
                (context_payload_local, context_tag_local),
            ],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_json_apply_reviver_result(
            role,
            holder_payload_local,
            holder_tag_local,
            key_payload_local,
            array_index_local,
            result_payload_local,
            result_tag_local,
            function,
        )?;

        self.release_temp_local(source_tag_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(source_key_local);
        self.release_temp_local(context_tag_local);
        self.release_temp_local(context_payload_local);
        Ok(())
    }

    /// Applies the result of a completed reviver call.
    ///
    /// The synthetic root and an ordinary property named `""` are distinct:
    /// only the typed role selects the no-mutation root path. Dynamic callers
    /// may carry `-1` in `array_index_local`; the holder-tag/index guard keeps
    /// that sentinel out of Array storage.
    fn emit_json_apply_reviver_result(
        &mut self,
        role: JsonReviverPropertyRole,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key_payload_local: u32,
        array_index_local: Option<u32>,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match role {
            JsonReviverPropertyRole::Root => {}
            JsonReviverPropertyRole::Nested => {
                function.instruction(&Instruction::LocalGet(result_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                if let Some(array_index_local) = array_index_local {
                    function.instruction(&Instruction::LocalGet(array_index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::LocalGet(holder_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_delete(
                        holder_payload_local,
                        array_index_local,
                        self.scratch_local,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                    self.emit_object_delete(
                        holder_payload_local,
                        holder_tag_local,
                        key_payload_local,
                        self.scratch_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                } else {
                    self.emit_object_delete(
                        holder_payload_local,
                        holder_tag_local,
                        key_payload_local,
                        self.scratch_local,
                        function,
                    )?;
                }
                function.instruction(&Instruction::Else);
                if let Some(array_index_local) = array_index_local {
                    function.instruction(&Instruction::LocalGet(array_index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::LocalGet(holder_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_create_data_property_silent(
                        holder_payload_local,
                        array_index_local,
                        result_payload_local,
                        result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_json_create_data_property(
                        holder_payload_local,
                        holder_tag_local,
                        key_payload_local,
                        result_payload_local,
                        result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                } else {
                    self.emit_json_create_data_property(
                        holder_payload_local,
                        holder_tag_local,
                        key_payload_local,
                        result_payload_local,
                        result_tag_local,
                        function,
                    )?;
                }
                function.instruction(&Instruction::End);
            }
        }
        Ok(())
    }

    fn emit_json_create_data_property(
        &mut self,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key_payload_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let boxed_kind_local = self.reserve_temp_local();

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(holder_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            holder_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_create_data_property_or_throw(
            holder_payload_local,
            holder_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            "Cannot redefine JSON reviver property",
            "Cannot add JSON reviver property",
            None,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_object_create_data_property_silent(
            holder_payload_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_create_data_property_or_throw(
            holder_payload_local,
            holder_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            "Cannot redefine JSON reviver property",
            "Cannot add JSON reviver property",
            None,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(boxed_kind_local);
        Ok(())
    }

    fn json_static_object_property_order(properties: &[(String, JsonStaticValueIr)]) -> Vec<usize> {
        let mut integer_indices = Vec::new();
        let mut string_indices = Vec::new();

        for (index, (key, _)) in properties.iter().enumerate() {
            if let Some(array_index) = Self::json_static_array_index_key(key) {
                integer_indices.push((array_index, index));
            } else {
                string_indices.push(index);
            }
        }

        integer_indices.sort_by_key(|(array_index, _)| *array_index);
        integer_indices
            .into_iter()
            .map(|(_, index)| index)
            .chain(string_indices)
            .collect()
    }

    fn json_static_array_index_key(key: &str) -> Option<u32> {
        if key.is_empty() || (key.len() > 1 && key.starts_with('0')) {
            return None;
        }
        let value = key.parse::<u32>().ok()?;
        (value != u32::MAX && value.to_string() == key).then_some(value)
    }

    fn emit_json_static_key_payload(
        &mut self,
        key: JsonStaticPropertyKey<'_>,
        key_payload_local: u32,
        function: &mut Function,
    ) {
        match key {
            JsonStaticPropertyKey::String(key) => {
                function.instruction(&Instruction::I64Const(self.strings.payload(key)));
            }
            JsonStaticPropertyKey::ArrayIndex(index) => {
                function.instruction(&Instruction::I64Const(
                    self.strings.payload(&index.to_string()),
                ));
            }
        }
        function.instruction(&Instruction::LocalSet(key_payload_local));
    }

    pub(crate) fn emit_json_quote_string_payload(
        &mut self,
        string_payload_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let src_offset_local = self.reserve_temp_local();
        let src_len_local = self.reserve_temp_local();
        let dst_len_capacity_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let advance_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let next_byte_local = self.reserve_temp_local();
        let next_codepoint_local = self.reserve_temp_local();
        let next_advance_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let actual_len_local = self.reserve_temp_local();
        let digit_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            src_offset_local,
            src_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64Const(6));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_len_capacity_local));
        function.instruction(&Instruction::LocalGet(dst_len_capacity_local));
        function.instruction(&Instruction::I64Const(7));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(!7_i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(dst_len_capacity_local));
        self.emit_heap_alloc_from_local(dst_len_capacity_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        Self::emit_store_u8_const_and_advance(b'"', dst_pos_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(byte_local));

        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            index_local,
            src_len_local,
            byte_local,
            codepoint_local,
            advance_local,
            temp_local,
            function,
        );

        self.emit_is_high_surrogate_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(src_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Load8U(Self::memarg8(0)));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(next_byte_local));
        self.emit_decode_utf8_scalar_at_index(
            src_offset_local,
            next_index_local,
            src_len_local,
            next_byte_local,
            next_codepoint_local,
            next_advance_local,
            temp_local,
            function,
        );
        self.emit_is_low_surrogate_i32(next_codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0xD800));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalGet(next_codepoint_local));
        function.instruction(&Instruction::I64Const(0xDC00));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(0x10000));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(codepoint_local));
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalGet(next_advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'u', dst_pos_local, function);
        for shift in [12, 8, 4, 0] {
            self.emit_store_json_lower_hex_digit_from_byte(
                codepoint_local,
                digit_local,
                shift,
                dst_pos_local,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_is_low_surrogate_i32(codepoint_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'u', dst_pos_local, function);
        for shift in [12, 8, 4, 0] {
            self.emit_store_json_lower_hex_digit_from_byte(
                codepoint_local,
                digit_local,
                shift,
                dst_pos_local,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_local_and_advance(codepoint_local, dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(8));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'b', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(9));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b't', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'n', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(12));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'f', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(13));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'r', dst_pos_local, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(32));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        Self::emit_store_u8_const_and_advance(b'\\', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'u', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'0', dst_pos_local, function);
        Self::emit_store_u8_const_and_advance(b'0', dst_pos_local, function);
        self.emit_store_json_lower_hex_digit_from_byte(
            codepoint_local,
            digit_local,
            4,
            dst_pos_local,
            function,
        );
        self.emit_store_json_lower_hex_digit_from_byte(
            codepoint_local,
            digit_local,
            0,
            dst_pos_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(src_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(next_index_local));
        self.emit_copy_bytes(next_index_local, dst_pos_local, advance_local, function);
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(advance_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Self::emit_store_u8_const_and_advance(b'"', dst_pos_local, function);
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(actual_len_local));
        self.emit_pack_string_payload(dst_offset_local, actual_len_local, function);
        function.instruction(&Instruction::LocalSet(output_local));

        self.release_temp_local(digit_local);
        self.release_temp_local(actual_len_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(next_advance_local);
        self.release_temp_local(next_codepoint_local);
        self.release_temp_local(next_byte_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(advance_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(dst_len_capacity_local);
        self.release_temp_local(src_len_local);
        self.release_temp_local(src_offset_local);
        Ok(())
    }

    pub(crate) fn emit_store_u8_const_and_advance(
        byte: u8,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(byte as i32));
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
    }

    pub(crate) fn emit_store_u8_local_and_advance(
        byte_local: u32,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
    }

    pub(crate) fn emit_store_json_lower_hex_digit_from_byte(
        &self,
        byte_local: u32,
        digit_local: u32,
        shift: i64,
        dst_pos_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(byte_local));
        if shift != 0 {
            function.instruction(&Instruction::I64Const(shift));
            function.instruction(&Instruction::I64ShrU);
        }
        function.instruction(&Instruction::I64Const(15));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(digit_local));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(10));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(digit_local));
        function.instruction(&Instruction::I64Const((b'a' - 10) as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I32Store8(Self::memarg8(0)));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(dst_pos_local));
    }

    pub(crate) fn emit_json_apply_replacer_with_this(
        &mut self,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        this_payload_local: u32,
        this_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_is_callable_i32(replacer_tag_local, replacer_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        // Invoke the replacer, leaving any throw completion in place rather than
        // returning `self.result_local` directly: the thrown value lands in
        // `value_payload_local`/`value_tag_local`, so it must be moved into the
        // result slot before the helper returns. Using the auto-returning call
        // variant here would surface a stale `result_local` and drop the actual
        // error object (see replacer-function-abrupt.js).
        let argc_local = self.reserve_temp_local();
        let argv_local = self.reserve_temp_local();
        self.emit_pre_evaluated_arg_vector(
            &[
                (key_payload_local, key_tag_local),
                (value_payload_local, value_tag_local),
            ],
            argc_local,
            argv_local,
            function,
        )?;
        self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
            replacer_payload_local,
            replacer_tag_local,
            this_payload_local,
            this_tag_local,
            argc_local,
            argv_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.release_temp_local(argv_local);
        self.release_temp_local(argc_local);
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_json_omits_value_i32(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_callable_i32(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        Ok(())
    }

    pub(crate) fn emit_json_array_element_string_payload(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        value_string_local: u32,
        indent_level_local: u32,
        seen_stack_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_json_omits_value_i32(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::LocalSet(value_string_local));
        function.instruction(&Instruction::Else);
        self.emit_json_stringify_value_call(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            indent_level_local,
            seen_stack_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Emits a `call` to the shared JSON.stringify value helper (runtime
    /// recursion) and surfaces its result. On a normal completion the serialized
    /// string payload is written to `output_local`; on a throw completion the
    /// thrown value is re-raised through the current completion.
    pub(crate) fn emit_json_stringify_value_call(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level_local: u32,
        seen_stack_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper_index = self
            .json_stringify_value_helper_function_index()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: JSON.stringify helper without heap",
                )
            })?;
        let result_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalGet(replacer_payload_local));
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::LocalGet(gap_payload_local));
        function.instruction(&Instruction::LocalGet(indent_level_local));
        function.instruction(&Instruction::LocalGet(seen_stack_local));
        function.instruction(&Instruction::Call(helper_index));
        function.instruction(&Instruction::LocalSet(self.completion_aux_local));
        function.instruction(&Instruction::LocalSet(self.completion_local));
        function.instruction(&Instruction::LocalSet(result_tag_local));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(result_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(result_tag_local);
        Ok(())
    }

    pub(crate) fn emit_json_property_key_payload_is_symbol_i32(
        &self,
        key_payload_local: u32,
        function: &mut Function,
    ) {
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
    }

    pub(crate) fn emit_json_throw_bigint_serialization_type_error(
        &mut self,
        seen_stack_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            seen_stack_local,
            JSON_STRINGIFY_SEEN_REALM_OFFSET,
            realm_local,
            function,
        );
        self.emit_load_realm_intrinsic_prototype_or_global(
            realm_local,
            HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET,
            TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            prototype_local,
            function,
        );
        self.emit_throw_runtime_error_with_prototype_local(
            TYPE_ERROR_NAME,
            "Do not know how to serialize a BigInt",
            prototype_local,
            payload_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(prototype_local);
        self.release_temp_local(realm_local);
        Ok(())
    }

    pub(crate) fn emit_json_apply_to_json(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        key_payload_local: u32,
        key_tag_local: u32,
        seen_stack_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let property_key_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let fallback_prototype_local = self.reserve_temp_local();
        let realm_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("toJSON")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(fallback_prototype_local));
        self.load_i64_to_local_from_offset(
            fallback_prototype_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            fallback_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(realm_local));
        self.load_i64_to_local_from_offset(
            seen_stack_local,
            JSON_STRINGIFY_SEEN_REALM_OFFSET,
            realm_local,
            function,
        );
        self.emit_load_realm_intrinsic_prototype_or_local(
            realm_local,
            HEAP_REALM_INTRINSICS_BIGINT_PROTOTYPE_OFFSET,
            fallback_prototype_local,
            prototype_payload_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(method_tag_local));
        self.emit_object_read(
            prototype_payload_local,
            method_tag_local,
            value_payload_local,
            value_tag_local,
            property_key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_named_prop_read(
            value_payload_local,
            property_key_local,
            method_payload_local,
            method_tag_local,
            None,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            property_key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_or_proxy_call_leave_throw_completion(
            method_payload_local,
            method_tag_local,
            value_payload_local,
            value_tag_local,
            &[(key_payload_local, key_tag_local)],
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(realm_local);
        self.release_temp_local(fallback_prototype_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(property_key_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        Ok(())
    }

    pub(crate) fn emit_json_boxed_object_to_primitive_payload(
        &mut self,
        object_payload_local: u32,
        hint: ToPrimitiveHint,
        output_payload_local: u32,
        output_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let hook_names: &[&str] = match hint {
            ToPrimitiveHint::String => &["Symbol.toPrimitive", "toString", "valueOf"],
            ToPrimitiveHint::Default | ToPrimitiveHint::Number => {
                &["Symbol.toPrimitive", "valueOf", "toString"]
            }
        };
        let object_tag_local = self.reserve_temp_local();
        let hook_value_payload = self.reserve_temp_local();
        let hook_value_tag = self.reserve_temp_local();
        let call_result_payload = self.reserve_temp_local();
        let call_result_tag = self.reserve_temp_local();
        let primitive_result_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(primitive_result_local));

        for hook_name in hook_names {
            let key_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalGet(primitive_result_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            // `Symbol.toPrimitive` is a symbol-valued PropertyKey: its lookup
            // payload must carry `PROPERTY_KEY_SYMBOL_MARKER`, because a plain
            // string payload with the same bytes is a *different* key and never
            // matches a stored `[Symbol.toPrimitive]` entry. `toString` and
            // `valueOf` stay plain string keys.
            function.instruction(&Instruction::I64Const(
                self.strings.static_builtin_property_key_payload(hook_name),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_read(
                object_payload_local,
                object_tag_local,
                object_payload_local,
                object_tag_local,
                key_local,
                hook_value_payload,
                hook_value_tag,
                function,
            )?;
            self.emit_return_current_completion_if_throw(function);
            if *hook_name == "Symbol.toPrimitive" {
                self.emit_is_callable_i32(hook_value_tag, hook_value_payload, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                let hint_payload_local = self.reserve_temp_local();
                let hint_tag_local = self.reserve_temp_local();
                let hint = match hint {
                    ToPrimitiveHint::String => "string",
                    ToPrimitiveHint::Number => "number",
                    ToPrimitiveHint::Default => "default",
                };
                function.instruction(&Instruction::I64Const(self.strings.payload(hint)));
                function.instruction(&Instruction::LocalSet(hint_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(hint_tag_local));
                self.emit_function_or_proxy_call_leave_throw_completion(
                    hook_value_payload,
                    hook_value_tag,
                    object_payload_local,
                    object_tag_local,
                    &[(hint_payload_local, hint_tag_local)],
                    call_result_payload,
                    call_result_tag,
                    function,
                )?;
                self.release_temp_local(hint_tag_local);
                self.release_temp_local(hint_payload_local);
                self.emit_propagate_throw_from_locals_if_needed(
                    call_result_payload,
                    call_result_tag,
                    function,
                )?;
                self.emit_is_primitive_tag_i32(call_result_tag, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(call_result_payload));
                function.instruction(&Instruction::LocalSet(output_payload_local));
                function.instruction(&Instruction::LocalGet(call_result_tag));
                function.instruction(&Instruction::LocalSet(output_tag_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::Else);
                self.emit_throw_current_function_realm_type_error(
                    "Cannot convert object to primitive value",
                    output_payload_local,
                    output_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::LocalGet(hook_value_tag));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_current_function_realm_type_error(
                    "Cannot convert object to primitive value",
                    output_payload_local,
                    output_tag_local,
                    function,
                )?;
                self.emit_return_current_completion(function);
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            } else {
                self.emit_is_callable_i32(hook_value_tag, hook_value_payload, function)?;
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_function_or_proxy_call_leave_throw_completion(
                    hook_value_payload,
                    hook_value_tag,
                    object_payload_local,
                    object_tag_local,
                    &[],
                    call_result_payload,
                    call_result_tag,
                    function,
                )?;
                self.emit_propagate_throw_from_locals_if_needed(
                    call_result_payload,
                    call_result_tag,
                    function,
                )?;
                self.emit_is_primitive_tag_i32(call_result_tag, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(call_result_payload));
                function.instruction(&Instruction::LocalSet(output_payload_local));
                function.instruction(&Instruction::LocalGet(call_result_tag));
                function.instruction(&Instruction::LocalSet(output_tag_local));
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::LocalSet(primitive_result_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            function.instruction(&Instruction::End);
            self.release_temp_local(key_local);
        }

        function.instruction(&Instruction::LocalGet(primitive_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot convert object to primitive value",
            output_payload_local,
            output_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(primitive_result_local);
        self.release_temp_local(call_result_tag);
        self.release_temp_local(call_result_payload);
        self.release_temp_local(hook_value_tag);
        self.release_temp_local(hook_value_payload);
        self.release_temp_local(object_tag_local);
        Ok(())
    }

    pub(crate) fn emit_json_gap_is_non_empty_i32(
        &self,
        gap_payload_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(gap_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
    }

    pub(crate) fn emit_json_normalize_replacer_array(
        &mut self,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let is_array_local = self.reserve_temp_local();
        let len_payload_local = self.reserve_temp_local();
        let len_tag_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let property_list_payload_local = self.reserve_temp_local();
        let property_list_len_local = self.reserve_temp_local();
        let previous_index_local = self.reserve_temp_local();
        let previous_key_payload_local = self.reserve_temp_local();
        let previous_key_tag_local = self.reserve_temp_local();
        let accepted_local = self.reserve_temp_local();
        let duplicate_local = self.reserve_temp_local();

        self.emit_is_array_i64(
            replacer_payload_local,
            replacer_tag_local,
            is_array_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            replacer_payload_local,
            replacer_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            len_payload_local,
            len_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_to_length_i64_from_value_locals(
            len_tag_local,
            len_payload_local,
            len_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(property_list_len_local));
        self.emit_alloc_array_payload_with_length(
            property_list_len_local,
            property_list_payload_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get(
            replacer_payload_local,
            index_local,
            replacer_payload_local,
            replacer_tag_local,
            value_payload_local,
            value_tag_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            replacer_payload_local,
            replacer_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(accepted_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(accepted_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(accepted_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_boxed_object_to_primitive_payload(
            value_payload_local,
            ToPrimitiveHint::String,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_primitive_to_string_payload(
            value_payload_local,
            value_tag_local,
            PrimitiveToStringAbruptRoute::ReturnCurrentFunction,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(accepted_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(accepted_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_local));
        self.load_i64_to_local_from_offset(
            property_list_payload_local,
            HEAP_LEN_OFFSET,
            property_list_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::LocalGet(property_list_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            property_list_payload_local,
            previous_index_local,
            previous_key_payload_local,
            previous_key_tag_local,
            function,
        );
        self.emit_string_payload_equality_i32(
            previous_key_payload_local,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(duplicate_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            property_list_payload_local,
            property_list_len_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(property_list_payload_local));
        function.instruction(&Instruction::LocalSet(replacer_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(replacer_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(duplicate_local);
        self.release_temp_local(accepted_local);
        self.release_temp_local(previous_key_tag_local);
        self.release_temp_local(previous_key_payload_local);
        self.release_temp_local(previous_index_local);
        self.release_temp_local(property_list_len_local);
        self.release_temp_local(property_list_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(len_tag_local);
        self.release_temp_local(len_payload_local);
        self.release_temp_local(is_array_local);
        Ok(())
    }

    pub(crate) fn emit_json_indent_payload(
        &mut self,
        gap_payload_local: u32,
        indent_level_local: u32,
        output_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let counter_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(counter_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(counter_local));
        function.instruction(&Instruction::LocalGet(indent_level_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_concat_string_payloads_local(output_local, gap_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(counter_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(counter_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(counter_local);
        Ok(())
    }

    pub(crate) fn emit_json_append_newline_indent(
        &mut self,
        output_local: u32,
        gap_payload_local: u32,
        indent_level_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let token_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("\n")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_indent_payload(
            gap_payload_local,
            indent_level_local,
            token_local,
            function,
        )?;
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        self.release_temp_local(token_local);
        Ok(())
    }

    pub(crate) fn emit_json_append_optional_newline_indent(
        &mut self,
        output_local: u32,
        gap_payload_local: u32,
        indent_level_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_json_gap_is_non_empty_i32(gap_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_newline_indent(
            output_local,
            gap_payload_local,
            indent_level_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(crate) fn emit_json_append_colon(
        &mut self,
        output_local: u32,
        gap_payload_local: u32,
        token_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_json_gap_is_non_empty_i32(gap_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(": ")));
        function.instruction(&Instruction::LocalSet(token_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(":")));
        function.instruction(&Instruction::LocalSet(token_local));
        function.instruction(&Instruction::End);
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        Ok(())
    }

    /// Pushes `container_payload_local` onto the runtime cycle-detection stack
    /// referenced by `parent_seen_local` (0 = empty), storing the new top-of-stack
    /// node pointer into `dest_local`. Each node carries the container payload,
    /// its parent pointer, and the defining realm inherited from the root
    /// stringify context.
    pub(crate) fn emit_json_push_seen_stack(
        &mut self,
        container_payload_local: u32,
        parent_seen_local: u32,
        dest_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(JSON_STRINGIFY_SEEN_NODE_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(dest_local));
        self.store_i64_local_at_offset(
            dest_local,
            JSON_STRINGIFY_SEEN_VALUE_OFFSET,
            container_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            dest_local,
            JSON_STRINGIFY_SEEN_PARENT_OFFSET,
            parent_seen_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            parent_seen_local,
            JSON_STRINGIFY_SEEN_REALM_OFFSET,
            realm_local,
            function,
        );
        self.store_i64_local_at_offset(
            dest_local,
            JSON_STRINGIFY_SEEN_REALM_OFFSET,
            realm_local,
            function,
        );
        self.release_temp_local(realm_local);
        Ok(())
    }

    pub(crate) fn emit_json_create_seen_root(
        &mut self,
        realm_local: u32,
        root_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_heap_alloc_const(JSON_STRINGIFY_SEEN_NODE_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(root_local));
        self.store_i64_const_at_offset(root_local, JSON_STRINGIFY_SEEN_VALUE_OFFSET, 0, function);
        self.store_i64_const_at_offset(root_local, JSON_STRINGIFY_SEEN_PARENT_OFFSET, 0, function);
        self.store_i64_local_at_offset(
            root_local,
            JSON_STRINGIFY_SEEN_REALM_OFFSET,
            realm_local,
            function,
        );
        Ok(())
    }

    /// Runtime cyclic-structure check. Walks the seen-stack linked list rooted at
    /// `seen_stack_local` and throws a TypeError if the object/array value in
    /// `value_payload_local` is already being serialized (identical heap payload).
    pub(crate) fn emit_json_throw_if_in_seen_stack(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        seen_stack_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let node_local = self.reserve_temp_local();
        let node_payload_local = self.reserve_temp_local();
        let callable_local = self.reserve_temp_local();

        self.emit_is_callable_i32(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(callable_local));

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(callable_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(seen_stack_local));
        function.instruction(&Instruction::LocalSet(node_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(node_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            node_local,
            JSON_STRINGIFY_SEEN_VALUE_OFFSET,
            node_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(node_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalGet(node_payload_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Converting circular structure to JSON",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            node_local,
            JSON_STRINGIFY_SEEN_PARENT_OFFSET,
            node_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(callable_local);
        self.release_temp_local(node_payload_local);
        self.release_temp_local(node_local);
        Ok(())
    }

    pub(crate) fn emit_json_replacer_allows_key(
        &mut self,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        key_payload_local: u32,
        allowed_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let list_payload_local = self.reserve_temp_local();
        let list_tag_local = self.reserve_temp_local();
        let is_array_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let handler_tag_local = self.reserve_temp_local();
        let trap_payload_local = self.reserve_temp_local();
        let trap_tag_local = self.reserve_temp_local();
        let property_key_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let element_payload_local = self.reserve_temp_local();
        let element_tag_local = self.reserve_temp_local();
        let element_method_payload_local = self.reserve_temp_local();
        let element_method_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(allowed_local));
        function.instruction(&Instruction::LocalGet(replacer_payload_local));
        function.instruction(&Instruction::LocalSet(list_payload_local));
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::LocalSet(list_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(is_array_local));

        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            replacer_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            replacer_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            replacer_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            boxed_kind_local,
            handler_tag_local,
            boxed_kind_local,
            handler_tag_local,
            property_key_local,
            trap_payload_local,
            trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(list_payload_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(list_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            target_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(list_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            list_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            list_payload_local,
            list_tag_local,
            list_payload_local,
            list_tag_local,
            property_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(element_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(list_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_index_get(
            list_payload_local,
            index_local,
            list_payload_local,
            list_tag_local,
            element_payload_local,
            element_tag_local,
            None,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            list_payload_local,
            list_tag_local,
            list_payload_local,
            list_tag_local,
            property_key_local,
            element_payload_local,
            element_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(element_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            element_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
        function.instruction(&Instruction::LocalSet(property_key_local));
        self.emit_object_read(
            element_payload_local,
            element_tag_local,
            element_payload_local,
            element_tag_local,
            property_key_local,
            element_method_payload_local,
            element_method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(element_method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            element_method_payload_local,
            element_method_tag_local,
            Some((element_payload_local, Some(element_tag_local))),
            &[],
            element_payload_local,
            element_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_primitive_to_string_payload(
            element_payload_local,
            element_tag_local,
            PrimitiveToStringAbruptRoute::ReturnCurrentFunction,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(element_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(element_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(element_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(element_payload_local, key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(allowed_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(element_method_tag_local);
        self.release_temp_local(element_method_payload_local);
        self.release_temp_local(element_tag_local);
        self.release_temp_local(element_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(property_key_local);
        self.release_temp_local(trap_tag_local);
        self.release_temp_local(trap_payload_local);
        self.release_temp_local(handler_tag_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(is_array_local);
        self.release_temp_local(list_tag_local);
        self.release_temp_local(list_payload_local);
        Ok(())
    }

    pub(crate) fn emit_json_stringify_value_payload(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level_local: u32,
        seen_stack_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let string_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let proxy_array_payload_local = self.reserve_temp_local();
        let proxy_array_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_is_array_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("undefined")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Block(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_RAW_JSON as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("rawJSON")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_local,
            output_local,
            string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_BIGINT as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_throw_bigint_serialization_type_error(
            seen_stack_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_boxed_object_to_primitive_payload(
            value_payload_local,
            ToPrimitiveHint::Number,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_primitive_to_number_payload(value_tag_local, value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_boxed_object_to_primitive_payload(
            value_payload_local,
            ToPrimitiveHint::String,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_primitive_to_string_payload(
            value_payload_local,
            value_tag_local,
            PrimitiveToStringAbruptRoute::ReturnCurrentFunction,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_BOOLEAN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(proxy_array_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_array_tag_local));
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            value_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            boxed_kind_local,
            proxy_handler_tag_local,
            boxed_kind_local,
            proxy_handler_tag_local,
            key_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(proxy_array_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_array_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(proxy_is_array_local));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(proxy_target_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(proxy_is_array_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Proxy handler is null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(proxy_is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_stringify_proxy_array_payload(
            proxy_array_payload_local,
            proxy_array_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            output_local,
            indent_level_local,
            seen_stack_local,
            function,
        )?;
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_json_stringify_object_payload(
            value_payload_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            output_local,
            indent_level_local,
            seen_stack_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_stringify_array_payload(
            value_payload_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            output_local,
            indent_level_local,
            seen_stack_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_quote_string_payload(value_payload_local, output_local, function)?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Abs);
        function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
        function.instruction(&Instruction::F64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        self.emit_number_to_string_payload(value_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("false")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("true")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("null")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_throw_bigint_serialization_type_error(
            seen_stack_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_is_array_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(proxy_array_tag_local);
        self.release_temp_local(proxy_array_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(string_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(brand_local);
        Ok(())
    }

    /// Compiles the shared JSON.stringify value helper. The full SerializeJSON
    /// value/object/array/proxy state machine is emitted once here and reached
    /// with a plain `call`; nested values recurse through the same helper at
    /// runtime (rather than the former compile-time unrolling) so the emitted
    /// function stays far below Cranelift's per-function code-size limit.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`] (seven i64 params, four i64
    /// results). Params: 0=value payload, 1=value tag, 2=replacer payload,
    /// 3=replacer tag, 4=gap payload, 5=indent level, 6=seen-stack pointer.
    /// Results are the `(result, result_tag, completion, completion_aux)` tuple
    /// where a normal completion carries the serialized string payload.
    pub(crate) fn compile_json_stringify_value_helper(&mut self) -> Result<Function, EmitError> {
        // `JsonStringifyValue` is the one helper whose seam stays live inside
        // its own body: nested-value serialization is a real self-call through
        // `emit_json_stringify_value_call`. `begin_helper_body` encodes that in
        // its no-seam arm, so this is a declaration of identity, not a clear.
        let mut function = self.begin_helper_body(RuntimeHelperId::JsonStringifyValue);
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        let output_local = self.reserve_temp_local();
        self.emit_json_stringify_value_payload(0, 1, 2, 3, 4, output_local, 5, 6, &mut function)?;
        function.instruction(&Instruction::LocalGet(output_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(output_local);
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    pub(crate) fn emit_json_stringify_array_payload(
        &mut self,
        array_payload_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level_local: u32,
        seen_stack_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let value_string_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let token_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let array_tag_local = self.reserve_temp_local();
        let child_indent_local = self.reserve_temp_local();
        let nested_seen_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(indent_level_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(child_indent_local));
        self.emit_json_push_seen_stack(
            array_payload_local,
            seen_stack_local,
            nested_seen_local,
            function,
        )?;

        self.load_i64_to_local_from_offset(
            array_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            array_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(array_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("[")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_array_index_get(
            array_payload_local,
            index_local,
            array_payload_local,
            array_tag_local,
            value_payload_local,
            value_tag_local,
            None,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            seen_stack_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            array_payload_local,
            array_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_seen_stack(
            value_payload_local,
            value_tag_local,
            nested_seen_local,
            function,
        )?;
        self.emit_json_array_element_string_payload(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            child_indent_local,
            nested_seen_local,
            function,
        )?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        self.release_temp_local(nested_seen_local);
        self.release_temp_local(child_indent_local);
        self.release_temp_local(array_tag_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(token_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(value_string_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_json_stringify_proxy_array_payload(
        &mut self,
        array_payload_local: u32,
        array_tag_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level_local: u32,
        seen_stack_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let value_string_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let token_local = self.reserve_temp_local();
        let child_indent_local = self.reserve_temp_local();
        let nested_seen_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(indent_level_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(child_indent_local));
        self.emit_json_push_seen_stack(
            array_payload_local,
            seen_stack_local,
            nested_seen_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            array_payload_local,
            array_tag_local,
            array_payload_local,
            array_tag_local,
            key_payload_local,
            length_payload_local,
            length_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(length_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_number_payload(length_tag_local, length_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(length_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(length_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("[")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_object_read(
            array_payload_local,
            array_tag_local,
            array_payload_local,
            array_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            seen_stack_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            array_payload_local,
            array_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_seen_stack(
            value_payload_local,
            value_tag_local,
            nested_seen_local,
            function,
        )?;
        self.emit_json_array_element_string_payload(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            child_indent_local,
            nested_seen_local,
            function,
        )?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));

        self.release_temp_local(nested_seen_local);
        self.release_temp_local(child_indent_local);
        self.release_temp_local(token_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(value_string_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        Ok(())
    }

    pub(crate) fn emit_json_stringify_object_payload(
        &mut self,
        object_payload_local: u32,
        replacer_payload_local: u32,
        replacer_tag_local: u32,
        gap_payload_local: u32,
        output_local: u32,
        indent_level_local: u32,
        seen_stack_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let first_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let key_string_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let value_string_local = self.reserve_temp_local();
        let token_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let keys_function_payload_local = self.reserve_temp_local();
        let keys_function_tag_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let keys_tag_local = self.reserve_temp_local();
        let keys_arg_payload_local = self.reserve_temp_local();
        let keys_arg_tag_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let key_allowed_local = self.reserve_temp_local();
        let completed_local = self.reserve_temp_local();
        let duplicate_key_local = self.reserve_temp_local();
        let previous_index_local = self.reserve_temp_local();
        let previous_key_payload_local = self.reserve_temp_local();
        let previous_key_tag_local = self.reserve_temp_local();
        let child_indent_local = self.reserve_temp_local();
        let nested_seen_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(indent_level_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(child_indent_local));
        self.emit_json_push_seen_stack(
            object_payload_local,
            seen_stack_local,
            nested_seen_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(completed_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(object_payload_local));
        function.instruction(&Instruction::LocalSet(keys_arg_payload_local));
        function.instruction(&Instruction::LocalGet(object_tag_local));
        function.instruction(&Instruction::LocalSet(keys_arg_tag_local));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            proxy_target_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(proxy_handler_tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("ownKeys")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_object_read(
            boxed_kind_local,
            proxy_handler_tag_local,
            boxed_kind_local,
            proxy_handler_tag_local,
            key_payload_local,
            proxy_trap_payload_local,
            proxy_trap_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(proxy_trap_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            proxy_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(proxy_target_payload_local));
        function.instruction(&Instruction::LocalSet(keys_arg_payload_local));
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::LocalSet(keys_arg_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        let keys_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.keys`",
                )
            })?;
        self.emit_function_value_payload(&keys_meta, function)?;
        function.instruction(&Instruction::LocalSet(keys_function_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(keys_function_tag_local));
        self.emit_function_handle_call(
            keys_function_payload_local,
            keys_function_tag_local,
            None,
            &[(keys_arg_payload_local, keys_arg_tag_local)],
            keys_payload_local,
            keys_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(self.strings.payload("{")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(first_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(keys_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            keys_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            keys_payload_local,
            index_local,
            key_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_replacer_allows_key(
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            key_allowed_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(key_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            keys_arg_payload_local,
            keys_arg_tag_local,
            keys_arg_payload_local,
            keys_arg_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            seen_stack_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            keys_arg_payload_local,
            keys_arg_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_seen_stack(
            value_payload_local,
            value_tag_local,
            nested_seen_local,
            function,
        )?;
        self.emit_json_omits_value_i32(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_stringify_value_call(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            child_indent_local,
            nested_seen_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_json_quote_string_payload(key_payload_local, key_string_local, function)?;
        self.emit_concat_string_payloads_local(output_local, key_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_colon(output_local, gap_payload_local, token_local, function)?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("}")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(completed_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(replacer_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("{")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(first_local));
        self.load_i64_to_local_from_offset(
            replacer_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_index_get(
            replacer_payload_local,
            index_local,
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            key_tag_local,
            None,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(key_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            key_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_object_own_data_field_read(
            key_payload_local,
            key_tag_local,
            key_string_local,
            key_allowed_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            value_payload_local,
            value_tag_local,
            Some((key_payload_local, Some(key_tag_local))),
            &[],
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            key_payload_local,
            key_tag_local,
            key_payload_local,
            key_tag_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            value_payload_local,
            value_tag_local,
            Some((key_payload_local, Some(key_tag_local))),
            &[],
            key_payload_local,
            key_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(key_payload_local, key_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_primitive_to_string_payload(
            key_payload_local,
            key_tag_local,
            PrimitiveToStringAbruptRoute::ReturnCurrentFunction,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(duplicate_key_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_index_get(
            replacer_payload_local,
            previous_index_local,
            replacer_payload_local,
            replacer_tag_local,
            previous_key_payload_local,
            previous_key_tag_local,
            None,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(previous_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_number_to_string_payload(previous_key_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(previous_key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(previous_key_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            previous_key_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_NUMBER as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("toString")));
        function.instruction(&Instruction::LocalSet(key_string_local));
        self.emit_object_own_data_field_read(
            previous_key_payload_local,
            previous_key_tag_local,
            key_string_local,
            key_allowed_local,
            value_payload_local,
            value_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            value_payload_local,
            value_tag_local,
            Some((previous_key_payload_local, Some(previous_key_tag_local))),
            &[],
            previous_key_payload_local,
            previous_key_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_object_read(
            previous_key_payload_local,
            previous_key_tag_local,
            previous_key_payload_local,
            previous_key_tag_local,
            key_string_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_function_handle_call(
            value_payload_local,
            value_tag_local,
            Some((previous_key_payload_local, Some(previous_key_tag_local))),
            &[],
            previous_key_payload_local,
            previous_key_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(
            previous_key_payload_local,
            previous_key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(previous_key_payload_local));
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_primitive_to_string_payload(
            previous_key_payload_local,
            previous_key_tag_local,
            PrimitiveToStringAbruptRoute::ReturnCurrentFunction,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(previous_key_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(previous_key_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_string_payload_equality_i32(
            previous_key_payload_local,
            key_payload_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(duplicate_key_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(duplicate_key_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            seen_stack_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_seen_stack(
            value_payload_local,
            value_tag_local,
            nested_seen_local,
            function,
        )?;
        self.emit_json_omits_value_i32(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_stringify_value_call(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            child_indent_local,
            nested_seen_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_json_quote_string_payload(key_payload_local, key_string_local, function)?;
        self.emit_concat_string_payloads_local(output_local, key_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_colon(output_local, gap_payload_local, token_local, function)?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("}")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(completed_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(completed_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            object_payload_local,
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("{")));
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(first_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_json_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_replacer_allows_key(
            replacer_payload_local,
            replacer_tag_local,
            key_payload_local,
            key_allowed_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(key_allowed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_object_read(
            object_payload_local,
            object_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_json_apply_to_json(
            value_payload_local,
            value_tag_local,
            key_payload_local,
            key_tag_local,
            seen_stack_local,
            function,
        )?;
        self.emit_json_apply_replacer_with_this(
            replacer_payload_local,
            replacer_tag_local,
            object_payload_local,
            object_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_json_throw_if_in_seen_stack(
            value_payload_local,
            value_tag_local,
            nested_seen_local,
            function,
        )?;
        self.emit_json_omits_value_i32(value_payload_local, value_tag_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_stringify_value_call(
            value_payload_local,
            value_tag_local,
            replacer_payload_local,
            replacer_tag_local,
            gap_payload_local,
            value_string_local,
            child_indent_local,
            nested_seen_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(",")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(first_local));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            child_indent_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_json_quote_string_payload(key_payload_local, key_string_local, function)?;
        self.emit_concat_string_payloads_local(output_local, key_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        self.emit_json_append_colon(output_local, gap_payload_local, token_local, function)?;
        self.emit_concat_string_payloads_local(output_local, value_string_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(first_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_append_optional_newline_indent(
            output_local,
            gap_payload_local,
            indent_level_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("}")));
        function.instruction(&Instruction::LocalSet(token_local));
        self.emit_concat_string_payloads_local(output_local, token_local, function)?;
        function.instruction(&Instruction::LocalSet(output_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(nested_seen_local);
        self.release_temp_local(child_indent_local);
        self.release_temp_local(previous_key_tag_local);
        self.release_temp_local(previous_key_payload_local);
        self.release_temp_local(previous_index_local);
        self.release_temp_local(duplicate_key_local);
        self.release_temp_local(completed_local);
        self.release_temp_local(key_allowed_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(keys_arg_tag_local);
        self.release_temp_local(keys_arg_payload_local);
        self.release_temp_local(keys_tag_local);
        self.release_temp_local(keys_payload_local);
        self.release_temp_local(keys_function_tag_local);
        self.release_temp_local(keys_function_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(token_local);
        self.release_temp_local(value_string_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_string_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(first_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    pub(crate) fn emit_validate_json_raw_json_text(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let first_byte_local = self.reserve_temp_local();
        let last_byte_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();
        let parsed_flag_local = self.reserve_temp_local();
        let parsed_payload_local = self.reserve_temp_local();
        let parsed_tag_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));

        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(string_offset_local, index_local, first_byte_local, function);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_load_string_byte(string_offset_local, index_local, last_byte_local, function);
        for byte in [b'\t', b'\n', b'\r', b' '] {
            self.emit_or_byte_equals_flag(first_byte_local, byte, invalid_local, function);
            self.emit_or_byte_equals_flag(last_byte_local, byte, invalid_local, function);
        }
        for byte in [b'{', b'['] {
            self.emit_or_byte_equals_flag(first_byte_local, byte, invalid_local, function);
        }
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.rawJSON text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_try_parse_json_string_text(
            string_payload_local,
            parsed_payload_local,
            parsed_tag_local,
            parsed_flag_local,
            function,
        )?;
        self.emit_try_parse_json_keyword_text(
            string_payload_local,
            parsed_payload_local,
            parsed_tag_local,
            parsed_flag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parsed_flag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_validate_json_parse_number_text(string_payload_local, function)?;
        function.instruction(&Instruction::End);

        self.release_temp_local(parsed_tag_local);
        self.release_temp_local(parsed_payload_local);
        self.release_temp_local(parsed_flag_local);
        self.release_temp_local(invalid_local);
        self.release_temp_local(last_byte_local);
        self.release_temp_local(first_byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_validate_json_parse_number_text(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_non_number_start_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'1' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'E' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(invalid_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_validate_json_parse_no_raw_string_controls(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let in_string_local = self.reserve_temp_local();
        let escaped_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);

        function.instruction(&Instruction::LocalGet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_byte_is_json_escape_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(0x20));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(invalid_local);
        self.release_temp_local(escaped_local);
        self.release_temp_local(in_string_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_validate_json_parse_no_structural_trailing_commas(
        &mut self,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let in_string_local = self.reserve_temp_local();
        let escaped_local = self.reserve_temp_local();
        let previous_significant_local = self.reserve_temp_local();
        let structural_depth_local = self.reserve_temp_local();
        let structural_stack_local = self.reserve_temp_local();
        let structural_mask_local = self.reserve_temp_local();
        let structured_seen_local = self.reserve_temp_local();
        let structured_closed_local = self.reserve_temp_local();
        let object_key_needs_colon_local = self.reserve_temp_local();
        let keyword_byte_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();

        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structural_stack_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structural_mask_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structured_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(structured_closed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(object_key_needs_colon_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);

        function.instruction(&Instruction::LocalGet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_byte_is_json_escape_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(escaped_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_stack_local));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(object_key_needs_colon_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structured_closed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(in_string_local));
        function.instruction(&Instruction::Else);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(structured_closed_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(object_key_needs_colon_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(object_key_needs_colon_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(object_key_needs_colon_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_byte_is_json_structural_or_value_start_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(structured_seen_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Shl);
        function.instruction(&Instruction::LocalSet(structural_mask_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_stack_local));
        function.instruction(&Instruction::LocalGet(structural_mask_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(structural_stack_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(structural_stack_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalGet(structural_mask_local));
        function.instruction(&Instruction::I64Xor);
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(structural_stack_local));
        function.instruction(&Instruction::End);
        self.emit_increment_local(structural_depth_local, 1, function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'}' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(structural_stack_local));
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64ShrU);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(structural_mask_local));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'}' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(structural_mask_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(structural_mask_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(structural_depth_local));
        function.instruction(&Instruction::LocalGet(structured_seen_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(structured_closed_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'r' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            4,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 3, function);
        function.instruction(&Instruction::I64Const(b't' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'f' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'a' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            2,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'l' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b's' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            4,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 4, function);
        function.instruction(&Instruction::I64Const(b'f' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'n' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(3));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            2,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'l' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            keyword_byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(keyword_byte_local));
        function.instruction(&Instruction::I64Const(b'l' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 3, function);
        function.instruction(&Instruction::I64Const(b'n' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            keyword_byte_local,
            function,
        );
        self.emit_byte_is_digit_i32(keyword_byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'}' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(previous_significant_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::LocalSet(previous_significant_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(in_string_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'1' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'E' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        for delimiter in [b',', b']', b'}'] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(delimiter as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(in_string_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(structural_depth_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(in_string_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::LocalGet(escaped_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(invalid_local);
        self.release_temp_local(keyword_byte_local);
        self.release_temp_local(object_key_needs_colon_local);
        self.release_temp_local(structured_closed_local);
        self.release_temp_local(structured_seen_local);
        self.release_temp_local(structural_mask_local);
        self.release_temp_local(structural_stack_local);
        self.release_temp_local(structural_depth_local);
        self.release_temp_local(previous_significant_local);
        self.release_temp_local(escaped_local);
        self.release_temp_local(in_string_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    fn emit_json_parse_syntax_error(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_throw_runtime_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }

    fn emit_skip_json_whitespace(
        &self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        byte_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
    }

    fn emit_parse_json_string_at_index(
        &mut self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        value_payload_local: u32,
        invalid_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let byte_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let decoded_len_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let first_hex_local = self.reserve_temp_local();
        let second_hex_local = self.reserve_temp_local();
        let third_hex_local = self.reserve_temp_local();
        let fourth_hex_local = self.reserve_temp_local();

        self.emit_increment_local(index_local, 1, function);
        self.emit_heap_alloc_from_local(string_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(0x20));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(codepoint_local));
        for (escape, value) in [
            (b'"', b'"'),
            (b'\\', b'\\'),
            (b'/', b'/'),
            (b'b', 0x08),
            (b'f', 0x0c),
            (b'n', b'\n'),
            (b'r', b'\r'),
            (b't', b'\t'),
        ] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(escape as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(value as i64));
            function.instruction(&Instruction::LocalSet(codepoint_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            first_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(first_hex_local, first_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            2,
            second_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(second_hex_local, second_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            third_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(third_hex_local, third_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            4,
            fourth_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(fourth_hex_local, fourth_hex_local, function);
        self.emit_all_hex_valid_i32(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pack_four_hex_to_code_unit(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            codepoint_local,
            function,
        );
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        self.emit_increment_local(index_local, 4, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_byte_local(dst_pos_local, codepoint_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_store_byte_local(dst_pos_local, byte_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::BrIf(1));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(decoded_len_local));
        self.emit_pack_string_payload(dst_offset_local, decoded_len_local, function);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(fourth_hex_local);
        self.release_temp_local(third_hex_local);
        self.release_temp_local(second_hex_local);
        self.release_temp_local(first_hex_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(decoded_len_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(byte_local);
        Ok(())
    }

    fn emit_push_json_parse_frame(
        &mut self,
        frame_buffer_local: u32,
        frame_capacity_local: u32,
        frame_len_local: u32,
        payload_local: u32,
        tag_local: u32,
        state_local: u32,
        key_or_index_local: u32,
        metadata_payload_local: u32,
        metadata_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_capacity_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let copy_index_local = self.reserve_temp_local();
        let old_frame_local = self.reserve_temp_local();
        let new_frame_local = self.reserve_temp_local();
        let frame_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::LocalGet(frame_capacity_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frame_capacity_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_capacity_local));
        function.instruction(&Instruction::LocalGet(new_capacity_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_frame_local));
        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_frame_local));
        for offset in [
            JSON_PARSE_FRAME_PAYLOAD_OFFSET,
            JSON_PARSE_FRAME_TAG_OFFSET,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_FRAME_KEY_OR_INDEX_OFFSET,
            JSON_PARSE_FRAME_METADATA_PAYLOAD_OFFSET,
            JSON_PARSE_FRAME_METADATA_TAG_OFFSET,
        ] {
            self.load_i64_from_offset(old_frame_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_frame_local, offset, self.scratch_local, function);
        }
        self.emit_increment_local(copy_index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(new_capacity_local));
        function.instruction(&Instruction::LocalSet(frame_capacity_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frame_local));
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            state_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_KEY_OR_INDEX_OFFSET,
            key_or_index_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_METADATA_PAYLOAD_OFFSET,
            metadata_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_METADATA_TAG_OFFSET,
            metadata_tag_local,
            function,
        );
        self.emit_increment_local(frame_len_local, 1, function);

        self.release_temp_local(frame_local);
        self.release_temp_local(new_frame_local);
        self.release_temp_local(old_frame_local);
        self.release_temp_local(copy_index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(new_capacity_local);
        Ok(())
    }

    fn emit_json_literal_matches_i32(
        &self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        byte_local: u32,
        literal: &[u8],
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(literal.len() as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I32)));
        function.instruction(&Instruction::I32Const(1));
        for (delta, expected) in literal.iter().enumerate() {
            self.emit_load_string_byte_at_delta(
                string_offset_local,
                index_local,
                delta as i64,
                byte_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(*expected as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32And);
        }
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I32Const(0));
        function.instruction(&Instruction::End);
    }

    fn emit_parse_json_number_at_index(
        &mut self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        invalid_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let start_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let token_offset_local = self.reserve_temp_local();
        let token_len_local = self.reserve_temp_local();
        let token_payload_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(start_local));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'0' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'1' as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'9' as i64));
        function.instruction(&Instruction::I64LeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'.' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'e' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'E' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'+' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_advance_json_parse_digit_run(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(token_offset_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(token_len_local));
        self.emit_pack_string_payload(token_offset_local, token_len_local, function);
        function.instruction(&Instruction::LocalSet(token_payload_local));
        self.emit_decimal_to_binary64_payload(token_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(token_payload_local);
        self.release_temp_local(token_len_local);
        self.release_temp_local(token_offset_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(start_local);
        Ok(())
    }

    fn emit_parse_json_value_at_index(
        &mut self,
        string_offset_local: u32,
        string_len_local: u32,
        index_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        nested_state_local: u32,
        invalid_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let byte_local = self.reserve_temp_local();
        let empty_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(nested_state_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_parse_json_string_at_index(
            string_offset_local,
            string_len_local,
            index_local,
            value_payload_local,
            invalid_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'[' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(empty_len_local));
        self.emit_alloc_array_payload_with_length(empty_len_local, value_payload_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_ARRAY_FIRST_OR_END));
        function.instruction(&Instruction::LocalSet(nested_state_local));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'{' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_OBJECT_FIRST_KEY_OR_END));
        function.instruction(&Instruction::LocalSet(nested_state_local));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        for (literal, payload, tag) in [
            (&b"null"[..], 0, ValueKind::Null),
            (&b"false"[..], 0, ValueKind::Boolean),
            (&b"true"[..], 1, ValueKind::Boolean),
        ] {
            self.emit_json_literal_matches_i32(
                string_offset_local,
                string_len_local,
                index_local,
                byte_local,
                literal,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(payload));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(tag.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(literal.len() as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(index_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }

        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'-' as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_byte_is_digit_i32(byte_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_parse_json_number_at_index(
            string_offset_local,
            string_len_local,
            index_local,
            value_payload_local,
            value_tag_local,
            invalid_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(empty_len_local);
        self.release_temp_local(byte_local);
        Ok(())
    }

    fn emit_json_array_append(
        &mut self,
        array_local: u32,
        payload_local: u32,
        tag_local: u32,
        final_element_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let capacity_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(array_local, HEAP_CAP_OFFSET, capacity_local, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::LocalGet(capacity_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(final_element_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64DivU);
        function.instruction(&Instruction::LocalSet(capacity_local));
        function.instruction(&Instruction::End);
        self.emit_array_grow_buffer(
            array_local,
            buffer_local,
            len_local,
            capacity_local,
            len_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.store_i64_local_at_offset(entry_local, HEAP_ARRAY_TAG_OFFSET, tag_local, function);
        self.store_i64_local_at_offset(
            entry_local,
            HEAP_ARRAY_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_const_at_offset(
            entry_local,
            HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
            ARRAY_DESCRIPTOR_NORMAL_DATA,
            function,
        );
        self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_SETTER_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(entry_local, HEAP_ARRAY_SETTER_PAYLOAD_OFFSET, 0, function);
        self.emit_increment_local(len_local, 1, function);
        self.store_i64_local_at_offset(array_local, HEAP_LEN_OFFSET, len_local, function);

        self.release_temp_local(entry_local);
        self.release_temp_local(capacity_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_finalize_json_array_present_indexes(
        &mut self,
        array_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let present_buffer_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let present_entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(array_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(present_buffer_local));
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::LocalGet(present_buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_ARRAY_PRESENT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(present_entry_local));
        self.store_i64_local_at_offset(
            present_entry_local,
            HEAP_ARRAY_PRESENT_ENTRY_INDEX_OFFSET,
            index_local,
            function,
        );
        for (source_offset, destination_offset) in [
            (HEAP_ARRAY_TAG_OFFSET, HEAP_ARRAY_PRESENT_ENTRY_TAG_OFFSET),
            (
                HEAP_ARRAY_PAYLOAD_OFFSET,
                HEAP_ARRAY_PRESENT_ENTRY_PAYLOAD_OFFSET,
            ),
            (
                HEAP_ARRAY_SETTER_TAG_OFFSET,
                HEAP_ARRAY_PRESENT_ENTRY_SETTER_TAG_OFFSET,
            ),
            (
                HEAP_ARRAY_SETTER_PAYLOAD_OFFSET,
                HEAP_ARRAY_PRESENT_ENTRY_SETTER_PAYLOAD_OFFSET,
            ),
            (
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                HEAP_ARRAY_PRESENT_ENTRY_DESCRIPTOR_KIND_OFFSET,
            ),
        ] {
            self.load_i64_from_offset(entry_local, source_offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                present_entry_local,
                destination_offset,
                self.scratch_local,
                function,
            );
        }
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_PTR_OFFSET,
            present_buffer_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_LEN_OFFSET,
            len_local,
            function,
        );
        self.store_i64_local_at_offset(
            array_local,
            HEAP_ARRAY_PRESENT_INDEXES_CAP_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(present_entry_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(present_buffer_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        Ok(())
    }

    fn emit_alloc_json_parse_metadata(
        &mut self,
        string_offset_local: u32,
        value_start_local: u32,
        value_end_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        metadata_local: u32,
        children_payload_local: u32,
        children_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let source_offset_local = self.reserve_temp_local();
        let source_len_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let empty_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(source_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(children_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(children_tag_local));

        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(empty_len_local));
        self.emit_alloc_array_payload_with_length(
            empty_len_local,
            children_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(children_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(children_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(children_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(string_offset_local));
        function.instruction(&Instruction::LocalGet(value_start_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(source_offset_local));
        function.instruction(&Instruction::LocalGet(value_end_local));
        function.instruction(&Instruction::LocalGet(value_start_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(source_len_local));
        self.emit_pack_string_payload(source_offset_local, source_len_local, function);
        function.instruction(&Instruction::LocalSet(source_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_heap_alloc_const(JSON_PARSE_METADATA_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(metadata_local));
        self.store_i64_local_at_offset(
            metadata_local,
            JSON_PARSE_METADATA_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            metadata_local,
            JSON_PARSE_METADATA_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            metadata_local,
            JSON_PARSE_METADATA_SOURCE_OFFSET,
            source_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            metadata_local,
            JSON_PARSE_METADATA_CHILDREN_PAYLOAD_OFFSET,
            children_payload_local,
            function,
        );
        self.store_i64_local_at_offset(
            metadata_local,
            JSON_PARSE_METADATA_CHILDREN_TAG_OFFSET,
            children_tag_local,
            function,
        );

        self.release_temp_local(empty_len_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(source_len_local);
        self.release_temp_local(source_offset_local);
        Ok(())
    }

    fn emit_store_json_reviver_state(
        &self,
        frame_local: u32,
        state: JsonReviverFrameState,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            frame_local,
            JSON_REVIVER_FRAME_STATE_OFFSET,
            state.word(),
            function,
        );
    }

    fn emit_push_json_reviver_frame(
        &mut self,
        frame_buffer_local: u32,
        frame_capacity_local: u32,
        frame_len_local: u32,
        holder_payload_local: u32,
        holder_tag_local: u32,
        key_payload_local: u32,
        key_index_local: u32,
        metadata_local: u32,
        role: JsonReviverPropertyRole,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_capacity_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let new_buffer_local = self.reserve_temp_local();
        let copy_index_local = self.reserve_temp_local();
        let old_frame_local = self.reserve_temp_local();
        let new_frame_local = self.reserve_temp_local();
        let frame_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::LocalGet(frame_capacity_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frame_capacity_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_capacity_local));
        function.instruction(&Instruction::LocalGet(new_capacity_local));
        function.instruction(&Instruction::I64Const(JSON_REVIVER_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_buffer_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(copy_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(JSON_REVIVER_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(old_frame_local));
        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalGet(copy_index_local));
        function.instruction(&Instruction::I64Const(JSON_REVIVER_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(new_frame_local));
        for offset in (0..JSON_REVIVER_FRAME_SIZE).step_by(8) {
            self.load_i64_from_offset(old_frame_local, offset, function);
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(new_frame_local, offset, self.scratch_local, function);
        }
        self.emit_increment_local(copy_index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_buffer_local));
        function.instruction(&Instruction::LocalSet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(new_capacity_local));
        function.instruction(&Instruction::LocalSet(frame_capacity_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(JSON_REVIVER_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frame_local));
        for (offset, local) in [
            (
                JSON_REVIVER_FRAME_HOLDER_PAYLOAD_OFFSET,
                holder_payload_local,
            ),
            (JSON_REVIVER_FRAME_HOLDER_TAG_OFFSET, holder_tag_local),
            (JSON_REVIVER_FRAME_KEY_PAYLOAD_OFFSET, key_payload_local),
            (JSON_REVIVER_FRAME_KEY_INDEX_OFFSET, key_index_local),
            (JSON_REVIVER_FRAME_METADATA_OFFSET, metadata_local),
        ] {
            self.store_i64_local_at_offset(frame_local, offset, local, function);
        }
        self.store_i64_const_at_offset(
            frame_local,
            JSON_REVIVER_FRAME_ROLE_OFFSET,
            role.word(),
            function,
        );
        for (offset, value) in [
            (JSON_REVIVER_FRAME_VALUE_PAYLOAD_OFFSET, 0),
            (
                JSON_REVIVER_FRAME_VALUE_TAG_OFFSET,
                ValueKind::Undefined.tag() as u64,
            ),
            (JSON_REVIVER_FRAME_CURSOR_OFFSET, 0),
            (JSON_REVIVER_FRAME_LIMIT_OFFSET, 0),
            (JSON_REVIVER_FRAME_KEYS_PAYLOAD_OFFSET, 0),
        ] {
            self.store_i64_const_at_offset(frame_local, offset, value, function);
        }
        self.emit_store_json_reviver_state(frame_local, JsonReviverFrameState::Enter, function);
        self.emit_increment_local(frame_len_local, 1, function);

        self.release_temp_local(frame_local);
        self.release_temp_local(new_frame_local);
        self.release_temp_local(old_frame_local);
        self.release_temp_local(copy_index_local);
        self.release_temp_local(new_buffer_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(new_capacity_local);
        Ok(())
    }

    fn emit_json_reviver_metadata_child(
        &mut self,
        metadata_local: u32,
        key_payload_local: u32,
        key_index_local: u32,
        child_metadata_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let children_payload_local = self.reserve_temp_local();
        let children_tag_local = self.reserve_temp_local();
        let child_payload_local = self.reserve_temp_local();
        let child_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(child_metadata_local));
        function.instruction(&Instruction::LocalGet(metadata_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            metadata_local,
            JSON_PARSE_METADATA_CHILDREN_PAYLOAD_OFFSET,
            children_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            metadata_local,
            JSON_PARSE_METADATA_CHILDREN_TAG_OFFSET,
            children_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::LocalGet(children_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_read(
            children_payload_local,
            key_index_local,
            child_payload_local,
            child_tag_local,
            function,
        );
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(children_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            children_payload_local,
            children_tag_local,
            children_payload_local,
            children_tag_local,
            key_payload_local,
            child_payload_local,
            child_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(child_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(child_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(child_payload_local));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncSatF64U);
        function.instruction(&Instruction::LocalSet(child_metadata_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(child_tag_local);
        self.release_temp_local(child_payload_local);
        self.release_temp_local(children_tag_local);
        self.release_temp_local(children_payload_local);
        Ok(())
    }

    pub(crate) fn emit_json_internalize_dynamic(
        &mut self,
        root_payload_local: u32,
        root_tag_local: u32,
        root_metadata_local: u32,
        reviver_payload_local: u32,
        reviver_tag_local: u32,
        result_payload_local: u32,
        result_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let frame_buffer_local = self.reserve_temp_local();
        let frame_capacity_local = self.reserve_temp_local();
        let frame_len_local = self.reserve_temp_local();
        let frame_local = self.reserve_temp_local();
        let holder_payload_local = self.reserve_temp_local();
        let holder_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_index_local = self.reserve_temp_local();
        let metadata_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let limit_local = self.reserve_temp_local();
        let keys_payload_local = self.reserve_temp_local();
        let role_local = self.reserve_temp_local();
        let child_metadata_local = self.reserve_temp_local();
        let child_key_payload_local = self.reserve_temp_local();
        let child_key_index_local = self.reserve_temp_local();
        let expected_payload_local = self.reserve_temp_local();
        let expected_tag_local = self.reserve_temp_local();
        let is_array_local = self.reserve_temp_local();
        let length_payload_local = self.reserve_temp_local();
        let length_tag_local = self.reserve_temp_local();
        let context_payload_local = self.reserve_temp_local();
        let context_tag_local = self.reserve_temp_local();
        let source_payload_local = self.reserve_temp_local();
        let source_key_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let source_tag_local = self.reserve_temp_local();
        let object_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectKeys.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.keys`",
                )
            })?;

        function.instruction(&Instruction::I64Const(
            JSON_REVIVER_INITIAL_FRAME_CAPACITY as i64,
        ));
        function.instruction(&Instruction::LocalSet(frame_capacity_local));
        function.instruction(&Instruction::I64Const(
            (JSON_REVIVER_INITIAL_FRAME_CAPACITY * JSON_REVIVER_FRAME_SIZE) as i64,
        ));
        function.instruction(&Instruction::LocalSet(frame_len_local));
        self.emit_heap_alloc_from_local(frame_len_local, function)?;
        function.instruction(&Instruction::LocalSet(frame_buffer_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(frame_len_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(key_index_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(source_tag_local));
        self.emit_push_json_reviver_frame(
            frame_buffer_local,
            frame_capacity_local,
            frame_len_local,
            root_payload_local,
            root_tag_local,
            key_payload_local,
            key_index_local,
            root_metadata_local,
            JsonReviverPropertyRole::Root,
            function,
        )?;

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(JSON_REVIVER_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frame_local));
        for (offset, local) in [
            (
                JSON_REVIVER_FRAME_HOLDER_PAYLOAD_OFFSET,
                holder_payload_local,
            ),
            (JSON_REVIVER_FRAME_HOLDER_TAG_OFFSET, holder_tag_local),
            (JSON_REVIVER_FRAME_KEY_PAYLOAD_OFFSET, key_payload_local),
            (JSON_REVIVER_FRAME_KEY_INDEX_OFFSET, key_index_local),
            (JSON_REVIVER_FRAME_METADATA_OFFSET, metadata_local),
            (JSON_REVIVER_FRAME_VALUE_PAYLOAD_OFFSET, value_payload_local),
            (JSON_REVIVER_FRAME_VALUE_TAG_OFFSET, value_tag_local),
            (JSON_REVIVER_FRAME_STATE_OFFSET, state_local),
            (JSON_REVIVER_FRAME_CURSOR_OFFSET, cursor_local),
            (JSON_REVIVER_FRAME_LIMIT_OFFSET, limit_local),
            (JSON_REVIVER_FRAME_KEYS_PAYLOAD_OFFSET, keys_payload_local),
            (JSON_REVIVER_FRAME_ROLE_OFFSET, role_local),
        ] {
            self.load_i64_to_local_from_offset(frame_local, offset, local, function);
        }

        for state in JsonReviverFrameState::ALL.iter().copied() {
            function.instruction(&Instruction::LocalGet(state_local));
            function.instruction(&Instruction::I64Const(state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            match state {
                JsonReviverFrameState::Enter => {
                    function.instruction(&Instruction::LocalGet(key_index_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64GeS);
                    function.instruction(&Instruction::LocalGet(holder_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32And);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_index_get_with_prototype(
                        holder_payload_local,
                        key_index_local,
                        holder_payload_local,
                        holder_tag_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    self.emit_return_current_completion_if_throw(function);
                    function.instruction(&Instruction::Else);
                    self.emit_object_read(
                        holder_payload_local,
                        holder_tag_local,
                        holder_payload_local,
                        holder_tag_local,
                        key_payload_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.emit_propagate_throw_from_locals_if_needed(
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    self.store_i64_local_at_offset(
                        frame_local,
                        JSON_REVIVER_FRAME_VALUE_PAYLOAD_OFFSET,
                        value_payload_local,
                        function,
                    );
                    self.store_i64_local_at_offset(
                        frame_local,
                        JSON_REVIVER_FRAME_VALUE_TAG_OFFSET,
                        value_tag_local,
                        function,
                    );

                    function.instruction(&Instruction::LocalGet(metadata_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::Else);
                    self.load_i64_to_local_from_offset(
                        metadata_local,
                        JSON_PARSE_METADATA_VALUE_PAYLOAD_OFFSET,
                        expected_payload_local,
                        function,
                    );
                    self.load_i64_to_local_from_offset(
                        metadata_local,
                        JSON_PARSE_METADATA_VALUE_TAG_OFFSET,
                        expected_tag_local,
                        function,
                    );
                    self.emit_tagged_payload_same_value_i32(
                        value_tag_local,
                        value_payload_local,
                        expected_tag_local,
                        expected_payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::I32Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(metadata_local));
                    self.store_i64_const_at_offset(
                        frame_local,
                        JSON_REVIVER_FRAME_METADATA_OFFSET,
                        0,
                        function,
                    );
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::End);

                    function.instruction(&Instruction::LocalGet(value_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::LocalGet(value_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Or);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_is_array_i64(
                        value_payload_local,
                        value_tag_local,
                        is_array_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(is_array_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_direct_js_call(
                        &object_keys_meta,
                        None,
                        &[(value_payload_local, value_tag_local)],
                        keys_payload_local,
                        key_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        keys_payload_local,
                        key_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(limit_local));
                    function.instruction(&Instruction::LocalGet(key_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.load_i64_to_local_from_offset(
                        keys_payload_local,
                        HEAP_LEN_OFFSET,
                        limit_local,
                        function,
                    );
                    function.instruction(&Instruction::End);
                    self.store_i64_local_at_offset(
                        frame_local,
                        JSON_REVIVER_FRAME_KEYS_PAYLOAD_OFFSET,
                        keys_payload_local,
                        function,
                    );
                    self.store_i64_local_at_offset(
                        frame_local,
                        JSON_REVIVER_FRAME_LIMIT_OFFSET,
                        limit_local,
                        function,
                    );
                    self.emit_store_json_reviver_state(
                        frame_local,
                        JsonReviverFrameState::ObjectChildren,
                        function,
                    );
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(self.strings.payload("length")));
                    function.instruction(&Instruction::LocalSet(child_key_payload_local));
                    self.emit_object_read(
                        value_payload_local,
                        value_tag_local,
                        value_payload_local,
                        value_tag_local,
                        child_key_payload_local,
                        length_payload_local,
                        length_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        length_payload_local,
                        length_tag_local,
                        function,
                    )?;
                    self.emit_to_length_i64_from_value_locals(
                        length_tag_local,
                        length_payload_local,
                        limit_local,
                        function,
                    )?;
                    self.store_i64_local_at_offset(
                        frame_local,
                        JSON_REVIVER_FRAME_LIMIT_OFFSET,
                        limit_local,
                        function,
                    );
                    self.emit_store_json_reviver_state(
                        frame_local,
                        JsonReviverFrameState::ArrayChildren,
                        function,
                    );
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::Else);
                    self.emit_store_json_reviver_state(
                        frame_local,
                        JsonReviverFrameState::Apply,
                        function,
                    );
                    function.instruction(&Instruction::End);
                }
                JsonReviverFrameState::ArrayChildren => {
                    function.instruction(&Instruction::LocalGet(cursor_local));
                    function.instruction(&Instruction::LocalGet(limit_local));
                    function.instruction(&Instruction::I64LtU);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::LocalGet(cursor_local));
                    function.instruction(&Instruction::LocalSet(child_key_index_local));
                    function.instruction(&Instruction::LocalGet(cursor_local));
                    function.instruction(&Instruction::F64ConvertI64U);
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(child_key_payload_local));
                    self.emit_number_to_string_payload(child_key_payload_local, function)?;
                    function.instruction(&Instruction::LocalSet(child_key_payload_local));
                    self.emit_json_reviver_metadata_child(
                        metadata_local,
                        child_key_payload_local,
                        child_key_index_local,
                        child_metadata_local,
                        function,
                    )?;
                    self.emit_increment_local(cursor_local, 1, function);
                    self.store_i64_local_at_offset(
                        frame_local,
                        JSON_REVIVER_FRAME_CURSOR_OFFSET,
                        cursor_local,
                        function,
                    );
                    self.emit_push_json_reviver_frame(
                        frame_buffer_local,
                        frame_capacity_local,
                        frame_len_local,
                        value_payload_local,
                        value_tag_local,
                        child_key_payload_local,
                        child_key_index_local,
                        child_metadata_local,
                        JsonReviverPropertyRole::Nested,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_store_json_reviver_state(
                        frame_local,
                        JsonReviverFrameState::Apply,
                        function,
                    );
                    function.instruction(&Instruction::End);
                }
                JsonReviverFrameState::ObjectChildren => {
                    function.instruction(&Instruction::LocalGet(cursor_local));
                    function.instruction(&Instruction::LocalGet(limit_local));
                    function.instruction(&Instruction::I64LtU);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_array_read(
                        keys_payload_local,
                        cursor_local,
                        child_key_payload_local,
                        key_tag_local,
                        function,
                    );
                    function.instruction(&Instruction::I64Const(-1));
                    function.instruction(&Instruction::LocalSet(child_key_index_local));
                    self.emit_json_reviver_metadata_child(
                        metadata_local,
                        child_key_payload_local,
                        child_key_index_local,
                        child_metadata_local,
                        function,
                    )?;
                    self.emit_increment_local(cursor_local, 1, function);
                    self.store_i64_local_at_offset(
                        frame_local,
                        JSON_REVIVER_FRAME_CURSOR_OFFSET,
                        cursor_local,
                        function,
                    );
                    self.emit_push_json_reviver_frame(
                        frame_buffer_local,
                        frame_capacity_local,
                        frame_len_local,
                        value_payload_local,
                        value_tag_local,
                        child_key_payload_local,
                        child_key_index_local,
                        child_metadata_local,
                        JsonReviverPropertyRole::Nested,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_store_json_reviver_state(
                        frame_local,
                        JsonReviverFrameState::Apply,
                        function,
                    );
                    function.instruction(&Instruction::End);
                }
                JsonReviverFrameState::Apply => {
                    self.emit_alloc_plain_object_with_prototype(
                        None,
                        Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(context_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(context_tag_local));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(source_payload_local));
                    function.instruction(&Instruction::LocalGet(metadata_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::Else);
                    self.load_i64_to_local_from_offset(
                        metadata_local,
                        JSON_PARSE_METADATA_SOURCE_OFFSET,
                        source_payload_local,
                        function,
                    );
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(source_payload_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    function.instruction(&Instruction::Else);
                    function.instruction(&Instruction::I64Const(self.strings.payload("source")));
                    function.instruction(&Instruction::LocalSet(source_key_local));
                    self.emit_object_define_enumerable_data(
                        context_payload_local,
                        source_key_local,
                        source_payload_local,
                        source_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                    function.instruction(&Instruction::LocalSet(key_tag_local));
                    self.emit_indirect_call_from_locals(
                        reviver_payload_local,
                        reviver_tag_local,
                        Some((holder_payload_local, holder_tag_local)),
                        &[
                            (key_payload_local, key_tag_local),
                            (value_payload_local, value_tag_local),
                            (context_payload_local, context_tag_local),
                        ],
                        result_payload_local,
                        result_tag_local,
                        function,
                    )?;
                    self.emit_propagate_throw_from_locals_if_needed(
                        result_payload_local,
                        result_tag_local,
                        function,
                    )?;
                    self.set_completion_kind(CompletionKind::Normal, function);

                    function.instruction(&Instruction::Block(BlockType::Empty));
                    for role in JsonReviverPropertyRole::ALL.iter().copied() {
                        function.instruction(&Instruction::LocalGet(role_local));
                        function.instruction(&Instruction::I64Const(role.word() as i64));
                        function.instruction(&Instruction::I64Eq);
                        function.instruction(&Instruction::If(BlockType::Empty));
                        self.emit_json_apply_reviver_result(
                            role,
                            holder_payload_local,
                            holder_tag_local,
                            key_payload_local,
                            Some(key_index_local),
                            result_payload_local,
                            result_tag_local,
                            function,
                        )?;
                        function.instruction(&Instruction::Br(1));
                        function.instruction(&Instruction::End);
                    }
                    function.instruction(&Instruction::Unreachable);
                    function.instruction(&Instruction::End);
                    function.instruction(&Instruction::LocalGet(frame_len_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Sub);
                    function.instruction(&Instruction::LocalSet(frame_len_local));
                }
            }
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(source_tag_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(source_key_local);
        self.release_temp_local(source_payload_local);
        self.release_temp_local(context_tag_local);
        self.release_temp_local(context_payload_local);
        self.release_temp_local(length_tag_local);
        self.release_temp_local(length_payload_local);
        self.release_temp_local(is_array_local);
        self.release_temp_local(expected_tag_local);
        self.release_temp_local(expected_payload_local);
        self.release_temp_local(child_key_index_local);
        self.release_temp_local(child_key_payload_local);
        self.release_temp_local(child_metadata_local);
        self.release_temp_local(role_local);
        self.release_temp_local(keys_payload_local);
        self.release_temp_local(limit_local);
        self.release_temp_local(cursor_local);
        self.release_temp_local(state_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(metadata_local);
        self.release_temp_local(key_index_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(holder_tag_local);
        self.release_temp_local(holder_payload_local);
        self.release_temp_local(frame_local);
        self.release_temp_local(frame_len_local);
        self.release_temp_local(frame_capacity_local);
        self.release_temp_local(frame_buffer_local);
        Ok(())
    }

    pub(crate) fn emit_try_parse_json_text(
        &mut self,
        string_payload_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        parsed_flag_local: u32,
        capture_metadata_local: u32,
        root_metadata_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();
        let nested_state_local = self.reserve_temp_local();
        let frame_buffer_local = self.reserve_temp_local();
        let frame_capacity_local = self.reserve_temp_local();
        let frame_len_local = self.reserve_temp_local();
        let frame_local = self.reserve_temp_local();
        let frame_payload_local = self.reserve_temp_local();
        let frame_tag_local = self.reserve_temp_local();
        let frame_state_local = self.reserve_temp_local();
        let key_or_index_local = self.reserve_temp_local();
        let frame_metadata_payload_local = self.reserve_temp_local();
        let frame_metadata_tag_local = self.reserve_temp_local();
        let root_payload_local = self.reserve_temp_local();
        let root_tag_local = self.reserve_temp_local();
        let lookahead_index_local = self.reserve_temp_local();
        let final_array_element_local = self.reserve_temp_local();
        let value_start_local = self.reserve_temp_local();
        let metadata_local = self.reserve_temp_local();
        let metadata_children_payload_local = self.reserve_temp_local();
        let metadata_children_tag_local = self.reserve_temp_local();
        let metadata_value_payload_local = self.reserve_temp_local();
        let metadata_value_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(parsed_flag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(root_metadata_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_skip_json_whitespace(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(parsed_flag_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(value_start_local));
        self.emit_parse_json_value_at_index(
            string_offset_local,
            string_len_local,
            index_local,
            value_payload_local,
            value_tag_local,
            nested_state_local,
            invalid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(root_payload_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(root_tag_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(metadata_children_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(metadata_children_tag_local));
        function.instruction(&Instruction::LocalGet(capture_metadata_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_json_parse_metadata(
            string_offset_local,
            value_start_local,
            index_local,
            value_payload_local,
            value_tag_local,
            root_metadata_local,
            metadata_children_payload_local,
            metadata_children_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            JSON_PARSE_INITIAL_FRAME_CAPACITY as i64,
        ));
        function.instruction(&Instruction::LocalSet(frame_capacity_local));
        function.instruction(&Instruction::I64Const(
            (JSON_PARSE_INITIAL_FRAME_CAPACITY * JSON_PARSE_FRAME_SIZE) as i64,
        ));
        function.instruction(&Instruction::LocalSet(frame_len_local));
        self.emit_heap_alloc_from_local(frame_len_local, function)?;
        function.instruction(&Instruction::LocalSet(frame_buffer_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(frame_len_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_or_index_local));
        function.instruction(&Instruction::LocalGet(nested_state_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_push_json_parse_frame(
            frame_buffer_local,
            frame_capacity_local,
            frame_len_local,
            value_payload_local,
            value_tag_local,
            nested_state_local,
            key_or_index_local,
            metadata_children_payload_local,
            metadata_children_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_skip_json_whitespace(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(frame_buffer_local));
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::I64Const(JSON_PARSE_FRAME_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(frame_local));
        self.load_i64_to_local_from_offset(
            frame_local,
            JSON_PARSE_FRAME_PAYLOAD_OFFSET,
            frame_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            frame_local,
            JSON_PARSE_FRAME_TAG_OFFSET,
            frame_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            frame_state_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            frame_local,
            JSON_PARSE_FRAME_KEY_OR_INDEX_OFFSET,
            key_or_index_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            frame_local,
            JSON_PARSE_FRAME_METADATA_PAYLOAD_OFFSET,
            frame_metadata_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            frame_local,
            JSON_PARSE_FRAME_METADATA_TAG_OFFSET,
            frame_metadata_tag_local,
            function,
        );

        function.instruction(&Instruction::LocalGet(frame_state_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_ARRAY_FIRST_OR_END));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalize_json_array_present_indexes(frame_payload_local, function)?;
        function.instruction(&Instruction::LocalGet(capture_metadata_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalize_json_array_present_indexes(frame_metadata_payload_local, function)?;
        function.instruction(&Instruction::End);
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(frame_len_local));
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_ARRAY_VALUE as u64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_state_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_ARRAY_VALUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(value_start_local));
        self.emit_parse_json_value_at_index(
            string_offset_local,
            string_len_local,
            index_local,
            value_payload_local,
            value_tag_local,
            nested_state_local,
            invalid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(final_array_element_local));
        function.instruction(&Instruction::LocalGet(nested_state_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64LtS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(lookahead_index_local));
        self.emit_skip_json_whitespace(
            string_offset_local,
            string_len_local,
            lookahead_index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(lookahead_index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(
            string_offset_local,
            lookahead_index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(final_array_element_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_json_array_append(
            frame_payload_local,
            value_payload_local,
            value_tag_local,
            final_array_element_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(metadata_children_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(metadata_children_tag_local));
        function.instruction(&Instruction::LocalGet(capture_metadata_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_json_parse_metadata(
            string_offset_local,
            value_start_local,
            index_local,
            value_payload_local,
            value_tag_local,
            metadata_local,
            metadata_children_payload_local,
            metadata_children_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(metadata_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(metadata_value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(metadata_value_tag_local));
        self.emit_json_array_append(
            frame_metadata_payload_local,
            metadata_value_payload_local,
            metadata_value_tag_local,
            final_array_element_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_ARRAY_COMMA_OR_END as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(nested_state_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_or_index_local));
        self.emit_push_json_parse_frame(
            frame_buffer_local,
            frame_capacity_local,
            frame_len_local,
            value_payload_local,
            value_tag_local,
            nested_state_local,
            key_or_index_local,
            metadata_children_payload_local,
            metadata_children_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_state_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_ARRAY_COMMA_OR_END));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b']' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalize_json_array_present_indexes(frame_payload_local, function)?;
        function.instruction(&Instruction::LocalGet(capture_metadata_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_finalize_json_array_present_indexes(frame_metadata_payload_local, function)?;
        function.instruction(&Instruction::End);
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(frame_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        self.store_i64_const_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_ARRAY_VALUE as u64,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_state_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_OBJECT_FIRST_KEY_OR_END));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'}' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(frame_len_local));
        function.instruction(&Instruction::Else);
        self.store_i64_const_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_OBJECT_KEY as u64,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_state_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_OBJECT_KEY));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        self.emit_parse_json_string_at_index(
            string_offset_local,
            string_len_local,
            index_local,
            key_or_index_local,
            invalid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            frame_local,
            JSON_PARSE_FRAME_KEY_OR_INDEX_OFFSET,
            key_or_index_local,
            function,
        );
        self.store_i64_const_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_OBJECT_COLON as u64,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_state_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_OBJECT_COLON));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b':' as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        self.emit_increment_local(index_local, 1, function);
        self.store_i64_const_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_OBJECT_VALUE as u64,
            function,
        );
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_state_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_OBJECT_VALUE));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(value_start_local));
        self.emit_parse_json_value_at_index(
            string_offset_local,
            string_len_local,
            index_local,
            value_payload_local,
            value_tag_local,
            nested_state_local,
            invalid_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        self.emit_object_create_data_property_silent(
            frame_payload_local,
            key_or_index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(metadata_children_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(metadata_children_tag_local));
        function.instruction(&Instruction::LocalGet(capture_metadata_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_json_parse_metadata(
            string_offset_local,
            value_start_local,
            index_local,
            value_payload_local,
            value_tag_local,
            metadata_local,
            metadata_children_payload_local,
            metadata_children_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(metadata_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(metadata_value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalSet(metadata_value_tag_local));
        self.emit_object_create_data_property_silent(
            frame_metadata_payload_local,
            key_or_index_local,
            metadata_value_payload_local,
            metadata_value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_OBJECT_COMMA_OR_END as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(nested_state_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_or_index_local));
        self.emit_push_json_parse_frame(
            frame_buffer_local,
            frame_capacity_local,
            frame_len_local,
            value_payload_local,
            value_tag_local,
            nested_state_local,
            key_or_index_local,
            metadata_children_payload_local,
            metadata_children_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(frame_state_local));
        function.instruction(&Instruction::I64Const(JSON_PARSE_OBJECT_COMMA_OR_END));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'}' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(frame_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(frame_len_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b',' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        self.store_i64_const_at_offset(
            frame_local,
            JSON_PARSE_FRAME_STATE_OFFSET,
            JSON_PARSE_OBJECT_KEY as u64,
            function,
        );
        function.instruction(&Instruction::Else);
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_skip_json_whitespace(
            string_offset_local,
            string_len_local,
            index_local,
            byte_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_json_parse_syntax_error(function)?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(root_payload_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::LocalGet(root_tag_local));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::End);

        self.release_temp_local(metadata_value_tag_local);
        self.release_temp_local(metadata_value_payload_local);
        self.release_temp_local(metadata_children_tag_local);
        self.release_temp_local(metadata_children_payload_local);
        self.release_temp_local(metadata_local);
        self.release_temp_local(value_start_local);
        self.release_temp_local(final_array_element_local);
        self.release_temp_local(lookahead_index_local);
        self.release_temp_local(root_tag_local);
        self.release_temp_local(root_payload_local);
        self.release_temp_local(frame_metadata_tag_local);
        self.release_temp_local(frame_metadata_payload_local);
        self.release_temp_local(key_or_index_local);
        self.release_temp_local(frame_state_local);
        self.release_temp_local(frame_tag_local);
        self.release_temp_local(frame_payload_local);
        self.release_temp_local(frame_local);
        self.release_temp_local(frame_len_local);
        self.release_temp_local(frame_capacity_local);
        self.release_temp_local(frame_buffer_local);
        self.release_temp_local(nested_state_local);
        self.release_temp_local(invalid_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_try_parse_json_string_text(
        &mut self,
        string_payload_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        parsed_flag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let string_offset_local = self.reserve_temp_local();
        let string_len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let byte_local = self.reserve_temp_local();
        let dst_offset_local = self.reserve_temp_local();
        let dst_pos_local = self.reserve_temp_local();
        let decoded_len_local = self.reserve_temp_local();
        let invalid_local = self.reserve_temp_local();
        let codepoint_local = self.reserve_temp_local();
        let temp_local = self.reserve_temp_local();
        let first_hex_local = self.reserve_temp_local();
        let second_hex_local = self.reserve_temp_local();
        let third_hex_local = self.reserve_temp_local();
        let fourth_hex_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(parsed_flag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(invalid_local));
        self.emit_unpack_string_payload(
            string_payload_local,
            string_offset_local,
            string_len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(parsed_flag_local));
        self.emit_increment_local(index_local, 1, function);
        self.emit_heap_alloc_from_local(string_len_local, function)?;
        function.instruction(&Instruction::LocalSet(dst_offset_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::LocalSet(dst_pos_local));

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'"' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(0x20));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'\\' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(codepoint_local));
        for (escape, value) in [
            (b'"', b'"'),
            (b'\\', b'\\'),
            (b'/', b'/'),
            (b'b', 0x08),
            (b'f', 0x0c),
            (b'n', b'\n'),
            (b'r', b'\r'),
            (b't', b'\t'),
        ] {
            function.instruction(&Instruction::LocalGet(byte_local));
            function.instruction(&Instruction::I64Const(escape as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(value as i64));
            function.instruction(&Instruction::LocalSet(codepoint_local));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(byte_local));
        function.instruction(&Instruction::I64Const(b'u' as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(4));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Else);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            1,
            first_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(first_hex_local, first_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            2,
            second_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(second_hex_local, second_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            3,
            third_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(third_hex_local, third_hex_local, function);
        self.emit_load_string_byte_at_delta(
            string_offset_local,
            index_local,
            4,
            fourth_hex_local,
            function,
        );
        self.emit_hex_value_or_minus_one(fourth_hex_local, fourth_hex_local, function);
        self.emit_all_hex_valid_i32(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_pack_four_hex_to_code_unit(
            first_hex_local,
            second_hex_local,
            third_hex_local,
            fourth_hex_local,
            codepoint_local,
            function,
        );
        self.emit_store_utf8_codepoint(dst_pos_local, codepoint_local, temp_local, function);
        self.emit_increment_local(index_local, 4, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(codepoint_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64GeS);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_store_byte_local(dst_pos_local, codepoint_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Else);
        self.emit_store_byte_local(dst_pos_local, byte_local, function);
        self.emit_increment_local(dst_pos_local, 1, function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::BrIf(1));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(string_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_load_string_byte(string_offset_local, index_local, byte_local, function);
        self.emit_byte_is_json_whitespace_i32(byte_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_increment_local(index_local, 1, function);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(invalid_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(dst_pos_local));
        function.instruction(&Instruction::LocalGet(dst_offset_local));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(decoded_len_local));
        self.emit_pack_string_payload(dst_offset_local, decoded_len_local, function);
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(invalid_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_error(
            SYNTAX_ERROR_NAME,
            "Invalid JSON.parse text",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(fourth_hex_local);
        self.release_temp_local(third_hex_local);
        self.release_temp_local(second_hex_local);
        self.release_temp_local(first_hex_local);
        self.release_temp_local(temp_local);
        self.release_temp_local(codepoint_local);
        self.release_temp_local(invalid_local);
        self.release_temp_local(decoded_len_local);
        self.release_temp_local(dst_pos_local);
        self.release_temp_local(dst_offset_local);
        self.release_temp_local(byte_local);
        self.release_temp_local(index_local);
        self.release_temp_local(string_len_local);
        self.release_temp_local(string_offset_local);
        Ok(())
    }

    pub(crate) fn emit_try_parse_json_keyword_text(
        &mut self,
        string_payload_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        parsed_flag_local: u32,
        function: &mut Function,
    ) {
        let compare_payload_local = self.reserve_temp_local();

        for (text, payload, tag) in [
            ("null", 0, ValueKind::Null),
            ("false", 0, ValueKind::Boolean),
            ("true", 1, ValueKind::Boolean),
        ] {
            function.instruction(&Instruction::LocalGet(parsed_flag_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(text)));
            function.instruction(&Instruction::LocalSet(compare_payload_local));
            self.emit_string_payload_equality_i32(
                string_payload_local,
                compare_payload_local,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(payload));
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(tag.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(parsed_flag_local));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        }

        self.release_temp_local(compare_payload_local);
    }
}
