use super::*;

/// An allocated `Intl.Locale` result that is not yet branded or initialized.
///
/// The raw local is private and this state is deliberately non-`Copy`:
/// `Intl.Locale` must perform `OrdinaryCreateFromConstructor` before observing
/// its tag, but an abrupt tag/options completion must not publish that partial
/// object. Only `emit_initialize_intl_locale_object` can consume this state.
#[must_use]
pub(super) struct ReservedIntlLocaleObjectLocal(u32);

/// An `Intl.Locale` result whose complete represented record and brand exist.
///
/// Only this state can cross the constructor's result boundary. Keeping the
/// raw local private makes publishing a reserved object a Rust type error.
#[must_use]
pub(super) struct InitializedIntlLocaleObjectLocal(u32);

impl<'a> FunctionBuilder<'a> {
    /// ECMA-402 `Intl.Locale` step 6: resolve `NewTarget.prototype` and reserve
    /// the result object before the first observable tag or options operation.
    pub(super) fn emit_reserve_intl_locale_object(
        &mut self,
        function: &mut Function,
    ) -> Result<ReservedIntlLocaleObjectLocal, EmitError> {
        // The retained object local is reserved first. Prototype locals can
        // then be released in strict LIFO order while the lifecycle keeps the
        // object live across tag/options work.
        let object_payload_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let prototype = TaggedLocals::new(prototype_payload_local, prototype_tag_local);
        let result = (|| {
            self.emit_new_target_prototype_to_locals(
                INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX,
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
        Ok(ReservedIntlLocaleObjectLocal(object_payload_local))
    }

    /// Consume the unreachable reserved result and install every Locale slot
    /// represented by the current backend before making it publishable.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_initialize_intl_locale_object(
        &mut self,
        reserved: ReservedIntlLocaleObjectLocal,
        tag_payload_local: u32,
        language_payload_local: u32,
        script_payload_local: u32,
        region_payload_local: u32,
        base_name_payload_local: u32,
        function: &mut Function,
    ) -> Result<InitializedIntlLocaleObjectLocal, EmitError> {
        let object_payload_local = reserved.0;
        let record_local = self.reserve_temp_local();
        if let Err(error) = self.emit_heap_alloc_const(HEAP_INTL_LOCALE_RECORD_SIZE, function) {
            self.release_temp_local(record_local);
            self.release_temp_local(object_payload_local);
            return Err(error);
        }
        function.instruction(&Instruction::LocalSet(record_local));
        for (offset, value_local) in [
            (HEAP_INTL_LOCALE_TAG_OFFSET, tag_payload_local),
            (HEAP_INTL_LOCALE_LANGUAGE_OFFSET, language_payload_local),
            (HEAP_INTL_LOCALE_SCRIPT_OFFSET, script_payload_local),
            (HEAP_INTL_LOCALE_REGION_OFFSET, region_payload_local),
            (HEAP_INTL_LOCALE_BASE_NAME_OFFSET, base_name_payload_local),
        ] {
            self.store_i64_local_at_offset(record_local, offset, value_local, function);
        }
        self.store_i64_const_at_offset(
            object_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_INTL_LOCALE,
            function,
        );
        self.store_i64_local_at_offset(
            object_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record_local,
            function,
        );
        self.release_temp_local(record_local);
        Ok(InitializedIntlLocaleObjectLocal(object_payload_local))
    }

    /// Publish the only `Intl.Locale` lifecycle state allowed to escape.
    pub(super) fn emit_publish_intl_locale_object(
        &mut self,
        initialized: InitializedIntlLocaleObjectLocal,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(initialized.0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(initialized.0);
    }
}
