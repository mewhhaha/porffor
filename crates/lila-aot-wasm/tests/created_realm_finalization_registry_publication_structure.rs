const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CREATED_REALM_FINALIZATION_REGISTRY_SOURCE: &str =
    include_str!("../src/builtins/host/created_realm_finalization_registry_intrinsics.rs");
const FINALIZATION_REGISTRY_SOURCE: &str = include_str!("../src/builtins/finalization_registry.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const ENTRY_INTRINSICS_SOURCE: &str = include_str!("../src/intrinsics/collections.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/iterator.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_finalization_registry_created_realm.js");

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
        CREATED_REALM_FINALIZATION_REGISTRY_SOURCE,
        "    pub(super) fn emit_materialize_created_realm_finalization_registry_intrinsics(",
        "    pub(super) fn emit_publish_created_realm_finalization_registry_intrinsics(",
    )
}

fn publisher() -> &'static str {
    CREATED_REALM_FINALIZATION_REGISTRY_SOURCE
        .split_once(
            "    pub(super) fn emit_publish_created_realm_finalization_registry_intrinsics(",
        )
        .expect("created-Realm FinalizationRegistry publisher")
        .1
        .split_once("\n    }\n}")
        .expect("created-Realm FinalizationRegistry publisher end")
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

#[test]
fn finalization_registry_publication_requires_a_private_one_shot_token() {
    let token = bounded(
        CREATED_REALM_FINALIZATION_REGISTRY_SOURCE,
        "pub(super) struct CreatedRealmFinalizationRegistryIntrinsics {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert!(CREATED_REALM_FINALIZATION_REGISTRY_SOURCE.contains(
        "#[must_use = \"created-Realm FinalizationRegistry intrinsics must be published\"]"
    ));
    assert_eq!(
        HOST_SOURCE
            .matches("mod created_realm_finalization_registry_intrinsics;")
            .count(),
        1
    );
    assert!(!HOST_SOURCE.contains("CreatedRealmFinalizationRegistryIntrinsics"));
    assert!(token.contains("prototype_local: u32,"));
    assert!(token.contains("constructor_local: u32,"));
    assert!(!token.contains("derive("));
    for capability in ["Clone", "Copy"] {
        assert!(
            !CREATED_REALM_FINALIZATION_REGISTRY_SOURCE.contains(&format!(
                "impl {capability} for CreatedRealmFinalizationRegistryIntrinsics"
            ))
        );
    }
    assert!(!token.contains("pub prototype_local"));
    assert!(!token.contains("pub constructor_local"));
    assert_eq!(
        CREATED_REALM_FINALIZATION_REGISTRY_SOURCE
            .matches("CreatedRealmFinalizationRegistryIntrinsics")
            .count(),
        5
    );

    let materializer = materializer();
    assert!(materializer
        .contains(") -> Result<CreatedRealmFinalizationRegistryIntrinsics, EmitError> {"));
    assert_eq!(
        materializer
            .matches("Ok(CreatedRealmFinalizationRegistryIntrinsics {")
            .count(),
        1
    );

    let publisher = publisher();
    assert!(publisher.contains("intrinsics: CreatedRealmFinalizationRegistryIntrinsics,"));
    assert!(publisher.contains("let CreatedRealmFinalizationRegistryIntrinsics {"));
    assert!(publisher.contains("self.release_temp_local(constructor_local);"));
    assert!(publisher.contains("self.release_temp_local(prototype_local);"));
    assert_eq!(
        HOST_SOURCE
            .matches(".emit_materialize_created_realm_finalization_registry_intrinsics(")
            .count(),
        1
    );
    assert_eq!(
        HOST_SOURCE
            .matches("self.emit_publish_created_realm_finalization_registry_intrinsics(")
            .count(),
        1
    );
}

#[test]
fn materialization_stores_the_realm_prototype_before_exposure() {
    let materializer = materializer();
    assert!(FUNCTIONS_SOURCE.contains("FinalizationRegistryPrototype,"));
    assert!(FUNCTIONS_SOURCE.contains(concat!(
        "Self::FinalizationRegistryPrototype => {\n",
        "                HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET"
    )));
    assert!(materializer.contains(
        "self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;"
    ));
    assert!(materializer.contains("NonArrayRealmIntrinsicSlot::FinalizationRegistryPrototype,"));
    assert!(materializer.contains("realm_record.index(),"));
    assert_before(
        materializer,
        "self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;",
        "NonArrayRealmIntrinsicSlot::FinalizationRegistryPrototype,",
    );
    assert_before(
        materializer,
        "NonArrayRealmIntrinsicSlot::FinalizationRegistryPrototype,",
        "self.emit_function_value_payload_in_realm(",
    );

    let constructor = bounded(
        FINALIZATION_REGISTRY_SOURCE,
        "    pub(crate) fn emit_finalization_registry_constructor(",
        "    pub(crate) fn emit_finalization_registry_register(",
    );
    assert!(constructor.contains(concat!(
        "NewTargetPrototypeFallback::RealmIntrinsic(\n",
        "                HEAP_REALM_INTRINSICS_FINALIZATION_REGISTRY_PROTOTYPE_OFFSET,"
    )));
}

