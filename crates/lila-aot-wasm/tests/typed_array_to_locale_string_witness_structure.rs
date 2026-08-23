const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

const TYPED_ARRAY_ARM_START: &str =
    "if matches!(receiver_kind, ToLocaleStringReceiverKind::TypedArray) {";
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

const TYPED_RECEIVER_DISABLED_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(typed_receiver_local));
"#;

const TYPED_RECEIVER_ENABLED_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::LocalSet(typed_receiver_local));
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
    self.emit_typed_array_current_byte_length(
        receiver_payload_local,
        receiver_tag_local,
        typed_buffer_payload_local,
        typed_byte_offset_local,
        typed_byte_length_local,
        function,
    )?;
    function.instruction(&Instruction::LocalGet(typed_byte_length_local));
    function.instruction(&Instruction::LocalGet(typed_bytes_per_element_local));
    function.instruction(&Instruction::I64DivU);
    function.instruction(&Instruction::LocalSet(len_local));
"#;

const ARRAYLIKE_ARRAY_LENGTH_WIRING: &str = r#"
    self.load_i64_to_local_from_offset(
        receiver_payload_local,
        HEAP_LEN_OFFSET,
        len_local,
        function,
    );
"#;

const GENERIC_OBJECT_LENGTH_WIRING: &str = r#"
    self.emit_to_length_i64_from_value_locals(
        element_tag_local,
        element_payload_local,
        len_local,
        function,
    )?;
"#;

const LIVE_TYPED_ARRAY_READ_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(typed_receiver_local));
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::I64Ne);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_typed_array_or_object_index_read_from_locals(
        receiver_payload_local,
        receiver_tag_local,
        index_local,
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
    let typed_receiver = unique_normalized_position(
        &normalized,
        TYPED_RECEIVER_ENABLED_WIRING,
        "direct TypedArray live-read routing",
    );
    let private_state =
        unique_normalized_position(&normalized, PRIVATE_STATE_WIRING, "private-state load");
    let view = unique_normalized_position(&normalized, VIEW_WIRING, "immutable view");
    let witness = unique_normalized_position(&normalized, WITNESS_WIRING, "method-entry witness");
    assert!(
        brand < typed_receiver
            && typed_receiver < private_state
            && private_state < view
            && view < witness,
        "brand validation, live-read routing, private view construction and witness consumption must retain specification order"
    );
}

