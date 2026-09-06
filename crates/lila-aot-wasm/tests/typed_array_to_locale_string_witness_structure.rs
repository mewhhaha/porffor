const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_array_to_locale_string_core.js");

const ODD_BYTE_FIXTURE_WIRING: &str = r#"
let oddRab = new ArrayBuffer(6, { maxByteLength: 8 });
let oddTracking = new Uint16Array(oddRab);
oddTracking[0] = 7;
oddTracking[1] = 9;
oddTracking[2] = 11;
oddRab.resize(5);
if (Array.prototype.toLocaleString.call(oddTracking) !== "7" + separator + "9") {
  failures |= 65536;
}
"#;

const DETACHED_FIXTURE_WIRING: &str = r#"
let detachedBuffer = new ArrayBuffer(4);
let detached = new Uint8Array(detachedBuffer);
detachedBuffer.transfer();
if (Array.prototype.toLocaleString.call(detached) !== "") failures |= 131072;
"#;

const FINAL_FIXTURE_PUBLICATION: &str = "failures === 0;";

const TYPED_ARRAY_ARM_START: &str = "if typed_array_entry {";
const ARRAYLIKE_ARM_START: &str = r#"
        } else {
            self.compile_nullish_tagged_i32(receiver_tag_local, function)?;
"#;
const LOOP_START: &str = r#"
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
"#;

const INITIAL_LENGTH_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(len_local));
"#;

const BRAND_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(typed_brand_local));
    function.instruction(&Instruction::I64Const(
        OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
    ));
    function.instruction(&Instruction::I64Ne);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_throw_current_function_realm_type_error(
        "TypedArray.prototype.toLocaleString requires TypedArray",
        self.result_local,
        self.result_tag_local,
        function,
    )?;
    self.emit_return_current_completion(function);
    function.instruction(&Instruction::End);
"#;

const PRIVATE_STATE_WIRING: &str = r#"
    self.emit_load_typed_array_private_state(
        receiver_payload_local,
        typed_buffer_payload_local,
        typed_byte_offset_local,
        typed_byte_length_local,
        typed_bytes_per_element_local,
        function,
    );
"#;

const VIEW_WIRING: &str = r#"
    let typed_view = TypedArrayViewLocals::new(
        receiver_payload_local,
        typed_buffer_payload_local,
        typed_byte_offset_local,
        typed_byte_length_local,
        typed_bytes_per_element_local,
    );
"#;

const WITNESS_WIRING: &str = r#"
    self.emit_typed_array_witness(
        &typed_view,
        TypedArrayWitnessUse::ValidatedMethodEntry {
            length_local: len_local,
        },
        function,
    )?;
"#;

const GENERIC_LENGTH_WIRING: &str = r#"
    self.emit_array_like_length_snapshot(
        receiver_payload_local,
        receiver_tag_local,
        key_local,
        len_local,
        element_tag_local,
        function,
    )?;
    self.emit_return_current_completion_if_throw(function);
"#;

const LIVE_TYPED_ARRAY_READ_WIRING: &str = r#"
    self.emit_typed_array_or_object_index_read_from_locals(
        receiver_payload_local,
        receiver_tag_local,
        index_local,
        element_payload_local,
        element_tag_local,
        function,
    )?;
    self.emit_propagate_throw_from_locals_if_needed(
        element_payload_local,
        element_tag_local,
        function,
    )?;
"#;

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn shared_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "fn compile_to_locale_string_builtin(",
        "pub(crate) fn emit_object_has_array_index_key_in_range_i32(",
    )
}

fn array_entry_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_array_prototype_to_locale_string_builtin(",
        "pub(crate) fn compile_typed_array_prototype_to_locale_string_builtin(",
    )
}

fn typed_array_entry_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_typed_array_prototype_to_locale_string_builtin(",
        "fn emit_validate_to_locale_string_invocation(",
    )
}

