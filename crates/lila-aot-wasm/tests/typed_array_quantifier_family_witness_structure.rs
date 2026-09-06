const CALLBACK_ITERATION_SOURCE: &str = include_str!("../src/builtins/array/callback_iteration.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

const BRAND_WIRING: &str = r#"
    function.instruction(&Instruction::LocalGet(receiver_brand_local));
    function.instruction(&Instruction::I64Const(
        OBJECT_INTERNAL_BRAND_TYPED_ARRAY as i64,
    ));
    function.instruction(&Instruction::I64Ne);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_throw_current_function_realm_type_error(
        match &quantifier {
            TypedArrayQuantifierKind::Every => "TypedArray every method requires a TypedArray",
            TypedArrayQuantifierKind::Some => "TypedArray some method requires a TypedArray",
        },
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
        receiver_buffer_local,
        receiver_byte_offset_local,
        receiver_byte_length_local,
        receiver_bytes_per_element_local,
        function,
    );
"#;

const VIEW_WIRING: &str = r#"
    let receiver_view = TypedArrayViewLocals::new(
        receiver_payload_local,
        receiver_buffer_local,
        receiver_byte_offset_local,
        receiver_byte_length_local,
        receiver_bytes_per_element_local,
    );
"#;

const WITNESS_WIRING: &str = r#"
    self.emit_typed_array_witness(
        &receiver_view,
        TypedArrayWitnessUse::ValidatedMethodEntry {
            length_local: len_local,
        },
        function,
    )?;
"#;

const CALLBACK_VALIDATION_WIRING: &str = r#"
    self.emit_builtin_arg_to_locals(0, callback_payload_local, callback_tag_local, function);
    self.emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?;
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_throw_current_function_realm_type_error(
        match &quantifier {
            TypedArrayQuantifierKind::Every => {
                "TypedArray.prototype.every callback is not callable"
            }
            TypedArrayQuantifierKind::Some => {
                "TypedArray.prototype.some callback is not callable"
            }
        },
        self.result_local,
        self.result_tag_local,
        function,
    )?;
    self.emit_return_current_completion(function);
    function.instruction(&Instruction::End);
"#;

const THIS_ARG_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(this_arg_payload_local));
    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
    function.instruction(&Instruction::LocalSet(this_arg_tag_local));
    function.instruction(&Instruction::LocalGet(self.argc_param_local()));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64GtU);
    function.instruction(&Instruction::If(BlockType::Empty));
    self.emit_builtin_arg_to_locals(1, this_arg_payload_local, this_arg_tag_local, function);
    function.instruction(&Instruction::End);
"#;

const LOOP_BOUND_WIRING: &str = r#"
    function.instruction(&Instruction::I64Const(0));
    function.instruction(&Instruction::LocalSet(index_local));
    function.instruction(&Instruction::Block(BlockType::Empty));
    function.instruction(&Instruction::Loop(BlockType::Empty));
    function.instruction(&Instruction::LocalGet(index_local));
    function.instruction(&Instruction::LocalGet(len_local));
    function.instruction(&Instruction::I64GeU);
    function.instruction(&Instruction::BrIf(1));
"#;

const LIVE_READ_AND_INDEX_WIRING: &str = r#"
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
    function.instruction(&Instruction::LocalGet(index_local));
    function.instruction(&Instruction::F64ConvertI64U);
    function.instruction(&Instruction::I64ReinterpretF64);
    function.instruction(&Instruction::LocalSet(index_payload_local));
    function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
    function.instruction(&Instruction::LocalSet(index_tag_local));
"#;

const CALLBACK_CALL_WIRING: &str = r#"
    self.emit_pre_evaluated_arg_vector(
        &[
            (element_payload_local, element_tag_local),
            (index_payload_local, index_tag_local),
            (receiver_payload_local, receiver_tag_local),
        ],
        argc_local,
        argv_local,
        function,
    )?;
    self.emit_function_or_proxy_call_with_argv_leave_throw_completion(
        callback_payload_local,
        callback_tag_local,
        this_arg_payload_local,
        this_arg_tag_local,
        argc_local,
        argv_local,
        callback_result_payload_local,
        callback_result_tag_local,
        function,
    )?;
"#;

