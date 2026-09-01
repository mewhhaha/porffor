use super::super::*;
use super::string::UriCodecKind;

enum UriBuiltin {
    Escape,
    Unescape,
    Encode(UriCodecKind),
    Decode(UriCodecKind),
}

#[allow(non_upper_case_globals)]
impl UriBuiltin {
    const EncodeUri: Self = Self::Encode(UriCodecKind::Uri);
    const EncodeUriComponent: Self = Self::Encode(UriCodecKind::Component);
    const DecodeUri: Self = Self::Decode(UriCodecKind::Uri);
    const DecodeUriComponent: Self = Self::Decode(UriCodecKind::Component);
}

impl<'a> FunctionBuilder<'a> {
    fn emit_uri_builtin(
        &mut self,
        builtin: UriBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let string_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalSet(string_local));
        match builtin {
            UriBuiltin::Escape => {
                self.emit_annexb_escape_string_payload(string_local, function)?;
            }
            UriBuiltin::Unescape => {
                self.emit_annexb_unescape_string_payload(string_local, function)?;
            }
            UriBuiltin::Encode(codec_kind) => {
                self.emit_uri_encode_string_payload(string_local, codec_kind, function)?;
            }
            UriBuiltin::Decode(codec_kind) => {
                self.emit_uri_decode_string_payload(string_local, codec_kind, function)?;
            }
        }
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(string_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn emit_escape_builtin(&mut self, function: &mut Function) -> Result<(), EmitError> {
        self.emit_uri_builtin(UriBuiltin::Escape, function)
    }

    pub(super) fn emit_unescape_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_uri_builtin(UriBuiltin::Unescape, function)
    }

    pub(super) fn emit_encode_uri_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_uri_builtin(UriBuiltin::EncodeUri, function)
    }

    pub(super) fn emit_encode_uri_component_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_uri_builtin(UriBuiltin::EncodeUriComponent, function)
    }

    pub(super) fn emit_decode_uri_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_uri_builtin(UriBuiltin::DecodeUri, function)
    }

    pub(super) fn emit_decode_uri_component_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_uri_builtin(UriBuiltin::DecodeUriComponent, function)
    }
}
