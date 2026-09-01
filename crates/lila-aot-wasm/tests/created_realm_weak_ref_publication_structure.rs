const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CREATED_REALM_WEAK_REF_SOURCE: &str =
    include_str!("../src/builtins/host/created_realm_weak_ref_intrinsics.rs");
const WEAK_REF_SOURCE: &str = include_str!("../src/builtins/weak_ref.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const ENTRY_INTRINSICS_SOURCE: &str = include_str!("../src/intrinsics/collections.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/iterator.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_weak_ref_created_realm.js");

fn publication_token() -> &'static str {
    CREATED_REALM_WEAK_REF_SOURCE
        .split_once("pub(super) struct CreatedRealmWeakRefIntrinsics {")
        .expect("created-Realm WeakRef publication token")
        .1
        .split_once("impl<'a> FunctionBuilder<'a> {")
        .expect("created-Realm WeakRef publication token end")
        .0
}

fn materializer() -> &'static str {
    CREATED_REALM_WEAK_REF_SOURCE
        .split_once("    pub(super) fn emit_materialize_created_realm_weak_ref_intrinsics(")
        .expect("created-Realm WeakRef materializer")
        .1
        .split_once("    pub(super) fn emit_publish_created_realm_weak_ref_intrinsics(")
        .expect("created-Realm WeakRef materializer end")
        .0
}

fn publisher() -> &'static str {
    CREATED_REALM_WEAK_REF_SOURCE
        .split_once("    pub(super) fn emit_publish_created_realm_weak_ref_intrinsics(")
        .expect("created-Realm WeakRef publisher")
        .1
        .split_once("\n    }\n}")
        .expect("created-Realm WeakRef publisher end")
        .0
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
fn weak_ref_publication_requires_a_private_one_shot_token() {
    let token = publication_token();
    assert!(CREATED_REALM_WEAK_REF_SOURCE
        .contains("#[must_use = \"created-Realm WeakRef intrinsics must be published\"]"));
    assert_eq!(
        HOST_SOURCE
            .matches("mod created_realm_weak_ref_intrinsics;")
            .count(),
        1
    );
    assert!(!HOST_SOURCE.contains("CreatedRealmWeakRefIntrinsics"));
    assert!(!HOST_SOURCE.contains("created_realm_weak_ref_intrinsics::"));
    assert!(token.contains("prototype_local: u32,"));
    assert!(token.contains("constructor_local: u32,"));
    assert!(!token.contains("derive("));
    for capability in ["Clone", "Copy"] {
        assert!(!CREATED_REALM_WEAK_REF_SOURCE.contains(&format!(
            "impl {capability} for CreatedRealmWeakRefIntrinsics"
        )));
    }
    assert!(!token.contains("pub prototype_local"));
    assert!(!token.contains("pub constructor_local"));
    assert_eq!(
        CREATED_REALM_WEAK_REF_SOURCE
            .matches("CreatedRealmWeakRefIntrinsics")
            .count(),
        5
    );

    let materializer = materializer();
    assert!(materializer.contains(") -> Result<CreatedRealmWeakRefIntrinsics, EmitError> {"));
    assert_eq!(
        materializer
            .matches("Ok(CreatedRealmWeakRefIntrinsics {")
            .count(),
        1
    );

    let publisher = publisher();
    assert!(publisher.contains("intrinsics: CreatedRealmWeakRefIntrinsics,"));
    assert!(publisher.contains("let CreatedRealmWeakRefIntrinsics {"));
    assert!(publisher.contains("self.release_temp_local(constructor_local);"));
    assert!(publisher.contains("self.release_temp_local(prototype_local);"));
    assert_eq!(
        HOST_SOURCE
            .matches(".emit_materialize_created_realm_weak_ref_intrinsics(")
            .count(),
        1
    );
    assert_eq!(
        HOST_SOURCE
            .matches("self.emit_publish_created_realm_weak_ref_intrinsics(")
            .count(),
        1
    );
    assert_eq!(
        CREATED_REALM_WEAK_REF_SOURCE
            .matches("fn emit_materialize_created_realm_weak_ref_intrinsics(")
            .count(),
        1
    );
    assert_eq!(
        CREATED_REALM_WEAK_REF_SOURCE
            .matches("fn emit_publish_created_realm_weak_ref_intrinsics(")
            .count(),
        1
    );
}

