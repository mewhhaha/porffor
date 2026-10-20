use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_chunking(source: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    #[cfg(feature = "spec-exec-oracle")]
    let backends = [ExecutionBackend::WasmAot, ExecutionBackend::SpecExec];
    #[cfg(not(feature = "spec-exec-oracle"))]
    let backends = [ExecutionBackend::WasmAot];
    for backend in backends {
        for directive in ["", "'use strict';\n"] {
            let source = format!("{directive}function check(value) {{ if (!value) throw Error('chunking regression'); }}\n{source}\ntrue;");
            let observed = Engine::new(RealmBuilder::new().build())
                .observe_script(
                    &source,
                    CompileOptions {
                        host_surface_policy: HostSurfacePolicy::Test262,
                        ..CompileOptions::default()
                    },
                    RunOptions {
                        backend,
                        timeout_ms: Some(30_000),
                        ..RunOptions::default()
                    },
                )
                .expect("Iterator chunking compiles and executes");
            assert_eq!(observed.backend_used, backend);
            assert_eq!(
                observed.completion,
                ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
                "{backend:?}: {observed:?}\n{source}"
            );
        }
    }
}

#[test]
fn chunking_admission_closes_before_next_without_argument_coercion() {
    assert_chunking(include_str!("../fixtures/iterator_chunking/admission.js"));
}

#[test]
fn chunking_preserves_whole_values_and_fresh_window_arrays() {
    assert_chunking(include_str!("../fixtures/iterator_chunking/values.js"));
}

#[test]
fn chunking_lifecycle_keeps_original_abrupts_and_closes_once() {
    assert_chunking(include_str!("../fixtures/iterator_chunking/lifecycle.js"));
}

#[test]
fn chunking_borrowed_entries_keep_captured_and_unstarted_realms() {
    assert_chunking(include_str!("../fixtures/iterator_chunking/realms.js"));
}
