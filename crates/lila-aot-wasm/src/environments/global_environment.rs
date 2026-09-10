use super::*;

// A Global Environment terminates the ordinary lexical parent chain. Its
// declarative entries refer to the same cells used by captured lexical reads.
pub(crate) const GLOBAL_ENV_REALM_OFFSET: u64 = 8;
pub(crate) const GLOBAL_ENV_LEXICAL_ENTRIES_OFFSET: u64 = 16;
pub(crate) const GLOBAL_ENV_LEXICAL_COUNT_OFFSET: u64 = 24;
const GLOBAL_ENV_SIZE: u64 = 32;
pub(crate) const GLOBAL_LEXICAL_KEY_OFFSET: u64 = 0;
pub(crate) const GLOBAL_LEXICAL_CELL_OFFSET: u64 = 8;
pub(crate) const GLOBAL_LEXICAL_MUTABLE_OFFSET: u64 = 16;
pub(crate) const GLOBAL_LEXICAL_ENTRY_SIZE: u64 = 24;

pub(crate) enum GlobalBindingFailure {
    BeforeInitialization,
    Immutable,
    Unresolvable,
    UnresolvableAssignment,
    NonConfigurableDelete,
}

pub(crate) enum SourceLiteralPrototype {
    Object,
    Array,
    RegExp,
}

