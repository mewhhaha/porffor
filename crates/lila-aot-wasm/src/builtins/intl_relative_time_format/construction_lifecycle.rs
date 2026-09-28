use super::*;

#[must_use]
pub(super) struct ReservedRelativeTimeFormatObjectLocal(u32);

#[must_use]
pub(super) struct InitializedRelativeTimeFormatObjectLocal(u32);

impl FunctionBuilder<'_> {
    pub(super) fn emit_reserve_relative_time_format_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedRelativeTimeFormatObjectLocal, EmitError> {
        let object = self.reserve_temp_local();
        let prototype = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        let result = (|| {
            self.compile_new_target_to_locals(prototype, tag, function)?;
            function.instruction(&Instruction::LocalGet(tag));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_rtf_type_error("Intl.RelativeTimeFormat must be called with new", function)?;
            function.instruction(&Instruction::End);
            self.emit_new_target_prototype_to_locals(
                INTL_RELATIVE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
                NewTargetPrototypeFallback::RequiredResolvedRealmOrdinary(
                    OrdinaryDefaultPrototype::IntlRelativeTimeFormat,
                ),
                prototype,
                tag,
                function,
            )?;
            self.emit_alloc_plain_object_with_prototype_and_tag(
                Some(prototype),
                Some(tag),
                None,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(object));
            Ok(())
        })();
        self.release_temp_local(tag);
        self.release_temp_local(prototype);
        if let Err(error) = result {
            self.release_temp_local(object);
            return Err(error);
        }
        Ok(ReservedRelativeTimeFormatObjectLocal(object))
    }

    pub(super) fn emit_initialize_relative_time_format_object(
        &self,
        reserved: ReservedRelativeTimeFormatObjectLocal,
        record: u32,
        function: &mut Function,
    ) -> InitializedRelativeTimeFormatObjectLocal {
        self.store_i64_const_at_offset(
            reserved.0,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_INTL_RELATIVE_TIME_FORMAT,
            function,
        );
        self.store_i64_local_at_offset(
            reserved.0,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record,
            function,
        );
        InitializedRelativeTimeFormatObjectLocal(reserved.0)
    }

    pub(super) fn emit_publish_relative_time_format_object(
        &mut self,
        initialized: InitializedRelativeTimeFormatObjectLocal,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(initialized.0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_rtf_set_const(
            self.result_tag_local,
            ValueKind::Object.tag() as i64,
            function,
        );
        self.release_temp_local(initialized.0);
    }
}
