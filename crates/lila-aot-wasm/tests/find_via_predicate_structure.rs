const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn find_via_predicate_kind_has_exactly_four_inhabitants() {
    let body = ARRAY_SOURCE
        .split_once("pub(crate) enum FindViaPredicateKind {")
        .expect("find kind")
        .1
        .split_once('}')
        .expect("find kind end")
        .0;
    let variants = body
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        ["Find,", "FindIndex,", "FindLast,", "FindLastIndex,"]
    );
}

#[test]
fn predicate_witness_has_one_validator_and_one_proxy_aware_consumer() {
    let declaration = ARRAY_SOURCE
        .split_once("struct ValidatedFindPredicateLocals")
        .expect("validated predicate declaration")
        .0
        .rsplit_once("\n\n")
        .expect("attribute boundary")
        .1;
    assert!(!declaration.contains("derive"));
    assert!(!declaration.contains("pub"));
    assert!(!ARRAY_SOURCE.contains("impl Copy for ValidatedFindPredicateLocals"));
    assert_eq!(
        ARRAY_SOURCE
            .matches("ValidatedFindPredicateLocals(")
            .count(),
        3
    );

    let validator = ARRAY_SOURCE
        .split_once("fn emit_validate_find_predicate(")
        .expect("validator")
        .1
        .split_once("fn emit_call_validated_find_predicate(")
        .expect("validator end")
        .0;
    assert_eq!(validator.matches("emit_is_callable_i32").count(), 1);
    assert!(!validator.contains("ValueKind::Function"));

    let consumer = ARRAY_SOURCE
        .split_once("fn emit_call_validated_find_predicate(")
        .expect("consumer")
        .1
        .split_once("fn emit_initialize_find_result(")
        .expect("consumer end")
        .0;
    assert_eq!(
        consumer
            .matches("emit_function_or_proxy_call_with_argv_leave_throw_completion")
            .count(),
        1
    );
    assert!(!consumer.contains("emit_function_handle_call_with_argv"));
}

#[test]
fn array_and_typed_array_entries_share_the_closed_four_kind_dispatch() {
    let typed_entry = ARRAY_SOURCE
        .split_once("pub(crate) fn compile_typed_array_prototype_find_builtin(")
        .expect("typed entry")
        .1
        .split_once("pub(crate) fn compile_array_prototype_find_builtin(")
        .expect("typed entry end")
        .0;
    let array_entry = ARRAY_SOURCE
        .split_once("pub(crate) fn compile_array_prototype_find_builtin(")
        .expect("array entry")
        .1
        .split_once("fn emit_array_iteration_to_object(")
        .expect("array entry end")
        .0;
    assert_before(
        typed_entry,
        "emit_validate_typed_array_current_byte_length(",
        "emit_validate_find_predicate(",
    );
    assert_before(
        array_entry,
        "emit_array_iteration_to_object(",
        "emit_validate_find_predicate(",
    );
    assert_before(
        array_entry,
        "TypedArrayWitnessUse::ArrayLikeLengthSnapshot",
        "emit_validate_find_predicate(",
    );
    assert_before(
        array_entry,
        "emit_to_length_i64_from_value_locals(",
        "emit_validate_find_predicate(",
    );
    for entry in [typed_entry, array_entry] {
        assert_eq!(entry.matches("emit_validate_find_predicate(").count(), 1);
        assert_eq!(
            entry.matches("emit_call_validated_find_predicate(").count(),
            1
        );
        assert!(!entry.contains("emit_function_handle_call_with_argv"));
        assert!(!entry.contains("emit_function_or_proxy_call_with_argv_leave_throw_completion"));
        assert!(!entry.contains("emit_is_callable_i32"));
    }
    for forbidden in [
        "typed_array_only",
        "return_index",
        "reverse",
        "typed_brand_local",
        "typed_buffer_tag_local",
    ] {
        assert!(!array_entry.contains(forbidden));
    }

    for variant in ["Find", "FindIndex", "FindLast", "FindLastIndex"] {
        let comma_uses = STANDARD_SOURCE
            .matches(&format!("FindViaPredicateKind::{variant},"))
            .count();
        let closing_uses = STANDARD_SOURCE
            .matches(&format!("FindViaPredicateKind::{variant})"))
            .count();
        assert_eq!(comma_uses + closing_uses, 2);
    }
    assert!(!STANDARD_SOURCE.contains("TypedArrayFindKind"));
}
