const ENUMERABLE_OWN_PROPERTIES_SOURCE: &str =
    include_str!("../src/builtins/object/enumerable_own_properties.rs");

fn enumerable_own_properties_body() -> &'static str {
    ENUMERABLE_OWN_PROPERTIES_SOURCE
        .split_once("    fn compile_object_enumerable_own_properties_builtin(")
        .expect("Object.values/Object.entries compiler")
        .1
        .split_once("    pub(in crate::builtins) fn compile_object_entries_builtin(")
        .expect("Object.values/Object.entries compiler end")
        .0
}

#[test]
fn object_values_and_entries_preserve_the_realm_array_prototype_representation() {
    let body = enumerable_own_properties_body();

    assert_eq!(
        body.matches("HEAP_REALM_INTRINSICS_ARRAY_PROTOTYPE_OFFSET")
            .count(),
        1,
        "the shared algorithm must load its defining realm's Array prototype once"
    );
    assert_eq!(
        body.matches("ARRAY_PROTOTYPE_GLOBAL_INDEX").count(),
        1,
        "the entry Array prototype is only the missing-realm fallback, never a tag discriminator"
    );
    assert_eq!(
        body.matches("HEAP_ARRAY_PROTOTYPE_TAG_OFFSET").count(),
        2,
        "the result array and each Object.entries pair must both record their prototype tag"
    );
    assert_eq!(
        body.matches("ValueKind::Array.tag() as u64").count(),
        2,
        "every realm's %Array.prototype% is an Array exotic"
    );
    assert!(!body.contains("array_prototype_tag_local"));
    assert!(!body.contains("GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX)"));
}
