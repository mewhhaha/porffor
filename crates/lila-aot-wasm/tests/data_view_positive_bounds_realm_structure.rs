//! Algorithm/error ordering remains covered by the actual CLI cohorts.
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/data_view.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_dataview_constructor_range_error_realm.js");
const TYPE_ERROR_CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_dataview_type_error_realm.js");

#[test]
fn focused_cli_fixture_pins_all_published_constructor_bound_families() {
    let test = CLI_TESTS
        .split_once("fn run_wasm_backend_uses_borrowed_dataview_constructor_range_error_realm()")
        .expect("missing focused DataView constructor Realm test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused DataView constructor Realm test")
        .0;
    assert!(test.contains("wasm_dataview_constructor_range_error_realm.js"));
    assert!(test.contains("boolean(true)"));

    for marker in [
        "borrowed DataView byteOffset ToIndex realm",
        "borrowed DataView byteOffset capacity realm",
        "borrowed DataView byteLength ToIndex realm",
        "borrowed DataView byteLength capacity realm",
        "borrowed DataView post-prototype byteOffset realm",
        "borrowed DataView post-prototype byteLength realm",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}

#[test]
fn focused_cli_fixture_pins_published_data_view_type_error_families_and_order() {
    assert!(CLI_TESTS.contains("fn run_wasm_backend_uses_borrowed_dataview_type_error_realm()"));
    assert!(CLI_TESTS.contains("wasm_dataview_type_error_realm.js"));
    for marker in [
        "borrowed DataView requires new",
        "borrowed DataView invalid buffer",
        "invalid buffer precedes offset coercion",
        "offset coercion precedes detached constructor check",
        "borrowed DataView post-prototype detachment",
        "borrowed DataView getter invalid receiver",
        "borrowed DataView setter invalid receiver",
        "setter receiver check precedes value coercion",
        "borrowed DataView private-slot getter invalid receiver",
        "borrowed DataView getter detached buffer",
        "index coercion precedes detached method check",
        "borrowed DataView getter out-of-bounds view",
    ] {
        assert!(
            TYPE_ERROR_CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}
