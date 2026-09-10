use crate::{PrivateNameId, Strictness};
use lila_front::{DirectEvalParseContext, EvalInvocationContext};
use std::collections::BTreeMap;

/// Equal keys mean one emitted body can use either caller's runtime records.
/// Source bindings are always resolved at runtime, never specialized to slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectEvalContextIr {
    derived_constructor_owner: Option<crate::FunctionId>,
    strictness: Strictness,
    invocation: EvalInvocationContext,
    private_names: BTreeMap<String, PrivateNameId>,
}

impl DirectEvalContextIr {
    pub(crate) fn new(
        strictness: Strictness,
        invocation: EvalInvocationContext,
        private_names: BTreeMap<String, PrivateNameId>,
        derived_constructor_owner: Option<crate::FunctionId>,
    ) -> Self {
        Self {
            derived_constructor_owner,
            strictness,
            invocation,
            private_names,
        }
    }
    pub fn derived_constructor_owner(&self) -> Option<&crate::FunctionId> {
        self.derived_constructor_owner.as_ref()
    }
    pub fn parse_context(&self) -> DirectEvalParseContext {
        DirectEvalParseContext {
            strict_caller: self.strictness == Strictness::Strict,
            invocation: self.invocation,
            private_names: self.private_names.keys().cloned().collect(),
        }
    }
    pub fn private_names(&self) -> &BTreeMap<String, PrivateNameId> {
        &self.private_names
    }
    pub const fn strictness(&self) -> Strictness {
        self.strictness
    }
    pub const fn invocation(&self) -> EvalInvocationContext {
        self.invocation
    }
}
