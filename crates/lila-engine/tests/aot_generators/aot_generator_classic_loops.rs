use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_loop_fixture(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
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
        .expect("classic generator loops compile and execute through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(observed.output_events, expected);
}

#[test]
fn classic_generator_heads_and_branches_resume_each_acquired_phase_once() {
    assert_loop_fixture(
        include_str!("../fixtures/generator_classic_loops/phases_and_branches.js"),
        &[
            "init-a:false",
            "test-a:0:false",
            "test-b:0:false",
            "body-a:0:false",
            "body-b:0:false",
            "update-a:0:false",
            "update-b:0:false",
            "test-a:2:false",
            "test-b:2:false",
            "else:2:false",
            "done:2:true",
            "while-test:0:false",
            "while-a:0:false",
            "while-b:0:false",
            "while-test:1:false",
            "1:true",
            "do-body:0:false",
            "do-test:1:false",
            "do-body:1:false",
            "do-test:2:false",
            "2:true",
        ],
    );
}

#[test]
fn classic_generator_loops_keep_labelled_finalizers_iteration_cells_and_whole_abrupts() {
    assert_loop_fixture(
        include_str!("../fixtures/generator_classic_loops/completions_and_environments.js"),
        &[
            "labels:true",
            "nested:true",
            "cells:true",
            "return:true",
            "throw:true",
        ],
    );
}
