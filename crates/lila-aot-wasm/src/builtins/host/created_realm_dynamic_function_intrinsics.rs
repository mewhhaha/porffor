use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_initialize_created_realm_dynamic_function_intrinsics(
        &mut self,
        realm_record: RealmRecordLocal,
        realm_functions: &RealmFunctionMaterializationContext,
        function_constructor_local: u32,
        object_prototype_local: u32,
        iterator_prototype_local: u32,
        type_error_prototype_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let intrinsics_local = self.reserve_temp_local();
        let function_prototype_local = self.reserve_temp_local();
        let generator_prototype_local = self.reserve_temp_local();
        let async_iterator_prototype_local = self.reserve_temp_local();
        let async_generator_prototype_local = self.reserve_temp_local();
        let derived_prototype_local = self.reserve_temp_local();
        let constructor_local = self.reserve_temp_local();
        let method_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            realm_record.index(),
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
            function_prototype_local,
            function,
        );
        for (destination, parent, slot) in [
            (
                generator_prototype_local,
                iterator_prototype_local,
                NonArrayRealmIntrinsicSlot::GeneratorPrototype,
            ),
            (
                async_iterator_prototype_local,
                object_prototype_local,
                NonArrayRealmIntrinsicSlot::AsyncIteratorPrototype,
            ),
            (
                async_generator_prototype_local,
                async_iterator_prototype_local,
                NonArrayRealmIntrinsicSlot::AsyncGeneratorPrototype,
            ),
        ] {
            self.emit_alloc_plain_object_with_prototype(Some(parent), None, function)?;
            function.instruction(&Instruction::LocalSet(destination));
            self.emit_store_non_array_realm_intrinsic(
                realm_record.index(),
                slot,
                destination,
                function,
            );
        }

        for (builtin, constructor_slot, prototype_slot, instance_prototype) in [
            (
                HostBuiltinId::GeneratorFunctionConstructor,
                NonArrayRealmIntrinsicSlot::GeneratorFunctionConstructor,
                NonArrayRealmIntrinsicSlot::GeneratorFunctionPrototype,
                Some(generator_prototype_local),
            ),
            (
                HostBuiltinId::AsyncFunctionConstructor,
                NonArrayRealmIntrinsicSlot::AsyncFunctionConstructor,
                NonArrayRealmIntrinsicSlot::AsyncFunctionPrototype,
                None,
            ),
            (
                HostBuiltinId::AsyncGeneratorFunctionConstructor,
                NonArrayRealmIntrinsicSlot::AsyncGeneratorFunctionConstructor,
                NonArrayRealmIntrinsicSlot::AsyncGeneratorFunctionPrototype,
                Some(async_generator_prototype_local),
            ),
        ] {
            // Every derived Function prototype is an ordinary object whose
            // parent is the realm's callable %Function.prototype% (ECMA-262 27.4.3).
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_alloc_plain_object_with_prototype_and_tag(
                Some(function_prototype_local),
                Some(tag_local),
                None,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(derived_prototype_local));
            self.emit_store_non_array_realm_intrinsic(
                realm_record.index(),
                prototype_slot,
                derived_prototype_local,
                function,
            );
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!("missing {} metadata", builtin.debug_name()))
                })?;
            self.emit_function_value_payload_in_realm(
                &meta,
                realm_functions,
                constructor_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                constructor_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                function_constructor_local,
                function,
            );
            self.emit_set_function_prototype_data_with_flags(
                constructor_local,
                derived_prototype_local,
                false,
                false,
                false,
                false,
                function,
            )?;
            self.emit_store_non_array_realm_intrinsic(
                realm_record.index(),
                constructor_slot,
                constructor_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_local_data_with_flags(
                derived_prototype_local,
                "constructor",
                constructor_local,
                tag_local,
                false,
                false,
                true,
                function,
            )?;
            if let Some(instance_prototype) = instance_prototype {
                function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                for (owner, key, value) in [
                    (derived_prototype_local, "prototype", instance_prototype),
                    (instance_prototype, "constructor", derived_prototype_local),
                ] {
                    self.emit_object_define_local_data_with_flags(
                        owner, key, value, tag_local, false, false, true, function,
                    )?;
                }
            }
            function.instruction(&Instruction::I64Const(
                self.strings.payload(builtin.as_str()),
            ));
            function.instruction(&Instruction::LocalSet(value_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_local_data_with_flags(
                derived_prototype_local,
                "Symbol.toStringTag",
                value_local,
                tag_local,
                false,
                false,
                true,
                function,
            )?;
        }

        for (prototype, name, builtin) in [
            (
                generator_prototype_local,
                "next",
                StandardBuiltinId::GeneratorPrototypeNext,
            ),
            (
                generator_prototype_local,
                "return",
                StandardBuiltinId::GeneratorPrototypeReturn,
            ),
            (
                generator_prototype_local,
                "throw",
                StandardBuiltinId::GeneratorPrototypeThrow,
            ),
            (
                async_generator_prototype_local,
                "next",
                StandardBuiltinId::AsyncGeneratorPrototypeNext,
            ),
            (
                async_generator_prototype_local,
                "return",
                StandardBuiltinId::AsyncGeneratorPrototypeReturn,
            ),
            (
                async_generator_prototype_local,
                "throw",
                StandardBuiltinId::AsyncGeneratorPrototypeThrow,
            ),
            (
                async_iterator_prototype_local,
                "Symbol.asyncIterator",
                StandardBuiltinId::ArrayIteratorIdentity,
            ),
            (
                async_iterator_prototype_local,
                "Symbol.asyncDispose",
                StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose,
            ),
        ] {
            let mut meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!("missing {} metadata", builtin.debug_name()))
                })?;
            if builtin == StandardBuiltinId::ArrayIteratorIdentity {
                meta.name = "[Symbol.asyncIterator]".to_string();
                meta.to_string_value =
                    "function [Symbol.asyncIterator]() { [native code] }".to_string();
            }
            self.emit_function_value_payload_in_realm(
                &meta,
                realm_functions,
                method_local,
                function,
            )?;
            self.store_i64_local_at_offset(
                method_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                method_local,
                function,
            );
            self.store_i64_local_at_offset(
                method_local,
                HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,
                type_error_prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_local_data(prototype, name, method_local, tag_local, function)?;
        }
        for (prototype, name) in [
            (generator_prototype_local, "Generator"),
            (async_generator_prototype_local, "AsyncGenerator"),
        ] {
            function.instruction(&Instruction::I64Const(self.strings.payload(name)));
            function.instruction(&Instruction::LocalSet(value_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            self.emit_object_define_local_data_with_flags(
                prototype,
                "Symbol.toStringTag",
                value_local,
                tag_local,
                false,
                false,
                true,
                function,
            )?;
        }

        self.release_temp_local(tag_local);
        self.release_temp_local(value_local);
        self.release_temp_local(method_local);
        self.release_temp_local(constructor_local);
        self.release_temp_local(derived_prototype_local);
        self.release_temp_local(async_generator_prototype_local);
        self.release_temp_local(async_iterator_prototype_local);
        self.release_temp_local(generator_prototype_local);
        self.release_temp_local(function_prototype_local);
        self.release_temp_local(intrinsics_local);
        Ok(())
    }
}
