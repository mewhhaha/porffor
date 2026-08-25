const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/typed_array.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_typedarray_subarray_buffer_witness.js");

const PRIVATE_STATE_WIRING: &str = r#"
                self.emit_load_typed_array_private_state(
                    receiver_payload_local,
                    buffer_payload_local,
                    byte_offset_local,
                    stored_byte_length_local,
                    bytes_per_element_local,
                    function,
                );
"#;

const VIEW_WIRING: &str = r#"
                let typed_array_view = TypedArrayViewLocals::new(
                    receiver_payload_local,
                    buffer_payload_local,
                    byte_offset_local,
                    stored_byte_length_local,
                    bytes_per_element_local,
                );
"#;

const WITNESS_WIRING: &str = r#"
                self.emit_typed_array_witness(
                    &typed_array_view,
                    TypedArrayWitnessUse::ArrayLikeLengthSnapshot { length_local },
                    function,
                )?;
"#;

const RESULT_PRIVATE_STATE_WIRING: &str = r#"
                self.emit_load_typed_array_private_state(
                    self.result_local,
                    result_buffer_payload_local,
                    result_byte_offset_local,
                    result_stored_byte_length_local,
                    result_bytes_per_element_local,
                    function,
                );
"#;

const RESULT_VIEW_WIRING: &str = r#"
                let result_typed_array_view = TypedArrayViewLocals::new(
                    self.result_local,
                    result_buffer_payload_local,
                    result_byte_offset_local,
                    result_stored_byte_length_local,
                    result_bytes_per_element_local,
                );
"#;

const RESULT_WITNESS_WIRING: &str = r#"
                self.emit_typed_array_witness(
                    &result_typed_array_view,
                    TypedArrayWitnessUse::ValidatedMethodEntry {
                        length_local: result_length_local,
                    },
                    function,
                )?;
"#;

const SPECIES_ARGUMENT_LISTS: &str = r#"
                function.instruction(&Instruction::LocalGet(length_tracking_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(end_tag_local));
                function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.emit_pre_evaluated_arg_vector(
                    &[
                        (buffer_payload_local, buffer_tag_local),
                        (new_byte_offset_payload_local, number_tag_local),
                    ],
                    argc_local,
                    argv_local,
                    function,
                )?;
                function.instruction(&Instruction::Else);
                self.emit_pre_evaluated_arg_vector(
                    &[
                        (buffer_payload_local, buffer_tag_local),
                        (new_byte_offset_payload_local, number_tag_local),
                        (new_length_payload_local, number_tag_local),
                    ],
                    argc_local,
                    argv_local,
                    function,
                )?;
                function.instruction(&Instruction::End);
"#;

const ARGUMENTS_VECTOR_LENGTH_LOAD: &str = r#"
        self.load_i64_to_local_from_offset(
            self.argv_param_local(),
            HEAP_LEN_OFFSET,
            len_local,
            function,
        );
"#;

fn subarray_arm() -> &'static str {
    STANDARD_SOURCE
        .split_once("StandardBuiltinId::TypedArrayPrototypeSubarray => {")
        .expect("missing TypedArray.prototype.subarray builtin arm")
        .1
        .split_once("StandardBuiltinId::DateNow => {")
        .expect("missing TypedArray.prototype.subarray builtin boundary")
        .0
}

