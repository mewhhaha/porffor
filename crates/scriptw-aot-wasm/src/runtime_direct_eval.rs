//! A fresh direct-eval P, bound to the exact caller witness and immutable R.
use crate::{EmitError, RuntimeArtifact, ValidatedRuntimeArtifact};
use scriptw_intl::IntlCompilationProfile;
#[cfg(test)]
use scriptw_intl::IntlDataSelection;
use scriptw_ir::{RuntimeDirectEvalCaller, RuntimeDirectEvalProgram};
use std::sync::Arc;

#[cfg(test)]
mod tests;
mod witness;
pub(crate) use witness::require_factory_plan;

pub(crate) const RUNTIME_DIRECT_EVAL_FACTORY_EXPORT: &str = "scriptw.runtime_direct_eval_factory";

/// Native hosts retain this complete witness when caching or entering P2.
/// Descriptor-visible bindings do not replace the caller's hidden-scope floor.
#[derive(Debug)]
pub struct RuntimeDirectEvalArtifact {
    bytes: Arc<[u8]>,
    runtime: Arc<RuntimeArtifact>,
    caller: RuntimeDirectEvalCaller,
    source: Arc<[u16]>,
}
impl RuntimeDirectEvalArtifact {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn runtime(&self) -> &Arc<RuntimeArtifact> {
        &self.runtime
    }
    pub fn caller(&self) -> &RuntimeDirectEvalCaller {
        &self.caller
    }
    pub fn source(&self) -> &[u16] {
        &self.source
    }
    pub fn program_initialize_export(&self) -> &'static str {
        crate::runtime_function::PROGRAM_INITIALIZE_EXPORT
    }
    pub fn factory_export(&self) -> &'static str {
        RUNTIME_DIRECT_EVAL_FACTORY_EXPORT
    }
}

pub fn emit_runtime_direct_eval(
    program: &RuntimeDirectEvalProgram,
    runtime: &Arc<RuntimeArtifact>,
    intl_profile: &IntlCompilationProfile,
) -> Result<RuntimeDirectEvalArtifact, EmitError> {
    if let Some(diagnostic) = program.program().wasm_blocking_diagnostic() {
        return Err(EmitError::unsupported(diagnostic.message.clone()));
    }
    let source = witness::root_source(program)?;
    let validated = ValidatedRuntimeArtifact::new(Arc::clone(runtime), intl_profile.clone())?;
    emit_validated(program, &validated, &source)
}

/// Emit P against an immutable R whose current layout and selected Intl image
/// were admitted once. The proof retains the exact R Arc; Wasm linking still
/// checks the actual graph when the native host instantiates this P.
pub fn emit_runtime_direct_eval_with_validated_runtime(
    program: &RuntimeDirectEvalProgram,
    validated: &ValidatedRuntimeArtifact,
) -> Result<RuntimeDirectEvalArtifact, EmitError> {
    if let Some(diagnostic) = program.program().wasm_blocking_diagnostic() {
        return Err(EmitError::unsupported(diagnostic.message.clone()));
    }
    let source = witness::root_source(program)?;
    emit_validated(program, validated, &source)
}

fn emit_validated(
    program: &RuntimeDirectEvalProgram,
    validated: &ValidatedRuntimeArtifact,
    source: &[u16],
) -> Result<RuntimeDirectEvalArtifact, EmitError> {
    let runtime = validated.runtime();
    let emitted = crate::emit::emit_runtime_direct_eval_module(program, validated)?;
    if emitted.runtime_layout.is_some()
        || !emitted
            .artifact
            .runtime()
            .is_some_and(|actual| Arc::ptr_eq(actual, runtime))
    {
        return Err(EmitError::unsupported(
            "native direct eval did not retain its exact linked runtime",
        ));
    }
    Ok(RuntimeDirectEvalArtifact {
        bytes: Arc::from(emitted.artifact.bytes),
        runtime: Arc::clone(runtime),
        caller: program.caller().clone(),
        source: Arc::from(source),
    })
}
