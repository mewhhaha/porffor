//! A fresh native Function program linked to the caller's exact immutable R.
use std::sync::Arc;

use lila_intl::IntlCompilationProfile;
#[cfg(test)]
use lila_intl::IntlDataSelection;
use lila_ir::RuntimeFunctionProgram;

use crate::{EmitError, RuntimeArtifact, ValidatedRuntimeArtifact};
pub(crate) mod runtime_profile;
#[cfg(test)]
mod tests;

pub(crate) const PROGRAM_INITIALIZE_EXPORT: &str = "lila.program_init";
pub(crate) const RUNTIME_FUNCTION_FACTORY_EXPORT: &str = "lila.runtime_function_factory";

/// P bytes and auxiliary exports admitted together against this exact R Arc.
/// The native host instantiates only these P bytes in its existing Store.
#[derive(Debug)]
pub struct RuntimeFunctionArtifact {
    bytes: Arc<[u8]>,
    runtime: Arc<RuntimeArtifact>,
}
impl RuntimeFunctionArtifact {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn runtime(&self) -> &Arc<RuntimeArtifact> {
        &self.runtime
    }
    pub fn program_initialize_export(&self) -> &'static str {
        PROGRAM_INITIALIZE_EXPORT
    }
    pub fn factory_export(&self) -> &'static str {
        RUNTIME_FUNCTION_FACTORY_EXPORT
    }
}

pub fn emit_runtime_function(
    program: &RuntimeFunctionProgram,
    runtime: &Arc<RuntimeArtifact>,
    intl_profile: &IntlCompilationProfile,
) -> Result<RuntimeFunctionArtifact, EmitError> {
    if let Some(diagnostic) = program.program().wasm_blocking_diagnostic() {
        return Err(EmitError::unsupported(diagnostic.message.clone()));
    }
    let validated = ValidatedRuntimeArtifact::new(Arc::clone(runtime), intl_profile.clone())?;
    emit_validated(program, &validated)
}

/// Emit P against an immutable R whose current layout and selected Intl image
/// were admitted once. The proof retains the exact R Arc; Wasm linking still
/// checks the actual graph when the native host instantiates this P.
pub fn emit_runtime_function_with_validated_runtime(
    program: &RuntimeFunctionProgram,
    validated: &ValidatedRuntimeArtifact,
) -> Result<RuntimeFunctionArtifact, EmitError> {
    if let Some(diagnostic) = program.program().wasm_blocking_diagnostic() {
        return Err(EmitError::unsupported(diagnostic.message.clone()));
    }
    emit_validated(program, validated)
}

fn emit_validated(
    program: &RuntimeFunctionProgram,
    validated: &ValidatedRuntimeArtifact,
) -> Result<RuntimeFunctionArtifact, EmitError> {
    let runtime = validated.runtime();
    let emitted = crate::emit::emit_runtime_function_module(program, validated)?;
    if emitted.runtime_layout.is_some()
        || !emitted
            .artifact
            .runtime()
            .is_some_and(|actual| Arc::ptr_eq(actual, runtime))
    {
        return Err(EmitError::unsupported(
            "native Function emission did not retain its exact linked runtime",
        ));
    }
    Ok(RuntimeFunctionArtifact {
        bytes: Arc::from(emitted.artifact.bytes),
        runtime: Arc::clone(runtime),
    })
}

/// The exported helper must contain this witness's compiled first request,
/// including its actual kind and root FunctionId, after registry construction.
pub(crate) fn require_factory_plan(
    program: &RuntimeFunctionProgram,
    functions: &crate::planning::FunctionMetaRegistry,
) -> Result<(), EmitError> {
    let first = functions
        .prepared_dynamic_functions()
        .first()
        .ok_or_else(|| {
            EmitError::unsupported("native Function program has no compiled factory request")
        })?;
    if first.kind != program.kind()
        || !matches!(&first.outcome, lila_ir::PreparedDynamicFunctionOutcome::Compiled { function_id }
            if function_id == program.function_id())
        || functions.get(program.function_id()).is_none()
    {
        return Err(EmitError::unsupported(
            "native Function factory does not own the compiled request",
        ));
    }
    Ok(())
}