fn arguments_object_payload() -> &'static str {
    FUNCTIONS_SOURCE
        .split_once("pub(crate) fn emit_arguments_object_payload(")
        .expect("missing arguments-object payload emitter")
        .1
        .split_once("pub(crate) fn emit_arguments_length(")
        .expect("missing arguments-object payload emitter boundary")
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
fn subarray_separates_the_source_snapshot_from_constructed_result_validation() {
    let arm = subarray_arm();

    for (needle, expected, role) in [
        (
            "emit_load_typed_array_private_state(",
            2,
            "source and result private-state loads",
        ),
        ("TypedArrayViewLocals::new(", 2, "immutable views"),
        ("emit_typed_array_witness(", 2, "buffer witnesses"),
        (
            "TypedArrayWitnessUse::ArrayLikeLengthSnapshot",
            1,
            "non-throwing length projection",
        ),
        (
            "TypedArrayWitnessUse::ValidatedMethodEntry",
            1,
            "constructed-result validation projection",
        ),
        (
            "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
            1,
            "result arity metadata load",
        ),
        (
            "HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET",
            2,
            "source and result element-kind loads",
        ),
    ] {
        assert_eq!(
            arm.matches(needle).count(),
            expected,
            "TypedArray.prototype.subarray must have exactly {expected} {role}"
        );
    }

    for forbidden in [
        "emit_typed_array_current_byte_length(",
        "emit_validate_typed_array_current_byte_length(",
        "emit_load_array_buffer_byte_length(",
        "emit_load_array_buffer_data(",
        "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_OFFSET",
        "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
        "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
        "TypedArrayWitnessUse::IntegerIndexedProperty",
        "TypedArrayWitnessUse::Accessor",
        "Instruction::I64DivU",
        "Instruction::LocalSet(length_local)",
        "Instruction::LocalSet(buffer_payload_local)",
        "Instruction::LocalSet(byte_offset_local)",
        "Instruction::LocalSet(stored_byte_length_local)",
        "Instruction::LocalSet(bytes_per_element_local)",
        "Instruction::LocalSet(result_buffer_payload_local)",
        "Instruction::LocalSet(result_byte_offset_local)",
        "Instruction::LocalSet(result_stored_byte_length_local)",
        "Instruction::LocalSet(result_bytes_per_element_local)",
        "Instruction::LocalSet(result_length_local)",
    ] {
        assert!(
            !arm.contains(forbidden),
            "TypedArray.prototype.subarray must not bypass or overwrite its witness through {forbidden}"
        );
    }

    let normalized = without_whitespace(arm);
    let brand_error = normalized
        .find("TypedArray.prototype.subarrayrequiresTypedArray")
        .expect("missing receiver-brand error");
    let private_state = unique_normalized_position(
        &normalized,
        PRIVATE_STATE_WIRING,
        "exact private-state wiring",
    );
    let view = unique_normalized_position(&normalized, VIEW_WIRING, "exact immutable-view wiring");
    let source_element_kind = normalized
        .find("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,element_kind_local")
        .expect("missing source element-kind load");
    let tracking = normalized
        .find("HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET,length_tracking_local")
        .expect("missing length-tracking metadata load");
    let witness = unique_normalized_position(
        &normalized,
        WITNESS_WIRING,
        "exact array-like length witness wiring",
    );
    let begin = normalized
        .find("emit_builtin_arg_to_locals(0,begin_payload_local,begin_tag_local,function)")
        .expect("missing begin argument load");
    let end = normalized
        .find("emit_builtin_arg_to_locals(1,end_payload_local,end_tag_local,function)")
        .expect("missing end argument load");
    let species = normalized
        .find("property_key_symbol_payload(\"Symbol.species\")")
        .expect("missing species lookup");
    let species_argument_lists = unique_normalized_position(
        &normalized,
        SPECIES_ARGUMENT_LISTS,
        "exclusive species argument lists",
    );
    let construct = normalized
        .find("emit_function_handle_construct_with_argv(")
        .expect("missing species construction");
    let result_brand_error = normalized
        .find("TypedArray.prototype.subarrayspeciesdidnotreturnaTypedArray")
        .expect("missing constructed-result brand error");
    let result_private_state = unique_normalized_position(
        &normalized,
        RESULT_PRIVATE_STATE_WIRING,
        "exact constructed-result private-state wiring",
    );
    let result_view = unique_normalized_position(
        &normalized,
        RESULT_VIEW_WIRING,
        "exact constructed-result immutable-view wiring",
    );
    let result_witness = unique_normalized_position(
        &normalized,
        RESULT_WITNESS_WIRING,
        "exact constructed-result validating witness wiring",
    );
    let result_element_kind = normalized
        .find("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET,result_element_kind_local")
        .expect("missing result element-kind load");

    assert!(
        brand_error < private_state
            && private_state < view
            && view < source_element_kind
            && source_element_kind < tracking
            && tracking < witness
            && witness < begin
            && begin < end
            && end < species
            && species < species_argument_lists
            && species_argument_lists < construct
            && construct < result_brand_error
            && result_brand_error < result_private_state
            && result_private_state < result_view
            && result_view < result_witness
            && result_witness < result_element_kind,
        "subarray must snapshot the source before coercion, then validate the constructed result before content-type acceptance"
    );

    let result_release_order = without_whitespace(
        r#"
                self.release_temp_local(result_element_kind_local);
                self.release_temp_local(result_length_local);
                self.release_temp_local(result_bytes_per_element_local);
                self.release_temp_local(result_stored_byte_length_local);
                self.release_temp_local(result_byte_offset_local);
                self.release_temp_local(result_buffer_payload_local);
                self.release_temp_local(result_brand_local);
"#,
    );
    assert_eq!(
        normalized.matches(&result_release_order).count(),
        1,
        "constructed-result view locals must retain reverse-order release"
    );

    let release_order = without_whitespace(
        r#"
                self.release_temp_local(length_local);
                self.release_temp_local(length_tracking_local);
                self.release_temp_local(element_kind_local);
                self.release_temp_local(bytes_per_element_local);
                self.release_temp_local(stored_byte_length_local);
                self.release_temp_local(byte_offset_local);
                self.release_temp_local(buffer_tag_local);
                self.release_temp_local(buffer_payload_local);
                self.release_temp_local(typed_array_brand_local);
"#,
    );
    assert_eq!(
        normalized.matches(&release_order).count(),
        1,
        "subarray view locals must retain reverse-order release"
    );
}

