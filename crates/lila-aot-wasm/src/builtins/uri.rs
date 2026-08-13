use super::super::*;
use super::string::UriCodecKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UriBuiltin {
    Escape,
    Unescape,
    EncodeUri,
    EncodeUriComponent,
    DecodeUri,
    DecodeUriComponent,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_uri_builtin(
        &mut self,
        builtin: UriBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            UriBuiltin::Escape | UriBuiltin::Unescape => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let string_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalSet(string_local));
                if builtin == UriBuiltin::Escape {
                    self.emit_annexb_escape_string_payload(string_local, function)?;
                } else {
                    self.emit_annexb_unescape_string_payload(string_local, function)?;
                }
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(string_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            UriBuiltin::EncodeUri
            | UriBuiltin::EncodeUriComponent
            | UriBuiltin::DecodeUri
            | UriBuiltin::DecodeUriComponent => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let string_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
                self.emit_return_current_completion_if_throw(function);
                function.instruction(&Instruction::LocalSet(string_local));
                let codec_kind = match builtin {
                    UriBuiltin::EncodeUri | UriBuiltin::DecodeUri => UriCodecKind::Uri,
                    UriBuiltin::EncodeUriComponent | UriBuiltin::DecodeUriComponent => {
                        UriCodecKind::Component
                    }
                    UriBuiltin::Escape | UriBuiltin::Unescape => unreachable!(),
                };
                match builtin {
                    UriBuiltin::EncodeUri | UriBuiltin::EncodeUriComponent => {
                        self.emit_uri_encode_string_payload(string_local, codec_kind, function)?;
                    }
                    UriBuiltin::DecodeUri | UriBuiltin::DecodeUriComponent => {
                        self.emit_uri_decode_string_payload(string_local, codec_kind, function)?;
                    }
                    UriBuiltin::Escape | UriBuiltin::Unescape => unreachable!(),
                }
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));

                self.release_temp_local(string_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
        }
        Ok(())
    }
}
