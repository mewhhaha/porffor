use super::*;

pub(crate) const DIRECT_EVAL_CLASS_CONTEXT_OFFSET: u64 = 0;
pub(crate) const DIRECT_EVAL_HOME_OBJECT_PAYLOAD_OFFSET: u64 = 8;
pub(crate) const DIRECT_EVAL_HOME_OBJECT_TAG_OFFSET: u64 = 16;
pub(crate) const DIRECT_EVAL_THIS_CELL_OFFSET: u64 = 24;
pub(crate) const DIRECT_EVAL_THIS_STATUS_CELL_OFFSET: u64 = 32;
pub(crate) const DIRECT_EVAL_NEW_TARGET_CELL_OFFSET: u64 = 40;
pub(crate) const DIRECT_EVAL_ACTIVE_FUNCTION_CELL_OFFSET: u64 = 48;
pub(crate) const DIRECT_EVAL_THIS_PAYLOAD_OFFSET: u64 = 56;
pub(crate) const DIRECT_EVAL_THIS_TAG_OFFSET: u64 = 64;
pub(crate) const DIRECT_EVAL_NEW_TARGET_PAYLOAD_OFFSET: u64 = 72;
pub(crate) const DIRECT_EVAL_NEW_TARGET_TAG_OFFSET: u64 = 80;
const DIRECT_EVAL_EXECUTION_CONTEXT_SIZE: u64 = 88;

#[must_use = "captured direct-eval invocation locals must be passed and released"]
pub(crate) struct DirectEvalInvocationLocals {
    pub(crate) this_payload: u32,
    pub(crate) this_tag: u32,
    pub(crate) new_target_payload: u32,
    pub(crate) new_target_tag: u32,
    pub(crate) execution_context: u32,
}

