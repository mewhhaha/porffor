const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const OWNER_SOURCE: &str =
    include_str!("../src/builtins/host/created_realm_weak_collection_intrinsics.rs");
const COLLECTIONS_SOURCE: &str = include_str!("../src/builtins/collections.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const ENTRY_INTRINSICS_SOURCE: &str = include_str!("../src/intrinsics/collections.rs");
const CATALOG_SOURCE: &str = include_str!("../../lila-ir/src/builtins/catalog.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/iterator.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_weak_collections_created_realm.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn materializer() -> &'static str {
    bounded(
        OWNER_SOURCE,
        "    pub(super) fn emit_materialize_created_realm_weak_collection_intrinsics(",
        "    pub(super) fn emit_publish_created_realm_weak_collection_intrinsics(",
    )
}

fn publisher() -> &'static str {
    OWNER_SOURCE
        .split_once("    pub(super) fn emit_publish_created_realm_weak_collection_intrinsics(")
        .expect("created-Realm weak collection publisher")
        .1
        .split_once("\n    }\n}")
        .expect("created-Realm weak collection publisher end")
        .0
}

fn create_realm_host() -> &'static str {
    bounded(
        HOST_SOURCE,
        "    pub(crate) fn compile_host_create_realm_builtin(",
        "    /// Defensive body for the Test262 realm-evaluation capability.",
    )
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn identifier_offsets(source: &str, identifier: &str) -> Vec<usize> {
    source
        .match_indices(identifier)
        .filter_map(|(offset, _)| {
            let boundary = source.as_bytes().get(offset + identifier.len());
            match boundary {
                Some(byte) if byte.is_ascii_alphanumeric() || *byte == b'_' => None,
                _ => Some(offset),
            }
        })
        .collect()
}

#[test]
fn weak_collection_publication_requires_one_private_consumed_token() {
    let token = bounded(
        OWNER_SOURCE,
        "pub(super) struct CreatedRealmWeakCollectionIntrinsics {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert!(OWNER_SOURCE
        .contains("#[must_use = \"created-Realm weak collection intrinsics must be published\"]"));
    assert_eq!(
        HOST_SOURCE
            .matches("mod created_realm_weak_collection_intrinsics;")
            .count(),
        1
    );
    assert!(!HOST_SOURCE.contains("CreatedRealmWeakCollectionIntrinsics"));
    for field in [
        "weak_map_prototype_local: u32,",
        "weak_map_constructor_local: u32,",
        "weak_set_prototype_local: u32,",
        "weak_set_constructor_local: u32,",
    ] {
        assert!(
            token.contains(field),
            "missing private token field `{field}`"
        );
        assert!(!token.contains(&format!("pub {field}")));
    }
    assert!(!token.contains("derive("));
    assert_eq!(
        OWNER_SOURCE
            .matches("CreatedRealmWeakCollectionIntrinsics")
            .count(),
        5
    );

    let materializer = materializer();
    assert!(materializer.contains(") -> Result<CreatedRealmWeakCollectionIntrinsics, EmitError> {"));
    assert_eq!(
        materializer
            .matches("Ok(CreatedRealmWeakCollectionIntrinsics {")
            .count(),
        1
    );

    let publisher = publisher();
    assert!(publisher.contains("intrinsics: CreatedRealmWeakCollectionIntrinsics,"));
    assert!(publisher.contains("let CreatedRealmWeakCollectionIntrinsics {"));
    assert_eq!(
        HOST_SOURCE
            .matches(".emit_materialize_created_realm_weak_collection_intrinsics(")
            .count(),
        1
    );
    assert_eq!(
        HOST_SOURCE
            .matches("self.emit_publish_created_realm_weak_collection_intrinsics(")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("fn emit_materialize_created_realm_weak_collection_intrinsics(")
            .count(),
        1
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("fn emit_publish_created_realm_weak_collection_intrinsics(")
            .count(),
        1
    );
}

