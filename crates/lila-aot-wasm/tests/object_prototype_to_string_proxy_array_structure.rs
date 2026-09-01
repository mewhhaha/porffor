const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const OBJECT_SOURCE: &str = include_str!("../src/builtins/object.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_object_prototype_to_string_proxy_array.js");
const CLI_REGISTRATION: &str = include_str!("../../lila-cli/tests/cli/object.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing `{end}` after `{start}`"))
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
fn tostring_consumers_use_typed_brand_authorities_before_tag_lookup() {
    assert!(DATA_SOURCE.contains("\"[object RegExp]\","));
    assert_eq!(
        OBJECTS_SOURCE
            .matches("pub(crate) fn emit_is_array_i64(")
            .count(),
        1
    );

    let is_array = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_is_array_i64(",
        "pub(crate) fn emit_alloc_plain_object_with_prototype(",
    );
    assert!(is_array.contains("self.emit_throw_current_function_realm_type_error("));
    assert!(!is_array.contains("self.emit_throw_runtime_error("));

    let direct = bounded(
        OBJECT_SOURCE,
        "pub(super) fn compile_object_prototype_to_string_builtin(",
        "pub(super) fn compile_object_prototype_value_of_builtin(",
    );
    let fallback = bounded(
        ARRAY_SOURCE,
        "pub(crate) fn emit_object_prototype_to_string_result_from_locals(",
        "pub(crate) fn compile_typed_array_prototype_to_string_builtin(",
    );

    for source in [direct, fallback] {
        assert_eq!(source.matches("self.emit_is_array_i64(").count(), 1);
        assert_eq!(source.matches("self.emit_is_callable_i32(").count(), 1);
        assert_before(source, "self.emit_is_array_i64(", "self.emit_object_read(");
        assert_before(
            source,
            "self.emit_is_array_i64(",
            "self.emit_is_callable_i32(",
        );
        assert_before(
            source,
            "self.emit_is_callable_i32(",
            "self.emit_object_read(",
        );
        for builtin_tag_authority in [
            "(ValueKind::String, \"[object String]\")",
            "(ValueKind::Arguments, \"[object Arguments]\")",
            "BOXED_PRIMITIVE_KIND_BOOLEAN",
            "BOXED_PRIMITIVE_KIND_NUMBER",
            "OBJECT_INTERNAL_BRAND_ERROR",
            "OBJECT_INTERNAL_BRAND_DATE",
            "OBJECT_INTERNAL_BRAND_REGEXP",
        ] {
            assert!(
                source.contains(builtin_tag_authority),
                "missing builtin-tag authority: {builtin_tag_authority}"
            );
        }
        assert!(!source.contains("HEAP_OBJECT_BOXED_PAYLOAD_OFFSET"));
        assert!(!source.contains("PROXY_HANDLER_PAYLOAD_MIN"));
    }
}

#[test]
fn product_fixture_covers_direct_nested_and_revoked_proxy_arrays() {
    for marker in [
        "var direct = new Proxy(target, {});",
        "var nested = new Proxy(direct, {});",
        "Object.prototype.toString.call(direct)",
        "Object.prototype.toString.call(nested)",
        "Array.prototype.toString.call(direct)",
        "Array.prototype.toString.call(nested)",
        "var revocable = Proxy.revocable([], {});",
        "revocable.revoke();",
        "revokedObjectThrows = error instanceof TypeError;",
        "revokedArrayFallbackThrows = error instanceof TypeError;",
        "var mainTypedArrayToString = new Uint8Array(0).toString;",
        "var other = __lilaCreateRealm().global;",
        "other.Object.prototype.toString.call(revocable.proxy)",
        "other.Array.isArray(revocable.proxy)",
        "other.Array.prototype.toString.call(revocable.proxy)",
        "otherArrayToString === otherTypedArrayToString",
        "mainTypedArrayToString === Array.prototype.toString",
        "otherArrayToString.call(direct) === \"[object Array]\"",
        "Object.getPrototypeOf(otherObjectError) === other.TypeError.prototype",
        "Object.getPrototypeOf(otherArrayIsArrayError) === other.TypeError.prototype",
        "Object.getPrototypeOf(otherArrayToStringError) === other.TypeError.prototype",
    ] {
        assert!(FIXTURE.contains(marker), "missing fixture marker: {marker}");
    }
    assert!(CLI_REGISTRATION.contains(
        "fn object_prototype_tostring_classifies_proxy_arrays_and_rejects_revoked_proxies()"
    ));
    assert!(CLI_REGISTRATION.contains("wasm_object_prototype_to_string_proxy_array.js"));
}