fn focused_cli_test_body() -> &'static str {
    bounded(
        CLI_TESTS,
        "fn run_wasm_backend_succeeds_for_supported_array_to_locale_string_fixture() {",
        "\n#[test]\n",
    )
}

fn typed_array_arm() -> &'static str {
    bounded(shared_body(), TYPED_ARRAY_ARM_START, ARRAYLIKE_ARM_START)
}

fn arraylike_arm() -> &'static str {
    bounded(shared_body(), ARRAYLIKE_ARM_START, LOOP_START)
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn local_sequence<'a>(body: &'a str, prefix: &str, suffix: &str) -> Vec<&'a str> {
    body.lines()
        .filter_map(|line| line.trim().strip_prefix(prefix)?.strip_suffix(suffix))
        .collect()
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
fn direct_typed_array_entry_uses_one_validated_method_witness() {
    let arm = typed_array_arm();
    let normalized = without_whitespace(arm);

    assert_eq!(
        arm.matches("emit_load_typed_array_private_state(").count(),
        1
    );
    assert_eq!(arm.matches("TypedArrayViewLocals::new(").count(), 1);
    assert_eq!(arm.matches("emit_typed_array_witness(").count(), 1);
    assert_eq!(
        arm.matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        1
    );
    assert_eq!(arm.matches("typed_view").count(), 2);

    for forbidden in [
        "emit_validate_typed_array_current_byte_length(",
        "emit_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "emit_load_array_buffer_data(",
        "HEAP_ARRAY_BUFFER_",
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
        "Instruction::I64DivU",
        "Instruction::LocalSet(len_local)",
        "emit_throw_runtime_error(",
        "TYPE_ERROR_NAME",
        "typed_buffer_tag_local",
    ] {
        assert!(
            !arm.contains(forbidden),
            "the direct TypedArray arm must not bypass its witness through {forbidden}"
        );
    }

    let brand = unique_normalized_position(&normalized, BRAND_WIRING, "receiver-brand guard");
    let private_state =
        unique_normalized_position(&normalized, PRIVATE_STATE_WIRING, "private-state load");
    let view = unique_normalized_position(&normalized, VIEW_WIRING, "immutable view");
    let witness = unique_normalized_position(&normalized, WITNESS_WIRING, "method-entry witness");
    assert!(
        brand < private_state && private_state < view && view < witness,
        "brand validation, private view construction and witness consumption must retain specification order"
    );
}

#[test]
fn generic_arraylike_policy_and_shared_live_loop_remain_distinct() {
    let shared = shared_body();
    let generic = arraylike_arm();
    let normalized = without_whitespace(shared);
    let normalized_generic = without_whitespace(generic);
    assert_eq!(shared.matches("receiver_kind").count(), 4);
    for consumer in [
        "letmethod_name=match&receiver_kind{",
        "lettyped_array_entry=match&receiver_kind{",
        "iftyped_array_entry{",
        "self.emit_validate_to_locale_string_invocation(&receiver_kind,",
    ] {
        assert_eq!(normalized.matches(consumer).count(), 1, "{consumer}");
    }
    assert_eq!(
        shared.matches("Instruction::LocalSet(len_local)").count(),
        1
    );
    assert_eq!(
        shared
            .matches("self.emit_array_like_length_snapshot(")
            .count(),
        1
    );
    assert_eq!(
        shared
            .matches("self.emit_typed_array_or_object_index_read_from_locals(")
            .count(),
        1
    );
    for forbidden in [
        "typed_receiver_local",
        "emit_arguments_read(",
        "emit_array_index_get_with_prototype(",
        "TypedArrayWitnessUse::ArrayLikeLengthSnapshot",
        "emit_typed_array_current_byte_length(",
        "emit_validate_typed_array_current_byte_length(",
    ] {
        assert!(
            !shared.contains(forbidden),
            "retired private bypass: {forbidden}"
        );
    }
    for operation in [
        "emit_load_typed_array_private_state(",
        "TypedArrayViewLocals::new(",
        "emit_typed_array_witness(",
        "TypedArrayWitnessUse::ValidatedMethodEntry",
    ] {
        assert_eq!(
            shared.matches(operation).count(),
            1,
            "direct entry owns {operation}"
        );
        assert!(
            !generic.contains(operation),
            "generic length must not use {operation}"
        );
    }
    for forbidden in [
        "emit_load_array_buffer_byte_length(",
        "emit_load_array_buffer_data(",
        "HEAP_TYPED_ARRAY_",
        "HEAP_LEN_OFFSET",
        "Instruction::I64DivU",
        "Instruction::LocalSet(len_local)",
        "emit_array_iteration_to_object(",
        "emit_object_read(",
        "emit_to_length_i64_from_value_locals(",
    ] {
        assert!(
            !generic.contains(forbidden),
            "generic length must delegate, not use {forbidden}"
        );
    }
    unique_normalized_position(
        &normalized_generic,
        GENERIC_LENGTH_WIRING,
        "one observable LengthOfArrayLike",
    );
    let initial =
        unique_normalized_position(&normalized, INITIAL_LENGTH_WIRING, "length initialization");
    let direct =
        unique_normalized_position(&normalized, WITNESS_WIRING, "direct validation and length");
    let generic = unique_normalized_position(
        &normalized,
        GENERIC_LENGTH_WIRING,
        "generic observable length",
    );
    let bound = unique_normalized_position(&normalized, LOOP_START, "captured-length loop bound");
    let read = unique_normalized_position(
        &normalized,
        LIVE_TYPED_ARRAY_READ_WIRING,
        "shared live indexed Get and abrupt completion",
    );
    let nullish = normalized
        .find("self.compile_nullish_tagged_i32(element_tag_local,function)")
        .unwrap();
    let validate = normalized
        .find("self.emit_validate_to_locale_string_invocation(")
        .unwrap();
    let call = normalized
        .find("self.emit_call_validated_to_locale_string_invocation(")
        .unwrap();
    assert!(
        initial < direct
            && direct < generic
            && generic < bound
            && bound < read
            && read < nullish
            && nullish < validate
            && validate < call
    );
    let loop_tail = shared.split_once(LOOP_START).expect("shared loop start").1;
    for forbidden in [
        "emit_load_typed_array_private_state(",
        "emit_typed_array_witness(",
        "emit_array_like_length_snapshot(",
        "Instruction::LocalSet(len_local)",
    ] {
        assert!(
            !loop_tail.contains(forbidden),
            "loop must not replace its bound via {forbidden}"
        );
    }
}

#[test]
fn shared_length_owner_boxes_gets_propagates_and_normalizes_in_order() {
    let helper = bounded(
        ARRAY_SOURCE,
        "pub(crate) fn emit_array_like_length_snapshot(",
        "pub(crate) fn compile_array_prototype_map_builtin(",
    );
    let normalized = without_whitespace(helper);
    let mut previous = 0;
    for (index, operation) in [
        "self.emit_value_to_current_function_realm_object_locals(receiver_payload_local,receiver_tag_local,receiver_payload_local,receiver_tag_local,function,)?;",
        "Instruction::I64Const(self.strings.payload(\"length\"))",
        "self.emit_object_read(receiver_payload_local,receiver_tag_local,receiver_payload_local,receiver_tag_local,key_local,length_payload_local,length_tag_local,function,)?;",
        "self.emit_propagate_throw_from_locals_if_needed(length_payload_local,length_tag_local,function,)?;",
        "self.emit_to_length_i64_from_value_locals(length_tag_local,length_payload_local,length_payload_local,function,)?;",
    ].iter().enumerate() {
        let position = unique_normalized_position(&normalized, operation, "shared length operation");
        if index > 0 { assert!(previous < position); }
        previous = position;
    }
    for forbidden in ["HEAP_", "TypedArrayWitnessUse", "Instruction::I64TruncF64U"] {
        assert!(
            !helper.contains(forbidden),
            "observable length cannot use {forbidden}"
        );
    }
}

#[test]
fn entry_dispatch_and_shared_temporary_lifecycle_are_exact() {
    let array_entry = array_entry_body();
    assert_eq!(
        array_entry
            .matches("compile_to_locale_string_builtin(ToLocaleStringReceiverKind::ArrayLike")
            .count(),
        1
    );
    assert_eq!(
        array_entry
            .matches("compile_to_locale_string_builtin(")
            .count(),
        1
    );
    assert!(!array_entry.contains("ToLocaleStringReceiverKind::TypedArray"));

    let typed_array_entry = typed_array_entry_body();
    assert_eq!(
        typed_array_entry
            .matches("compile_to_locale_string_builtin(ToLocaleStringReceiverKind::TypedArray")
            .count(),
        1
    );
    assert_eq!(
        typed_array_entry
            .matches("compile_to_locale_string_builtin(")
            .count(),
        1
    );
    assert!(!typed_array_entry.contains("ToLocaleStringReceiverKind::ArrayLike"));

    let normalized_standard = without_whitespace(STANDARD_SOURCE);
    for (builtin, mapping) in [
        (
            "StandardBuiltinId::ArrayPrototypeToLocaleString",
            "StandardBuiltinId::ArrayPrototypeToLocaleString=>{self.compile_array_prototype_to_locale_string_builtin(function)?;}",
        ),
        (
            "StandardBuiltinId::TypedArrayPrototypeToLocaleString",
            "StandardBuiltinId::TypedArrayPrototypeToLocaleString=>{self.compile_typed_array_prototype_to_locale_string_builtin(function)?;}",
        ),
    ] {
        assert_eq!(
            STANDARD_SOURCE.matches(builtin).count(),
            1,
            "dispatcher builtin must have exactly one owner: {builtin}"
        );
        assert_eq!(
            normalized_standard.matches(mapping).count(),
            1,
            "dispatcher mapping must occur exactly once: {mapping}"
        );
    }

    let body = shared_body();
    let normalized_body = without_whitespace(body);
    assert!(!body.contains("typed_buffer_tag_local"));
    let reservations = local_sequence(body, "let ", " = self.reserve_temp_local();");
    let releases = local_sequence(body, "self.release_temp_local(", ");");
    assert_eq!(
        body.matches("reserve_temp_local()").count(),
        reservations.len(),
        "every temporary reservation must use the reviewed local-binding shape"
    );
    assert_eq!(
        body.matches("release_temp_local(").count(),
        releases.len(),
        "every temporary release must use the reviewed local-release shape"
    );
    let mut unique_reservations = reservations.clone();
    unique_reservations.sort_unstable();
    unique_reservations.dedup();
    assert_eq!(
        unique_reservations.len(),
        reservations.len(),
        "each temporary local must be reserved exactly once"
    );
    let mut expected_releases = reservations.clone();
    expected_releases.reverse();
    assert_eq!(
        releases, expected_releases,
        "temporary locals must be released exactly once in reverse reservation order"
    );

    let result_publication = unique_normalized_position(
        &normalized_body,
        r#"
        function.instruction(&Instruction::LocalGet(joined_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        "#,
        "final joined-string publication",
    );
    let first_release = normalized_body
        .find("self.release_temp_local(")
        .expect("shared compiler temporary release block");
    assert!(
        result_publication < first_release,
        "all compiler temporaries must remain live through final result publication"
    );
}

#[test]
fn focused_cli_fixture_covers_non_throwing_generic_typed_array_snapshots() {
    let cli_test = focused_cli_test_body();
    assert_eq!(
        cli_test
            .matches("wasm_array_to_locale_string_core.js")
            .count(),
        1,
        "the focused CLI test must execute the exact core fixture once"
    );
    assert!(cli_test.contains("backend_used: WasmAot"));
    assert!(cli_test.contains("boolean(true)"));

    for preserved_control in [
        "Array.prototype.toLocaleString.call(tracking)",
        "Array.prototype.toLocaleString.call(fixed) !== \"\"",
    ] {
        assert!(
            CLI_FIXTURE.contains(preserved_control),
            "the focused CLI fixture must retain {preserved_control}"
        );
    }

    let normalized_fixture = without_whitespace(CLI_FIXTURE);
    let odd_scenario = unique_normalized_position(
        &normalized_fixture,
        ODD_BYTE_FIXTURE_WIRING,
        "coupled odd-byte scenario",
    );
    let detached_scenario = unique_normalized_position(
        &normalized_fixture,
        DETACHED_FIXTURE_WIRING,
        "coupled detached scenario",
    );
    let final_publication = unique_normalized_position(
        &normalized_fixture,
        FINAL_FIXTURE_PUBLICATION,
        "final zero-failure publication",
    );

    let mut load_bearing_positions = Vec::new();
    for (snippet, label) in [
        (
            "let oddRab = new ArrayBuffer(6, { maxByteLength: 8 });",
            "odd-byte buffer setup",
        ),
        (
            "let oddTracking = new Uint16Array(oddRab);",
            "odd-byte tracking view setup",
        ),
        ("oddRab.resize(5);", "odd-byte resize"),
        (
            "Array.prototype.toLocaleString.call(oddTracking) !== \"7\" + separator + \"9\"",
            "odd-byte assertion",
        ),
        ("failures |= 65536;", "odd-byte failure bit"),
        (
            "let detachedBuffer = new ArrayBuffer(4);",
            "detached buffer setup",
        ),
        (
            "let detached = new Uint8Array(detachedBuffer);",
            "detached view setup",
        ),
        ("detachedBuffer.transfer();", "detachment"),
        (
            "Array.prototype.toLocaleString.call(detached) !== \"\"",
            "detached assertion",
        ),
        ("failures |= 131072;", "detached failure bit"),
        (FINAL_FIXTURE_PUBLICATION, "final zero-failure publication"),
    ] {
        load_bearing_positions.push(unique_normalized_position(
            &normalized_fixture,
            snippet,
            label,
        ));
    }

    assert!(
        load_bearing_positions.windows(2).all(|pair| pair[0] < pair[1]),
        "odd-byte setup, resize, assertion and failure bit must precede detached setup, transfer, assertion, failure bit and the final publication"
    );
    assert!(
        odd_scenario < detached_scenario && detached_scenario < final_publication,
        "the coupled odd-byte and detached scenarios must precede the sole final publication"
    );
}

#[test]
fn ordinary_get_distinguishes_arguments_length_descriptors_from_array_storage() {
    let body = bounded(
        include_str!("../src/objects.rs"),
        "pub(crate) fn emit_object_read_ordinary_inner(",
        "// Array-like exotic elements and named properties live in",
    );
    let descriptor = unique_normalized_position(
        body,
        "HEAP_ARGUMENTS_LENGTH_DESCRIPTOR_KIND_OFFSET",
        "arguments own length descriptor",
    );
    let present = unique_normalized_position(
        body,
        "HEAP_ARGUMENTS_LENGTH_VALUE_OFFSET",
        "arguments own data value",
    );
    assert!(descriptor < present);
    for field in [
        "HEAP_ARGUMENTS_LENGTH_VALUE_TAG_OFFSET",
        "HEAP_ARGUMENTS_LENGTH_GETTER_PAYLOAD_OFFSET",
        "HEAP_ARGUMENTS_LENGTH_GETTER_TAG_OFFSET",
    ] {
        assert!(
            body.contains(field),
            "missing arguments length field: {field}"
        );
    }
    assert!(without_whitespace(body).contains(&without_whitespace(
        r#"
        self.emit_function_or_proxy_call_leave_throw_completion(
            getter_payload_local,
            getter_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            &[],
            payload_local,
            tag_local,
            function,
        )?;
        "#
    )));
    assert!(body.contains("self.emit_load_prototype_to_current_locals("));
}
