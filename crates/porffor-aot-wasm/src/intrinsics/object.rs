//! `object` intrinsic installation.
//!
//! Extracted verbatim from `builtins/bootstrap.rs::init_builtin_constructor_object`.
//! Property installation order is observable through `Object.keys`, so the
//! statement order inside each installer is load-bearing — do not reorder.

use super::super::*;
use super::IntrinsicInstall;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_object_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Re-bind the shared preamble values under the names the moved body
        // already uses, so the body below is a verbatim copy of the arm it
        // replaced. Most families read only a few of them.
        #[allow(unused_variables)]
        let IntrinsicInstall {
            builtin,
            meta,
            prototype_global_index,
            constructor_global_index,
            object_local,
            key_local,
            payload_local,
            tag_local,
            prototype_object_local,
        } = *context;

        let prototype_object_local = self.reserve_temp_local();
        let group_by_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGroupBy.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.groupBy`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "groupBy", &group_by_meta, function)?;
        let from_entries_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectFromEntries.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.fromEntries`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "fromEntries",
            &from_entries_meta,
            function,
        )?;
        let assign_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectAssign.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.assign`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "assign", &assign_meta, function)?;
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
        let lookup_getter_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectPrototypeLookupGetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.__lookupGetter__`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(
            prototype_object_local,
            "__lookupGetter__",
            lookup_getter_meta,
            function,
        )?;
        let lookup_setter_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectPrototypeLookupSetter.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.prototype.__lookupSetter__`",
                )
            })?;
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_define_function_data(
            prototype_object_local,
            "__lookupSetter__",
            lookup_setter_meta,
            function,
        )?;
        let proto_getter_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectPrototypeProtoGetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `get Object.prototype.__proto__`",
                )
            })?;
        let proto_setter_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectPrototypeProtoSetter.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `set Object.prototype.__proto__`",
                )
            })?;
        let proto_key_local = self.reserve_temp_local();
        let proto_getter_payload_local = self.reserve_temp_local();
        let proto_getter_tag_local = self.reserve_temp_local();
        let proto_setter_payload_local = self.reserve_temp_local();
        let proto_setter_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("__proto__")));
        function.instruction(&Instruction::LocalSet(proto_key_local));
        self.emit_function_value_payload(&proto_getter_meta, function)?;
        function.instruction(&Instruction::LocalSet(proto_getter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(proto_getter_tag_local));
        self.emit_function_value_payload(&proto_setter_meta, function)?;
        function.instruction(&Instruction::LocalSet(proto_setter_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(proto_setter_tag_local));
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(prototype_object_local));
        self.emit_object_append_accessor_property_with_flags(
            prototype_object_local,
            proto_key_local,
            Some((proto_getter_payload_local, proto_getter_tag_local)),
            Some((proto_setter_payload_local, proto_setter_tag_local)),
            false,
            true,
            function,
        )?;
        self.release_temp_local(proto_setter_tag_local);
        self.release_temp_local(proto_setter_payload_local);
        self.release_temp_local(proto_getter_tag_local);
        self.release_temp_local(proto_getter_payload_local);
        self.release_temp_local(proto_key_local);
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
        self.emit_object_define_function_data(object_local, "create", create_meta, function)?;
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
        let get_own_property_descriptors_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectGetOwnPropertyDescriptors.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.getOwnPropertyDescriptors`",
                )
            })?;
        self.emit_object_define_function_data(
            object_local,
            "getOwnPropertyDescriptors",
            get_own_property_descriptors_meta,
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
        self.emit_object_define_function_data(object_local, "values", values_meta, function)?;
        let entries_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectEntries.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.entries`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "entries", entries_meta, function)?;
        let has_own_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectHasOwn.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.hasOwn`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "hasOwn", has_own_meta, function)?;
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
        self.emit_object_define_function_data(object_local, "isSealed", is_sealed_meta, function)?;
        let is_frozen_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectIsFrozen.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.isFrozen`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "isFrozen", is_frozen_meta, function)?;
        let seal_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectSeal.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.seal`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "seal", seal_meta, function)?;
        let freeze_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectFreeze.function_id())
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in porffor wasm-aot first slice: missing builtin meta `Object.freeze`",
                )
            })?;
        self.emit_object_define_function_data(object_local, "freeze", freeze_meta, function)?;
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

        Ok(())
    }
}
