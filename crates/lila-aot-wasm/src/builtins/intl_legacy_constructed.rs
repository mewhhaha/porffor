//! ECMA-402's normative-optional constructor mode (4.3 Note 1).
//!
//! `Intl.NumberFormat` and `Intl.DateTimeFormat` called as functions on an
//! object that inherits from their prototype chain the new formatter onto that
//! object under `%Intl%.[[FallbackSymbol]]` (ChainNumberFormat,
//! ChainDateTimeFormat). The `format` getters and `resolvedOptions` read it
//! back through a real, Proxy-observable `Get` (UnwrapNumberFormat,
//! UnwrapDateTimeFormat) before their ordinary RequireInternalSlot.
//!
//! Every Realm owns one fallback Symbol. The entry Realm keeps it in a global
//! and in its Realm intrinsic record; a created Realm keeps it only in its
//! record. Builtin bodies resolve it, and `%Intl.X.prototype%`, from the Realm
//! of the running builtin exactly as other active-Realm intrinsics are
//! resolved, so a foreign Realm's method never sees another Realm's Symbol.

use super::super::*;
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::objects::{DescriptorFlag, DescriptorObjectPrototype, TaggedLocals};
use lila_ir::property_descriptor::CompleteDescriptor;

/// The `[[Description]]` of every Realm's `%Intl%.[[FallbackSymbol]]`.
pub(crate) const INTL_FALLBACK_SYMBOL_DESCRIPTION: &str = "IntlLegacyConstructedSymbol";

/// The two services whose constructors participate in the legacy mode.
///
/// Each variant owns its internal-slot brand and its prototype intrinsic
/// together, so a chain or unwrap step cannot test one service's slot against
/// another service's prototype chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntlLegacyConstructedService {
    NumberFormat,
    DateTimeFormat,
}

impl IntlLegacyConstructedService {
    /// The brand stored for `[[InitializedNumberFormat]]` or
    /// `[[InitializedDateTimeFormat]]`.
    const fn brand(self) -> u64 {
        match self {
            Self::NumberFormat => OBJECT_INTERNAL_BRAND_INTL_NUMBER_FORMAT,
            Self::DateTimeFormat => OBJECT_INTERNAL_BRAND_INTL_DATE_TIME_FORMAT,
        }
    }

    const fn prototype(self) -> ActiveBuiltinRealmIntlValue {
        match self {
            Self::NumberFormat => ActiveBuiltinRealmIntlValue::NumberFormatPrototype,
            Self::DateTimeFormat => ActiveBuiltinRealmIntlValue::DateTimeFormatPrototype,
        }
    }
}

/// A per-Realm Intl value read from the Realm of the running builtin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveBuiltinRealmIntlValue {
    NumberFormatPrototype,
    DateTimeFormatPrototype,
    FallbackSymbol,
}

impl ActiveBuiltinRealmIntlValue {
    const fn entry_global(self) -> u32 {
        match self {
            Self::NumberFormatPrototype => INTL_NUMBER_FORMAT_PROTOTYPE_GLOBAL_INDEX,
            Self::DateTimeFormatPrototype => INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
            Self::FallbackSymbol => INTL_FALLBACK_SYMBOL_GLOBAL_INDEX,
        }
    }

    const fn realm_slot(self) -> NonArrayRealmIntrinsicSlot {
        match self {
            Self::NumberFormatPrototype => NonArrayRealmIntrinsicSlot::IntlNumberFormatPrototype,
            Self::DateTimeFormatPrototype => {
                NonArrayRealmIntrinsicSlot::IntlDateTimeFormatPrototype
            }
            Self::FallbackSymbol => NonArrayRealmIntrinsicSlot::IntlFallbackSymbol,
        }
    }
}

