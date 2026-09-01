use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_SPECIES_REALM_CONTEXT_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_species_realm_context.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_internal_callback_realm.js");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

fn count_in_rust_sources(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| {
            entry
                .expect("source directory entry should be readable")
                .path()
        })
        .map(|path| {
            if path.is_dir() {
                count_in_rust_sources(&path, needle)
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                    .matches(needle)
                    .count()
            } else {
                0
            }
        })
        .sum()
}

#[test]
fn promise_species_realm_context_is_private_paired_and_consumed() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_species_realm_context;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_species_realm_context;"));
    assert!(!PROMISE_SOURCE.contains("promise_species_realm_context::"));
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_species_realm_context::"),
        0,
        "the species context owner must have no import or re-export",
    );
    assert!(!PROMISE_SOURCE.contains("PromiseSpeciesRealmContext"));
    assert_eq!(
        count_in_rust_sources(&source_root, "PromiseSpeciesRealmContext"),
        4,
        "the private child must own every species context type use",
    );

    let context = between(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE,
        "#[must_use = \"Promise species Realm context must be consumed\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert!(PROMISE_SPECIES_REALM_CONTEXT_SOURCE
        .contains("#[must_use = \"Promise species Realm context must be consumed\"]"));
    assert!(context.contains("pub(super) struct PromiseSpeciesRealmContext"));
    assert!(context.contains("default_constructor_payload_local: u32"));
    assert!(context.contains("type_error_prototype_local: u32"));
    assert!(!context.contains("#[derive"));
    assert_eq!(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE
            .matches(
                "PromiseSpeciesRealmContext {\n            default_constructor_payload_local,",
            )
            .count(),
        1,
        "the private child must own the sole paired-context construction",
    );
    assert_eq!(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE
            .matches("context.default_constructor_payload_local")
            .count(),
        2,
    );
    assert_eq!(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE
            .matches("context.type_error_prototype_local")
            .count(),
        3,
    );
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(!PROMISE_SPECIES_REALM_CONTEXT_SOURCE
            .contains(&format!("impl {capability} for PromiseSpeciesRealmContext")));
    }
    assert!(PROMISE_SPECIES_REALM_CONTEXT_SOURCE.lines().count() <= 220);
    assert_eq!(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE
            .matches("emit_current_function_promise_species_realm_context(")
            .count(),
        1,
        "the child must own the sole context factory"
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_current_function_promise_species_realm_context(")
            .count(),
        2,
        "then and finally must remain the exact context factory callers"
    );
    assert_eq!(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE
            .matches("emit_promise_species_constructor(")
            .count(),
        1,
        "the child must own the sole consuming helper"
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_promise_species_constructor(")
            .count(),
        2,
        "then and finally must remain the exact consuming-helper callers"
    );
}

#[test]
fn promise_species_context_uses_one_strict_defining_realm_catalog() {
    let factory = between(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE,
        "pub(super) fn emit_current_function_promise_species_realm_context(",
        "pub(super) fn emit_promise_species_constructor(",
    );
    for marker in [
        "self.current_env_local",
        "PROMISE_CONSTRUCTOR_GLOBAL_INDEX",
        "TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX",
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET",
        "HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(
            factory.contains(marker),
            "missing Realm authority: {marker}"
        );
    }
    assert!(!factory.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!factory.contains("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET"));
    assert_eq!(
        factory.matches("Instruction::Unreachable").count(),
        3,
        "Realm and intrinsics trap directly; the two catalog slots share one loop-emitted trap"
    );
    assert!(factory.contains("for (offset, destination_local) in ["));
    let entry_end = factory.find("Instruction::Else").unwrap();
    assert!(factory.find("PROMISE_CONSTRUCTOR_GLOBAL_INDEX").unwrap() < entry_end);
    assert!(factory.find("TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX").unwrap() < entry_end);
    assert!(factory.find("HEAP_FUNCTION_DEFINING_REALM_OFFSET").unwrap() > entry_end);
    assert!(
        factory
            .find("HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET")
            .unwrap()
            > entry_end
    );
    assert!(
        factory
            .find("HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET")
            .unwrap()
            > entry_end
    );

    let constructor_reservation = factory
        .find("let default_constructor_payload_local")
        .unwrap();
    let type_error_reservation = factory.find("let type_error_prototype_local").unwrap();
    let realm_reservation = factory.find("let realm_local").unwrap();
    let intrinsics_reservation = factory.find("let intrinsics_local").unwrap();
    assert!(constructor_reservation < type_error_reservation);
    assert!(type_error_reservation < realm_reservation);
    assert!(realm_reservation < intrinsics_reservation);
    assert!(
        factory
            .find("release_temp_local(intrinsics_local)")
            .unwrap()
            < factory.find("release_temp_local(realm_local)").unwrap()
    );
}