#[test]
fn subarray_species_arguments_keep_call_count_and_vector_length_coherent() {
    let arguments_object = arguments_object_payload();
    assert_eq!(
        arguments_object
            .matches(ARGUMENTS_VECTOR_LENGTH_LOAD)
            .count(),
        1,
        "arguments-object construction must derive its observable length from the call vector header"
    );

    let arm = subarray_arm();
    assert_eq!(
        arm.matches("emit_pre_evaluated_arg_vector(").count(),
        2,
        "the two- and three-entry species lists must each have one vector producer"
    );
    assert_eq!(
        arm.matches("emit_function_handle_construct_with_argv(")
            .count(),
        1,
        "both species argument-list branches must converge on one construct"
    );
    assert!(
        !arm.contains("Instruction::LocalSet(argc_local)"),
        "subarray must not change argc independently of the vector header"
    );

    let normalized = without_whitespace(arm);
    let argument_lists = unique_normalized_position(
        &normalized,
        SPECIES_ARGUMENT_LISTS,
        "exclusive species argument lists",
    );
    let construct = normalized
        .find("emit_function_handle_construct_with_argv(")
        .expect("missing species construction");
    assert!(
        argument_lists < construct,
        "both coherent argument-list producers must precede species construction"
    );
}

#[test]
fn focused_cli_fixture_pins_subarray_snapshot_result_shape_and_error_realm() {
    const REGISTRATION: &str =
        "#[test]\nfn run_wasm_backend_subarray_uses_non_throwing_typed_array_buffer_witness() {";
    assert_eq!(
        CLI_TESTS.matches(REGISTRATION).count(),
        1,
        "the focused TypedArray.prototype.subarray CLI owner must have one active registration"
    );
    let registration_offset = CLI_TESTS
        .find(REGISTRATION)
        .expect("missing active TypedArray.prototype.subarray CLI registration");
    let attached_source = CLI_TESTS[..registration_offset]
        .rsplit_once("\n}\n")
        .expect("missing CLI owner before the TypedArray.prototype.subarray registration")
        .1;
    let normalized_attached_source = without_whitespace(attached_source);
    for disabling_attribute in ["#[cfg", "#[cfg_attr", "#[ignore"] {
        assert!(
            !normalized_attached_source.contains(disabling_attribute),
            "the focused TypedArray.prototype.subarray CLI owner must not carry {disabling_attribute}"
        );
    }
    for comment_delimiter in ["//", "/*", "*/"] {
        assert!(
            !attached_source.contains(comment_delimiter),
            "the active CLI registration must not be supplied by {comment_delimiter} comment text"
        );
    }

    let test_body = CLI_TESTS
        .split_once(REGISTRATION)
        .expect("missing focused TypedArray.prototype.subarray CLI test")
        .1
        .split_once("\n#[test]")
        .expect("missing test after focused TypedArray.prototype.subarray CLI test")
        .0;
    for comment_delimiter in ["/*", "*/"] {
        assert!(
            !test_body.contains(comment_delimiter),
            "the active CLI owner must not contain {comment_delimiter} comment text"
        );
    }
    assert!(
        test_body
            .lines()
            .all(|line| !line.trim_start().starts_with("//")),
        "the active CLI owner must not contain line-commented controls"
    );
    let normalized_test_body = without_whitespace(test_body);
    for (wiring, role) in [
        (
            r#"
                let output = Command::new(env!("CARGO_BIN_EXE_lila"))
                    .arg("run")
                    .arg("--execution-backend")
                    .arg("wasm")
                    .arg(fixture_path("wasm_typedarray_subarray_buffer_witness.js"))
                    .output()
                    .expect("run command should run");
            "#,
            "exact Wasm fixture command",
        ),
        (
            r#"
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            "#,
            "successful CLI status assertion",
        ),
        (
            r#"let stdout = String::from_utf8_lossy(&output.stdout);"#,
            "successful process stdout binding",
        ),
        (
            r#"assert!(stdout.contains("backend_used: WasmAot"));"#,
            "Wasm-AOT backend assertion",
        ),
        (
            r#"assert!(stdout.contains("number(967"), "{stdout}");"#,
            "fixture success sentinel",
        ),
    ] {
        let wiring = without_whitespace(wiring);
        assert_eq!(
            normalized_test_body.matches(wiring.as_str()).count(),
            1,
            "the focused CLI owner must retain one {role}"
        );
    }

    for comment_delimiter in ["//", "/*", "*/"] {
        assert!(
            !CLI_FIXTURE.contains(comment_delimiter),
            "the focused fixture must not preserve controls only inside {comment_delimiter} comment text"
        );
    }

    for marker in [
        "fixed result out of bounds",
        "fixed result regrown",
        "tracking odd-byte shrink floor",
        "tracking odd-byte growth floor",
        "explicit end creates fixed result",
        "fixed Number species arguments",
        "tracking Number omitted-end species arguments",
        "tracking Number explicit-end species arguments",
        "fixed BigInt species arguments",
        "tracking BigInt omitted-end species arguments",
        "tracking BigInt explicit-end species arguments",
        "Object.getOwnPropertyDescriptor(actualArguments, \"2\")",
        "begin,end,species",
        "out-of-bounds zero length snapshot",
        "__lilaDetachArrayBuffer(detachedBuffer)",
        "detached coercion order",
        "custom detached species reached",
        "entry constructor owns detached error",
        "__lilaDetachArrayBuffer(detachedSpeciesResult.buffer)",
        "species detached result validation",
        "detached result species reached",
        "outOfBoundsSpeciesBuffer.resize(1)",
        "species out-of-bounds result validation",
        "out-of-bounds result species reached",
        "BigInt element kind",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing CLI control: {marker}"
        );
    }

    assert_eq!(
        CLI_FIXTURE
            .matches("assertSubarraySpeciesArguments(")
            .count(),
        7,
        "the fixture must define one argument-list assertion and execute all six Number/BigInt arity controls"
    );
    assert_eq!(
        CLI_FIXTURE.matches("Arguments = arguments;").count(),
        4,
        "each fixed or length-tracking Number/BigInt species must retain its escaped arguments object"
    );

    assert_eq!(
        CLI_FIXTURE.matches("other.TypeError.prototype").count(),
        2,
        "both post-species result-validation errors must belong to the borrowed method's Realm"
    );
}