#[test]
fn weak_collection_owner_preserves_internal_lifo_and_realm_slots() {
    let materializer = materializer();
    assert_before(
        materializer,
        "let weak_set_prototype_local = self.reserve_temp_local();",
        "let weak_set_constructor_local = self.reserve_temp_local();",
    );
    assert_before(
        materializer,
        "let weak_set_constructor_local = self.reserve_temp_local();",
        "let weak_map_prototype_local = self.reserve_temp_local();",
    );
    assert_before(
        materializer,
        "let weak_map_prototype_local = self.reserve_temp_local();",
        "let weak_map_constructor_local = self.reserve_temp_local();",
    );

    let publisher = publisher();
    assert_before(
        publisher,
        "self.release_temp_local(weak_map_constructor_local);",
        "self.release_temp_local(weak_map_prototype_local);",
    );
    assert_before(
        publisher,
        "self.release_temp_local(weak_map_prototype_local);",
        "self.release_temp_local(weak_set_constructor_local);",
    );
    assert_before(
        publisher,
        "self.release_temp_local(weak_set_constructor_local);",
        "self.release_temp_local(weak_set_prototype_local);",
    );

    for (start, end, slot_call, slot_mapping) in [
        (
            "let weak_set_prototype_local = self.reserve_temp_local();",
            "let weak_map_prototype_local = self.reserve_temp_local();",
            "weak_set_intrinsic.realm_slot(),",
            "Self::WeakSet => NonArrayRealmIntrinsicSlot::WeakSetPrototype,",
        ),
        (
            "let weak_map_prototype_local = self.reserve_temp_local();",
            "Ok(CreatedRealmWeakCollectionIntrinsics {",
            "weak_map_intrinsic.realm_slot(),",
            "Self::WeakMap => NonArrayRealmIntrinsicSlot::WeakMapPrototype,",
        ),
    ] {
        let region = bounded(materializer, start, end);
        assert!(region.contains(
            "self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;"
        ));
        assert!(region.contains(slot_call));
        assert!(region.contains("realm_record.index(),"));
        assert_before(
            region,
            "self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;",
            slot_call,
        );
        assert_before(region, slot_call, "for (name, builtin) in [");
        assert!(ENTRY_INTRINSICS_SOURCE.contains(slot_mapping));
    }

    for marker in [
        "WeakMapPrototype,",
        "WeakSetPrototype,",
        "Self::WeakMapPrototype => HEAP_REALM_INTRINSICS_WEAK_MAP_PROTOTYPE_OFFSET,",
        "Self::WeakSetPrototype => HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET,",
    ] {
        assert!(
            FUNCTIONS_SOURCE.contains(marker),
            "missing realm slot `{marker}`"
        );
    }
    assert!(COLLECTIONS_SOURCE
        .contains("Self::WeakMap => HEAP_REALM_INTRINSICS_WEAK_MAP_PROTOTYPE_OFFSET,"));
    assert!(COLLECTIONS_SOURCE
        .contains("Self::WeakSet => HEAP_REALM_INTRINSICS_WEAK_SET_PROTOTYPE_OFFSET,"));
    assert_eq!(
        COLLECTIONS_SOURCE
            .matches(
                "NewTargetPrototypeFallback::RealmIntrinsic(collection_kind.realm_prototype_offset()),"
            )
            .count(),
        2,
        "WeakMap and WeakSet constructors must both use private Realm slots"
    );
}

