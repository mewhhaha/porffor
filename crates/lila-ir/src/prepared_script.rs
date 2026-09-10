//! Independently compiled Script records for syntax-proven source text.

use crate::{BlockIr, FunctionId, GlobalBindingPlan, OwnedEnvBindingIr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedScriptKind {
    DirectEval(crate::DirectEvalContextIr),
    RealmScript,
    IndirectEval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StaticScriptId(pub(crate) u32);

impl StaticScriptId {
    pub fn function_id(self) -> FunctionId {
        format!("$static.script.{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedScriptAdmission {
    ResolvedIntrinsic,
    RuntimeCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedScript {
    pub admission: PreparedScriptAdmission,
    pub kind: PreparedScriptKind,
    pub source: String,
    pub outcome: PreparedScriptOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedScriptOutcome {
    Executable(PreparedScriptUnit),
    DeferredSyntaxError { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedScriptUnit {
    pub eval_environment: Option<crate::EvalEnvironmentRoleIr>,
    pub id: StaticScriptId,
    pub kind: PreparedScriptKind,
    pub strict: bool,
    pub body: BlockIr,
    pub owned_env_bindings: Vec<OwnedEnvBindingIr>,
    pub global_bindings: GlobalBindingPlan,
    pub declarations: RuntimeGlobalDeclarationPlan,
    pub function_ids: Vec<FunctionId>,
}

impl PreparedScriptUnit {
    pub const fn has_global_variable_environment(&self) -> bool {
        match &self.kind {
            PreparedScriptKind::RealmScript => true,
            PreparedScriptKind::IndirectEval => !self.strict,
            PreparedScriptKind::DirectEval(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalFunctionDeclarationIr {
    pub name: String,
    pub function_id: FunctionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeGlobalDeclarationPlan {
    pub lexical_names_in_source_order: Vec<String>,
    /// Last declaration of each name, in reverse source order. Validation uses
    /// this order; installation reverses it, as GlobalDeclarationInstantiation
    /// requires.
    pub functions_in_reverse_order: Vec<GlobalFunctionDeclarationIr>,
    /// Required var declarations, in source order and without duplicate names.
    /// Function names are retained here only when a var also declares the name.
    pub var_names: Vec<String>,
    /// Annex B block functions are optional var bindings. A runtime collision
    /// suppresses their variable-environment copy instead of rejecting Script.
    pub annex_b_candidates: Vec<AnnexBGlobalDeclarationIr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnexBGlobalDeclarationIr {
    pub name: String,
    pub admission: OwnedEnvBindingIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DynamicScriptSource {
    pub(crate) admission: PreparedScriptAdmission,
    pub(crate) kind: PreparedScriptKind,
    pub(crate) source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum ScriptInstantiation {
    #[default]
    FreshEntry,
    Prepared(PreparedScriptKind),
}

impl ScriptInstantiation {
    pub(crate) const fn has_global_variable_environment(&self, strict: bool) -> bool {
        match self {
            Self::FreshEntry | Self::Prepared(PreparedScriptKind::RealmScript) => true,
            Self::Prepared(PreparedScriptKind::IndirectEval) => !strict,
            Self::Prepared(PreparedScriptKind::DirectEval(_)) => false,
        }
    }
}
