const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn direct_pop_branch_delegates_without_a_second_array_algorithm() {
    let direct = bounded(
        FUNCTIONS_SOURCE,
        "if matches!(key, PropertyKeyIr::StaticString(name) if name == \"pop\") {",
        "if matches!(key, PropertyKeyIr::StaticString(name) if name == \"splice\") {",
    );

    assert_eq!(
        direct
            .matches("self.emit_array_direct_builtin_method_call(")
            .count(),
        1
    );
    assert_eq!(
        direct
            .matches("StandardBuiltinId::ArrayPrototypePop,")
            .count(),
        1
    );
    assert_eq!(direct.matches("\"Array.prototype.pop\",").count(), 1);

    for forbidden in [
        "HEAP_LEN_OFFSET",
        "emit_array_read(",
        "load_i64_to_local_from_offset(",
        "store_i64_local_at_offset(",
        "reserve_temp_local(",
        "compile_expr_to_locals(",
        "emit_object_read(",
        "emit_object_delete(",
        "emit_object_write",
        "emit_throw_runtime_error(",
        "ValueKind::Array",
    ] {
        assert!(
            !direct.contains(forbidden),
            "direct pop branch must not retain parallel operation `{forbidden}`"
        );
    }
}

#[test]
fn standard_pop_body_owns_the_complete_ordered_algorithm() {
    let canonical = bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::ArrayPrototypePop => {",
        "StandardBuiltinId::ArrayPrototypePush => {",
    );

    assert_eq!(
        canonical
            .matches("emit_value_to_current_function_realm_object_locals(")
            .count(),
        1
    );
    assert_eq!(
        canonical
            .matches("self.strings.payload(\"length\")")
            .count(),
        2
    );
    assert_eq!(canonical.matches("self.emit_object_read(").count(), 2);
    assert_eq!(
        canonical
            .matches("emit_to_length_i64_from_value_locals(")
            .count(),
        1
    );
    assert_eq!(canonical.matches("self.emit_object_delete(").count(), 1);
    assert_eq!(
        canonical
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        2
    );
    assert_eq!(
        canonical.matches("self.emit_object_write_strict(").count(),
        1
    );

    assert_before(
        canonical,
        "emit_value_to_current_function_realm_object_locals(",
        "self.strings.payload(\"length\")",
    );
    assert_before(
        canonical,
        "self.strings.payload(\"length\")",
        "emit_to_length_i64_from_value_locals(",
    );

    let after_length = canonical
        .split_once("emit_to_length_i64_from_value_locals(")
        .expect("LengthOfArrayLike conversion")
        .1;
    assert_before(
        after_length,
        "self.emit_object_read(",
        "self.emit_object_delete(",
    );

    let after_delete = canonical
        .split_once("self.emit_object_delete(")
        .expect("DeletePropertyOrThrow boundary")
        .1;
    assert_before(
        after_delete,
        "emit_throw_current_function_realm_type_error(",
        "self.emit_object_write_strict(",
    );
}