#[test]
fn created_and_entry_realms_publish_the_same_weak_collection_methods() {
    let materializer = materializer();
    let created_weak_set = bounded(
        materializer,
        "let weak_set_prototype_local = self.reserve_temp_local();",
        "let weak_map_prototype_local = self.reserve_temp_local();",
    );
    let created_weak_map = bounded(
        materializer,
        "let weak_map_prototype_local = self.reserve_temp_local();",
        "Ok(CreatedRealmWeakCollectionIntrinsics {",
    );
    let entry_weak_map = bounded(
        ENTRY_INTRINSICS_SOURCE,
        "    pub(crate) fn install_weak_map_constructor_intrinsics(",
        "    pub(crate) fn install_weak_set_constructor_intrinsics(",
    );
    let entry_weak_set = bounded(
        ENTRY_INTRINSICS_SOURCE,
        "    pub(crate) fn install_weak_set_constructor_intrinsics(",
        "    pub(crate) fn install_weak_ref_constructor_intrinsics(",
    );

    for (created, entry, builtins) in [
        (
            created_weak_map,
            entry_weak_map,
            &[
                "WeakMapPrototypeDelete",
                "WeakMapPrototypeGet",
                "WeakMapPrototypeGetOrInsert",
                "WeakMapPrototypeGetOrInsertComputed",
                "WeakMapPrototypeHas",
                "WeakMapPrototypeSet",
            ][..],
        ),
        (
            created_weak_set,
            entry_weak_set,
            &[
                "WeakSetPrototypeAdd",
                "WeakSetPrototypeDelete",
                "WeakSetPrototypeHas",
            ][..],
        ),
    ] {
        let mut previous_created_offset = 0;
        let mut previous_entry_offset = 0;
        for builtin in builtins {
            let marker = format!("StandardBuiltinId::{builtin}");
            let created_offsets = identifier_offsets(created, &marker);
            let entry_offsets = identifier_offsets(entry, &marker);
            assert_eq!(created_offsets.len(), 1, "created {builtin}");
            assert_eq!(entry_offsets.len(), 1, "entry {builtin}");
            let created_offset = created_offsets[0];
            let entry_offset = entry_offsets[0];
            assert!(
                created_offset >= previous_created_offset,
                "created {builtin} order"
            );
            assert!(
                entry_offset >= previous_entry_offset,
                "entry {builtin} order"
            );
            previous_created_offset = created_offset;
            previous_entry_offset = entry_offset;
        }
    }

    for builtin in ["WeakMapConstructor", "WeakSetConstructor"] {
        assert_eq!(
            materializer
                .matches(&format!("StandardBuiltinId::{builtin}.function_id()"))
                .count(),
            1,
            "{builtin} metadata"
        );
    }
    assert_eq!(
        materializer
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        4,
        "one method loop and one constructor site per collection"
    );
    assert_eq!(
        materializer
            .matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET")
            .count(),
        4
    );
    assert_eq!(
        materializer
            .matches("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET")
            .count(),
        4
    );
    assert_eq!(
        materializer
            .matches("self.emit_set_function_prototype_data_with_flags(")
            .count(),
        2
    );
    assert!(
        OWNER_SOURCE.contains("use crate::intrinsics::collections::CollectionPrototypeIntrinsic;")
    );
    assert_eq!(
        materializer
            .matches("CollectionPrototypeIntrinsic::WeakMap")
            .count(),
        1
    );
    assert_eq!(
        materializer
            .matches("CollectionPrototypeIntrinsic::WeakSet")
            .count(),
        1
    );
    assert_eq!(
        materializer
            .matches("self.emit_collection_prototype_to_string_tag(")
            .count(),
        2
    );
    assert!(!OWNER_SOURCE.contains("property_key_symbol_payload(\"Symbol.toStringTag\")"));
    assert!(!OWNER_SOURCE.contains("emit_object_append_data_property_with_flags"));
    let shared_to_string_tag = bounded(
        ENTRY_INTRINSICS_SOURCE,
        "    pub(crate) fn emit_collection_prototype_to_string_tag(",
        "    pub(crate) fn install_map_constructor_intrinsics(",
    );
    assert!(shared_to_string_tag.contains(".property_key_symbol_payload(\"Symbol.toStringTag\")"));
    assert!(shared_to_string_tag.contains("self.emit_object_append_data_property_with_flags("));

    let publisher = publisher();
    assert!(publisher.contains("global_local,\n            WEAK_MAP_NAME,"));
    assert!(publisher.contains("global_local,\n            WEAK_SET_NAME,"));

    let catalog = bounded(
        CATALOG_SOURCE,
        "    MapPrototypeSizeGetter {",
        "    WeakRefConstructor {",
    );
    for constructor in ["WeakMapConstructor", "WeakSetConstructor"] {
        let entry = catalog
            .split_once(&format!("    {constructor} {{"))
            .unwrap_or_else(|| panic!("missing catalog constructor `{constructor}`"))
            .1
            .split_once("    }")
            .expect("catalog constructor end")
            .0;
        assert!(entry.contains("flags: [CONSTRUCTABLE],"));
    }
    for method in [
        "WeakMapPrototypeDelete",
        "WeakMapPrototypeGet",
        "WeakMapPrototypeGetOrInsert",
        "WeakMapPrototypeGetOrInsertComputed",
        "WeakMapPrototypeHas",
        "WeakMapPrototypeSet",
        "WeakSetPrototypeAdd",
        "WeakSetPrototypeDelete",
        "WeakSetPrototypeHas",
    ] {
        let entry = catalog
            .split_once(&format!("    {method} {{"))
            .unwrap_or_else(|| panic!("missing catalog method `{method}`"))
            .1
            .split_once("    }")
            .expect("catalog method end")
            .0;
        assert!(
            !entry.contains("CONSTRUCTABLE"),
            "{method} must not construct"
        );
    }
}

