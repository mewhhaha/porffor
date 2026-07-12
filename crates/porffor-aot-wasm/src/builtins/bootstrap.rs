use super::super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn init_builtin_constructor_object(
        &mut self,
        builtin: StandardBuiltinId,
        prototype_global_index: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
            EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                builtin.debug_name()
            ))
        })?;
        let constructor_global_index =
            standard_builtin_constructor_global_index(builtin).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin constructor global `{}`",
                    builtin.debug_name()
                ))
            })?;
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        let prototype_object_local = self.reserve_temp_local();

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
                let prototype_kind = if builtin == StandardBuiltinId::ArrayConstructor {
                    ValueKind::Array
                } else {
                    ValueKind::Object
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

                function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
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

        match builtin {
            StandardBuiltinId::FunctionConstructor => {
                let prototype_object_local = self.reserve_temp_local();
                let call_meta = self
                    .functions
                    .get(&StandardBuiltinId::FunctionPrototypeCall.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.call`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "call",
                    call_meta,
                    function,
                )?;
                let apply_meta = self
                    .functions
                    .get(&StandardBuiltinId::FunctionPrototypeApply.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.apply`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "apply",
                    apply_meta,
                    function,
                )?;
                let bind_meta = self
                    .functions
                    .get(&StandardBuiltinId::FunctionPrototypeBind.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.bind`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "bind",
                    bind_meta,
                    function,
                )?;
                let to_string_meta = self
                    .functions
                    .get(&StandardBuiltinId::FunctionPrototypeToString.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Function.prototype.toString`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "toString",
                    to_string_meta,
                    function,
                )?;
                self.release_temp_local(prototype_object_local);
            }
            StandardBuiltinId::ObjectConstructor => {
                let prototype_object_local = self.reserve_temp_local();
                let has_own_property_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectPrototypeHasOwnProperty.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.hasOwnProperty`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "hasOwnProperty",
                    has_own_property_meta,
                    function,
                )?;
                let property_is_enumerable_meta = self
                    .functions
                    .get(
                        &StandardBuiltinId::ObjectPrototypePropertyIsEnumerable.function_id(),
                    )
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.propertyIsEnumerable`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "propertyIsEnumerable",
                    property_is_enumerable_meta,
                    function,
                )?;
                let is_prototype_of_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectPrototypeIsPrototypeOf.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.isPrototypeOf`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "isPrototypeOf",
                    is_prototype_of_meta,
                    function,
                )?;
                let to_string_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectPrototypeToString.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.toString`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "toString",
                    to_string_meta,
                    function,
                )?;
                let to_locale_string_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectPrototypeToLocaleString.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.toLocaleString`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "toLocaleString",
                    to_locale_string_meta,
                    function,
                )?;
                let value_of_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectPrototypeValueOf.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.valueOf`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "valueOf",
                    value_of_meta,
                    function,
                )?;
                self.release_temp_local(prototype_object_local);

                let create_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectCreate.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.create`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "create",
                    create_meta,
                    function,
                )?;
                let get_proto_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectGetPrototypeOf.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getPrototypeOf`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "getPrototypeOf",
                    get_proto_meta,
                    function,
                )?;
                let set_proto_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectSetPrototypeOf.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.setPrototypeOf`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "setPrototypeOf",
                    set_proto_meta,
                    function,
                )?;
                let define_property_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.defineProperty`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "defineProperty",
                    define_property_meta,
                    function,
                )?;
                let get_own_property_descriptor_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyDescriptor`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "getOwnPropertyDescriptor",
                    get_own_property_descriptor_meta,
                    function,
                )?;
                let get_own_property_names_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectGetOwnPropertyNames.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyNames`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "getOwnPropertyNames",
                    get_own_property_names_meta,
                    function,
                )?;
                let get_own_property_symbols_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectGetOwnPropertySymbols.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertySymbols`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "getOwnPropertySymbols",
                    get_own_property_symbols_meta,
                    function,
                )?;
                let keys_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectKeys.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.keys`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "keys", keys_meta, function)?;
                let values_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectValues.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.values`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "values",
                    values_meta,
                    function,
                )?;
                let has_own_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectHasOwn.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.hasOwn`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "hasOwn",
                    has_own_meta,
                    function,
                )?;
                let define_properties_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectDefineProperties.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.defineProperties`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "defineProperties",
                    define_properties_meta,
                    function,
                )?;
                let is_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectIs.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.is`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "is", is_meta, function)?;
                let is_sealed_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectIsSealed.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.isSealed`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isSealed",
                    is_sealed_meta,
                    function,
                )?;
                let is_frozen_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectIsFrozen.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.isFrozen`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isFrozen",
                    is_frozen_meta,
                    function,
                )?;
                let freeze_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectFreeze.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.freeze`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "freeze",
                    freeze_meta,
                    function,
                )?;
                let is_extensible_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectIsExtensible.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.isExtensible`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isExtensible",
                    is_extensible_meta,
                    function,
                )?;
                let prevent_extensions_meta = self
                    .functions
                    .get(&StandardBuiltinId::ObjectPreventExtensions.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.preventExtensions`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "preventExtensions",
                    prevent_extensions_meta,
                    function,
                )?;
            }
            StandardBuiltinId::ProxyConstructor => {
                let revocable_meta = self
                    .functions
                    .get(&StandardBuiltinId::ProxyRevocable.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Proxy.revocable`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "revocable",
                    revocable_meta,
                    function,
                )?;
            }
            StandardBuiltinId::RegExpConstructor => {
                let key_local = self.reserve_temp_local();
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                let match_all_meta = self
                    .functions
                    .get(&StandardBuiltinId::RegExpPrototypeSymbolMatchAll.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.matchAll]`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(REGEXP_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(object_local));
                for (name, getter) in [
                    ("source", StandardBuiltinId::RegExpPrototypeSourceGetter),
                    (
                        "hasIndices",
                        StandardBuiltinId::RegExpPrototypeHasIndicesGetter,
                    ),
                    ("global", StandardBuiltinId::RegExpPrototypeGlobalGetter),
                    (
                        "ignoreCase",
                        StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter,
                    ),
                    (
                        "multiline",
                        StandardBuiltinId::RegExpPrototypeMultilineGetter,
                    ),
                    ("dotAll", StandardBuiltinId::RegExpPrototypeDotAllGetter),
                    ("unicode", StandardBuiltinId::RegExpPrototypeUnicodeGetter),
                    (
                        "unicodeSets",
                        StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter,
                    ),
                    ("sticky", StandardBuiltinId::RegExpPrototypeStickyGetter),
                    ("flags", StandardBuiltinId::RegExpPrototypeFlagsGetter),
                ] {
                    let getter_meta = self
                        .functions
                        .get(&getter.function_id())
                        .cloned()
                        .ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                                getter.debug_name()
                            ))
                        })?;
                    function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_function_value_payload(&getter_meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.emit_object_append_accessor_property_with_flags(
                        object_local,
                        key_local,
                        Some((payload_local, tag_local)),
                        None,
                        false,
                        true,
                        function,
                    )?;
                }
                function.instruction(&Instruction::I64Const(self.strings.payload("Symbol.match")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(
                    REGEXP_PROTOTYPE_SYMBOL_MATCH_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_append_data_property_with_flags(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.matchAll"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_function_value_payload(&match_all_meta, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_append_data_property_with_flags(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.search"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(
                    REGEXP_PROTOTYPE_SYMBOL_SEARCH_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_append_data_property_with_flags(
                    object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
                function.instruction(&Instruction::GlobalGet(REGEXP_CONSTRUCTOR_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(object_local));
                let escape_meta = self
                    .functions
                    .get(&StandardBuiltinId::RegExpEscape.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.escape`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "escape",
                    &escape_meta,
                    function,
                )?;
                let species_meta = self
                    .functions
                    .get(&StandardBuiltinId::RegExpSpeciesGetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp[Symbol.species]`",
                        )
                    })?;
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.species"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_function_value_payload(species_meta, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_append_accessor_property_with_flags(
                    object_local,
                    key_local,
                    Some((payload_local, tag_local)),
                    None,
                    false,
                    true,
                    function,
                )?;
                let getter_meta = self
                    .functions
                    .get(&StandardBuiltinId::RegExpLegacyStaticGetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp legacy static getter`",
                        )
                    })?;
                self.emit_function_value_payload(getter_meta, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                let getter = (payload_local, tag_local);
                let setter_payload_local = self.reserve_temp_local();
                let setter_tag_local = self.reserve_temp_local();
                let setter_meta = self
                    .functions
                    .get(&StandardBuiltinId::RegExpLegacyStaticSetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp legacy static setter`",
                        )
                    })?;
                self.emit_function_value_payload(setter_meta, function)?;
                function.instruction(&Instruction::LocalSet(setter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(setter_tag_local));
                let setter = (setter_payload_local, setter_tag_local);
                for name in ["input", "$_"] {
                    function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_append_accessor_property_with_flags(
                        object_local,
                        key_local,
                        Some(getter),
                        Some(setter),
                        false,
                        true,
                        function,
                    )?;
                }
                for name in [
                    "lastMatch",
                    "$&",
                    "lastParen",
                    "$+",
                    "leftContext",
                    "$`",
                    "rightContext",
                    "$'",
                    "$1",
                    "$2",
                    "$3",
                    "$4",
                    "$5",
                    "$6",
                    "$7",
                    "$8",
                    "$9",
                ] {
                    let payload = self.strings.payload(name);
                    function.instruction(&Instruction::I64Const(payload));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_object_append_accessor_property_with_flags(
                        object_local,
                        key_local,
                        Some(getter),
                        None,
                        false,
                        true,
                        function,
                    )?;
                }
                self.release_temp_local(setter_tag_local);
                self.release_temp_local(setter_payload_local);
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                self.release_temp_local(key_local);
            }
            StandardBuiltinId::IteratorConstructor => {
                let prototype_object_local = self.reserve_temp_local();
                let key_local = self.reserve_temp_local();
                let payload_local = self.reserve_temp_local();
                let tag_local = self.reserve_temp_local();
                let setter_payload_local = self.reserve_temp_local();
                let setter_tag_local = self.reserve_temp_local();
                let to_array_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeToArray.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.toArray`",
                        )
                    })?;
                let for_each_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeForEach.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.forEach`",
                        )
                    })?;
                let every_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeEvery.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.every`",
                        )
                    })?;
                let some_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeSome.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.some`",
                        )
                    })?;
                let find_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeFind.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.find`",
                        )
                    })?;
                let reduce_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeReduce.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.reduce`",
                        )
                    })?;
                let map_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeMap.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.map`",
                        )
                    })?;
                let filter_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeFilter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.filter`",
                        )
                    })?;
                let flat_map_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeFlatMap.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.flatMap`",
                        )
                    })?;
                let take_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeTake.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.take`",
                        )
                    })?;
                let drop_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeDrop.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.prototype.drop`",
                        )
                    })?;
                let constructor_getter_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeConstructorGetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `get %IteratorPrototype%.constructor`",
                        )
                    })?;
                let constructor_setter_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeConstructorSetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `set %IteratorPrototype%.constructor`",
                        )
                    })?;
                let from_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorFrom.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Iterator.from`",
                        )
                    })?;
                let wrapper_return_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorFromWrapperReturn.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `%WrapForValidIteratorPrototype%.return`",
                        )
                    })?;
                let wrapper_next_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorFromWrapperNext.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `%WrapForValidIteratorPrototype%.next`",
                        )
                    })?;
                let iterator_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayIteratorIdentity.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `%IteratorPrototype%[Symbol.iterator]`",
                        )
                    })?;
                let dispose_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeSymbolDispose.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `%IteratorPrototype%[Symbol.dispose]`",
                        )
                    })?;
                let to_string_tag_getter_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeToStringTagGetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `get %IteratorPrototype%[Symbol.toStringTag]`",
                        )
                    })?;
                let to_string_tag_setter_meta = self
                    .functions
                    .get(&StandardBuiltinId::IteratorPrototypeToStringTagSetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `set %IteratorPrototype%[Symbol.toStringTag]`",
                        )
                    })?;

                self.emit_object_define_function_data(object_local, "from", from_meta, function)?;
                function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::GlobalGet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
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

                function.instruction(&Instruction::GlobalGet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "toArray",
                    to_array_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "forEach",
                    for_each_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "every",
                    every_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "some",
                    some_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "find",
                    find_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "reduce",
                    reduce_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "map",
                    map_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "filter",
                    filter_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "flatMap",
                    flat_map_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "take",
                    take_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "drop",
                    drop_meta,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_function_value_payload(constructor_getter_meta, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_function_value_payload(constructor_setter_meta, function)?;
                function.instruction(&Instruction::LocalSet(setter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(setter_tag_local));
                self.emit_object_append_accessor_property_with_flags(
                    prototype_object_local,
                    key_local,
                    Some((payload_local, tag_local)),
                    Some((setter_payload_local, setter_tag_local)),
                    false,
                    true,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "Symbol.iterator",
                    iterator_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "Symbol.dispose",
                    dispose_meta,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.toStringTag"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_function_value_payload(to_string_tag_getter_meta, function)?;
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_function_value_payload(to_string_tag_setter_meta, function)?;
                function.instruction(&Instruction::LocalSet(setter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(setter_tag_local));
                self.emit_object_append_accessor_property_with_flags(
                    prototype_object_local,
                    key_local,
                    Some((payload_local, tag_local)),
                    Some((setter_payload_local, setter_tag_local)),
                    false,
                    true,
                    function,
                )?;
                function.instruction(&Instruction::GlobalGet(
                    ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
                ));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "next",
                    wrapper_next_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "return",
                    wrapper_return_meta,
                    function,
                )?;
                self.release_temp_local(setter_tag_local);
                self.release_temp_local(setter_payload_local);
                self.release_temp_local(tag_local);
                self.release_temp_local(payload_local);
                self.release_temp_local(key_local);
                self.release_temp_local(prototype_object_local);
            }
            StandardBuiltinId::ArrayConstructor => {
                let from_meta = self.functions.get(&StandardBuiltinId::ArrayFrom.function_id()).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.from`",
                    )
                })?;
                self.emit_object_define_function_data(object_local, "from", from_meta, function)?;
                let of_meta = self.functions.get(&StandardBuiltinId::ArrayOf.function_id()).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.of`",
                    )
                })?;
                self.emit_object_define_function_data(object_local, "of", of_meta, function)?;
                let is_array_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayIsArray.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.isArray`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isArray",
                    is_array_meta,
                    function,
                )?;
                let key_local = self.reserve_temp_local();
                let getter_payload_local = self.reserve_temp_local();
                let getter_tag_local = self.reserve_temp_local();
                let species_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArraySpeciesGetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array[Symbol.species]`",
                        )
                    })?;
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.species"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_function_value_payload(species_meta, function)?;
                function.instruction(&Instruction::LocalSet(getter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(getter_tag_local));
                self.emit_object_append_accessor_property_with_flags(
                    object_local,
                    key_local,
                    Some((getter_payload_local, getter_tag_local)),
                    None,
                    false,
                    true,
                    function,
                )?;
                self.release_temp_local(getter_tag_local);
                self.release_temp_local(getter_payload_local);
                self.release_temp_local(key_local);
                let concat_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeConcat.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.concat`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(object_local));
                self.emit_object_define_function_global_data(
                    object_local,
                    "toString",
                    ARRAY_TYPED_ARRAY_TO_STRING_GLOBAL_INDEX,
                    function,
                )?;
                self.emit_object_define_function_data(
                    object_local,
                    "concat",
                    concat_meta,
                    function,
                )?;
                let join_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeJoin.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.join`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "join", join_meta, function)?;
                let splice_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeSplice.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.splice`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "splice",
                    splice_meta,
                    function,
                )?;
                let to_locale_string_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeToLocaleString.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.toLocaleString`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "toLocaleString",
                    to_locale_string_meta,
                    function,
                )?;
                let flat_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeFlat.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.flat`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "flat", flat_meta, function)?;
                let flat_map_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeFlatMap.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.flatMap`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "flatMap",
                    flat_map_meta,
                    function,
                )?;
                let at_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeAt.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.at`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "at", at_meta, function)?;
                let includes_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeIncludes.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.includes`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "includes",
                    includes_meta,
                    function,
                )?;
                let index_of_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeIndexOf.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.indexOf`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "indexOf",
                    index_of_meta,
                    function,
                )?;
                let last_index_of_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeLastIndexOf.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.lastIndexOf`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "lastIndexOf",
                    last_index_of_meta,
                    function,
                )?;
                let find_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeFind.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.find`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "find", find_meta, function)?;
                let find_index_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeFindIndex.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.findIndex`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "findIndex",
                    find_index_meta,
                    function,
                )?;
                let find_last_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeFindLast.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.findLast`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "findLast",
                    find_last_meta,
                    function,
                )?;
                let find_last_index_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeFindLastIndex.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.findLastIndex`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "findLastIndex",
                    find_last_index_meta,
                    function,
                )?;
                let every_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeEvery.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.every`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "every", every_meta, function)?;
                let some_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeSome.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.some`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "some", some_meta, function)?;
                let for_each_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeForEach.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.forEach`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "forEach",
                    for_each_meta,
                    function,
                )?;
                let filter_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeFilter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.filter`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "filter",
                    filter_meta,
                    function,
                )?;
                let map_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeMap.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.map`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "map", map_meta, function)?;
                let reduce_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeReduce.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.reduce`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "reduce",
                    reduce_meta,
                    function,
                )?;
                let reduce_right_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeReduceRight.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.reduceRight`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "reduceRight",
                    reduce_right_meta,
                    function,
                )?;
                let pop_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypePop.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.pop`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "pop", pop_meta, function)?;
                let push_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypePush.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.push`",
                        )
                })?;
                self.emit_object_define_function_data(object_local, "push", push_meta, function)?;
                let keys_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeKeys.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.keys`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "keys", keys_meta, function)?;
                let entries_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeEntries.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.entries`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "entries",
                    entries_meta,
                    function,
                )?;
                let values_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.values`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "values",
                    values_meta,
                    function,
                )?;
                self.emit_object_define_function_data(
                    object_local,
                    "Symbol.iterator",
                    values_meta,
                    function,
                )?;
            }
            StandardBuiltinId::StringConstructor => {
                function.instruction(&Instruction::GlobalGet(STRING_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(object_local));
                function.instruction(&Instruction::I64Const(self.strings.payload("")));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_store_boxed_primitive_metadata(
                    object_local,
                    BOXED_PRIMITIVE_KIND_STRING,
                    payload_local,
                    tag_local,
                    function,
                );
                for builtin in [
                    StandardBuiltinId::StringPrototypeToString,
                    StandardBuiltinId::StringPrototypeValueOf,
                    StandardBuiltinId::StringPrototypeCharAt,
                    StandardBuiltinId::StringPrototypeCharCodeAt,
                    StandardBuiltinId::StringPrototypeCodePointAt,
                    StandardBuiltinId::StringPrototypeAt,
                    StandardBuiltinId::StringPrototypeAnchor,
                    StandardBuiltinId::StringPrototypeBig,
                    StandardBuiltinId::StringPrototypeBlink,
                    StandardBuiltinId::StringPrototypeBold,
                    StandardBuiltinId::StringPrototypeFixed,
                    StandardBuiltinId::StringPrototypeFontcolor,
                    StandardBuiltinId::StringPrototypeFontsize,
                    StandardBuiltinId::StringPrototypeItalics,
                    StandardBuiltinId::StringPrototypeLink,
                    StandardBuiltinId::StringPrototypeSmall,
                    StandardBuiltinId::StringPrototypeStrike,
                    StandardBuiltinId::StringPrototypeSub,
                    StandardBuiltinId::StringPrototypeSubstr,
                    StandardBuiltinId::StringPrototypeSubstring,
                    StandardBuiltinId::StringPrototypeSup,
                    StandardBuiltinId::StringPrototypeMatch,
                    StandardBuiltinId::StringPrototypeMatchAll,
                    StandardBuiltinId::StringPrototypeReplace,
                    StandardBuiltinId::StringPrototypeReplaceAll,
                    StandardBuiltinId::StringPrototypeSearch,
                    StandardBuiltinId::StringPrototypeIndexOf,
                    StandardBuiltinId::StringPrototypeLastIndexOf,
                    StandardBuiltinId::StringPrototypeSlice,
                    StandardBuiltinId::StringPrototypeSplit,
                    StandardBuiltinId::StringPrototypePadStart,
                    StandardBuiltinId::StringPrototypePadEnd,
                    StandardBuiltinId::StringPrototypeRepeat,
                    StandardBuiltinId::StringPrototypeEndsWith,
                    StandardBuiltinId::StringPrototypeIncludes,
                    StandardBuiltinId::StringPrototypeStartsWith,
                    StandardBuiltinId::StringPrototypeToUpperCase,
                    StandardBuiltinId::StringPrototypeTrim,
                    StandardBuiltinId::StringPrototypeTrimStart,
                    StandardBuiltinId::StringPrototypeTrimEnd,
                    StandardBuiltinId::StringPrototypeIsWellFormed,
                    StandardBuiltinId::StringPrototypeToWellFormed,
                ] {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    match builtin {
                        StandardBuiltinId::StringPrototypeTrimStart => {
                            self.emit_object_define_function_data_with_aliases(
                                object_local,
                                "trimStart",
                                &["trimLeft"],
                                meta,
                                function,
                            )?;
                        }
                        StandardBuiltinId::StringPrototypeTrimEnd => {
                            self.emit_object_define_function_data_with_aliases(
                                object_local,
                                "trimEnd",
                                &["trimRight"],
                                meta,
                                function,
                            )?;
                        }
                        _ => self.emit_object_define_function_data(
                            object_local,
                            builtin.string_prototype_method_name().unwrap(),
                            meta,
                            function,
                        )?,
                    }
                }
                let values_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `String.prototype[Symbol.iterator]`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "Symbol.iterator",
                    values_meta,
                    function,
                )?;
            }
            StandardBuiltinId::ArrayBufferConstructor
            | StandardBuiltinId::SharedArrayBufferConstructor => {
                let is_view_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayBufferIsView.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `ArrayBuffer.isView`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isView",
                    is_view_meta,
                    function,
                )?;

                let key_local = self.reserve_temp_local();
                let getter_payload_local = self.reserve_temp_local();
                let getter_tag_local = self.reserve_temp_local();
                let species_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayBufferSpeciesGetter.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `ArrayBuffer[Symbol.species]`",
                        )
                    })?;
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.species"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                self.emit_function_value_payload(species_meta, function)?;
                function.instruction(&Instruction::LocalSet(getter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(getter_tag_local));
                self.emit_object_append_accessor_property_with_flags(
                    object_local,
                    key_local,
                    Some((getter_payload_local, getter_tag_local)),
                    None,
                    false,
                    true,
                    function,
                )?;

                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(object_local));
                for (name, builtin) in [(
                    "byteLength",
                    if matches!(builtin, StandardBuiltinId::SharedArrayBufferConstructor) {
                        StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
                    } else {
                        StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
                    },
                )] {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(getter_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(getter_tag_local));
                    self.emit_object_append_accessor_property_with_flags(
                        object_local,
                        key_local,
                        Some((getter_payload_local, getter_tag_local)),
                        None,
                        false,
                        true,
                        function,
                    )?;
                }
                if matches!(builtin, StandardBuiltinId::SharedArrayBufferConstructor) {
                    let grow_meta = self
                        .functions
                        .get(&StandardBuiltinId::SharedArrayBufferPrototypeGrow.function_id())
                        .ok_or_else(|| {
                            EmitError::unsupported(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `SharedArrayBuffer.prototype.grow`",
                            )
                        })?;
                    self.emit_object_define_function_data(
                        object_local,
                        "grow",
                        grow_meta,
                        function,
                    )?;
                    for (name, builtin) in [
                        (
                            "maxByteLength",
                            StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter,
                        ),
                        (
                            "growable",
                            StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter,
                        ),
                    ] {
                        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                                builtin.debug_name()
                            ))
                        })?;
                        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                        function.instruction(&Instruction::LocalSet(key_local));
                        self.emit_function_value_payload(meta, function)?;
                        function.instruction(&Instruction::LocalSet(getter_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                        function.instruction(&Instruction::LocalSet(getter_tag_local));
                        self.emit_object_append_accessor_property_with_flags(
                            object_local,
                            key_local,
                            Some((getter_payload_local, getter_tag_local)),
                            None,
                            false,
                            true,
                            function,
                        )?;
                    }
                } else {
                    for (name, builtin) in [
                        (
                            "detached",
                            StandardBuiltinId::ArrayBufferPrototypeDetachedGetter,
                        ),
                        (
                            "maxByteLength",
                            StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter,
                        ),
                        (
                            "resizable",
                            StandardBuiltinId::ArrayBufferPrototypeResizableGetter,
                        ),
                    ] {
                        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                                builtin.debug_name()
                            ))
                        })?;
                        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                        function.instruction(&Instruction::LocalSet(key_local));
                        self.emit_function_value_payload(meta, function)?;
                        function.instruction(&Instruction::LocalSet(getter_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                        function.instruction(&Instruction::LocalSet(getter_tag_local));
                        self.emit_object_append_accessor_property_with_flags(
                            object_local,
                            key_local,
                            Some((getter_payload_local, getter_tag_local)),
                            None,
                            false,
                            true,
                            function,
                        )?;
                    }
                }
                let slice_builtin =
                    if matches!(builtin, StandardBuiltinId::SharedArrayBufferConstructor) {
                        StandardBuiltinId::SharedArrayBufferPrototypeSlice
                    } else {
                        StandardBuiltinId::ArrayBufferPrototypeSlice
                    };
                let slice_meta = self
                    .functions
                    .get(&slice_builtin.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `ArrayBuffer.prototype.slice`",
                        )
                    })?;
                self.emit_object_define_function_data(object_local, "slice", slice_meta, function)?;
                let resize_meta = self
                    .functions
                    .get(&StandardBuiltinId::ArrayBufferPrototypeResize.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `ArrayBuffer.prototype.resize`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "resize",
                    resize_meta,
                    function,
                )?;
                for (name, builtin) in [
                    ("transfer", StandardBuiltinId::ArrayBufferPrototypeTransfer),
                    (
                        "transferToFixedLength",
                        StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength,
                    ),
                    (
                        "transferToImmutable",
                        StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable,
                    ),
                    (
                        "sliceToImmutable",
                        StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable,
                    ),
                ] {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_object_define_function_data(object_local, name, meta, function)?;
                }
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.toStringTag"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::I64Const(self.strings.payload(
                    if matches!(builtin, StandardBuiltinId::SharedArrayBufferConstructor) {
                        "SharedArrayBuffer"
                    } else {
                        "ArrayBuffer"
                    },
                )));
                function.instruction(&Instruction::LocalSet(getter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(getter_tag_local));
                self.emit_object_append_data_property_with_flags(
                    object_local,
                    key_local,
                    getter_payload_local,
                    getter_tag_local,
                    false,
                    false,
                    true,
                    function,
                )?;
                self.release_temp_local(getter_tag_local);
                self.release_temp_local(getter_payload_local);
                self.release_temp_local(key_local);
            }
            StandardBuiltinId::DataViewConstructor => {
                let prototype_object_local = self.reserve_temp_local();
                let key_local = self.reserve_temp_local();
                let getter_payload_local = self.reserve_temp_local();
                let getter_tag_local = self.reserve_temp_local();
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                for (name, builtin) in [
                    ("buffer", StandardBuiltinId::DataViewPrototypeBufferGetter),
                    (
                        "byteLength",
                        StandardBuiltinId::DataViewPrototypeByteLengthGetter,
                    ),
                    (
                        "byteOffset",
                        StandardBuiltinId::DataViewPrototypeByteOffsetGetter,
                    ),
                ] {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(getter_payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(getter_tag_local));
                    self.emit_object_append_accessor_property_with_flags(
                        prototype_object_local,
                        key_local,
                        Some((getter_payload_local, getter_tag_local)),
                        None,
                        false,
                        true,
                        function,
                    )?;
                }
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                for (name, builtin) in [
                    ("getUint8", StandardBuiltinId::DataViewPrototypeGetUint8),
                    ("setUint8", StandardBuiltinId::DataViewPrototypeSetUint8),
                    ("getInt8", StandardBuiltinId::DataViewPrototypeGetInt8),
                    ("setInt8", StandardBuiltinId::DataViewPrototypeSetInt8),
                    ("getUint16", StandardBuiltinId::DataViewPrototypeGetUint16),
                    ("setUint16", StandardBuiltinId::DataViewPrototypeSetUint16),
                    ("getInt16", StandardBuiltinId::DataViewPrototypeGetInt16),
                    ("setInt16", StandardBuiltinId::DataViewPrototypeSetInt16),
                    ("getUint32", StandardBuiltinId::DataViewPrototypeGetUint32),
                    ("setUint32", StandardBuiltinId::DataViewPrototypeSetUint32),
                    ("getInt32", StandardBuiltinId::DataViewPrototypeGetInt32),
                    ("setInt32", StandardBuiltinId::DataViewPrototypeSetInt32),
                    ("getFloat16", StandardBuiltinId::DataViewPrototypeGetFloat16),
                    ("setFloat16", StandardBuiltinId::DataViewPrototypeSetFloat16),
                    ("getFloat32", StandardBuiltinId::DataViewPrototypeGetFloat32),
                    ("setFloat32", StandardBuiltinId::DataViewPrototypeSetFloat32),
                    ("getFloat64", StandardBuiltinId::DataViewPrototypeGetFloat64),
                    ("setFloat64", StandardBuiltinId::DataViewPrototypeSetFloat64),
                    (
                        "getBigInt64",
                        StandardBuiltinId::DataViewPrototypeGetBigInt64,
                    ),
                    (
                        "setBigInt64",
                        StandardBuiltinId::DataViewPrototypeSetBigInt64,
                    ),
                    (
                        "getBigUint64",
                        StandardBuiltinId::DataViewPrototypeGetBigUint64,
                    ),
                    (
                        "setBigUint64",
                        StandardBuiltinId::DataViewPrototypeSetBigUint64,
                    ),
                ] {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_object_define_function_data(
                        prototype_object_local,
                        name,
                        meta,
                        function,
                    )?;
                }
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.toStringTag"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::I64Const(self.strings.payload("DataView")));
                function.instruction(&Instruction::LocalSet(getter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(getter_tag_local));
                self.emit_object_append_data_property_with_flags(
                    prototype_object_local,
                    key_local,
                    getter_payload_local,
                    getter_tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
                self.release_temp_local(getter_tag_local);
                self.release_temp_local(getter_payload_local);
                self.release_temp_local(key_local);
                self.release_temp_local(prototype_object_local);
            }
            StandardBuiltinId::DateConstructor => {
                let now_meta = self.functions.get(&StandardBuiltinId::DateNow.function_id()).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `Date.now`",
                    )
                })?;
                self.emit_object_define_function_data(object_local, "now", now_meta, function)?;
                let date_utc_meta = self.functions.get(&StandardBuiltinId::DateUtc.function_id()).ok_or_else(|| {
                    EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `Date.UTC`",
                    )
                })?;
                self.emit_object_define_function_data(
                    object_local,
                    "UTC",
                    date_utc_meta,
                    function,
                )?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(object_local));
                let utc_meta = self
                    .functions
                    .get(&StandardBuiltinId::DatePrototypeToUtcString.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Date.prototype.toUTCString`",
                        )
                    })?;
                for (name, builtin) in [
                    ("getTime", StandardBuiltinId::DatePrototypeGetTime),
                    ("setTime", StandardBuiltinId::DatePrototypeSetTime),
                    ("valueOf", StandardBuiltinId::DatePrototypeValueOf),
                    ("getFullYear", StandardBuiltinId::DatePrototypeGetFullYear),
                    (
                        "getUTCFullYear",
                        StandardBuiltinId::DatePrototypeGetUtcFullYear,
                    ),
                    ("getMonth", StandardBuiltinId::DatePrototypeGetMonth),
                    ("getUTCMonth", StandardBuiltinId::DatePrototypeGetUtcMonth),
                    ("getDate", StandardBuiltinId::DatePrototypeGetDate),
                    ("getUTCDate", StandardBuiltinId::DatePrototypeGetUtcDate),
                    ("getDay", StandardBuiltinId::DatePrototypeGetDay),
                    ("getUTCDay", StandardBuiltinId::DatePrototypeGetUtcDay),
                    ("getHours", StandardBuiltinId::DatePrototypeGetHours),
                    ("getUTCHours", StandardBuiltinId::DatePrototypeGetUtcHours),
                    ("getMinutes", StandardBuiltinId::DatePrototypeGetMinutes),
                    (
                        "getUTCMinutes",
                        StandardBuiltinId::DatePrototypeGetUtcMinutes,
                    ),
                    ("getSeconds", StandardBuiltinId::DatePrototypeGetSeconds),
                    (
                        "getUTCSeconds",
                        StandardBuiltinId::DatePrototypeGetUtcSeconds,
                    ),
                    (
                        "getMilliseconds",
                        StandardBuiltinId::DatePrototypeGetMilliseconds,
                    ),
                    (
                        "getUTCMilliseconds",
                        StandardBuiltinId::DatePrototypeGetUtcMilliseconds,
                    ),
                    (
                        "getTimezoneOffset",
                        StandardBuiltinId::DatePrototypeGetTimezoneOffset,
                    ),
                    ("getYear", StandardBuiltinId::DatePrototypeGetYear),
                    ("setYear", StandardBuiltinId::DatePrototypeSetYear),
                    ("setFullYear", StandardBuiltinId::DatePrototypeSetFullYear),
                    (
                        "setUTCFullYear",
                        StandardBuiltinId::DatePrototypeSetUtcFullYear,
                    ),
                    ("setMonth", StandardBuiltinId::DatePrototypeSetMonth),
                    ("setUTCMonth", StandardBuiltinId::DatePrototypeSetUtcMonth),
                    ("setDate", StandardBuiltinId::DatePrototypeSetDate),
                    ("setUTCDate", StandardBuiltinId::DatePrototypeSetUtcDate),
                    ("setHours", StandardBuiltinId::DatePrototypeSetHours),
                    ("setUTCHours", StandardBuiltinId::DatePrototypeSetUtcHours),
                    ("setMinutes", StandardBuiltinId::DatePrototypeSetMinutes),
                    (
                        "setUTCMinutes",
                        StandardBuiltinId::DatePrototypeSetUtcMinutes,
                    ),
                    ("setSeconds", StandardBuiltinId::DatePrototypeSetSeconds),
                    (
                        "setUTCSeconds",
                        StandardBuiltinId::DatePrototypeSetUtcSeconds,
                    ),
                    (
                        "setMilliseconds",
                        StandardBuiltinId::DatePrototypeSetMilliseconds,
                    ),
                    (
                        "setUTCMilliseconds",
                        StandardBuiltinId::DatePrototypeSetUtcMilliseconds,
                    ),
                ] {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_object_define_function_data(object_local, name, meta, function)?;
                }
                let utc_payload_local = self.reserve_temp_local();
                let utc_tag_local = self.reserve_temp_local();
                self.emit_function_value_payload(utc_meta, function)?;
                function.instruction(&Instruction::LocalSet(utc_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(utc_tag_local));
                self.emit_object_append_local_data_property_with_flags(
                    object_local,
                    "toUTCString",
                    utc_payload_local,
                    utc_tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
                self.emit_object_append_local_data_property_with_flags(
                    object_local,
                    "toGMTString",
                    utc_payload_local,
                    utc_tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
                self.release_temp_local(utc_tag_local);
                self.release_temp_local(utc_payload_local);
            }
            StandardBuiltinId::ErrorConstructor => {
                let prototype_object_local = self.reserve_temp_local();
                let to_string_meta = self
                    .functions
                    .get(&StandardBuiltinId::ErrorPrototypeToString.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Error.prototype.toString`",
                        )
                    })?;
                function.instruction(&Instruction::GlobalGet(prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                self.emit_object_define_function_data(
                    prototype_object_local,
                    "toString",
                    to_string_meta,
                    function,
                )?;
                let is_error_meta = self
                    .functions
                    .get(&StandardBuiltinId::ErrorIsError.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Error.isError`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isError",
                    is_error_meta,
                    function,
                )?;
                self.release_temp_local(prototype_object_local);
            }
            StandardBuiltinId::BigIntConstructor => {
                self.emit_alloc_plain_object_with_prototype(
                    None,
                    Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                self.emit_store_realm_intrinsic_prototype(
                    self.scratch_local,
                    HEAP_REALM_INTRINSICS_BIGINT_PROTOTYPE_OFFSET,
                    prototype_object_local,
                    function,
                );
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
                function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::LocalGet(prototype_object_local));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
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
                function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_define_data(
                    prototype_object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    function,
                )?;
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.toStringTag"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::I64Const(self.strings.payload("BigInt")));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_append_data_property_with_flags(
                    prototype_object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    false,
                    false,
                    true,
                    function,
                )?;
                for (name, builtin) in [
                    ("toString", StandardBuiltinId::BigIntPrototypeToString),
                    (
                        "toLocaleString",
                        StandardBuiltinId::BigIntPrototypeToLocaleString,
                    ),
                    ("valueOf", StandardBuiltinId::BigIntPrototypeValueOf),
                ] {
                    if let Some(meta) = self.functions.get(&builtin.function_id()) {
                        self.emit_object_define_function_data(
                            prototype_object_local,
                            name,
                            meta,
                            function,
                        )?;
                    }
                }
                for (name, builtin) in [
                    ("asIntN", StandardBuiltinId::BigIntAsIntN),
                    ("asUintN", StandardBuiltinId::BigIntAsUintN),
                ] {
                    if let Some(meta) = self.functions.get(&builtin.function_id()) {
                        self.emit_object_define_function_data(object_local, name, meta, function)?;
                    }
                }
            }
            StandardBuiltinId::SymbolConstructor => {
                // `Symbol` has a [[Construct]] internal method per spec (it
                // may appear as an `extends` target; a `super()` call into
                // it throws) even though `new Symbol()` always throws, so
                // `constructable()` is true and the generic prototype/
                // constructor wiring above already defined the
                // non-writable/non-enumerable/non-configurable `prototype`
                // data property and the writable/non-enumerable/
                // configurable `Symbol.prototype.constructor` back-reference
                // (see the `else` branch gated on `builtin.constructable()`
                // near the top of this function). Fetch the prototype object
                // and continue with the well-known symbols, the global
                // registry, and `Symbol.prototype`'s remaining members.
                function.instruction(&Instruction::GlobalGet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));

                // Symbol.prototype[Symbol.toStringTag] === "Symbol"
                // (non-writable, non-enumerable, configurable).
                function.instruction(&Instruction::I64Const(
                    self.strings.payload("Symbol.toStringTag"),
                ));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::I64Const(self.strings.payload("Symbol")));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_object_append_data_property_with_flags(
                    prototype_object_local,
                    key_local,
                    payload_local,
                    tag_local,
                    false,
                    false,
                    true,
                    function,
                )?;

                for (key, value) in [
                    ("iterator", "Symbol.iterator"),
                    ("asyncIterator", "Symbol.asyncIterator"),
                    ("hasInstance", "Symbol.hasInstance"),
                    ("isConcatSpreadable", "Symbol.isConcatSpreadable"),
                    ("match", "Symbol.match"),
                    ("matchAll", "Symbol.matchAll"),
                    ("replace", "Symbol.replace"),
                    ("search", "Symbol.search"),
                    ("species", "Symbol.species"),
                    ("split", "Symbol.split"),
                    ("toPrimitive", "Symbol.toPrimitive"),
                    ("toStringTag", "Symbol.toStringTag"),
                    ("unscopables", "Symbol.unscopables"),
                    ("dispose", "Symbol.dispose"),
                    ("asyncDispose", "Symbol.asyncDispose"),
                ] {
                    function.instruction(&Instruction::I64Const(self.strings.payload(key)));
                    function.instruction(&Instruction::LocalSet(key_local));
                    function.instruction(&Instruction::I64Const(self.strings.payload(value)));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.emit_object_append_data_property_with_flags(
                        object_local,
                        key_local,
                        payload_local,
                        tag_local,
                        false,
                        false,
                        false,
                        function,
                    )?;
                }

                // Global symbol registry backing `Symbol.for` / `Symbol.keyFor`:
                // a null-prototype ordinary object mapping description strings to
                // the canonical registered symbol values.
                self.emit_alloc_plain_object_with_prototype(None, None, function)?;
                function.instruction(&Instruction::GlobalSet(SYMBOL_REGISTRY_GLOBAL_INDEX));

                if let Some(for_meta) = self
                    .functions
                    .get(&StandardBuiltinId::SymbolFor.function_id())
                    .cloned()
                {
                    self.emit_object_define_function_data(
                        object_local,
                        "for",
                        &for_meta,
                        function,
                    )?;
                }
                if let Some(key_for_meta) = self
                    .functions
                    .get(&StandardBuiltinId::SymbolKeyFor.function_id())
                    .cloned()
                {
                    self.emit_object_define_function_data(
                        object_local,
                        "keyFor",
                        &key_for_meta,
                        function,
                    )?;
                }

                // Symbol.prototype.toString / Symbol.prototype.valueOf:
                // ordinary writable/non-enumerable/configurable methods.
                function.instruction(&Instruction::GlobalGet(SYMBOL_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                if let Some(to_string_meta) = self
                    .functions
                    .get(&StandardBuiltinId::SymbolPrototypeToString.function_id())
                    .cloned()
                {
                    self.emit_object_define_function_data(
                        prototype_object_local,
                        "toString",
                        &to_string_meta,
                        function,
                    )?;
                }
                if let Some(value_of_meta) = self
                    .functions
                    .get(&StandardBuiltinId::SymbolPrototypeValueOf.function_id())
                    .cloned()
                {
                    self.emit_object_define_function_data(
                        prototype_object_local,
                        "valueOf",
                        &value_of_meta,
                        function,
                    )?;
                }

                // Symbol.prototype.description: accessor (getter-only,
                // non-enumerable, configurable) — Object.getOwnPropertyDescriptor
                // must see a real getter function.
                if let Some(description_getter_meta) = self
                    .functions
                    .get(&StandardBuiltinId::SymbolPrototypeDescriptionGetter.function_id())
                    .cloned()
                {
                    function
                        .instruction(&Instruction::I64Const(self.strings.payload("description")));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_function_value_payload(&description_getter_meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.emit_object_append_accessor_property_with_flags(
                        prototype_object_local,
                        key_local,
                        Some((payload_local, tag_local)),
                        None,
                        false,
                        true,
                        function,
                    )?;
                }

                // Symbol.prototype[Symbol.toPrimitive]: non-writable,
                // non-enumerable, configurable data property.
                if let Some(to_primitive_meta) = self
                    .functions
                    .get(&StandardBuiltinId::SymbolPrototypeToPrimitive.function_id())
                    .cloned()
                {
                    function.instruction(&Instruction::I64Const(
                        self.strings.payload("Symbol.toPrimitive"),
                    ));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.emit_function_value_payload(&to_primitive_meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.emit_object_append_data_property_with_flags(
                        prototype_object_local,
                        key_local,
                        payload_local,
                        tag_local,
                        false,
                        false,
                        true,
                        function,
                    )?;
                }
            }
            StandardBuiltinId::NumberConstructor => {
                function.instruction(&Instruction::GlobalGet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
                function.instruction(&Instruction::I64ReinterpretF64);
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_store_boxed_primitive_metadata(
                    prototype_object_local,
                    BOXED_PRIMITIVE_KIND_NUMBER,
                    payload_local,
                    tag_local,
                    function,
                );
                for (name, value) in [
                    ("NaN", f64::NAN),
                    ("POSITIVE_INFINITY", f64::INFINITY),
                    ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
                    ("MAX_VALUE", f64::MAX),
                    ("MIN_VALUE", f64::from_bits(1)),
                    ("EPSILON", f64::EPSILON),
                    ("MAX_SAFE_INTEGER", 9007199254740991.0),
                    ("MIN_SAFE_INTEGER", -9007199254740991.0),
                ] {
                    function.instruction(&Instruction::I64Const(self.strings.payload(name)));
                    function.instruction(&Instruction::LocalSet(key_local));
                    function.instruction(&Instruction::F64Const(Ieee64::from(value)));
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                    self.emit_object_append_data_property_with_flags(
                        object_local,
                        key_local,
                        payload_local,
                        tag_local,
                        false,
                        false,
                        false,
                        function,
                    )?;
                }
                let is_integer_meta = self
                    .functions
                    .get(&StandardBuiltinId::NumberIsInteger.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.isInteger`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isInteger",
                    is_integer_meta,
                    function,
                )?;
                let is_safe_integer_meta = self
                    .functions
                    .get(&StandardBuiltinId::NumberIsSafeInteger.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.isSafeInteger`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isSafeInteger",
                    is_safe_integer_meta,
                    function,
                )?;
                let is_finite_meta = self
                    .functions
                    .get(&StandardBuiltinId::NumberIsFinite.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.isFinite`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isFinite",
                    is_finite_meta,
                    function,
                )?;
                let is_nan_meta = self
                    .functions
                    .get(&StandardBuiltinId::NumberIsNaN.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.isNaN`",
                        )
                    })?;
                self.emit_object_define_function_data(
                    object_local,
                    "isNaN",
                    is_nan_meta,
                    function,
                )?;
                let parse_int_meta = self
                    .functions
                    .get(&HostBuiltinId::ParseInt.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.parseInt`",
                        )
                    })?;
                self.emit_ensure_canonical_host_function(
                    &parse_int_meta,
                    PARSE_INT_FUNCTION_GLOBAL_INDEX,
                    function,
                )?;
                self.emit_object_define_function_global_data(
                    object_local,
                    "parseInt",
                    PARSE_INT_FUNCTION_GLOBAL_INDEX,
                    function,
                )?;
                let parse_float_meta = self
                    .functions
                    .get(&HostBuiltinId::ParseFloat.function_id())
                    .cloned()
                    .ok_or_else(|| {
                        EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `Number.parseFloat`",
                        )
                    })?;
                self.emit_ensure_canonical_host_function(
                    &parse_float_meta,
                    PARSE_FLOAT_FUNCTION_GLOBAL_INDEX,
                    function,
                )?;
                self.emit_object_define_function_global_data(
                    object_local,
                    "parseFloat",
                    PARSE_FLOAT_FUNCTION_GLOBAL_INDEX,
                    function,
                )?;
                function.instruction(&Instruction::GlobalGet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                for (name, builtin) in [
                    ("toFixed", StandardBuiltinId::NumberPrototypeToFixed),
                    (
                        "toExponential",
                        StandardBuiltinId::NumberPrototypeToExponential,
                    ),
                    ("toPrecision", StandardBuiltinId::NumberPrototypeToPrecision),
                    ("toString", StandardBuiltinId::NumberPrototypeToString),
                    (
                        "toLocaleString",
                        StandardBuiltinId::NumberPrototypeToLocaleString,
                    ),
                    ("valueOf", StandardBuiltinId::NumberPrototypeValueOf),
                ] {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_object_define_function_data(
                        prototype_object_local,
                        name,
                        meta,
                        function,
                    )?;
                }
            }
            StandardBuiltinId::BooleanConstructor => {
                function.instruction(&Instruction::GlobalGet(BOOLEAN_PROTOTYPE_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalSet(prototype_object_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                self.emit_store_boxed_primitive_metadata(
                    prototype_object_local,
                    BOXED_PRIMITIVE_KIND_BOOLEAN,
                    payload_local,
                    tag_local,
                    function,
                );
                for (name, builtin) in [
                    ("toString", StandardBuiltinId::BooleanPrototypeToString),
                    ("valueOf", StandardBuiltinId::BooleanPrototypeValueOf),
                ] {
                    let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                            builtin.debug_name()
                        ))
                    })?;
                    self.emit_object_define_function_data(
                        prototype_object_local,
                        name,
                        meta,
                        function,
                    )?;
                }
            }
            StandardBuiltinId::BigIntAsIntN
            | StandardBuiltinId::BigIntAsUintN
            | StandardBuiltinId::BigIntPrototypeToString
            | StandardBuiltinId::BigIntPrototypeToLocaleString
            | StandardBuiltinId::BigIntPrototypeValueOf
            | StandardBuiltinId::Float64ArrayConstructor
            | StandardBuiltinId::Float32ArrayConstructor
            | StandardBuiltinId::Int32ArrayConstructor
            | StandardBuiltinId::Int16ArrayConstructor
            | StandardBuiltinId::Int8ArrayConstructor
            | StandardBuiltinId::Uint32ArrayConstructor
            | StandardBuiltinId::Uint16ArrayConstructor
            | StandardBuiltinId::Uint8ArrayConstructor
            | StandardBuiltinId::Uint8ClampedArrayConstructor
            | StandardBuiltinId::BigInt64ArrayConstructor
            | StandardBuiltinId::BigUint64ArrayConstructor
            | StandardBuiltinId::ArrayBufferSpeciesGetter
            | StandardBuiltinId::RegExpSpeciesGetter
            | StandardBuiltinId::EvalErrorConstructor
            | StandardBuiltinId::AggregateErrorConstructor
            | StandardBuiltinId::SuppressedErrorConstructor
            | StandardBuiltinId::RangeErrorConstructor
            | StandardBuiltinId::SyntaxErrorConstructor
            | StandardBuiltinId::TypeErrorConstructor
            | StandardBuiltinId::URIErrorConstructor
            | StandardBuiltinId::ReferenceErrorConstructor
            | StandardBuiltinId::ErrorIsError
            | StandardBuiltinId::FunctionPrototypeCall
            | StandardBuiltinId::FunctionPrototypeApply
            | StandardBuiltinId::FunctionPrototypeBind
            | StandardBuiltinId::FunctionPrototypeToString
            | StandardBuiltinId::StringPrototypeToString
            | StandardBuiltinId::StringPrototypeValueOf
            | StandardBuiltinId::StringPrototypeCharAt
            | StandardBuiltinId::StringPrototypeCharCodeAt
            | StandardBuiltinId::StringPrototypeCodePointAt
            | StandardBuiltinId::StringPrototypeAt
            | StandardBuiltinId::ObjectCreate
            | StandardBuiltinId::ObjectGetPrototypeOf
            | StandardBuiltinId::ObjectSetPrototypeOf
            | StandardBuiltinId::ObjectDefineProperty
            | StandardBuiltinId::ObjectDefineProperties
            | StandardBuiltinId::ObjectGetOwnPropertyDescriptor
            | StandardBuiltinId::ObjectGetOwnPropertyNames
            | StandardBuiltinId::ObjectGetOwnPropertySymbols
            | StandardBuiltinId::ObjectKeys
            | StandardBuiltinId::ObjectValues
            | StandardBuiltinId::ObjectHasOwn
            | StandardBuiltinId::ObjectIs
            | StandardBuiltinId::ObjectIsSealed
            | StandardBuiltinId::ObjectIsFrozen
            | StandardBuiltinId::ObjectFreeze
            | StandardBuiltinId::ObjectIsExtensible
            | StandardBuiltinId::ObjectPreventExtensions
            | StandardBuiltinId::ObjectPrototypeHasOwnProperty
            | StandardBuiltinId::ObjectPrototypePropertyIsEnumerable
            | StandardBuiltinId::ObjectPrototypeIsPrototypeOf
            | StandardBuiltinId::ObjectPrototypeToString
            | StandardBuiltinId::ObjectPrototypeToLocaleString
            | StandardBuiltinId::ObjectPrototypeValueOf
            | StandardBuiltinId::ProxyRevocable
            | StandardBuiltinId::ProxyRevoke
            | StandardBuiltinId::ReflectConstruct
            | StandardBuiltinId::ReflectApply
            | StandardBuiltinId::ReflectGet
            | StandardBuiltinId::ReflectGetPrototypeOf
            | StandardBuiltinId::ReflectGetOwnPropertyDescriptor
            | StandardBuiltinId::ReflectSet
            | StandardBuiltinId::ReflectHas
            | StandardBuiltinId::ReflectDefineProperty
            | StandardBuiltinId::ReflectDeleteProperty
            | StandardBuiltinId::ReflectIsExtensible
            | StandardBuiltinId::ReflectPreventExtensions
            | StandardBuiltinId::ReflectSetPrototypeOf
            | StandardBuiltinId::ReflectOwnKeys
            | StandardBuiltinId::ArrayFrom
            | StandardBuiltinId::ArrayIsArray
            | StandardBuiltinId::NumberIsInteger
            | StandardBuiltinId::NumberIsSafeInteger
            | StandardBuiltinId::NumberIsFinite
            | StandardBuiltinId::NumberIsNaN
            | StandardBuiltinId::NumberPrototypeToExponential
            | StandardBuiltinId::NumberPrototypeToFixed
            | StandardBuiltinId::NumberPrototypeToPrecision
            | StandardBuiltinId::NumberPrototypeToString
            | StandardBuiltinId::NumberPrototypeToLocaleString
            | StandardBuiltinId::NumberPrototypeValueOf
            | StandardBuiltinId::BooleanPrototypeToString
            | StandardBuiltinId::BooleanPrototypeValueOf
            | StandardBuiltinId::GlobalIsFinite
            | StandardBuiltinId::GlobalIsNaN
            | StandardBuiltinId::MathAbs
            | StandardBuiltinId::MathAcos
            | StandardBuiltinId::MathAcosh
            | StandardBuiltinId::MathAsin
            | StandardBuiltinId::MathAsinh
            | StandardBuiltinId::MathAtan
            | StandardBuiltinId::MathAtan2
            | StandardBuiltinId::MathAtanh
            | StandardBuiltinId::MathCbrt
            | StandardBuiltinId::MathCeil
            | StandardBuiltinId::MathClz32
            | StandardBuiltinId::MathCos
            | StandardBuiltinId::MathCosh
            | StandardBuiltinId::MathExp
            | StandardBuiltinId::MathExpm1
            | StandardBuiltinId::MathF16Round
            | StandardBuiltinId::MathFloor
            | StandardBuiltinId::MathFround
            | StandardBuiltinId::MathHypot
            | StandardBuiltinId::MathImul
            | StandardBuiltinId::MathLog
            | StandardBuiltinId::MathLog10
            | StandardBuiltinId::MathLog1p
            | StandardBuiltinId::MathLog2
            | StandardBuiltinId::MathPow
            | StandardBuiltinId::MathRandom
            | StandardBuiltinId::MathRound
            | StandardBuiltinId::MathSign
            | StandardBuiltinId::MathSin
            | StandardBuiltinId::MathSinh
            | StandardBuiltinId::MathSqrt
            | StandardBuiltinId::MathSumPrecise
            | StandardBuiltinId::MathTan
            | StandardBuiltinId::MathTanh
            | StandardBuiltinId::MathTrunc
            | StandardBuiltinId::MathMin
            | StandardBuiltinId::MathMax
            | StandardBuiltinId::ArrayPrototypeConcat
            | StandardBuiltinId::ArrayPrototypeJoin
            | StandardBuiltinId::ArrayPrototypeSplice
            | StandardBuiltinId::ArrayPrototypeToLocaleString
            | StandardBuiltinId::ArrayPrototypeFlat
            | StandardBuiltinId::ArrayPrototypeFlatMap
            | StandardBuiltinId::ArrayPrototypeAt
            | StandardBuiltinId::ArrayPrototypeIncludes
            | StandardBuiltinId::ArrayPrototypeIndexOf
            | StandardBuiltinId::ArrayPrototypeLastIndexOf
            | StandardBuiltinId::ArrayPrototypeFind
            | StandardBuiltinId::ArrayPrototypeFindIndex
            | StandardBuiltinId::ArrayPrototypeFindLast
            | StandardBuiltinId::ArrayPrototypeFindLastIndex
            | StandardBuiltinId::ArrayPrototypeEvery
            | StandardBuiltinId::ArrayPrototypeSome
            | StandardBuiltinId::ArrayPrototypeForEach
            | StandardBuiltinId::ArrayPrototypeFilter
            | StandardBuiltinId::ArrayPrototypeMap
            | StandardBuiltinId::ArrayPrototypeReduce
            | StandardBuiltinId::ArrayPrototypeReduceRight
            | StandardBuiltinId::ArrayPrototypePop
            | StandardBuiltinId::ArrayPrototypePush
            | StandardBuiltinId::ArrayPrototypeKeys
            | StandardBuiltinId::ArrayPrototypeEntries
            | StandardBuiltinId::ArrayPrototypeValues
            | StandardBuiltinId::ArrayIteratorNext
            | StandardBuiltinId::ArrayIteratorIdentity
            | StandardBuiltinId::IteratorFrom
            | StandardBuiltinId::IteratorPrototypeToArray
            | StandardBuiltinId::IteratorPrototypeForEach
            | StandardBuiltinId::IteratorPrototypeEvery
            | StandardBuiltinId::IteratorPrototypeSome
            | StandardBuiltinId::IteratorPrototypeFind
            | StandardBuiltinId::IteratorPrototypeReduce
            | StandardBuiltinId::IteratorPrototypeMap
            | StandardBuiltinId::IteratorMapNext
            | StandardBuiltinId::IteratorMapReturn
            | StandardBuiltinId::IteratorPrototypeFilter
            | StandardBuiltinId::IteratorFilterNext
            | StandardBuiltinId::IteratorFilterReturn
            | StandardBuiltinId::IteratorPrototypeFlatMap
            | StandardBuiltinId::IteratorFlatMapNext
            | StandardBuiltinId::IteratorFlatMapReturn
            | StandardBuiltinId::IteratorPrototypeTake
            | StandardBuiltinId::IteratorTakeNext
            | StandardBuiltinId::IteratorTakeReturn
            | StandardBuiltinId::IteratorPrototypeDrop
            | StandardBuiltinId::IteratorDropNext
            | StandardBuiltinId::IteratorDropReturn
            | StandardBuiltinId::IteratorPrototypeConstructorGetter
            | StandardBuiltinId::IteratorPrototypeConstructorSetter
            | StandardBuiltinId::IteratorPrototypeSymbolDispose
            | StandardBuiltinId::IteratorPrototypeToStringTagGetter
            | StandardBuiltinId::IteratorPrototypeToStringTagSetter
            | StandardBuiltinId::IteratorFromWrapperNext
            | StandardBuiltinId::IteratorFromWrapperReturn
            | StandardBuiltinId::ArrayBufferIsView
            | StandardBuiltinId::ArrayBufferPrototypeDetachedGetter
            | StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter
            | StandardBuiltinId::ArrayBufferPrototypeResizableGetter
            | StandardBuiltinId::ArrayBufferPrototypeResize
            | StandardBuiltinId::ArrayBufferPrototypeSlice
            | StandardBuiltinId::SharedArrayBufferPrototypeSlice
            | StandardBuiltinId::ArrayBufferPrototypeTransfer
            | StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength
            | StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable
            | StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable
            | StandardBuiltinId::DataViewPrototypeGetUint8
            | StandardBuiltinId::DataViewPrototypeSetUint8
            | StandardBuiltinId::DataViewPrototypeGetInt8
            | StandardBuiltinId::DataViewPrototypeSetInt8
            | StandardBuiltinId::DataViewPrototypeGetUint16
            | StandardBuiltinId::DataViewPrototypeSetUint16
            | StandardBuiltinId::DataViewPrototypeGetInt16
            | StandardBuiltinId::DataViewPrototypeSetInt16
            | StandardBuiltinId::DataViewPrototypeGetUint32
            | StandardBuiltinId::DataViewPrototypeSetUint32
            | StandardBuiltinId::DataViewPrototypeGetInt32
            | StandardBuiltinId::DataViewPrototypeSetInt32
            | StandardBuiltinId::DataViewPrototypeGetFloat16
            | StandardBuiltinId::DataViewPrototypeSetFloat16
            | StandardBuiltinId::DataViewPrototypeGetFloat32
            | StandardBuiltinId::DataViewPrototypeSetFloat32
            | StandardBuiltinId::DataViewPrototypeGetFloat64
            | StandardBuiltinId::DataViewPrototypeSetFloat64
            | StandardBuiltinId::DataViewPrototypeGetBigInt64
            | StandardBuiltinId::DataViewPrototypeSetBigInt64
            | StandardBuiltinId::DataViewPrototypeGetBigUint64
            | StandardBuiltinId::DataViewPrototypeSetBigUint64
            | StandardBuiltinId::DataViewPrototypeBufferGetter
            | StandardBuiltinId::DataViewPrototypeByteLengthGetter
            | StandardBuiltinId::DataViewPrototypeByteOffsetGetter
            | StandardBuiltinId::TypedArrayPrototypeBufferGetter
            | StandardBuiltinId::TypedArrayPrototypeByteLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter
            | StandardBuiltinId::TypedArrayPrototypeLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeToString
            | StandardBuiltinId::TypedArrayPrototypeToLocaleString
            | StandardBuiltinId::TypedArrayFrom
            | StandardBuiltinId::TypedArrayOf
            | StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeGrow
            | StandardBuiltinId::ArraySpeciesGetter
            | StandardBuiltinId::StringPrototypeAnchor
            | StandardBuiltinId::StringPrototypeBig
            | StandardBuiltinId::StringPrototypeBlink
            | StandardBuiltinId::StringPrototypeBold
            | StandardBuiltinId::StringPrototypeFixed
            | StandardBuiltinId::StringPrototypeFontcolor
            | StandardBuiltinId::StringPrototypeFontsize
            | StandardBuiltinId::StringPrototypeItalics
            | StandardBuiltinId::StringPrototypeLink
            | StandardBuiltinId::StringPrototypeSmall
            | StandardBuiltinId::StringPrototypeStrike
            | StandardBuiltinId::StringPrototypeSub
            | StandardBuiltinId::StringPrototypeSubstr
            | StandardBuiltinId::StringPrototypeSubstring
            | StandardBuiltinId::StringPrototypeSup
            | StandardBuiltinId::StringPrototypeMatch
            | StandardBuiltinId::StringPrototypeMatchAll
            | StandardBuiltinId::StringPrototypeReplace
            | StandardBuiltinId::StringPrototypeReplaceAll
            | StandardBuiltinId::StringPrototypeSearch
            | StandardBuiltinId::StringPrototypeIndexOf
            | StandardBuiltinId::StringPrototypeLastIndexOf
            | StandardBuiltinId::StringPrototypeSlice
            | StandardBuiltinId::StringPrototypeSplit
            | StandardBuiltinId::StringPrototypePadStart
            | StandardBuiltinId::StringPrototypePadEnd
            | StandardBuiltinId::StringPrototypeRepeat
            | StandardBuiltinId::RegExpPrototypeFlagsGetter
            | StandardBuiltinId::RegExpPrototypeSourceGetter
            | StandardBuiltinId::RegExpPrototypeHasIndicesGetter
            | StandardBuiltinId::RegExpPrototypeGlobalGetter
            | StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter
            | StandardBuiltinId::RegExpPrototypeMultilineGetter
            | StandardBuiltinId::RegExpPrototypeDotAllGetter
            | StandardBuiltinId::RegExpPrototypeUnicodeGetter
            | StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter
            | StandardBuiltinId::RegExpPrototypeStickyGetter
            | StandardBuiltinId::RegExpPrototypeSymbolMatch
            | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
            | StandardBuiltinId::RegExpPrototypeSymbolSearch
            | StandardBuiltinId::StringPrototypeEndsWith
            | StandardBuiltinId::StringPrototypeIncludes
            | StandardBuiltinId::StringPrototypeStartsWith
            | StandardBuiltinId::StringPrototypeToUpperCase
            | StandardBuiltinId::StringPrototypeTrim
            | StandardBuiltinId::StringPrototypeTrimStart
            | StandardBuiltinId::StringPrototypeTrimEnd
            | StandardBuiltinId::StringPrototypeIsWellFormed
            | StandardBuiltinId::StringPrototypeToWellFormed
            | StandardBuiltinId::DateNow
            | StandardBuiltinId::DatePrototypeGetTime
            | StandardBuiltinId::DatePrototypeSetTime
            | StandardBuiltinId::DatePrototypeValueOf
            | StandardBuiltinId::DatePrototypeGetFullYear
            | StandardBuiltinId::DatePrototypeGetUtcFullYear
            | StandardBuiltinId::DatePrototypeGetMonth
            | StandardBuiltinId::DatePrototypeGetUtcMonth
            | StandardBuiltinId::DatePrototypeGetDate
            | StandardBuiltinId::DatePrototypeGetUtcDate
            | StandardBuiltinId::DatePrototypeGetDay
            | StandardBuiltinId::DatePrototypeGetUtcDay
            | StandardBuiltinId::DatePrototypeGetHours
            | StandardBuiltinId::DatePrototypeGetUtcHours
            | StandardBuiltinId::DatePrototypeGetMinutes
            | StandardBuiltinId::DatePrototypeGetUtcMinutes
            | StandardBuiltinId::DatePrototypeGetSeconds
            | StandardBuiltinId::DatePrototypeGetUtcSeconds
            | StandardBuiltinId::DatePrototypeGetMilliseconds
            | StandardBuiltinId::DatePrototypeGetUtcMilliseconds
            | StandardBuiltinId::DatePrototypeGetTimezoneOffset
            | StandardBuiltinId::DatePrototypeGetYear
            | StandardBuiltinId::DatePrototypeSetYear
            | StandardBuiltinId::DatePrototypeSetFullYear
            | StandardBuiltinId::DatePrototypeSetUtcFullYear
            | StandardBuiltinId::DatePrototypeSetMonth
            | StandardBuiltinId::DatePrototypeSetUtcMonth
            | StandardBuiltinId::DatePrototypeSetDate
            | StandardBuiltinId::DatePrototypeSetUtcDate
            | StandardBuiltinId::DatePrototypeSetHours
            | StandardBuiltinId::DatePrototypeSetUtcHours
            | StandardBuiltinId::DatePrototypeSetMinutes
            | StandardBuiltinId::DatePrototypeSetUtcMinutes
            | StandardBuiltinId::DatePrototypeSetSeconds
            | StandardBuiltinId::DatePrototypeSetUtcSeconds
            | StandardBuiltinId::DatePrototypeSetMilliseconds
            | StandardBuiltinId::DatePrototypeSetUtcMilliseconds
            | StandardBuiltinId::DatePrototypeToUtcString
            | StandardBuiltinId::DateUtc
            | StandardBuiltinId::ArrayOf
            | StandardBuiltinId::ErrorPrototypeToString
            | StandardBuiltinId::BoundFunctionInvoker
            | StandardBuiltinId::RegExpLegacyStaticGetter
            | StandardBuiltinId::RegExpLegacyStaticSetter
            | StandardBuiltinId::RegExpEscape
            | StandardBuiltinId::JsonParse
            | StandardBuiltinId::JsonStringify
            | StandardBuiltinId::JsonRawJson
            | StandardBuiltinId::JsonIsRawJson
            | StandardBuiltinId::AtomicsAdd
            | StandardBuiltinId::AtomicsAnd
            | StandardBuiltinId::AtomicsCompareExchange
            | StandardBuiltinId::AtomicsExchange
            | StandardBuiltinId::AtomicsLoad
            | StandardBuiltinId::AtomicsNotify
            | StandardBuiltinId::AtomicsOr
            | StandardBuiltinId::AtomicsPause
            | StandardBuiltinId::AtomicsSub
            | StandardBuiltinId::AtomicsStore
            | StandardBuiltinId::AtomicsWait
            | StandardBuiltinId::AtomicsWaitAsync
            | StandardBuiltinId::AtomicsXor
            | StandardBuiltinId::AtomicsIsLockFree
            | StandardBuiltinId::EvalFunction
            | StandardBuiltinId::ThrowTypeError
            | StandardBuiltinId::Escape
            | StandardBuiltinId::Unescape
            | StandardBuiltinId::SymbolFor
            | StandardBuiltinId::SymbolKeyFor
            | StandardBuiltinId::SymbolPrototypeDescriptionGetter
            | StandardBuiltinId::SymbolPrototypeToString
            | StandardBuiltinId::SymbolPrototypeValueOf
            | StandardBuiltinId::SymbolPrototypeToPrimitive => {}
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `%ThrowTypeError%`",
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
        let construct_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectConstruct.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.construct`",
                )
            })?;
        let apply_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectApply.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.apply`",
                )
            })?;
        let get_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGet.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.get`",
                )
            })?;
        let get_prototype_of_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetPrototypeOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.getPrototypeOf`",
                )
            })?;
        let get_own_property_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.getOwnPropertyDescriptor`",
                )
            })?;
        let set_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSet.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.set`",
                )
            })?;
        let has_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectHas.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.has`",
                )
            })?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.defineProperty`",
                )
            })?;
        let delete_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDeleteProperty.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.deleteProperty`",
                )
            })?;
        let is_extensible_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectIsExtensible.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.isExtensible`",
                )
            })?;
        let prevent_extensions_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectPreventExtensions.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.preventExtensions`",
                )
            })?;
        let set_prototype_of_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectSetPrototypeOf.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.setPrototypeOf`",
                )
            })?;
        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Reflect.ownKeys`",
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
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::GlobalSet(REFLECT_OBJECT_GLOBAL_INDEX));
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
        }
        let key_local = self.reserve_temp_local();
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.toStringTag"),
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
            self.strings.payload("Symbol.toStringTag"),
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
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
            self.strings.payload("Symbol.toStringTag"),
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

        for (name, builtin) in [
            ("add", StandardBuiltinId::AtomicsAdd),
            ("and", StandardBuiltinId::AtomicsAnd),
            ("compareExchange", StandardBuiltinId::AtomicsCompareExchange),
            ("exchange", StandardBuiltinId::AtomicsExchange),
            ("load", StandardBuiltinId::AtomicsLoad),
            ("notify", StandardBuiltinId::AtomicsNotify),
            ("or", StandardBuiltinId::AtomicsOr),
            ("pause", StandardBuiltinId::AtomicsPause),
            ("store", StandardBuiltinId::AtomicsStore),
            ("sub", StandardBuiltinId::AtomicsSub),
            ("wait", StandardBuiltinId::AtomicsWait),
            ("waitAsync", StandardBuiltinId::AtomicsWaitAsync),
            ("xor", StandardBuiltinId::AtomicsXor),
            ("isLockFree", StandardBuiltinId::AtomicsIsLockFree),
        ] {
            let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                    builtin.debug_name()
                ))
            })?;
            self.emit_object_define_function_data(object_local, name, meta, function)?;
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

        let function_meta = self
            .functions
            .get(&StandardBuiltinId::FunctionConstructor.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Function`",
                )
            })?;
        self.emit_function_value_payload(function_meta, function)?;
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
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(payload_local));
        self.emit_object_define_data(
            typed_array_constructor_local,
            key_local,
            typed_array_prototype_local,
            tag_local,
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
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

        let at_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeAt.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.at`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "at",
            at_meta,
            function,
        )?;

        let values_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeValues.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.values`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "values",
            values_meta,
            function,
        )?;
        let keys_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeKeys.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.keys`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "keys",
            keys_meta,
            function,
        )?;
        let entries_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayPrototypeEntries.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.entries`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "entries",
            entries_meta,
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.prototype.toLocaleString`",
                )
            })?;
        self.emit_object_define_function_data(
            typed_array_prototype_local,
            "toLocaleString",
            to_locale_string_meta,
            function,
        )?;
        self.emit_function_value_payload(values_meta, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            typed_array_prototype_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;

        let from_meta = self
            .functions
            .get(&StandardBuiltinId::TypedArrayFrom.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.from`",
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `TypedArray.of`",
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
                    "unsupported in porffor wasm-aot first slice: missing builtin constructor global `{}`",
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
        let realm_intrinsic_offset = typed_array_realm_intrinsics_prototype_offset(builtin)
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing realm intrinsic prototype offset `{}`",
                    builtin.debug_name()
                ))
            })?;
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_intrinsic_prototype(
            self.scratch_local,
            realm_intrinsic_offset,
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
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array Iterator.prototype.next`",
                )
            })?;
        let iterator_meta = self
            .functions
            .get(&StandardBuiltinId::ArrayIteratorIdentity.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Array Iterator.prototype[Symbol.iterator]`",
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
            self.strings.payload("Symbol.toStringTag"),
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

    pub(crate) fn init_runtime_roots(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::GlobalSet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        let object_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_prototype_local));
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_object_prototype(
            self.scratch_local,
            object_prototype_local,
            function,
        );
        self.release_temp_local(object_prototype_local);
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(FUNCTION_PROTOTYPE_GLOBAL_INDEX));
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
        self.emit_store_current_realm_global_intrinsic(
            ARRAY_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ITERATOR_PROTOTYPE_GLOBAL_INDEX));
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ITERATOR_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(
            ITERATOR_FROM_WRAPPER_PROTOTYPE_GLOBAL_INDEX,
        ));
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
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(NUMBER_PROTOTYPE_GLOBAL_INDEX));
        self.emit_store_current_realm_global_intrinsic(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            HEAP_REALM_INTRINSICS_NUMBER_PROTOTYPE_OFFSET,
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
            HEAP_REALM_INTRINSICS_STRING_PROTOTYPE_OFFSET,
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
            HEAP_REALM_INTRINSICS_BOOLEAN_PROTOTYPE_OFFSET,
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
            HEAP_REALM_INTRINSICS_SYMBOL_PROTOTYPE_OFFSET,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::GlobalSet(ERROR_PROTOTYPE_GLOBAL_INDEX));
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
        function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_store_realm_type_error_prototype(
            self.scratch_local,
            native_error_prototype_local,
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
        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        let regexp_prototype_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(regexp_prototype_local));
        self.store_i64_const_at_offset(
            regexp_prototype_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_REGEXP,
            function,
        );
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
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.match]`",
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
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.matchAll]`",
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
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `RegExp.prototype[Symbol.search]`",
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
                        "unsupported in porffor wasm-aot first slice: missing builtin meta `Array.prototype.toString`",
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
            .clone()
            .into_iter()
            .map(|(name, kind)| ScriptGlobalBindingIr { name, kind })
            .filter(|binding| {
                self.runtime_bootstrap_plan
                    .should_install_script_global_binding(binding.kind)
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
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            self.scratch_local,
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
            match binding.kind {
                ScriptGlobalBindingKind::Intrinsic => {
                    function.instruction(&Instruction::LocalGet(object_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Infinity => {
                    function.instruction(&Instruction::F64Const(Ieee64::from(f64::INFINITY)));
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::NaN => {
                    function.instruction(&Instruction::F64Const(Ieee64::from(f64::NAN)));
                    function.instruction(&Instruction::I64ReinterpretF64);
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Undefined => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Var => {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::Function => {
                    let meta = self.functions.values().find(|meta| meta.name == binding.name).ok_or_else(
                        || {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: unknown script-global function `{}`",
                                binding.name
                            ))
                        },
                    )?;
                    self.emit_function_value_payload(meta, function)?;
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::ReflectObject => {
                    function.instruction(&Instruction::GlobalGet(REFLECT_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::MathObject => {
                    function.instruction(&Instruction::GlobalGet(MATH_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::JsonObject => {
                    function.instruction(&Instruction::GlobalGet(JSON_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::AtomicsObject => {
                    function.instruction(&Instruction::GlobalGet(ATOMICS_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::BuiltinFunction(builtin) => {
                    if let Some(global_index) = standard_builtin_constructor_global_index(builtin) {
                        function.instruction(&Instruction::GlobalGet(global_index));
                        function.instruction(&Instruction::LocalSet(payload_local));
                    } else {
                        let meta = self.functions.get(&builtin.function_id()).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: missing builtin meta `{}`",
                                builtin.debug_name()
                            ))
                        })?;
                        self.emit_function_value_payload(meta, function)?;
                        function.instruction(&Instruction::LocalSet(payload_local));
                    }
                    function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
                ScriptGlobalBindingKind::HostFunction(builtin) => {
                    let meta = self
                        .functions
                        .get(&builtin.function_id())
                        .cloned()
                        .ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in porffor wasm-aot first slice: unknown script-global host function `{}`",
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
                !matches!(
                    binding.kind,
                    ScriptGlobalBindingKind::Infinity
                        | ScriptGlobalBindingKind::NaN
                        | ScriptGlobalBindingKind::Undefined
                ),
                false,
                !matches!(
                    binding.kind,
                    ScriptGlobalBindingKind::Intrinsic
                        | ScriptGlobalBindingKind::Infinity
                        | ScriptGlobalBindingKind::NaN
                        | ScriptGlobalBindingKind::Undefined
                        | ScriptGlobalBindingKind::Var
                        | ScriptGlobalBindingKind::Function
                ),
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
