//! Native compilation of already-converted Function source Strings in one Store.
//! Entry/ABI and AOT factory consumers are required before this can be merged.

use super::*;
use lila_aot_wasm::{GcHostImport, RuntimeArtifact};
use lila_intl::IntlCompilationProfile;
use lila_ir::{lower_runtime_function, DynamicFunctionKind, HostSurfacePolicy};
use wasmtime::{ExternType, RootScope, Val};

mod factory;
#[cfg(test)]
mod observer;
mod outcome;
mod signature;
mod sources;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(super) use observer::{NativeFunctionCompilationObserver, NativeFunctionCompilationSnapshot};
use outcome::{CompileRejection, EscapedReply};
use signature::NativeFunctionSignature;
use sources::ConvertedRequest;

/// Execution-owned authority. No P modules, instances or source catalogue are
/// cached here. The linker already contains this Store's actual R instance.
#[derive(Clone)]
pub(super) struct NativeFunctionCompilerContext {
    linker: WasmtimeLinker<WasmHostState>,
    runtime: Arc<RuntimeArtifact>,
    intl_profile: IntlCompilationProfile,
    host_surface_policy: HostSurfacePolicy,
    execution: Arc<WasmAgentExecutionControl>,
    #[cfg(test)]
    test_observer: Option<Arc<NativeFunctionCompilationObserver>>,
}

impl NativeFunctionCompilerContext {
    pub(super) fn new(
        linker: &WasmtimeLinker<WasmHostState>,
        runtime: Arc<RuntimeArtifact>,
        intl_profile: IntlCompilationProfile,
        host_surface_policy: HostSurfacePolicy,
        execution: Arc<WasmAgentExecutionControl>,
    ) -> Self {
        Self {
            linker: linker.clone(),
            runtime,
            intl_profile,
            host_surface_policy,
            execution,
            #[cfg(test)]
            test_observer: None,
        }
    }

    #[cfg(test)]
    pub(super) fn with_test_observer(
        mut self,
        observer: Option<Arc<NativeFunctionCompilationObserver>>,
    ) -> Self {
        self.test_observer = observer;
        self
    }

    fn check_budget(&self) -> wasmtime::Result<()> {
        self.execution
            .remaining()
            .map(|_| ())
            .map_err(wasmtime::Error::new)
    }
}

fn abi_error(message: impl std::fmt::Display) -> wasmtime::Error {
    wasmtime::Error::new(EngineError::new(format!(
        "invalid native Function compiler ABI: {message}"
    )))
}

pub(super) fn link(
    linker: &mut WasmtimeLinker<WasmHostState>,
    module: &WasmtimeModule,
) -> Result<(), EngineError> {
    let row = GcHostImport::CompileDynamicFunction;
    let mut imports = module
        .imports()
        .filter(|import| import.module() == row.module() && import.name() == row.name());
    let Some(import) = imports.next() else {
        return Ok(());
    };
    if imports.next().is_some() {
        return Err(EngineError::new(
            "duplicate native Function compiler import",
        ));
    }
    let ExternType::Func(function) = import.ty() else {
        return Err(EngineError::new(
            "native Function compiler import is not a function",
        ));
    };
    let signature = NativeFunctionSignature::validate(function).map_err(|error| {
        EngineError::new(format!("native Function compiler signature: {error:#}"))
    })?;
    linker
        .func_new(
            row.module(),
            row.name(),
            signature.function.clone(),
            move |caller, inputs, outputs| compile(caller, inputs, outputs, &signature),
        )
        .map_err(|error| {
            EngineError::new(format!("native Function compiler linkage: {error:#}"))
        })?;
    Ok(())
}

fn compile(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    inputs: &[Val],
    outputs: &mut [Val],
    signature: &NativeFunctionSignature,
) -> wasmtime::Result<()> {
    let context = {
        caller
            .data()
            .native_function_compiler
            .clone()
            .ok_or_else(|| abi_error("linked execution has no armed native compiler context"))?
    }; // No State borrow crosses decoding, compilation or a Wasm invocation.
    if !WasmtimeEngine::same(caller.engine(), context.linker.engine()) {
        return Err(abi_error("compiler linker belongs to another Engine"));
    }
    context.check_budget()?;
    #[cfg(test)]
    if let Some(observer) = &context.test_observer {
        observer.request();
    }
    let engine = caller.engine().clone();
    let reply = {
        let mut roots = RootScope::new(&mut caller);
        let request = ConvertedRequest::decode(&mut roots, inputs, signature, &context.execution)?;
        context.check_budget()?;
        let lowered = lower_runtime_function(
            request.kind,
            &request.arguments,
            context.host_surface_policy,
        );
        context.check_budget()?;
        match lowered {
            Ok(program) => {
                let completion = factory::instantiate_and_create(
                    &mut roots, &engine, &context, &request, &program, signature,
                )?;
                let reply = EscapedReply::completion(&mut roots, completion)?;
                #[cfg(test)]
                if let Some(observer) = &context.test_observer {
                    observer.completed_factory();
                }
                reply
            }
            Err(error) => match CompileRejection::classify(error)? {
                CompileRejection::Syntax(message) => {
                    let reply = EscapedReply::syntax(&mut roots, &message, signature)?;
                    context.check_budget()?;
                    #[cfg(test)]
                    if let Some(observer) = &context.test_observer {
                        observer.syntax_rejection();
                    }
                    reply
                }
            },
        }
    }; // Only the actual outgoing reference roots escape the temporary scope.
    context.check_budget()?;
    reply.write(&mut caller, outputs)
}
