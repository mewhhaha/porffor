use super::super::*;
use super::atomics::ATOMICS_PUBLICATION_ORDER;
use crate::functions::{
    ErrorMessageConstructorKind, FunctionPrototypeMaterialization, NonArrayRealmIntrinsicSlot,
};
use lila_ir::StandardBuiltinInstaller;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn init_builtin_constructor_object(
        &mut self,
        builtin: StandardBuiltinId,
        prototype_global_index: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                builtin.debug_name()
            ))
        })?;
        let constructor_global_index =
            standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin constructor global `{}`",
                    builtin.debug_name()
                ))
            })?;
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let prototype_object_local = self.reserve_temp_local();

        // Bundled once so a family installer in `intrinsics/` receives the same
        // values this function computed, without each extraction growing a
        // nine-parameter signature. See `intrinsics::IntrinsicInstall`.
        let intrinsic_context = IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        };

        self.emit_function_value_payload(meta, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(constructor_global_index));

        if builtin.constructable() && !matches!(builtin, StandardBuiltinId::BigIntConstructor) {
            if matches!(
                builtin,
                StandardBuiltinId::EvalErrorConstructor
                    | StandardBuiltinId::AggregateErrorConstructor
                    | StandardBuiltinId::SuppressedErrorConstructor
                    | StandardBuiltinId::RangeErrorConstructor
                    | StandardBuiltinId::SyntaxErrorConstructor
                    | StandardBuiltinId::TypeErrorConstructor
                    | StandardBuiltinId::URIErrorConstructor
                    | StandardBuiltinId::ReferenceErrorConstructor
            ) {
                function.instruction(&Instruction::GlobalGet(ERROR_CONSTRUCTOR_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_PROTOTYPE_OFFSET,
                    self.scratch_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Function.tag() as u64,
                    function,
                );
            }
            if is_typed_array_constructor(builtin) {
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                    object_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET,
                    typed_array_bytes_per_element(builtin),
                    function,
                );
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_TYPED_ARRAY_ELEMENT_KIND_OFFSET,
                    typed_array_element_kind(builtin),
                    function,
                );
                function.instruction(&Instruction::GlobalGet(
                    TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_PROTOTYPE_OFFSET,
                    self.scratch_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Function.tag() as u64,
                    function,
                );
                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                    ValueKind::Object.tag() as u64,
                    function,
                );
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                    prototype_object_local,
                    function,
                );
                self.emit_object_define_number_data_from_f64_const_with_flags(
                    object_local,
                    "BYTES_PER_ELEMENT",
                    typed_array_bytes_per_element(builtin) as f64,
                    false,
                    false,
                    false,
                    function,
                )?;
                self.emit_object_define_number_data_from_f64_const_with_flags(
                    prototype_object_local,
                    "BYTES_PER_ELEMENT",
                    typed_array_bytes_per_element(builtin) as f64,
                    false,
                    false,
                    false,
                    function,
                )?;
            } else {
                let prototype_kind = match builtin {
                    StandardBuiltinId::ArrayConstructor => ValueKind::Array,
                    StandardBuiltinId::FunctionConstructor => ValueKind::Function,
                    _ => ValueKind::Object,
                };
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                    prototype_kind.tag() as u64,
                    function,
                );
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.store_i64_local_at_offset(
                    object_local,
                    HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                    self.scratch_local,
                    function,
                );
                function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(prototype_kind.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_define_data_with_configurable(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    false,
                    false,
                    false,
                    function,
                )?;

                if builtin != StandardBuiltinId::IteratorConstructor {
                    function
                        .instruction(&Instruction::I64Const(self.strings.payload("constructor")));
                    function.instruction(&Instruction::LocalSet(key_local));
                    function.instruction(&Instruction::LocalGet(object_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    function.instruction(&Instruction::GlobalGet(prototype_global_index));
                    function.instruction(&Instruction::LocalSet(prototype_object_local));
                    self.emit_object_append_data_property_with_flags(
                        prototype_object_local,
                        key_local,
                        payload_local,
                        tag_local,
                        true,
                        false,
                        true,
                        function,
                    )?;
                }
            }
        }

        match builtin.intrinsic_installer() {
            StandardBuiltinInstaller::None => {}
            StandardBuiltinInstaller::Function => {
                self.install_function_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Promise => {
                self.install_promise_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Map => {
                self.install_map_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::WeakMap => {
                self.install_weak_map_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::WeakSet => {
                self.install_weak_set_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::WeakRef => {
                self.install_weak_ref_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::FinalizationRegistry => self
                .install_finalization_registry_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinInstaller::AsyncDisposableStack => self
                .install_async_disposable_stack_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinInstaller::DisposableStack => {
                self.install_disposable_stack_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Set => {
                self.install_set_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Object => {
                self.install_object_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Proxy => {
                self.install_proxy_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::RegExp => {
                self.install_regexp_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Iterator => {
                self.install_iterator_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Array => {
                self.install_array_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::String => {
                self.install_string_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::ArrayBuffer => {
                self.install_array_buffer_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::DataView => {
                self.install_data_view_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::TemporalInstant => {
                self.install_temporal_instant_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::TemporalZonedDateTime => self
                .install_temporal_zoned_date_time_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinInstaller::TemporalPlainDate => self
                .install_temporal_plain_date_constructor_intrinsics(&intrinsic_context, function)?,
            StandardBuiltinInstaller::TemporalDuration => {
                self.install_temporal_duration_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::TemporalPlainTime => self
                .install_temporal_plain_time_constructor_intrinsics(&intrinsic_context, function)?,
            StandardBuiltinInstaller::TemporalPlainDateTime => self
                .install_temporal_plain_date_time_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinInstaller::TemporalPlainYearMonth => self
                .install_temporal_plain_year_month_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinInstaller::TemporalPlainMonthDay => self
                .install_temporal_plain_month_day_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinInstaller::IntlLocale => {
                self.install_intl_locale_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::IntlDateTimeFormat => self
                .install_intl_date_time_format_constructor_intrinsics(
                    &intrinsic_context,
                    function,
                )?,
            StandardBuiltinInstaller::Date => {
                self.install_date_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Error => {
                self.install_error_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::BigInt => {
                self.install_big_int_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Symbol => {
                self.install_symbol_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Number => {
                self.install_number_constructor_intrinsics(&intrinsic_context, function)?
            }
            StandardBuiltinInstaller::Boolean => {
                self.install_boolean_constructor_intrinsics(&intrinsic_context, function)?
            }
        }

        self.release_temp_local(prototype_object_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_throw_type_error_intrinsic(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thrower_meta = self
            .functions
            .get(&StandardBuiltinId::ThrowTypeError.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `%ThrowTypeError%`",
                )
            })?;
        let function_prototype_local = self.reserve_temp_local();
        let thrower_payload_local = self.reserve_temp_local();
        let thrower_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        self.emit_function_value_payload(thrower_meta, function)?;
        function.instruction(&Instruction::LocalSet(thrower_payload_local));
        function.instruction(&Instruction::LocalGet(thrower_payload_local));
        function.instruction(&Instruction::GlobalSet(THROW_TYPE_ERROR_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            THROW_TYPE_ERROR_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::ThrowTypeError,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(thrower_tag_local));

        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(function_prototype_local));
        for name in ["arguments", "caller"] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_object_append_accessor_property_with_flags(
                function_prototype_local,
                key_local,
                Some((thrower_payload_local, thrower_tag_local)),
                Some((thrower_payload_local, thrower_tag_local)),
                false,
                true,
                function,
            )?;
        }

        self.release_temp_local(key_local);
        self.release_temp_local(thrower_tag_local);
        self.release_temp_local(thrower_payload_local);
        self.release_temp_local(function_prototype_local);
        Ok(())
    }

    pub(crate) fn init_reflect_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let construct_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectConstruct.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.construct`",
                )
            })?;
        let apply_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectApply.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.apply`",
                )
            })?;
        let get_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGet.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.get`",
                )
            })?;
        let get_prototype_of_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetPrototypeOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.getPrototypeOf`",
                )
            })?;
        let get_own_property_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.getOwnPropertyDescriptor`",
                )
            })?;
        let set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;
        let has_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectHas.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.has`",
                )
            })?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                )
            })?;
        let delete_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDeleteProperty.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.deleteProperty`",
                )
            })?;
        let is_extensible_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectIsExtensible.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.isExtensible`",
                )
            })?;
        let prevent_extensions_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectPreventExtensions.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.preventExtensions`",
                )
            })?;
        let set_prototype_of_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSetPrototypeOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.setPrototypeOf`",
                )
            })?;
        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Reflect.ownKeys`",
                )
            })?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_function_data(object_local, "construct", construct_meta, function)?;
        self.emit_object_define_function_data(object_local, "apply", apply_meta, function)?;
        self.emit_object_define_function_data(object_local, "get", get_meta, function)?;
        self.emit_object_define_function_data(
            object_local,
            "getPrototypeOf",
            get_prototype_of_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "getOwnPropertyDescriptor",
            get_own_property_descriptor_meta,
            function,
        )?;
        self.emit_object_define_function_data(object_local, "set", set_meta, function)?;
        self.emit_object_define_function_data(object_local, "has", has_meta, function)?;
        self.emit_object_define_function_data(
            object_local,
            "defineProperty",
            define_property_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "deleteProperty",
            delete_property_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "isExtensible",
            is_extensible_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "preventExtensions",
            prevent_extensions_meta,
            function,
        )?;
        self.emit_object_define_function_data(
            object_local,
            "setPrototypeOf",
            set_prototype_of_meta,
            function,
        )?;
        self.emit_object_define_function_data(object_local, "ownKeys", own_keys_meta, function)?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Reflect")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(REFLECT_OBJECT_GLOBAL_INDEX));
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    /// Temporal proposal 2.2: the `Temporal.Now` namespace object. It is an
    /// ordinary object, not a constructor, so it gets no prototype global or
    /// branded record. The namespace witness makes every function advertised
    /// by the IR shape available before this installer can be called.
    fn init_temporal_now_object(
        &mut self,
        members: TemporalNamespaceMembers,
        object_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Temporal.Now")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);

        for (name, builtin) in members.now_members_in_installation_order() {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        Ok(())
    }

    pub(crate) fn init_temporal_object(
        &mut self,
        members: TemporalNamespaceMembers,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let constructor_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        let now_local = self.reserve_temp_local();
        let now_tag_local = self.reserve_temp_local();
        self.init_temporal_now_object(members, now_local, function)?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(now_tag_local));
        self.emit_object_append_local_data_property_with_flags(
            object_local,
            TEMPORAL_NOW_NAME,
            now_local,
            now_tag_local,
            true,
            false,
            true,
            function,
        )?;
        self.release_temp_local(now_tag_local);
        self.release_temp_local(now_local);

        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        for (name, builtin) in members.constructors_in_installation_order() {
            let global_index =
                standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: \
                         missing Temporal constructor global `{}`",
                        builtin.debug_name()
                    ))
                })?;
            function.instruction(&Instruction::GlobalGet(global_index));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                name,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Temporal")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(TEMPORAL_OBJECT_GLOBAL_INDEX));

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    /// ECMA-402 8: the `Intl` namespace object. Only the properties this
    /// backend actually implements are installed — nothing is stubbed.
    ///
    /// `members` is a proof, obtainable only from
    /// `RuntimeBootstrapPlan::intl_namespace_members`, that every member is
    /// rooted. Holding it is what lets this function install the whole list
    /// unconditionally; it is also the reason the function cannot be called for
    /// a program that does not get an `Intl` object at all.
    pub(crate) fn init_intl_object(
        &mut self,
        members: IntlNamespaceMembers,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let constructor_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        let get_canonical_locales_meta = self
            .functions
            .get(&StandardBuiltinId::IntlGetCanonicalLocales.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Intl.getCanonicalLocales`",
                )
            })?;
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_define_function_data(
            object_local,
            "getCanonicalLocales",
            get_canonical_locales_meta,
            function,
        )?;
        // One list, `INTL_NAMESPACE_CONSTRUCTORS`, decides both what the IR
        // shape claims `Intl` has (`ScriptLowerer::intl_object_value_info`) and
        // what actually gets installed here. They used to be two
        // hand-maintained lists and they drifted: `DateTimeFormat` was declared
        // and never installed, so constant-folded member access hid the gap —
        // `new Intl.DateTimeFormat()` worked while
        // `Object.getOwnPropertyDescriptor(Intl, "DateTimeFormat")`,
        // `Object.keys(Intl)`, `Intl["DateTimeFormat"]` and destructuring all
        // saw nothing. That is `intl402/DateTimeFormat/prop-desc.js`'s
        // "Expected descriptor to exist".
        //
        // Unifying the lists closed the drift but left a second divergence
        // point right here: a per-member `should_initialize_standard_builtin`
        // check with a `continue`, which reintroduced exactly the same wrong
        // object whenever the plan under-rooted the namespace. That check is
        // gone. `members` is the proof that it would have been vacuous, and it
        // is the only way to reach the list at all, so a partially installed
        // `Intl` is now unrepresentable rather than untested.
        //
        // Installation order is `Object.getOwnPropertyNames(Intl)` order, so it
        // is the slice's order and must not be sorted here.
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        for (name, builtin) in members.in_installation_order() {
            let global_index =
                standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: \
                         missing Intl constructor global `{}`",
                        builtin.debug_name()
                    ))
                })?;
            function.instruction(&Instruction::GlobalGet(global_index));
            function.instruction(&Instruction::LocalSet(constructor_local));
            self.emit_object_append_local_data_property_with_flags(
                object_local,
                name,
                constructor_local,
                constructor_tag_local,
                true,
                false,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Intl")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(INTL_OBJECT_GLOBAL_INDEX));

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_math_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        for (name, value) in [
            ("E", std::f64::consts::E),
            ("LN10", std::f64::consts::LN_10),
            ("LN2", std::f64::consts::LN_2),
            ("LOG10E", std::f64::consts::LOG10_E),
            ("LOG2E", std::f64::consts::LOG2_E),
            ("PI", std::f64::consts::PI),
            ("SQRT1_2", std::f64::consts::FRAC_1_SQRT_2),
            ("SQRT2", std::f64::consts::SQRT_2),
        ] {
            self.emit_object_define_number_data_from_f64_const_with_flags(
                object_local,
                name,
                value,
                false,
                false,
                false,
                function,
            )?;
        }
        for (name, builtin) in [
            ("abs", StandardBuiltinId::MathAbs),
            ("acos", StandardBuiltinId::MathAcos),
            ("acosh", StandardBuiltinId::MathAcosh),
            ("asin", StandardBuiltinId::MathAsin),
            ("asinh", StandardBuiltinId::MathAsinh),
            ("atan", StandardBuiltinId::MathAtan),
            ("atan2", StandardBuiltinId::MathAtan2),
            ("atanh", StandardBuiltinId::MathAtanh),
            ("cbrt", StandardBuiltinId::MathCbrt),
            ("ceil", StandardBuiltinId::MathCeil),
            ("clz32", StandardBuiltinId::MathClz32),
            ("cos", StandardBuiltinId::MathCos),
            ("cosh", StandardBuiltinId::MathCosh),
            ("exp", StandardBuiltinId::MathExp),
            ("expm1", StandardBuiltinId::MathExpm1),
            ("f16round", StandardBuiltinId::MathF16Round),
            ("floor", StandardBuiltinId::MathFloor),
            ("fround", StandardBuiltinId::MathFround),
            ("hypot", StandardBuiltinId::MathHypot),
            ("imul", StandardBuiltinId::MathImul),
            ("log", StandardBuiltinId::MathLog),
            ("log10", StandardBuiltinId::MathLog10),
            ("log1p", StandardBuiltinId::MathLog1p),
            ("log2", StandardBuiltinId::MathLog2),
            ("pow", StandardBuiltinId::MathPow),
            ("random", StandardBuiltinId::MathRandom),
            ("round", StandardBuiltinId::MathRound),
            ("sign", StandardBuiltinId::MathSign),
            ("sin", StandardBuiltinId::MathSin),
            ("sinh", StandardBuiltinId::MathSinh),
            ("sqrt", StandardBuiltinId::MathSqrt),
            ("sumPrecise", StandardBuiltinId::MathSumPrecise),
            ("tan", StandardBuiltinId::MathTan),
            ("tanh", StandardBuiltinId::MathTanh),
            ("trunc", StandardBuiltinId::MathTrunc),
            ("min", StandardBuiltinId::MathMin),
            ("max", StandardBuiltinId::MathMax),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Math")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(MATH_OBJECT_GLOBAL_INDEX));
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_json_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(JSON_NAME)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        for (name, builtin) in [
            ("parse", StandardBuiltinId::JsonParse),
            ("stringify", StandardBuiltinId::JsonStringify),
            ("rawJSON", StandardBuiltinId::JsonRawJson),
            ("isRawJSON", StandardBuiltinId::JsonIsRawJson),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(JSON_OBJECT_GLOBAL_INDEX));
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_atomics_object(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(ATOMICS_NAME)));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            object_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);

        for builtin in ATOMICS_PUBLICATION_ORDER {
            let name = builtin.native_function_name().ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing native name for `{}`",
                    builtin.debug_name()
                ))
            })?;
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            let method_payload_local = self.reserve_temp_local();
            let method_tag_local = self.reserve_temp_local();
            self.emit_function_value_payload(&meta, function)?;
            function.instruction(&Instruction::LocalSet(method_payload_local));
            self.store_i64_local_at_offset(
                method_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                method_payload_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(method_tag_local));
            self.emit_object_define_local_data(
                object_local,
                name,
                method_payload_local,
                method_tag_local,
                function,
            )?;
            self.release_temp_local(method_tag_local);
            self.release_temp_local(method_payload_local);
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(ATOMICS_OBJECT_GLOBAL_INDEX));
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn init_typed_array_intrinsic(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let typed_array_constructor_local = self.reserve_temp_local();
        let typed_array_prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        let typed_array_constructor_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayConstructor.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `%TypedArray%`",
                )
            })?;
        self.emit_function_value_payload_with_prototype_materialization(
            typed_array_constructor_meta,
            FunctionPrototypeMaterialization::BootstrapSupplied,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(typed_array_constructor_local));
        function.instruction(&Instruction::LocalGet(typed_array_constructor_local));
        function.instruction(&Instruction::GlobalSet(
            TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::GlobalGet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(typed_array_prototype_local));
        self.store_i64_local_at_offset(
            typed_array_constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            typed_array_constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            typed_array_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_append_data_property_with_flags(
            typed_array_constructor_local,
            key_local,
            typed_array_prototype_local,
            tag_local,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(typed_array_constructor_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            typed_array_prototype_local,
            key_local,
            payload_local,
            tag_local,
            true,
            false,
            true,
            function,
        )?;

        let species_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArraySpeciesGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray[Symbol.species]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.species"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(species_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_accessor_property_with_flags(
            typed_array_constructor_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            false,
            true,
            function,
        )?;

        for (name, builtin) in [
            ("buffer", StandardBuiltinId::TypedArrayPrototypeBufferGetter),
            (
                "byteLength",
                StandardBuiltinId::TypedArrayPrototypeByteLengthGetter,
            ),
            (
                "byteOffset",
                StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter,
            ),
            ("length", StandardBuiltinId::TypedArrayPrototypeLengthGetter),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            function.instruction(&Instruction::I64Const(self.strings.payload(&name)));
            function.instruction(&Instruction::LocalSet(key_local));
            self.emit_function_value_payload(meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            function.instruction(&Instruction::GlobalGet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(typed_array_prototype_local));
            self.emit_object_append_accessor_property_with_flags(
                typed_array_prototype_local,
                key_local,
                Some((payload_local, tag_local)),
                None,
                false,
                true,
                function,
            )?;
        }

        let to_string_tag_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeToStringTagGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `get TypedArray.prototype[Symbol.toStringTag]`",
                )
            })?;
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_function_value_payload(to_string_tag_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_accessor_property_with_flags(
            typed_array_prototype_local,
            key_local,
            Some((payload_local, tag_local)),
            None,
            false,
            true,
            function,
        )?;

        let at_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeAt.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.at`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "at",
            at_meta,
            function,
        )?;

        for (name, builtin) in [
            ("includes", StandardBuiltinId::TypedArrayPrototypeIncludes),
            ("indexOf", StandardBuiltinId::TypedArrayPrototypeIndexOf),
            (
                "lastIndexOf",
                StandardBuiltinId::TypedArrayPrototypeLastIndexOf,
            ),
            ("find", StandardBuiltinId::TypedArrayPrototypeFind),
            ("findIndex", StandardBuiltinId::TypedArrayPrototypeFindIndex),
            ("findLast", StandardBuiltinId::TypedArrayPrototypeFindLast),
            (
                "findLastIndex",
                StandardBuiltinId::TypedArrayPrototypeFindLastIndex,
            ),
            ("every", StandardBuiltinId::TypedArrayPrototypeEvery),
            ("some", StandardBuiltinId::TypedArrayPrototypeSome),
            ("map", StandardBuiltinId::TypedArrayPrototypeMap),
            ("filter", StandardBuiltinId::TypedArrayPrototypeFilter),
            ("forEach", StandardBuiltinId::TypedArrayPrototypeForEach),
            ("reduce", StandardBuiltinId::TypedArrayPrototypeReduce),
            (
                "reduceRight",
                StandardBuiltinId::TypedArrayPrototypeReduceRight,
            ),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(
                typed_array_prototype_local,
                name,
                meta,
                function,
            )?;
        }

        let values_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeValues.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.values`",
                )
            })?;
        self.emit_object_define_function_data_with_aliases(
            typed_array_prototype_local,
            "values",
            &["Symbol.iterator"],
            values_meta,
            function,
        )?;
        for (name, builtin) in [
            ("keys", StandardBuiltinId::TypedArrayPrototypeKeys),
            ("entries", StandardBuiltinId::TypedArrayPrototypeEntries),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(
                typed_array_prototype_local,
                name,
                meta,
                function,
            )?;
        }

        let fill_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeFill.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.fill`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "fill",
            fill_meta,
            function,
        )?;

        let join_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeJoin.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.join`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "join",
            join_meta,
            function,
        )?;

        let subarray_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeSubarray.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.subarray`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "subarray",
            subarray_meta,
            function,
        )?;

        let slice_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeSlice.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.slice`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "slice",
            slice_meta,
            function,
        )?;

        let set_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeSet.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.set`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "set",
            set_meta,
            function,
        )?;

        let reverse_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeReverse.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.reverse`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "reverse",
            reverse_meta,
            function,
        )?;

        let copy_within_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeCopyWithin.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.copyWithin`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "copyWithin",
            copy_within_meta,
            function,
        )?;

        let sort_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeSort.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.sort`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "sort",
            sort_meta,
            function,
        )?;

        let to_reversed_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeToReversed.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.toReversed`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "toReversed",
            to_reversed_meta,
            function,
        )?;

        let to_sorted_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeToSorted.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.toSorted`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "toSorted",
            to_sorted_meta,
            function,
        )?;

        let with_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeWith.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.with`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "with",
            with_meta,
            function,
        )?;

        self.emit_object_define_function_global_data(
            typed_array_prototype_local,
            "toString",
            ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX,
            function,
        )?;
        let to_locale_string_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayPrototypeToLocaleString.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.prototype.toLocaleString`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "toLocaleString",
            to_locale_string_meta,
            function,
        )?;
        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayFrom.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.from`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_constructor_local,
            "from",
            from_meta,
            function,
        )?;
        let of_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `TypedArray.of`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_constructor_local,
            "of",
            of_meta,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(typed_array_prototype_local);
        self.release_temp_local(typed_array_constructor_local);
        Ok(())
    }

    pub(crate) fn repair_typed_array_constructor_graph(
        &mut self,
        builtin: StandardBuiltinId,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_global_index =
            standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing builtin constructor global `{}`",
                    builtin.debug_name()
                ))
            })?;
        let constructor_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(constructor_global_index));
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::GlobalGet(
            TYPED_ARRAY_CONSTRUCTOR_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            constructor_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Function.tag() as u64,
            function,
        );
        self.load_i64_to_local_from_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data_with_configurable(
            constructor_local,
            key_local,
            prototype_local,
            tag_local,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_define_data(
            prototype_local,
            key_local,
            constructor_local,
            tag_local,
            function,
        )?;
        let slot = NonArrayRealmIntrinsicSlot::for_typed_array_constructor(builtin)
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing realm intrinsic prototype slot `{}`",
                    builtin.debug_name()
                ))
            })?;
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_non_array_realm_intrinsic(
            self.scratch_local,
            slot,
            prototype_local,
            function,
        );
        self.release_temp_local(tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    pub(crate) fn repair_error_constructor_graph(
        &mut self,
        constructor_global_index: u32,
        prototype_global_index: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(constructor_global_index));
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::GlobalGet(ERROR_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            prototype_local,
            key_local,
            constructor_local,
            tag_local,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    pub(crate) fn repair_native_error_constructor_graphs(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (constructor_global_index, prototype_global_index) in [
            (
                EVAL_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                AGGREGATE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                SUPPRESSED_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                RANGE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                SYNTAX_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                TYPE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                URI_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
            (
                REFERENCE_ERROR_CONSTRUCTOR_GLOBAL_INDEX,
                REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ),
        ] {
            self.repair_error_constructor_graph(
                constructor_global_index,
                prototype_global_index,
                function,
            )?;
        }
        Ok(())
    }

    pub(crate) fn init_array_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let next_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorNext.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array Iterator.prototype.next`",
                )
            })?;
        let iterator_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorIdentity.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Array Iterator.prototype[Symbol.iterator]`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(prototype_local, "next", next_meta, function)?;
        self.emit_object_define_function_data(
            prototype_local,
            "Symbol.iterator",
            iterator_meta,
            function,
        )?;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Array Iterator"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_string_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let next_meta = self
            .functions
            .get(&StandardBuiltinId::StringIteratorNext.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `String Iterator.prototype.next`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(prototype_local, "next", next_meta, function)?;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("String Iterator"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_map_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let next_meta = self
            .functions
            .get(&StandardBuiltinId::MapIteratorNext.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Map Iterator.prototype.next`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(prototype_local, "next", &next_meta, function)?;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Map Iterator")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_set_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        let next_meta = self
            .functions
            .get(&StandardBuiltinId::SetIteratorNext.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Set Iterator.prototype.next`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_object_define_function_data(prototype_local, "next", &next_meta, function)?;
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Set Iterator")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_generator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(GENERATOR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_local));
        let constructor_key_local = self.reserve_temp_local();
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(constructor_key_local));
        function.instruction(&Instruction::GlobalGet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(constructor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            constructor_key_local,
            constructor_payload_local,
            constructor_tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        self.release_temp_local(constructor_key_local);
        for (name, builtin) in [
            ("next", StandardBuiltinId::GeneratorPrototypeNext),
            ("return", StandardBuiltinId::GeneratorPrototypeReturn),
            ("throw", StandardBuiltinId::GeneratorPrototypeThrow),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_object_define_function_data(prototype_local, name, &meta, function)?;
        }
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("Generator")));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_generator_function_intrinsics(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thrower_meta = self
            .functions
            .get(&StandardBuiltinId::ThrowTypeError.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `%ThrowTypeError%`",
                )
            })?;
        let mut constructor_meta = thrower_meta;
        constructor_meta.name = "GeneratorFunction".to_string();
        constructor_meta.to_string_value =
            "function GeneratorFunction() { [native code] }".to_string();
        constructor_meta.length = 1;
        constructor_meta.length_name_configurable = true;
        constructor_meta.protocol = FunctionProtocolIr::OrdinaryCallAndConstruct;

        self.emit_function_value_payload_with_prototype_materialization(
            &constructor_meta,
            FunctionPrototypeMaterialization::BootstrapSupplied,
            function,
        )?;
        let constructor_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::GlobalGet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );

        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            constructor_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(constructor_local));
        function.instruction(&Instruction::GlobalSet(
            GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::GeneratorFunctionConstructor,
            function,
        );

        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        for (name, value_global_index, value_kind) in [
            (
                "prototype",
                GENERATOR_PROTOTYPE_GLOBAL_INDEX,
                ValueKind::Object,
            ),
            (
                "constructor",
                GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                ValueKind::Function,
            ),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::GlobalGet(value_global_index));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(value_kind.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_data_property_with_flags(
                prototype_local,
                key_local,
                payload_local,
                tag_local,
                false,
                false,
                true,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("GeneratorFunction"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;
        self.release_temp_local(prototype_local);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    pub(crate) fn init_async_iterator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        let mut identity_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorIdentity.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing iterator identity builtin meta",
                )
            })?;
        identity_meta.name = "[Symbol.asyncIterator]".to_string();
        identity_meta.to_string_value =
            "function [Symbol.asyncIterator]() { [native code] }".to_string();
        self.emit_object_define_function_data(
            prototype_local,
            "Symbol.asyncIterator",
            &identity_meta,
            function,
        )?;
        let async_dispose_meta = self
            .functions
            .get(
                &StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose
                    .function_id(),
            )
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing AsyncIterator asyncDispose builtin meta",
                )
            })?;
        self.emit_object_define_function_data(
            prototype_local,
            "Symbol.asyncDispose",
            &async_dispose_meta,
            function,
        )?;
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_async_generator_prototype(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));

        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(
            ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        for (name, builtin) in [
            ("next", StandardBuiltinId::AsyncGeneratorPrototypeNext),
            ("return", StandardBuiltinId::AsyncGeneratorPrototypeReturn),
            ("throw", StandardBuiltinId::AsyncGeneratorPrototypeThrow),
        ] {
            let method_meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_function_value_payload(&method_meta, function)?;
            function.instruction(&Instruction::LocalSet(payload_local));
            self.store_i64_local_at_offset(
                payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                payload_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_local_data(
                prototype_local,
                name,
                payload_local,
                tag_local,
                function,
            )?;
        }

        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("AsyncGenerator"),
        ));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_object_append_data_property_with_flags(
            prototype_local,
            key_local,
            payload_local,
            tag_local,
            false,
            false,
            true,
            function,
        )?;

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_async_function_intrinsics(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let thrower_meta = self
            .functions
            .get(&StandardBuiltinId::ThrowTypeError.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `%ThrowTypeError%`",
                )
            })?;

        for (name, source, prototype_global_index, constructor_global_index, slot) in [
            (
                "AsyncFunction",
                "function AsyncFunction() { [native code] }",
                ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
                ASYNC_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                NonArrayRealmIntrinsicSlot::AsyncFunctionConstructor,
            ),
            (
                "AsyncGeneratorFunction",
                "function AsyncGeneratorFunction() { [native code] }",
                ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
                ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                NonArrayRealmIntrinsicSlot::AsyncGeneratorFunctionConstructor,
            ),
        ] {
            let mut constructor_meta = thrower_meta.clone();
            constructor_meta.name = name.to_string();
            constructor_meta.to_string_value = source.to_string();
            constructor_meta.length = 1;
            constructor_meta.length_name_configurable = true;
            constructor_meta.protocol = FunctionProtocolIr::OrdinaryCallAndConstruct;

            self.emit_function_value_payload_with_prototype_materialization(
                &constructor_meta,
                FunctionPrototypeMaterialization::BootstrapSupplied,
                function,
            )?;
            let constructor_local = self.reserve_temp_local();
            function.instruction(&Instruction::LocalSet(constructor_local));
            function.instruction(&Instruction::GlobalGet(FUNCTION_CONSTRUCTOR_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                self.scratch_local,
                function,
            );
            self.store_i64_const_at_offset(
                constructor_local,
                HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                ValueKind::Function.tag() as u64,
                function,
            );

            let key_local = self.reserve_temp_local();
            let payload_local = self.reserve_temp_local();
            let tag_local = self.reserve_temp_local();
            function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_data_property_with_flags(
                constructor_local,
                key_local,
                payload_local,
                tag_local,
                false,
                false,
                false,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(constructor_local));
            function.instruction(&Instruction::GlobalSet(constructor_global_index));
            self.emit_store_current_realm_global_intrinsic(
                constructor_global_index,
                slot,
                function,
            );
            self.release_temp_local(tag_local);
            self.release_temp_local(payload_local);
            self.release_temp_local(key_local);
            self.release_temp_local(constructor_local);
        }

        let prototype_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        for (
            prototype_global_index,
            constructor_global_index,
            instance_prototype_global_index,
            to_string_tag,
        ) in [
            (
                ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
                ASYNC_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                None,
                "AsyncFunction",
            ),
            (
                ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
                ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_GLOBAL_INDEX,
                Some(ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX),
                "AsyncGeneratorFunction",
            ),
        ] {
            function.instruction(&Instruction::GlobalGet(prototype_global_index));
            function.instruction(&Instruction::LocalSet(prototype_local));
            if let Some(instance_prototype_global_index) = instance_prototype_global_index {
                function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(instance_prototype_global_index));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_append_data_property_with_flags(
                    prototype_local,
                    key_local,
                    payload_local,
                    tag_local,
                    false,
                    false,
                    true,
                    function,
                )?;
            }

            function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::GlobalGet(constructor_global_index));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_data_property_with_flags(
                prototype_local,
                key_local,
                payload_local,
                tag_local,
                false,
                false,
                true,
                function,
            )?;

            function.instruction(&Instruction::I64Const(
                self.strings
                    .property_key_symbol_payload("Symbol.toStringTag"),
            ));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(self.strings.payload(to_string_tag)));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_append_data_property_with_flags(
                prototype_local,
                key_local,
                payload_local,
                tag_local,
                false,
                false,
                true,
                function,
            )?;
        }

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn init_runtime_roots(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::GlobalSet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        let object_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_prototype_local));
        self.store_i64_const_at_offset(
            object_prototype_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_IMMUTABLE_PROTOTYPE,
            function,
        );
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_object_prototype(
            self.scratch_local,
            object_prototype_local,
            function,
        );
        self.release_temp_local(object_prototype_local);
        let function_prototype_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionPrototype.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Function.prototype`",
                )
            })?;
        self.emit_function_value_payload(&function_prototype_meta, function)?;
        function.instruction(&Instruction::GlobalSet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::FunctionPrototype,
            function,
        );
        let function_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(function_prototype_local));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            function_prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            function_prototype_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        self.release_temp_local(function_prototype_local);
        // Array.prototype is itself an Array exotic object.  Allocate it with
        // the array layout, then repair the allocator's default prototype
        // (which would otherwise point back at the not-yet-initialized global).
        let array_prototype_length_local = self.reserve_temp_local();
        let array_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(array_prototype_length_local));
        self.emit_alloc_array_payload_with_length(
            array_prototype_length_local,
            array_prototype_local,
            function,
        )?;
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            array_prototype_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            array_prototype_local,
            HEAP_ARRAY_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(array_prototype_local));
        function.instruction(&Instruction::GlobalSet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
        self.release_temp_local(array_prototype_local);
        self.release_temp_local(array_prototype_length_local);
        self.emit_store_current_realm_array_prototype_global(function);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::IteratorPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ITERATOR_HELPER_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ITERATOR_HELPER_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::IteratorHelperPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::IteratorFromWrapperPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            MAP_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::MapIteratorPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            SET_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::SetIteratorPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(GENERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            GENERATOR_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::GeneratorPrototype,
            function,
        );
        let callable_function_prototype_local = self.reserve_temp_local();
        let callable_function_prototype_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(callable_function_prototype_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(
            callable_function_prototype_tag_local,
        ));
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(callable_function_prototype_local),
            Some(callable_function_prototype_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::GeneratorFunctionPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::AsyncIteratorPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(callable_function_prototype_local),
            Some(callable_function_prototype_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::AsyncFunctionPrototype,
            function,
        );
        self.release_temp_local(callable_function_prototype_tag_local);
        self.release_temp_local(callable_function_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ASYNC_ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ASYNC_GENERATOR_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::AsyncGeneratorPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ASYNC_FUNCTION_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            ASYNC_GENERATOR_FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::AsyncGeneratorFunctionPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::NumberPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(STRING_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::StringPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(BOOLEAN_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::BooleanPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(PROMISE_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            PROMISE_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::PromisePrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(MAP_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            MAP_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::MapPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(WEAK_MAP_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            WEAK_MAP_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::WeakMapPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(WEAK_REF_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            WEAK_REF_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::WeakRefPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_global_intrinsic(
            FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::FinalizationRegistryPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(WEAK_SET_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            WEAK_SET_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::WeakSetPrototype,
            function,
        );
        // `%AsyncDisposableStack.prototype%` deliberately gets no
        // `HEAP_REALM_INTRINSICS_*` slot. The only case that could observe one
        // is `proto-from-ctor-realm.js`, which is a policy case
        // (`Function constructor dynamic code generation`) and cannot pass on
        // this backend; the constructor therefore falls back to the current
        // realm's global (`NewTargetPrototypeFallback::CurrentGlobal`) and the
        // realm-intrinsics record does not gain an AsyncDisposableStack slot.
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ASYNC_DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
        ));
        // `%DisposableStack.prototype%` follows the same source-free realm
        // policy as its async sibling. Dynamic Function construction is the
        // only pinned test that distinguishes a created-realm slot.
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SET_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            SET_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::SetPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            SYMBOL_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::SymbolPrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ERROR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_message_error_prototype(
            ErrorMessageConstructorKind::Error,
            function,
        );
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            TYPE_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.emit_store_current_realm_message_error_prototype(
            ErrorMessageConstructorKind::TypeError,
            function,
        );
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_store_current_realm_message_error_prototype(
            ErrorMessageConstructorKind::ReferenceError,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            REFERENCE_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_message_error_prototype(
            ErrorMessageConstructorKind::EvalError,
            function,
        );
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            EVAL_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::GlobalGet(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            AGGREGATE_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::GlobalGet(
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            SUPPRESSED_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_message_error_prototype(
            ErrorMessageConstructorKind::RangeError,
            function,
        );
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            RANGE_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_message_error_prototype(
            ErrorMessageConstructorKind::SyntaxError,
            function,
        );
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            SYNTAX_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ERROR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(URI_ERROR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_message_error_prototype(
            ErrorMessageConstructorKind::URIError,
            function,
        );
        let native_error_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(URI_ERROR_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(native_error_prototype_local));
        self.emit_object_define_string_data(
            native_error_prototype_local,
            "name",
            URI_ERROR_NAME,
            function,
        )?;
        self.emit_object_define_string_data(native_error_prototype_local, "message", "", function)?;
        self.release_temp_local(native_error_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            SHARED_ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(DATA_VIEW_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(DATE_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            DATE_PROTOTYPE_GLOBAL_INDEX,
            NonArrayRealmIntrinsicSlot::DatePrototype,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
        ));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        let regexp_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(regexp_prototype_local));
        self.store_i64_const_at_offset(
            regexp_prototype_local,
            HEAP_REGEXP_ORIGINAL_SOURCE_PAYLOAD_OFFSET,
            self.strings.payload("(?:)") as u64,
            function,
        );
        self.store_i64_const_at_offset(
            regexp_prototype_local,
            HEAP_REGEXP_ORIGINAL_FLAGS_PAYLOAD_OFFSET,
            self.strings.payload("") as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(regexp_prototype_local));
        function.instruction(&Instruction::GlobalSet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_non_array_realm_intrinsic(
            self.scratch_local,
            NonArrayRealmIntrinsicSlot::RegExpPrototype,
            regexp_prototype_local,
            function,
        );
        self.release_temp_local(regexp_prototype_local);
        // These per-realm function-value globals cache the `@@match`/`@@matchAll`/
        // `@@search` methods and the shared Array/TypedArray `toString`. Their only
        // readers are inside constructor-init and builtin bodies that are
        // themselves gated on (or force-compiled from) the same planned kind: the
        // RegExp `@@` slots are read by `init_builtin_constructor_object(RegExp)`
        // and by the String regexp-protocol method bodies (which force RegExp), and
        // the shared `toString` slot is read by the Array / TypedArray prototype
        // setup. When the guarding constructor cannot exist in this module, the
        // slot is never read, so materializing it here would only force a
        // dead builtin body through the emission fixpoint. Skip it (shape-guarded
        // recording — see `FunctionMetaRegistry`).
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::RegExpConstructor)
        {
            let regexp_match_meta = self
                .functions
                .get(&StandardBuiltinId::RegExpPrototypeSymbolMatch.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.match]`",
                    )
                })?;
            self.emit_function_value_payload(&regexp_match_meta, function)?;
            function.instruction(&Instruction::GlobalSet(
                REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX,
            ));
            let regexp_match_all_meta = self
                .functions
                .get(&StandardBuiltinId::RegExpPrototypeSymbolMatchAll.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.matchAll]`",
                    )
                })?;
            self.emit_function_value_payload(&regexp_match_all_meta, function)?;
            function.instruction(&Instruction::GlobalSet(
                REGEXP_PROTOTYPE_SYMBOL_MATCH_ALL_GLOBAL_INDEX,
            ));
            let regexp_search_meta = self
                .functions
                .get(&StandardBuiltinId::RegExpPrototypeSymbolSearch.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.search]`",
                    )
                })?;
            self.emit_function_value_payload(&regexp_search_meta, function)?;
            function.instruction(&Instruction::GlobalSet(
                REGEXP_PROTOTYPE_SYMBOL_SEARCH_GLOBAL_INDEX,
            ));
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ArrayConstructor)
            || self.runtime_bootstrap_plan.needs_typed_array_intrinsic()
        {
            let array_typed_array_to_string_meta = self
                .functions
                .get(&StandardBuiltinId::TypedArrayPrototypeToString.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `Array.prototype.toString`",
                    )
                })?;
            self.emit_function_value_payload(&array_typed_array_to_string_meta, function)?;
            function.instruction(&Instruction::GlobalSet(
                ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX,
            ));
        }
        self.init_builtin_constructor_object(
            StandardBuiltinId::FunctionConstructor,
            FUNCTION_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_throw_type_error_intrinsic(function)?;
        self.init_generator_function_intrinsics(function)?;
        self.init_async_function_intrinsics(function)?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ObjectConstructor,
            OBJECT_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::ProxyConstructor,
                OBJECT_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::IteratorConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::IteratorConstructor,
                ITERATOR_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ArrayConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::ArrayConstructor,
                ARRAY_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        self.init_array_iterator_prototype(function)?;
        self.init_string_iterator_prototype(function)?;
        self.init_map_iterator_prototype(function)?;
        self.init_set_iterator_prototype(function)?;
        self.init_generator_prototype(function)?;
        self.init_async_iterator_prototype(function)?;
        self.init_async_generator_prototype(function)?;
        let array_iterator_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            ARRAY_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(array_iterator_prototype_local));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_array_iterator_prototype(
            self.scratch_local,
            array_iterator_prototype_local,
            function,
        );
        self.release_temp_local(array_iterator_prototype_local);
        let string_iterator_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(
            STRING_ITERATOR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(string_iterator_prototype_local));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_string_iterator_prototype(
            self.scratch_local,
            string_iterator_prototype_local,
            function,
        );
        self.release_temp_local(string_iterator_prototype_local);
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ArrayBufferConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::ArrayBufferConstructor,
                ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::SharedArrayBufferConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::SharedArrayBufferConstructor,
                SHARED_ARRAY_BUFFER_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::DataViewConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::DataViewConstructor,
                DATA_VIEW_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::DateConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::DateConstructor,
                DATE_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalInstantConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalInstantConstructor,
                TEMPORAL_INSTANT_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainDateConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainDateConstructor,
                TEMPORAL_PLAIN_DATE_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalDurationConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalDurationConstructor,
                TEMPORAL_DURATION_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainTimeConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainTimeConstructor,
                TEMPORAL_PLAIN_TIME_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainDateTimeConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainDateTimeConstructor,
                TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(
                StandardBuiltinId::TemporalPlainYearMonthConstructor,
            )
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainYearMonthConstructor,
                TEMPORAL_PLAIN_YEAR_MONTH_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalPlainMonthDayConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalPlainMonthDayConstructor,
                TEMPORAL_PLAIN_MONTH_DAY_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::TemporalZonedDateTimeConstructor,
                TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::IntlLocaleConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::IntlLocaleConstructor,
                INTL_LOCALE_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::IntlDateTimeFormatConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::IntlDateTimeFormatConstructor,
                INTL_DATE_TIME_FORMAT_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::RegExpConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::RegExpConstructor,
                REGEXP_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self.runtime_bootstrap_plan.needs_typed_array_intrinsic() {
            self.init_typed_array_intrinsic(function)?;
        }
        for builtin in [
            StandardBuiltinId::Float64ArrayConstructor,
            StandardBuiltinId::Float32ArrayConstructor,
            StandardBuiltinId::Int32ArrayConstructor,
            StandardBuiltinId::Int16ArrayConstructor,
            StandardBuiltinId::Int8ArrayConstructor,
            StandardBuiltinId::Uint32ArrayConstructor,
            StandardBuiltinId::Uint16ArrayConstructor,
            StandardBuiltinId::Uint8ArrayConstructor,
            StandardBuiltinId::Uint8ClampedArrayConstructor,
            StandardBuiltinId::BigInt64ArrayConstructor,
            StandardBuiltinId::BigUint64ArrayConstructor,
        ] {
            if self
                .runtime_bootstrap_plan
                .should_initialize_standard_builtin(builtin)
            {
                self.init_builtin_constructor_object(
                    builtin,
                    TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX,
                    function,
                )?;
            }
        }
        for builtin in [
            StandardBuiltinId::Float64ArrayConstructor,
            StandardBuiltinId::Float32ArrayConstructor,
            StandardBuiltinId::Int32ArrayConstructor,
            StandardBuiltinId::Int16ArrayConstructor,
            StandardBuiltinId::Int8ArrayConstructor,
            StandardBuiltinId::Uint32ArrayConstructor,
            StandardBuiltinId::Uint16ArrayConstructor,
            StandardBuiltinId::Uint8ArrayConstructor,
            StandardBuiltinId::Uint8ClampedArrayConstructor,
            StandardBuiltinId::BigInt64ArrayConstructor,
            StandardBuiltinId::BigUint64ArrayConstructor,
        ] {
            if self
                .runtime_bootstrap_plan
                .should_initialize_standard_builtin(builtin)
            {
                self.repair_typed_array_constructor_graph(builtin, function)?;
            }
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::NumberConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::NumberConstructor,
                NUMBER_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::StringConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::StringConstructor,
                STRING_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
            let string_constructor_local = self.reserve_temp_local();
            let from_code_point_meta = self
                .functions
                .get(&StandardBuiltinId::StringFromCodePoint.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `String.fromCodePoint`",
                    )
                })?;
            function.instruction(&Instruction::GlobalGet(STRING_CONSTRUCTOR_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(string_constructor_local));
            let from_char_code_meta = self
                .functions
                .get(&StandardBuiltinId::StringFromCharCode.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `String.fromCharCode`",
                    )
                })?;
            self.emit_object_define_function_data(
                string_constructor_local,
                "fromCharCode",
                from_char_code_meta,
                function,
            )?;
            self.emit_object_define_function_data(
                string_constructor_local,
                "fromCodePoint",
                from_code_point_meta,
                function,
            )?;
            let raw_meta = self
                .functions
                .get(&StandardBuiltinId::StringRaw.function_id())
                .ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `String.raw`",
                    )
                })?;
            self.emit_object_define_function_data(
                string_constructor_local,
                "raw",
                raw_meta,
                function,
            )?;
            self.release_temp_local(string_constructor_local);
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::BooleanConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::BooleanConstructor,
                BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::PromiseConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::PromiseConstructor,
                PROMISE_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
            self.emit_store_current_realm_global_intrinsic(
                PROMISE_CONSTRUCTOR_GLOBAL_INDEX,
                NonArrayRealmIntrinsicSlot::PromiseConstructor,
                function,
            );
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::MapConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::MapConstructor,
                MAP_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::WeakMapConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::WeakMapConstructor,
                WEAK_MAP_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::WeakRefConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::WeakRefConstructor,
                WEAK_REF_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::FinalizationRegistryConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::FinalizationRegistryConstructor,
                FINALIZATION_REGISTRY_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::WeakSetConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::WeakSetConstructor,
                WEAK_SET_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::AsyncDisposableStackConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::AsyncDisposableStackConstructor,
                ASYNC_DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::DisposableStackConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::DisposableStackConstructor,
                DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::SetConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::SetConstructor,
                SET_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::SymbolConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::SymbolConstructor,
                SYMBOL_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::BigIntConstructor)
        {
            self.init_builtin_constructor_object(
                StandardBuiltinId::BigIntConstructor,
                OBJECT_PROTOTYPE_GLOBAL_INDEX,
                function,
            )?;
        }
        self.init_builtin_constructor_object(
            StandardBuiltinId::ErrorConstructor,
            ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::EvalErrorConstructor,
            EVAL_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::AggregateErrorConstructor,
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::SuppressedErrorConstructor,
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::RangeErrorConstructor,
            RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::SyntaxErrorConstructor,
            SYNTAX_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::TypeErrorConstructor,
            TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::URIErrorConstructor,
            URI_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.init_builtin_constructor_object(
            StandardBuiltinId::ReferenceErrorConstructor,
            REFERENCE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            function,
        )?;
        self.repair_native_error_constructor_graphs(function)?;
        let object_prototype_local = self.reserve_temp_local();
        let object_constructor_local = self.reserve_temp_local();
        let object_constructor_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_prototype_local));
        function.instruction(&Instruction::GlobalGet(OBJECT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_constructor_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_constructor_tag_local));
        self.emit_object_append_local_data_property_with_flags(
            object_prototype_local,
            "constructor",
            object_constructor_local,
            object_constructor_tag_local,
            true,
            false,
            true,
            function,
        )?;
        self.release_temp_local(object_constructor_tag_local);
        self.release_temp_local(object_constructor_local);
        self.release_temp_local(object_prototype_local);
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.reflect_object
        {
            self.init_reflect_object(function)?;
        }
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.math_object
        {
            self.init_math_object(function)?;
        }
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.json_object
        {
            self.init_json_object(function)?;
        }
        if self.runtime_bootstrap_plan.full_standard_globals
            || self.runtime_bootstrap_plan.atomics_object
        {
            self.init_atomics_object(function)?;
        }
        if let Some(temporal_namespace_members) =
            self.runtime_bootstrap_plan.temporal_namespace_members()
        {
            self.init_temporal_object(temporal_namespace_members, function)?;
        }
        // Unlike its five siblings above, the `Intl` gate hands back the member
        // list rather than a bool: "install `Intl`" and "every member the IR
        // shape declares is rooted" are one decision, made once in `planning`.
        if let Some(intl_namespace_members) = self.runtime_bootstrap_plan.intl_namespace_members() {
            self.init_intl_object(intl_namespace_members, function)?;
        }
        Ok(())
    }

    pub(crate) fn init_script_global_object(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }

        let object_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let script_global_bindings = self
            .script_global_bindings
            .expect("main builder must carry the global binding plan")
            .iter()
            .cloned()
            .filter(|binding| {
                self.runtime_bootstrap_plan
                    .should_install_script_global_binding(&binding.initializer)
            })
            .collect::<Vec<_>>();
        let capacity = (script_global_bindings.len() as u64).max(MIN_HEAP_CAPACITY);

        self.emit_heap_alloc_const(HEAP_HEADER_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_heap_alloc_const(capacity * HEAP_OBJECT_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(buffer_local));
        self.store_i64_local_at_offset(object_local, HEAP_PTR_OFFSET, buffer_local, function);
        self.store_i64_const_at_offset(object_local, HEAP_LEN_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_CAP_OFFSET, capacity, function);
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            BOXED_PRIMITIVE_KIND_NONE,
            function,
        );
        self.store_i64_const_at_offset(object_local, HEAP_OBJECT_BOXED_TAG_OFFSET, 0, function);
        self.store_i64_const_at_offset(object_local, HEAP_OBJECT_BOXED_PAYLOAD_OFFSET, 0, function);
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            0,
            function,
        );
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_PROTOTYPE_TAG_OFFSET,
            ValueKind::Object.tag() as u64,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            self.scratch_local,
            HEAP_REALM_GLOBAL_OBJECT_OFFSET,
            object_local,
            function,
        );
        self.store_i64_local_at_offset(
            self.scratch_local,
            HEAP_REALM_GLOBAL_THIS_OFFSET,
            object_local,
            function,
        );
        self.store_i64_local_at_offset(
            self.scratch_local,
            HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
            self.current_env_local,
            function,
        );

        for binding in script_global_bindings {
            function.instruction(&Instruction::I64Const(self.strings.payload(&binding.name)));
            function.instruction(&Instruction::LocalSet(key_local));
            match &binding.initializer {
                GlobalPropertyInitializerIr::Intrinsic => {
                    function.instruction(&Instruction::LocalGet(object_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::Infinity => {
                    function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::NaN => {
                    function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::Undefined => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::FreshUndefined => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::SourceFunction(function_id) => {
                    let meta = self.functions.get(function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unknown script-global function `{function_id}`"
                        ))
                    })?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::ReflectObject => {
                    function.instruction(&Instruction::GlobalGet(REFLECT_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::MathObject => {
                    function.instruction(&Instruction::GlobalGet(MATH_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::JsonObject => {
                    function.instruction(&Instruction::GlobalGet(JSON_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::AtomicsObject => {
                    function.instruction(&Instruction::GlobalGet(ATOMICS_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::TemporalObject => {
                    function.instruction(&Instruction::GlobalGet(TEMPORAL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::IntlObject => {
                    function.instruction(&Instruction::GlobalGet(INTL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::BuiltinFunction(builtin) => {
                    if let Some(global_index) = standard_builtin_constructor_global_index(*builtin)
                    {
                        function.instruction(&Instruction::GlobalGet(global_index));
                        function.instruction(&Instruction::LocalSet(payload_local));
                    } else {
                        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                                builtin.debug_name()
                            ))
                        })?;
                        self.emit_function_value_payload(meta, function)?;
                        function.instruction(&Instruction::LocalSet(payload_local));
                    }
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                GlobalPropertyInitializerIr::HostFunction(builtin) => {
                    let meta = self
                        .functions
                        .get(&builtin.function_id())
                        .cloned()
                        .ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: unknown script-global host function `{}`",
                                builtin.as_str()
                            ))
                        })?;
                    // `parseInt`/`parseFloat` must be the same object as
                    // `Number.parseInt`/`Number.parseFloat`; source them from the
                    // canonical per-realm global rather than a fresh allocation.
                    if let Some(global_index) =
                        canonical_host_function_global_index_by_name(binding.name.as_str())
                    {
                        self.emit_ensure_canonical_host_function(&meta, global_index, function)?;
                        function.instruction(&Instruction::GlobalGet(global_index));
                    } else {
                        self.emit_function_value_payload(&meta, function)?;
                    }
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
            }
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                payload_local,
                tag_local,
                binding.initializer.writable(),
                binding.initializer.enumerable(),
                binding.initializer.configurable(),
                function,
            )?;
        }

        if let Some(slot) = self.owned_env_slot(LEXICAL_THIS_NAME) {
            function.instruction(&Instruction::LocalGet(object_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.write_env_slot_from_locals(slot, 0, payload_local, tag_local, function);
        }

        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(object_local);
        Ok(())
    }
}
