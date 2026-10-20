//! Native-host lowering of already-coerced CreateDynamicFunction source units.
//!
//! This owns syntax and IR only. The Wasm constructor owns coercion, prototype
//! effects and Realm selection; the host must instantiate the emitted P in the
//! calling Store and pass that actual constructor Realm/global environment.

use crate::{
    lower_with_host_surface_policy, CallableToStringRepresentation, DynamicFunctionKind, ExprIr,
    FunctionId, FunctionProtocolIr, GlobalDeclarationSetIr, HostSurfacePolicy, IrDiagnostic,
    ProgramIr, StatementIr, TypedExpr,
};
use scriptw_front::{
    encode_js_string_utf16, prepare_dynamic_function_utf16, FunctionParseKind, ParseError,
    ParsedSource,
};

/// A native compiler rejection retains the producing diagnostic domain.
/// Parser capability gaps and lowering failures are never reported as a
/// successful function or silently converted into JavaScript SyntaxError.
#[derive(Debug)]
pub enum RuntimeFunctionCompileError {
    Parse(ParseError),
    Lowering(Vec<IrDiagnostic>),
    InvalidCompilerOutput(&'static str),
}

impl core::fmt::Display for RuntimeFunctionCompileError {
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

impl std::error::Error for RuntimeFunctionCompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(error) => Some(error),
            Self::Lowering(_) | Self::InvalidCompilerOutput(_) => None,
        }
    }
}

/// A fresh program whose last expression creates exactly the requested source
/// function. Construction verifies the independent global-function grammar and
/// excludes a captured Script activation; callers cannot mint this witness from
/// an arbitrary Script or replace its entry with another FunctionId.
#[derive(Debug)]
pub struct RuntimeFunctionProgram {
    program: ProgramIr,
    function_id: FunctionId,
    kind: DynamicFunctionKind,
}

impl RuntimeFunctionProgram {
    pub fn with_language_features(mut self, policy: crate::LanguageFeaturePolicy) -> Self {
        self.program = self.program.with_language_features(policy);
        self
    }

    pub fn program(&self) -> &ProgramIr {
        &self.program
    }

    pub fn function_id(&self) -> &FunctionId {
        &self.function_id
    }

    pub fn kind(&self) -> DynamicFunctionKind {
        self.kind
    }
}

/// Compile genuine UTF-16 constructor Strings after the runtime has converted
/// each argument exactly once, in order. There is no catalogue or source cap:
/// every request follows the existing front end and ordinary IR lowerer.
///
/// This function executes no JavaScript and creates no Realm or host Engine.
/// The four independent grammars and combined early errors are owned by the
/// same front-end entry used for prepared constructor source candidates.
pub fn lower_runtime_function(
    kind: DynamicFunctionKind,
    arguments: &[Vec<u16>],
    host_surface_policy: HostSurfacePolicy,
) -> Result<RuntimeFunctionProgram, RuntimeFunctionCompileError> {
    let parse_kind = match kind {
        DynamicFunctionKind::Ordinary => FunctionParseKind::Ordinary,
        DynamicFunctionKind::Generator => FunctionParseKind::Generator,
        DynamicFunctionKind::Async => FunctionParseKind::Async,
        DynamicFunctionKind::AsyncGenerator => FunctionParseKind::AsyncGenerator,
    };
    let parsed = prepare_dynamic_function_utf16(parse_kind, arguments)
        .map_err(RuntimeFunctionCompileError::Parse)?;
    let mut program =
        lower_with_host_surface_policy(&ParsedSource::Script(parsed), host_surface_policy);
    if !program.diagnostics.is_empty() {
        return Err(RuntimeFunctionCompileError::Lowering(program.diagnostics));
    }
    let script =
        program
            .script
            .as_mut()
            .ok_or(RuntimeFunctionCompileError::InvalidCompilerOutput(
                "function grammar has no Script IR",
            ))?;
    if !script.owned_env_bindings.is_empty()
        || script.global_bindings.lexical_names().len() != 0
        || script
            .global_bindings
            .iter()
            .any(|binding| binding.declarations != GlobalDeclarationSetIr::None)
    {
        return Err(RuntimeFunctionCompileError::InvalidCompilerOutput(
            "independent function grammar owns Script declarations",
        ));
    }
    let function_id = match script.body.statements.last() {
        Some(StatementIr::Expression(TypedExpr {
            expr: ExprIr::FunctionValue(function_id),
            ..
        })) => function_id.clone(),
        _ => {
            return Err(RuntimeFunctionCompileError::InvalidCompilerOutput(
                "independent function grammar has no function expression result",
            ));
        }
    };
    let function = script
        .functions
        .iter_mut()
        .find(|function| function.id == function_id)
        .ok_or(RuntimeFunctionCompileError::InvalidCompilerOutput(
            "function expression has no declared body",
        ))?;
    let protocol = match kind {
        DynamicFunctionKind::Ordinary => FunctionProtocolIr::OrdinaryCallAndConstruct,
        DynamicFunctionKind::Generator => FunctionProtocolIr::Generator,
        DynamicFunctionKind::Async => FunctionProtocolIr::Async,
        DynamicFunctionKind::AsyncGenerator => FunctionProtocolIr::AsyncGenerator,
    };
    if function.protocol != protocol {
        return Err(RuntimeFunctionCompileError::InvalidCompilerOutput(
            "function expression uses a different execution protocol",
        ));
    }
    if !function.captured_bindings.is_empty() || function.is_named_expression {
        return Err(RuntimeFunctionCompileError::InvalidCompilerOutput(
            "dynamic function captures an independent Script activation",
        ));
    }
    function.name = "anonymous".to_string();
    function.to_string_representation = CallableToStringRepresentation::ExactSource(
        encode_js_string_utf16(&parse_kind.source_text_utf16(arguments)),
    );
    // The native factory reuses the existing program-owned constructor helper
    // for actual Realm, new.target and prototype effects. This row describes
    // this genuinely compiled request; it does not predict future source or
    // substitute a finite source catalogue for native compilation. Place it
    // before any nested prepared requests discovered inside the new body.
    script.prepared_dynamic_functions.insert(
        0,
        crate::PreparedDynamicFunction {
            kind,
            arguments: arguments
                .iter()
                .map(|units| encode_js_string_utf16(units))
                .collect(),
            outcome: crate::PreparedDynamicFunctionOutcome::Compiled {
                function_id: function_id.clone(),
            },
        },
    );
    Ok(RuntimeFunctionProgram {
        program,
        function_id,
        kind,
    })
}

#[cfg(test)]
mod tests;
