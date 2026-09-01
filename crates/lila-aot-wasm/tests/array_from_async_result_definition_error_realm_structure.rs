const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const SET_PATH_REALM_SOURCE: &str = include_str!("../src/objects/set_path_realm.rs");
const ARRAY_FROM_ASYNC_SOURCE: &str = include_str!("../src/builtins/array_from_async.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/array.rs");
const CLI_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_array_from_async_result_definition_error_realm.js"
);

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
fn mutation_error_realm_is_an_exhaustive_three_source_two_authority_domain() {
    let source_domain = bounded(
        EMIT_SOURCE,
        "pub(crate) enum ObjectMutationErrorRealmSource {",
        "impl ObjectReadErrorRealmSource",
    );
    for state in [
        "GlobalFallback",
        "StandardBuiltinEnvironment",
        "SetPathHelperArgument",
    ] {
        assert_eq!(source_domain.matches(state).count() >= 1, true, "{state}");
    }

    let authority_domain = bounded(
        SET_PATH_REALM_SOURCE,
        "pub(super) enum ObjectMutationErrorRealm {",
        "pub(super) const fn set_path_realm_environment_argument(",
    );
    for state in ["TrustedCurrentEnvironment", "MainRealmFallback"] {
        assert_eq!(authority_domain.matches(state).count(), 1, "{state}");
    }

    let projection = bounded(
        SET_PATH_REALM_SOURCE,
        "pub(super) const fn object_mutation_error_realm(",
        "#[cfg(test)]",
    );
    for state in [
        "ObjectMutationErrorRealmSource::GlobalFallback",
        "ObjectMutationErrorRealmSource::StandardBuiltinEnvironment",
        "ObjectMutationErrorRealmSource::SetPathHelperArgument",
    ] {
        assert_eq!(projection.matches(state).count(), 1, "{state}");
    }
    assert!(!projection.contains("_ =>"));
}

#[test]
fn every_ordinary_descriptor_failure_uses_the_typed_mutation_error_owner() {
    let descriptor_validation = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_define_entry_validated(",
        "pub(crate) fn emit_create_data_property_or_throw(",
    );
    assert_eq!(
        descriptor_validation
            .matches("emit_object_mutation_type_error(TYPE_ERROR_NAME, function)?;")
            .count(),
        7
    );
    assert_eq!(
        descriptor_validation
            .matches("emit_object_mutation_type_error_without_message(function)?;")
            .count(),
        1
    );
    assert!(!descriptor_validation.contains("emit_throw_runtime_error("));
    assert!(!descriptor_validation.contains("TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX"));

    let no_message_emitter = bounded(
        ERRORS_SOURCE,
        "fn emit_throw_type_error_without_message_with_prototype_local(",
        "pub(crate) fn emit_throw_current_function_realm_range_error(",
    );
    assert!(no_message_emitter.contains("emit_set_thrown_error_text(TYPE_ERROR_NAME, None"));
    assert!(!no_message_emitter.contains("strings.payload(\"message\")"));

    let create_data_property = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_create_data_property_or_throw(",
        "fn emit_object_write_via_helper(",
    );
    assert_eq!(
        create_data_property
            .matches("emit_object_mutation_type_error(")
            .count(),
        2
    );
    assert!(!create_data_property.contains("emit_throw_runtime_error("));
}

#[test]
fn ordinary_set_failures_and_array_from_async_routes_retain_typed_realm_authority() {
    let set_failure = bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_write_set_failure_else(",
        "pub(crate) fn emit_proxy_set_false_result_throw(",
    );
    assert_eq!(
        set_failure
            .matches("emit_object_mutation_type_error_to_active_handler(message, function)?;")
            .count(),
        2
    );
    assert!(!set_failure.contains("emit_throw_runtime_error_to_active_handler("));

    let non_extensible_failure = bounded(
        OBJECTS_SOURCE,
        "fn emit_object_write_non_extensible_failure(",
        "pub(crate) fn emit_object_write_strict(",
    );
    assert_eq!(
        non_extensible_failure
            .matches("emit_object_mutation_type_error_to_active_handler(")
            .count(),
        2
    );
    assert!(!non_extensible_failure.contains("emit_throw_runtime_error_to_active_handler("));

    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("self.emit_array_from_async_define_current_value(")
            .count(),
        1
    );
    assert_eq!(
        ARRAY_FROM_ASYNC_SOURCE
            .matches("self.emit_array_from_async_set_length(")
            .count(),
        2
    );
}

#[test]
fn focused_fixture_distinguishes_method_constructor_result_and_user_error_authority() {
    for marker in [
        "new Proxy(other.Object",
        "new Proxy(Object",
        "entry method ignores foreign constructor Realm for index failure",
        "created method ignores entry constructor Realm for index failure",
        "entry method ignores foreign constructor Realm for length failure",
        "created method ignores entry constructor Realm for length failure",
        "zero-length fast path uses created method Realm",
        "non-extensible result TypeError has no own message",
        "length setter error identity",
        "array-from-async-result-definition-error-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(
        CLI_TESTS.contains("fn array_from_async_result_definition_errors_use_the_method_realm()")
    );
    assert!(CLI_TESTS.contains("wasm_array_from_async_result_definition_error_realm.js"));
}