#[test]
fn typed_array_prototype_installs_the_witnessed_subarray_builtin() {
    let installation = BOOTSTRAP_SOURCE
        .split_once("let subarray_meta = self")
        .expect("missing TypedArray.prototype.subarray installation")
        .1
        .split_once("let slice_meta = self")
        .expect("missing boundary after TypedArray.prototype.subarray installation")
        .0;

    assert_eq!(
        installation
            .matches("StandardBuiltinId::TypedArrayPrototypeSubarray.function_id()")
            .count(),
        1,
        "the public subarray property must resolve to the witnessed builtin ID"
    );
    assert_eq!(
        installation
            .matches(
                "typed_array_prototype_local,\n            \"subarray\",\n            subarray_meta"
            )
            .count(),
        1,
        "the witnessed builtin must remain installed as TypedArray.prototype.subarray"
    );

    let created_realm_methods = HOST_SOURCE
        .split_once("let typed_array_method_metas = [")
        .expect("missing created-Realm TypedArray method inventory")
        .1
        .split_once("let number_meta = self")
        .expect("missing boundary after created-Realm TypedArray method inventory")
        .0;
    assert_eq!(
        created_realm_methods
            .matches("StandardBuiltinId::TypedArrayPrototypeSubarray.function_id()")
            .count(),
        1,
        "created Realms must materialize their own TypedArray.prototype.subarray function"
    );
    let created_realm_materialization = HOST_SOURCE
        .split_once("for (name, meta) in &typed_array_method_metas {")
        .expect("missing created-Realm TypedArray method materialization")
        .1
        .split_once("let typed_array_buffer_key_local")
        .expect("missing boundary after created-Realm TypedArray method materialization")
        .0;
    for realm_binding in [
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert_eq!(
            created_realm_materialization.matches(realm_binding).count(),
            1,
            "created-Realm subarray must inherit the shared method materializer's {realm_binding} binding"
        );
    }
}
