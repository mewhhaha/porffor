//! Closed module-state reads consume only rooted GC ModuleRecord owners.
use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_load_module_state_strict(
        &self,
        record: &GcLocal<ModuleRecord>,
        out: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::STATE)
            .read(record, schema, function)
            .store(out, function);
        function.instruction(&Instruction::I32Const(0));
        for state in ModuleEvaluationState::ALL {
            out.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(state)));
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_load_module_completion_strict(
        &self,
        record: &GcLocal<ModuleRecord>,
        out: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::COMPLETION)
            .read(record, schema, function)
            .store(out, function);
        function.instruction(&Instruction::I32Const(0));
        for state in ModuleEvaluationCompletion::ALL {
            out.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(state)));
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_load_module_body_state_strict(
        &self,
        record: &GcLocal<ModuleRecord>,
        out: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::BODY_STATE)
            .read(record, schema, function)
            .store(out, function);
        function.instruction(&Instruction::I32Const(0));
        for state in ModuleBodyState::ALL {
            out.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(state)));
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }

    pub(super) fn emit_load_module_activation_kind_strict(
        &self,
        record: &GcLocal<ModuleRecord>,
        out: I32Local,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        schema
            .struct_type::<ModuleRecord>()
            .field(ModuleRecordSchema::ACTIVATION_KIND)
            .read(record, schema, function)
            .store(out, function);
        function.instruction(&Instruction::I32Const(0));
        for state in ModuleActivationKind::ALL {
            out.load(function);
            function.instruction(&Instruction::I32Const(GcI32Constant::encode(state)));
            function.instruction(&Instruction::I32Eq);
            function.instruction(&Instruction::I32Or);
        }
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
    }
}
