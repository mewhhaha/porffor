const CLI_SOURCE: &str = include_str!("../../../lila-cli/tests/cli/functions.rs");
const NORMAL_FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections.js");
const PRIMARY_THROW_FIXTURE: &str = include_str!(
    "../../../lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections_with_primary_throw.js"
);

#[test]
fn public_rejection_regressions_retain_their_fixture_routes() {
    for (test_name, fixture_name, fixture) in [
        (
            "run_wasm_backend_reports_every_unhandled_rejection_in_fifo_order",
            "wasm_multiple_unhandled_rejections.js",
            NORMAL_FIXTURE,
        ),
        (
            "run_wasm_backend_reports_all_rejections_without_replacing_a_primary_throw",
            "wasm_multiple_unhandled_rejections_with_primary_throw.js",
            PRIMARY_THROW_FIXTURE,
        ),
    ] {
        assert_eq!(
            CLI_SOURCE.matches(&format!("fn {test_name}() {{")).count(),
            1
        );
        assert_eq!(CLI_SOURCE.matches(fixture_name).count(), 1);
        assert!(fixture.contains("Promise.reject"));
    }
}
