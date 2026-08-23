use super::*;
use crate::functions::{ErrorMessageConstructorKind, NewTargetPrototypeFallback};

/// The payload and representation tag selected by one shared-message Error
/// constructor's single `GetPrototypeFromConstructor` operation.
///
/// The fields and constructor remain private, and the witness is neither
/// `Copy` nor `Clone`. The instance allocator accepts only this state, so an
/// explicit Function, Array or Arguments prototype cannot be silently
/// reconstructed with the ordinary Object tag.
#[must_use = "the resolved Error-family prototype must be used for allocation and released"]
pub(super) struct ErrorConstructorPrototypeLocals {
    payload: u32,
    tag: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_error_message_constructor(
        &mut self,
        kind: ErrorMessageConstructorKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let message_payload_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        let prototype = self.emit_error_constructor_prototype(kind, function)?;
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_error_instance_from_locals(
            &prototype,
            None,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_install_error_cause_from_arg(
            self.result_local,
            ErrorCauseOptionsArgument::MessageError,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
        function.instruction(&Instruction::LocalSet(message_payload_local));
        self.emit_alloc_error_instance_from_locals(
            &prototype,
            Some(message_payload_local),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_install_error_cause_from_arg(
            self.result_local,
            ErrorCauseOptionsArgument::MessageError,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.release_error_constructor_prototype(prototype);
        self.release_temp_local(message_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn emit_alloc_error_instance_from_locals(
        &mut self,
        prototype: &ErrorConstructorPrototypeLocals,
        message_payload_local: Option<u32>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(prototype.payload),
            Some(prototype.tag),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(super) fn emit_error_constructor_prototype(
        &mut self,
        kind: ErrorMessageConstructorKind,
        function: &mut Function,
    ) -> Result<ErrorConstructorPrototypeLocals, EmitError> {
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        self.emit_new_target_prototype_to_locals(
            kind.prototype_global_index(),
            NewTargetPrototypeFallback::RequiredResolvedRealmMessageErrorActive(kind),
            payload,
            tag,
            function,
        )?;
        Ok(ErrorConstructorPrototypeLocals { payload, tag })
    }

    pub(super) fn release_error_constructor_prototype(
        &mut self,
        prototype: ErrorConstructorPrototypeLocals,
    ) {
        self.release_temp_local(prototype.tag);
        self.release_temp_local(prototype.payload);
    }
}
