use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_class_inferred_name(
        &mut self,
        class: &ClassDefinitionIr,
        constructor: &crate::gc_types::GcLocal<crate::gc_types::FunctionObject>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        use crate::gc_types::{FunctionContext, FunctionContextSchema, StoredValue, ValueArray};
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        match &class.name_inference {
            ClassNameInferenceIr::None => {
                value.clear(function);
                return Ok(());
            }
            ClassNameInferenceIr::PropertyKeyBinding(name) => {
                let storage = self.lookup_binding(name).ok_or_else(|| {
                    EmitError::unsupported(
                        "compiler invariant: inferred class name has no captured property key",
                    )
                })?;
                self.read_binding_to_locals(storage, &value, function)?;
            }
            ClassNameInferenceIr::FieldInitializer(name) => {
                let meta = self.current_function_meta().ok_or_else(|| {
                    EmitError::unsupported(
                        "compiler invariant: class field named evaluation has no source owner",
                    )
                })?;
                if !matches!(
                    meta.class_element_execution_kind,
                    ClassElementExecutionKind::InstanceFieldInitializer
                        | ClassElementExecutionKind::StaticFieldInitializer
                ) {
                    return Err(EmitError::unsupported(
                        "compiler invariant: named class is outside its field initializer",
                    ));
                }
                match name {
                    ClassFieldNameIr::Static(name) => {
                        let string = schema.reserve_gc_local(function).initialize(
                            self.emit_interned_string_reference(name, function)?,
                            function,
                        );
                        value.set_reference(&string, schema, function);
                        string.clear(function);
                    }
                    ClassFieldNameIr::Computed(slot) => {
                        let context = self
                            .body_entry_locals()
                            .and_then(|entry| entry.function_context())
                            .expect("source initializer owns complete class captures");
                        let keys = schema.reserve_gc_local(function).initialize(
                            schema
                                .struct_type::<FunctionContext>()
                                .field(FunctionContextSchema::FIELD_KEYS)
                                .read(context, schema, function)
                                .reference()
                                .require_non_null(function),
                            function,
                        );
                        let index = schema.reserve_i32_local(function);
                        function.instruction(&Instruction::I32Const(*slot as i32));
                        index.store(function);
                        let stored = schema.reserve_gc_local(function).initialize(
                            schema
                                .array_type::<ValueArray>()
                                .read(&keys, index, schema, function)
                                .reference(),
                            function,
                        );
                        schema
                            .struct_type::<StoredValue>()
                            .read_into(&stored, &value, schema, function);
                        stored.clear(function);
                        schema.release_i32_local(index, function);
                        keys.clear(function);
                    }
                }
            }
        }
        let key = self.emit_value_to_property_key_locals(&value, function)?;
        self.emit_set_function_name(constructor, &key, FunctionNamePrefix::None, function)?;
        key.clear(function);
        value.clear(function);
        Ok(())
    }

    pub(super) fn emit_class_element_definition(
        &mut self,
        definition: &ClassElementDefinitionIr,
        constructor: &crate::gc_types::GcLocal<crate::gc_types::FunctionObject>,
        prototype: &crate::gc_types::GcLocal<crate::gc_types::OrdinaryObject>,
        private_environment: &crate::gc_types::GcLocal<
            crate::gc_types::PrivateEnvironment,
            crate::gc_types::Nullable,
        >,
        field_keys: &crate::gc_types::GcLocal<
            crate::gc_types::ValueArray,
            crate::gc_types::Nullable,
        >,
        instance_methods: &crate::gc_types::GcLocal<
            crate::gc_types::PrivateElementTable,
            crate::gc_types::Nullable,
        >,
        static_methods: &crate::gc_types::GcLocal<
            crate::gc_types::PrivateElementTable,
            crate::gc_types::Nullable,
        >,
        instance_slots: &BTreeMap<PrivateNameId, u32>,
        static_slots: &BTreeMap<PrivateNameId, u32>,
        result: &crate::gc_types::CompletionLocals,
        finish: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        use class_private_definitions::ClassMethodPlan;
        let schema = self.runtime_schema();
        if let ClassElementDefinitionIr::ComputedFieldKey { slot, key } = definition {
            let key = self.compile_object_key_to_locals(key, function)?;
            self.emit_store_class_field_key(field_keys, *slot, key.value(), function);
            key.clear(function);
            return Ok(());
        }
        let (placement, key, private_name_id, functions) = match definition {
            ClassElementDefinitionIr::PublicMethod(method) => (
                method.placement,
                Some(self.compile_object_key_to_locals(&method.key, function)?),
                None,
                ClassMethodPlan::Single {
                    function: &method.function_id,
                    kind: method.kind,
                },
            ),
            ClassElementDefinitionIr::PrivateMethod(method) => (
                method.placement,
                None,
                Some(method.private_name_id),
                ClassMethodPlan::Single {
                    function: &method.function_id,
                    kind: method.kind,
                },
            ),
            ClassElementDefinitionIr::AutoAccessor(accessor) => {
                if let Some(computed) = &accessor.computed_key {
                    let ClassFieldKeyIr::ComputedPublic(slot) = accessor.key else {
                        return Err(EmitError::unsupported(
                            "compiler invariant: auto-accessor computed key has no cache slot",
                        ));
                    };
                    let key = self.compile_object_key_to_locals(computed, function)?;
                    self.emit_store_class_field_key(field_keys, slot, key.value(), function);
                    key.clear(function);
                }
                let (key, private) = match accessor.key {
                    ClassFieldKeyIr::Private(id) => (None, Some(id)),
                    ClassFieldKeyIr::Public(_) | ClassFieldKeyIr::ComputedPublic(_) => (
                        Some(self.emit_class_public_field_key(
                            &accessor.key,
                            field_keys,
                            function,
                        )?),
                        None,
                    ),
                };
                (
                    accessor.placement,
                    key,
                    private,
                    ClassMethodPlan::Accessor(&accessor.functions),
                )
            }
            ClassElementDefinitionIr::ComputedFieldKey { .. } => {
                unreachable!("computed keys are handled before method allocation")
            }
        };
        let private_name = private_name_id
            .map(|id| {
                self.emit_private_name_token_from_environment(id, private_environment, function)
            })
            .transpose()?;
        let private_key = if let Some(name) = &private_name {
            let description = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<crate::gc_types::PrivateName>()
                    .field(crate::gc_types::PrivateNameSchema::DESCRIPTION)
                    .read(name, schema, function)
                    .reference()
                    .require_non_null(function),
                function,
            );
            let key =
                crate::operations::PropertyKeyLocals::from_string(schema, &description, function);
            description.clear(function);
            Some(key)
        } else {
            None
        };
        let name_key = key
            .as_ref()
            .or(private_key.as_ref())
            .expect("every defined method has a property/private name");
        let home = match placement {
            ClassMethodPlacementIr::Instance => FunctionHomeObject::Object(prototype),
            ClassMethodPlacementIr::Static => FunctionHomeObject::Constructor(constructor),
        };
        let methods = self.emit_class_method_values(
            functions,
            home,
            private_environment,
            field_keys,
            name_key,
            function,
        )?;
        let definition = methods.definition();
        if let Some(id) = private_name_id {
            let (table, slots) = match placement {
                ClassMethodPlacementIr::Instance => (instance_methods, instance_slots),
                ClassMethodPlacementIr::Static => (static_methods, static_slots),
            };
            let table = schema.reserve_gc_local(function).initialize(
                table.load(schema, function).require_non_null(function),
                function,
            );
            let index = schema.reserve_i32_local(function);
            function.instruction(&Instruction::I32Const(slots[&id] as i32));
            index.store(function);
            self.emit_class_private_definition(
                &table,
                index,
                private_name
                    .as_ref()
                    .expect("private definition owns its name"),
                definition,
                function,
            );
            schema.release_i32_local(index, function);
            table.clear(function);
        } else {
            let receiver = schema.reserve_value_local(function);
            match placement {
                ClassMethodPlacementIr::Instance => {
                    receiver.set_reference(prototype, schema, function)
                }
                ClassMethodPlacementIr::Static => {
                    receiver.set_reference(constructor, schema, function)
                }
            }
            self.emit_define_class_public_member(
                &receiver, name_key, definition, result, function,
            )?;
            receiver.clear(function);
        }
        methods.clear(function);
        if let Some(key) = key {
            key.clear(function);
        }
        if let Some(key) = private_key {
            key.clear(function);
        }
        if let Some(name) = private_name {
            name.clear(function);
        }
        self.emit_class_adopt_abrupt(result, finish, function);
        Ok(())
    }

    fn emit_define_class_public_member(
        &mut self,
        receiver: &crate::gc_types::ValueLocals,
        key: &crate::operations::PropertyKeyLocals,
        definition: class_private_definitions::ClassPrivateDefinition<'_>,
        result: &crate::gc_types::CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        use class_private_definitions::ClassPrivateDefinition;
        let schema = self.runtime_schema();
        let descriptor = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(None, function)?,
            function,
        );
        let fields: &[(&str, &crate::gc_types::ValueLocals)] = match definition {
            ClassPrivateDefinition::Method(value) => &[("value", value)],
            ClassPrivateDefinition::Getter(value) => &[("get", value)],
            ClassPrivateDefinition::Setter(value) => &[("set", value)],
            ClassPrivateDefinition::Accessor { getter, setter } => {
                &[("get", getter), ("set", setter)]
            }
        };
        let data = matches!(definition, ClassPrivateDefinition::Method(_));
        for (name, value) in fields {
            let field = self.emit_function_string_key(name, function)?;
            self.emit_object_append_data_property_with_flags(
                &descriptor,
                &field,
                value,
                true,
                true,
                true,
                function,
            )?;
            field.clear(function);
        }
        let flag = schema.reserve_value_local(function);
        for (name, enabled) in [
            ("enumerable", false),
            ("configurable", true),
            ("writable", true),
        ] {
            if name == "writable" && !data {
                continue;
            }
            flag.set_scalar(crate::gc_types::ScalarValue::Boolean(enabled), function);
            let field = self.emit_function_string_key(name, function)?;
            self.emit_object_append_data_property_with_flags(
                &descriptor,
                &field,
                &flag,
                true,
                true,
                true,
                function,
            )?;
            field.clear(function);
        }
        flag.clear(function);
        let value = schema.reserve_value_local(function);
        value.set_reference(&descriptor, schema, function);
        let define = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("class definitions require planned ObjectDefineProperty")
            })?;
        self.emit_direct_js_call(
            &define,
            None,
            &[receiver, key.value(), &value],
            result,
            function,
        )?;
        value.clear(function);
        descriptor.clear(function);
        Ok(())
    }

    fn emit_store_class_field_key(
        &mut self,
        keys: &crate::gc_types::GcLocal<crate::gc_types::ValueArray, crate::gc_types::Nullable>,
        slot: u32,
        value: &crate::gc_types::ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let index = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(slot as i32));
        index.store(function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<crate::gc_types::StoredValue>()
                .from_value(value, function),
            function,
        );
        schema.array_type::<crate::gc_types::ValueArray>().write(
            keys,
            index,
            crate::gc_types::GcOperand::reference(&stored, schema),
            schema,
            function,
        );
        stored.clear(function);
        schema.release_i32_local(index, function);
    }

    fn emit_class_public_field_key(
        &mut self,
        key: &ClassFieldKeyIr,
        keys: &crate::gc_types::GcLocal<crate::gc_types::ValueArray, crate::gc_types::Nullable>,
        function: &mut Function,
    ) -> Result<crate::operations::PropertyKeyLocals, EmitError> {
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(function);
        match key {
            ClassFieldKeyIr::Public(name) => {
                let string = schema.reserve_gc_local(function).initialize(
                    self.emit_interned_string_reference(name, function)?,
                    function,
                );
                value.set_reference(&string, schema, function);
                string.clear(function);
            }
            ClassFieldKeyIr::ComputedPublic(slot) => {
                let index = schema.reserve_i32_local(function);
                function.instruction(&Instruction::I32Const(*slot as i32));
                index.store(function);
                let stored = schema.reserve_gc_local(function).initialize(
                    schema
                        .array_type::<crate::gc_types::ValueArray>()
                        .read(keys, index, schema, function)
                        .reference(),
                    function,
                );
                schema
                    .struct_type::<crate::gc_types::StoredValue>()
                    .read_into(&stored, &value, schema, function);
                stored.clear(function);
                schema.release_i32_local(index, function);
            }
            ClassFieldKeyIr::Private(_) => {
                return Err(EmitError::unsupported(
                    "compiler invariant: private class name used as a public property key",
                ))
            }
        }
        // Retained computed keys are already String/Symbol, so this cannot
        // re-run user coercion on each receiver or field initializer.
        let result = self.emit_value_to_property_key_locals(&value, function)?;
        value.clear(function);
        Ok(result)
    }

    pub(super) fn emit_class_static_element(
        &mut self,
        element: &ClassStaticElementIr,
        context: &crate::gc_types::GcLocal<crate::gc_types::FunctionContext>,
        receiver: &crate::gc_types::ValueLocals,
        private_environment: &crate::gc_types::GcLocal<
            crate::gc_types::PrivateEnvironment,
            crate::gc_types::Nullable,
        >,
        keys: &crate::gc_types::GcLocal<crate::gc_types::ValueArray, crate::gc_types::Nullable>,
        result: &crate::gc_types::CompletionLocals,
        finish: ControlTarget,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (initializer, expected) = match element {
            ClassStaticElementIr::Field(field) => (
                field.init_function_id.as_ref(),
                ClassElementExecutionKind::StaticFieldInitializer,
            ),
            ClassStaticElementIr::AutoAccessorBacking(accessor) => (
                accessor.init_function_id.as_ref(),
                ClassElementExecutionKind::StaticFieldInitializer,
            ),
            ClassStaticElementIr::Block(block) => (
                Some(&block.function_id),
                ClassElementExecutionKind::StaticBlock,
            ),
        };
        if let Some(id) = initializer {
            let meta = self.functions.get(id).cloned().ok_or_else(|| {
                EmitError::unsupported("class static element has no planned source body")
            })?;
            if meta.class_element_execution_kind != expected {
                return Err(EmitError::unsupported(
                    "compiler invariant: static element has the wrong callable role",
                ));
            }
            self.emit_direct_class_element_js_call(
                &meta,
                context,
                Some(receiver),
                &[],
                result,
                function,
            )?;
        } else {
            result.initialize(function);
        }
        self.emit_class_adopt_abrupt(result, finish, function);
        match element {
            ClassStaticElementIr::Block(_) => {}
            ClassStaticElementIr::AutoAccessorBacking(accessor) => {
                let value = self.runtime_schema().reserve_value_local(function);
                value.copy_from(result.value(), function);
                self.emit_private_field_define(
                    receiver,
                    private_environment,
                    accessor.backing_name.private_name_id(),
                    &value,
                    result,
                    function,
                )?;
                value.clear(function);
                self.emit_class_adopt_abrupt(result, finish, function);
            }
            ClassStaticElementIr::Field(field) => {
                let value = self.runtime_schema().reserve_value_local(function);
                value.copy_from(result.value(), function);
                match &field.key {
                    ClassFieldKeyIr::Private(id) => {
                        self.emit_private_field_define(
                            receiver,
                            private_environment,
                            *id,
                            &value,
                            result,
                            function,
                        )?;
                    }
                    ClassFieldKeyIr::Public(_) | ClassFieldKeyIr::ComputedPublic(_) => {
                        let key = self.emit_class_public_field_key(&field.key, keys, function)?;
                        self.emit_define_public_class_field(
                            receiver, &key, &value, result, function,
                        )?;
                        key.clear(function);
                    }
                }
                value.clear(function);
                self.emit_class_adopt_abrupt(result, finish, function);
            }
        }
        Ok(())
    }
}
