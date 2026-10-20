//! Typed consumption of the separately emitted P bootstrap result.
use super::*;
use crate::function_layout::ProgramInitializeIndex;

impl RuntimeSchema {
    pub(crate) fn call_program_initialize(
        &self,
        index: ProgramInitializeIndex,
        function: &mut Function,
    ) -> ProgramInstanceLocals {
        function.instruction(&Instruction::Call(index.raw()));
        // ProgramInitializeIndex can only name the layout's dedicated body;
        // its completed-body token owns the registered non-null P result ABI.
        ProgramInstanceLocals::bind(self, GcStackReference::new(), function)
    }
}
