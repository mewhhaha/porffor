//! Native direct eval retains the actual caller's grammar and runtime records.
//! This compiler seam executes no JavaScript and selects no caller environment.

use crate::{HostSurfacePolicy, IrDiagnostic, PreparedScriptKind, ProgramIr, StaticScriptId};
use lila_front::{prepare_eval_source_utf16, EvalParseContext, ParseError};

pub(crate) mod caller;
mod codec;
pub(crate) mod private_capacity;
pub use caller::{
    RuntimeDirectEvalCaller, RuntimeDirectEvalCallerDescriptor, RuntimeDirectEvalCallerError,
    RuntimeDirectEvalDerivedOwner, RuntimeDirectEvalImportOrigin, RuntimeDirectEvalPrivateScope,
};

#[derive(Debug)]
pub enum RuntimeDirectEvalCompileError {
    Parse(ParseError),
    Lowering(Vec<IrDiagnostic>),
    InvalidCompilerOutput(&'static str),
}
impl core::fmt::Display for RuntimeDirectEvalCompileError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(error) => core::fmt::Display::fmt(error, formatter),
            Self::Lowering(diagnostics) => {
                for (index, diagnostic) in diagnostics.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str("; ")?;
                    }
                    formatter.write_str(&diagnostic.message)?;
                }
                Ok(())
            }
            Self::InvalidCompilerOutput(message) => formatter.write_str(message),
        }
    }
}
impl std::error::Error for RuntimeDirectEvalCompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Lowering(_) | Self::InvalidCompilerOutput(_) => None,
        }
    }
}

/// The root is a true prepared direct-eval entry, never a fresh Script main.
/// Invoke it with the same actual caller records that validated `caller`.
#[derive(Debug)]
pub struct RuntimeDirectEvalProgram {
    program: ProgramIr,
    script_id: StaticScriptId,
    caller: RuntimeDirectEvalCaller,
}
impl RuntimeDirectEvalProgram {
    pub fn with_language_features(mut self, policy: crate::LanguageFeaturePolicy) -> Self {
        self.program = self.program.with_language_features(policy);
        self
    }

    pub fn program(&self) -> &ProgramIr {
        &self.program
    }
    pub fn script_id(&self) -> StaticScriptId {
        self.script_id
    }
    pub fn caller(&self) -> &RuntimeDirectEvalCaller {
        &self.caller
    }
}

/// Parse exact code units under caller grammar, then compile a separate owner.
/// The native host retains lexical/variable/private environments, this,
/// new.target, home object and derived cells; this API invents none of them.
pub fn lower_runtime_direct_eval(
    caller: &RuntimeDirectEvalCaller,
    source_units: &[u16],
    host_surface_policy: HostSurfacePolicy,
) -> Result<RuntimeDirectEvalProgram, RuntimeDirectEvalCompileError> {
    let context = caller.context();
    let parsed = prepare_eval_source_utf16(
        source_units,
        &EvalParseContext::Direct(context.parse_context()),
    )
    .map_err(RuntimeDirectEvalCompileError::Parse)?;
    let (program, script_id) = crate::lowering::lower_runtime_direct_eval_program(
        &parsed,
        context.clone(),
        caller.private_environment_floor(),
        host_surface_policy,
    );
    if !program.diagnostics.is_empty() {
        return Err(RuntimeDirectEvalCompileError::Lowering(program.diagnostics));
    }
    let entry = program
        .script
        .as_ref()
        .and_then(|script| {
            script
                .prepared_script_units()
                .find(|unit| unit.id == script_id)
        })
        .ok_or(RuntimeDirectEvalCompileError::InvalidCompilerOutput(
            "native direct eval has no prepared Script root",
        ))?;
    if entry.kind != PreparedScriptKind::DirectEval(context.clone()) {
        return Err(RuntimeDirectEvalCompileError::InvalidCompilerOutput(
            "native direct eval root has a different caller role",
        ));
    }
    Ok(RuntimeDirectEvalProgram {
        program,
        script_id,
        caller: caller.clone(),
    })
}

#[cfg(test)]
mod tests;
