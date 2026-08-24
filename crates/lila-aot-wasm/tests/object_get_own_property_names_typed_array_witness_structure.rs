const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_object_get_own_property_names_typed_array_witness.js"
);

const PRIVATE_STATE_WIRING: &str = r#"
        self.emit_load_typed_array_private_state(
            arg_payload_local,
            typed_array_buffer_payload_local,
            typed_array_byte_offset_local,
            typed_array_stored_byte_length_local,
            typed_array_bytes_per_element_local,
            function,
        );
"#;

const VIEW_WIRING: &str = r#"
        let typed_array_view = TypedArrayViewLocals::new(
            arg_payload_local,
            typed_array_buffer_payload_local,
            typed_array_byte_offset_local,
            typed_array_stored_byte_length_local,
            typed_array_bytes_per_element_local,
        );
"#;

const WITNESS_WIRING: &str = r#"
        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::ArrayLikeLengthSnapshot {
                length_local: typed_array_length_local,
            },
            function,
        )?;
"#;

fn owner_body() -> &'static str {
    OBJECT_SOURCE
        .split_once("pub(super) fn compile_object_get_own_property_names_builtin(")
        .expect("missing Object.getOwnPropertyNames compiler")
        .1
        .split_once("pub(super) fn compile_object_get_own_property_symbols_builtin(")
        .expect("missing Object.getOwnPropertyNames compiler boundary")
        .0
}

fn typed_array_observation() -> &'static str {
    owner_body()
        .split_once(
            r#"
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(typed_array_brand_local));
"#,
        )
        .expect("missing TypedArray brand initialization")
        .1
        .split_once(
            "self.load_i64_to_local_from_offset(arg_payload_local, HEAP_LEN_OFFSET, len_local, function);",
        )
        .expect("missing ordinary-key scan after TypedArray observation")
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn unique_normalized_position(body: &str, snippet: &str, label: &str) -> usize {
    let snippet = without_whitespace(snippet);
    assert_eq!(
        body.matches(snippet.as_str()).count(),
        1,
        "{label} must occur exactly once"
    );
    body.find(snippet.as_str())
        .unwrap_or_else(|| panic!("missing normalized sentinel: {label}"))
}

#[test]
fn standard_dispatch_reaches_the_witnessed_compiler() {
    let dispatch = without_whitespace(STANDARD_SOURCE);
    let edge = without_whitespace(
        r#"
            StandardBuiltinId::ObjectGetOwnPropertyNames => {
                self.compile_object_get_own_property_names_builtin(function)?
            }
        "#,
    );

    assert_eq!(
        dispatch.matches(edge.as_str()).count(),
        1,
        "the standard builtin dispatcher must route Object.getOwnPropertyNames through the witnessed compiler"
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("compile_object_get_own_property_names_builtin(")
            .count(),
        1,
        "the witnessed Object.getOwnPropertyNames compiler must have one standard dispatch edge"
    );
}

#[test]
fn get_own_property_names_uses_one_non_throwing_typed_array_length_witness() {
    let owner = owner_body();
    let observation = typed_array_observation();

    for (needle, expected, role) in [
        (
            "emit_load_typed_array_private_state(",
            1,
            "private-state load",
        ),
        ("TypedArrayViewLocals::new(", 1, "immutable view"),
        ("emit_typed_array_witness(", 1, "buffer witness"),
        (
            "TypedArrayWitnessUse::ArrayLikeLengthSnapshot",
            1,
            "array-like length projection",
        ),
    ] {
        assert_eq!(
            observation.matches(needle).count(),
            expected,
            "Object.getOwnPropertyNames TypedArray observation must have exactly {expected} {role}"
        );
    }

    assert_eq!(
        owner.matches("emit_typed_array_witness(").count(),
        1,
        "Object.getOwnPropertyNames must have only its one TypedArray witness"
    );
    assert_eq!(
        owner
            .matches("Instruction::LocalSet(typed_array_length_local)")
            .count(),
        0,
        "the compiler must not overwrite the witness-produced TypedArray length"
    );

    for forbidden in [
        "emit_typed_array_current_byte_length(",
        "emit_validate_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "emit_load_array_buffer_data(",
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
        "Instruction::I64DivU",
    ] {
        assert!(
            !owner.contains(forbidden),
            "Object.getOwnPropertyNames must not bypass its witness through {forbidden}"
        );
    }

    for forbidden in [
        "emit_throw_runtime_error(",
        "emit_throw_current_function_realm_type_error(",
    ] {
        assert!(
            !observation.contains(forbidden),
            "Object.getOwnPropertyNames must not bypass its non-throwing witness through {forbidden}"
        );
    }

    let normalized = without_whitespace(observation);
    let brand = normalized
        .find("HEAP_OBJECT_INTERNAL_BRAND_OFFSET")
        .expect("missing TypedArray brand load");
    let private_state = unique_normalized_position(
        &normalized,
        PRIVATE_STATE_WIRING,
        "exact private-state wiring",
    );
    let view = unique_normalized_position(&normalized, VIEW_WIRING, "exact immutable-view wiring");
    let witness = unique_normalized_position(
        &normalized,
        WITNESS_WIRING,
        "exact array-like length witness wiring",
    );
    assert!(
        brand < private_state && private_state < view && view < witness,
        "Object.getOwnPropertyNames must brand-check before loading one view and snapshotting its length"
    );

    let proxy = owner
        .find("emit_proxy_own_keys_trap_result(")
        .expect("missing Proxy ownKeys path");
    let typed_array = owner
        .find("LocalSet(typed_array_brand_local)")
        .expect("missing TypedArray branch");
    assert!(
        proxy < typed_array,
        "Proxy ownKeys dispatch must remain before the direct TypedArray observation"
    );

    let release_order = without_whitespace(
        r#"
        self.release_temp_local(ordinary_string_count_local);
        self.release_temp_local(typed_array_length_local);
        self.release_temp_local(typed_array_bytes_per_element_local);
        self.release_temp_local(typed_array_stored_byte_length_local);
        self.release_temp_local(typed_array_byte_offset_local);
        self.release_temp_local(typed_array_buffer_payload_local);
        self.release_temp_local(typed_array_brand_local);
"#,
    );
    assert_eq!(
        without_whitespace(owner).matches(&release_order).count(),
        1,
        "TypedArray own-key locals must retain reverse-order release"
    );
}

#[test]
fn focused_cli_fixture_pins_typed_array_name_snapshot_and_ordinary_keys() {
    let test_body = CLI_TESTS
        .split_once("fn run_wasm_backend_get_own_property_names_uses_typedarray_buffer_witness()")
        .expect("missing focused Object.getOwnPropertyNames TypedArray CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused Object.getOwnPropertyNames CLI test")
        .0;
    assert!(test_body.contains("wasm_object_get_own_property_names_typed_array_witness.js"));
    assert!(test_body.contains("number(951"));

    for marker in [
        "__lilaDetachArrayBuffer(detachedBuffer)",
        "[\"visible\", \"hidden\"]",
        "fixed in bounds",
        "fixed out of bounds",
        "fixed regrown",
        "tracking partial shrink",
        "tracking partial growth",
        "tracking offset out of bounds",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}