const CALLBACK_OUTCOME_TAIL_WIRING: &str = r#"
    self.emit_propagate_throw_from_locals_if_needed(
        callback_result_payload_local,
        callback_result_tag_local,
        function,
    )?;
    self.compile_truthy_tagged_i32(
        callback_result_tag_local,
        callback_result_payload_local,
        function,
    )?;
    match &quantifier {
        TypedArrayQuantifierKind::Every => {
            function.instruction(&Instruction::I32Eqz);
        }
        TypedArrayQuantifierKind::Some => {}
    }
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::I64Const(match &quantifier {
        TypedArrayQuantifierKind::Every => 0,
        TypedArrayQuantifierKind::Some => 1,
    }));
    function.instruction(&Instruction::LocalSet(self.result_local));
    function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
    function.instruction(&Instruction::LocalSet(self.result_tag_local));
    self.emit_return_current_completion(function);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::LocalGet(index_local));
    function.instruction(&Instruction::I64Const(1));
    function.instruction(&Instruction::I64Add);
    function.instruction(&Instruction::LocalSet(index_local));
    function.instruction(&Instruction::Br(0));
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);

    function.instruction(&Instruction::I64Const(match &quantifier {
        TypedArrayQuantifierKind::Every => 1,
        TypedArrayQuantifierKind::Some => 0,
    }));
    function.instruction(&Instruction::LocalSet(self.result_local));
    function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
    function.instruction(&Instruction::LocalSet(self.result_tag_local));
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

fn quantifier_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "fn compile_typed_array_prototype_quantifier_builtin(",
        "pub(crate) fn compile_array_prototype_every_builtin(",
    )
}

fn array_every_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_array_prototype_every_builtin(",
        "pub(crate) fn compile_array_prototype_some_builtin(",
    )
}

fn array_some_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_array_prototype_some_builtin(",
        "pub(crate) fn compile_array_prototype_filter_builtin(",
    )
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
fn typed_array_quantifier_kind_has_exactly_two_inhabitants() {
    let declaration = ARRAY_SOURCE
        .split_once("enum TypedArrayQuantifierKind {")
        .expect("TypedArray quantifier kind")
        .1
        .split_once('}')
        .expect("TypedArray quantifier kind end")
        .0;
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Every,", "Some,"]);
    let declaration_offset = ARRAY_SOURCE
        .find("enum TypedArrayQuantifierKind {")
        .expect("TypedArray quantifier kind declaration");
    assert_eq!(
        ARRAY_SOURCE[..declaration_offset]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("}")
    );
    for forbidden in [
        "pub enum TypedArrayQuantifierKind",
        "pub(crate) enum TypedArrayQuantifierKind",
        "pub(super) enum TypedArrayQuantifierKind",
        "impl Clone for TypedArrayQuantifierKind",
        "impl Copy for TypedArrayQuantifierKind",
        "impl PartialEq for TypedArrayQuantifierKind",
        "impl Eq for TypedArrayQuantifierKind",
        "impl Default for TypedArrayQuantifierKind",
    ] {
        assert!(
            !ARRAY_SOURCE.contains(forbidden),
            "the quantifier authority must not expose `{forbidden}`"
        );
    }
}

