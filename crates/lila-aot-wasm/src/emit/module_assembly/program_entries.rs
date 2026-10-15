//! Typed export plans and native/ordinary module entry points.
use super::*;
use crate::function_layout::ProgramInitializeIndex;
use crate::gc_types::RuntimeSchema;

/// Emits one module. A heap-backed script is split at the runtime/program
/// boundary of the function space (see [`FunctionIndexLayout`]): the runtime
/// module compiles every standard and host builtin and runtime helper, and a
/// program module compiles only what the script owns, importing the rest.
pub(crate) fn emit_script_module(
    script: &ScriptIr,
    kind: ModuleKind<'_>,
    promise_rejection_policy: PromiseRejectionPolicy,
    intl_selection: &IntlDataSelection,
) -> Result<EmittedModule, EmitError> {
    emit_script_module_with_factory(script, kind, promise_rejection_policy, intl_selection, None)
}

/// Force a fresh P against the supplied R; ordinary emit() is never consulted.
pub(crate) fn emit_runtime_function_module(
    program: &lila_ir::RuntimeFunctionProgram,
    runtime: &std::sync::Arc<crate::RuntimeArtifact>,
    intl_selection: &IntlDataSelection,
) -> Result<EmittedModule, EmitError> {
    let mut script = program
        .program()
        .script
        .as_ref()
        .ok_or_else(|| EmitError::unsupported("native Function has no lowered Script IR"))?
        .clone();
    crate::builtins::append_empty_dynamic_function_bodies(&mut script);
    for function in script.functions.iter().filter(|function| {
        function.protocol.execution_kind() == FunctionExecutionKind::AsyncGenerator
    }) {
        if let Some(feature) = function
            .body
            .statements
            .iter()
            .find_map(async_generator_dispatcher_unsupported_feature)
        {
            return Err(EmitError::unsupported(format!("unsupported in lila wasm-aot first slice: async-generator body dispatcher for `{}` does not yet support {feature}", function.name)));
        }
    }
    emit_script_module_with_factory(
        &script,
        ModuleKind::Program(runtime),
        PromiseRejectionPolicy::default(),
        intl_selection,
        Some(program),
    )
}

/// Only the actual layout and validated native source witness mint this plan.
/// Callers cannot choose an independent initializer index or factory slot.
pub(super) struct ProgramExports {
    main: Option<u32>,
    initialize: Option<ProgramInitializeIndex>,
    factory: Option<u32>,
    owns_globals: bool,
}

impl ProgramExports {
    pub(super) fn new(
        kind: ModuleKind<'_>,
        layout: &FunctionIndexLayout,
        factory: Option<&lila_ir::RuntimeFunctionProgram>,
        functions: &FunctionMetaRegistry,
    ) -> Result<Self, EmitError> {
        if matches!(kind, ModuleKind::Program(_)) != layout.program_initialize_index().is_some() {
            return Err(EmitError::unsupported(
                "program exports require their actual typed initializer layout",
            ));
        }
        let factory = if let Some(program) = factory {
            if !matches!(kind, ModuleKind::Program(_)) {
                return Err(EmitError::unsupported(
                    "native Function factory requires its actual linked P",
                ));
            }
            crate::runtime_function::require_factory_plan(program, functions)?;
            Some(RuntimeHelperId::PreparedDynamicFunction.index(layout.helper_base()))
        } else {
            None
        };
        Ok(Self {
            main: kind.compiles_program().then(|| layout.main_index()),
            initialize: layout.program_initialize_index(),
            factory,
            owns_globals: kind.owns_globals(),
        })
    }

    pub(super) fn append(
        &self,
        schema: &RuntimeSchema,
        has_module_entry: bool,
        exports: &mut ExportSection,
    ) {
        if let Some(main) = self.main {
            exports.export("main", ExportKind::Func, main);
            if let Some(index) = self.initialize {
                exports.export(
                    crate::runtime_function::PROGRAM_INITIALIZE_EXPORT,
                    ExportKind::Func,
                    index.raw(),
                );
            }
            if let Some(index) = self.factory {
                exports.export(
                    crate::runtime_function::RUNTIME_FUNCTION_FACTORY_EXPORT,
                    ExportKind::Func,
                    index,
                );
            }
            if has_module_entry {
                exports.export(
                    MODULE_EVALUATION_STATUS_EXPORT,
                    ExportKind::Global,
                    MODULE_EVALUATION_STATUS_GLOBAL_INDEX,
                );
            }
        }
        // Diagnostics/snapshot roots remain in the instance owning the globals.
        if self.owns_globals {
            schema.export_throw_diagnostics(exports);
            schema.export_snapshot_roots(exports);
        }
    }
}