#[test]
fn generic_arraylike_policy_and_shared_live_loop_remain_distinct() {
    let shared = shared_body();
    let generic = arraylike_arm();
    let normalized_shared = without_whitespace(shared);
    let normalized_generic = without_whitespace(generic);

    assert_eq!(
        shared.matches("receiver_kind").count(),
        4,
        "the closed entry policy must have one binding and only the method-name, direct-arm and invocation consumers"
    );
    assert_eq!(
        normalized_shared
            .matches("letmethod_name=receiver_kind.method_name();")
            .count(),
        1
    );
    assert_eq!(
        normalized_shared
            .matches("ifmatches!(receiver_kind,ToLocaleStringReceiverKind::TypedArray){")
            .count(),
        1
    );
    assert_eq!(
        normalized_shared
            .matches("self.emit_validate_to_locale_string_invocation(receiver_kind,")
            .count(),
        1
    );
    let typed_receiver_disabled_wiring = without_whitespace(TYPED_RECEIVER_DISABLED_WIRING);
    let typed_receiver_enabled_wiring = without_whitespace(TYPED_RECEIVER_ENABLED_WIRING);

    assert_eq!(shared.matches("len_local").count(), 8);
    assert_eq!(
        shared.matches("Instruction::LocalSet(len_local)").count(),
        2,
        "only initialization and the preserved generic TypedArray projection may directly write len_local"
    );
    assert_eq!(shared.matches("typed_receiver_local").count(), 6);
    assert_eq!(
        shared
            .matches("Instruction::LocalSet(typed_receiver_local)")
            .count(),
        3,
        "typed_receiver_local must have one zero initialization and one true writer in each entry arm"
    );
    assert_eq!(
        shared
            .matches("Instruction::LocalGet(typed_receiver_local)")
            .count(),
        1,
        "the shared live indexed-read dispatch must be the sole consumer"
    );
    assert_eq!(
        normalized_shared
            .matches(typed_receiver_disabled_wiring.as_str())
            .count(),
        1
    );
    assert_eq!(
        normalized_shared
            .matches(typed_receiver_enabled_wiring.as_str())
            .count(),
        2
    );

    assert_eq!(
        shared
            .matches("emit_validate_typed_array_current_byte_length(")
            .count(),
        0,
        "the shared compiler must contain no legacy throwing validator"
    );
    assert_eq!(
        shared
            .matches("emit_load_typed_array_private_state(")
            .count(),
        2,
        "the statically distinct direct and generic TypedArray arms each load one private record"
    );
    assert_eq!(shared.matches("TypedArrayViewLocals::new(").count(), 1);
    assert_eq!(shared.matches("emit_typed_array_witness(").count(), 1);
    assert_eq!(
        shared
            .matches("emit_typed_array_current_byte_length(")
            .count(),
        1,
        "only the preserved generic ArrayLike arm may retain the non-throwing raw observation"
    );
    assert_eq!(
        generic
            .matches("emit_load_typed_array_private_state(")
            .count(),
        1
    );
    assert_eq!(
        generic
            .matches("emit_typed_array_current_byte_length(")
            .count(),
        1
    );
    assert_eq!(generic.matches("Instruction::I64DivU").count(), 1);
    assert!(!generic.contains("TypedArrayViewLocals::new("));
    assert!(!generic.contains("emit_typed_array_witness("));
    assert!(!generic.contains("TypedArrayWitnessUse::ValidatedMethodEntry"));

    let generic_check = normalized_generic
        .find("self.emit_is_typed_array_i32(receiver_payload_local,receiver_tag_local,function);")
        .expect("generic TypedArray detection");
    let generic_typed_receiver_in_arm = unique_normalized_position(
        &normalized_generic,
        TYPED_RECEIVER_ENABLED_WIRING,
        "generic TypedArray live-read routing",
    );
    let generic_private = unique_normalized_position(
        &normalized_generic,
        PRIVATE_STATE_WIRING,
        "generic private-state load",
    );
    let generic_length_in_arm = unique_normalized_position(
        &normalized_generic,
        GENERIC_LENGTH_WIRING,
        "generic non-throwing length observation",
    );
    assert!(
        generic_check < generic_typed_receiver_in_arm
            && generic_typed_receiver_in_arm < generic_private
            && generic_private < generic_length_in_arm,
        "generic TypedArray detection and live-read routing must precede its distinct non-throwing length snapshot"
    );

    let initial_length = unique_normalized_position(
        &normalized_shared,
        INITIAL_LENGTH_WIRING,
        "length initialization",
    );
    let typed_receiver_disabled = unique_normalized_position(
        &normalized_shared,
        TYPED_RECEIVER_DISABLED_WIRING,
        "TypedArray live-read routing initialization",
    );
    let direct_typed_receiver_write = normalized_shared
        .find(typed_receiver_enabled_wiring.as_str())
        .expect("direct TypedArray live-read routing");
    let generic_typed_receiver_write = normalized_shared
        .rfind(typed_receiver_enabled_wiring.as_str())
        .expect("generic TypedArray live-read routing");
    assert_ne!(direct_typed_receiver_write, generic_typed_receiver_write);
    let witness = unique_normalized_position(
        &normalized_shared,
        WITNESS_WIRING,
        "direct method-entry witness",
    );
    let array_length = unique_normalized_position(
        &normalized_shared,
        ARRAYLIKE_ARRAY_LENGTH_WIRING,
        "generic Array/arguments length",
    );
    let generic_length = unique_normalized_position(
        &normalized_shared,
        GENERIC_LENGTH_WIRING,
        "generic TypedArray length",
    );
    let object_length = unique_normalized_position(
        &normalized_shared,
        GENERIC_OBJECT_LENGTH_WIRING,
        "generic object ToLength",
    );
    let loop_bound =
        unique_normalized_position(&normalized_shared, LOOP_START, "captured-length loop bound");
    let live_read = unique_normalized_position(
        &normalized_shared,
        LIVE_TYPED_ARRAY_READ_WIRING,
        "live TypedArray indexed read",
    );
    let validate_invocation = normalized_shared
        .find("self.emit_validate_to_locale_string_invocation(")
        .expect("validated element invocation");
    let call_invocation = normalized_shared
        .find("self.emit_call_validated_to_locale_string_invocation(")
        .expect("validated element call");
    assert!(
        initial_length < typed_receiver_disabled
            && typed_receiver_disabled < direct_typed_receiver_write
            && direct_typed_receiver_write < witness
            && witness < array_length
            && array_length < generic_typed_receiver_write
            && generic_typed_receiver_write < generic_length
            && generic_length < object_length
            && object_length < loop_bound
            && loop_bound < live_read
            && live_read < validate_invocation
            && validate_invocation < call_invocation,
        "entry validation, captured loop bound, live read and validated element invocation must retain specification order"
    );

    let loop_tail = shared.split_once(LOOP_START).expect("shared loop start").1;
    for forbidden in [
        "emit_load_typed_array_private_state(",
        "emit_typed_array_witness(",
        "emit_validate_typed_array_current_byte_length(",
        "emit_typed_array_current_byte_length(",
        "Instruction::LocalSet(len_local)",
    ] {
        assert!(
            !loop_tail.contains(forbidden),
            "the shared loop must not re-observe or replace its captured length through {forbidden}"
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
