use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn generator_references_preserve_get_put_order_and_whole_abrupt_completion() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for (fixture, expected, strict_allowed) in [
        (
            include_str!("../fixtures/generator_reference_operands/references.js"),
            "generator-reference-operands:ok",
            true,
        ),
        (
            include_str!("../fixtures/generator_reference_operands/web_compat.js"),
            "generator-call-target:ok",
            false,
        ),
    ] {
        let mut sources = vec![fixture.to_owned()];
        if strict_allowed {
            sources.push(format!("'use strict';\n{fixture}"));
        }
        for source in sources {
            let observed = Engine::new(RealmBuilder::new().build())
                .observe_script(
                    &source,
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
                .unwrap();
            assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
            assert!(
                matches!(observed.completion, ObservedCompletion::Normal(_)),
                "{:?}",
                observed.completion
            );
            assert_eq!(
                observed.output_events,
                [HostOutputEvent::PrintLine(expected.into())]
            );
        }
    }
}