#[test]
fn prototype_constructor_is_defined_before_methods_and_to_string_tag() {
    let materializer = materializer();
    for (start, end) in [
        (
            "let weak_set_prototype_local = self.reserve_temp_local();",
            "let weak_map_prototype_local = self.reserve_temp_local();",
        ),
        (
            "let weak_map_prototype_local = self.reserve_temp_local();",
            "Ok(CreatedRealmWeakCollectionIntrinsics {",
        ),
    ] {
        let region = bounded(materializer, start, end);
        assert_before(
            region,
            "self.emit_set_function_prototype_data_with_flags(",
            "for (name, builtin) in [",
        );
        assert_before(
            region,
            "for (name, builtin) in [",
            "self.emit_collection_prototype_to_string_tag(",
        );
    }
}

#[test]
fn host_nests_weak_collection_publication_inside_existing_tokens() {
    let create_realm = create_realm_host();
    for (earlier, later) in [
        (
            ".emit_materialize_created_realm_finalization_registry_intrinsics(",
            ".emit_materialize_created_realm_weak_ref_intrinsics(",
        ),
        (
            ".emit_materialize_created_realm_weak_ref_intrinsics(",
            ".emit_materialize_created_realm_weak_collection_intrinsics(",
        ),
        (
            "self.emit_publish_created_realm_weak_collection_intrinsics(",
            "self.emit_publish_created_realm_weak_ref_intrinsics(",
        ),
        (
            "self.emit_publish_created_realm_weak_ref_intrinsics(",
            "self.emit_publish_created_realm_finalization_registry_intrinsics(",
        ),
    ] {
        assert_before(create_realm, earlier, later);
    }

    let materialize = create_realm
        .find(".emit_materialize_created_realm_weak_collection_intrinsics(")
        .expect("created-Realm weak collection materialization");
    let global_allocation = create_realm
        .find(concat!(
            "self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;\n",
            "        function.instruction(&Instruction::LocalSet(global_local));"
        ))
        .expect("created-Realm global allocation");
    let publish = create_realm
        .find("self.emit_publish_created_realm_weak_collection_intrinsics(")
        .expect("created-Realm weak collection publication");
    assert!(materialize < global_allocation);
    assert!(global_allocation < publish);

    for (earlier, later) in [
        (
            "global_local,\n            MAP_NAME,",
            "self.emit_publish_created_realm_weak_collection_intrinsics(",
        ),
        (
            "self.emit_publish_created_realm_weak_collection_intrinsics(",
            "self.emit_publish_created_realm_weak_ref_intrinsics(",
        ),
        (
            "self.emit_publish_created_realm_weak_ref_intrinsics(",
            "self.emit_publish_created_realm_finalization_registry_intrinsics(",
        ),
        (
            "self.emit_publish_created_realm_finalization_registry_intrinsics(",
            "global_local,\n            SET_NAME,",
        ),
    ] {
        assert_before(create_realm, earlier, later);
    }
    assert_before(publisher(), "WEAK_MAP_NAME,", "WEAK_SET_NAME,");
}