impl FunctionBuilder<'_> {
    pub(crate) fn initialize_direct_eval_execution_context(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let Some(context) = self.direct_eval_execution_context_local() else {
            return Ok(());
        };
        if self.captured_direct_eval_execution_context_local.is_some() {
            let capture = self
                .captured_bindings
                .iter()
                .find(|binding| binding.name == lila_ir::DIRECT_EVAL_EXECUTION_CONTEXT_NAME)
                .expect("captured context local is derived from the capture plan");
            self.read_env_slot_to_locals(
                capture.slot,
                capture.hops,
                context,
                self.scratch_local,
                function,
            );
        } else {
            let slot = self
                .owned_env_slot(lila_ir::DIRECT_EVAL_EXECUTION_CONTEXT_NAME)
                .expect("direct Script owns its source-unspellable context cell");
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(self.scratch_local));
            self.write_env_slot_from_locals(slot, 0, context, self.scratch_local, function);
        }
        self.load_i64_to_local_from_offset(
            context,
            DIRECT_EVAL_CLASS_CONTEXT_OFFSET,
            self.class_function_context_local,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_capture_direct_eval_invocation(
        &mut self,
        function: &mut Function,
    ) -> Result<DirectEvalInvocationLocals, EmitError> {
        let invocation = DirectEvalInvocationLocals {
            this_payload: self.reserve_temp_local(),
            this_tag: self.reserve_temp_local(),
            new_target_payload: self.reserve_temp_local(),
            new_target_tag: self.reserve_temp_local(),
            execution_context: self.reserve_temp_local(),
        };
        if let Some(context) = self.direct_eval_execution_context_local() {
            function.instruction(&Instruction::LocalGet(context));
            function.instruction(&Instruction::LocalSet(invocation.execution_context));
        } else {
            self.emit_heap_alloc_const(DIRECT_EVAL_EXECUTION_CONTEXT_SIZE, function)?;
            function.instruction(&Instruction::LocalSet(invocation.execution_context));
            self.store_i64_local_at_offset(
                invocation.execution_context,
                DIRECT_EVAL_CLASS_CONTEXT_OFFSET,
                self.class_function_context_local,
                function,
            );
            for offset in [
                DIRECT_EVAL_THIS_CELL_OFFSET,
                DIRECT_EVAL_THIS_STATUS_CELL_OFFSET,
                DIRECT_EVAL_NEW_TARGET_CELL_OFFSET,
                DIRECT_EVAL_ACTIVE_FUNCTION_CELL_OFFSET,
            ] {
                self.store_i64_const_at_offset(invocation.execution_context, offset, 0, function);
            }
            if let Some(activation) = self.lexical_derived_activation.cloned() {
                let cell = self.reserve_temp_local();
                for (name, offset) in [
                    (&activation.this_binding, DIRECT_EVAL_THIS_CELL_OFFSET),
                    (
                        &activation.this_status_binding,
                        DIRECT_EVAL_THIS_STATUS_CELL_OFFSET,
                    ),
                    (
                        &activation.new_target_binding,
                        DIRECT_EVAL_NEW_TARGET_CELL_OFFSET,
                    ),
                    (
                        &activation.active_function_binding,
                        DIRECT_EVAL_ACTIVE_FUNCTION_CELL_OFFSET,
                    ),
                ] {
                    let storage = self.derived_activation_storage(name)?;
                    self.emit_direct_eval_binding_cell(storage, cell, function);
                    self.store_i64_local_at_offset(
                        invocation.execution_context,
                        offset,
                        cell,
                        function,
                    );
                }
                self.emit_get_derived_active_function_to_locals(
                    invocation.this_payload,
                    invocation.this_tag,
                    function,
                )?;
                self.load_i64_to_local_from_offset(
                    invocation.this_payload,
                    HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                    cell,
                    function,
                );
                self.store_i64_local_at_offset(
                    invocation.execution_context,
                    DIRECT_EVAL_CLASS_CONTEXT_OFFSET,
                    cell,
                    function,
                );
                self.release_temp_local(cell);
            }
            let home_payload = self.reserve_temp_local();
            let home_tag = self.reserve_temp_local();
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(home_payload));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(home_tag));
            if self.function_flavor == FunctionFlavor::Arrow {
                if let Some(storage) = self.lookup_binding(LEXICAL_HOME_OBJECT_NAME) {
                    self.read_binding_to_locals(storage, home_payload, home_tag, function)?;
                }
            } else {
                let class_context = self.reserve_temp_local();
                self.load_i64_to_local_from_offset(
                    invocation.execution_context,
                    DIRECT_EVAL_CLASS_CONTEXT_OFFSET,
                    class_context,
                    function,
                );
                function.instruction(&Instruction::LocalGet(class_context));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    class_context,
                    HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                    home_payload,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    class_context,
                    HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                    home_tag,
                    function,
                );
                function.instruction(&Instruction::End);
                self.release_temp_local(class_context);
            }
            self.store_i64_local_at_offset(
                invocation.execution_context,
                DIRECT_EVAL_HOME_OBJECT_PAYLOAD_OFFSET,
                home_payload,
                function,
            );
            self.store_i64_local_at_offset(
                invocation.execution_context,
                DIRECT_EVAL_HOME_OBJECT_TAG_OFFSET,
                home_tag,
                function,
            );
            self.release_temp_local(home_tag);
            self.release_temp_local(home_payload);
        }
        let this_cell = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            invocation.execution_context,
            DIRECT_EVAL_THIS_CELL_OFFSET,
            this_cell,
            function,
        );
        function.instruction(&Instruction::LocalGet(this_cell));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if self.lexical_derived_activation.is_none() {
            self.compile_this_to_locals(invocation.this_payload, invocation.this_tag, function)?;
            self.compile_new_target_to_locals(
                invocation.new_target_payload,
                invocation.new_target_tag,
                function,
            )?;
        } else {
            // A derived activation always published a shared cell above.
            function.instruction(&Instruction::Unreachable);
        }
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            this_cell,
            ENV_SLOT_PAYLOAD_OFFSET,
            invocation.this_payload,
            function,
        );
        self.load_i64_to_local_from_offset(
            this_cell,
            ENV_SLOT_TAG_OFFSET,
            invocation.this_tag,
            function,
        );
        self.load_i64_to_local_from_offset(
            invocation.execution_context,
            DIRECT_EVAL_NEW_TARGET_CELL_OFFSET,
            this_cell,
            function,
        );
        self.load_i64_to_local_from_offset(
            this_cell,
            ENV_SLOT_PAYLOAD_OFFSET,
            invocation.new_target_payload,
            function,
        );
        self.load_i64_to_local_from_offset(
            this_cell,
            ENV_SLOT_TAG_OFFSET,
            invocation.new_target_tag,
            function,
        );
        function.instruction(&Instruction::End);
        self.release_temp_local(this_cell);
        for (offset, value) in [
            (DIRECT_EVAL_THIS_PAYLOAD_OFFSET, invocation.this_payload),
            (DIRECT_EVAL_THIS_TAG_OFFSET, invocation.this_tag),
            (
                DIRECT_EVAL_NEW_TARGET_PAYLOAD_OFFSET,
                invocation.new_target_payload,
            ),
            (DIRECT_EVAL_NEW_TARGET_TAG_OFFSET, invocation.new_target_tag),
        ] {
            self.store_i64_local_at_offset(invocation.execution_context, offset, value, function);
        }
        Ok(invocation)
    }

    fn emit_direct_eval_binding_cell(
        &self,
        storage: BindingStorage,
        cell: u32,
        function: &mut Function,
    ) {
        let BindingStorage::EnvSlot { slot, hops } = storage else {
            panic!("shared derived-constructor state must have an environment cell");
        };
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(cell));
        for _ in 0..hops {
            self.load_i64_to_local_from_offset(cell, ENV_PARENT_OFFSET, cell, function);
        }
        function.instruction(&Instruction::LocalGet(cell));
        function.instruction(&Instruction::I64Const(
            (ENV_SLOT_BASE_OFFSET + u64::from(slot) * ENV_SLOT_SIZE) as i64,
        ));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cell));
    }

    pub(crate) fn release_direct_eval_invocation(
        &mut self,
        invocation: DirectEvalInvocationLocals,
    ) {
        self.release_temp_local(invocation.execution_context);
        self.release_temp_local(invocation.new_target_tag);
        self.release_temp_local(invocation.new_target_payload);
        self.release_temp_local(invocation.this_tag);
        self.release_temp_local(invocation.this_payload);
    }
}
