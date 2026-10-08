const CLI_MAIN_SOURCE: &str = include_str!("../../lila-cli/tests/cli/main.rs");
const CLI_FUNCTIONS_SOURCE: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CONTRACT: &str = include_str!("../../../docs/rust-rewrite/contracts/main-job-checkpoint.md");
const NORMAL_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections.js");
const PRIMARY_THROW_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections_with_primary_throw.js"
);

#[test]
fn public_rejection_fixtures_require_reporting_without_source_print_calls() {
    assert!(
        !NORMAL_FIXTURE.contains("print(") && !PRIMARY_THROW_FIXTURE.contains("print("),
        "the public fixtures must prove reporting without source-level print reachability"
    );
}

#[test]
fn public_rejection_cli_regressions_remain_registered() {
    assert_eq!(CLI_MAIN_SOURCE.matches("mod functions;").count(), 1);
    for (test_name, fixture_name) in [
        (
            "run_wasm_backend_reports_every_unhandled_rejection_in_fifo_order",
            "wasm_multiple_unhandled_rejections.js",
        ),
        (
            "run_wasm_backend_reports_all_rejections_without_replacing_a_primary_throw",
            "wasm_multiple_unhandled_rejections_with_primary_throw.js",
        ),
    ] {
        let registration = format!("#[test]\nfn {test_name}() {{");
        assert_eq!(
            CLI_FUNCTIONS_SOURCE.matches(&registration).count(),
            1,
            "public CLI regression lost test registration: {test_name}"
        );
        assert_eq!(
            CLI_FUNCTIONS_SOURCE.matches(fixture_name).count(),
            1,
            "public CLI regression lost fixture routing: {fixture_name}"
        );
    }
}

#[test]
fn contract_and_public_fixtures_pin_values_ordering_and_failure_precedence() {
    for marker in [
        "wasm_multiple_unhandled_rejections.js",
        "wasm_multiple_unhandled_rejections_with_primary_throw.js",
        "SymbolDescriptiveString",
        "unhandled rejection diagnostic ToString threw",
        "FIFO order",
        "finite FIFO snapshot",
        "recursively rejecting",
        "print_line_utf8",
        "emit_track_unhandled_rejection",
        "emit_script_with_forced_builtins",
        "realm-owned",
    ] {
        assert!(CONTRACT.contains(marker), "contract lost marker: {marker}");
    }
    for marker in [
        "checkpoint-first",
        "checkpoint-handled",
        "checkpoint-second",
        "checkpoint-symbol",
        "checkpoint-conversion",
        "checkpoint-reentrant",
        "Promise.reject(recursivelyRejected)",
        "checkpoint-third",
    ] {
        assert!(
            NORMAL_FIXTURE.contains(marker),
            "normal-completion fixture lost witness: {marker}"
        );
    }
    for marker in [
        "primary-first",
        "primary-handled",
        "primary-second",
        "primary-conversion",
        "primary-third",
        "primary-script-failure",
    ] {
        assert!(
            PRIMARY_THROW_FIXTURE.contains(marker),
            "primary-throw fixture lost witness: {marker}"
        );
    }
}
