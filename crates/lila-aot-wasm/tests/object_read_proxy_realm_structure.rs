const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_object_prototype_to_string_proxy_array.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn object_read_realm_source_exhaustively_projects_every_helper_body() {
    let source_domain = bounded(
        EMIT_SOURCE,
        "pub(crate) enum ObjectReadErrorRealmSource {",
        "pub(crate) enum OrdinarySetDataOnReceiverEmission",
    );
    for state in [
        "GlobalFallback",
        "StandardBuiltinEnvironment",
        "ObjectReadHelperArgument",
        "ProxyDispatchHelperArgument",
    ] {
        assert!(
            source_domain.contains(state),
            "missing Realm source state: {state}"
        );
    }
    let source_projection = bounded(source_domain, "impl ObjectReadErrorRealmSource {", "\n}\n");
    let helper_projection = bounded(
        source_projection,
        "pub(crate) const fn for_runtime_helper(helper: RuntimeHelperId) -> Self {",
        "\n    }",
    );
    assert_eq!(
        helper_projection
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>()
            .matches("RuntimeHelperId::ObjectRead|RuntimeHelperId::ObjectReadProxy|RuntimeHelperId::IndexedElementRead=>Self::ObjectReadHelperArgument,")
            .count(),
        1
    );
    assert_eq!(
        helper_projection
            .matches("Self::ObjectReadHelperArgument")
            .count(),
        1
    );
    assert_eq!(
        helper_projection
            .matches("RuntimeHelperId::ProxyCall | RuntimeHelperId::ProxyConstruct")
            .count(),
        1
    );
    assert_eq!(
        helper_projection
            .matches("Self::ProxyDispatchHelperArgument")
            .count(),
        1
    );
    assert!(!helper_projection.contains("_ =>"));

    let helper_entry = bounded(
        EMIT_SOURCE,
        "pub(crate) fn begin_helper_body(&mut self, helper: RuntimeHelperId) -> Function {",
        "Function::new_with_locals_types",
    );
    assert_eq!(
        helper_entry
            .matches("ObjectReadErrorRealmSource::for_runtime_helper(helper)")
            .count(),
        1
    );
}

#[test]
fn outlined_and_inline_proxy_reads_consume_only_the_typed_realm_projection() {
    for projection in [
        "fn outlined_object_read_realm_argument(",
        "fn object_read_revocation_error_realm(",
    ] {
        let body = bounded(OBJECTS_SOURCE, projection, "\n}\n");
        for state in [
            "ObjectReadErrorRealmSource::GlobalFallback",
            "ObjectReadErrorRealmSource::StandardBuiltinEnvironment",
            "ObjectReadErrorRealmSource::ObjectReadHelperArgument",
            "ObjectReadErrorRealmSource::ProxyDispatchHelperArgument",
        ] {
            assert_eq!(body.matches(state).count(), 1, "{projection}: {state}");
        }
        assert!(!body.contains("_ =>"));
    }

    for (start, end) in [
        (
            "fn compile_object_read_helper(&mut self)",
            "fn compile_object_write_helper(&mut self)",
        ),
        (
            "fn compile_object_read_proxy_helper(&mut self)",
            "// `compile_indexed_element_read_helper`",
        ),
    ] {
        let helper_body = bounded(EMIT_SOURCE, start, end);
        assert!(helper_body.contains("Instruction::LocalGet(6)"));
        assert!(helper_body.contains("Instruction::LocalSet(self.current_env_local)"));
    }

    let object_read = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_read_with_key_tag(",
        "pub(crate) fn emit_object_read_ordinary(",
    );
    assert_eq!(
        object_read
            .matches("self.emit_outlined_object_read_realm_argument(function);")
            .count(),
        1
    );
    assert_eq!(
        object_read
            .matches("object_read_revocation_error_realm(self.object_read_error_realm_source())")
            .count(),
        1
    );
    let revoked_proxy = bounded(
        object_read,
        "match object_read_revocation_error_realm(self.object_read_error_realm_source())",
        "self.emit_return_current_completion(function);",
    );
    assert_eq!(
        revoked_proxy.matches("\"Proxy handler is null\"").count(),
        2
    );
    assert_eq!(
        revoked_proxy
            .matches("emit_throw_current_function_realm_type_error(")
            .count(),
        1
    );
    assert_eq!(
        revoked_proxy.matches("emit_throw_runtime_error(").count(),
        1
    );

    let non_propagating_read = bounded(
        OBJECTS_SOURCE,
        "fn emit_object_read_without_throw_propagation_inner(",
        "pub(crate) fn emit_object_read_with_key_tag(",
    );
    assert_eq!(
        non_propagating_read
            .matches("self.emit_outlined_object_read_realm_argument(function);")
            .count(),
        1
    );
}

#[test]
fn borrowed_created_realm_array_to_string_pins_revoked_get_error_realm() {
    for marker in [
        "other.Array.prototype.toString.call(revocable.proxy)",
        "Object.getPrototypeOf(otherArrayToStringError) === other.TypeError.prototype",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(CLI_TESTS.contains(
        "fn object_prototype_tostring_classifies_proxy_arrays_and_rejects_revoked_proxies()"
    ));
    assert!(CLI_TESTS.contains("wasm_object_prototype_to_string_proxy_array.js"));
}
