use crate::{PrivateNameId, Strictness};
use lila_front::{DirectEvalParseContext, EvalInvocationContext};
use std::collections::BTreeMap;

/// A direct eval's source belongs to this compilation's actual Script entry.
/// Discovery has no graph ID yet; only the loaded graph can bind the record.
/// Module, indirect-eval and independent-function origins do not mint this proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptImportOrigin {
    Discovery,
    LoadedRecord(crate::ModuleUnitId),
}

impl ScriptImportOrigin {
    pub(crate) const fn referrer(self) -> Option<crate::ModuleUnitId> {
        match self {
            Self::Discovery => None,
            Self::LoadedRecord(record) => Some(record),
        }
    }
}

/// Equal keys mean one emitted body can use either caller's runtime records.
/// Source bindings are always resolved at runtime, never specialized to slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectEvalContextIr {
    derived_constructor_owner: Option<crate::FunctionId>,
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
            derived_constructor_owner,
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