#[test]
fn typed_array_quantifier_dispatch_and_witness_are_closed() {
    let body = quantifier_body();
    let normalized_body = without_whitespace(body);

    assert_eq!(body.matches("match &quantifier").count(), 7);
    for forbidden in [
        "match quantifier",
        "quantifier ==",
        "quantifier !=",
        "matches!(quantifier",
        "quantifier: bool",
        "_ =>",
        "unreachable!",
    ] {
        assert!(
            !body.contains(forbidden),
            "the quantifier authority must be projected exhaustively, not through `{forbidden}`"
        );
    }

    assert_eq!(
        body.matches("emit_load_typed_array_private_state(").count(),
        1,
        "the quantifier compiler must load one immutable private view record"
    );
    assert_eq!(
        body.matches("TypedArrayViewLocals::new(").count(),
        1,
        "the quantifier compiler must construct one immutable view projection"
    );
    assert_eq!(
        body.matches("emit_typed_array_witness(").count(),
        1,
        "the quantifier compiler must create one live buffer witness"
    );
    assert_eq!(
        body.matches("TypedArrayWitnessUse::ValidatedMethodEntry")
            .count(),
        1,
        "the quantifier compiler must select the throwing method-entry projection"
    );
    assert_eq!(body.matches("receiver_view").count(), 2);

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
    ] {
        assert!(
            !body.contains(forbidden),
            "the quantifier compiler must not bypass its witness through {forbidden}"
        );
    }

    let brand = unique_normalized_position(
        &normalized_body,
        BRAND_WIRING,
        "completed receiver-brand guard",
    );
    let private_state = unique_normalized_position(
        &normalized_body,
        PRIVATE_STATE_WIRING,
        "exact private-state wiring",
    );
    let view =
        unique_normalized_position(&normalized_body, VIEW_WIRING, "exact immutable-view wiring");
    let witness = unique_normalized_position(
        &normalized_body,
        WITNESS_WIRING,
        "exact validated method-entry witness wiring",
    );
    let callback_validation = unique_normalized_position(
        &normalized_body,
        CALLBACK_VALIDATION_WIRING,
        "exact callback validation wiring",
    );
    assert!(
        brand < private_state
            && private_state < view
            && view < witness
            && witness < callback_validation,
        "brand validation, private view, method-entry witness and callback validation must retain specification order"
    );

    for message in [
        r#""TypedArray every method requires a TypedArray""#,
        r#""TypedArray some method requires a TypedArray""#,
    ] {
        assert_eq!(
            body.matches(message).count(),
            1,
            "receiver-brand anchor must occur once in the bounded quantifier compiler"
        );
    }

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

    let normalized_array = without_whitespace(ARRAY_SOURCE).replace(",)", ")");
    for (compiler, variant) in [("every", "Every"), ("some", "Some")] {
        let mapping = format!(
            "compile_typed_array_prototype_{compiler}_builtin(&mutself,function:&mutFunction)->Result<(),EmitError>{{self.compile_typed_array_prototype_quantifier_builtin(function,TypedArrayQuantifierKind::{variant})}}"
        );
        unique_normalized_position(
            &normalized_array,
            &mapping,
            &format!("TypedArray {compiler} wrapper -> TypedArrayQuantifierKind::{variant}"),
        );
    }
    assert_eq!(
        ARRAY_SOURCE
            .matches("compile_typed_array_prototype_quantifier_builtin(")
            .count(),
        3,
        "the private shared compiler definition and both wrapper producers must be the complete authority census"
    );
    for forbidden in [
        "pub fn compile_typed_array_prototype_quantifier_builtin(",
        "pub(crate) fn compile_typed_array_prototype_quantifier_builtin(",
        "pub(super) fn compile_typed_array_prototype_quantifier_builtin(",
    ] {
        assert!(
            !ARRAY_SOURCE.contains(forbidden),
            "the shared quantifier compiler must remain private: `{forbidden}`"
        );
    }
    assert!(!STANDARD_SOURCE.contains("TypedArrayQuantifierKind"));

    let normalized_standard = without_whitespace(STANDARD_SOURCE).replace(",)", ")");
    for (builtin, compiler) in [
        ("TypedArrayPrototypeEvery", "every"),
        ("TypedArrayPrototypeSome", "some"),
    ] {
        let mapping = format!(
            "StandardBuiltinId::{builtin}=>{{self.compile_typed_array_prototype_{compiler}_builtin(function)?;}}"
        );
        unique_normalized_position(
            &normalized_standard,
            &mapping,
            &format!("StandardBuiltinId::{builtin} -> TypedArray {compiler} wrapper"),
        );
    }
}

#[test]
fn array_and_typed_array_quantifier_entry_families_are_disjoint() {
    for (method, body) in [
        ("Array.prototype.every", array_every_body()),
        ("Array.prototype.some", array_some_body()),
    ] {
        for forbidden in [
            "typed_array_only",
            "typed_brand_local",
            "TypedArrayWitnessUse::ValidatedMethodEntry",
            "method requires a TypedArray",
        ] {
            assert!(
                !body.contains(forbidden),
                "{method} must not retain the unreachable strict TypedArray entry projection {forbidden}"
            );
        }
        assert!(body.contains("self.compile_array_callback_iteration("));
        assert!(!body.contains("TypedArrayWitnessUse::"));
        assert_eq!(
            CALLBACK_ITERATION_SOURCE
                .matches("self.emit_object_has_property_i32(")
                .count(),
            1
        );
        assert_eq!(
            CALLBACK_ITERATION_SOURCE
                .matches("self.emit_typed_array_or_object_index_read_from_locals(")
                .count(),
            1
        );
        assert!(!CALLBACK_ITERATION_SOURCE.contains("TypedArrayWitnessUse::ValidatedMethodEntry"));
    }

    let normalized_standard = without_whitespace(STANDARD_SOURCE).replace(",)", ")");
    for (builtin, compiler) in [
        (
            "ArrayPrototypeEvery",
            "compile_array_prototype_every_builtin",
        ),
        ("ArrayPrototypeSome", "compile_array_prototype_some_builtin"),
    ] {
        let mapping = format!("StandardBuiltinId::{builtin}=>{{self.{compiler}(function)?;}}");
        unique_normalized_position(
            &normalized_standard,
            &mapping,
            &format!("StandardBuiltinId::{builtin} -> {compiler}"),
        );
        assert_eq!(
            STANDARD_SOURCE.matches(&format!("{compiler}(")).count(),
            1,
            "the generic {builtin} compiler must have exactly one dispatcher producer"
        );
    }
}

