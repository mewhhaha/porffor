//! Separate GC-result bootstrap: no Script, Realm, activation, or Completion.
use super::*;
use crate::function_layout::ProgramInitializeIndex;
use crate::gc_types::{
    ProgramInstance, ProgramInstanceHooks, ProgramInstanceImage, ProgramInstanceLocals,
};

pub(crate) struct ProgramInitializeCompilation<'a, 'pool> {
    image: &'a ProgramInstanceImage<'pool>,
    strings: &'pool StringPool,
    functions: &'a FunctionMetaRegistry,
    runtime_bootstrap_plan: RuntimeBootstrapPlan,
    helper_base: crate::runtime_helpers::RuntimeHelperFunctionBase,
    index: ProgramInitializeIndex,
}

/// Only this bootstrap owner can complete a non-null ProgramInstance result.
/// No constructor accepts an untyped Function or an ordinary Main Completion.
pub(crate) struct CompletedProgramInitializeBody {
    index: ProgramInitializeIndex,
    body: Function,
}
impl CompletedProgramInitializeBody {
    pub(crate) const fn index(&self) -> ProgramInitializeIndex {
        self.index
    }
    pub(crate) fn into_parts(self) -> (ProgramInitializeIndex, Function) {
        (self.index, self.body)
    }
}

impl<'a, 'pool> ProgramInitializeCompilation<'a, 'pool> {
    pub(crate) fn new(
        image: &'a ProgramInstanceImage<'pool>,
        strings: &'pool StringPool,
        functions: &'a FunctionMetaRegistry,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        helper_base: crate::runtime_helpers::RuntimeHelperFunctionBase,
        index: ProgramInitializeIndex,
    ) -> Result<Self, EmitError> {
        image.require_pool(strings)?;
        Ok(Self {
            image,
            strings,
            functions,
            runtime_bootstrap_plan,
            helper_base,
            index,
        })
    }

    pub(crate) fn compile(
        self,
        globals: &FinalizedModuleGlobals,
    ) -> Result<CompletedProgramInitializeBody, EmitError> {
        let schema = globals.schema();
        let mut builder = FunctionBuilder::new_program_initialize(
            schema,
            self.strings,
            self.functions,
            self.runtime_bootstrap_plan,
            Some(self.helper_base),
        );
        let mut function = builder.begin_program_initialize_body();
        builder.initialize_runtime_literal_roots(&mut function)?;
        let (literals, metadata) =
            builder.produce_program_literal_roots(self.image, &mut function)?;
        let hooks = ProgramInstanceHooks::from_program(
            schema,
            self.image,
            self.functions,
            self.helper_base,
            &mut function,
        );
        let program = ProgramInstanceLocals::publish(
            schema,
            self.image,
            hooks,
            literals,
            metadata,
            &mut function,
        )?;
        let result: GcStackReference<ProgramInstance> = program.reference(schema, &mut function);
        // The non-null result remains a Wasm operand-stack root while the
        // temporary local is retired. No allocation occurs before return.
        program.clear(&mut function);
        let _ = result;
        function.instruction(&Instruction::End);
        Ok(CompletedProgramInitializeBody {
            index: self.index,
            body: builder.finish_function(function),
        })
    }
}
