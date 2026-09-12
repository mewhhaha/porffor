use super::*;
use lila_ir::{ClassEvaluationPrefixIr, ResumableClassDefinitionIr};

impl FunctionBuilder<'_> {
    pub(crate) fn compile_class_definition_payload(
        &mut self,
        class: &ClassDefinitionIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.compile_class_evaluation(class, None, function)
    }

    pub(crate) fn compile_resumable_class_definition(
        &mut self,
        plan: &ResumableClassDefinitionIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, kind) in [
            (Some(plan.constructor_binding()), ValueKind::Function),
            (plan.name_environment_binding(), ValueKind::Number),
            (Some(plan.completion_binding()), ValueKind::Dynamic),
        ] {
            if let Some(name) = name {
                assert!(
                    self.owned_env_slot(name).is_some(),
                    "class continuation storage must belong to its activation"
                );
                self.allocate_binding(name.to_string(), BindingMode::Let, kind);
            }
        }
        self.compile_class_evaluation(plan.class(), Some(plan), function)?;
        function.instruction(&Instruction::Drop);
        Ok(())
    }

    fn compile_class_evaluation(
        &mut self,
        class: &ClassDefinitionIr,
        plan: Option<&ResumableClassDefinitionIr>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let resume_offset = plan.map(|_| {
            match self
                .current_function_meta()
                .expect("resumable class has a function owner")
                .protocol
                .execution_kind()
            {
                FunctionExecutionKind::Generator => HEAP_GENERATOR_RESUME_STATE_OFFSET,
                FunctionExecutionKind::Async => HEAP_ASYNC_RESUME_STATE_OFFSET,
                FunctionExecutionKind::AsyncGenerator => HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
                FunctionExecutionKind::Ordinary => {
                    unreachable!("resumable class requires a resumable owner")
                }
            }
        });
        if let (Some(plan), Some(offset)) = (plan, resume_offset) {
            self.emit_class_resume_state(offset, function);
            function.instruction(&Instruction::I64Const(plan.entry_state() as i64));
            function.instruction(&Instruction::I64GeU);
            self.emit_class_resume_state(offset, function);
            function.instruction(&Instruction::I64Const(plan.exit_state() as i64));
            function.instruction(&Instruction::I64LeU);
            function.instruction(&Instruction::I32And);
            self.open_frame(ControlFrameKind::If, function);
            self.open_class_evaluation_state(plan.entry_state(), offset, function);
            let completion = self
                .lookup_binding(plan.completion_binding())
                .expect("class completion belongs to the activation");
            self.write_binding_from_locals(
                completion,
                self.result_local,
                self.result_tag_local,
                function,
            );
            self.close_class_evaluation_state(function);
        }
        let constructor_meta = self
            .functions
            .get(&class.constructor_function_id)
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unknown class constructor `{}`",
                    class.constructor_function_id
                ))
            })?
            .clone();
        if let Some(name_binding) = &class.name_binding {
            self.push_scope();
            if let (Some(plan), Some(offset)) = (plan, resume_offset) {
                let storage = self
                    .lookup_binding(
                        plan.name_environment_binding()
                            .expect("named class keeps its environment"),
                    )
                    .expect("class environment belongs to the activation");
                self.open_class_evaluation_state(plan.entry_state(), offset, function);
                self.emit_allocate_lexical_environment_record(&name_binding.environment, function)?;
                function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                // The storage is addressed from the outer environment, before
                // attaching the new environment to the compiler binding view.
                let parent = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(self.current_env_local));
                function.instruction(&Instruction::LocalSet(parent));
                self.load_i64_to_local_from_offset(
                    parent,
                    ENV_PARENT_OFFSET,
                    self.current_env_local,
                    function,
                );
                self.write_binding_from_locals(storage, parent, self.scratch_local, function);
                function.instruction(&Instruction::LocalGet(parent));
                function.instruction(&Instruction::LocalSet(self.current_env_local));
                self.release_temp_local(parent);
                function.instruction(&Instruction::Else);
                self.read_binding_to_locals(
                    storage,
                    self.current_env_local,
                    self.scratch_local,
                    function,
                )?;
                self.close_class_evaluation_state(function);
                self.begin_existing_lexical_environment_scope(&name_binding.environment);
            } else {
                self.emit_enter_lexical_environment(&name_binding.environment, function)?;
            }
        }
        let constructor_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let heritage_payload_local = self.reserve_temp_local();
        let heritage_tag_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let flags_local = self.reserve_temp_local();
        let computed_field_key_count = class
            .element_plan
            .definitions
            .iter()
            .filter_map(|definition| match definition {
                ClassElementDefinitionIr::ComputedFieldKey { slot, .. } => Some(*slot + 1),
                ClassElementDefinitionIr::AutoAccessor(accessor) => match &accessor.key {
                    ClassFieldKeyIr::ComputedPublic(slot) => Some(*slot + 1),
                    ClassFieldKeyIr::Public(_) | ClassFieldKeyIr::Private(_) => None,
                },
                ClassElementDefinitionIr::PublicMethod(_)
                | ClassElementDefinitionIr::PrivateMethod(_) => None,
            })
            .max()
            .unwrap_or(0);
        let class_element_context_local = class
            .element_plan
            .static_elements
            .iter()
            .any(|element| match element {
                ClassStaticElementIr::Field(field) => field.init_function_id.is_some(),
                ClassStaticElementIr::AutoAccessorBacking(accessor) => {
                    accessor.init_function_id.is_some()
                }
                ClassStaticElementIr::Block(_) => true,
            })
            .then(|| self.reserve_temp_local());
        let field_keys_local = (computed_field_key_count > 0).then(|| self.reserve_temp_local());
        let class_private_scope = class
            .private_environment
            .map(|environment| environment.class_scope());
        debug_assert!(class
            .private_name_ids
            .values()
            .all(|private_name_id| Some(private_name_id.class_scope()) == class_private_scope));
        let private_environment_local = Some(self.reserve_temp_local());

        if let (Some(plan), Some(offset)) = (plan, resume_offset) {
            self.compile_class_evaluation_prefix(plan.heritage_prefix(), offset, function)?;
            self.open_class_evaluation_state(plan.heritage_prefix().exit_state(), offset, function);
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(heritage_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(heritage_tag_local));
        if let Some(heritage) = &class.heritage {
            self.compile_expr_to_locals(
                heritage,
                heritage_payload_local,
                heritage_tag_local,
                function,
            )?;
        }

        match class.heritage_kind {
            ClassHeritageKind::Constructable => {
                if class.heritage.is_none() {
                    return Err(EmitError::unsupported(
                        "unsupported in lila wasm-aot first slice: missing class heritage",
                    ));
                }
                function.instruction(&Instruction::LocalGet(heritage_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(heritage_payload_local));
                function.instruction(&Instruction::Else);
                self.emit_is_constructor_i32(heritage_tag_local, heritage_payload_local, function)?;
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    "TypeError",
                    "class extends value is not a constructor or null",
                    self.result_local,
                    self.result_tag_local,
                    function,
                )?;
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, function);
                } else {
                    self.emit_return_current_completion(function);
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            }
            ClassHeritageKind::Null | ClassHeritageKind::None => {}
        }

        self.emit_function_value_payload(&constructor_meta, function)?;
        function.instruction(&Instruction::LocalSet(constructor_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(constructor_tag_local));
        if let Some(private_environment_local) = private_environment_local {
            self.emit_current_private_environment_to_local(key_local, function);
            if let Some(class_private_scope) = class_private_scope {
                self.emit_heap_alloc_const(
                    HEAP_PRIVATE_ENV_SLOT_BASE_OFFSET
                        + class
                            .private_environment
                            .expect("class private scope must have an environment")
                            .slot_count() as u64
                            * HEAP_PRIVATE_ENV_SLOT_SIZE,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(private_environment_local));
                self.store_i64_local_at_offset(
                    private_environment_local,
                    HEAP_PRIVATE_ENV_PARENT_OFFSET,
                    key_local,
                    function,
                );
                self.store_i64_const_at_offset(
                    private_environment_local,
                    HEAP_PRIVATE_ENV_CLASS_SCOPE_OFFSET,
                    class_private_scope as u64,
                    function,
                );
            } else {
                function.instruction(&Instruction::LocalGet(key_local));
                function.instruction(&Instruction::LocalSet(private_environment_local));
            }
            self.load_i64_to_local_from_offset(
                constructor_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                key_local,
                function,
            );
            self.store_i64_local_at_offset(
                key_local,
                HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                private_environment_local,
                function,
            );
        }
        if let Some(field_keys_local) = field_keys_local {
            self.load_i64_to_local_from_offset(
                constructor_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                key_local,
                function,
            );
            self.emit_heap_alloc_const(
                ENV_SLOT_BASE_OFFSET + computed_field_key_count as u64 * ENV_SLOT_SIZE,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(field_keys_local));
            self.store_i64_const_at_offset(field_keys_local, ENV_PARENT_OFFSET, 0, function);
            self.store_i64_local_at_offset(
                key_local,
                HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
                field_keys_local,
                function,
            );
        }
        if class.heritage_kind == ClassHeritageKind::Constructable {
            function.instruction(&Instruction::LocalGet(heritage_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_load_function_flags(constructor_local, flags_local, function);
            function.instruction(&Instruction::LocalGet(flags_local));
            function.instruction(&Instruction::I64Const(
                FUNCTION_FLAG_NULL_HERITAGE_CONSTRUCTOR as i64,
            ));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::LocalSet(flags_local));
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_FLAGS_OFFSET,
                flags_local,
                function,
            );
            function.instruction(&Instruction::Else);
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_PROTOTYPE_OFFSET,
                heritage_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                constructor_local,
                HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
                heritage_tag_local,
                function,
            );
            function.instruction(&Instruction::End);
        }

        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        if class.heritage_kind == ClassHeritageKind::Constructable {
            function.instruction(&Instruction::LocalGet(heritage_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_plain_object_with_prototype(None, None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::Else);
            self.emit_object_read(
                heritage_payload_local,
                heritage_tag_local,
                heritage_payload_local,
                heritage_tag_local,
                prototype_key_local,
                value_payload_local,
                value_tag_local,
                function,
            )?;
            self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
            function.instruction(&Instruction::LocalGet(value_tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::I32Or);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_alloc_plain_object_with_prototype_and_tag(
                Some(value_payload_local),
                Some(value_tag_local),
                None,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
            function.instruction(&Instruction::Else);
            self.emit_throw_runtime_error(
                "TypeError",
                "class extends prototype is not an object or null",
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            if let Some(target) = self.active_throw_target() {
                self.emit_branch_to_target(target, function);
            } else {
                self.emit_return_current_completion(function);
            }
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
        } else if class.heritage_kind == ClassHeritageKind::Null {
            self.emit_alloc_plain_object_with_prototype(None, None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
        } else {
            self.emit_source_literal_prototype_payload(
                crate::environments::global_environment::SourceLiteralPrototype::Object,
                function,
            );
            function.instruction(&Instruction::LocalSet(value_payload_local));
            self.emit_alloc_plain_object_with_prototype(Some(value_payload_local), None, function)?;
            function.instruction(&Instruction::LocalSet(prototype_payload_local));
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
            prototype_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            constructor_local,
            HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
            prototype_payload_local,
            function,
        );
        // Constructors are allocated before their `.prototype` exists.  Now
        // that the exact instance home object has been created, complete the
        // immutable class-function context used by direct constructor `super`.
        self.store_function_home_object(
            constructor_local,
            prototype_payload_local,
            ValueKind::Object,
            function,
        );
        self.emit_object_define_data_with_configurable(
            constructor_local,
            prototype_key_local,
            prototype_payload_local,
            prototype_tag_local,
            false,
            false,
            false,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::LocalGet(constructor_local));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            prototype_payload_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        if let (Some(plan), Some(offset)) = (plan, resume_offset) {
            let storage = self
                .lookup_binding(plan.constructor_binding())
                .expect("prepared class constructor belongs to activation");
            self.write_binding_from_locals(
                storage,
                constructor_local,
                constructor_tag_local,
                function,
            );
            self.close_class_evaluation_state(function);
            self.emit_class_resume_state(offset, function);
            function.instruction(&Instruction::I64Const(
                plan.heritage_prefix().exit_state() as i64
            ));
            function.instruction(&Instruction::I64GeU);
            self.open_frame(ControlFrameKind::If, function);
            self.read_binding_to_locals(
                storage,
                constructor_local,
                constructor_tag_local,
                function,
            )?;
            self.load_i64_to_local_from_offset(
                constructor_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                prototype_payload_local,
                function,
            );
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(prototype_tag_local));
            function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
            function.instruction(&Instruction::LocalSet(prototype_key_local));
            self.load_i64_to_local_from_offset(
                constructor_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                key_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                key_local,
                HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                private_environment_local.expect("class private cursor exists"),
                function,
            );
            if let Some(field_keys_local) = field_keys_local {
                self.load_i64_to_local_from_offset(
                    key_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_FIELD_KEYS_OFFSET,
                    field_keys_local,
                    function,
                );
            }
        }
        if let Some(private_environment_local) = private_environment_local {
            self.active_private_environment_locals
                .push(private_environment_local);
        }
        if let Some(class_element_context_local) = class_element_context_local {
            self.emit_alloc_class_execution_context(
                self.current_env_local,
                Some((constructor_local, ValueKind::Function)),
                class_element_context_local,
                function,
            )?;
            if let Some(private_environment_local) = private_environment_local {
                self.store_i64_local_at_offset(
                    class_element_context_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_PRIVATE_ENV_OFFSET,
                    private_environment_local,
                    function,
                );
            }
        }
        let mut segment_state = plan.map(|plan| plan.heritage_prefix().exit_state());
        let mut static_private_method_brands = BTreeSet::new();
        for (definition_index, definition) in class.element_plan.definitions.iter().enumerate() {
            if let (Some(plan), Some(offset)) = (plan, resume_offset) {
                if let Some(prefix) = plan.element_prefix(definition_index) {
                    self.compile_class_evaluation_prefix(prefix, offset, function)?;
                    segment_state = Some(prefix.exit_state());
                }
                self.open_class_evaluation_state(
                    segment_state.expect("resumable element has state"),
                    offset,
                    function,
                );
            }
            if let ClassElementDefinitionIr::AutoAccessor(accessor) = definition {
                if let Some(computed_key) = &accessor.computed_key {
                    let ClassFieldKeyIr::ComputedPublic(slot) = accessor.key else {
                        return Err(EmitError::unsupported(
                            "auto-accessor computed key requires a computed key slot",
                        ));
                    };
                    let field_keys_local =
                        field_keys_local.expect("computed field key cache must be allocated");
                    self.compile_object_key_to_locals(
                        computed_key,
                        key_local,
                        value_tag_local,
                        function,
                    )?;
                    self.store_i64_local_at_offset(
                        field_keys_local,
                        ENV_SLOT_BASE_OFFSET
                            + slot as u64 * ENV_SLOT_SIZE
                            + ENV_SLOT_PAYLOAD_OFFSET,
                        key_local,
                        function,
                    );
                    self.store_i64_local_at_offset(
                        field_keys_local,
                        ENV_SLOT_BASE_OFFSET + slot as u64 * ENV_SLOT_SIZE + ENV_SLOT_TAG_OFFSET,
                        value_tag_local,
                        function,
                    );
                }
                match &accessor.key {
                    ClassFieldKeyIr::Public(key) => {
                        function.instruction(&Instruction::I64Const(self.strings.payload(key)));
                        function.instruction(&Instruction::LocalSet(key_local));
                    }
                    ClassFieldKeyIr::ComputedPublic(slot) => {
                        let field_keys_local =
                            field_keys_local.expect("computed field key cache must be allocated");
                        self.load_i64_to_local_from_offset(
                            field_keys_local,
                            ENV_SLOT_BASE_OFFSET
                                + *slot as u64 * ENV_SLOT_SIZE
                                + ENV_SLOT_PAYLOAD_OFFSET,
                            key_local,
                            function,
                        );
                    }
                    ClassFieldKeyIr::Private(private_name_id) => {
                        self.emit_private_name_token_to_local(
                            *private_name_id,
                            key_local,
                            function,
                        )?;
                    }
                }
                let target_local = match accessor.placement {
                    ClassMethodPlacementIr::Instance => prototype_payload_local,
                    ClassMethodPlacementIr::Static => constructor_local,
                };
                let getter_meta = self
                    .functions
                    .get(accessor.functions.getter())
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unknown auto-accessor getter `{}`",
                            accessor.functions.getter()
                        ))
                    })?;
                self.emit_class_function_value_payload(
                    getter_meta,
                    target_local,
                    private_environment_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(value_payload_local));
                let setter_payload_local = self.reserve_temp_local();
                let setter_meta = self
                    .functions
                    .get(accessor.functions.setter())
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unknown auto-accessor setter `{}`",
                            accessor.functions.setter()
                        ))
                    })?;
                self.emit_class_function_value_payload(
                    setter_meta,
                    target_local,
                    private_environment_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalSet(setter_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(value_tag_local));
                if matches!(accessor.key, ClassFieldKeyIr::Private(_)) {
                    self.emit_private_getter_definition_add(
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    self.emit_private_setter_definition_add(
                        key_local,
                        setter_payload_local,
                        value_tag_local,
                        function,
                    )?;
                    if accessor.placement == ClassMethodPlacementIr::Static {
                        let ClassFieldKeyIr::Private(private_name_id) = accessor.key else {
                            unreachable!()
                        };
                        static_private_method_brands.insert(private_name_id);
                    }
                } else {
                    if accessor.placement == ClassMethodPlacementIr::Static {
                        self.emit_reject_static_class_prototype_definition(
                            key_local,
                            prototype_key_local,
                            function,
                        )?;
                    }
                    self.emit_object_define_accessor(
                        target_local,
                        key_local,
                        AccessorDescriptorLocals::GetterAndSetter {
                            getter: AccessorGetterLocals::new(TaggedLocals::new(
                                value_payload_local,
                                value_tag_local,
                            )),
                            setter: AccessorSetterLocals::new(TaggedLocals::new(
                                setter_payload_local,
                                value_tag_local,
                            )),
                        },
                        function,
                    )?;
                }
                self.release_temp_local(setter_payload_local);
                if plan.is_some() {
                    self.close_class_evaluation_state(function);
                }
                continue;
            }
            let (function_id, placement, kind, private_name_id) = match definition {
                ClassElementDefinitionIr::PublicMethod(method) => {
                    let compiled_key_local =
                        self.compile_object_key_to_local(&method.key, function)?;
                    function.instruction(&Instruction::LocalGet(compiled_key_local));
                    function.instruction(&Instruction::LocalSet(key_local));
                    self.release_temp_local(compiled_key_local);
                    (&method.function_id, method.placement, method.kind, None)
                }
                ClassElementDefinitionIr::PrivateMethod(method) => (
                    &method.function_id,
                    method.placement,
                    method.kind,
                    Some(method.private_name_id),
                ),
                ClassElementDefinitionIr::ComputedFieldKey { slot, key } => {
                    let field_keys_local =
                        field_keys_local.expect("computed field key cache must be allocated");
                    self.compile_object_key_to_locals(key, key_local, value_tag_local, function)?;
                    self.store_i64_local_at_offset(
                        field_keys_local,
                        ENV_SLOT_BASE_OFFSET
                            + *slot as u64 * ENV_SLOT_SIZE
                            + ENV_SLOT_PAYLOAD_OFFSET,
                        key_local,
                        function,
                    );
                    self.store_i64_local_at_offset(
                        field_keys_local,
                        ENV_SLOT_BASE_OFFSET + *slot as u64 * ENV_SLOT_SIZE + ENV_SLOT_TAG_OFFSET,
                        value_tag_local,
                        function,
                    );
                    if plan.is_some() {
                        self.close_class_evaluation_state(function);
                    }
                    continue;
                }
                ClassElementDefinitionIr::AutoAccessor(_) => unreachable!(),
            };
            let target_local = match placement {
                ClassMethodPlacementIr::Instance => prototype_payload_local,
                ClassMethodPlacementIr::Static => constructor_local,
            };
            let meta = self.functions.get(function_id).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: unknown class method `{function_id}`"
                ))
            })?;
            self.emit_class_function_value_payload(
                meta,
                target_local,
                private_environment_local,
                function,
            )?;
            function.instruction(&Instruction::LocalSet(value_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            if placement == ClassMethodPlacementIr::Static && private_name_id.is_none() {
                self.emit_reject_static_class_prototype_definition(
                    key_local,
                    prototype_key_local,
                    function,
                )?;
            }
            match kind {
                ClassMethodKindIr::Method => {
                    if let Some(private_name_id) = private_name_id {
                        self.emit_private_name_token_to_local(
                            private_name_id,
                            key_local,
                            function,
                        )?;
                        self.emit_private_method_definition_add(
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        self.emit_object_define_data(
                            target_local,
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    }
                }
                ClassMethodKindIr::Getter => {
                    if let Some(private_name_id) = private_name_id {
                        self.emit_private_name_token_to_local(
                            private_name_id,
                            key_local,
                            function,
                        )?;
                        self.emit_private_getter_definition_add(
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        self.emit_object_define_accessor(
                            target_local,
                            key_local,
                            AccessorDescriptorLocals::Getter(AccessorGetterLocals::new(
                                TaggedLocals::new(value_payload_local, value_tag_local),
                            )),
                            function,
                        )?;
                    }
                }
                ClassMethodKindIr::Setter => {
                    if let Some(private_name_id) = private_name_id {
                        self.emit_private_name_token_to_local(
                            private_name_id,
                            key_local,
                            function,
                        )?;
                        self.emit_private_setter_definition_add(
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        self.emit_object_define_accessor(
                            target_local,
                            key_local,
                            AccessorDescriptorLocals::Setter(AccessorSetterLocals::new(
                                TaggedLocals::new(value_payload_local, value_tag_local),
                            )),
                            function,
                        )?;
                    }
                }
            }
            if placement == ClassMethodPlacementIr::Static {
                if let Some(private_name_id) = private_name_id {
                    static_private_method_brands.insert(private_name_id);
                }
            }
            if plan.is_some() {
                self.close_class_evaluation_state(function);
            }
        }

        if let (Some(plan), Some(offset)) = (plan, resume_offset) {
            self.open_class_evaluation_state(plan.exit_state(), offset, function);
        }
        if let Some(name_binding) = &class.name_binding {
            let storage = self
                .lookup_current_scope_binding(&name_binding.storage_name)
                .expect("class name environment must expose its binding");
            self.write_binding_from_locals(
                storage,
                constructor_local,
                constructor_tag_local,
                function,
            );
        }
        for private_name_id in static_private_method_brands {
            self.emit_private_name_token_to_local(private_name_id, key_local, function)?;
            self.emit_private_brand_add(
                constructor_local,
                constructor_tag_local,
                key_local,
                function,
            )?;
        }

        for static_element in &class.element_plan.static_elements {
            match static_element {
                ClassStaticElementIr::Field(field) => {
                    self.load_i64_to_local_from_offset(
                        constructor_local,
                        HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                        value_tag_local,
                        function,
                    );
                    self.emit_class_field_key_to_local(
                        &field.key,
                        value_tag_local,
                        key_local,
                        function,
                    );
                    if let Some(init_function_id) = &field.init_function_id {
                        let meta = self.functions.get(init_function_id).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: unknown class field init `{init_function_id}`"
                            ))
                        })?;
                        if meta.class_element_execution_kind
                            != ClassElementExecutionKind::StaticFieldInitializer
                        {
                            return Err(EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: class field init `{init_function_id}` has invalid execution kind"
                            )));
                        }
                        self.emit_direct_class_element_js_call(
                            meta,
                            class_element_context_local
                                .expect("static initializer context must exist"),
                            Some((constructor_local, Some(constructor_tag_local))),
                            &[],
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(value_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(value_tag_local));
                    }
                    if let ClassFieldKeyIr::Private(private_name_id) = &field.key {
                        self.emit_private_name_token_to_local(
                            *private_name_id,
                            key_local,
                            function,
                        )?;
                        self.emit_private_field_add(
                            constructor_local,
                            constructor_tag_local,
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        self.emit_reject_static_class_prototype_definition(
                            key_local,
                            prototype_key_local,
                            function,
                        )?;
                        self.emit_object_define_enumerable_data(
                            constructor_local,
                            key_local,
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    }
                }
                ClassStaticElementIr::AutoAccessorBacking(accessor) => {
                    if let Some(init_function_id) = &accessor.init_function_id {
                        let meta = self.functions.get(init_function_id).ok_or_else(|| {
                            EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: unknown auto-accessor init `{init_function_id}`"
                            ))
                        })?;
                        if meta.class_element_execution_kind
                            != ClassElementExecutionKind::StaticFieldInitializer
                        {
                            return Err(EmitError::unsupported(format!(
                                "unsupported in lila wasm-aot first slice: auto-accessor init `{init_function_id}` has invalid execution kind"
                            )));
                        }
                        self.emit_direct_class_element_js_call(
                            meta,
                            class_element_context_local
                                .expect("static initializer context must exist"),
                            Some((constructor_local, Some(constructor_tag_local))),
                            &[],
                            value_payload_local,
                            value_tag_local,
                            function,
                        )?;
                    } else {
                        function.instruction(&Instruction::I64Const(0));
                        function.instruction(&Instruction::LocalSet(value_payload_local));
                        function
                            .instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                        function.instruction(&Instruction::LocalSet(value_tag_local));
                    }
                    self.emit_private_name_token_to_local(
                        accessor.backing_name.private_name_id(),
                        key_local,
                        function,
                    )?;
                    self.emit_private_field_add(
                        constructor_local,
                        constructor_tag_local,
                        key_local,
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
                ClassStaticElementIr::Block(block) => {
                    let meta = self.functions.get(&block.function_id).ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: unknown class static block `{}`",
                            block.function_id
                        ))
                    })?;
                    if meta.class_element_execution_kind != ClassElementExecutionKind::StaticBlock {
                        return Err(EmitError::unsupported(format!(
                            "unsupported in lila wasm-aot first slice: class static block `{}` has invalid execution kind",
                            block.function_id
                        )));
                    }
                    self.emit_direct_class_element_js_call(
                        meta,
                        class_element_context_local.expect("static block context must exist"),
                        Some((constructor_local, Some(constructor_tag_local))),
                        &[],
                        value_payload_local,
                        value_tag_local,
                        function,
                    )?;
                }
            }
        }

        if private_environment_local.is_some() {
            self.active_private_environment_locals.pop();
        }
        if plan.is_some() {
            self.close_class_evaluation_state(function);
            self.close_class_evaluation_state(function);
        }
        if class.name_binding.is_some() {
            self.emit_leave_lexical_environment(function);
            self.pop_scope();
        }
        if let Some(plan) = plan {
            self.emit_propagate_throw_from_locals_if_needed(
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            let completion = self
                .lookup_binding(plan.completion_binding())
                .expect("class completion belongs to activation");
            self.read_binding_to_locals(
                completion,
                self.result_local,
                self.result_tag_local,
                function,
            )?;
            self.close_class_evaluation_state(function);
            // The statement result is empty. Its caller drops this payload;
            // the residual expression reads the activation's constructor slot.
            function.instruction(&Instruction::I64Const(0));
        } else {
            function.instruction(&Instruction::LocalGet(constructor_local));
        }
        if let Some(private_environment_local) = private_environment_local {
            self.release_temp_local(private_environment_local);
        }
        if let Some(field_keys_local) = field_keys_local {
            self.release_temp_local(field_keys_local);
        }
        if let Some(class_element_context_local) = class_element_context_local {
            self.release_temp_local(class_element_context_local);
        }
        self.release_temp_local(flags_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(heritage_tag_local);
        self.release_temp_local(heritage_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_local);
        Ok(())
    }

    fn emit_class_resume_state(&mut self, offset: u64, function: &mut Function) {
        let activation = self
            .new_target_payload_local()
            .expect("class continuation has an activation");
        self.load_i64_to_local_from_offset(activation, offset, self.scratch_local, function);
        function.instruction(&Instruction::LocalGet(self.scratch_local));
    }

    fn open_class_evaluation_state(&mut self, state: u32, offset: u64, function: &mut Function) {
        self.emit_class_resume_state(offset, function);
        function.instruction(&Instruction::I64Const(state as i64));
        function.instruction(&Instruction::I64Eq);
        self.open_frame(ControlFrameKind::If, function);
    }

    fn close_class_evaluation_state(&mut self, function: &mut Function) {
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }

    fn compile_class_evaluation_prefix(
        &mut self,
        prefix: &ClassEvaluationPrefixIr,
        offset: u64,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if offset == HEAP_GENERATOR_RESUME_STATE_OFFSET
            && self
                .current_function_meta()
                .expect("resumable class owner")
                .protocol
                .execution_kind()
                == FunctionExecutionKind::Generator
        {
            self.compile_generator_statement_sequence(
                prefix.statements(),
                prefix.entry_state(),
                function,
            )
        } else {
            self.compile_async_statement_sequence(
                prefix.statements(),
                prefix.entry_state(),
                offset,
                function,
            )
        }
    }
}
