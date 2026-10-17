//! A native global Script P bound to the caller's exact immutable runtime R.
use crate::{EmitError, RuntimeArtifact, ValidatedRuntimeArtifact};
use lila_intl::IntlCompilationProfile;
#[cfg(test)]
use lila_intl::IntlDataSelection;
use lila_ir::{PreparedScriptOutcome, RuntimeGlobalScriptProgram};
use std::sync::Arc;

pub(crate) const RUNTIME_GLOBAL_SCRIPT_FACTORY_EXPORT: &str = "lila.runtime_global_script_factory";

/// Only successful emission against this exact R can mint the artifact. The
/// host must invoke init/factory in the same Store, never this P's main export.
#[derive(Debug)]
pub struct RuntimeGlobalScriptArtifact {
    bytes: Arc<[u8]>,
    runtime: Arc<RuntimeArtifact>,
}

impl RuntimeGlobalScriptArtifact {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn runtime(&self) -> &Arc<RuntimeArtifact> {
        &self.runtime
    }
    pub fn program_initialize_export(&self) -> &'static str {
        crate::runtime_function::PROGRAM_INITIALIZE_EXPORT
    }
    pub fn factory_export(&self) -> &'static str {
        RUNTIME_GLOBAL_SCRIPT_FACTORY_EXPORT
    }
}

pub fn emit_runtime_global_script(
    program: &RuntimeGlobalScriptProgram,
    runtime: &Arc<RuntimeArtifact>,
    intl_profile: &IntlCompilationProfile,
) -> Result<RuntimeGlobalScriptArtifact, EmitError> {
    if let Some(diagnostic) = program.program().wasm_blocking_diagnostic() {
        return Err(EmitError::unsupported(diagnostic.message.clone()));
    }
    let validated = ValidatedRuntimeArtifact::new(Arc::clone(runtime), intl_profile.clone())?;
    emit_validated(program, &validated)
}

/// Emit P against an immutable R whose current layout and selected Intl image
/// were admitted once. The proof retains the exact R Arc; Wasm linking still
/// checks the actual graph when the native host instantiates this P.
pub fn emit_runtime_global_script_with_validated_runtime(
    program: &RuntimeGlobalScriptProgram,
    validated: &ValidatedRuntimeArtifact,
) -> Result<RuntimeGlobalScriptArtifact, EmitError> {
    if let Some(diagnostic) = program.program().wasm_blocking_diagnostic() {
        return Err(EmitError::unsupported(diagnostic.message.clone()));
    }
    emit_validated(program, validated)
}

fn emit_validated(
    program: &RuntimeGlobalScriptProgram,
    validated: &ValidatedRuntimeArtifact,
) -> Result<RuntimeGlobalScriptArtifact, EmitError> {
    let runtime = validated.runtime();
    let emitted = crate::emit::emit_runtime_global_script_module(program, validated)?;
    if emitted.runtime_layout.is_some()
        || !emitted
            .artifact
            .runtime()
            .is_some_and(|actual| Arc::ptr_eq(actual, runtime))
    {
        return Err(EmitError::unsupported(
            "native global Script did not retain its exact linked runtime",
        ));
    }
    Ok(RuntimeGlobalScriptArtifact {
        bytes: Arc::from(emitted.artifact.bytes),
        runtime: Arc::clone(runtime),
    })
}

/// The first source row must name the witness's exact role/source/root ID and
/// actual PreparedScript entry. Ordered dispatch then enters that root before
/// any nested row. The host compares the decoded request to this same witness
/// before invoking the factory and accepts only Entered, including JS Throw.
pub(crate) fn require_factory_plan(
    program: &RuntimeGlobalScriptProgram,
    functions: &crate::planning::FunctionMetaRegistry,
) -> Result<(), EmitError> {
    let first = functions
        .prepared_scripts()
        .first()
        .ok_or_else(|| EmitError::unsupported("native global Script has no factory source row"))?;
    let unit = require_root_source_row(program, first)?;
    functions
        .get(&unit.id.function_id())
        .ok_or_else(|| {
            EmitError::unsupported(
                "compiler invariant: native global Script root has no planned entry",
            )
        })?
        .entry
        .prepared_script()?;
    Ok(())
}

fn require_root_source_row<'a>(
    program: &RuntimeGlobalScriptProgram,
    first: &'a lila_ir::PreparedScript,
) -> Result<&'a lila_ir::PreparedScriptUnit, EmitError> {
    let expected = program
        .program()
        .script
        .as_ref()
        .and_then(|script| script.prepared_scripts.first())
        .ok_or_else(|| {
            EmitError::unsupported(
                "compiler invariant: native global Script witness has no root source row",
            )
        })?;
    let expected_kind = match program.kind() {
        lila_ir::RuntimeGlobalScriptKind::IndirectEval => lila_ir::PreparedScriptKind::IndirectEval,
        lila_ir::RuntimeGlobalScriptKind::RealmScript => lila_ir::PreparedScriptKind::RealmScript,
    };
    if first.kind != expected_kind || first != expected {
        return Err(EmitError::unsupported(
            "compiler invariant: native global Script factory differs from its source witness",
        ));
    }
    let PreparedScriptOutcome::Executable(unit) = &first.outcome else {
        return Err(EmitError::unsupported(
            "compiler invariant: native global Script factory root is not executable",
        ));
    };
    if unit.id != program.script_id() || unit.kind != expected_kind {
        return Err(EmitError::unsupported(
            "compiler invariant: native global Script factory does not own its actual root ID",
        ));
    }
    Ok(unit)
}

#[cfg(test)]
mod tests;
