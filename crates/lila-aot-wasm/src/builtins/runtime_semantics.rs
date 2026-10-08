use crate::{emit::FunctionBuilder, *};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_reject_runtime_semantics(
        &self,
        rejection: lila_ir::RuntimeSemanticRejection,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(rejection.abi_code()));
        function.instruction(&Instruction::Call(
            self.functions
                .reject_runtime_semantics_import_function_index(),
        ));
        // The host returns a typed semantic or runtime-capability rejection outside JavaScript
        // completion handling. Returning normally violates this closed ABI.
        function.instruction(&Instruction::Unreachable);
    }
}
