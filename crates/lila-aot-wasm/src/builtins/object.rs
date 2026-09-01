use super::super::*;
use super::binary_data::{TypedArrayViewLocals, TypedArrayWitnessUse};
use crate::objects::{
    ObjectPreventExtensionsRequest, PreventExtensionsResultLocal,
    PreventExtensionsTraversalTargetLocals, PropertyKeyLocals, ProxyHandlerLocals,
    ProxyOwnKeysTrapLocals, ProxyOwnKeysTrapResultLocals, ProxyRevocationRoute, ProxySlotLocals,
    ProxyTargetLocals, StoredDescriptorDataLocals, StoredDescriptorGetterLocals,
    StoredDescriptorLocals, StoredDescriptorSetterLocals, TaggedLocals, WasmDescriptor,
    WasmPartialDescriptor,
};

mod assign;
mod define_property;
mod enumerable_own_properties;
mod get_own_property_descriptor;
mod get_own_property_descriptors;
mod integrity_test;
mod object_to_locale_string_invoke;
mod own_descriptor_predicate;
mod prototype_lookup;

impl<'a> FunctionBuilder<'a> {
    pub(super) fn compile_object_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let object_prototype_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        for kind in [
            ValueKind::Object,
            ValueKind::Array,
            ValueKind::Function,
            ValueKind::Arguments,
        ] {
            function.instruction(&Instruction::LocalGet(arg_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(arg_payload_local));
            function.instruction(&Instruction::LocalSet(self.result_local));
            function.instruction(&Instruction::LocalGet(arg_tag_local));
            function.instruction(&Instruction::LocalSet(self.result_tag_local));
            function.instruction(&Instruction::Br(1));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            NUMBER_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_NUMBER,
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            STRING_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_STRING,
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            BOOLEAN_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_BOOLEAN,
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_boxed_wrapper_from_locals(
            SYMBOL_PROTOTYPE_GLOBAL_INDEX,
            BOXED_PRIMITIVE_KIND_SYMBOL,
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::BigInt.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(HEAP_BIGINT_VALUE_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        let bigint_constructor_local = self.reserve_temp_local();
        let bigint_fallback_prototype_local = self.reserve_temp_local();
        let bigint_prototype_local = self.reserve_temp_local();
        let bigint_realm_local = self.reserve_temp_local();
        function.instruction(&Instruction::GlobalGet(BIGINT_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(bigint_constructor_local));
        self.load_i64_to_local_from_offset(
            bigint_constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            bigint_fallback_prototype_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(bigint_realm_local));
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            bigint_realm_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.emit_load_realm_intrinsic_prototype_or_local(
            bigint_realm_local,
            HEAP_REALM_INTRINSICS_BIGINT_PROTOTYPE_OFFSET,
            bigint_fallback_prototype_local,
            bigint_prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(bigint_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.emit_store_boxed_primitive_metadata(
            self.result_local,
            BOXED_PRIMITIVE_KIND_BIGINT,
            arg_payload_local,
            arg_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(bigint_realm_local);
        self.release_temp_local(bigint_prototype_local);
        self.release_temp_local(bigint_fallback_prototype_local);
        self.release_temp_local(bigint_constructor_local);
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::GlobalGet(OBJECT_PROTOTYPE_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_prototype_local));
        if let (Some(new_target_payload_local), Some(new_target_tag_local)) =
            (self.new_target_payload_local(), self.new_target_tag_local())
        {
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_function_defining_realm_object_prototype(
                new_target_payload_local,
                object_prototype_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(object_prototype_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_create_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let properties_payload_local = self.reserve_temp_local();
        let properties_tag_local = self.reserve_temp_local();
        let define_properties_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperties.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Object.defineProperties`",
                )
            })?;
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_builtin_arg_to_locals(
            1,
            properties_payload_local,
            properties_tag_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.create prototype must be object or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(arg_payload_local),
            Some(arg_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(properties_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_direct_js_call(
            &define_properties_meta,
            None,
            &[
                (self.result_local, self.result_tag_local),
                (properties_payload_local, properties_tag_local),
            ],
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.release_temp_local(properties_tag_local);
        self.release_temp_local(properties_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_get_prototype_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let argument_payload_local = self.reserve_temp_local();
        let argument_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, argument_payload_local, argument_tag_local, function);
        self.compile_nullish_tagged_i32(argument_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Cannot convert undefined or null to object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
            argument_payload_local,
            argument_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_object_get_prototype_of(
                object_payload_local,
                object_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else {
            self.emit_object_get_prototype_of_without_proxy(
                object_payload_local,
                object_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(argument_tag_local);
        self.release_temp_local(argument_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_set_prototype_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let set_result_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(1, proto_payload_local, proto_tag_local, function);

        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.setPrototypeOf target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(proto_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.setPrototypeOf prototype must be object or null",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_set_prototype_of_i32(
            target_payload_local,
            target_tag_local,
            proto_payload_local,
            proto_tag_local,
            set_result_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(set_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.setPrototypeOf returned false",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(set_result_local);
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_define_properties_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let properties_arg_payload_local = self.reserve_temp_local();
        let properties_arg_tag_local = self.reserve_temp_local();
        let properties_payload_local = self.reserve_temp_local();
        let properties_tag_local = self.reserve_temp_local();
        let converted_descriptors_payload_local = self.reserve_temp_local();
        let converted_descriptors_tag_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let own_property_key_local = self.reserve_temp_local();
        let own_descriptor_payload_local = self.reserve_temp_local();
        let own_descriptor_tag_local = self.reserve_temp_local();
        let enumerable_key_local = self.reserve_temp_local();
        let enumerable_present_local = self.reserve_temp_local();
        let enumerable_payload_local = self.reserve_temp_local();
        let enumerable_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let converted_descriptor_payload_local = self.reserve_temp_local();
        let converted_descriptor_tag_local = self.reserve_temp_local();
        let converted_descriptor_present_local = self.reserve_temp_local();
        let define_result_payload_local = self.reserve_temp_local();
        let define_result_tag_local = self.reserve_temp_local();
        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;
        let object_define_meta = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Object.defineProperty builtin"))?;

        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, function);
        self.emit_builtin_arg_to_locals(
            1,
            properties_arg_payload_local,
            properties_arg_tag_local,
            function,
        );
        self.emit_is_heap_object_like_tag_i32(target_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.defineProperties target must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.compile_nullish_tagged_i32(properties_arg_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.defineProperties properties must not be null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        self.emit_value_to_current_function_realm_object_locals(
            properties_arg_payload_local,
            properties_arg_tag_local,
            properties_payload_local,
            properties_tag_local,
            function,
        )?;

        // Convert every enumerable descriptor before defining anything
        // on the target. A fresh ordinary object stores the completed
        // descriptors without introducing observable operations.
        self.emit_alloc_plain_object_with_prototype(None, None, function)?;
        function.instruction(&Instruction::LocalSet(converted_descriptors_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(converted_descriptors_tag_local));
        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(properties_payload_local, properties_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("enumerable")));
        function.instruction(&Instruction::LocalSet(enumerable_key_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (properties_payload_local, properties_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            own_descriptor_payload_local,
            own_descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(own_descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_own_data_field_read(
            own_descriptor_payload_local,
            own_descriptor_tag_local,
            enumerable_key_local,
            enumerable_present_local,
            enumerable_payload_local,
            enumerable_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(enumerable_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(enumerable_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_payload_local));
        function.instruction(&Instruction::LocalSet(own_property_key_local));
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_property_key_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(own_property_key_local));
        function.instruction(&Instruction::End);
        self.emit_object_read_with_key_tag(
            properties_payload_local,
            properties_tag_local,
            properties_payload_local,
            properties_tag_local,
            own_property_key_local,
            Some(own_key_tag_local),
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        // Store the completed descriptor itself, not a property whose
        // attributes were taken from it: a heap property entry keeps
        // only the four attribute bits, so defining the descriptor as
        // a real property would turn every absent field into an
        // explicit `false` for the second pass.
        let descriptor = self.emit_to_property_descriptor(
            TaggedLocals::new(descriptor_payload_local, descriptor_tag_local),
            "Object.defineProperties descriptor must be object",
            function,
        )?;
        self.emit_from_present_property_descriptor(
            descriptor,
            TaggedLocals::new(
                converted_descriptor_payload_local,
                converted_descriptor_tag_local,
            ),
            function,
        )?;
        self.emit_object_define_enumerable_data(
            converted_descriptors_payload_local,
            own_property_key_local,
            converted_descriptor_payload_local,
            converted_descriptor_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(own_key_payload_local));
        function.instruction(&Instruction::LocalSet(own_property_key_local));
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_property_key_local));
        function.instruction(&Instruction::I64Const(PROPERTY_KEY_SYMBOL_MARKER as i64));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(own_property_key_local));
        function.instruction(&Instruction::End);
        // The scratch object is ours, so an own data read returns the
        // stored descriptor without observable operations, and its
        // presence bit marks the keys the first pass skipped.
        self.emit_object_own_data_field_read(
            converted_descriptors_payload_local,
            converted_descriptors_tag_local,
            own_property_key_local,
            converted_descriptor_present_local,
            descriptor_payload_local,
            descriptor_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(converted_descriptor_present_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_js_call(
            &object_define_meta,
            None,
            &[
                (target_payload_local, target_tag_local),
                (own_key_payload_local, own_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(target_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(target_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(define_result_tag_local);
        self.release_temp_local(define_result_payload_local);
        self.release_temp_local(converted_descriptor_present_local);
        self.release_temp_local(converted_descriptor_tag_local);
        self.release_temp_local(converted_descriptor_payload_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(enumerable_tag_local);
        self.release_temp_local(enumerable_payload_local);
        self.release_temp_local(enumerable_present_local);
        self.release_temp_local(enumerable_key_local);
        self.release_temp_local(own_descriptor_tag_local);
        self.release_temp_local(own_descriptor_payload_local);
        self.release_temp_local(own_property_key_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(converted_descriptors_tag_local);
        self.release_temp_local(converted_descriptors_payload_local);
        self.release_temp_local(properties_tag_local);
        self.release_temp_local(properties_payload_local);
        self.release_temp_local(properties_arg_tag_local);
        self.release_temp_local(properties_arg_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_get_own_property_names_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let boxed_string_payload_local = self.reserve_temp_local();
        let boxed_string_offset_local = self.reserve_temp_local();
        let boxed_string_byte_len_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let key_index_local = self.reserve_temp_local();
        let key_index_found_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        let own_property_names_string_tag_local = self.reserve_temp_local();
        let typed_array_brand_local = self.reserve_temp_local();
        let typed_array_buffer_payload_local = self.reserve_temp_local();
        let typed_array_byte_offset_local = self.reserve_temp_local();
        let typed_array_stored_byte_length_local = self.reserve_temp_local();
        let typed_array_bytes_per_element_local = self.reserve_temp_local();
        let typed_array_length_local = self.reserve_temp_local();
        let ordinary_string_count_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(own_property_names_string_tag_local));

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.getOwnPropertyNames called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let proxy_trap_result = self.emit_proxy_own_keys_trap_result(
            TaggedLocals::new(arg_payload_local, arg_tag_local),
            proxy_handled_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            ProxyOwnKeysTrapLocals::new(proxy_trap_payload_local, proxy_trap_tag_local),
            ProxyOwnKeysTrapResultLocals::new(
                proxy_trap_result_payload_local,
                proxy_trap_result_tag_local,
            ),
            key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_own_keys_filtered_result(
            ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
            proxy_trap_result,
            ValueKind::String,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.emit_alloc_array_payload_with_length(entry_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            len_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boxed_string_payload_local,
            function,
        );
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_LEN_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        self.load_i64_to_local_from_offset(
            write_index_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            key_index_local,
            key_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::Else);
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(entry_local, result_payload_local, function)?;
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_LEN_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            key_index_local,
            key_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            key_index_local,
            key_index_found_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            typed_array_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(typed_array_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_load_typed_array_private_state(
            arg_payload_local,
            typed_array_buffer_payload_local,
            typed_array_byte_offset_local,
            typed_array_stored_byte_length_local,
            typed_array_bytes_per_element_local,
            function,
        );
        let typed_array_view = TypedArrayViewLocals::new(
            arg_payload_local,
            typed_array_buffer_payload_local,
            typed_array_byte_offset_local,
            typed_array_stored_byte_length_local,
            typed_array_bytes_per_element_local,
        );
        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
                length_local: typed_array_length_local,
            },
            function,
        )?;

        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(ordinary_string_count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(ordinary_string_count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(ordinary_string_count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(typed_array_length_local));
        function.instruction(&Instruction::LocalGet(ordinary_string_count_local));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(ordinary_string_count_local));
        self.emit_alloc_array_payload_with_length(
            ordinary_string_count_local,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(typed_array_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_array_write(
            result_payload_local,
            index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(typed_array_length_local));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            arg_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_all_named_string_props_count(
            arg_payload_local,
            write_index_local,
            function,
        );
        self.emit_alloc_array_payload_with_length(
            write_index_local,
            result_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            arg_payload_local,
            index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_all_named_string_props_write_keys(
            arg_payload_local,
            result_payload_local,
            write_index_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_alloc_array_payload_with_length(
            write_index_local,
            result_payload_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            result_payload_local,
            index_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(key_index_found_local));
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            key_index_local,
            key_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_index_found_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(len_local));
        self.emit_array_read(
            result_payload_local,
            len_local,
            buffer_local,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(ordinary_string_count_local));
        self.emit_known_array_index_from_property_key(
            buffer_local,
            index_number_payload_local,
            ordinary_string_count_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(ordinary_string_count_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::LocalGet(key_index_local));
        function.instruction(&Instruction::I64GtU);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_write(
            result_payload_local,
            entry_local,
            buffer_local,
            boxed_kind_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(entry_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.emit_array_write(
            result_payload_local,
            entry_local,
            key_payload_local,
            own_property_names_string_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(ordinary_string_count_local);
        self.release_temp_local(typed_array_length_local);
        self.release_temp_local(typed_array_bytes_per_element_local);
        self.release_temp_local(typed_array_stored_byte_length_local);
        self.release_temp_local(typed_array_byte_offset_local);
        self.release_temp_local(typed_array_buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
        self.release_temp_local(own_property_names_string_tag_local);
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(key_index_found_local);
        self.release_temp_local(key_index_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(boxed_string_byte_len_local);
        self.release_temp_local(boxed_string_offset_local);
        self.release_temp_local(boxed_string_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_get_own_property_symbols_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.getOwnPropertySymbols called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let proxy_trap_result = self.emit_proxy_own_keys_trap_result(
            TaggedLocals::new(arg_payload_local, arg_tag_local),
            proxy_handled_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            ProxyOwnKeysTrapLocals::new(proxy_trap_payload_local, proxy_trap_tag_local),
            ProxyOwnKeysTrapResultLocals::new(
                proxy_trap_result_payload_local,
                proxy_trap_result_tag_local,
            ),
            key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_own_keys_filtered_result(
            ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
            proxy_trap_result,
            ValueKind::Symbol,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_LEN_OFFSET,
            write_index_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            write_index_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(len_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(len_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_property_key_payload_to_value_payload(key_payload_local, function);
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARRAY_NAMED_PROPS_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARRAY_NAMED_PROPS_LEN_OFFSET,
            len_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_payload_is_symbol_i32(key_payload_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Symbol.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_property_key_payload_to_value_payload(key_payload_local, function);
        function.instruction(&Instruction::LocalSet(key_payload_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_keys_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let buffer_local = self.reserve_temp_local();
        let len_local = self.reserve_temp_local();
        let scan_index_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        let count_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let write_index_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let index_number_payload_local = self.reserve_temp_local();
        let previous_index_local = self.reserve_temp_local();
        let next_index_local = self.reserve_temp_local();
        let next_key_payload_local = self.reserve_temp_local();
        let found_index_local = self.reserve_temp_local();
        let proxy_target_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_handler_tag_local = self.reserve_temp_local();
        let proxy_trap_payload_local = self.reserve_temp_local();
        let proxy_trap_tag_local = self.reserve_temp_local();
        let proxy_trap_result_payload_local = self.reserve_temp_local();
        let proxy_trap_result_tag_local = self.reserve_temp_local();
        let proxy_handled_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.keys requires object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        let proxy_trap_result = self.emit_proxy_own_keys_trap_result(
            TaggedLocals::new(arg_payload_local, arg_tag_local),
            proxy_handled_local,
            ProxySlotLocals::new(
                ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
                ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            ),
            ProxyOwnKeysTrapLocals::new(proxy_trap_payload_local, proxy_trap_tag_local),
            ProxyOwnKeysTrapResultLocals::new(
                proxy_trap_result_payload_local,
                proxy_trap_result_tag_local,
            ),
            key_payload_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(proxy_handled_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_proxy_object_keys_from_own_keys_result(
            ProxyTargetLocals::new(proxy_target_payload_local, proxy_target_tag_local),
            ProxyHandlerLocals::new(proxy_handler_payload_local, proxy_handler_tag_local),
            proxy_trap_result,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::I64Const(0xFFFF_FFFFu64 as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::LocalSet(count_local));
        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            arg_payload_local,
            scan_index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_descriptor_kind_for_index(
            arg_payload_local,
            scan_index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_enumerable_named_string_props_count(
            arg_payload_local,
            count_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_array_advance_to_next_present_index(
            arg_payload_local,
            scan_index_local,
            len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_descriptor_kind_for_index(
            arg_payload_local,
            scan_index_local,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_ARGUMENTS_CALLEE_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("callee")));
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_enumerable_named_string_props_write_keys(
            arg_payload_local,
            result_payload_local,
            write_index_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(self.scratch_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_unpack_string_payload(key_payload_local, buffer_local, len_local, function);
        self.emit_utf16_code_unit_len_from_utf8_locals(
            buffer_local,
            len_local,
            count_local,
            function,
        );
        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::F64ConvertI64U);
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(index_number_payload_local));
        self.emit_number_to_string_payload(index_number_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(key_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_PTR_OFFSET,
            buffer_local,
            function,
        );
        self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_tag_from_payload(key_payload_local, key_tag_local, function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(count_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.emit_alloc_array_payload_with_length(count_local, result_payload_local, function)?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::I64Const(-1));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(found_index_local));
        function.instruction(&Instruction::I64Const(MAX_ARRAY_LENGTH as i64));
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(next_key_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_tag_from_payload(key_payload_local, key_tag_local, function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            index_number_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::LocalGet(previous_index_local));
        function.instruction(&Instruction::I64GtS);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_number_payload_local));
        function.instruction(&Instruction::LocalSet(next_index_local));
        function.instruction(&Instruction::LocalGet(key_payload_local));
        function.instruction(&Instruction::LocalSet(next_key_payload_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(found_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(found_index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            next_key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::LocalGet(next_index_local));
        function.instruction(&Instruction::LocalSet(previous_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(buffer_local));
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(HEAP_OBJECT_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_DESCRIPTOR_KIND_OFFSET,
            descriptor_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_kind_local));
        function.instruction(&Instruction::I64Const(OBJECT_DESCRIPTOR_ENUMERABLE as i64));
        function.instruction(&Instruction::I64And);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_OBJECT_KEY_OFFSET,
            key_payload_local,
            function,
        );
        self.emit_property_key_tag_from_payload(key_payload_local, key_tag_local, function);
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_known_array_index_from_property_key(
            key_payload_local,
            index_number_payload_local,
            key_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(key_tag_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(key_tag_local));
        self.emit_array_write(
            result_payload_local,
            write_index_local,
            key_payload_local,
            key_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(write_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(write_index_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(scan_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(scan_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(result_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(proxy_handled_local);
        self.release_temp_local(proxy_trap_result_tag_local);
        self.release_temp_local(proxy_trap_result_payload_local);
        self.release_temp_local(proxy_trap_tag_local);
        self.release_temp_local(proxy_trap_payload_local);
        self.release_temp_local(proxy_handler_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_target_payload_local);
        self.release_temp_local(found_index_local);
        self.release_temp_local(next_key_payload_local);
        self.release_temp_local(next_index_local);
        self.release_temp_local(previous_index_local);
        self.release_temp_local(index_number_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(write_index_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(count_local);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(scan_index_local);
        self.release_temp_local(len_local);
        self.release_temp_local(buffer_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_is_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let lhs_payload_local = self.reserve_temp_local();
        let lhs_tag_local = self.reserve_temp_local();
        let rhs_payload_local = self.reserve_temp_local();
        let rhs_tag_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, lhs_payload_local, lhs_tag_local, function);
        self.emit_builtin_arg_to_locals(1, rhs_payload_local, rhs_tag_local, function);

        self.emit_tagged_payload_same_value_i32(
            lhs_tag_local,
            lhs_payload_local,
            rhs_tag_local,
            rhs_payload_local,
            function,
        )?;
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(rhs_tag_local);
        self.release_temp_local(rhs_payload_local);
        self.release_temp_local(lhs_tag_local);
        self.release_temp_local(lhs_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_is_extensible_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_object_is_extensible_i32(
            arg_payload_local,
            arg_tag_local,
            self.result_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_seal_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let prevent_extensions_result_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let descriptor_payload_local = self.reserve_temp_local();
        let descriptor_tag_local = self.reserve_temp_local();
        let define_result_payload_local = self.reserve_temp_local();
        let define_result_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let boxed_string_payload_local = self.reserve_temp_local();
        let boxed_string_offset_local = self.reserve_temp_local();
        let boxed_string_byte_len_local = self.reserve_temp_local();
        let boxed_string_len_local = self.reserve_temp_local();
        let boxed_string_index_local = self.reserve_temp_local();
        let boxed_string_index_found_local = self.reserve_temp_local();
        let boxed_string_exotic_key_local = self.reserve_temp_local();

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.defineProperty builtin"))?;

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_object_prevent_extensions(
            ObjectPreventExtensionsRequest::new(
                PreventExtensionsTraversalTargetLocals::new(arg_payload_local, arg_tag_local),
                PreventExtensionsResultLocal::new(prevent_extensions_result_local),
            ),
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(prevent_extensions_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.seal could not prevent extensions",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(arg_payload_local, arg_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(descriptor_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(own_key_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(define_result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_result_tag_local));
        self.emit_object_define_data(
            descriptor_payload_local,
            own_key_payload_local,
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_tag_local));

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        // Direct String exotic indices and `length` are already
        // permanently non-configurable. Proxies still take the
        // observable Reflect.defineProperty path below.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(boxed_kind_local));
        self.emit_string_payload_equality_i32(own_key_payload_local, boxed_kind_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::Else);
        self.emit_known_array_index_from_property_key(
            own_key_payload_local,
            boxed_string_index_local,
            boxed_string_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_string_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boxed_string_payload_local,
            function,
        );
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            boxed_string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_string_index_local));
        function.instruction(&Instruction::LocalGet(boxed_string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_js_call(
            &define_property_meta,
            None,
            &[
                (arg_payload_local, arg_tag_local),
                (own_key_payload_local, own_key_tag_local),
                (descriptor_payload_local, descriptor_tag_local),
            ],
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(define_result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(define_result_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.seal could not make an own property non-configurable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(boxed_string_exotic_key_local);
        self.release_temp_local(boxed_string_index_found_local);
        self.release_temp_local(boxed_string_index_local);
        self.release_temp_local(boxed_string_len_local);
        self.release_temp_local(boxed_string_byte_len_local);
        self.release_temp_local(boxed_string_offset_local);
        self.release_temp_local(boxed_string_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(define_result_tag_local);
        self.release_temp_local(define_result_payload_local);
        self.release_temp_local(descriptor_tag_local);
        self.release_temp_local(descriptor_payload_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(prevent_extensions_result_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_freeze_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let prevent_extensions_result_local = self.reserve_temp_local();
        let own_keys_payload_local = self.reserve_temp_local();
        let own_keys_tag_local = self.reserve_temp_local();
        let own_keys_length_local = self.reserve_temp_local();
        let own_key_index_local = self.reserve_temp_local();
        let own_key_payload_local = self.reserve_temp_local();
        let own_key_tag_local = self.reserve_temp_local();
        let current_descriptor_payload_local = self.reserve_temp_local();
        let current_descriptor_tag_local = self.reserve_temp_local();
        let configurable_descriptor_payload_local = self.reserve_temp_local();
        let frozen_data_descriptor_payload_local = self.reserve_temp_local();
        let selected_descriptor_payload_local = self.reserve_temp_local();
        let descriptor_object_tag_local = self.reserve_temp_local();
        let descriptor_field_key_local = self.reserve_temp_local();
        let descriptor_has_get_local = self.reserve_temp_local();
        let descriptor_has_set_local = self.reserve_temp_local();
        let accessor_descriptor_local = self.reserve_temp_local();
        let define_result_payload_local = self.reserve_temp_local();
        let define_result_tag_local = self.reserve_temp_local();
        let boxed_kind_local = self.reserve_temp_local();
        let boxed_string_payload_local = self.reserve_temp_local();
        let boxed_string_offset_local = self.reserve_temp_local();
        let boxed_string_byte_len_local = self.reserve_temp_local();
        let boxed_string_len_local = self.reserve_temp_local();
        let boxed_string_index_local = self.reserve_temp_local();
        let boxed_string_index_found_local = self.reserve_temp_local();
        let boxed_string_exotic_key_local = self.reserve_temp_local();

        let own_keys_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectOwnKeys.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.ownKeys builtin"))?;
        let get_own_descriptor_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("missing Reflect.getOwnPropertyDescriptor builtin")
            })?;
        let define_property_meta = self
            .functions
            .get(&StandardBuiltinId::ReflectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| EmitError::unsupported("missing Reflect.defineProperty builtin"))?;

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));

        self.emit_object_prevent_extensions(
            ObjectPreventExtensionsRequest::new(
                PreventExtensionsTraversalTargetLocals::new(arg_payload_local, arg_tag_local),
                PreventExtensionsResultLocal::new(prevent_extensions_result_local),
            ),
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::LocalGet(prevent_extensions_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.freeze could not prevent extensions",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_direct_js_call(
            &own_keys_meta,
            None,
            &[(arg_payload_local, arg_tag_local)],
            own_keys_payload_local,
            own_keys_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.load_i64_to_local_from_offset(
            own_keys_payload_local,
            HEAP_LEN_OFFSET,
            own_keys_length_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(define_result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_result_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(descriptor_object_tag_local));

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(
            configurable_descriptor_payload_local,
        ));
        function.instruction(&Instruction::I64Const(self.strings.payload("configurable")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_define_data(
            configurable_descriptor_payload_local,
            descriptor_field_key_local,
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(frozen_data_descriptor_payload_local));
        self.emit_object_define_data(
            frozen_data_descriptor_payload_local,
            descriptor_field_key_local,
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("writable")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_define_data(
            frozen_data_descriptor_payload_local,
            descriptor_field_key_local,
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::LocalGet(own_keys_length_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            own_keys_payload_local,
            own_key_index_local,
            own_key_payload_local,
            own_key_tag_local,
            function,
        );
        self.emit_direct_js_call(
            &get_own_descriptor_meta,
            None,
            &[
                (arg_payload_local, arg_tag_local),
                (own_key_payload_local, own_key_tag_local),
            ],
            current_descriptor_payload_local,
            current_descriptor_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);

        function.instruction(&Instruction::LocalGet(current_descriptor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("get")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_own_property_present(
            current_descriptor_payload_local,
            current_descriptor_tag_local,
            descriptor_field_key_local,
            descriptor_has_get_local,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("set")));
        function.instruction(&Instruction::LocalSet(descriptor_field_key_local));
        self.emit_object_own_property_present(
            current_descriptor_payload_local,
            current_descriptor_tag_local,
            descriptor_field_key_local,
            descriptor_has_set_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(descriptor_has_get_local));
        function.instruction(&Instruction::LocalGet(descriptor_has_set_local));
        function.instruction(&Instruction::I64Or);
        function.instruction(&Instruction::LocalSet(accessor_descriptor_local));
        function.instruction(&Instruction::LocalGet(accessor_descriptor_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(
            configurable_descriptor_payload_local,
        ));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(frozen_data_descriptor_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalSet(selected_descriptor_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(own_key_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            boxed_kind_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_kind_local));
        function.instruction(&Instruction::I64Const(BOXED_PRIMITIVE_KIND_STRING as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("length")));
        function.instruction(&Instruction::LocalSet(boxed_kind_local));
        self.emit_string_payload_equality_i32(own_key_payload_local, boxed_kind_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::Else);
        self.emit_known_array_index_from_property_key(
            own_key_payload_local,
            boxed_string_index_local,
            boxed_string_index_found_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_string_index_found_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            arg_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            boxed_string_payload_local,
            function,
        );
        self.emit_unpack_string_payload(
            boxed_string_payload_local,
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            function,
        );
        self.emit_utf16_code_unit_len_from_utf8_locals(
            boxed_string_offset_local,
            boxed_string_byte_len_local,
            boxed_string_len_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(boxed_string_index_local));
        function.instruction(&Instruction::LocalGet(boxed_string_len_local));
        function.instruction(&Instruction::I64LtU);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(boxed_string_exotic_key_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_direct_js_call(
            &define_property_meta,
            None,
            &[
                (arg_payload_local, arg_tag_local),
                (own_key_payload_local, own_key_tag_local),
                (
                    selected_descriptor_payload_local,
                    descriptor_object_tag_local,
                ),
            ],
            define_result_payload_local,
            define_result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(define_result_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(define_result_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(define_result_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "Object.freeze could not make an own property non-configurable and non-writable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(own_key_index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(own_key_index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(boxed_string_exotic_key_local);
        self.release_temp_local(boxed_string_index_found_local);
        self.release_temp_local(boxed_string_index_local);
        self.release_temp_local(boxed_string_len_local);
        self.release_temp_local(boxed_string_byte_len_local);
        self.release_temp_local(boxed_string_offset_local);
        self.release_temp_local(boxed_string_payload_local);
        self.release_temp_local(boxed_kind_local);
        self.release_temp_local(define_result_tag_local);
        self.release_temp_local(define_result_payload_local);
        self.release_temp_local(accessor_descriptor_local);
        self.release_temp_local(descriptor_has_set_local);
        self.release_temp_local(descriptor_has_get_local);
        self.release_temp_local(descriptor_field_key_local);
        self.release_temp_local(descriptor_object_tag_local);
        self.release_temp_local(selected_descriptor_payload_local);
        self.release_temp_local(frozen_data_descriptor_payload_local);
        self.release_temp_local(configurable_descriptor_payload_local);
        self.release_temp_local(current_descriptor_tag_local);
        self.release_temp_local(current_descriptor_payload_local);
        self.release_temp_local(own_key_tag_local);
        self.release_temp_local(own_key_payload_local);
        self.release_temp_local(own_key_index_local);
        self.release_temp_local(own_keys_length_local);
        self.release_temp_local(own_keys_tag_local);
        self.release_temp_local(own_keys_payload_local);
        self.release_temp_local(prevent_extensions_result_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prevent_extensions_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let result_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_prevent_extensions(
            ObjectPreventExtensionsRequest::new(
                PreventExtensionsTraversalTargetLocals::new(arg_payload_local, arg_tag_local),
                PreventExtensionsResultLocal::new(result_local),
            ),
            function,
        )?;
        function.instruction(&Instruction::LocalGet(result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_to_active_handler(
            TYPE_ERROR_NAME,
            "Proxy preventExtensions trap returned false",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(result_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_proto_getter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.__proto__ getter receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.__proto__ getter receiver",
            )
        })?;
        if self
            .runtime_bootstrap_plan
            .should_initialize_standard_builtin(StandardBuiltinId::ProxyConstructor)
        {
            self.emit_object_get_prototype_of(
                receiver_payload_local,
                receiver_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        } else {
            self.emit_object_get_prototype_of_without_proxy(
                receiver_payload_local,
                receiver_tag_local,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
        }
        Ok(())
    }

    pub(super) fn compile_object_prototype_proto_setter_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.__proto__ setter receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.__proto__ setter receiver",
            )
        })?;
        let proto_payload_local = self.reserve_temp_local();
        let proto_tag_local = self.reserve_temp_local();
        let set_result_local = self.reserve_temp_local();
        self.emit_builtin_arg_to_locals(0, proto_payload_local, proto_tag_local, function);

        self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.prototype.__proto__ setter called on null or undefined",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        function.instruction(&Instruction::LocalGet(proto_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        self.emit_is_heap_object_like_tag_i32(proto_tag_local, function);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_set_prototype_of_i32(
            receiver_payload_local,
            receiver_tag_local,
            proto_payload_local,
            proto_tag_local,
            set_result_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(set_result_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "Object.prototype.__proto__ setter could not set prototype",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(set_result_local);
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_is_prototype_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.isPrototypeOf receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.isPrototypeOf receiver",
            )
        })?;
        let arg_payload_local = self.reserve_temp_local();
        let arg_tag_local = self.reserve_temp_local();
        let object_payload_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        let current_proto_local = self.reserve_temp_local();
        let current_proto_tag_local = self.reserve_temp_local();
        let next_proto_local = self.reserve_temp_local();
        let next_proto_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            object_payload_local,
            object_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(arg_payload_local));
        function.instruction(&Instruction::LocalSet(current_proto_local));
        function.instruction(&Instruction::LocalGet(arg_tag_local));
        function.instruction(&Instruction::LocalSet(current_proto_tag_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(current_proto_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.emit_object_get_prototype_of(
            current_proto_local,
            current_proto_tag_local,
            next_proto_local,
            next_proto_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_tagged_payload_same_value_i32(
            object_tag_local,
            object_payload_local,
            next_proto_tag_local,
            next_proto_local,
            function,
        )?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(next_proto_local));
        function.instruction(&Instruction::LocalSet(current_proto_local));
        function.instruction(&Instruction::LocalGet(next_proto_tag_local));
        function.instruction(&Instruction::LocalSet(current_proto_tag_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(next_proto_tag_local);
        self.release_temp_local(next_proto_local);
        self.release_temp_local(current_proto_tag_local);
        self.release_temp_local(current_proto_local);
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_payload_local);
        self.release_temp_local(arg_tag_local);
        self.release_temp_local(arg_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_to_string_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.toString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.toString receiver",
            )
        })?;
        let tag_payload_local = self.reserve_temp_local();
        let is_array_local = self.reserve_temp_local();
        let brand_local = self.reserve_temp_local();
        let to_string_tag_key_local = self.reserve_temp_local();
        let custom_tag_payload_local = self.reserve_temp_local();
        let custom_tag_tag_local = self.reserve_temp_local();
        let prefix_local = self.reserve_temp_local();
        let suffix_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Object]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));

        for (kind, tag) in [
            (ValueKind::Undefined, "[object Undefined]"),
            (ValueKind::Null, "[object Null]"),
            (ValueKind::Boolean, "[object Boolean]"),
            (ValueKind::Number, "[object Number]"),
            (ValueKind::String, "[object String]"),
            (ValueKind::Symbol, "[object Symbol]"),
            (ValueKind::Object, "[object Object]"),
            (ValueKind::Array, "[object Array]"),
            (ValueKind::Function, "[object Function]"),
            (ValueKind::Arguments, "[object Arguments]"),
            (ValueKind::BigInt, "[object BigInt]"),
        ] {
            function.instruction(&Instruction::LocalGet(receiver_tag_local));
            function.instruction(&Instruction::I64Const(kind.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(tag)));
            function.instruction(&Instruction::LocalSet(tag_payload_local));
            function.instruction(&Instruction::End);
        }

        self.emit_is_array_i64(
            receiver_payload_local,
            receiver_tag_local,
            is_array_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(is_array_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Array]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);

        self.emit_is_callable_i32(receiver_tag_local, receiver_payload_local, function)?;
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Function]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            brand_local,
            function,
        );
        for (boxed_kind, tag) in [
            (BOXED_PRIMITIVE_KIND_BOOLEAN, "[object Boolean]"),
            (BOXED_PRIMITIVE_KIND_NUMBER, "[object Number]"),
            (BOXED_PRIMITIVE_KIND_STRING, "[object String]"),
            (BOXED_PRIMITIVE_KIND_SYMBOL, "[object Symbol]"),
            (BOXED_PRIMITIVE_KIND_BIGINT, "[object BigInt]"),
        ] {
            function.instruction(&Instruction::LocalGet(brand_local));
            function.instruction(&Instruction::I64Const(boxed_kind as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(self.strings.payload(tag)));
            function.instruction(&Instruction::LocalSet(tag_payload_local));
            function.instruction(&Instruction::End);
        }
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_ERROR as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Error]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_DATE as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object Date]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(brand_local));
        function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_REGEXP as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.payload("[object RegExp]"),
        ));
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(to_string_tag_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            to_string_tag_key_local,
            custom_tag_payload_local,
            custom_tag_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(custom_tag_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("[object ")));
        function.instruction(&Instruction::LocalSet(prefix_local));
        self.emit_concat_string_payloads_local(prefix_local, custom_tag_payload_local, function)?;
        function.instruction(&Instruction::LocalSet(prefix_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("]")));
        function.instruction(&Instruction::LocalSet(suffix_local));
        self.emit_concat_string_payloads_local(prefix_local, suffix_local, function)?;
        function.instruction(&Instruction::LocalSet(tag_payload_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(tag_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.release_temp_local(suffix_local);
        self.release_temp_local(prefix_local);
        self.release_temp_local(custom_tag_tag_local);
        self.release_temp_local(custom_tag_payload_local);
        self.release_temp_local(to_string_tag_key_local);
        self.release_temp_local(brand_local);
        self.release_temp_local(is_array_local);
        self.release_temp_local(tag_payload_local);
        Ok(())
    }

    pub(super) fn compile_object_prototype_value_of_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.valueOf receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Object.prototype.valueOf receiver",
            )
        })?;
        self.emit_value_to_current_function_realm_object_locals(
            receiver_payload_local,
            receiver_tag_local,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        Ok(())
    }
}
