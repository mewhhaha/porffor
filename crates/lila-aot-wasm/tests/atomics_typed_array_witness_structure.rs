const ATOMICS_SOURCE: &str = include_str!("../src/builtins/atomics.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/binary_data.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_atomics_typed_array_buffer_witness.js");

const PRIVATE_STATE_WIRING: &str = r#"
        self.emit_load_typed_array_private_state(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
            function,
        );
"#;

const VIEW_WIRING: &str = r#"
        let typed_array_view = TypedArrayViewLocals::new(
            typed_array_payload_local,
            buffer_payload_local,
            byte_offset_local,
            stored_byte_length_local,
            bytes_per_element_local,
        );
"#;

const WITNESS_WIRING: &str = r#"
        self.emit_typed_array_witness(
            &typed_array_view,
            TypedArrayWitnessUse::ValidatedMethodEntry {
                length_local: element_length_local,
            },
            function,
        )?;
"#;

const ELEMENT_BOUND_WIRING: &str = r#"
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(element_length_local));
        function.instruction(&Instruction::I64GeU);
"#;

fn owner_body(start: &str, end: &str) -> &'static str {
    ATOMICS_SOURCE
        .split_once(start)
        .unwrap_or_else(|| panic!("missing Atomics owner: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing Atomics owner boundary: {end}"))
        .0
}