#[test]
fn realm_local_callables_capture_identity_and_type_error_authority() {
    let materializer = materializer();
    assert_eq!(
        materializer
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        2,
        "one loop site materializes both methods and one site materializes the constructor"
    );
    assert_eq!(
        materializer
            .matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET")
            .count(),
        2
    );
    assert_eq!(
        materializer
            .matches("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET")
            .count(),
        2
    );
    for builtin in [
        "FinalizationRegistryConstructor",
        "FinalizationRegistryPrototypeRegister",
        "FinalizationRegistryPrototypeUnregister",
    ] {
        assert_eq!(
            materializer
                .matches(&format!("StandardBuiltinId::{builtin}"))
                .count(),
            1,
            "{builtin} metadata"
        );
    }
    assert!(materializer.contains(
        "constructor_local,\n            prototype_local,\n            false,\n            false,\n            false,\n            true,"
    ));
    assert!(materializer.contains(".property_key_symbol_payload(\"Symbol.toStringTag\")"));
    assert!(materializer.contains("self.strings.payload(FINALIZATION_REGISTRY_NAME)"));

    let publisher = publisher();
    assert!(publisher.contains("global_local,\n            FINALIZATION_REGISTRY_NAME,"));
    assert!(publisher.contains("ValueKind::Function.tag()"));

    let entry_installer = bounded(
        ENTRY_INTRINSICS_SOURCE,
        "    pub(crate) fn install_finalization_registry_constructor_intrinsics(",
        "    /// `%AsyncDisposableStack.prototype%",
    );
    for builtin in [
        "FinalizationRegistryPrototypeRegister",
        "FinalizationRegistryPrototypeUnregister",
    ] {
        assert!(entry_installer.contains(&format!("StandardBuiltinId::{builtin}")));
    }
    assert!(entry_installer.contains(".property_key_symbol_payload(\"Symbol.toStringTag\")"));
}

#[test]
fn prototype_constructor_is_defined_before_methods_and_to_string_tag() {
    let materializer = materializer();
    assert_before(
        materializer,
        "&constructor_meta,\n            realm_functions,\n            constructor_local,",
        "self.emit_set_function_prototype_data_with_flags(",
    );
    assert_before(
        materializer,
        "self.emit_set_function_prototype_data_with_flags(",
        "for (name, builtin) in [",
    );
    assert_before(
        materializer,
        "for (name, builtin) in [",
        ".property_key_symbol_payload(\"Symbol.toStringTag\")",
    );
}

#[test]
fn host_publishes_materialized_intrinsics_only_after_the_global_object_exists() {
    let create_realm_host = create_realm_host();
    let materialize = create_realm_host
        .find(".emit_materialize_created_realm_finalization_registry_intrinsics(")
        .expect("created-Realm FinalizationRegistry materialization");
    let global_allocation = create_realm_host
        .find(concat!(
            "self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;\n",
            "        function.instruction(&Instruction::LocalSet(global_local));"
        ))
        .expect("created-Realm global allocation");
    let publish = create_realm_host
        .find("self.emit_publish_created_realm_finalization_registry_intrinsics(")
        .expect("created-Realm FinalizationRegistry publication");
    assert!(materialize < global_allocation);
    assert!(global_allocation < publish);
}

#[test]
fn host_materializes_nested_tokens_in_reverse_publication_order() {
    let create_realm_host = create_realm_host();
    assert_before(
        create_realm_host,
        ".emit_materialize_created_realm_finalization_registry_intrinsics(",
        ".emit_materialize_created_realm_weak_ref_intrinsics(",
    );
    assert_before(
        create_realm_host,
        "self.emit_publish_created_realm_weak_ref_intrinsics(",
        "self.emit_publish_created_realm_finalization_registry_intrinsics(",
    );
}

#[test]
fn focused_fixture_covers_created_realm_finalization_registry_ownership() {
    for marker in [
        "created FinalizationRegistry constructor identity",
        "created FinalizationRegistry prototype identity",
        "created FinalizationRegistry register identity",
        "created FinalizationRegistry unregister identity",
        "created FinalizationRegistry global descriptor",
        "created FinalizationRegistry prototype descriptor",
        "created FinalizationRegistry prototype own keys",
        "created FinalizationRegistry register descriptor",
        "created FinalizationRegistry unregister descriptor",
        "created FinalizationRegistry toStringTag descriptor",
        "created FinalizationRegistry instance prototype",
        "created FinalizationRegistry register result",
        "created FinalizationRegistry unregister match",
        "created FinalizationRegistry unregister miss",
        "created register accepts entry FinalizationRegistry",
        "created unregister accepts entry FinalizationRegistry",
        "entry register accepts created FinalizationRegistry",
        "entry unregister accepts created FinalizationRegistry",
        "created FinalizationRegistry requires-new TypeError",
        "created FinalizationRegistry cleanup-callback TypeError",
        "borrowed created FinalizationRegistry register TypeError",
        "borrowed created FinalizationRegistry unregister TypeError",
        "foreign NewTarget private-slot FinalizationRegistry fallback",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture control: {marker}"
        );
    }
    assert!(CLI_FIXTURE.contains("__lilaCreateRealm"));
    assert!(CLI_FIXTURE.contains("other.Object.bind(null)"));
    assert!(CLI_FIXTURE.contains("other.FinalizationRegistry = null;"));
    assert!(CLI_FIXTURE.contains("thrown instanceof other.TypeError"));
    assert!(CLI_FIXTURE.contains("!(thrown instanceof TypeError)"));
    assert!(!CLI_FIXTURE.contains("evalScript"));
    assert!(!CLI_FIXTURE.contains("new other.Function"));

    let cli_test = CLI_TESTS
        .split_once(
            "fn run_wasm_backend_succeeds_for_created_realm_finalization_registry_publication()",
        )
        .expect("focused created-Realm FinalizationRegistry CLI test")
        .1
        .split_once("\n#[test]")
        .expect("test after created-Realm FinalizationRegistry CLI test")
        .0;
    assert!(cli_test.contains("wasm_finalization_registry_created_realm.js"));
    assert!(cli_test.contains("backend_used: WasmAot"));
    assert!(cli_test.contains("boolean(true)"));
}
