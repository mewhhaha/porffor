//! Actual Engine direct-eval misses, with one observer per native execution.
use super::*;
use wasm_native_direct_eval_compiler::{
    NativeDirectEvalCompilationObserver, NativeDirectEvalCompilationSnapshot,
};

mod caller_records;
mod empty_blocks;
mod empty_cases;
mod ownership;
mod source_and_errors;

fn converted_text(text: &str) -> String {
    format!(
        "String.fromCharCode({})",
        text.encode_utf16()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}
fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn emit_native_artifact(engine: &Engine, unit: &CompilationUnit) -> Artifact {
    // Keep the inspected IR, selecting the same host profile as native execution.
    // Public emit_wasm deliberately produces a portable artifact instead.
    let artifact = run_on_sized_stack(|| {
        engine.emit_wasm_with_cache_on_current_thread(
            unit,
            None,
            lila_aot_wasm::RuntimeSourceCompilationProfile::NativeHost,
        )
    })
    .unwrap();
    assert_eq!(
        lila_aot_wasm::RuntimeSourceCompilationProfile::from_module(&artifact.bytes).unwrap(),
        lila_aot_wasm::RuntimeSourceCompilationProfile::NativeHost
    );
    artifact
}

fn assert_trace(
    source: &str,
    expected: &[&str],
    unmatched: &[Vec<u16>],
    counts: (usize, usize, usize),
) -> NativeDirectEvalCompilationSnapshot {
    configure_compilation_jobs(1).unwrap();
    let observer = Arc::new(NativeDirectEvalCompilationObserver::default());
    let mut engine = Engine::new(RealmBuilder::new().build());
    engine.native_direct_eval_compilation_observer = Some(Arc::clone(&observer));
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
    for source in unmatched {
        assert!(
            !script.prepared_scripts.iter().any(|row| matches!(
                row.kind,
                lila_ir::PreparedScriptKind::DirectEval(_)
            ) && lila_front::decode_js_string_utf16(
                &row.source
            ) == *source),
            "request must miss the actual finite direct-eval guard: {source:?}"
        );
    }
    let artifact = emit_native_artifact(&engine, &unit);
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
            .map(|text| HostOutputEvent::PrintLine((*text).into()))
            .collect::<Vec<_>>(),
        "{source}"
    );
    let observed = observer.snapshot();
    assert_eq!(
        (
            observed.requests,
            observed.completed_factories,
            observed.syntax_rejections
        ),
        counts,
        "actual native direct-eval phases"
    );
    observed
}
