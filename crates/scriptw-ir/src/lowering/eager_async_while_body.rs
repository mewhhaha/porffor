use super::*;

/// Only source proven not to suspend may omit async clause states. Consuming
/// this carrier owns that temporary omission and retains the analyzed protocol.
pub(super) struct EagerAsyncWhileBody<'ast> {
    source: &'ast Statement,
}

impl<'ast> EagerAsyncWhileBody<'ast> {
    pub(super) fn new(source: &'ast Statement) -> Option<Self> {
        (!super::synchronous_resource_loop::source_statement_suspends(source)
            && !crate::async_with_source::contains_plain_async_phase(source))
        .then_some(Self { source })
    }

    pub(super) fn lower(self, lowerer: &mut ScriptLowerer<'_>) -> (StatementIr, ValueKind) {
        let continuation = lowerer.current_async_resume_state.take();
        let result = lowerer.lower_loop_body(self.source);
        assert!(
            lowerer.current_async_resume_state.is_none(),
            "an eager async while body must not allocate a continuation"
        );
        lowerer.current_async_resume_state = continuation;
        result
    }
}