impl FunctionBuilder<'_> {
    /// Allocate one Realm's `%Intl%.[[FallbackSymbol]]`: a new Symbol whose
    /// `[[Description]]` is "IntlLegacyConstructedSymbol".
    pub(crate) fn emit_alloc_intl_fallback_symbol(
        &mut self,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_heap_alloc_const(HEAP_SYMBOL_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(result_local));
        self.store_i64_const_at_offset(
            result_local,
            HEAP_SYMBOL_DESCRIPTION_TAG_OFFSET,
            ValueKind::String.tag() as u64,
            function,
        );
        self.store_i64_const_at_offset(
            result_local,
            HEAP_SYMBOL_DESCRIPTION_PAYLOAD_OFFSET,
            self.strings.payload(INTL_FALLBACK_SYMBOL_DESCRIPTION) as u64,
            function,
        );
        Ok(())
    }

    /// Load a per-Realm Intl value of the running builtin's Realm.
    ///
    /// `current_env_local` is zero only in the entry Realm; otherwise it is a
    /// function object whose defining Realm is the running Realm. Every link
    /// is complete before user code can reach an Intl formatter, so a zero is
    /// a compiler bug and traps rather than borrowing another Realm's value.
    fn emit_load_active_builtin_realm_intl_value(
        &mut self,
        value: ActiveBuiltinRealmIntlValue,
        result_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(value.entry_global()));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(result_local));
        for offset in [
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            HEAP_REALM_INTRINSICS_OFFSET,
            value.realm_slot().offset(),
        ] {
            self.load_i64_to_local_from_offset(result_local, offset, result_local, function);
            function.instruction(&Instruction::LocalGet(result_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    /// `? OrdinaryHasInstance(%Intl.X%, value)` for the running Realm's
    /// constructor, leaving 0 or 1 in `result_local`.
    ///
    /// `%Intl.X%` is callable and unbound, and its own `prototype` property is
    /// a non-writable, non-configurable data property whose value is the
    /// Realm's `%Intl.X.prototype%`, so step 4's `Get(C, "prototype")` has no
    /// observable effect and always yields that intrinsic. Step 6's walk is
    /// performed through `[[GetPrototypeOf]]`, invoking Proxy traps.
    fn emit_intl_legacy_ordinary_has_instance(
        &mut self,
        service: IntlLegacyConstructedService,
        value: TaggedLocals,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let search_payload_local = self.reserve_temp_local();
        let search_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(result_local));
        // Step 3: a primitive has no prototype chain.
        self.emit_is_heap_object_like_tag_i32(value.tag, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_active_builtin_realm_intl_value(
            service.prototype(),
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(value.payload));
        function.instruction(&Instruction::LocalSet(search_payload_local));
        function.instruction(&Instruction::LocalGet(value.tag));
        function.instruction(&Instruction::LocalSet(search_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(search_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_object_get_prototype_of(
                search_payload_local,
                search_tag_local,
                next_payload_local,
                next_tag_local,
                function,
            )?;
        } else {
            self.emit_ordinary_get_prototype_of(
                search_payload_local,
                search_tag_local,
                next_payload_local,
                next_tag_local,
                function,
            );
        }
        function.instruction(&Instruction::LocalGet(next_payload_local));
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_payload_local));
        function.instruction(&Instruction::LocalSet(search_payload_local));
        function.instruction(&Instruction::LocalGet(next_tag_local));
        function.instruction(&Instruction::LocalSet(search_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            next_tag_local,
            next_payload_local,
            search_tag_local,
            search_payload_local,
            prototype_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// ChainNumberFormat / ChainDateTimeFormat, applied to the formatter the
    /// constructor has just published in the result locals.
    ///
    /// When NewTarget is undefined and the `this` value inherits from the
    /// running Realm's `%Intl.X.prototype%`, the formatter is defined on it
    /// under `%Intl%.[[FallbackSymbol]]` as `{ [[Writable]]: false,
    /// [[Enumerable]]: false, [[Configurable]]: false }` and `this` is
    /// returned. DefinePropertyOrThrow goes through the canonical
    /// `Object.defineProperty` body with a null-prototype private descriptor,
    /// so exotic and Proxy receivers observe their ordinary
    /// `[[DefineOwnProperty]]` and a refused definition throws a TypeError of
    /// the running Realm.
    pub(crate) fn emit_intl_chain_legacy_constructed(
        &mut self,
        service: IntlLegacyConstructedService,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported("Intl legacy constructor chaining requires a this value")
        })?;
        let this_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported("Intl legacy constructor chaining requires a this tag")
        })?;
        let new_target_tag_local = self.new_target_tag_local().ok_or_else(|| {
            EmitError::unsupported("Intl legacy constructor chaining requires NewTarget")
        })?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "Intl legacy constructor chaining requires `Object.defineProperty`",
                )
            })?;
        let formatter = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let has_instance_local = self.reserve_temp_local();
        let symbol = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let descriptor = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());
        let call = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());

        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(formatter.payload));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(formatter.tag));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(has_instance_local));

        // Step 1: NewTarget is undefined.
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Step 1: ? OrdinaryHasInstance(%Intl.X%, this).
        self.emit_intl_legacy_ordinary_has_instance(
            service,
            TaggedLocals::new(this_payload_local, this_tag_local),
            has_instance_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_instance_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Step 1.a: ? DefinePropertyOrThrow(this, %Intl%.[[FallbackSymbol]],
        // { [[Value]]: formatter, [[Writable]]: false, [[Enumerable]]: false,
        // [[Configurable]]: false }).
        self.emit_load_active_builtin_realm_intl_value(
            ActiveBuiltinRealmIntlValue::FallbackSymbol,
            symbol.payload,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(symbol.tag));
        self.emit_from_complete_property_descriptor(
            DescriptorObjectPrototype::PrivateCarrier,
            CompleteDescriptor::Data {
                value: formatter,
                writable: DescriptorFlag::Known(false),
                enumerable: DescriptorFlag::Known(false),
                configurable: DescriptorFlag::Known(false),
            },
            descriptor.payload,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor.tag));
        self.emit_direct_js_call(
            &define_property_meta,
            None,
            &[
                (this_payload_local, this_tag_local),
                (symbol.payload, symbol.tag),
                (descriptor.payload, descriptor.tag),
            ],
            call.payload,
            call.tag,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        // The prototype walk and the definition call reuse the builtin's
        // completion locals, so the result is always written from the saved
        // pair: step 1.b returns `this`, step 2 returns the formatter.
        function.instruction(&Instruction::LocalGet(has_instance_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(formatter.payload));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(formatter.tag));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);

        for local in [
            call.tag,
            call.payload,
            descriptor.tag,
            descriptor.payload,
            symbol.tag,
            symbol.payload,
            has_instance_local,
            formatter.tag,
            formatter.payload,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// UnwrapNumberFormat / UnwrapDateTimeFormat, replacing `value` in place.
    ///
    /// A non-Object throws the method's receiver TypeError (step 1). An Object
    /// that lacks the service's internal slot but inherits from the running
    /// Realm's `%Intl.X.prototype%` is replaced by `? Get(value,
    /// %Intl%.[[FallbackSymbol]])` (step 2); the caller's RequireInternalSlot
    /// then validates whatever that read produced.
    pub(crate) fn emit_intl_unwrap_legacy_constructed(
        &mut self,
        service: IntlLegacyConstructedService,
        value: TaggedLocals,
        receiver_error: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let brand_local = self.reserve_temp_local();
        let has_instance_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let read = TaggedLocals::new(self.reserve_temp_local(), self.reserve_temp_local());

        // Step 1: If nf is not an Object, throw a TypeError exception.
        self.emit_is_heap_object_like_tag_i32(value.tag, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            receiver_error,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // Step 2: the internal slot exists only on the service's branded
        // ordinary objects.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(brand_local));
        function.instruction(&Instruction::LocalGet(value.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            value.payload,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(service.brand() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_intl_legacy_ordinary_has_instance(service, value, has_instance_local, function)?;
        function.instruction(&Instruction::LocalGet(has_instance_local));
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::If(BlockType::Empty));
        // Step 2.a: Return ? Get(nf, %Intl%.[[FallbackSymbol]]).
        self.emit_load_active_builtin_realm_intl_value(
            ActiveBuiltinRealmIntlValue::FallbackSymbol,
            key_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            value.payload,
            value.tag,
            value.payload,
            value.tag,
            key_local,
            read.payload,
            read.tag,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(read.payload, read.tag, function)?;
        function.instruction(&Instruction::LocalGet(read.payload));
        function.instruction(&Instruction::LocalSet(value.payload));
        function.instruction(&Instruction::LocalGet(read.tag));
        function.instruction(&Instruction::LocalSet(value.tag));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            read.tag,
            read.payload,
            key_local,
            has_instance_local,
            brand_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }
}
