//! Production Engine controls for actual global-source misses.
use super::*;
use lila_ir::PreparedScriptKind;
use wasm_native_function_compiler::{
    NativeFunctionCompilationObserver, NativeFunctionCompilationSnapshot,
};
use wasm_native_global_script_compiler::{
    NativeGlobalScriptCompilationObserver, NativeGlobalScriptCompilationSnapshot,
};
mod declarations;
mod ownership;
mod realm;
mod source_units;

fn converted_text(text: &str) -> String {
    format!(
        "String.fromCharCode({})",
        text.encode_utf16()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn global_counts(
    requests: usize,
    factories: usize,
    syntax: usize,
) -> NativeGlobalScriptCompilationSnapshot {
    NativeGlobalScriptCompilationSnapshot {
        requests,
        completed_factories: factories,
        syntax_rejections: syntax,
    }
}
fn function_counts(factories: usize) -> NativeFunctionCompilationSnapshot {
    NativeFunctionCompilationSnapshot {
        requests: factories,
        completed_factories: factories,
        syntax_rejections: 0,
    }
}
fn assert_native_trace(
    source: &str,
    expected: &[&str],
    unmatched: &[(PreparedScriptKind, Vec<u16>)],
    iterator: bool,
    counts: NativeGlobalScriptCompilationSnapshot,
    function_factories: usize,
) {
    configure_compilation_jobs(1).unwrap();
    let observer = Arc::new(NativeGlobalScriptCompilationObserver::default());
    let function_observer = Arc::new(NativeFunctionCompilationObserver::default());
    let mut engine = Engine::new(RealmBuilder::new().build());
    engine.native_global_script_compilation_observer = Some(Arc::clone(&observer));
    engine.native_function_compilation_observer = Some(Arc::clone(&function_observer));
    let unit = engine
        .compile_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
        )
        .unwrap();
    let script = unit.ir.script.as_ref().unwrap();
    for (kind, source_units) in unmatched {
        assert!(
            !script.prepared_scripts.iter().any(|row| &row.kind == kind
                && lila_front::decode_js_string_utf16(&row.source) == *source_units),
            "actual native request must be excluded from the finite prepared guard: {kind:?} {source_units:?}"
        );
    }
    if iterator {
        assert!(script
            .body
            .statements
            .iter()
            .any(|s| matches!(s, lila_ir::StatementIr::ForOfIterator { .. })));
    }
    // Reuse this exact inspected IR and emit once; do not compile the outer source again.
    let artifact = engine.emit_wasm(&unit).unwrap();
    let outcome = run_on_sized_stack(|| {
        engine
            .execute_with_wasm_bytes_inner(
                artifact.program(),
                Some(30_000),
                false,
                WasmModuleMemoryCachePolicy::Retain,
                &WasmExecutionMode::Structured,
            )
            .and_then(WasmExecutionOutcome::into_structured)
    })
    .unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|s| HostOutputEvent::PrintLine((*s).into()))
            .collect::<Vec<_>>(),
        "{source}"
    );
    assert_eq!(observer.snapshot(), counts, "actual global compiler phases");
    assert_eq!(
        function_observer.snapshot(),
        function_counts(function_factories),
        "actual independent Function compiler phases"
    );
}
