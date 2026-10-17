use super::*;
pub(super) fn module(
    engine: &WasmtimeEngine,
    mode: WasmNativeCompilationMode,
    bytes: &[u8],
) -> Result<(WasmtimeModule, bool), EngineError> {
    let key = wasm_module_memory_cache_key(WasmModuleSource::Program(bytes), mode);
    let modules = memory_wasm_modules();
    if let Some(module) = modules
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get_for_engine(&key, engine)
    {
        return Ok((module, true));
    }
    let module = compile_wasm_module(engine, bytes)?;
    let module = modules
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .retain_compiled_for_engine(key, module);
    Ok((module, false))
}
