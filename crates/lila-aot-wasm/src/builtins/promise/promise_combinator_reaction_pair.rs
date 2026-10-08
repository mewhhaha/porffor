use super::*;

/// Borrows completed callable values; no method is reconstructed or reread.
struct PromiseCombinatorReactionPairLocals<'a> {
    on_fulfilled: &'a ValueLocals,
    on_rejected: &'a ValueLocals,
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_invoke_promise_combinator_reaction_pair(
        &mut self,
        mode: PromiseCombinatorMode,
        then: &ValueLocals,
        promise: &ValueLocals,
        resolve_element: &ValueLocals,
        reject: &ValueLocals,
        reject_element: &ValueLocals,
        resolve: &ValueLocals,
        result: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let pair = match mode {
            PromiseCombinatorMode::Values => PromiseCombinatorReactionPairLocals {
                on_fulfilled: resolve_element,
                on_rejected: reject,
            },
            PromiseCombinatorMode::SettledRecords => PromiseCombinatorReactionPairLocals {
                on_fulfilled: resolve_element,
                on_rejected: reject_element,
            },
            PromiseCombinatorMode::FirstFulfillment => PromiseCombinatorReactionPairLocals {
                on_fulfilled: resolve,
                on_rejected: reject_element,
            },
            PromiseCombinatorMode::Race => PromiseCombinatorReactionPairLocals {
                on_fulfilled: resolve,
                on_rejected: reject,
            },
        };
        let arguments =
            self.emit_pre_evaluated_arg_vector(&[pair.on_fulfilled, pair.on_rejected], function);
        let emitted =
            self.emit_function_or_proxy_call_with_argv(then, promise, &arguments, result, function);
        arguments.clear(function);
        emitted
    }
}
