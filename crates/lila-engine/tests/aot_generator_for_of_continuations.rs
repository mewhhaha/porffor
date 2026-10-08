use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_script(source: &str, fixture: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .unwrap_or_else(|error| {
            panic!("{fixture} must compile and execute through Wasm AOT: {error}")
        });
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
        "{fixture}: {}\noutput: {:?}",
        observed.note,
        observed.output_events,
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine("ok".into())],
        "{fixture}: {}",
        observed.note,
    );
}

macro_rules! fixture_test {
    ($test:ident, $file:literal) => {
        #[test]
        fn $test() {
            let source = include_str!(concat!("fixtures/generator_for_of_continuations/", $file));
            assert_script(source, concat!($file, " (sloppy)"));
            let strict_source = format!("\"use strict\";\n{source}");
            assert_script(&strict_source, concat!($file, " (strict)"));
        }
    };
}

fixture_test!(
    cached_next_and_iterator_values_survive_multiple_yields,
    "protocol-cached-next-multiple-yields.js"
);
fixture_test!(
    captured_iteration_body_and_catch_cells_remain_per_owner,
    "captured-iteration-body-catch-interleaved.js"
);
fixture_test!(
    const_head_capture_survives_resumption,
    "const-head-captured-cell.js"
);
fixture_test!(
    delegated_yields_and_finalizers_complete_each_iteration_once,
    "yield-star-try-finally-segments.js"
);
fixture_test!(
    return_waits_for_yielding_finally_before_iterator_close,
    "injected-return-finally-reyield-close.js"
);
fixture_test!(
    throw_is_caught_or_escapes_with_original_identity,
    "injected-throw-caught-and-escaping.js"
);
fixture_test!(
    iterator_close_preserves_throw_and_replaces_return_errors,
    "iterator-close-completion-precedence.js"
);
fixture_test!(
    finalizer_completion_replaces_pending_return_or_throw_before_close,
    "finally-replaces-pending-completion.js"
);
fixture_test!(
    iterator_operation_errors_do_not_enter_the_body_close_path,
    "iterator-operation-errors-do-not-close.js"
);
fixture_test!(
    local_control_keeps_cached_next_and_per_iteration_cells,
    "local-control-cached-next-cells.js"
);
fixture_test!(
    yielding_finalizers_select_the_local_completion_before_close,
    "finalizer-replaces-local-control.js"
);
fixture_test!(
    local_break_close_errors_reach_outer_catch_and_finally,
    "local-break-close-route.js"
);
fixture_test!(
    assignment_head_retains_outer_binding_across_yields_and_interleaved_instances,
    "assignment-head-capture-interleaved.js"
);
fixture_test!(
    assignment_head_errors_preserve_throw_through_close_and_yielding_finally,
    "assignment-head-errors-close.js"
);
fixture_test!(
    assignment_head_local_and_injected_completions_wait_for_finalizers,
    "assignment-head-local-finalizers.js"
);

fixture_test!(
    property_head_order_and_receiver_are_repeated_only_at_iteration_entry,
    "property-head-order-and-resume.js"
);
fixture_test!(
    property_head_abrupts_close_before_body_and_outer_yielding_cleanup,
    "property-head-errors-close.js"
);
fixture_test!(
    property_head_receivers_and_local_completions_survive_yielding_finalizers,
    "property-head-receivers-and-finalizers.js"
);

fixture_test!(
    lexical_patterns_initialize_fresh_cells_once_before_each_yielded_iteration,
    "lexical-pattern-capture-interleaved.js"
);
fixture_test!(
    lexical_pattern_abrupts_preserve_identity_and_close_the_correct_iterator,
    "lexical-pattern-errors-close.js"
);
fixture_test!(
    lexical_pattern_cells_survive_local_and_injected_yielding_finalizers,
    "lexical-pattern-local-finalizers.js"
);
