//! Exact typed constructor hook projection; no eqref function transport.
use super::*;
use crate::runtime_helpers::{RuntimeHelperFunctionBase, RuntimeHelperId};

impl RuntimeSchema {
    pub(crate) fn instance_elements_initializer_reference(
        &self,
        base: RuntimeHelperFunctionBase,
        present: bool,
        function: &mut Function,
    ) -> FunctionStackReference<InitializeInstanceElementsHook, Nullable> {
        if present {
            function.instruction(&Instruction::RefFunc(
                RuntimeHelperId::InitializeInstanceElements.index(base),
            ));
        } else {
            function.instruction(&Instruction::RefNull(HeapType::Concrete(
                InitializeInstanceElementsHook::SIGNATURE.type_index(),
            )));
        }
        FunctionStackReference { shape: PhantomData }
    }

    pub(crate) fn call_instance_elements_initializer(
        &self,
        callee: &FunctionLocal<InitializeInstanceElementsHook>,
        defining_program: &ProgramInstanceLocals,
        constructor_context: &GcLocal<FunctionContext>,
        receiver: &ValueLocals,
        function: &mut Function,
    ) -> GcCallResult {
        // ABI order is defining P!, actual constructor context!, receiver Value3.
        defining_program.reference(self, function);
        constructor_context.load(self, function);
        receiver.emit(function);
        callee.load(function);
        function.instruction(&Instruction::CallRef(
            InitializeInstanceElementsHook::SIGNATURE.type_index(),
        ));
        GcCallResult::emitted()
    }
}
