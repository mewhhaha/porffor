//! Shared native code with fresh per-P data and unchanged calling Store/Realm.
use super::*;
use crate::wasm_native_source_compiler::{CallbackTrace, NativeSourceCompilerContext, Phase};
use lila_aot_wasm::{NativeProgramTemplate, NATIVE_PROGRAM_DATA_MODULE};
use wasmtime::{AsContextMut, Instance};
mod bindings;
mod cache;
pub(super) use bindings::NativeArrayConstructorCache;

pub(super) fn instantiate(
    store: &mut impl AsContextMut<Data = WasmHostState>,
    engine: &WasmtimeEngine,
    context: &NativeSourceCompilerContext,
    template: &NativeProgramTemplate,
    trace: Option<&CallbackTrace<'_>>,
) -> wasmtime::Result<Instance> {
    context.check_budget()?;
    if !WasmtimeEngine::same(engine, context.linker.engine())
        || !Arc::ptr_eq(template.runtime(), context.runtime())
    {
        return Err(wasmtime::Error::msg(
            "native template has a different actual Engine or R owner",
        ));
    }
    let lookup = trace.map(|trace| trace.phase(Phase::CompiledModuleLookup));
    let (module, hit) = cache::module(engine, context.native_compilation_mode, template.bytes())
        .map_err(wasmtime::Error::new)?;
    context.check_budget()?;
    drop(lookup);
    let bind = trace.map(|trace| trace.phase(Phase::BindImports));
    let mut imports = Vec::new();
    for import in module.imports() {
        let value = if import.module() == NATIVE_PROGRAM_DATA_MODULE {
            bindings::data_import(&mut *store, context, template, &import)?
        } else {
            context
                .linker
                .get(&mut *store, import.module(), import.name())
                .map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "native template lacks actual import {}.{}: {error}",
                        import.module(),
                        import.name()
                    ))
                })?
        };
        imports.push(value);
    }
    context.check_budget()?;
    drop(bind);
    let instantiate = trace.map(|trace| trace.phase(Phase::Instantiate));
    let instance = Instance::new(&mut *store, &module, &imports)?;
    context.check_budget()?;
    drop(instantiate);
    if std::env::var_os("LILA_WASM_TRACE").is_some() {
        eprintln!(
            "lila native template: cache={} code_bytes={} imports={} images={} factories={}",
            if hit { "hit" } else { "miss" },
            template.bytes().len(),
            imports.len(),
            template.data_images().len(),
            template.array_factories().len()
        );
    }
    Ok(instance)
}
