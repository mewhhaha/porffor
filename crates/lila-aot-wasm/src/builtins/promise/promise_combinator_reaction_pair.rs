use super::*;

#[must_use = "a Promise combinator reaction pair must be consumed by its then invocation"]
struct PromiseCombinatorReactionPairLocals {
    on_fulfilled: TaggedLocals,
    on_rejected: TaggedLocals,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_invoke_promise_combinator_reaction_pair(
        &mut self,
        mode: PromiseCombinatorMode,
        then_payload_local: u32,
        then_tag_local: u32,
        next_promise_payload_local: u32,
        next_promise_tag_local: u32,
        resolve_element_payload_local: u32,
        resolve_element_tag_local: u32,
        reject_payload_local: u32,
        reject_tag_local: u32,
        reject_element_payload_local: u32,
        reject_element_tag_local: u32,
        resolve_payload_local: u32,
        resolve_tag_local: u32,
        call_payload_local: u32,
        call_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let reaction_pair = match mode {
            PromiseCombinatorMode::Values => PromiseCombinatorReactionPairLocals {
                on_fulfilled: TaggedLocals::new(
                    resolve_element_payload_local,
                    resolve_element_tag_local,
                ),
                on_rejected: TaggedLocals::new(reject_payload_local, reject_tag_local),
            },
            PromiseCombinatorMode::SettledRecords => PromiseCombinatorReactionPairLocals {
                on_fulfilled: TaggedLocals::new(
                    resolve_element_payload_local,
                    resolve_element_tag_local,
                ),
                on_rejected: TaggedLocals::new(
                    reject_element_payload_local,
                    reject_element_tag_local,
                ),
            },
            PromiseCombinatorMode::FirstFulfillment => PromiseCombinatorReactionPairLocals {
                on_fulfilled: TaggedLocals::new(resolve_payload_local, resolve_tag_local),
                on_rejected: TaggedLocals::new(
                    reject_element_payload_local,
                    reject_element_tag_local,
                ),
            },
        };
        let PromiseCombinatorReactionPairLocals {
            on_fulfilled,
            on_rejected,
        } = reaction_pair;
        self.emit_function_or_proxy_call_leave_throw_completion(
            then_payload_local,
            then_tag_local,
            next_promise_payload_local,
            next_promise_tag_local,
            &[
                (on_fulfilled.payload, on_fulfilled.tag),
                (on_rejected.payload, on_rejected.tag),
            ],
            call_payload_local,
            call_tag_local,
            function,
        )?;
        Ok(())
    }
}
