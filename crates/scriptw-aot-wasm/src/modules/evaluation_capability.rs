//! A host source representation does not invent a host module's Evaluate method.

use super::*;

impl FunctionBuilder<'_> {
    pub(super) fn emit_module_evaluation_capability_rejection(
        &mut self,
        module: u32,
        output: &ValueLocals,
        function: &mut Function,
    ) -> bool {
        let capability = self
            .functions
            .module_graph()
            .and_then(|graph| graph.unavailable_evaluation_capability(module));
        let Some(capability) = capability else {
            return false;
        };
        // Emission is unconditional only within this actual demand's body.
        // An unchosen import branch never executes the typed host rejection.
        self.emit_reject_runtime_semantics(
            scriptw_ir::RuntimeSemanticRejection::UnavailableCapability(capability),
            function,
        );
        output.set_undefined(function);
        true
    }
}
