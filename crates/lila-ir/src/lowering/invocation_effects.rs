use super::*;
use crate::source_call_flow_proof::SourceCallFlowEffects;

#[derive(Debug, PartialEq, Eq)]
enum InvocationCallerFlowState {
    ProvenNoFlowInvalidation,
    MayInvalidateCallerFlow,
}

#[derive(Debug, PartialEq, Eq)]
#[must_use = "invocation caller-flow effects must be consumed by flow invalidation"]
pub(super) struct InvocationCallerFlowEffects(InvocationCallerFlowState);

impl InvocationCallerFlowEffects {
    pub(super) const fn from_source_call(effects: SourceCallFlowEffects) -> Self {
        if effects.proves_no_flow_invalidation() {
            return Self::proven_no_flow_invalidation();
        }
        Self::may_invalidate()
    }

    pub(super) const fn from_host_builtin(builtin: HostBuiltinId) -> Self {
        if builtin.may_invalidate_caller_flow() {
            return Self::may_invalidate();
        }
        Self::proven_no_flow_invalidation()
    }

    pub(super) const fn may_invalidate() -> Self {
        Self(InvocationCallerFlowState::MayInvalidateCallerFlow)
    }

    pub(super) const fn combine(self, other: Self) -> Self {
        use InvocationCallerFlowState::{MayInvalidateCallerFlow, ProvenNoFlowInvalidation};

        match (self.0, other.0) {
            (ProvenNoFlowInvalidation, ProvenNoFlowInvalidation) => {
                Self::proven_no_flow_invalidation()
            }
            (ProvenNoFlowInvalidation, MayInvalidateCallerFlow)
            | (MayInvalidateCallerFlow, ProvenNoFlowInvalidation)
            | (MayInvalidateCallerFlow, MayInvalidateCallerFlow) => Self::may_invalidate(),
        }
    }

    pub(super) const fn may_invalidate_caller_flow(self) -> bool {
        matches!(self.0, InvocationCallerFlowState::MayInvalidateCallerFlow)
    }

    const fn proven_no_flow_invalidation() -> Self {
        Self(InvocationCallerFlowState::ProvenNoFlowInvalidation)
    }
}

#[must_use = "recorded invocation effects must be consumed by the emitted call"]
pub(super) struct AccountedInvocationEffects {
    attached_to_emitted_call: bool,
}

#[must_use = "builtin call analysis must be attached to its emitted call"]
pub(super) enum StandardBuiltinCallAnalysis {
    Ordinary(ValueInfo),
    Accounted {
        result: ValueInfo,
        effects: AccountedInvocationEffects,
    },
}

#[must_use = "analyzed invocation effects must be consumed by the emitted call"]
pub(super) enum AnalyzedInvocationEffects {
    AlreadyApplied,
    MustAttach(AccountedInvocationEffects),
}

impl StandardBuiltinCallAnalysis {
    pub(super) fn with_accounted_invocation_effects(result: ValueInfo) -> Self {
        Self::Accounted {
            result,
            effects: AccountedInvocationEffects::recorded(),
        }
    }

    pub(super) fn into_non_call_result(self) -> ValueInfo {
        match self {
            Self::Ordinary(result) => result,
            Self::Accounted { effects, .. } => effects.reject_non_call_expression(),
        }
    }

    pub(super) fn into_parts(self) -> (ValueInfo, AnalyzedInvocationEffects) {
        match self {
            Self::Ordinary(result) => (result, AnalyzedInvocationEffects::AlreadyApplied),
            Self::Accounted { result, effects } => {
                (result, AnalyzedInvocationEffects::MustAttach(effects))
            }
        }
    }
}

impl AnalyzedInvocationEffects {
    pub(super) fn already_applied() -> Self {
        Self::AlreadyApplied
    }

    pub(super) fn must_attach() -> Self {
        Self::MustAttach(AccountedInvocationEffects::recorded())
    }

    pub(super) fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::AlreadyApplied, effects) | (effects, Self::AlreadyApplied) => effects,
            (Self::MustAttach(existing), Self::MustAttach(additional)) => {
                Self::MustAttach(existing.combine(additional))
            }
        }
    }

    #[must_use = "the call carrying analyzed invocation effects must be retained"]
    pub(super) fn attach_to_emitted_call(self, call: TypedExpr) -> TypedExpr {
        match self {
            Self::AlreadyApplied => call,
            Self::MustAttach(effects) => effects.attach_to_emitted_call(call),
        }
    }
}

impl AccountedInvocationEffects {
    pub(super) fn recorded() -> Self {
        Self {
            attached_to_emitted_call: false,
        }
    }

    pub(super) fn combine(self, mut other: Self) -> Self {
        assert!(!self.attached_to_emitted_call);
        assert!(!other.attached_to_emitted_call);
        other.attached_to_emitted_call = true;
        self
    }

    #[must_use = "the call carrying accounted invocation effects must be retained"]
    pub(super) fn attach_to_emitted_call(mut self, call: TypedExpr) -> TypedExpr {
        let is_emitted_call = matches!(
            &call.expr,
            ExprIr::CallNamed { .. }
                | ExprIr::CallIndirect { .. }
                | ExprIr::CallMethod { .. }
                | ExprIr::Construct { .. }
                | ExprIr::JsonParseStaticReviver { .. }
                | ExprIr::OptionalPropertyChain { .. }
        );
        self.attached_to_emitted_call = true;
        assert!(
            is_emitted_call,
            "accounted invocation effects must be attached to emitted call IR"
        );
        call
    }

    fn reject_non_call_expression(mut self) -> ! {
        self.attached_to_emitted_call = true;
        panic!("effect-accounting proof cannot be discarded by a non-call expression")
    }
}

impl Drop for AccountedInvocationEffects {
    fn drop(&mut self) {
        if !self.attached_to_emitted_call && !std::thread::panicking() {
            panic!("recorded invocation effects were not attached to an emitted call")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_invocations_preserve_caller_flow_only_when_every_target_preserves_it() {
        let preserving_pair =
            InvocationCallerFlowEffects::from_host_builtin(HostBuiltinId::CreateRealm).combine(
                InvocationCallerFlowEffects::from_host_builtin(HostBuiltinId::CreateRealm),
            );
        let mixed_pair = InvocationCallerFlowEffects::from_host_builtin(HostBuiltinId::CreateRealm)
            .combine(InvocationCallerFlowEffects::from_host_builtin(
                HostBuiltinId::DetachArrayBuffer,
            ));

        assert!(!preserving_pair.may_invalidate_caller_flow());
        assert!(mixed_pair.may_invalidate_caller_flow());
    }
}
