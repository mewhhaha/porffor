const OPERATIONS_SOURCE: &str = include_str!("../../src/operations.rs");
const ERROR_SOURCE: &str = include_str!("../../src/builtins/errors/runtime_error.rs");
const CLI_TESTS: &str = include_str!("../../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_typedarray_set_buffer_witness.js");

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
fn to_index_range_errors_use_the_canonical_execution_realm_projection() {
    let projection = bounded(
        ERROR_SOURCE,
        "pub(crate) fn emit_execution_realm(",
        "pub(crate) fn emit_runtime_error_object(",
    );
    let explicit = projection
        .find("if let Some(realm) = self.helper_execution_realm()")
        .unwrap();
    let callable = projection
        .find("if let Some(context) = self.current_function_context()")
        .unwrap();
    let environment = projection
        .find("self.current_environment().load(schema, function)")
        .unwrap();
    let fallback = projection
        .find("let active = self.load_current_realm(function);")
        .unwrap();
    assert!(explicit < callable && callable < environment && environment < fallback);
    for authority in [
        "FunctionContextSchema::REALM",
        "EnvironmentSchema::DEFINING_REALM",
        "EnvironmentSchema::PARENT",
    ] {
        assert_eq!(projection.matches(authority).count(), 1);
    }
    assert!(!projection.contains("load_main_realm"));
    let error = bounded(
        ERROR_SOURCE,
        "pub(crate) fn emit_runtime_error_object(",
        "fn emit_fresh_native_error_object(",
    );
    assert_eq!(
        error
            .matches("let realm = self.emit_execution_realm(function);")
            .count(),
        1
    );
    assert!(error.contains("prototype_slot(kind)"));
    assert!(error
        .contains("self.emit_fresh_native_error_object(&prototype, message, value, function)?;"));
    let slots = bounded(
        ERROR_SOURCE,
        "fn prototype_slot(",
        "impl FunctionBuilder<'_> {",
    );
    assert_eq!(
        slots
            .matches(
                "NativeErrorKind::RangeError => NonArrayRealmIntrinsicSlot::RangeErrorPrototype"
            )
            .count(),
        1
    );
    assert!(!slots.contains("_ =>"));
}

#[test]
fn to_index_cannot_bypass_the_execution_realm_projection() {
    let to_index = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn emit_to_index_from_number_payload(",
        "pub(crate) fn emit_value_to_number_payload(",
    );
    let lower = to_index.find("Instruction::F64Lt").unwrap();
    let upper = to_index.find("Instruction::F64Gt").unwrap();
    let join = to_index.find("Instruction::I32Or").unwrap();
    let error = to_index.find("self.emit_throw_runtime_error(").unwrap();
    let success = to_index.find("Instruction::I64TruncF64U").unwrap();
    assert!(lower < upper && upper < join && join < error && error < success);
    assert!(to_index.contains("MAX_SAFE_INTEGER as f64"));
    assert!(to_index.contains("NativeErrorKind::RangeError,"));
    assert_eq!(
        to_index.matches("emit_throw_runtime_error(").count(),
        1,
        "both invalid ranges join at the same Realm-aware throw"
    );
    assert!(!to_index.contains("load_current_realm("));
    assert!(!to_index.contains("emit_throw_current_function_realm_range_error("));
    let conversion = bounded(
        OPERATIONS_SOURCE,
        "pub(crate) fn emit_to_index_i64_from_value_locals(",
        "pub(crate) fn emit_to_index_from_number_payload(",
    );
    assert!(conversion.contains("CompletionKind::Normal.code() as i32"));
    assert!(
        conversion
            .find("self.emit_value_to_number_payload(")
            .unwrap()
            < conversion
                .find("self.emit_to_index_from_number_payload(")
                .unwrap()
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
