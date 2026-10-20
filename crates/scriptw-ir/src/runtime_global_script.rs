//! Native-host parsing and IR lowering for independent global Script requests.
//!
//! Execution Realm, coercion, global declaration admission and completion
//! transport belong to the runtime. This module executes no source and creates
//! neither a Realm nor a host Engine.

use crate::{HostSurfacePolicy, IrDiagnostic, PreparedScriptKind, ProgramIr, StaticScriptId};
use scriptw_front::{parse_script_utf16, prepare_eval_source_utf16, EvalParseContext, ParseError};

/// Only source operations whose variable environment is selected independently
/// of a caller activation can enter this compiler route. Direct eval needs its
/// caller lexical/variable/private environments and DirectEvalContextIr instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeGlobalScriptKind {
    IndirectEval,
    RealmScript,
}

impl RuntimeGlobalScriptKind {
    pub(crate) fn prepared_kind(self) -> PreparedScriptKind {
        match self {
            Self::IndirectEval => PreparedScriptKind::IndirectEval,
            Self::RealmScript => PreparedScriptKind::RealmScript,
        }
    }
}

/// Syntax rejection retains the parser's classified diagnostic; unsupported
/// lowering remains a compiler rejection, never a fabricated SyntaxError.
#[derive(Debug)]
pub enum RuntimeGlobalScriptCompileError {
    Parse(ParseError),
    Lowering(Vec<IrDiagnostic>),
    InvalidCompilerOutput(&'static str),
}

impl core::fmt::Display for RuntimeGlobalScriptCompileError {
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

impl std::error::Error for RuntimeGlobalScriptCompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Lowering(_) | Self::InvalidCompilerOutput(_) => None,
        }
    }
}

/// One independently compiled Script entry and its complete ProgramIr owner.
/// The private witness cannot be minted from a fresh-entry Script or direct
/// eval. Execute script_id through the prepared-Script entry ABI, not main.
#[derive(Debug)]
pub struct RuntimeGlobalScriptProgram {
    program: ProgramIr,
    script_id: StaticScriptId,
    kind: RuntimeGlobalScriptKind,
}

impl RuntimeGlobalScriptProgram {
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

    pub fn kind(&self) -> RuntimeGlobalScriptKind {
        self.kind
    }
}

/// Parse the actual, already-coerced ECMAScript String on every request. The
/// independent eval grammar does not inherit caller strictness, private names,
/// new.target or super authority. Realm Script uses the actual Script grammar.
/// There is no source catalogue limit or source-shape/code-image shortcut here.
///
/// The host must retain the selected Realm and apply its normal declaration,
/// execution and error-identity rules. This IR seam defines no host ABI.
pub fn lower_runtime_global_script(
    kind: RuntimeGlobalScriptKind,
    source_units: &[u16],
    host_surface_policy: HostSurfacePolicy,
) -> Result<RuntimeGlobalScriptProgram, RuntimeGlobalScriptCompileError> {
    let parsed = match kind {
        RuntimeGlobalScriptKind::IndirectEval => {
            prepare_eval_source_utf16(source_units, &EvalParseContext::Indirect)
        }
        RuntimeGlobalScriptKind::RealmScript => parse_script_utf16(source_units),
    }
    .map_err(RuntimeGlobalScriptCompileError::Parse)?;
    let (program, script_id) =
        crate::lowering::lower_runtime_global_script_program(&parsed, kind, host_surface_policy);
    if !program.diagnostics.is_empty() {
        return Err(RuntimeGlobalScriptCompileError::Lowering(
            program.diagnostics,
        ));
    }
    let entry = program
        .script
        .as_ref()
        .and_then(|script| {
            script
                .prepared_script_units()
                .find(|unit| unit.id == script_id)
        })
        .ok_or(RuntimeGlobalScriptCompileError::InvalidCompilerOutput(
            "independent global source has no prepared Script entry",
        ))?;
    if entry.kind != kind.prepared_kind() {
        return Err(RuntimeGlobalScriptCompileError::InvalidCompilerOutput(
            "independent global source has a different Script execution role",
        ));
    }
    Ok(RuntimeGlobalScriptProgram {
        program,
        script_id,
        kind,
    })
}

#[cfg(test)]
mod tests;
