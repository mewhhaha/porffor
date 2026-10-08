use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_template_source(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("template owner source compiles and executes through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn independently_prepared_units_retain_their_own_template_sites() {
    assert_template_source(include_str!(
        "fixtures/tagged_template_owners/independent_units.js"
    ));
}
#[test]
fn fresh_parses_and_escaping_functions_keep_distinct_execution_owners() {
    assert_template_source(include_str!(
        "fixtures/tagged_template_owners/fresh_invocations_and_escaping_closures.js"
    ));
}
#[test]
fn borrowed_functions_and_realm_scripts_use_defining_realm_template_arrays() {
    assert_template_source(include_str!(
        "fixtures/tagged_template_owners/defining_realm.js"
    ));
}
#[test]
fn class_and_resumable_contexts_retain_their_original_template_owner() {
    assert_template_source(include_str!(
        "fixtures/tagged_template_owners/class_and_generator_contexts.js"
    ));
}

#[test]
fn async_resumption_reuses_the_original_captured_template_owner() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            include_str!("fixtures/tagged_template_owners/async_contexts.js"),
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("compiled async template owner source");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(observed.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine("async-template-ok".into())]
    );
}
