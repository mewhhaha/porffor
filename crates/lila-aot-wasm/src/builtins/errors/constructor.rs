use super::*;
use crate::functions::{NewTargetPrototypeFallback, OrdinaryDefaultPrototype};

/// The payload and representation tag selected by the Error constructor's
/// single `GetPrototypeFromConstructor` operation.
///
/// The fields and constructor remain private, and the witness is neither
/// `Copy` nor `Clone`. The Error instance allocator accepts only this state,
/// so an explicit Function, Array or Arguments prototype cannot be silently
/// reconstructed with the ordinary Object tag.
#[must_use = "the resolved Error prototype must be used for allocation and released"]
pub(super) struct ErrorConstructorPrototypeLocals {
    payload: u32,
    tag: u32,
}

impl<'a> FunctionBuilder<'a> {
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
        function: &mut Function,
    ) -> Result<ErrorConstructorPrototypeLocals, EmitError> {
        let payload = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        self.emit_new_target_prototype_to_locals(
            ERROR_PROTOTYPE_GLOBAL_INDEX,
            NewTargetPrototypeFallback::RequiredResolvedRealmOrdinaryActive(
                OrdinaryDefaultPrototype::Error,
            ),
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
