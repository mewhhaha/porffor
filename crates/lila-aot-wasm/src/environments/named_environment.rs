use super::*;
use lila_ir::{
    EvalBindingDeclarationIr, EvalDeclarativeEnvironmentKindIr, EvalEnvironmentRoleIr,
    EvalVisibleBindingIr,
};

pub(crate) const NAMED_BINDING_KEY_OFFSET: u64 = 0;
pub(crate) const NAMED_BINDING_CELL_OFFSET: u64 = 8;
pub(crate) const NAMED_BINDING_MUTABLE_OFFSET: u64 = 16;
pub(crate) const NAMED_BINDING_DELETABLE_OFFSET: u64 = 24;
pub(crate) const NAMED_BINDING_PRESENT_OFFSET: u64 = 32;
pub(crate) const NAMED_BINDING_LEXICAL_CONFLICT_OFFSET: u64 = 40;
pub(crate) const NAMED_BINDING_IMMUTABLE_STRICT_OFFSET: u64 = 48;
pub(crate) const NAMED_BINDING_SIZE: u64 = 56;

#[derive(Clone, Copy)]
pub(crate) enum NamedEnvironmentKind {
    Unexposed,
    Variable,
    GlobalLexical,
    Lexical,
    Catch,
    SimpleCatch,
    WithObject,
}

