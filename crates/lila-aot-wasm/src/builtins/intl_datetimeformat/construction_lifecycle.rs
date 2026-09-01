use super::*;

/// An allocated `Intl.DateTimeFormat` result that has not been branded or
/// connected to its internal record.
///
/// The raw local is private and this state is deliberately non-`Copy`:
/// `OrdinaryCreateFromConstructor` must happen before any locale or options
/// observation, but an abrupt initialization must not publish that object.
#[must_use]
pub(super) struct ReservedIntlDateTimeFormatObjectLocal(u32);

/// A reserved `Intl.DateTimeFormat` result whose complete represented record
/// and internal brand have been installed.
///
/// Only this state can cross the constructor result boundary.
#[must_use]
pub(super) struct InitializedIntlDateTimeFormatObjectLocal(u32);

impl<'a> FunctionBuilder<'a> {
    /// Resolve `NewTarget.prototype` and reserve the ordinary result before
    /// the first locale or options operation.
    pub(super) fn emit_reserve_intl_date_time_format_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedIntlDateTimeFormatObjectLocal, EmitError> {
        // Reserve the retained object first so both temporary prototype locals
        // can be released in strict LIFO order while it stays live.
        let object_payload_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let prototype = TaggedLocals::new(prototype_payload_local, prototype_tag_local);
        let result = (|| {
            self.emit_new_target_prototype_to_locals(
                INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
                NewTargetPrototypeFallback::CurrentGlobal,
                prototype.payload,
                prototype.tag,
                function,
            )?;
            self.emit_alloc_plain_object_with_prototype_and_tag(
                Some(prototype.payload),
                Some(prototype.tag),
                None,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(object_payload_local));
            Ok(())
        })();
        self.release_temp_local(prototype.tag);
        self.release_temp_local(prototype.payload);
        if let Err(error) = result {
            self.release_temp_local(object_payload_local);
            return Err(error);
        }
        Ok(ReservedIntlDateTimeFormatObjectLocal(object_payload_local))
    }

    /// Consume the unreachable reserved result after the DateTimeFormat record
    /// is complete, then make the object eligible for publication.
    pub(super) fn emit_initialize_intl_date_time_format_object(
        &self,
        reserved: ReservedIntlDateTimeFormatObjectLocal,
        record_local: u32,
        function: &mut Function,
    ) -> InitializedIntlDateTimeFormatObjectLocal {
        let object_payload_local = reserved.0;
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_INTL_DATE_TIME_FORMAT,
            function,
        );
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        InitializedIntlDateTimeFormatObjectLocal(object_payload_local)
    }

    /// Publish the only DateTimeFormat lifecycle state allowed to escape.
    pub(super) fn emit_publish_intl_date_time_format_object(
        &mut self,
        initialized: InitializedIntlDateTimeFormatObjectLocal,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(initialized.0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(initialized.0);
    }
}
