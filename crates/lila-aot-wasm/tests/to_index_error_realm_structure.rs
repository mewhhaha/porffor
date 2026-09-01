const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_set_buffer_witness.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn to_index_range_errors_use_the_closed_numeric_realm_projection() {
    let projection = bounded(
        OPERATIONS_SOURCE,
        "fn numeric_conversion_realm_access(",
        "fn spec_operation_property_key_operand(",
    );
    for source in [
        "NumericErrorRealmSource::GlobalFallback",
        "NumericErrorRealmSource::StandardBuiltinEnvironment",
        "NumericErrorRealmSource::NumericConversionHelperArgument",
    ] {
        assert_eq!(projection.matches(source).count(), 1, "{source}");
    }
    assert!(!projection.contains("_ =>"));

    let range_error = bounded(
        OPERATIONS_SOURCE,
        "fn emit_numeric_conversion_range_error(",
        "fn finish_may_throw_operation(",
    );
    assert_eq!(
        range_error
            .matches("numeric_conversion_realm_access(self.numeric_error_realm_source())")
            .count(),
        1
    );
    assert_eq!(
        range_error
            .matches("NumericConversionRealmAccess::TrustedCurrentEnvironment =>")
            .count(),
        1
    );
    assert_eq!(
        range_error
            .matches("NumericConversionRealmAccess::MainRealmFallback =>")
            .count(),
        1
    );
    assert_eq!(
        range_error
            .matches("emit_throw_current_function_realm_range_error(")
            .count(),
        1
    );
    assert_eq!(range_error.matches("emit_throw_runtime_error(").count(), 1);
    assert!(!range_error.contains("LocalGet(self.current_env_local)"));
}

#[test]
fn to_index_cannot_bypass_the_numeric_realm_projection() {
    let to_index = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn emit_to_index_from_number_payload(",
        "pub(crate) fn emit_value_to_number_payload(",
    );
    assert_eq!(
        to_index
            .matches("emit_numeric_conversion_range_error(")
            .count(),
        2
    );
    assert!(!to_index.contains("emit_throw_runtime_error("));
    assert!(!to_index.contains("emit_throw_current_function_realm_range_error("));

    let constructors = bounded(EMIT_SOURCE, "fn new_main(", "fn new(");
    assert_eq!(
        constructors
            .matches("NumericErrorRealmSource::GlobalFallback")
            .count(),
        4,
        "main, user, host and ordinary runtime-operation bodies must keep the main-Realm fallback"
    );
    assert_eq!(
        constructors
            .matches("NumericErrorRealmSource::StandardBuiltinEnvironment")
            .count(),
        1,
        "standard builtins must retain their self-backed Realm environment"
    );
}

#[test]
fn borrowed_typed_array_set_fixture_pins_the_to_index_error_realm() {
    let cli_test = CLI_TESTS
        .split_once("fn run_wasm_backend_revalidates_typedarray_set_buffer_witnesses()")
        .expect("missing focused TypedArray set witness CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused TypedArray set witness CLI test")
        .0;
    assert!(cli_test.contains("wasm_typedarray_set_buffer_witness.js"));

    assert!(CLI_FIXTURE.contains("borrowed set negative offset ToIndex"));
    assert!(CLI_FIXTURE.contains("otherSet.call(new Uint8Array(0), [], -1)"));
    assert!(CLI_FIXTURE
        .contains("other.RangeError.prototype, \"borrowed set negative offset ToIndex\""));
}