impl NamedEnvironmentKind {
    pub(crate) const fn code(self) -> u64 {
        match self {
            Self::Unexposed => 0,
            Self::Variable => 1,
            Self::GlobalLexical => 5,
            Self::Lexical => 2,
            Self::Catch => 3,
            Self::SimpleCatch => 6,
            Self::WithObject => 4,
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_initialize_named_environment_header(
        &mut self,
        environment_local: u32,
        role: Option<&EvalEnvironmentRoleIr>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for offset in [
            ENV_FUNCTION_BODY_OFFSET,
            ENV_NAMED_ENTRIES_OFFSET,
            ENV_NAMED_COUNT_OFFSET,
            ENV_WITH_OBJECT_OFFSET,
            ENV_RECORD_KIND_OFFSET,
        ] {
            self.store_i64_const_at_offset(environment_local, offset, 0, function);
        }
        let kind = match role {
            None => NamedEnvironmentKind::Unexposed,
            Some(EvalEnvironmentRoleIr::Declarative { kind, bindings }) => {
                self.emit_publish_named_declarative_environment(
                    environment_local,
                    bindings,
                    function,
                )?;
                match kind {
                    EvalDeclarativeEnvironmentKindIr::Variable => NamedEnvironmentKind::Variable,
                    EvalDeclarativeEnvironmentKindIr::GlobalLexical => {
                        NamedEnvironmentKind::GlobalLexical
                    }
                    EvalDeclarativeEnvironmentKindIr::Lexical
                    | EvalDeclarativeEnvironmentKindIr::Parameters => NamedEnvironmentKind::Lexical,
                    EvalDeclarativeEnvironmentKindIr::Catch => NamedEnvironmentKind::Catch,
                    EvalDeclarativeEnvironmentKindIr::SimpleCatch => {
                        NamedEnvironmentKind::SimpleCatch
                    }
                }
            }
            Some(EvalEnvironmentRoleIr::WithObject { object_slot }) => {
                let cell_local = self.reserve_temp_local();
                function.instruction(&Instruction::LocalGet(environment_local));
                function.instruction(&Instruction::I64Const(
                    Self::env_slot_offset(*object_slot, 0) as i64,
                ));
                function.instruction(&Instruction::I64Add);
                function.instruction(&Instruction::LocalSet(cell_local));
                self.store_i64_local_at_offset(
                    environment_local,
                    ENV_WITH_OBJECT_OFFSET,
                    cell_local,
                    function,
                );
                self.release_temp_local(cell_local);
                NamedEnvironmentKind::WithObject
            }
        };
        self.store_i64_const_at_offset(
            environment_local,
            ENV_RECORD_KIND_OFFSET,
            kind.code(),
            function,
        );
        Ok(())
    }

    fn emit_publish_named_declarative_environment(
        &mut self,
        environment_local: u32,
        bindings: &[EvalVisibleBindingIr],
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries_local = self.reserve_temp_local();
        let cell_local = self.reserve_temp_local();
        self.emit_heap_alloc_const(bindings.len() as u64 * NAMED_BINDING_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(entries_local));
        for (index, binding) in bindings.iter().enumerate() {
            let base = index as u64 * NAMED_BINDING_SIZE;
            self.store_i64_const_at_offset(
                entries_local,
                base + NAMED_BINDING_KEY_OFFSET,
                self.strings.payload(&binding.source_name) as u64,
                function,
            );
            function.instruction(&Instruction::LocalGet(environment_local));
            function.instruction(&Instruction::I64Const(
                Self::env_slot_offset(binding.slot, 0) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(cell_local));
            self.store_i64_local_at_offset(
                entries_local,
                base + NAMED_BINDING_CELL_OFFSET,
                cell_local,
                function,
            );
            self.store_i64_const_at_offset(
                entries_local,
                base + NAMED_BINDING_MUTABLE_OFFSET,
                match binding.mode {
                    BindingMode::Let | BindingMode::Var => 1,
                    BindingMode::Const => 0,
                },
                function,
            );
            self.store_i64_const_at_offset(
                entries_local,
                base + NAMED_BINDING_DELETABLE_OFFSET,
                0,
                function,
            );
            self.store_i64_const_at_offset(
                entries_local,
                base + NAMED_BINDING_PRESENT_OFFSET,
                1,
                function,
            );
            self.store_i64_const_at_offset(
                entries_local,
                base + NAMED_BINDING_LEXICAL_CONFLICT_OFFSET,
                match binding.declaration {
                    EvalBindingDeclarationIr::Parameter | EvalBindingDeclarationIr::Variable => 0,
                    EvalBindingDeclarationIr::Lexical
                    | EvalBindingDeclarationIr::NamedFunctionExpression => 1,
                },
                function,
            );
            self.store_i64_const_at_offset(
                entries_local,
                base + NAMED_BINDING_IMMUTABLE_STRICT_OFFSET,
                match binding.declaration {
                    EvalBindingDeclarationIr::NamedFunctionExpression => 0,
                    EvalBindingDeclarationIr::Parameter
                    | EvalBindingDeclarationIr::Variable
                    | EvalBindingDeclarationIr::Lexical => 1,
                },
                function,
            );
        }
        self.store_i64_local_at_offset(
            environment_local,
            ENV_NAMED_ENTRIES_OFFSET,
            entries_local,
            function,
        );
        self.store_i64_const_at_offset(
            environment_local,
            ENV_NAMED_COUNT_OFFSET,
            bindings.len() as u64,
            function,
        );
        self.release_temp_local(cell_local);
        self.release_temp_local(entries_local);
        Ok(())
    }

    /// Finds only this declarative record's live binding. Caller traverses the
    /// parent chain and dispatches Object/Global Records through their own HasBinding.
    pub(crate) fn emit_find_own_named_binding(
        &mut self,
        environment_local: u32,
        key_local: u32,
        entry_local: u32,
        function: &mut Function,
    ) {
        let entries_local = self.reserve_temp_local();
        let count_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let candidate_local = self.reserve_temp_local();
        let name_local = self.reserve_temp_local();
        let present_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_NAMED_ENTRIES_OFFSET,
            entries_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            environment_local,
            ENV_NAMED_COUNT_OFFSET,
            count_local,
            function,
        );
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(entries_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(NAMED_BINDING_SIZE as i64));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(candidate_local));
        self.load_i64_to_local_from_offset(
            candidate_local,
            NAMED_BINDING_PRESENT_OFFSET,
            present_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(present_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            candidate_local,
            NAMED_BINDING_KEY_OFFSET,
            name_local,
            function,
        );
        self.emit_string_payload_equality_i32(name_local, key_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(candidate_local));
        function.instruction(&Instruction::LocalSet(entry_local));
        function.instruction(&Instruction::Br(3));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.release_temp_local(present_local);
        self.release_temp_local(name_local);
        self.release_temp_local(candidate_local);
        self.release_temp_local(index_local);
        self.release_temp_local(count_local);
        self.release_temp_local(entries_local);
    }
}
