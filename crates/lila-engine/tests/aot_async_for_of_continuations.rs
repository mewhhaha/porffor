use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedRunOutcome, RealmBuilder, RunOptions,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn assert_output(observed: ObservedRunOutcome, expected: &str) {
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine(expected.into())]
    );
}

fn assert_script(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("async iterator body compiles and executes");
    assert_output(observed, "ok");
}

#[test]
fn catches_keep_exact_undefined_rejections() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/catch-undefined.js"
    ));
}

#[test]
fn each_nested_clause_runs_once_per_iteration() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/body-catch-finally-order.js"
    ));
}

#[test]
fn escaping_rejection_wins_over_iterator_close_failure() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/escaping-throw-preserves-identity-through-close.js"
    ));
}

#[test]
fn returning_body_awaits_finally_before_closing_once() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/return-through-awaited-finally-closes-once.js"
    ));
}

#[test]
fn finalizer_rejection_replaces_return_and_wins_over_close_failure() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/finally-rejection-replaces-return-before-close.js"
    ));
}

#[test]
fn closures_retain_distinct_iteration_body_and_catch_environments() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/captured-iteration-body-catch-environments.js"
    ));
}

#[test]
fn eager_clauses_and_conditional_awaits_compose_between_suspensions() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/eager-and-conditional-try-between-awaits.js"
    ));
}

#[test]
fn neighboring_direct_await_loop_keeps_its_behavior() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/direct-await-loop-control.js"
    ));
}

#[test]
fn neighboring_standalone_try_keeps_its_behavior() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/standalone-try-catch-finally-control.js"
    ));
}

#[test]
fn foreign_realm_rejection_retains_identity_through_all_clauses() {
    assert_script(include_str!(
        "fixtures/async_for_of_continuations/foreign-realm-rejection.js"
    ));
}

struct Modules(PathBuf);

impl Drop for Modules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn original_module_loop_catches_both_concurrent_import_rejections() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = Modules(std::env::temp_dir().join(format!(
        "lila-async-for-of-module-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    )));
    std::fs::create_dir_all(&fixture.0).expect("create module graph");
    let entry = include_str!("fixtures/async_for_of_continuations/original-module-loop/entry.js");
    std::fs::write(fixture.0.join("entry.js"), entry).expect("write entry");
    std::fs::write(
        fixture.0.join("bad.js"),
        include_str!("fixtures/async_for_of_continuations/original-module-loop/bad.js"),
    )
    .expect("write rejecting dependency");
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_module(
            entry,
            CompileOptions {
                filename: Some(fixture.0.join("entry.js").to_str().unwrap().into()),
                module_root: Some(fixture.0.to_str().unwrap().into()),
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("original module graph compiles and executes");
    assert_output(observed, "undefined import rejection");
}
