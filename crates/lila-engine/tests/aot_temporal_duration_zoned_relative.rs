use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions, WasmExecutionFailureKind,
};
use lila_ir::RuntimeSemanticGap;
fn options() -> CompileOptions {
    CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        ..CompileOptions::default()
    }
}
fn run_options() -> RunOptions {
    RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(120_000),
        ..RunOptions::default()
    }
}
fn assert_duration(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(&source, options(), run_options())
            .expect("zoned Duration must execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
    }
}
#[test]
fn duration_compare_uses_actual_dst_elapsed_targets() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/compare.js"
    ));
}
#[test]
fn duration_total_measures_actual_calendar_window_epochs() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/total.js"
    ));
}
#[test]
fn duration_round_nudges_and_bubbles_across_actual_zone_days() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/round_and_bubble.js"
    ));
}
#[test]
fn duration_relative_options_keep_order_brand_and_abrupt_identity() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/observation.js"
    ));
}
#[test]
fn duration_large_calendar_windows_use_exact_ratio_and_bucket_parity() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/large_windows.js"
    ));
}
#[test]
fn duration_apia_adjacent_supported_operations_keep_exact_results() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/apia_adjacent.js"
    ));
}
#[test]
fn duration_rounding_selects_prescribed_constrained_calendar_windows() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/selected_calendar_windows.js"
    ));
}
#[test]
fn duration_day_windows_retain_exact_dst_spans_and_signed_ties() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/selected_day_windows.js"
    ));
}
#[test]
fn duration_rounding_retains_larger_calendar_fields_and_exact_fold_origin() {
    assert_duration(include_str!(
        "fixtures/temporal_duration_zoned_relative/retained_calendar_window_origins.js"
    ));
}
#[test]
fn unresolved_apia_final_windows_are_owned_uncatchable_semantic_gaps() {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        for duration in ["{days:-1}", "{hours:-25}"] {
            for operation in [
                "d.round({smallestUnit:'days',relativeTo})",
                "d.total({unit:'days',relativeTo})",
                "relativeTo.until(relativeTo.add(d),{smallestUnit:'days'})",
            ] {
                let source = format!(
                    r#"{directive}var relativeTo=Temporal.ZonedDateTime.from('2012-01-01T12:00[Pacific/Apia]');var d=Temporal.Duration.from({duration});var caught=false;try{{{operation};}}catch(error){{caught=true;}}if(caught)throw new Error('gap caught as JS');262;"#
                );
                let error = Engine::new(RealmBuilder::new().build())
                    .observe_script(&source, options(), run_options())
                    .expect_err("unresolved selected window must fail outside JS completion");
                assert_eq!(
                    error.wasm_execution_failure_kind(),
                    Some(WasmExecutionFailureKind::SemanticGap),
                    "{error}\n{source}"
                );
                assert_eq!(
                    error.runtime_semantic_gaps(),
                    vec![RuntimeSemanticGap::TemporalZonedRoundingWindow],
                    "{error}\n{source}"
                );
                assert_eq!(error.wasm_javascript_exception_constructor_name(), None);
                assert!(error.parse_diagnostic().is_none());
                assert!(error.ir_diagnostic().is_none());
            }
        }
    }
}
