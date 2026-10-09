use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion,
    ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_script_in_both_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let script = format!("{directive}{source}\ntrue;");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &script,
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
            .unwrap_or_else(|error| panic!("parseInt radix execution failed: {error}\n{script}"));
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{script}"
        );
        assert!(observed.output_events.is_empty(), "{script}");
    }
}

#[test]
fn finite_extremes_and_nonfinite_radices_select_default_prefix_rules() {
    assert_script_in_both_modes(include_str!(
        "../fixtures/parse_int_radix/default_radices.js"
    ));
}

#[test]
fn modular_fractional_and_signed_radices_obey_the_closed_valid_range() {
    assert_script_in_both_modes(include_str!(
        "../fixtures/parse_int_radix/modular_radices.js"
    ));
}

#[test]
fn string_then_radix_conversion_runs_once_and_observes_mutation() {
    assert_script_in_both_modes(include_str!(
        "../fixtures/parse_int_radix/conversion_order.js"
    ));
}

#[test]
fn abrupt_conversion_preserves_thrown_values_and_skips_later_work() {
    assert_script_in_both_modes(include_str!(
        "../fixtures/parse_int_radix/abrupt_conversion.js"
    ));
}

const STA: &str = include_str!("../../../../test262/vendor/test262/harness/sta.js");
const ASSERT: &str = include_str!("../../../../test262/vendor/test262/harness/assert.js");

#[test]
fn complete_pinned_number_parse_int_source_passes_in_both_modes() {
    let source =
        include_str!("../../../../test262/vendor/test262/test/staging/sm/Number/parseInt-01.js");
    assert_script_in_both_modes(&format!("{STA}\n{ASSERT}\n{source}"));
}

#[test]
fn complete_pinned_global_parse_int_source_passes_in_both_modes() {
    let source =
        include_str!("../../../../test262/vendor/test262/test/staging/sm/global/parseInt-01.js");
    assert_script_in_both_modes(&format!("{STA}\n{ASSERT}\n{source}"));
}
