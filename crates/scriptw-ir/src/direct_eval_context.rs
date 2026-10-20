use crate::{PrivateNameId, Strictness};
use scriptw_front::{DirectEvalParseContext, EvalInvocationContext};
use std::collections::BTreeMap;

/// A direct eval's source belongs to this compilation's actual Script entry.
/// Discovery has no graph ID yet; only the loaded graph can bind the record.
/// Module, indirect-eval and independent-function origins do not mint this proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptImportOrigin {
    Discovery,
    LoadedRecord(crate::ModuleUnitId),
    ExternalCaller(crate::RuntimeDirectEvalImportOrigin),
}

impl ScriptImportOrigin {
    pub(crate) const fn referrer(self) -> Option<crate::ModuleUnitId> {
        match self {
            Self::Discovery => None,
            Self::LoadedRecord(record) => Some(record),
            Self::ExternalCaller(_) => None,
        }
    }
}

/// A constructor in this compilation, or the actual retained caller activation.
/// A foreign ProgramInstance's FunctionId is never a local planner lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectEvalDerivedOwner {
    LocalFunction(crate::FunctionId),
    ExternalCaller,
}

/// Equal keys mean one emitted body can use either caller's runtime records.
/// Source bindings are always resolved at runtime, never specialized to slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectEvalContextIr {
    derived_constructor_owner: Option<DirectEvalDerivedOwner>,
    script_import_origin: Option<ScriptImportOrigin>,
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
            derived_constructor_owner: derived_constructor_owner
                .map(DirectEvalDerivedOwner::LocalFunction),
            script_import_origin: None,
            strictness,
            invocation,
            private_names,
        }
    }
    pub(crate) fn with_script_import_origin(mut self, origin: Option<ScriptImportOrigin>) -> Self {
        self.script_import_origin = origin;
        self
    }
    pub(crate) const fn script_import_origin(&self) -> Option<ScriptImportOrigin> {
        self.script_import_origin
    }
    pub fn derived_constructor_owner(&self) -> Option<&crate::FunctionId> {
        match self.derived_owner() {
            Some(DirectEvalDerivedOwner::LocalFunction(owner)) => Some(owner),
            Some(DirectEvalDerivedOwner::ExternalCaller) | None => None,
        }
    }
    pub fn derived_owner(&self) -> Option<&DirectEvalDerivedOwner> {
        self.derived_constructor_owner.as_ref()
    }
    pub(crate) fn with_derived_owner(mut self, owner: Option<DirectEvalDerivedOwner>) -> Self {
        self.derived_constructor_owner = owner;
        self
    }
    /// Project only immutable compiler facts. Runtime environments and their
    /// actual private-name identities must be validated by the native host.
    pub fn runtime_descriptor(&self) -> crate::RuntimeDirectEvalCallerDescriptor {
        crate::runtime_direct_eval::caller::descriptor(self)
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