#[test]
fn array_quantifiers_have_no_array_result_or_species_residue() {
    for (method, body) in [
        ("Array.prototype.every", array_every_body()),
        ("Array.prototype.some", array_some_body()),
    ] {
        for forbidden in [
            "array_constructor_table_index",
            "constructor_table_index_local",
            "skip_species_local",
            "species_source_payload_local",
            "species_source_tag_local",
            "species_source_is_array_local",
            "species_proxy_kind_local",
            "species_payload_local",
            "species_tag_local",
            "result_payload_local",
            "target_payload_local",
            "target_tag_local",
            "zero_local",
            "out_index_local",
            "mapped_len_local",
            "mapped_flatten_payload_local",
            "mapped_flatten_tag_local",
            "mapped_index_local",
            "child_payload_local",
            "child_tag_local",
            "typed_buffer_tag_local",
            "typed_data_ptr_local",
            "typed_address_local",
            "emit_array_constructor_read(",
            "emit_mark_skip_species_for_cross_realm_array_constructor(",
            "property_key_symbol_payload(\"Symbol.species\")",
            "emit_alloc_array_payload_with_length(",
            "emit_function_handle_construct_with_argv(",
        ] {
            assert!(
                !body.contains(forbidden),
                "{method} must not retain Array-producing residue `{forbidden}`"
            );
        }

        assert!(body.contains("self.compile_array_callback_iteration("));
        assert_eq!(body.matches("reserve_temp_local()").count(), 0);
        assert_eq!(body.matches("release_temp_local(").count(), 0);
        for forbidden in [
            "TypedArrayViewLocals::new(",
            "emit_load_typed_array_private_state(",
        ] {
            assert!(
                !CALLBACK_ITERATION_SOURCE.contains(forbidden),
                "{method} must leave live integer-indexed policy with shared property operations"
            );
        }
        assert!(CALLBACK_ITERATION_SOURCE.contains(
            "ArrayCallbackIterationKind::Every | ArrayCallbackIterationKind::Some => {}"
        ));
    }
}

#[test]
fn typed_array_quantifier_preserves_live_callback_sequence_and_polarity() {
    let body = quantifier_body();
    let normalized_body = without_whitespace(body);

    assert_eq!(body.matches("emit_is_callable_i32(").count(), 1);
    assert_eq!(
        body.matches("emit_typed_array_or_object_index_read_from_locals(")
            .count(),
        1
    );
    assert_eq!(body.matches("emit_pre_evaluated_arg_vector(").count(), 1);
    assert_eq!(
        body.matches("emit_function_or_proxy_call_with_argv_leave_throw_completion(")
            .count(),
        1
    );
    assert_eq!(
        body.matches("emit_propagate_throw_from_locals_if_needed(")
            .count(),
        2,
        "the indexed read and callback result must each propagate abrupt completion"
    );
    assert_eq!(body.matches("compile_truthy_tagged_i32(").count(), 1);
    assert!(!body.contains("emit_function_handle_call_with_argv"));

    let callback_validation = unique_normalized_position(
        &normalized_body,
        CALLBACK_VALIDATION_WIRING,
        "callback validation",
    );
    let this_arg = unique_normalized_position(
        &normalized_body,
        THIS_ARG_WIRING,
        "preserved optional thisArg",
    );
    let loop_bound = unique_normalized_position(
        &normalized_body,
        LOOP_BOUND_WIRING,
        "captured length loop bound",
    );
    let live_read = unique_normalized_position(
        &normalized_body,
        LIVE_READ_AND_INDEX_WIRING,
        "live read, abrupt propagation and numeric index",
    );
    let callback_call = unique_normalized_position(
        &normalized_body,
        CALLBACK_CALL_WIRING,
        "Proxy-aware callback thisArg and value/index/receiver argument wiring",
    );
    let outcome_tail = unique_normalized_position(
        &normalized_body,
        CALLBACK_OUTCOME_TAIL_WIRING,
        "contiguous callback propagation, truthiness, polarity, advance and terminal tail",
    );
    assert_eq!(
        body.matches("Instruction::LocalSet(self.result_local)")
            .count(),
        2,
        "only the short-circuit and terminal projections may write the Boolean result payload"
    );
    assert_eq!(
        body.matches("Instruction::LocalSet(self.result_tag_local)")
            .count(),
        2,
        "only the short-circuit and terminal projections may write the Boolean result tag"
    );
    assert!(!body.contains("Instruction::LocalTee(self.result_local)"));
    assert!(!body.contains("Instruction::LocalTee(self.result_tag_local)"));

    assert!(
        callback_validation < this_arg
            && this_arg < loop_bound
            && loop_bound < live_read
            && live_read < callback_call
            && callback_call < outcome_tail,
        "quantifiers must preserve callback validation, snapshot-bound live reads, abrupt routing, truthiness, short-circuiting and terminal projection order"
    );
}
