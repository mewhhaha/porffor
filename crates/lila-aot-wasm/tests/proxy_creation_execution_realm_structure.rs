const PROXY_SOURCE: &str = include_str!("../src/builtins/proxy.rs");
const REALM_SOURCE: &str = include_str!("../src/functions/proxy_creation_execution_realm.rs");
const BOUND_FUNCTION_SOURCE: &str = include_str!("../src/functions/bound_function_allocation.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_proxy_creation_execution_realm.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn proxy_creation_realm_is_one_opaque_intrinsic_set() {
    for forbidden in [
        "TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX",
        "OBJECT_PROTOTYPE_GLOBAL_INDEX",
        "FUNCTION_PROTOTYPE_GLOBAL_INDEX",
    ] {
        assert!(
            !REALM_SOURCE.contains(forbidden),
            "realm-owned Proxy products must not select `{forbidden}`"
        );
    }

    let declaration = bounded(
        REALM_SOURCE,
        "#[must_use = \"Proxy creation execution Realm must be explicitly released\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    for marker in [
        "pub(crate) struct ProxyCreationExecutionRealm",
        "realm_local: u32",
        "object_prototype_local: u32",
        "function_prototype_local: u32",
        "type_error_prototype_local: u32",
    ] {
        assert!(
            declaration.contains(marker),
            "missing context field `{marker}`"
        );
    }
    assert!(!declaration.contains("derive("));

    let factory = bounded(
        REALM_SOURCE,
        "pub(crate) fn emit_proxy_creation_execution_realm(",
        "pub(crate) fn emit_throw_proxy_creation_type_error(",
    );
    for marker in [
        "LocalGet(self.current_env_local)",
        "GlobalGet(PROXY_CONSTRUCTOR_GLOBAL_INDEX)",
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "ProxyCreationRealmIntrinsic::ObjectPrototype",
        "ProxyCreationRealmIntrinsic::FunctionPrototype",
        "ProxyCreationRealmIntrinsic::TypeErrorPrototype",
        "Instruction::Unreachable",
    ] {
        assert!(
            factory.contains(marker),
            "missing realm factory marker `{marker}`"
        );
    }
    assert!(!factory.contains("CURRENT_REALM_GLOBAL_INDEX"));

    let release = bounded(
        REALM_SOURCE,
        "pub(crate) fn release_proxy_creation_execution_realm(",
        "\n    }\n}",
    );
    for local in [
        "type_error_prototype_local",
        "function_prototype_local",
        "object_prototype_local",
        "realm_local",
    ] {
        assert!(
            release.contains(&format!("release_temp_local(realm.{local})")),
            "missing release for `{local}`"
        );
    }

    let throw = bounded(
        REALM_SOURCE,
        "pub(crate) fn emit_throw_proxy_creation_type_error(",
        "pub(crate) fn emit_alloc_proxy_revocable_result_object(",
    );
    assert!(throw.contains("realm.type_error_prototype_local"));

    let result_object = bounded(
        REALM_SOURCE,
        "pub(crate) fn emit_alloc_proxy_revocable_result_object(",
        "pub(crate) fn emit_proxy_revoke_target_function(",
    );
    assert!(result_object.contains("Some(realm.object_prototype_local)"));

    let revoke_target = bounded(
        REALM_SOURCE,
        "pub(crate) fn emit_proxy_revoke_target_function(",
        "pub(crate) fn release_proxy_creation_execution_realm(",
    );
    for marker in [
        "emit_store_function_defining_realm(target_payload_local, realm.realm_local",
        "HEAP_PROTOTYPE_OFFSET,\n            realm.function_prototype_local",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,\n            realm.type_error_prototype_local",
        "HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n            target_payload_local",
    ] {
        assert!(
            revoke_target.contains(marker),
            "hidden revoke target is missing `{marker}`"
        );
    }
}

#[test]
fn proxy_creation_consumers_cannot_select_entry_realm_prototypes() {
    let constructor = bounded(
        PROXY_SOURCE,
        "pub(super) fn compile_proxy_constructor_builtin(",
        "pub(super) fn compile_proxy_revocable_builtin(",
    );
    let revocable = bounded(
        PROXY_SOURCE,
        "pub(super) fn compile_proxy_revocable_builtin(",
        "pub(super) fn compile_proxy_revoke_builtin(",
    );
    for body in [constructor, revocable] {
        assert_eq!(
            body.matches("emit_proxy_creation_execution_realm(").count(),
            1
        );
        assert_eq!(
            body.matches("release_proxy_creation_execution_realm(")
                .count(),
            1
        );
        assert!(!body.contains("emit_throw_runtime_error("));
        assert!(!body.contains("TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX"));
        assert!(!body.contains("OBJECT_PROTOTYPE_GLOBAL_INDEX"));
        assert!(!body.contains("FUNCTION_PROTOTYPE_GLOBAL_INDEX"));
    }
    assert_eq!(
        constructor
            .matches("emit_throw_proxy_creation_type_error(")
            .count(),
        2
    );
    assert_eq!(
        revocable
            .matches("emit_throw_proxy_creation_type_error(")
            .count(),
        2
    );
    for marker in [
        "emit_alloc_proxy_revocable_result_object(\n            &execution_realm",
        "emit_proxy_revoke_target_function(\n            &execution_realm",
        "proxy_payload_local,\n            &execution_realm,",
    ] {
        assert!(
            revocable.contains(marker),
            "missing revocable marker `{marker}`"
        );
    }

    let exact_source = bounded(
        BOUND_FUNCTION_SOURCE,
        "enum ExactBoundThisSource<'realm>",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert!(exact_source.contains("realm: &'realm ProxyCreationExecutionRealm"));
    let allocator = bounded(
        BOUND_FUNCTION_SOURCE,
        "fn emit_alloc_bound_function_from_exact_source(",
        "fn emit_alloc_bound_function_value(",
    );
    assert!(allocator.contains("LocalGet(realm.function_prototype_local)"));
    assert!(allocator.contains("internal_prototype_local"));
}

#[test]
fn created_realm_publication_and_behavior_witness_cover_all_products() {
    let proxy_publication = bounded(
        HOST_SOURCE,
        "self.emit_function_value_payload_in_realm(\n            &proxy_meta,",
        "self.emit_function_value_payload_in_realm(\n            &map_meta,",
    );
    for local in ["proxy_constructor_local", "revocable_payload_local"] {
        let self_backing =
            format!("{local},\n            HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n            {local},");
        assert!(
            proxy_publication.contains(&self_backing),
            "created-realm `{local}` is not self-backed"
        );
    }

    assert!(CLI_TESTS.contains("fn proxy_creation_uses_the_builtin_execution_realm()"));
    assert!(CLI_TESTS.contains("wasm_proxy_creation_execution_realm.js"));
    assert!(
        !CLI_FIXTURE.contains("evalScript"),
        "the Proxy fixture must not seed host-owned createRealm property names"
    );
    for marker in [
        "new OtherProxy(0, {})",
        "new OtherProxy({}, 0)",
        "otherRevocable(0, {})",
        "otherRevocable({}, 0)",
        "Object.getPrototypeOf(revocable) === other.Object.prototype",
        "Object.getPrototypeOf(revocable.revoke) === other.Function.prototype",
        "revocable.revoke();\nrevocable.revoke();",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker `{marker}`"
        );
    }
}
