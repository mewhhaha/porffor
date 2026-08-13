const COLLECTIONS_SOURCE: &str = include_str!("../src/builtins/collections.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");

fn impl_body(type_name: &str, next_impl: &str) -> &'static str {
    COLLECTIONS_SOURCE
        .split_once(&format!("impl {type_name} {{"))
        .unwrap_or_else(|| panic!("{type_name} impl"))
        .1
        .split_once(&format!("impl {next_impl} {{"))
        .unwrap_or_else(|| panic!("{type_name} impl end"))
        .0
}

fn between(start: &str, end: &str) -> &'static str {
    COLLECTIONS_SOURCE
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

fn host_between(start: &str, end: &str) -> &'static str {
    HOST_SOURCE
        .split_once(start)
        .unwrap_or_else(|| panic!("missing host start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing host end marker: {end}"))
        .0
}

#[test]
fn collection_data_brands_have_one_mapping_authority() {
    let authority = impl_body(
        "CollectionDataReceiverKind",
        "CollectionReceiverRequirement",
    );
    for mapping in [
        "Self::Map => OBJECT_INTERNAL_BRAND_MAP,",
        "Self::WeakMap => OBJECT_INTERNAL_BRAND_WEAK_MAP,",
        "Self::Set => OBJECT_INTERNAL_BRAND_SET,",
        "Self::WeakSet => OBJECT_INTERNAL_BRAND_WEAK_SET,",
    ] {
        assert_eq!(authority.matches(mapping).count(), 1, "{mapping}");
    }

    for brand in [
        "OBJECT_INTERNAL_BRAND_MAP",
        "OBJECT_INTERNAL_BRAND_WEAK_MAP",
        "OBJECT_INTERNAL_BRAND_SET",
        "OBJECT_INTERNAL_BRAND_WEAK_SET",
    ] {
        let exact_uses = COLLECTIONS_SOURCE
            .match_indices(brand)
            .filter(|(offset, _)| {
                COLLECTIONS_SOURCE[*offset + brand.len()..]
                    .chars()
                    .next()
                    .map(|next| !next.is_ascii_alphanumeric() && next != '_')
                    .unwrap_or(true)
            })
            .count();
        assert_eq!(exact_uses, 1, "{brand} must have one mapping authority");
    }

    let set_kind = impl_body("SetCollectionKind", "MapCollectionKind");
    let map_kind = COLLECTIONS_SOURCE
        .split_once("impl MapCollectionKind {")
        .expect("MapCollectionKind impl")
        .1
        .split_once("enum SetAlgebraOperation")
        .expect("MapCollectionKind impl end")
        .0;
    for kind_impl in [set_kind, map_kind] {
        assert!(!kind_impl.contains("OBJECT_INTERNAL_BRAND_"));
    }
    assert_eq!(
        COLLECTIONS_SOURCE
            .matches("fn brand(self) -> u64 {\n        self.receiver_kind().brand()\n    }")
            .count(),
        2,
        "MapCollectionKind and SetCollectionKind must delegate brand selection"
    );
}

#[test]
fn collection_algorithm_type_errors_use_one_current_function_realm_authority() {
    let map_constructor = between(
        "    fn emit_map_collection_constructor(",
        "    pub(crate) fn emit_map_group_by",
    );
    let map_for_each = between(
        "    pub(crate) fn emit_map_prototype_for_each(",
        "    pub(crate) fn emit_map_prototype_set(",
    );
    let set_constructor = between(
        "    fn emit_set_collection_constructor(",
        "    pub(crate) fn emit_set_prototype_add(",
    );
    let set_for_each = between(
        "    pub(crate) fn emit_set_prototype_for_each(",
        "    pub(crate) fn emit_set_prototype_size_getter(",
    );

    for (body, expected_calls) in [
        (map_constructor, 7),
        (map_for_each, 1),
        (set_constructor, 6),
        (set_for_each, 1),
    ] {
        assert_eq!(
            body.matches("emit_collection_algorithm_type_error(")
                .count(),
            expected_calls
        );
        assert!(!body.contains("emit_throw_runtime_error("));
        assert!(!body.contains("TYPE_ERROR_NAME"));
    }
    assert_eq!(
        COLLECTIONS_SOURCE
            .matches("self.emit_collection_algorithm_type_error(")
            .count(),
        15
    );

    let authority = between(
        "impl CollectionAlgorithmTypeError {",
        "impl MapConstructorTypeError {",
    );
    assert!(!authority.contains("_ =>"));
    assert_eq!(authority.matches("{} constructor {}").count(), 2);
    assert_eq!(
        authority
            .matches("{}.prototype.forEach callback must be callable")
            .count(),
        1
    );

    let map_stage = between(
        "impl MapConstructorTypeError {",
        "impl SetConstructorTypeError {",
    );
    let set_stage = between("impl SetConstructorTypeError {", "impl SetCollectionKind {");
    for stage in [map_stage, set_stage] {
        assert!(!stage.contains("_ =>"));
        for suffix in [
            "requires new",
            "iterator method is not callable",
            "iterator method must return an object",
            "iterator next method is not callable",
            "iterator next result must be an object",
        ] {
            assert_eq!(stage.matches(suffix).count(), 1, "{suffix}");
        }
    }
    assert_eq!(map_stage.matches("set method is not callable").count(), 1);
    assert_eq!(
        map_stage
            .matches("iterator value must be an object")
            .count(),
        1
    );
    assert_eq!(set_stage.matches("add method is not callable").count(), 1);

    let emitter = between(
        "    fn emit_collection_algorithm_type_error(",
        "    pub(crate) fn emit_map_constructor(",
    );
    assert_eq!(
        emitter
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert!(!emitter.contains("emit_throw_runtime_error("));

    for (wrapper, expected) in [
        (
            between(
                "    pub(crate) fn emit_weak_map_constructor(",
                "    fn emit_map_collection_constructor(",
            ),
            "emit_map_collection_constructor(MapCollectionKind::WeakMap, function)",
        ),
        (
            between(
                "    pub(crate) fn emit_weak_set_constructor(",
                "    fn emit_set_collection_constructor(",
            ),
            "emit_set_collection_constructor(SetCollectionKind::WeakSet, function)",
        ),
    ] {
        assert_eq!(wrapper.matches(expected).count(), 1, "{expected}");
        assert!(!wrapper.contains("emit_throw_runtime_error("));
    }

    for (constructor, local) in [
        (
            host_between(
                "            &map_meta,\n            &realm_functions,",
                "        let map_group_by_payload_local",
            ),
            "map_constructor_local",
        ),
        (
            host_between(
                "            &set_meta,\n            &realm_functions,",
                "            &regexp_meta,\n            &realm_functions,",
            ),
            "set_constructor_local",
        ),
    ] {
        assert_eq!(
            constructor
                .matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET")
                .count(),
            1,
            "{local} must be self-backed"
        );
        assert_eq!(
            constructor
                .matches("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET")
                .count(),
            1,
            "{local} must retain its Realm TypeError prototype"
        );
        assert!(constructor.matches(local).count() >= 4, "{local}");
    }
}
