//! Retain the actual borrowed-method cohort after duplicate publication retires.
const CLI_TESTS: &str = include_str!("../../../lila-cli/tests/cli/data_view.rs");
const CLI_FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_dataview_created_realm_prototype.js");

#[test]
fn focused_cli_fixture_borrows_created_realm_data_view_methods() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_borrows_created_realm_dataview_prototype_methods()"));
    assert!(CLI_TESTS.contains("wasm_dataview_created_realm_prototype.js"));
    for marker in [
        "borrowed getter positive bound",
        "borrowed setter positive bound",
        "getter method realm identity",
        "setter method realm identity",
        "created realm toStringTag descriptor",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}