#[test]
fn weak_ref_materialization_stores_the_realm_slot_before_exposure() {
    let materializer = materializer();
    assert!(FUNCTIONS_SOURCE.contains("WeakRefPrototype,"));
    assert!(FUNCTIONS_SOURCE
        .contains("Self::WeakRefPrototype => HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET,"));
    assert!(materializer.contains(
        "self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;"
    ));
    assert!(materializer.contains("NonArrayRealmIntrinsicSlot::WeakRefPrototype,"));
    assert!(materializer.contains("realm_record.index(),"));

    let fallback = WEAK_REF_SOURCE
        .split_once("pub(crate) fn emit_weak_ref_constructor(")
        .expect("WeakRef constructor")
        .1
        .split_once("pub(crate) fn emit_weak_ref_deref")
        .expect("WeakRef constructor end")
        .0;
    assert!(fallback.contains(
        "NewTargetPrototypeFallback::RealmIntrinsic(\n                HEAP_REALM_INTRINSICS_WEAK_REF_PROTOTYPE_OFFSET,"
    ));
    assert!(
        ENTRY_INTRINSICS_SOURCE.contains("StandardBuiltinId::WeakRefPrototypeDeref.function_id()")
    );
    assert!(
        ENTRY_INTRINSICS_SOURCE.contains(".property_key_symbol_payload(\"Symbol.toStringTag\")")
    );
}

#[test]
fn weak_ref_callables_capture_identity_and_type_error_realm() {
    let materializer = materializer();
    assert_eq!(
        materializer
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        2
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
    for builtin in ["WeakRefConstructor", "WeakRefPrototypeDeref"] {
        assert_eq!(
            materializer
                .matches(&format!("StandardBuiltinId::{builtin}.function_id()"))
                .count(),
            1,
            "{builtin} metadata"
        );
    }
    assert!(materializer.contains(
        "constructor_local,\n            prototype_local,\n            false,\n            false,\n            false,\n            true,"
    ));
    assert!(materializer.contains(".property_key_symbol_payload(\"Symbol.toStringTag\")"));
    assert!(materializer
        .contains("false,\n            false,\n            true,\n            function,"));

    let publisher = publisher();
    assert!(publisher.contains("global_local,\n            WEAK_REF_NAME,"));
    assert!(publisher.contains("ValueKind::Function.tag()"));
    for message in [
        "WeakRef constructor requires new",
        "WeakRef target cannot be held weakly",
        "WeakRef.prototype.deref receiver does not have [[WeakRefTarget]]",
    ] {
        assert!(WEAK_REF_SOURCE.contains(message), "{message}");
    }
    assert_eq!(
        WEAK_REF_SOURCE
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        3
    );
}

#[test]
fn prototype_constructor_is_defined_before_deref_and_to_string_tag() {
    let materializer = materializer();
    assert_before(
        materializer,
        "&constructor_meta,\n            realm_functions,\n            constructor_local,",
        "self.emit_set_function_prototype_data_with_flags(",
    );
    assert_before(
        materializer,
        "self.emit_set_function_prototype_data_with_flags(",
        "&deref_meta,\n            realm_functions,\n            deref_local,",
    );
    assert_before(
        materializer,
        "&deref_meta,\n            realm_functions,\n            deref_local,",
        ".property_key_symbol_payload(\"Symbol.toStringTag\")",
    );
}

#[test]
fn focused_fixture_covers_created_realm_weak_ref_ownership() {
    for marker in [
        "created WeakRef constructor identity",
        "created WeakRef prototype identity",
        "created WeakRef prototype parent",
        "created WeakRef prototype descriptor",
        "created WeakRef prototype own keys",
        "created WeakRef toStringTag descriptor",
        "created WeakRef instance prototype",
        "foreign NewTarget primitive prototype fallback",
        "created WeakRef requires-new TypeError",
        "created WeakRef invalid-target TypeError",
        "borrowed created WeakRef deref TypeError",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture control: {marker}"
        );
    }
    assert!(CLI_FIXTURE.contains("__lilaCreateRealm"));
    assert!(CLI_FIXTURE.contains("new other.Function()"));
    assert!(CLI_FIXTURE.contains("other.WeakRef = null;"));
    assert!(CLI_FIXTURE.contains("instanceof other.TypeError"));
    assert!(CLI_FIXTURE.contains("!(error instanceof TypeError)"));

    let cli_test = CLI_TESTS
        .split_once("fn run_wasm_backend_succeeds_for_created_realm_weak_ref_publication()")
        .expect("focused created-Realm WeakRef CLI test")
        .1
        .split_once("\n#[test]")
        .expect("test after created-Realm WeakRef CLI test")
        .0;
    assert!(cli_test.contains("wasm_weak_ref_created_realm.js"));
    assert!(cli_test.contains("backend_used: WasmAot"));
    assert!(cli_test.contains("boolean(true)"));
}
