const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");

fn typed_array_method_table() -> &'static str {
    HOST_SOURCE
        .split_once("        let typed_array_method_metas = [")
        .expect("created-Realm TypedArray method table")
        .1
        .split_once("        let array_typed_array_to_string_meta = self")
        .expect("created-Realm TypedArray method table end")
        .0
}

fn method_entry<'a>(table: &'a str, name: &str) -> &'a str {
    let start = format!("            (\n                \"{name}\",");
    assert_eq!(
        table.matches(&start).count(),
        1,
        "created-Realm TypedArray method `{name}` must occur exactly once"
    );
    table
        .split_once(&start)
        .expect("checked method entry")
        .1
        .split_once("            ),")
        .expect("method entry end")
        .0
}

#[test]
fn created_realm_publishes_the_complete_main_realm_typed_array_method_surface() {
    let table = typed_array_method_table();
    let expected = [
        ("at", "TypedArrayPrototypeAt"),
        ("includes", "TypedArrayPrototypeIncludes"),
        ("indexOf", "TypedArrayPrototypeIndexOf"),
        ("lastIndexOf", "TypedArrayPrototypeLastIndexOf"),
        ("find", "TypedArrayPrototypeFind"),
        ("findIndex", "TypedArrayPrototypeFindIndex"),
        ("findLast", "TypedArrayPrototypeFindLast"),
        ("findLastIndex", "TypedArrayPrototypeFindLastIndex"),
        ("every", "TypedArrayPrototypeEvery"),
        ("some", "TypedArrayPrototypeSome"),
        ("map", "TypedArrayPrototypeMap"),
        ("filter", "TypedArrayPrototypeFilter"),
        ("forEach", "TypedArrayPrototypeForEach"),
        ("reduce", "TypedArrayPrototypeReduce"),
        ("reduceRight", "TypedArrayPrototypeReduceRight"),
        ("values", "TypedArrayPrototypeValues"),
        ("keys", "TypedArrayPrototypeKeys"),
        ("entries", "TypedArrayPrototypeEntries"),
        ("fill", "ArrayPrototypeFill"),
        ("join", "TypedArrayPrototypeJoin"),
        ("subarray", "TypedArrayPrototypeSubarray"),
        ("slice", "TypedArrayPrototypeSlice"),
        ("set", "TypedArrayPrototypeSet"),
        ("reverse", "TypedArrayPrototypeReverse"),
        ("copyWithin", "TypedArrayPrototypeCopyWithin"),
        ("sort", "TypedArrayPrototypeSort"),
        ("toReversed", "TypedArrayPrototypeToReversed"),
        ("toSorted", "TypedArrayPrototypeToSorted"),
        ("with", "TypedArrayPrototypeWith"),
        ("toLocaleString", "TypedArrayPrototypeToLocaleString"),
    ];

    assert_eq!(table.matches(".function_id())").count(), expected.len());
    let mut previous_entry_offset = 0;
    for (name, builtin) in expected {
        let builtin_reference = format!("StandardBuiltinId::{builtin}");
        assert!(
            BOOTSTRAP_SOURCE.contains(&builtin_reference),
            "main-Realm TypedArray bootstrap must publish `{builtin}`"
        );
        let entry = method_entry(table, name);
        let entry_offset = table
            .find(&format!("            (\n                \"{name}\","))
            .expect("checked method entry offset");
        assert!(
            entry_offset >= previous_entry_offset,
            "created-Realm TypedArray method `{name}` is out of main-Realm publication order"
        );
        previous_entry_offset = entry_offset;
        assert_eq!(
            entry
                .matches(&format!("{builtin_reference}.function_id()"))
                .count(),
            1,
            "created-Realm TypedArray method `{name}` must use `{builtin}`"
        );
    }
}

#[test]
fn created_realm_array_and_typed_array_share_one_to_string_function() {
    let installer = HOST_SOURCE
        .split_once("        let array_typed_array_to_string_payload_local =")
        .expect("created-Realm shared Array/TypedArray toString installer")
        .1
        .split_once("        for (name, meta) in &object_prototype_method_metas {")
        .expect("created-Realm shared Array/TypedArray toString installer end")
        .0;

    assert_eq!(
        HOST_SOURCE
            .matches("StandardBuiltinId::TypedArrayPrototypeToString.function_id()")
            .count(),
        1,
        "the shared method must have one created-Realm metadata authority"
    );
    assert_eq!(
        installer
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        1,
        "the shared method must be materialized once"
    );
    assert_eq!(
        installer
            .matches("array_typed_array_to_string_payload_local,")
            .count(),
        6,
        "one payload must feed its Realm snapshots and both prototype properties"
    );
    for definition in [
        "self.emit_define_realm_array_prototype_data_with_flags(",
        "self.emit_object_define_local_data_with_flags(",
    ] {
        assert_eq!(installer.matches(definition).count(), 1);
    }
    for binding in [
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert_eq!(installer.matches(binding).count(), 1);
    }
    assert_eq!(installer.matches("\n            \"toString\",").count(), 2);
    assert_eq!(
        installer
            .matches("\n            true,\n            false,\n            true,")
            .count(),
        2
    );
}

#[test]
fn created_realm_typed_array_methods_capture_error_prototypes_in_their_environment() {
    let installer = HOST_SOURCE
        .split_once("        for (name, meta) in &typed_array_method_metas {")
        .expect("created-Realm TypedArray method installer")
        .1
        .split_once("        let typed_array_buffer_key_local")
        .expect("created-Realm TypedArray method installer end")
        .0;

    for binding in [
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
        "HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert_eq!(
            installer.matches(binding).count(),
            1,
            "created-Realm TypedArray method installer must set {binding} exactly once"
        );
    }
    assert_eq!(
        installer
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        1
    );
    assert_eq!(
        installer
            .matches("self.emit_object_define_local_data(")
            .count(),
        2,
        "one general property and the values/@@iterator alias must be installed"
    );
}
