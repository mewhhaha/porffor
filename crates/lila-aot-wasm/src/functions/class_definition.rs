use super::*;
use lila_ir::{ClassEvaluationPrefixIr, ResumableClassDefinitionIr};

impl FunctionBuilder<'_> {
    pub(super) fn emit_define_public_class_field(
        &mut self,
        receiver: &crate::gc_types::ValueLocals,
        key: &crate::operations::PropertyKeyLocals,
        value: &crate::gc_types::ValueLocals,
        result: &crate::gc_types::CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // CreateDataPropertyOrThrow must observe the receiver's actual
        // [[DefineOwnProperty]], including Proxy and integer-indexed exotics.
        let schema = self.runtime_schema();
        let descriptor = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(None, function)?,
            function,
        );
        let value_key = self.emit_function_string_key("value", function)?;
        self.emit_object_append_data_property_with_flags(
            &descriptor,
            &value_key,
            value,
            true,
            true,
            true,
            function,
        )?;
        value_key.clear(function);
        let flag = schema.reserve_value_local(function);
        flag.set_scalar(crate::gc_types::ScalarValue::Boolean(true), function);
        for name in ["writable", "enumerable", "configurable"] {
            let property = self.emit_function_string_key(name, function)?;
            self.emit_object_append_data_property_with_flags(
                &descriptor,
                &property,
                &flag,
                true,
                true,
                true,
                function,
            )?;
            property.clear(function);
        }
        flag.clear(function);
        let carrier = schema.reserve_value_local(function);
        carrier.set_reference(&descriptor, schema, function);
        let define = self
            .functions
            .get(&StandardBuiltinId::ObjectDefineProperty.function_id())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported("class fields require planned ObjectDefineProperty")
            })?;
        self.emit_direct_js_call(
            &define,
            None,
            &[receiver, key.value(), &carrier],
            result,
            function,
        )?;
        carrier.clear(function);
        descriptor.clear(function);
        Ok(())
    }

    pub(crate) fn compile_class_definition_payload(
        &mut self,
        class: &ClassDefinitionIr,
        output: &crate::gc_types::ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let result = self.runtime_schema().reserve_completion(function);
        self.emit_class_evaluation(class, None, &result, function)?;
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(result.value(), function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result.clear(function);
        self.emit_propagate_current_throw_if_needed(function);
        Ok(())
    }

    pub(crate) fn compile_resumable_class_definition(
        &mut self,
        plan: &ResumableClassDefinitionIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for name in [plan.constructor_binding(), plan.completion_binding()] {
            if self.owned_env_slot(name).is_none() {
                return Err(EmitError::unsupported(
                    "compiler invariant: suspended class storage is not activation-owned",
                ));
            }
            self.allocate_binding(
                name.to_owned(),
                BindingMode::Let,
                ValueKind::Dynamic,
                function,
            );
        }
        self.emit_class_resume_state(function)?;
        function.instruction(&Instruction::I32Const(plan.entry_state() as i32));
        function.instruction(&Instruction::I32GeU);
        self.emit_class_resume_state(function)?;
        function.instruction(&Instruction::I32Const(plan.exit_state() as i32));
        function.instruction(&Instruction::I32LeU);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        let result = self.runtime_schema().reserve_completion(function);
        self.emit_class_evaluation(plan.class(), Some(plan), &result, function)?;
        result.clear(function);
        // The class has restored both environments and retired its capture.
        // Only this active class may dispatch its complete abrupt result.
        self.emit_dispatch_current_completion(function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_class_evaluation(
        &mut self,
        class: &ClassDefinitionIr,
        plan: Option<&ResumableClassDefinitionIr>,
        result: &crate::gc_types::CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        use crate::gc_types::{
            Environment, EnvironmentSchema, FunctionContext, FunctionContextSchema, FunctionObject,
            FunctionObjectSchema, GcOperand, Nullable, OrdinaryObject, OrdinaryObjectSchema,
            PrivateElementTable, PrivateEnvironment, StoredValue, ValueArray,
        };
        let schema = self.runtime_schema();
        let constructor_meta = self
            .functions
            .get(&class.constructor_function_id)
            .cloned()
            .ok_or_else(|| EmitError::unsupported("class has no planned source constructor"))?;
        let instance_slots =
            Self::class_private_definition_slots(class, ClassMethodPlacementIr::Instance);
        let static_slots =
            Self::class_private_definition_slots(class, ClassMethodPlacementIr::Static);
        let field_key_count = class
            .element_plan
            .definitions
            .iter()
            .filter_map(|definition| match definition {
                ClassElementDefinitionIr::ComputedFieldKey { slot, .. } => Some(*slot + 1),
                ClassElementDefinitionIr::AutoAccessor(accessor) => match accessor.key {
                    ClassFieldKeyIr::ComputedPublic(slot) => Some(slot + 1),
                    ClassFieldKeyIr::Public(_) | ClassFieldKeyIr::Private(_) => None,
                },
                ClassElementDefinitionIr::PublicMethod(_)
                | ClassElementDefinitionIr::PrivateMethod(_) => None,
            })
            .max()
            .unwrap_or(0);
        let constructor = schema
            .reserve_gc_local::<FunctionObject, Nullable>(function)
            .initialize_null(schema, function);
        let prototype = schema
            .reserve_gc_local::<OrdinaryObject, Nullable>(function)
            .initialize_null(schema, function);
        let context = schema
            .reserve_gc_local::<FunctionContext, Nullable>(function)
            .initialize_null(schema, function);
        let private_environment = schema
            .reserve_gc_local::<PrivateEnvironment, Nullable>(function)
            .initialize_null(schema, function);
        let field_keys = schema
            .reserve_gc_local::<ValueArray, Nullable>(function)
            .initialize_null(schema, function);
        let instance_methods = schema
            .reserve_gc_local::<PrivateElementTable, Nullable>(function)
            .initialize_null(schema, function);
        let static_methods = schema
            .reserve_gc_local::<PrivateElementTable, Nullable>(function)
            .initialize_null(schema, function);
        let outer_environment = schema
            .reserve_gc_local::<Environment, Nullable>(function)
            .initialize(self.current_environment().load(schema, function), function);
        let outer_private_environment = schema
            .reserve_gc_local::<PrivateEnvironment, Nullable>(function)
            .initialize(
                self.current_private_environment().load(schema, function),
                function,
            );
        let saved_completion = schema.reserve_completion(function);
        saved_completion.copy_from(self.completion(), function);
        result.initialize(function);
        let class_environment_capture = plan
            .and_then(|plan| plan.name_environment_binding())
            .map(|name| self.class_environment_capture(name))
            .transpose()?;
        self.completion().initialize(function);
        let finish = self.open_frame(ControlFrameKind::Block, function);
        self.throw_handler_stack.push(finish);
        if plan.is_some() {
            // Injected Return must restore a suspended class just as Throw
            // does before reaching an enclosing finally or iterator close.
            self.finally_stack.push(finish);
        }
        if let Some(plan) = plan {
            self.open_class_evaluation_state(plan.entry_state(), function)?;
            let storage = self
                .lookup_binding(plan.completion_binding())
                .expect("owned class completion binding was registered");
            self.write_binding_from_locals(storage, saved_completion.value(), function);
            self.close_class_evaluation_state(function);
        }
        if let Some(name_binding) = &class.name_binding {
            self.push_scope();
            if let Some(plan) = plan {
                self.open_class_evaluation_state(plan.entry_state(), function)?;
                self.emit_allocate_lexical_environment_record(&name_binding.environment, function)?;
                let capture = class_environment_capture
                    .as_ref()
                    .expect("named suspended class owns an environment capture");
                let named = schema
                    .reserve_gc_local::<Environment, Nullable>(function)
                    .initialize(self.current_environment().load(schema, function), function);
                self.emit_store_class_environment_capture(capture, &named, function);
                named.clear(function);
                function.instruction(&Instruction::Else);
                let named = self.emit_load_class_environment_capture(capture, function);
                self.replace_current_environment(named.load(schema, function), function);
                named.clear(function);
                self.close_class_evaluation_state(function);
                self.begin_existing_lexical_environment_scope(&name_binding.environment);
            } else {
                self.emit_enter_lexical_environment(&name_binding.environment, function)?;
            }
            outer_environment.replace(
                schema
                    .struct_type::<Environment>()
                    .field(EnvironmentSchema::PARENT)
                    .read(self.current_environment(), schema, function)
                    .reference(),
                function,
            );
        }
        if let Some(plan) = plan {
            self.compile_class_evaluation_prefix(plan.heritage_prefix(), function)?;
            self.open_class_evaluation_state(plan.heritage_prefix().exit_state(), function)?;
        }
        let heritage = schema.reserve_value_local(function);
        heritage.set_undefined(function);
        if let Some(heritage_expr) = &class.heritage {
            self.compile_expr_to_value(heritage_expr, &heritage, function)?;
        }
        let realm = self.load_current_realm(function);
        let constructor_parent = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::FunctionPrototype,
            &constructor_parent,
            function,
        );
        let prototype_parent = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::ObjectPrototype,
            &prototype_parent,
            function,
        );
        match class.heritage_kind {
            ClassHeritageKind::None => {}
            ClassHeritageKind::Null => {
                prototype_parent.set_scalar(crate::gc_types::ScalarValue::Null, function)
            }
            ClassHeritageKind::Constructable => {
                if class.heritage.is_none() {
                    return Err(EmitError::unsupported(
                        "compiler invariant: class heritage has no operand",
                    ));
                }
                heritage.tag().load(function);
                function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null as i32));
                function.instruction(&Instruction::I32Eq);
                self.open_frame(ControlFrameKind::If, function);
                prototype_parent.set_scalar(crate::gc_types::ScalarValue::Null, function);
                function.instruction(&Instruction::Else);
                self.emit_is_constructor_i32(&heritage, function);
                function.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_throw_runtime_error(
                    NativeErrorKind::TypeError,
                    RuntimeErrorMessage::CLASS_EXTENDS_VALUE_IS_NOT_A_CONSTRUCTOR_OR_NULL,
                    result,
                    function,
                )?;
                self.completion().copy_from(result, function);
                self.emit_branch_to_target(finish, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                constructor_parent.copy_from(&heritage, function);
                let key = self.emit_function_string_key("prototype", function)?;
                self.emit_dynamic_property_read_with_key_locals(
                    &heritage, &heritage, &key, result, function,
                )?;
                key.clear(function);
                self.emit_class_adopt_abrupt(result, finish, function);
                self.emit_object_value_i32(result.value(), function);
                result.value().tag().load(function);
                function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Null as i32));
                function.instruction(&Instruction::I32Eq);
                function.instruction(&Instruction::I32Or);
                function.instruction(&Instruction::I32Eqz);
                self.open_frame(ControlFrameKind::If, function);
                self.emit_throw_runtime_error(
                    NativeErrorKind::TypeError,
                    RuntimeErrorMessage::CLASS_EXTENDS_PROTOTYPE_IS_NOT_AN_OBJECT_OR_NULL,
                    result,
                    function,
                )?;
                self.completion().copy_from(result, function);
                self.emit_branch_to_target(finish, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                prototype_parent.copy_from(result.value(), function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
            }
        }
        let created_prototype = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(&prototype_parent), function)?,
            function,
        );
        prototype.replace(
            created_prototype.load(schema, function).nullable(),
            function,
        );
        let private = self.emit_class_private_environment(class, function)?;
        private_environment.replace(private.load(schema, function), function);
        private.clear(function);
        if field_key_count != 0 {
            let undefined = schema.reserve_value_local(function);
            undefined.set_undefined(function);
            let stored = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<StoredValue>()
                    .from_value(&undefined, function),
                function,
            );
            let length = schema.reserve_i32_local(function);
            function.instruction(&Instruction::I32Const(field_key_count as i32));
            length.store(function);
            field_keys.replace(
                schema
                    .array_type::<ValueArray>()
                    .filled(GcOperand::reference(&stored, schema), length, function)
                    .nullable(),
                function,
            );
            schema.release_i32_local(length, function);
            stored.clear(function);
            undefined.clear(function);
        }
        self.emit_allocate_class_private_definitions(
            instance_slots.len(),
            &instance_methods,
            function,
        )?;
        self.emit_allocate_class_private_definitions(
            static_slots.len(),
            &static_methods,
            function,
        )?;
        let created_constructor = self.emit_source_class_constructor_record(
            &constructor_meta,
            &created_prototype,
            &constructor_parent,
            &private_environment,
            &field_keys,
            &instance_methods,
            function,
        )?;
        constructor.replace(
            created_constructor.load(schema, function).nullable(),
            function,
        );
        if !static_slots.is_empty() {
            let header = schema.reserve_gc_local(function).initialize(
                schema
                    .struct_type::<FunctionObject>()
                    .field(FunctionObjectSchema::OBJECT)
                    .read(&created_constructor, schema, function)
                    .reference(),
                function,
            );
            let table = schema.reserve_gc_local(function).initialize(
                static_methods
                    .load(schema, function)
                    .require_non_null(function),
                function,
            );
            schema
                .struct_type::<OrdinaryObject>()
                .field(OrdinaryObjectSchema::PRIVATE_ELEMENTS)
                .write(
                    &header,
                    GcOperand::reference(&table, schema),
                    schema,
                    function,
                );
            table.clear(function);
            header.clear(function);
        }
        self.emit_class_inferred_name(class, &created_constructor, function)?;
        if let Some(plan) = plan {
            let value = schema.reserve_value_local(function);
            value.set_reference(&created_constructor, schema, function);
            let storage = self
                .lookup_binding(plan.constructor_binding())
                .expect("owned constructor binding was registered");
            self.write_binding_from_locals(storage, &value, function);
            value.clear(function);
        }
        created_constructor.clear(function);
        created_prototype.clear(function);
        prototype_parent.clear(function);
        constructor_parent.clear(function);
        realm.clear(function);
        heritage.clear(function);
        if plan.is_some() {
            self.close_class_evaluation_state(function);
        }
        // Every later segment reloads complete captures from the retained constructor.
        if let Some(plan) = plan {
            let value = schema.reserve_value_local(function);
            let storage = self
                .lookup_binding(plan.constructor_binding())
                .expect("owned constructor binding was registered");
            self.read_binding_to_locals(storage, &value, function)?;
            constructor.replace(
                value
                    .cast_reference::<FunctionObject>(schema, function)
                    .nullable(),
                function,
            );
            value.clear(function);
        }
        let actual_constructor = schema.reserve_gc_local(function).initialize(
            constructor
                .load(schema, function)
                .require_non_null(function),
            function,
        );
        context.replace(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::CONTEXT)
                .read(&actual_constructor, schema, function)
                .reference()
                .nullable(),
            function,
        );
        let actual_context = schema.reserve_gc_local(function).initialize(
            context.load(schema, function).require_non_null(function),
            function,
        );
        let home = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::HOME_OBJECT)
                .read(&actual_context, schema, function)
                .reference(),
            function,
        );
        let home_value = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&home, &home_value, schema, function);
        prototype.replace(
            home_value
                .cast_reference::<OrdinaryObject>(schema, function)
                .nullable(),
            function,
        );
        home_value.clear(function);
        home.clear(function);
        private_environment.replace(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::PRIVATE_ENVIRONMENT)
                .read(&actual_context, schema, function)
                .reference(),
            function,
        );
        field_keys.replace(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::FIELD_KEYS)
                .read(&actual_context, schema, function)
                .reference(),
            function,
        );
        instance_methods.replace(
            schema
                .struct_type::<FunctionContext>()
                .field(FunctionContextSchema::INSTANCE_PRIVATE_METHODS)
                .read(&actual_context, schema, function)
                .reference(),
            function,
        );
        let header = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::OBJECT)
                .read(&actual_constructor, schema, function)
                .reference(),
            function,
        );
        static_methods.replace(
            schema
                .struct_type::<OrdinaryObject>()
                .field(OrdinaryObjectSchema::PRIVATE_ELEMENTS)
                .read(&header, schema, function)
                .reference()
                .nullable(),
            function,
        );
        header.clear(function);
        self.replace_current_private_environment(
            private_environment.load(schema, function),
            function,
        );
        let actual_prototype = schema.reserve_gc_local(function).initialize(
            prototype.load(schema, function).require_non_null(function),
            function,
        );
        let mut segment_state = plan.map(|plan| plan.heritage_prefix().exit_state());
        for (index, definition) in class.element_plan.definitions.iter().enumerate() {
            if let Some(plan) = plan {
                if let Some(prefix) = plan.element_prefix(index) {
                    self.compile_class_evaluation_prefix(prefix, function)?;
                    segment_state = Some(prefix.exit_state());
                }
                self.open_class_evaluation_state(
                    segment_state.expect("class element owns a state"),
                    function,
                )?;
            }
            self.emit_class_element_definition(
                definition,
                &actual_constructor,
                &actual_prototype,
                &private_environment,
                &field_keys,
                &instance_methods,
                &static_methods,
                &instance_slots,
                &static_slots,
                result,
                finish,
                function,
            )?;
            if plan.is_some() {
                self.close_class_evaluation_state(function);
            }
        }
        if let Some(plan) = plan {
            self.open_class_evaluation_state(plan.exit_state(), function)?;
        }
        if let Some(name_binding) = &class.name_binding {
            let storage = self
                .lookup_current_scope_binding(&name_binding.storage_name)
                .expect("class lexical name belongs to its scope");
            let value = schema.reserve_value_local(function);
            value.set_reference(&actual_constructor, schema, function);
            self.write_binding_from_locals(storage, &value, function);
            value.clear(function);
        }
        // Static definitions were completed while the constructor was private.
        // No observable operation intervenes between name initialization and use.
        let static_context =
            self.emit_class_static_execution_context(&actual_constructor, function);
        let receiver = schema.reserve_value_local(function);
        receiver.set_reference(&actual_constructor, schema, function);
        for element in &class.element_plan.static_elements {
            self.emit_class_static_element(
                element,
                &static_context,
                &receiver,
                &private_environment,
                &field_keys,
                result,
                finish,
                function,
            )?;
        }
        result.set_normal(&receiver, function);
        receiver.clear(function);
        static_context.clear(function);
        if plan.is_some() {
            self.close_class_evaluation_state(function);
        }
        actual_prototype.clear(function);
        actual_context.clear(function);
        actual_constructor.clear(function);
        if plan.is_some() {
            self.finally_stack.pop();
        }
        self.throw_handler_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.replace_current_environment(outer_environment.load(schema, function), function);
        self.replace_current_private_environment(
            outer_private_environment.load(schema, function),
            function,
        );
        if class.name_binding.is_some() {
            self.end_lexical_environment_scope();
            self.pop_scope();
        }
        if let Some(capture) = &class_environment_capture {
            self.emit_clear_class_environment_capture(capture, function);
        }
        self.completion().kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        result.copy_from(self.completion(), function);
        function.instruction(&Instruction::Else);
        if let Some(plan) = plan {
            let value = schema.reserve_value_local(function);
            let storage = self
                .lookup_binding(plan.completion_binding())
                .expect("owned class completion binding was registered");
            self.read_binding_to_locals(storage, &value, function)?;
            self.completion().set_normal(&value, function);
            value.clear(function);
        } else {
            self.completion().copy_from(&saved_completion, function);
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        if let Some(plan) = plan {
            // No suspended prefix remains after this common normal/abrupt
            // tail. Persist the restored outer record before outer dispatch.
            self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
            self.emit_save_resumable_environment(function)?;
        }
        saved_completion.clear(function);
        outer_private_environment.clear(function);
        outer_environment.clear(function);
        static_methods.clear(function);
        instance_methods.clear(function);
        field_keys.clear(function);
        private_environment.clear(function);
        context.clear(function);
        prototype.clear(function);
        constructor.clear(function);
        Ok(())
    }

    fn class_private_definition_slots(
        class: &ClassDefinitionIr,
        placement: ClassMethodPlacementIr,
    ) -> BTreeMap<PrivateNameId, u32> {
        let mut result = BTreeMap::new();
        for definition in &class.element_plan.definitions {
            let id = match definition {
                ClassElementDefinitionIr::PrivateMethod(method)
                    if method.placement == placement =>
                {
                    Some(method.private_name_id)
                }
                ClassElementDefinitionIr::AutoAccessor(accessor)
                    if accessor.placement == placement =>
                {
                    match accessor.key {
                        ClassFieldKeyIr::Private(id) => Some(id),
                        ClassFieldKeyIr::Public(_) | ClassFieldKeyIr::ComputedPublic(_) => None,
                    }
                }
                ClassElementDefinitionIr::PublicMethod(_)
                | ClassElementDefinitionIr::ComputedFieldKey { .. }
                | ClassElementDefinitionIr::PrivateMethod(_)
                | ClassElementDefinitionIr::AutoAccessor(_) => None,
            };
            if let Some(id) = id {
                let index = result.len() as u32;
                result.entry(id).or_insert(index);
            }
        }
        result
    }

    fn emit_allocate_class_private_definitions(
        &mut self,
        length: usize,
        output: &crate::gc_types::GcLocal<
            crate::gc_types::PrivateElementTable,
            crate::gc_types::Nullable,
        >,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if length == 0 {
            return Ok(());
        }
        let count = u32::try_from(length)
            .map_err(|_| EmitError::unsupported("class private definition count overflow"))?;
        let schema = self.runtime_schema();
        let local = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(count as i32));
        local.store(function);
        output.replace(
            schema
                .array_type::<crate::gc_types::PrivateElementTable>()
                .filled(crate::gc_types::GcOperand::null(schema), local, function)
                .nullable(),
            function,
        );
        schema.release_i32_local(local, function);
        Ok(())
    }

    pub(super) fn emit_class_adopt_abrupt(
        &mut self,
        result: &crate::gc_types::CompletionLocals,
        finish: ControlTarget,
        function: &mut Function,
    ) {
        result.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.completion().copy_from(result, function);
        self.emit_branch_to_target(finish, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    fn emit_class_resume_state(&self, function: &mut Function) -> Result<(), EmitError> {
        let point = self.emit_resumable_resume_point(function)?;
        point.load(function);
        self.runtime_schema().release_i32_local(point, function);
        Ok(())
    }

    fn open_class_evaluation_state(
        &mut self,
        state: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_class_resume_state(function)?;
        function.instruction(&Instruction::I32Const(state as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        Ok(())
    }
    fn close_class_evaluation_state(&mut self, function: &mut Function) {
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }
    fn compile_class_evaluation_prefix(
        &mut self,
        prefix: &ClassEvaluationPrefixIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_resumable_statement_sequence(
            prefix.statements(),
            prefix.entry_state(),
            function,
        )
    }
}
