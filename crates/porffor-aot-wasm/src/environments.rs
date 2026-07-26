use super::*;
use porffor_ir::LexicalEnvironmentIr;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_enter_for_in_of_tdz_scope(
        &mut self,
        mode: BindingMode,
        environment: &ForInOfEnvironmentIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.push_scope();
        if let Some(runtime_environment) = &environment.tdz_environment {
            self.emit_enter_lexical_environment(runtime_environment, function)?;
        }
        for name in &environment.tdz_binding_names {
            let storage = self
                .lookup_current_scope_binding(name)
                .unwrap_or_else(|| self.allocate_binding(name.clone(), mode, ValueKind::Dynamic));
            self.initialize_binding_uninitialized(storage, function);
        }
        Ok(())
    }

    pub(crate) fn emit_leave_for_in_of_tdz_scope(
        &mut self,
        environment: &ForInOfEnvironmentIr,
        function: &mut Function,
    ) {
        if environment.tdz_environment.is_some() {
            self.emit_leave_lexical_environment(function);
        }
        self.pop_scope();
    }

    pub(crate) fn emit_enter_lexical_environment(
        &mut self,
        environment: &LexicalEnvironmentIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let parent_env_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(parent_env_local));
        self.emit_heap_alloc_const(
            ENV_SLOT_BASE_OFFSET + environment.bindings.len() as u64 * ENV_SLOT_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.store_i64_local_at_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        for binding in &environment.bindings {
            self.store_i64_const_at_offset(
                self.current_env_local,
                Self::env_slot_offset(binding.slot, ENV_SLOT_TAG_OFFSET),
                ENV_SLOT_UNINITIALIZED_TAG as u64,
                function,
            );
            self.store_i64_const_at_offset(
                self.current_env_local,
                Self::env_slot_offset(binding.slot, ENV_SLOT_PAYLOAD_OFFSET),
                0,
                function,
            );
        }
        self.release_temp_local(parent_env_local);

        let outer_scope_count = self.binding_scopes.len().saturating_sub(1);
        for scope in &mut self.binding_scopes[..outer_scope_count] {
            for storage in scope.values_mut() {
                if let BindingStorage::EnvSlot { hops, .. } = storage {
                    *hops += 1;
                }
            }
        }
        let scope = self
            .binding_scopes
            .last_mut()
            .expect("block environment requires an active binding scope");
        for binding in &environment.bindings {
            scope.insert(
                binding.name.clone(),
                BindingStorage::EnvSlot {
                    slot: binding.slot,
                    hops: 0,
                },
            );
        }
        self.environment_depth += 1;
        Ok(())
    }

    pub(crate) fn emit_leave_lexical_environment(&mut self, function: &mut Function) {
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            self.current_env_local,
            function,
        );
        self.end_lexical_environment_scope();
    }

    pub(crate) fn end_lexical_environment_scope(&mut self) {
        self.environment_depth = self
            .environment_depth
            .checked_sub(1)
            .expect("block environment depth must not underflow");
        let outer_scope_count = self.binding_scopes.len().saturating_sub(1);
        for scope in &mut self.binding_scopes[..outer_scope_count] {
            for storage in scope.values_mut() {
                if let BindingStorage::EnvSlot { hops, .. } = storage {
                    *hops = hops
                        .checked_sub(1)
                        .expect("outer environment binding must have at least one hop");
                }
            }
        }
    }

    pub(crate) fn emit_replace_lexical_environment(
        &mut self,
        environment: &ForLexicalEnvironmentIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if environment.per_iteration_slots.is_empty() {
            return Ok(());
        }
        let previous_env_local = self.reserve_temp_local();
        let parent_env_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(previous_env_local));
        self.load_i64_to_local_from_offset(
            previous_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        self.emit_heap_alloc_const(
            ENV_SLOT_BASE_OFFSET + environment.bindings.len() as u64 * ENV_SLOT_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.store_i64_local_at_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        for binding in &environment.bindings {
            self.store_i64_const_at_offset(
                self.current_env_local,
                Self::env_slot_offset(binding.slot, ENV_SLOT_TAG_OFFSET),
                ENV_SLOT_UNINITIALIZED_TAG as u64,
                function,
            );
            self.store_i64_const_at_offset(
                self.current_env_local,
                Self::env_slot_offset(binding.slot, ENV_SLOT_PAYLOAD_OFFSET),
                0,
                function,
            );
        }
        for slot in &environment.per_iteration_slots {
            for field_offset in [ENV_SLOT_TAG_OFFSET, ENV_SLOT_PAYLOAD_OFFSET] {
                self.load_i64_to_local_from_offset(
                    previous_env_local,
                    Self::env_slot_offset(*slot, field_offset),
                    value_local,
                    function,
                );
                self.store_i64_local_at_offset(
                    self.current_env_local,
                    Self::env_slot_offset(*slot, field_offset),
                    value_local,
                    function,
                );
            }
        }
        self.release_temp_local(value_local);
        self.release_temp_local(parent_env_local);
        self.release_temp_local(previous_env_local);
        Ok(())
    }

    pub(crate) const fn env_slot_offset(slot: u32, field_offset: u64) -> u64 {
        ENV_SLOT_BASE_OFFSET + slot as u64 * ENV_SLOT_SIZE + field_offset
    }

    pub(crate) fn resolve_env_handle_local(&mut self, hops: u32, function: &mut Function) -> u32 {
        let env_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(env_local));
        for _ in 0..hops {
            self.load_i64_to_local_from_offset(env_local, ENV_PARENT_OFFSET, env_local, function);
        }
        env_local
    }

    pub(crate) fn read_env_slot_to_locals(
        &mut self,
        slot: u32,
        hops: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let env_local = self.resolve_env_handle_local(hops, function);
        self.load_i64_to_local_from_offset(
            env_local,
            Self::env_slot_offset(slot, ENV_SLOT_PAYLOAD_OFFSET),
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            env_local,
            Self::env_slot_offset(slot, ENV_SLOT_TAG_OFFSET),
            tag_local,
            function,
        );
        self.release_temp_local(env_local);
    }

    pub(crate) fn write_env_slot_from_locals(
        &mut self,
        slot: u32,
        hops: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let env_local = self.resolve_env_handle_local(hops, function);
        self.store_i64_local_at_offset(
            env_local,
            Self::env_slot_offset(slot, ENV_SLOT_TAG_OFFSET),
            tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            env_local,
            Self::env_slot_offset(slot, ENV_SLOT_PAYLOAD_OFFSET),
            payload_local,
            function,
        );
        self.release_temp_local(env_local);
    }

    pub(crate) fn initialize_binding_undefined(
        &mut self,
        storage: BindingStorage,
        function: &mut Function,
    ) {
        match storage {
            BindingStorage::Fixed { payload_local, .. } => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::Dynamic {
                tag_local,
                payload_local,
            } => {
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::EnvSlot { slot, hops } => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.write_env_slot_from_locals(
                    slot,
                    hops,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
            }
        }
    }

    pub(crate) fn initialize_binding_uninitialized(
        &mut self,
        storage: BindingStorage,
        function: &mut Function,
    ) {
        match storage {
            BindingStorage::Fixed { payload_local, .. } => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::Dynamic {
                tag_local,
                payload_local,
            } => {
                function.instruction(&Instruction::I64Const(ENV_SLOT_UNINITIALIZED_TAG));
                function.instruction(&Instruction::LocalSet(tag_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::EnvSlot { slot, hops } => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.scratch_local));
                function.instruction(&Instruction::I64Const(ENV_SLOT_UNINITIALIZED_TAG));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.write_env_slot_from_locals(
                    slot,
                    hops,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
            }
        }
    }

    pub(crate) fn write_binding_from_locals(
        &mut self,
        storage: BindingStorage,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        match storage {
            BindingStorage::Fixed {
                payload_local: binding_payload_local,
                ..
            } => {
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalSet(binding_payload_local));
            }
            BindingStorage::Dynamic {
                tag_local: binding_tag_local,
                payload_local: binding_payload_local,
            } => {
                function.instruction(&Instruction::LocalGet(payload_local));
                function.instruction(&Instruction::LocalSet(binding_payload_local));
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::LocalSet(binding_tag_local));
            }
            BindingStorage::EnvSlot { slot, hops } => {
                self.write_env_slot_from_locals(slot, hops, payload_local, tag_local, function);
            }
        }
    }

    pub(crate) fn bind_captured_bindings(&mut self, function: &mut Function) {
        for binding in self.captured_bindings {
            if binding.name == LEXICAL_ARGUMENTS_NAME {
                let payload_local = self.next_binding_local;
                let tag_local = self.next_binding_local + 1;
                self.next_binding_local += 2;
                let storage = BindingStorage::Dynamic {
                    tag_local,
                    payload_local,
                };
                self.read_env_slot_to_locals(
                    binding.slot,
                    binding.hops,
                    payload_local,
                    tag_local,
                    function,
                );
                self.binding_scopes
                    .last_mut()
                    .expect("binding scope stack must exist")
                    .insert(binding.name.clone(), storage);
            } else {
                self.binding_scopes
                    .last_mut()
                    .expect("binding scope stack must exist")
                    .insert(
                        binding.name.clone(),
                        BindingStorage::EnvSlot {
                            slot: binding.slot,
                            hops: binding.hops,
                        },
                    );
            }
        }
    }

    pub(crate) fn bind_parameters(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let parameter_initializers = self
            .body
            .statements
            .iter()
            .filter_map(|statement| {
                let StatementIr::ParameterInitialization {
                    parameter_index,
                    statements,
                } = statement
                else {
                    return None;
                };
                Some((*parameter_index, statements.clone()))
            })
            .collect::<Vec<_>>();
        if matches!(self.return_abi, ReturnAbi::MultiValue)
            && self.function_flavor == FunctionFlavor::Ordinary
        {
            let arguments_storage = self.allocate_dynamic_binding_storage(LEXICAL_ARGUMENTS_NAME);
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(LEXICAL_ARGUMENTS_NAME.to_string(), arguments_storage);
            self.initialize_arguments_binding(arguments_storage, function)?;
        }

        for param in self.params {
            let storage = self.allocate_dynamic_binding_storage(&param.name);
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(param.name.clone(), storage);
        }

        for (index, param) in self.params.iter().enumerate() {
            let storage = self.lookup_binding(&param.name).ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in porffor wasm-aot first slice: missing parameter binding `{}`",
                    param.name
                ))
            })?;
            if param.is_rest {
                self.initialize_rest_parameter(index, storage, function)?;
            } else {
                self.initialize_parameter(index, param, storage, function)?;
            }
            if let Some((_, statements)) = parameter_initializers
                .iter()
                .find(|(parameter_index, _)| *parameter_index == index)
            {
                for statement in statements {
                    self.compile_statement(statement, function)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn allocate_dynamic_binding_storage(&mut self, name: &str) -> BindingStorage {
        if let Some(slot) = self.owned_env_slot(name) {
            BindingStorage::EnvSlot { slot, hops: 0 }
        } else {
            let payload_local = self.next_binding_local;
            let tag_local = self.next_binding_local + 1;
            self.next_binding_local += 2;
            BindingStorage::Dynamic {
                tag_local,
                payload_local,
            }
        }
    }

    pub(crate) fn initialize_arguments_binding(
        &mut self,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.emit_arguments_object_payload(function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.write_binding_from_locals(storage, payload_local, tag_local, function);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn initialize_parameter(
        &mut self,
        index: usize,
        param: &FunctionParamIr,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.read_argument_at_index(index, payload_local, tag_local, function);

        if let Some(default_init) = &param.default_init {
            function.instruction(&Instruction::LocalGet(tag_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.compile_expr_to_locals(default_init, payload_local, tag_local, function)?;
            self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
            function.instruction(&Instruction::End);
        }

        self.write_binding_from_locals(storage, payload_local, tag_local, function);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn initialize_rest_parameter(
        &mut self,
        index: usize,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let payload_local = self.reserve_temp_local();
        let tag_local = self.reserve_temp_local();
        self.emit_rest_array_payload(index, function)?;
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.write_binding_from_locals(storage, payload_local, tag_local, function);
        self.release_temp_local(tag_local);
        self.release_temp_local(payload_local);
        Ok(())
    }

    pub(crate) fn bind_self_function(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let Some(function_name) = self.self_binding_name.as_ref() else {
            return Ok(());
        };
        let storage = if let Some(slot) = self.owned_env_slot(function_name) {
            BindingStorage::EnvSlot { slot, hops: 0 }
        } else {
            let payload_local = self.next_binding_local;
            self.next_binding_local += 1;
            BindingStorage::Fixed {
                payload_local,
                kind: ValueKind::Function,
            }
        };
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(function_name.clone(), storage);
        self.load_i64_to_local_from_offset(
            self.named_function_context_local,
            ENV_SLOT_BASE_OFFSET + ENV_SLOT_PAYLOAD_OFFSET,
            self.scratch_local,
            function,
        );
        match storage {
            BindingStorage::Fixed { payload_local, .. } => {
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::EnvSlot { .. } => {
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.write_binding_from_locals(
                    storage,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
            }
            BindingStorage::Dynamic { .. } => unreachable!(),
        }
        Ok(())
    }

    pub(crate) fn compile_this_payload(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self.function_flavor {
            FunctionFlavor::Ordinary => {
                if self.lexical_derived_activation.is_some() {
                    self.emit_get_derived_this_to_locals(
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                    return Ok(());
                }
                if let Some(this_payload_local) = self.this_payload_local {
                    function.instruction(&Instruction::LocalGet(this_payload_local));
                } else if self.is_main() {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                } else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: top-level `this`",
                    ));
                }
            }
            FunctionFlavor::Arrow => {
                if self.lexical_derived_activation.is_some() {
                    self.emit_get_derived_this_to_locals(
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(self.scratch_local));
                } else if let Some(storage) = self.lookup_binding(LEXICAL_THIS_NAME) {
                    self.read_binding_payload(storage, function)?;
                } else {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn compile_this_to_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match self.function_flavor {
            FunctionFlavor::Ordinary => {
                if self.lexical_derived_activation.is_some() {
                    return self.emit_get_derived_this_to_locals(
                        payload_local,
                        tag_local,
                        function,
                    );
                }
                if let (Some(this_payload_local), Some(this_tag_local)) =
                    (self.this_payload_local, self.this_tag_local)
                {
                    function.instruction(&Instruction::LocalGet(this_payload_local));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::LocalGet(this_tag_local));
                    function.instruction(&Instruction::LocalSet(tag_local));
                } else if self.is_main() {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                } else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing `this` tag local",
                    ));
                }
            }
            FunctionFlavor::Arrow => {
                if self.lexical_derived_activation.is_some() {
                    self.emit_get_derived_this_to_locals(payload_local, tag_local, function)?;
                } else if let Some(storage) = self.lookup_binding(LEXICAL_THIS_NAME) {
                    self.read_binding_to_locals(storage, payload_local, tag_local, function)?;
                } else {
                    function
                        .instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
                    function.instruction(&Instruction::LocalSet(payload_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
                    function.instruction(&Instruction::LocalSet(tag_local));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn emit_default_this(&self, function: &mut Function) {
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
    }

    pub(crate) fn emit_default_this_for_known_strictness(
        &self,
        strict: bool,
        function: &mut Function,
    ) {
        if strict {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        } else {
            self.emit_default_this(function);
        }
    }

    pub(crate) fn emit_undefined_new_target(&self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
    }

    pub(crate) fn compile_new_target_to_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if self.current_function_meta().is_some_and(|meta| {
            matches!(
                meta.execution_kind,
                FunctionExecutionKind::Generator | FunctionExecutionKind::Async
            )
        }) {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
            return Ok(());
        }
        if self.function_flavor == FunctionFlavor::Arrow {
            if let Some(storage) = self.lookup_binding(LEXICAL_NEW_TARGET_NAME) {
                self.read_binding_to_locals(storage, payload_local, tag_local, function)?;
                return Ok(());
            }
        }
        if let (Some(new_target_payload_local), Some(new_target_tag_local)) =
            (self.new_target_payload_local(), self.new_target_tag_local())
        {
            function.instruction(&Instruction::LocalGet(new_target_payload_local));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::LocalGet(new_target_tag_local));
            function.instruction(&Instruction::LocalSet(tag_local));
        } else {
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(tag_local));
        }
        Ok(())
    }

    pub(crate) const fn argc_param_local(&self) -> u32 {
        5
    }

    pub(crate) const fn argv_param_local(&self) -> u32 {
        6
    }

    pub(crate) const fn new_target_payload_local(&self) -> Option<u32> {
        if matches!(self.return_abi, ReturnAbi::MultiValue) {
            Some(3)
        } else {
            None
        }
    }

    pub(crate) const fn new_target_tag_local(&self) -> Option<u32> {
        if matches!(self.return_abi, ReturnAbi::MultiValue) {
            Some(4)
        } else {
            None
        }
    }

    pub(crate) fn has_simple_parameter_list(&self) -> bool {
        self.params
            .iter()
            .all(|param| !param.is_rest && param.default_init.is_none())
    }

    pub(crate) fn is_current_function_strict(&self) -> bool {
        self.function_id
            .as_ref()
            .and_then(|function_id| self.functions.get(function_id))
            .map_or(self.strict, |meta| meta.strict)
    }

    /// Per CreateMappedArgumentsObject (ES2023 10.4.4), a non-strict ordinary
    /// function with a simple parameter list gets an arguments object whose
    /// indexed slots alias the corresponding parameter bindings. Duplicate
    /// names map only their last occurrence; each mapped arguments entry stores
    /// the actual parameter environment slot used by reads and writes.
    pub(crate) fn uses_mapped_arguments_object(&self) -> bool {
        self.function_flavor == FunctionFlavor::Ordinary
            && !self.is_current_function_strict()
            && self.has_simple_parameter_list()
            && self
                .params
                .iter()
                .enumerate()
                .filter(|(index, param)| {
                    !self.params[index + 1..]
                        .iter()
                        .any(|later| later.name == param.name)
                })
                .all(|(_, param)| self.owned_env_slot(&param.name).is_some())
    }

    pub(crate) fn read_argument_at_index(
        &mut self,
        index: usize,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
        let index_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(index as i64));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        function.instruction(&Instruction::Else);
        self.emit_array_read(
            self.argv_param_local(),
            index_local,
            payload_local,
            tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.release_temp_local(index_local);
    }

    pub(crate) fn store_payload_to_binding(
        &self,
        storage: BindingStorage,
        function: &mut Function,
    ) {
        match storage {
            BindingStorage::Fixed { payload_local, .. } => {
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::Dynamic { .. } | BindingStorage::EnvSlot { .. } => {
                panic!("dynamic binding write needs tagged path");
            }
        }
    }

    pub(crate) fn read_binding_payload(
        &mut self,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match storage {
            BindingStorage::Fixed { payload_local, .. }
            | BindingStorage::Dynamic { payload_local, .. } => {
                function.instruction(&Instruction::LocalGet(payload_local));
            }
            BindingStorage::EnvSlot { .. } => {
                self.read_binding_to_locals(
                    storage,
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                )?;
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
        }
        Ok(())
    }

    pub(crate) fn read_binding_to_locals(
        &mut self,
        storage: BindingStorage,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match storage {
            BindingStorage::Fixed {
                payload_local: binding_payload_local,
                kind,
            } => {
                function.instruction(&Instruction::LocalGet(binding_payload_local));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::I64Const(kind.tag() as i64));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            BindingStorage::Dynamic {
                tag_local: binding_tag_local,
                payload_local: binding_payload_local,
            } => {
                function.instruction(&Instruction::LocalGet(binding_payload_local));
                function.instruction(&Instruction::LocalSet(payload_local));
                function.instruction(&Instruction::LocalGet(binding_tag_local));
                function.instruction(&Instruction::LocalSet(tag_local));
            }
            BindingStorage::EnvSlot { slot, hops } => {
                self.read_env_slot_to_locals(slot, hops, payload_local, tag_local, function);
                function.instruction(&Instruction::LocalGet(tag_local));
                function.instruction(&Instruction::I64Const(ENV_SLOT_UNINITIALIZED_TAG));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_throw_runtime_error(
                    "ReferenceError",
                    "lexical binding accessed before initialization",
                    payload_local,
                    tag_local,
                    function,
                )?;
                if let Some(target) = self.active_throw_target() {
                    self.emit_branch_to_target(target, 1, function);
                } else {
                    self.emit_return_current_completion(function);
                }
                function.instruction(&Instruction::End);
            }
        }
        Ok(())
    }

    pub(crate) fn allocate_binding(
        &mut self,
        name: String,
        mode: BindingMode,
        kind: ValueKind,
    ) -> BindingStorage {
        let storage = match mode {
            BindingMode::Let | BindingMode::Const if self.owned_env_slot(&name).is_some() => {
                BindingStorage::EnvSlot {
                    slot: self
                        .owned_env_slot(&name)
                        .expect("owned env slot should exist"),
                    hops: 0,
                }
            }
            BindingMode::Let => {
                let tag_local = self.next_binding_local;
                let payload_local = self.next_binding_local + 1;
                self.next_binding_local += 2;
                BindingStorage::Dynamic {
                    tag_local,
                    payload_local,
                }
            }
            BindingMode::Const
                if matches!(
                    kind,
                    ValueKind::Undefined
                        | ValueKind::Object
                        | ValueKind::Function
                        | ValueKind::Array
                        | ValueKind::BigInt
                        | ValueKind::Dynamic
                ) =>
            {
                let tag_local = self.next_binding_local;
                let payload_local = self.next_binding_local + 1;
                self.next_binding_local += 2;
                BindingStorage::Dynamic {
                    tag_local,
                    payload_local,
                }
            }
            BindingMode::Const => {
                let payload_local = self.next_binding_local;
                self.next_binding_local += 1;
                BindingStorage::Fixed {
                    payload_local,
                    kind,
                }
            }
            BindingMode::Var => panic!("var bindings are hoisted"),
        };
        self.binding_scopes
            .last_mut()
            .expect("binding scope stack must exist")
            .insert(name, storage);
        storage
    }

    pub(crate) fn lookup_binding(&self, name: &str) -> Option<BindingStorage> {
        self.binding_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    pub(crate) fn lookup_current_scope_binding(&self, name: &str) -> Option<BindingStorage> {
        self.binding_scopes
            .last()
            .and_then(|scope| scope.get(name).copied())
    }

    pub(crate) fn lookup_owner_binding(&self, name: &str) -> Option<BindingStorage> {
        self.binding_scopes
            .first()
            .and_then(|scope| scope.get(name).copied())
    }

    pub(crate) fn push_scope(&mut self) {
        self.binding_scopes.push(BTreeMap::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.binding_scopes.pop();
    }

    pub(crate) fn owned_env_slot(&self, name: &str) -> Option<u32> {
        self.owned_env_bindings
            .iter()
            .find(|binding| binding.name == name)
            .map(|binding| binding.slot)
    }

    /// Resolves a compiler-private derived-constructor activation binding.
    /// The owner has an owned slot; arrows reach the same slot through their
    /// recorded captured binding and hop count.
    fn derived_activation_storage(&self, name: &str) -> Result<BindingStorage, EmitError> {
        self.owned_env_slot(name)
            .map(|slot| BindingStorage::EnvSlot { slot, hops: 0 })
            .or_else(|| self.lookup_binding(name))
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "derived constructor activation is missing compiler-private binding `{name}`"
                ))
            })
    }

    fn emit_derived_this_reference_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error(
            "ReferenceError",
            message,
            payload_local,
            tag_local,
            function,
        )?;
        if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, 1, function);
        } else {
            self.emit_return_current_completion(function);
        }
        Ok(())
    }

    /// Reads the live derived `this` after checking its per-invocation
    /// initialization status.  The backing slot can be owned by the derived
    /// constructor or captured by an arrow through environment hops.
    pub(crate) fn emit_get_derived_this_to_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation = self.lexical_derived_activation.ok_or_else(|| {
            EmitError::unsupported("derived `this` requested without activation metadata")
        })?;
        let status = self.derived_activation_storage(&activation.this_status_binding)?;
        let this = self.derived_activation_storage(&activation.this_binding)?;
        let status_payload_local = self.reserve_temp_local();
        let status_tag_local = self.reserve_temp_local();
        self.read_binding_to_locals(status, status_payload_local, status_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(status_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_derived_this_reference_error(
            "must call super() before accessing `this`",
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.read_binding_to_locals(this, payload_local, tag_local, function)?;
        self.release_temp_local(status_tag_local);
        self.release_temp_local(status_payload_local);
        Ok(())
    }

    pub(crate) fn emit_get_derived_new_target_to_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation = self.lexical_derived_activation.ok_or_else(|| {
            EmitError::unsupported("derived new.target requested without activation metadata")
        })?;
        let storage = self.derived_activation_storage(&activation.new_target_binding)?;
        self.read_binding_to_locals(storage, payload_local, tag_local, function)?;
        Ok(())
    }

    pub(crate) fn emit_get_derived_active_function_to_locals(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation = self.lexical_derived_activation.ok_or_else(|| {
            EmitError::unsupported("derived active function requested without activation metadata")
        })?;
        let storage = self.derived_activation_storage(&activation.active_function_binding)?;
        self.read_binding_to_locals(storage, payload_local, tag_local, function)?;
        Ok(())
    }

    /// Initializes a derived constructor's `this` exactly once.  A second
    /// bind raises ReferenceError before any activation slot is overwritten.
    pub(crate) fn emit_bind_derived_this_from_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        error_payload_local: u32,
        error_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let activation = self.lexical_derived_activation.ok_or_else(|| {
            EmitError::unsupported("derived `this` bind requested without activation metadata")
        })?;
        let status = self.derived_activation_storage(&activation.this_status_binding)?;
        let this = self.derived_activation_storage(&activation.this_binding)?;
        let status_payload_local = self.reserve_temp_local();
        let status_tag_local = self.reserve_temp_local();
        self.read_binding_to_locals(status, status_payload_local, status_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(status_payload_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_derived_this_reference_error(
            "super() called twice in derived constructor",
            error_payload_local,
            error_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.write_binding_from_locals(this, value_payload_local, value_tag_local, function);
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(status_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(status_tag_local));
        self.write_binding_from_locals(status, status_payload_local, status_tag_local, function);
        self.release_temp_local(status_tag_local);
        self.release_temp_local(status_payload_local);
        Ok(())
    }

    pub(crate) fn emit_global_property_read(
        &mut self,
        name: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_read_without_throw_propagation(
            object_local,
            object_tag_local,
            object_local,
            object_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        self.release_temp_local(key_local);
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)
    }

    pub(crate) fn emit_global_property_write(
        &mut self,
        name: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_write(
            object_local,
            object_tag_local,
            key_local,
            payload_local,
            tag_local,
            function,
        )?;
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn emit_global_property_delete(
        &mut self,
        name: &str,
        result_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        let object_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(object_tag_local));
        self.emit_object_delete(
            object_local,
            object_tag_local,
            key_local,
            result_local,
            function,
        )?;
        self.release_temp_local(object_tag_local);
        self.release_temp_local(object_local);
        self.release_temp_local(key_local);
        Ok(())
    }

    pub(crate) fn mirror_binding_to_global_object(
        &mut self,
        name: &str,
        storage: BindingStorage,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.is_main() || !self.is_script_global_binding(name) {
            return Ok(());
        }
        if self
            .binding_scopes
            .first()
            .and_then(|scope| scope.get(name))
            .is_none_or(|global_storage| *global_storage != storage)
        {
            return Ok(());
        }

        let key_local = self.reserve_temp_local();
        let object_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(key_local));
        self.read_binding_to_locals(storage, self.scratch_local, self.result_tag_local, function)?;
        function.instruction(&Instruction::GlobalGet(SCRIPT_GLOBAL_OBJECT_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(object_local));
        self.emit_object_overwrite_own_data_or_define(
            object_local,
            key_local,
            self.scratch_local,
            self.result_tag_local,
            function,
        )?;
        self.release_temp_local(object_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