#[test]
fn focused_fixture_covers_created_realm_weak_collection_ownership() {
    for marker in [
        "created WeakMap constructor identity",
        "created WeakMap prototype identity",
        "created WeakSet constructor identity",
        "created WeakSet prototype identity",
        "created weak collection global catalog order",
        "created WeakMap global descriptor",
        "created WeakSet global descriptor",
        "created WeakMap IsConstructor",
        "created WeakSet IsConstructor",
        "created WeakMap constructor-before-method own-key order",
        "created WeakSet constructor-before-method own-key order",
        "created WeakMap object-iterable construction",
        "created WeakSet object-iterable construction",
        "entry WeakMap method accepts created instance",
        "created WeakMap method accepts entry instance",
        "entry WeakSet method accepts created instance",
        "created WeakSet method accepts entry instance",
        "created WeakMap requires-new TypeError",
        "created WeakSet requires-new TypeError",
        "created WeakMap invalid-key TypeError",
        "created WeakSet invalid-value TypeError",
        "borrowed created WeakMap method TypeError",
        "borrowed created WeakSet method TypeError",
        "private-slot foreign NewTarget primitive WeakMap fallback",
        "private-slot foreign NewTarget primitive WeakSet fallback",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture control: {marker}"
        );
    }
    assert!(CLI_FIXTURE.contains(concat!(
        "var weakMapMethodNames = [\n",
        "  \"delete\",\n",
        "  \"get\",\n",
        "  \"getOrInsert\",\n",
        "  \"getOrInsertComputed\",\n",
        "  \"has\",\n",
        "  \"set\",\n",
        "];\n",
        "var weakMapMethodLengths = [1, 1, 2, 2, 1, 2];"
    )));
    assert!(CLI_FIXTURE.contains("var weakSetMethodNames = [\"add\", \"delete\", \"has\"];"));
    assert!(CLI_FIXTURE.contains("var weakSetMethodLengths = [1, 1, 1];"));
    assert!(CLI_FIXTURE.contains("!isConstructor(weakMapMethod)"));
    assert!(CLI_FIXTURE.contains("!isConstructor(weakSetMethod)"));
    assert!(CLI_FIXTURE.contains("var foreignNewTarget = other.Object.bind(null);"));
    assert_before(
        CLI_FIXTURE,
        "other.WeakMap = null;",
        "Reflect.construct(WeakMap,",
    );
    assert_before(
        CLI_FIXTURE,
        "other.WeakSet = null;",
        "Reflect.construct(WeakSet,",
    );
    assert!(CLI_FIXTURE.contains(concat!(
        "var primitivePrototypes = [\n",
        "  undefined,\n",
        "  null,\n",
        "  true,\n",
        "  \"\",\n",
        "  Symbol(\"prototype\"),\n",
        "  -1,\n",
        "  0n,\n",
        "];"
    )));
    assert!(!CLI_FIXTURE.contains("evalScript"));
    assert!(!CLI_FIXTURE.contains("new other.Function"));

    let cli_test = CLI_TESTS
        .split_once("fn run_wasm_backend_succeeds_for_created_realm_weak_collection_publication()")
        .expect("focused created-Realm weak collection CLI test")
        .1
        .split_once("\n#[test]")
        .expect("test after created-Realm weak collection CLI test")
        .0;
    assert!(cli_test.contains("wasm_weak_collections_created_realm.js"));
    assert!(cli_test.contains("backend_used: WasmAot"));
    assert!(cli_test.contains("boolean(true)"));
}
