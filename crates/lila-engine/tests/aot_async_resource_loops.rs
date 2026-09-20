use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedRunOutcome, RealmBuilder, RunOptions,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

fn check(observed: ObservedRunOutcome) {
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        [HostOutputEvent::PrintLine("ok".into())]
    );
}

fn script(source: &str) {
    lila_engine::configure_compilation_jobs(1).unwrap();
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
        .expect("non-suspending resource loop compiles and executes");
    check(observed);
}

#[test]
fn classic_resources_survive_continue_and_dispose_before_the_following_await() {
    script(include_str!(
        "fixtures/async_resource_loops/classic-order.js"
    ));
}

#[test]
fn iteration_resources_close_after_disposal_and_keep_distinct_captures() {
    script(include_str!(
        "fixtures/async_resource_loops/iteration-close-and-captures.js"
    ));
}

#[test]
fn later_initializer_failure_disposes_already_acquired_resources() {
    script(include_str!(
        "fixtures/async_resource_loops/initializer-throw.js"
    ));
}

#[test]
fn disposal_suppression_precedes_iterator_close_and_preserves_exact_errors() {
    script(include_str!(
        "fixtures/async_resource_loops/body-disposal-close-errors.js"
    ));
}

#[test]
fn returning_a_thenable_disposes_before_async_result_adoption() {
    script(include_str!(
        "fixtures/async_resource_loops/return-disposal-before-adoption.js"
    ));
}

#[test]
fn nested_resource_owners_and_foreign_rejection_identity_are_retained() {
    script(include_str!(
        "fixtures/async_resource_loops/foreign-error-and-nested-resources.js"
    ));
}

#[test]
fn eager_class_evaluation_and_nested_async_function_owners_remain_separate() {
    script(include_str!(
        "fixtures/async_resource_loops/nested-functions-and-class-evaluation.js"
    ));
}

#[test]
fn eager_resource_loop_clauses_compose_inside_a_resumable_iterator_body() {
    script(include_str!(
        "fixtures/async_resource_loops/within-resumable-for-of.js"
    ));
}

struct Modules(PathBuf);
impl Drop for Modules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn async_module_resource_loops_retain_their_canonical_activation_owner() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = Modules(std::env::temp_dir().join(format!(
        "lila-async-resource-loop-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&fixture.0).unwrap();
    let entry = include_str!("fixtures/async_resource_loops/module/entry.js");
    std::fs::write(fixture.0.join("entry.js"), entry).unwrap();
    std::fs::write(
        fixture.0.join("dependency.js"),
        include_str!("fixtures/async_resource_loops/module/dependency.js"),
    )
    .unwrap();
    lila_engine::configure_compilation_jobs(1).unwrap();
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
        .expect("async module resource loop graph compiles and executes");
    check(observed);
}
