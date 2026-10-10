use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn async_array_patterns_preserve_iterator_records_original_targets_and_close_through_await() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    for directive in ["", "'use strict';\n"] {
        for (fixture, label) in [
            (
                include_str!("../fixtures/async_array_patterns/bindings.js"),
                "async-array-bindings:ok",
            ),
            (
                include_str!("../fixtures/async_array_patterns/assignments.js"),
                "async-array-assignments:ok",
            ),
        ] {
            let observed = Engine::new(RealmBuilder::new().build())
                .observe_script(
                    &format!("{directive}{fixture}"),
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
                vec![HostOutputEvent::PrintLine(label.into())]
            );
        }
    }
}

#[test]
fn suspended_nested_patterns_initialize_the_enclosing_binding_scope() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let source = r#"
        function check(ok) { if (!ok) throw 'pattern binding scope'; }
        const whole = {marker: 53};
        async function plain() {
            const [[received = await whole]] = [[undefined]];
            await 0; gc();
            return received;
        }
        function* sync() {
            const [[received = yield 'sync']] = [[undefined]];
            gc(); yield received;
            return received;
        }
        async function* mixed() {
            const [[received = await (yield 'mixed')]] = [[undefined]];
            await 0; gc(); yield received;
            return received;
        }
        async function run() {
            check(await plain() === whole);
            let iterator = sync();
            check(iterator.next().value === 'sync');
            check(iterator.next(whole).value === whole);
            let result = iterator.next(); check(result.done && result.value === whole);
            iterator = mixed();
            check((await iterator.next()).value === 'mixed');
            check((await iterator.next(whole)).value === whole);
            result = await iterator.next(); check(result.done && result.value === whole);
            print('pattern-binding-scope:ok');
        }
        run();
    "#;
    for directive in ["", "'use strict';\n"] {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
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
            [HostOutputEvent::PrintLine(
                "pattern-binding-scope:ok".into()
            )]
        );
    }
}
