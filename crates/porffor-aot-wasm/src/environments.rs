use super::*;

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn loop_binding_needs_iteration_env(&self, mode: BindingMode, name: &str) -> bool {
        mode != BindingMode::Var && self.owned_env_slot(name).is_some()
    }

    pub(crate) fn capture_iteration_env_source(
        &mut self,
        mode: BindingMode,
        name: &str,
        function: &mut Function,
    ) -> Option<u32> {
        if !self.loop_binding_needs_iteration_env(mode, name) {
            return None;
        }
        let source_env_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(source_env_local));
        Some(source_env_local)
    }

    pub(crate) fn emit_enter_iteration_env(
        &mut self,
        source_env_local: Option<u32>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(source_env_local) = source_env_local else {
            return Ok(());
        };
        let parent_env_local = self.reserve_temp_local();
        let value_local = self.reserve_temp_local();
        let slots = self
            .owned_env_bindings
            .iter()
            .map(|binding| binding.slot)
            .collect::<Vec<_>>();

        self.load_i64_to_local_from_offset(
            source_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        self.emit_heap_alloc_const(
            ENV_SLOT_BASE_OFFSET + self.owned_env_bindings.len() as u64 * ENV_SLOT_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.store_i64_local_at_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        for slot in slots {
            self.load_i64_to_local_from_offset(
                source_env_local,
                Self::env_slot_offset(slot, ENV_SLOT_TAG_OFFSET),
                value_local,
                function,
            );
            self.store_i64_local_at_offset(
                self.current_env_local,
                Self::env_slot_offset(slot, ENV_SLOT_TAG_OFFSET),
                value_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                source_env_local,
                Self::env_slot_offset(slot, ENV_SLOT_PAYLOAD_OFFSET),
                value_local,
                function,
            );
            self.store_i64_local_at_offset(
                self.current_env_local,
                Self::env_slot_offset(slot, ENV_SLOT_PAYLOAD_OFFSET),
                value_local,
                function,
            );
        }

        self.release_temp_local(value_local);
        self.release_temp_local(parent_env_local);
        Ok(())
    }

    pub(crate) fn emit_exit_iteration_env(
        &mut self,
        source_env_local: Option<u32>,
        loop_binding_name: &str,
        function: &mut Function,
    ) {
        let Some(source_env_local) = source_env_local else {
            return;
        };
        let loop_slot = self.owned_env_slot(loop_binding_name);
        let slots = self
            .owned_env_bindings
            .iter()
            .map(|binding| binding.slot)
            .filter(|slot| Some(*slot) != loop_slot)
            .collect::<Vec<_>>();
        let value_local = self.reserve_temp_local();

        for slot in slots {
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                Self::env_slot_offset(slot, ENV_SLOT_TAG_OFFSET),
                value_local,
                function,
            );
            self.store_i64_local_at_offset(
                source_env_local,
                Self::env_slot_offset(slot, ENV_SLOT_TAG_OFFSET),
                value_local,
                function,
            );
            self.load_i64_to_local_from_offset(
                self.current_env_local,
                Self::env_slot_offset(slot, ENV_SLOT_PAYLOAD_OFFSET),
                value_local,
                function,
            );
            self.store_i64_local_at_offset(
                source_env_local,
                Self::env_slot_offset(slot, ENV_SLOT_PAYLOAD_OFFSET),
                value_local,
                function,
            );
        }

        function.instruction(&Instruction::LocalGet(source_env_local));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.release_temp_local(value_local);
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
        if matches!(self.return_abi, ReturnAbi::MultiValue)
            && self.function_flavor == FunctionFlavor::Ordinary
        {
            let payload_local = self.next_binding_local;
            let tag_local = self.next_binding_local + 1;
            self.next_binding_local += 2;
            let arguments_storage = BindingStorage::Dynamic {
                tag_local,
                payload_local,
            };
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(LEXICAL_ARGUMENTS_NAME.to_string(), arguments_storage);
            self.initialize_arguments_binding(arguments_storage, function)?;
            if let Some(slot) = self.owned_env_slot(LEXICAL_ARGUMENTS_NAME) {
                self.write_env_slot_from_locals(slot, 0, payload_local, tag_local, function);
            }
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
                continue;
            }
            self.initialize_parameter(index, param, storage, function)?;
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
        let (Some(function_id), Some(function_name)) =
            (self.function_id.as_ref(), self.self_binding_name.as_ref())
        else {
            return Ok(());
        };
        let Some(meta) = self.functions.get(function_id) else {
            return Err(EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: unknown function `{function_id}`"
            )));
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
        self.emit_function_value_payload(meta, function)?;
        match storage {
            BindingStorage::Fixed { payload_local, .. } => {
                function.instruction(&Instruction::LocalSet(payload_local));
            }
            BindingStorage::EnvSlot { .. } => {
                function.instruction(&Instruction::LocalSet(self.scratch_local));
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
                if let Some(derived_this_initialized_local) = self.derived_this_initialized_local {
                    function.instruction(&Instruction::LocalGet(derived_this_initialized_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_throw_runtime_error(
                        "ReferenceError",
                        "must call super() before accessing `this`",
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    if !self.is_main() {
                        self.emit_return_current_completion(function);
                    } else if let Some(target) = self.throw_handler_stack.last() {
                        function.instruction(&Instruction::Br(self.depth_to(*target) + 1));
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
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
                if let Some(storage) = self.lookup_binding(LEXICAL_THIS_NAME) {
                    self.read_binding_payload(storage, function);
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
                if let Some(derived_this_initialized_local) = self.derived_this_initialized_local {
                    function.instruction(&Instruction::LocalGet(derived_this_initialized_local));
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_throw_runtime_error(
                        "ReferenceError",
                        "must call super() before accessing `this`",
                        payload_local,
                        tag_local,
                        function,
                    )?;
                    if !self.is_main() {
                        self.emit_return_current_completion(function);
                    } else if let Some(target) = self.throw_handler_stack.last() {
                        function.instruction(&Instruction::Br(self.depth_to(*target) + 1));
                    } else {
                        self.emit_return_current_completion(function);
                    }
                    function.instruction(&Instruction::End);
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
                if let Some(storage) = self.lookup_binding(LEXICAL_THIS_NAME) {
                    self.read_binding_to_locals(storage, payload_local, tag_local, function);
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
    ) {
        if self.function_flavor == FunctionFlavor::Arrow {
            if let Some(storage) = self.lookup_binding(LEXICAL_NEW_TARGET_NAME) {
                self.read_binding_to_locals(storage, payload_local, tag_local, function);
                return;
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
    /// indexed slots below `min(argc, param_count)` alias the parameter
    /// bindings. The aliasing machinery (`HEAP_ARGUMENTS_MAPPED_COUNT_OFFSET`
    /// + env-handle indirection in `emit_arguments_read`/`emit_arguments_
    /// write`) addresses parameter env slots by argument index, which is only
    /// valid because analysis pre-assigns simple parameters env slots
    /// `0..n-1` in declaration order (`collect_function_plan`); the
    /// `owned_env_slot` check below defends that invariant (destructured
    /// params, for example, get no pre-assigned slots).
    pub(crate) fn uses_mapped_arguments_object(&self) -> bool {
        self.function_flavor == FunctionFlavor::Ordinary
            && !self.is_current_function_strict()
            && self.has_simple_parameter_list()
            && self
                .params
                .iter()
                .enumerate()
                .all(|(index, param)| self.owned_env_slot(&param.name) == Some(index as u32))
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
    ) {
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
                );
                function.instruction(&Instruction::LocalGet(self.scratch_local));
            }
        }
    }

    pub(crate) fn read_binding_to_locals(
        &mut self,
        storage: BindingStorage,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) {
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
            }
        }
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
            BindingMode::Let | BindingMode::Const
                if matches!(
                    kind,
                    ValueKind::Undefined
                        | ValueKind::Object
                        | ValueKind::Function
                        | ValueKind::Array
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
            BindingMode::Let | BindingMode::Const => {
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
        self.emit_object_read(
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
        Ok(())
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
        if !self.is_script_global_binding(name) {
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
        self.read_binding_to_locals(storage, self.scratch_local, self.result_tag_local, function);
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