fn owners() -> [(&'static str, &'static str); 4] {
    [
        (
            "Atomics.notify",
            owner_body(
                "fn emit_atomics_notify(",
                "fn emit_atomics_require_agent_can_suspend(",
            ),
        ),
        (
            "Atomics.waitAsync",
            owner_body(
                "fn emit_atomics_wait_async(&mut self,",
                "fn emit_atomics_wait_async_timeout_checkpoint(",
            ),
        ),
        (
            "Atomics.wait",
            owner_body(
                "fn emit_atomics_wait(&mut self,",
                "fn emit_atomics_integer_operation(",
            ),
        ),
        (
            "Atomics integer operations",
            owner_body(
                "fn emit_atomics_integer_operation(",
                "fn emit_atomics_friendly_element_kind_i32(",
            ),
        ),
    ]
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
fn four_atomic_access_owners_use_one_validated_typed_array_witness() {
    for (label, body) in owners() {
        for (needle, expected, role) in [
            (
                "emit_load_typed_array_private_state(",
                1,
                "private-state load",
            ),
            ("TypedArrayViewLocals::new(", 1, "immutable view"),
            ("emit_typed_array_witness(", 1, "buffer witness"),
            (
                "TypedArrayWitnessUse::ValidatedMethodEntry",
                1,
                "validated method-entry projection",
            ),
            (
                "emit_load_array_buffer_data(buffer_payload_local, data_ptr_local, function)",
                1,
                "pre-coercion backing pointer snapshot",
            ),
            (
                "HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET",
                1,
                "element-kind load",
            ),
        ] {
            assert_eq!(
                body.matches(needle).count(),
                expected,
                "{label} must have exactly {expected} {role}"
            );
        }

        for forbidden in [
            "emit_typed_array_current_byte_length(",
            "emit_validate_typed_array_current_byte_length(",
            "emit_load_array_buffer_byte_length(",
            "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
            "HEAP_TYPED_ARRAY_BYTE_OFFSET",
            "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
            "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
            "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
        ] {
            assert!(
                !body.contains(forbidden),
                "{label} must not bypass its witness through {forbidden}"
            );
        }

        let normalized = without_whitespace(body);
        let private_state = unique_normalized_position(
            &normalized,
            PRIVATE_STATE_WIRING,
            &format!("{label} private-state wiring"),
        );
        let view =
            unique_normalized_position(&normalized, VIEW_WIRING, &format!("{label} view wiring"));
        let element_kind = normalized
            .find("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET")
            .unwrap_or_else(|| panic!("missing {label} element-kind validation"));
        let data_pointer = normalized
            .find("emit_load_array_buffer_data(buffer_payload_local,data_ptr_local,function)")
            .unwrap_or_else(|| panic!("missing {label} backing pointer snapshot"));
        let witness = unique_normalized_position(
            &normalized,
            WITNESS_WIRING,
            &format!("{label} witness wiring"),
        );
        assert!(
            private_state < view
                && view < element_kind
                && element_kind < data_pointer
                && data_pointer < witness,
            "{label} must load one immutable view, retain kind validation and reject detachment before its validated witness"
        );
    }
}

#[test]
fn atomics_bounds_use_the_witness_element_length_after_index_coercion() {
    for (label, body) in owners() {
        assert_eq!(
            body.matches(
                "emit_value_to_number_payload(index_tag_local, index_payload_local, function)"
            )
            .count(),
            1,
            "{label} must coerce the index once"
        );
        assert_eq!(
            body.matches("emit_to_index_from_number_payload(").count(),
            1,
            "{label} must normalize the index once"
        );
        assert_eq!(
            body.matches("Instruction::LocalGet(element_length_local)")
                .count(),
            1,
            "{label} must consume the witness length only in its index bound"
        );
        assert_eq!(
            body.matches("Instruction::LocalSet(element_length_local)")
                .count(),
            0,
            "{label} must not overwrite the witness-produced element length"
        );

        let normalized = without_whitespace(body);
        let witness =
            unique_normalized_position(&normalized, WITNESS_WIRING, &format!("{label} witness"));
        let to_number = normalized
            .find("emit_value_to_number_payload(index_tag_local,index_payload_local,function)")
            .unwrap_or_else(|| panic!("missing {label} index ToNumber"));
        let to_index = normalized
            .find("emit_to_index_from_number_payload(")
            .unwrap_or_else(|| panic!("missing {label} ToIndex"));
        let element_bound = unique_normalized_position(
            &normalized,
            ELEMENT_BOUND_WIRING,
            &format!("{label} element bound"),
        );
        let range_error = normalized
            .find("emit_throw_runtime_error(RANGE_ERROR_NAME,")
            .unwrap_or_else(|| panic!("missing {label} range error"));
        let later_coercion_marker = match label {
            "Atomics.notify" => {
                "emit_value_to_number_payload(count_tag_local,count_payload_local,function)"
            }
            "Atomics.waitAsync" | "Atomics.wait" => "emit_to_bigint_u64_word_from_value_locals(",
            "Atomics integer operations" => {
                "emit_atomics_bigint_element_kind_i32(element_kind_local,function)"
            }
            _ => unreachable!("closed Atomics owner census"),
        };
        let later_coercion = normalized
            .find(later_coercion_marker)
            .unwrap_or_else(|| panic!("missing {label} later argument coercion"));
        assert!(
            witness < to_number
                && to_number < to_index
                && to_index < element_bound
                && element_bound < range_error
                && range_error < later_coercion,
            "{label} must snapshot validated length before index coercion, apply its RangeError bound and only then coerce later arguments"
        );
    }
}

#[test]
fn focused_cli_fixture_pins_atomic_witness_error_and_length_policy() {
    let test_body = CLI_TESTS
        .split_once("fn run_wasm_backend_validates_atomics_access_through_typed_array_witness()")
        .expect("missing focused Atomics witness CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused Atomics witness CLI test")
        .0;
    assert!(test_body.contains("wasm_atomics_typed_array_buffer_witness.js"));
    assert!(test_body.contains("number(936"));

    for marker in [
        "detached index coercions",
        "out-of-bounds index coercions",
        "add snapshots zero length",
        "notify snapshots zero length",
        "wait snapshots zero length",
        "waitAsync snapshots zero length",
        "add floors partial element",
        "notify floors partial element",
        "wait floors partial element",
        "waitAsync floors partial element",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }
}
