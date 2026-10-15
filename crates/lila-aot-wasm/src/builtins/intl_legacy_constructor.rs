//! ECMA-402's normative optional Chain/Unwrap formatter constructor mode.

use super::super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::gc_types::*;

#[derive(Clone, Copy)]
pub(in crate::builtins) enum IntlLegacyFormatterKind {
    DateTimeFormat,
    NumberFormat,
}

impl IntlLegacyFormatterKind {
    fn constructor(self) -> NonArrayRealmIntrinsicSlot {
        match self {
            Self::DateTimeFormat => NonArrayRealmIntrinsicSlot::IntlDateTimeFormatConstructor,
            Self::NumberFormat => NonArrayRealmIntrinsicSlot::IntlNumberFormatConstructor,
        }
    }

    fn emit_initialized(self, receiver: &ValueLocals, schema: &RuntimeSchema, f: &mut Function) {
        receiver.reference().load(f);
        let heap_type = match self {
            Self::DateTimeFormat => {
                schema
                    .reference_type::<IntlDateTimeFormatObject>(GcNullability::NonNullable)
                    .heap_type
            }
            Self::NumberFormat => {
                schema
                    .reference_type::<IntlNumberFormatObject>(GcNullability::NonNullable)
                    .heap_type
            }
        };
        f.instruction(&Instruction::RefTestNonNull(heap_type));
    }
}

impl FunctionBuilder<'_> {
    /// The fresh formatter is already completely initialized. No test of the
    /// original `this`, prototype walk or fallback property runs before that.
    pub(in crate::builtins) fn emit_intl_chain_legacy_constructor(
        &mut self,
        kind: IntlLegacyFormatterKind,
        initialized: &ValueLocals,
        f: &mut Function,
    ) -> Result<ValueLocals, EmitError> {
        let schema = self.runtime_schema();
        let returned = schema.reserve_value_local(f);
        returned.copy_from(initialized, f);
        let receiver = schema.reserve_value_local(f);
        let new_target = schema.reserve_value_local(f);
        let entry = self.body_entry_locals().ok_or_else(|| {
            EmitError::unsupported("legacy formatter constructor requires its actual entry")
        })?;
        receiver.copy_from(entry.this_value(), f);
        new_target.copy_from(entry.new_target(), f);
        new_target.tag().load(f);
        f.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let constructor = schema.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(&realm, kind.constructor(), &constructor, f);
        let pending = schema.reserve_completion(f);
        self.emit_ordinary_has_instance_from_locals(&constructor, &receiver, &pending, f)?;
        self.emit_intl_number_adopt_completion(&pending, f);
        self.compile_truthy_tagged_i32(pending.value(), f)?;
        self.open_frame(ControlFrameKind::If, f);
        let symbol = self.emit_realm_intl_fallback_symbol(&realm, f)?;
        let key = crate::operations::PropertyKeyLocals::from_symbol(schema, &symbol, f);
        self.emit_object_define_data_with_configurable(
            &receiver,
            &key,
            initialized,
            false,
            false,
            false,
            &pending,
            f,
        )?;
        self.emit_intl_number_adopt_completion(&pending, f);
        returned.copy_from(&receiver, f);
        key.clear(f);
        symbol.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        pending.clear(f);
        constructor.clear(f);
        realm.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        new_target.clear(f);
        receiver.clear(f);
        Ok(returned)
    }

    /// Only the format getter and resolvedOptions call this operation. The
    /// caller performs RequireInternalSlot on the returned value afterwards.
    pub(in crate::builtins) fn emit_intl_unwrap_legacy_formatter(
        &mut self,
        kind: IntlLegacyFormatterKind,
        original: &ValueLocals,
        error: RuntimeErrorMessage,
        f: &mut Function,
    ) -> Result<ValueLocals, EmitError> {
        let schema = self.runtime_schema();
        self.emit_is_heap_object_like_tag_i32(original.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_intl_number_type_error(error, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let unwrapped = schema.reserve_value_local(f);
        unwrapped.copy_from(original, f);
        kind.emit_initialized(original, schema, f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        let realm = schema
            .reserve_gc_local(f)
            .initialize(self.emit_current_function_realm(f), f);
        let constructor = schema.reserve_value_local(f);
        self.emit_load_non_array_realm_intrinsic(&realm, kind.constructor(), &constructor, f);
        let pending = schema.reserve_completion(f);
        self.emit_ordinary_has_instance_from_locals(&constructor, original, &pending, f)?;
        self.emit_intl_number_adopt_completion(&pending, f);
        self.compile_truthy_tagged_i32(pending.value(), f)?;
        self.open_frame(ControlFrameKind::If, f);
        let symbol = self.emit_realm_intl_fallback_symbol(&realm, f)?;
        let key = crate::operations::PropertyKeyLocals::from_symbol(schema, &symbol, f);
        // Ordinary Get uses the original proxy as both target and receiver;
        // its get trap, inherited accessor `this` and exact throw are retained.
        self.emit_dynamic_property_read_with_key_locals(original, original, &key, &pending, f)?;
        self.emit_intl_number_adopt_completion(&pending, f);
        unwrapped.copy_from(pending.value(), f);
        key.clear(f);
        symbol.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        pending.clear(f);
        constructor.clear(f);
        realm.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(unwrapped)
    }
}
