use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use = "Promise invocation policy must govern executor observation and caller-flow invalidation"]
pub(super) enum PromiseInvocationPolicy {
    UseBuiltinCatalog,
    ProvenNoSynchronousUserCode,
    ConstructorMayInvokeExecutor,
}

impl PromiseInvocationPolicy {
    pub(super) fn for_call(
        builtin: StandardBuiltinId,
        args: &[TypedExpr],
        context: &BuiltinCallContext,
    ) -> Self {
        match builtin {
            StandardBuiltinId::PromiseConstructor
                if context == &BuiltinCallContext::Construct
                    && args.first().is_some_and(|executor| {
                        !executor
                            .possible_kinds
                            .is_subset_of(KindSet::PRIMITIVE_ONLY)
                    }) =>
            {
                Self::ConstructorMayInvokeExecutor
            }
            StandardBuiltinId::PromiseConstructor => Self::ProvenNoSynchronousUserCode,
            StandardBuiltinId::PromiseResolveFunction
                if args.first().is_none_or(|resolution| {
                    resolution
                        .possible_kinds
                        .is_subset_of(KindSet::PRIMITIVE_ONLY)
                }) =>
            {
                Self::ProvenNoSynchronousUserCode
            }
            _ => Self::UseBuiltinCatalog,
        }
    }

    pub(super) const fn constructor_may_invoke_executor(self) -> bool {
        matches!(self, Self::ConstructorMayInvokeExecutor)
    }

    pub(super) const fn bypasses_catalog_invalidation(self) -> bool {
        matches!(self, Self::ProvenNoSynchronousUserCode)
    }
}
