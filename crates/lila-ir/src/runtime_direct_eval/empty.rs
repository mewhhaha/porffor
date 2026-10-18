//! A source-independent completion is available only after actual parsing and lowering.
use super::*;
use crate::{PreparedScriptOutcome, RuntimeGlobalDeclarationPlan, ValueKind};
mod metadata;

/// Borrowed proof that this exact successfully compiled direct eval executes
/// no statements or instantiation effects and returns normal undefined.
/// Neither a source-text heuristic nor a result-kind guess can mint it.
#[derive(Debug)]
pub struct EmptyRuntimeDirectEval<'a> {
    _program: &'a RuntimeDirectEvalProgram,
}

impl RuntimeDirectEvalProgram {
    pub fn empty_completion(&self) -> Option<EmptyRuntimeDirectEval<'_>> {
        if !self.empty_syntax || !self.program.diagnostics.is_empty() {
            return None;
        }
        let script = self.program.script.as_ref()?;
        let [root] = script.prepared_scripts.as_slice() else {
            return None;
        };
        let PreparedScriptOutcome::Executable(unit) = &root.outcome else {
            return None;
        };
        if unit.id != self.script_id
            || unit.kind != PreparedScriptKind::DirectEval(self.caller.context().clone())
            || !unit.body.statements.is_empty()
            || unit.body.result_kind != ValueKind::Undefined
            || unit.body.lexical_environment.is_some()
            || !metadata::is_inert(unit)
            || unit.declarations != RuntimeGlobalDeclarationPlan::default()
            || !unit.function_ids.is_empty()
            || !script.functions.is_empty()
            || !script.prepared_dynamic_functions.is_empty()
            || script.module_prelude.is_some()
            || script.script_entry_after_prelude.is_some()
            || !script.body.statements.is_empty()
            || script.body.result_kind != ValueKind::Undefined
            || script.body.lexical_environment.is_some()
            || !script.owned_env_bindings.is_empty()
            || !script.global_bindings.is_empty()
            || script.global_bindings.lexical_names().next().is_some()
            || script.runtime_declarations != RuntimeGlobalDeclarationPlan::default()
            || !script.host_builtins.is_empty()
            || script.eval_environment.is_some()
            || self.program.modules.is_some()
        {
            return None;
        }
        // The unit's eval-environment role and source identity are metadata.
        // With no statements, bindings, functions or template sites, neither
        // its environment allocation nor its P/source owner can be observed.
        Some(EmptyRuntimeDirectEval { _program: self })
    }
}
