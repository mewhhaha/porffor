const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");

const PROMISE_SPECIES_REALM_CONTEXT_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_species_realm_context.rs");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_internal_callback_realm.js");

#[test]
fn promise_callback_fixture_observes_species_constructor_and_error_realms() {
    for marker in [
        "borrowed Promise.then default species constructor realm",
        "borrowed Promise.then constructor TypeError realm",
        "borrowed Promise.finally species TypeError realm",
        "other.Promise.prototype.then.call(",
        "other.Promise.prototype.finally.call(",
        "promise-internal-callback-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing runtime witness: {marker}"
        );
    }
}

#[test]
fn promise_species_realm_context_is_private_paired_and_consumed() {
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_species_realm_context;\n")
            .count(),
        1
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_species_realm_context;"));
    assert!(!PROMISE_SOURCE.contains("promise_species_realm_context::"));
    assert!(!PROMISE_SOURCE.contains("PromiseSpeciesRealmContext"));
    let declaration = between(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE,
        "#[must_use",
        "impl FunctionBuilder<'_>",
    );
    assert!(declaration.contains("pub(super) struct PromiseSpeciesRealmContext"));
    assert!(!declaration.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!PROMISE_SPECIES_REALM_CONTEXT_SOURCE
            .contains(&format!("impl {capability} for PromiseSpeciesRealmContext")));
    }
}