#[test]
fn promise_species_helper_releases_the_paired_context_on_emission_errors() {
    let helper = between(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE,
        "pub(super) fn emit_promise_species_constructor(",
        "\n}",
    );
    assert!(helper.contains("context: PromiseSpeciesRealmContext"));
    assert!(helper.contains("let result = (|| -> Result<(), EmitError>"));
    assert!(helper.contains("context.default_constructor_payload_local"));
    assert_eq!(
        helper
            .matches("emit_throw_runtime_error_with_prototype_local(")
            .count(),
        2
    );
    assert_eq!(
        helper.matches("context.type_error_prototype_local").count(),
        3
    );
    assert!(!helper.contains("self.emit_throw_runtime_error("));
    assert!(!helper.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));
    assert!(!helper.contains("CURRENT_REALM_GLOBAL_INDEX"));

    let helper_local_release = helper.find("release_temp_local(key_local)").unwrap();
    let type_error_release = helper
        .find("release_temp_local(context.type_error_prototype_local)")
        .unwrap();
    let constructor_release = helper
        .find("release_temp_local(context.default_constructor_payload_local)")
        .unwrap();
    let returned_result = helper.rfind("result").unwrap();
    assert!(helper_local_release < type_error_release);
    assert!(type_error_release < constructor_release);
    assert!(constructor_release < returned_result);
}

#[test]
fn then_and_finally_acquire_species_authority_after_receiver_checks() {
    let then_builtin = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_prototype_then(",
        "pub(crate) fn emit_promise_prototype_catch(",
    );
    let finally_builtin = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_prototype_finally(",
        "fn emit_run_async_continuation_job(",
    );

    let then_receiver_error = then_builtin
        .find("emit_throw_promise_then_incompatible_receiver_error(")
        .unwrap();
    let then_context = then_builtin
        .find("emit_current_function_promise_species_realm_context(function)")
        .unwrap();
    let then_species = then_builtin
        .find("emit_promise_species_constructor(")
        .unwrap();
    let then_capability = then_builtin.find("emit_new_promise_capability(").unwrap();
    assert!(then_receiver_error < then_context);
    assert!(then_context < then_species);
    assert!(then_species < then_capability);

    let finally_receiver_error = finally_builtin
        .find("emit_throw_promise_finally_non_object_receiver_error(")
        .unwrap();
    let finally_context = finally_builtin
        .find("emit_current_function_promise_species_realm_context(function)")
        .unwrap();
    let finally_species = finally_builtin
        .find("emit_promise_species_constructor(")
        .unwrap();
    let finally_closures = finally_builtin
        .find("HEAP_PROMISE_FINALLY_CONTEXT_SIZE")
        .unwrap();
    assert!(finally_receiver_error < finally_context);
    assert!(finally_context < finally_species);
    assert!(finally_species < finally_closures);
}

#[test]
fn species_constructor_preserves_property_and_validation_order() {
    let helper = between(
        PROMISE_SPECIES_REALM_CONTEXT_SOURCE,
        "pub(super) fn emit_promise_species_constructor(",
        "\n}",
    );
    let constructor_get = helper
        .find("self.strings.payload(\"constructor\")")
        .unwrap();
    let constructor_error = helper
        .find("Promise constructor property is not an object")
        .unwrap();
    let species_get = helper
        .find("property_key_symbol_payload(\"Symbol.species\")")
        .unwrap();
    let species_error = helper.find("Promise species is not a constructor").unwrap();
    assert!(constructor_get < constructor_error);
    assert!(constructor_error < species_get);
    assert!(species_get < species_error);
}

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