impl FunctionBuilder<'_> {
    pub(crate) fn has_source_execution_environment(&self) -> bool {
        self.is_main()
            || self
                .current_function_meta()
                .is_some_and(|meta| meta.standard_builtin.is_none() && meta.host_builtin.is_none())
    }

    pub(crate) fn emit_alloc_realm_global_environment(
        &mut self,
        realm_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let environment_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(GLOBAL_ENV_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(environment_local));
        self.store_i64_const_at_offset(environment_local, ENV_PARENT_OFFSET, 0, function);
        self.store_i64_local_at_offset(
            environment_local,
            GLOBAL_ENV_REALM_OFFSET,
            realm_local,
            function,
        );
        self.store_i64_const_at_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_ENTRIES_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_COUNT_OFFSET,
            0,
            function,
        );
        self.store_i64_local_at_offset(
            realm_local,
            HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
            environment_local,
            function,
        );
        self.release_temp_local(environment_local);
        Ok(())
    }

    pub(crate) fn emit_source_global_environment_to_local(
        &mut self,
        environment_local: u32,
        function: &mut Function,
    ) {
        assert!(self.has_source_execution_environment());
        let parent_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(environment_local));
        function.instruction(&Instruction::LocalGet(environment_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_PARENT_OFFSET,
            parent_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(parent_local));
        function.instruction(&Instruction::LocalSet(environment_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(parent_local);
    }

    pub(crate) fn emit_source_execution_realm_to_local(
        &mut self,
        realm_local: u32,
        function: &mut Function,
    ) {
        self.emit_source_global_environment_to_local(realm_local, function);
        self.load_i64_to_local_from_offset(
            realm_local,
            GLOBAL_ENV_REALM_OFFSET,
            realm_local,
            function,
        );
    }

    pub(crate) fn emit_execution_global_object_payload(&mut self, function: &mut Function) {
        let realm_local = self.reserve_temp_local();
        if self.has_source_execution_environment() {
            self.emit_source_execution_realm_to_local(realm_local, function);
        } else {
            function.instruction(&Instruction::GlobalGet(CURRENT_REALM_GLOBAL_INDEX));
            function.instruction(&Instruction::LocalSet(realm_local));
        }
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_GLOBAL_OBJECT_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_source_literal_prototype_payload(
        &mut self,
        prototype: SourceLiteralPrototype,
        function: &mut Function,
    ) {
        let (offset, entry_global) = match prototype {
            SourceLiteralPrototype::Object => (
                HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET,
                OBJECT_PROTOTYPE_GLOBAL_INDEX,
            ),
            SourceLiteralPrototype::Array => (
                HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET,
                ARRAY_PROTOTYPE_GLOBAL_INDEX,
            ),
            SourceLiteralPrototype::RegExp => (
                HEAP_REALM_INTRINSICS_REGEXP_PROTOTYPE_OFFSET,
                REGEXP_PROTOTYPE_GLOBAL_INDEX,
            ),
        };
        if !self.has_source_execution_environment() {
            function.instruction(&Instruction::GlobalGet(entry_global));
            return;
        }
        let prototype_local = self.reserve_temp_local();
        self.emit_source_execution_realm_to_local(prototype_local, function);
        self.load_i64_to_local_from_offset(
            prototype_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            prototype_local,
            function,
        );
        self.load_i64_to_local_from_offset(prototype_local, offset, prototype_local, function);
        function.instruction(&Instruction::LocalGet(prototype_local));
        self.release_temp_local(prototype_local);
    }

    pub(crate) fn emit_source_realm_function_context_payload(&mut self, function: &mut Function) {
        let context_local = self.reserve_temp_local();
        self.emit_source_execution_realm_to_local(context_local, function);
        self.load_i64_to_local_from_offset(
            context_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            context_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            context_local,
            HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
            context_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(context_local));
        self.release_temp_local(context_local);
    }

    pub(crate) fn emit_install_dynamic_function_global_environment(
        &mut self,
        meta: &WasmFunctionMeta,
        object_local: u32,
        realm_local: u32,
        function: &mut Function,
    ) {
        assert!(
            !meta.is_named_expression,
            "dynamic function body cannot own a named-expression environment"
        );
        let global_environment_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_GLOBAL_ENVIRONMENT_OFFSET,
            global_environment_local,
            function,
        );
        if meta.has_function_context() {
            let context_local = self.reserve_temp_local();
            self.load_i64_to_local_from_offset(
                object_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                context_local,
                function,
            );
            self.store_i64_local_at_offset(
                context_local,
                HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
                global_environment_local,
                function,
            );
            self.release_temp_local(context_local);
        } else {
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                global_environment_local,
                function,
            );
        }
        self.release_temp_local(global_environment_local);
    }

    pub(crate) fn emit_install_source_function_execution_realm(
        &mut self,
        meta: &WasmFunctionMeta,
        object_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        let intrinsics_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let instance_prototype_local = self.reserve_temp_local();
        self.emit_source_execution_realm_to_local(realm_local, function);
        self.emit_store_function_defining_realm(object_local, realm_local, function);
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_INTRINSICS_OFFSET,
            intrinsics_local,
            function,
        );
        let (function_prototype_offset, function_prototype_tag, instance_prototype_offset) =
            match meta.protocol.execution_kind() {
                FunctionExecutionKind::Ordinary => (
                    HEAP_REALM_INTRINSICS_FUNCTION_PROTOTYPE_OFFSET,
                    ValueKind::Function,
                    meta.protocol
                        .is_constructable()
                        .then_some(HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET),
                ),
                FunctionExecutionKind::Generator => (
                    HEAP_REALM_INTRINSICS_GENERATOR_FUNCTION_PROTOTYPE_OFFSET,
                    ValueKind::Object,
                    Some(HEAP_REALM_INTRINSICS_GENERATOR_PROTOTYPE_OFFSET),
                ),
                FunctionExecutionKind::Async => (
                    HEAP_REALM_INTRINSICS_ASYNC_FUNCTION_PROTOTYPE_OFFSET,
                    ValueKind::Object,
                    None,
                ),
                FunctionExecutionKind::AsyncGenerator => (
                    HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_FUNCTION_PROTOTYPE_OFFSET,
                    ValueKind::Object,
                    Some(HEAP_REALM_INTRINSICS_ASYNC_GENERATOR_PROTOTYPE_OFFSET),
                ),
            };
        self.load_i64_to_local_from_offset(
            intrinsics_local,
            function_prototype_offset,
            prototype_local,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_PROTOTYPE_OFFSET,
            prototype_local,
            function,
        );
        self.store_i64_const_at_offset(
            object_local,
            HEAP_FUNCTION_INTERNAL_PROTOTYPE_TAG_OFFSET,
            function_prototype_tag.tag() as u64,
            function,
        );
        if let Some(offset) = instance_prototype_offset {
            self.load_i64_to_local_from_offset(intrinsics_local, offset, prototype_local, function);
            self.load_i64_to_local_from_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                instance_prototype_local,
                function,
            );
            self.store_i64_local_at_offset(
                instance_prototype_local,
                HEAP_PROTOTYPE_OFFSET,
                prototype_local,
                function,
            );
        }
        self.release_temp_local(instance_prototype_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(intrinsics_local);
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_function_global_this_payload(
        &mut self,
        function_object_local: u32,
        function: &mut Function,
    ) {
        let realm_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            function_object_local,
            HEAP_FUNCTION_DEFINING_REALM_OFFSET,
            realm_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            realm_local,
            HEAP_REALM_GLOBAL_THIS_OFFSET,
            realm_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(realm_local));
        self.release_temp_local(realm_local);
    }

    pub(crate) fn emit_initialize_main_global_lexicals(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }
        let bindings = self
            .script_global_bindings
            .expect("main global binding plan");
        let lexicals = bindings.lexical_bindings().clone();
        let environment_local = self.reserve_temp_local();
        let entries_local = self.reserve_temp_local();
        let cell_local = self.reserve_temp_local();
        self.emit_source_global_environment_to_local(environment_local, function);
        self.emit_heap_alloc_const(lexicals.len() as u64 * GLOBAL_LEXICAL_ENTRY_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(entries_local));
        self.store_i64_local_at_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_ENTRIES_OFFSET,
            entries_local,
            function,
        );
        self.store_i64_const_at_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_COUNT_OFFSET,
            lexicals.len() as u64,
            function,
        );
        for (index, (name, mode)) in lexicals.iter().enumerate() {
            let offset = index as u64 * GLOBAL_LEXICAL_ENTRY_SIZE;
            let slot = self
                .owned_env_slot(name)
                .expect("global lexical must own a cell");
            self.store_i64_const_at_offset(
                entries_local,
                offset + GLOBAL_LEXICAL_KEY_OFFSET,
                self.strings.payload(name) as u64,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.current_env_local));
            function.instruction(&Instruction::I64Const(Self::env_slot_offset(slot, 0) as i64));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(cell_local));
            self.store_i64_local_at_offset(
                entries_local,
                offset + GLOBAL_LEXICAL_CELL_OFFSET,
                cell_local,
                function,
            );
            self.store_i64_const_at_offset(
                entries_local,
                offset + GLOBAL_LEXICAL_MUTABLE_OFFSET,
                match mode {
                    lila_ir::GlobalLexicalBindingModeIr::Mutable => 1,
                    lila_ir::GlobalLexicalBindingModeIr::Immutable => 0,
                },
                function,
            );
        }
        self.release_temp_local(cell_local);
        self.release_temp_local(entries_local);
        self.release_temp_local(environment_local);
        Ok(())
    }

    pub(crate) fn emit_global_lexical_entry_to_local(
        &mut self,
        key_local: u32,
        entry_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(entry_local));
        if !self.has_source_execution_environment() {
            return;
        }
        let environment_local = self.reserve_temp_local();
        let cursor_local = self.reserve_temp_local();
        let remaining_local = self.reserve_temp_local();
        let entry_key_local = self.reserve_temp_local();
        self.emit_source_global_environment_to_local(environment_local, function);
        self.load_i64_to_local_from_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_ENTRIES_OFFSET,
            cursor_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            environment_local,
            GLOBAL_ENV_LEXICAL_COUNT_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        self.load_i64_to_local_from_offset(
            cursor_local,
            GLOBAL_LEXICAL_KEY_OFFSET,
            entry_key_local,
            function,
        );
        self.emit_string_payload_equality_i32(key_local, entry_key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::Br(2));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(cursor_local));
        function.instruction(&Instruction::I64Const(GLOBAL_LEXICAL_ENTRY_SIZE as i64));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(cursor_local));
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(entry_key_local);
        self.release_temp_local(remaining_local);
        self.release_temp_local(cursor_local);
        self.release_temp_local(environment_local);
    }

    pub(crate) fn emit_global_lexical_read(
        &mut self,
        entry_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let cell_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            entry_local,
            GLOBAL_LEXICAL_CELL_OFFSET,
            cell_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            cell_local,
            ENV_SLOT_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(cell_local, ENV_SLOT_TAG_OFFSET, tag_local, function);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ENV_SLOT_UNINITIALIZED_TAG));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_global_binding_error(
            GlobalBindingFailure::BeforeInitialization,
            payload_local,
            tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(payload_local, tag_local, function)?;
        function.instruction(&Instruction::End);
        self.release_temp_local(cell_local);
        Ok(())
    }

    pub(crate) fn emit_global_lexical_write(
        &mut self,
        entry_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let cell_local = self.reserve_temp_local();
        let previous_payload_local = self.reserve_temp_local();
        let previous_tag_local = self.reserve_temp_local();
        self.emit_global_lexical_read(
            entry_local,
            previous_payload_local,
            previous_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            entry_local,
            GLOBAL_LEXICAL_MUTABLE_OFFSET,
            cell_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(cell_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_global_binding_error(
            GlobalBindingFailure::Immutable,
            previous_payload_local,
            previous_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            previous_payload_local,
            previous_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            entry_local,
            GLOBAL_LEXICAL_CELL_OFFSET,
            cell_local,
            function,
        );
        self.store_i64_local_at_offset(
            cell_local,
            ENV_SLOT_PAYLOAD_OFFSET,
            payload_local,
            function,
        );
        self.store_i64_local_at_offset(cell_local, ENV_SLOT_TAG_OFFSET, tag_local, function);
        self.release_temp_local(previous_tag_local);
        self.release_temp_local(previous_payload_local);
        self.release_temp_local(cell_local);
        Ok(())
    }

    pub(crate) fn emit_throw_global_binding_error(
        &mut self,
        failure: GlobalBindingFailure,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let (name, message) = match failure {
            GlobalBindingFailure::BeforeInitialization => (
                REFERENCE_ERROR_NAME,
                "lexical binding accessed before initialization",
            ),
            GlobalBindingFailure::Immutable => (TYPE_ERROR_NAME, "assignment to constant binding"),
            GlobalBindingFailure::Unresolvable => (REFERENCE_ERROR_NAME, "unbound identifier"),
            GlobalBindingFailure::UnresolvableAssignment => {
                (REFERENCE_ERROR_NAME, "assignment to unresolvable reference")
            }
            GlobalBindingFailure::NonConfigurableDelete => {
                (TYPE_ERROR_NAME, "Cannot delete property")
            }
        };
        self.emit_throw_runtime_error(name, message, payload_local, tag_local, function)
    }
}
