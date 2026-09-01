use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_with_resolvers_result_allocation.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_created_realm.js");

fn between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

fn count_in_rust_sources(root: &Path, fragment: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, fragment);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(fragment)
                .count()
        })
        .sum()
}

#[test]
fn with_resolvers_result_context_is_opaque_and_consumed_once() {
    let context_type = between(
        PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE,
        "#[must_use = \"Promise.withResolvers result allocation context must be consumed\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_with_resolvers_result_allocation;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_with_resolvers_result_allocation;"));
    assert!(!PROMISE_SOURCE.contains("promise_with_resolvers_result_allocation::"));
    assert!(!PROMISE_SOURCE.contains("PromiseWithResolversResultAllocationContext"));
    assert!(context_type.contains("pub(super) struct PromiseWithResolversResultAllocationContext"));
    assert!(context_type.contains("prototype_local: u32"));
    assert!(!context_type.contains("derive(Clone"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE.contains(&format!(
                "impl {capability} for PromiseWithResolversResultAllocationContext"
            )),
            "PromiseWithResolversResultAllocationContext must not acquire manual {capability}",
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "PromiseWithResolversResultAllocationContext"),
        4,
        "the private child must own every withResolvers result context use",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_with_resolvers_result_allocation::"),
        0,
        "the withResolvers result allocation owner must have no import or re-export",
    );
    assert_eq!(
        PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE
            .matches("PromiseWithResolversResultAllocationContext { prototype_local }")
            .count(),
        1,
        "only the private child may construct a withResolvers result context",
    );
    assert_eq!(
        PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE
            .matches("context.prototype_local")
            .count(),
        2,
        "only the consuming child installer may project and release the prototype local",
    );
    assert_eq!(
        PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE
            .matches("emit_current_function_promise_with_resolvers_result_allocation_context(")
            .count(),
        1,
        "the private child must own the sole context factory",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_current_function_promise_with_resolvers_result_allocation_context(")
            .count(),
        1,
        "Promise.withResolvers must remain the sole context factory caller",
    );
    assert_eq!(
        PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE
            .matches("emit_install_promise_with_resolvers_result_prototype(")
            .count(),
        1,
        "the private child must own the sole one-shot installer",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_install_promise_with_resolvers_result_prototype(")
            .count(),
        1,
        "Promise.withResolvers must remain the sole installer caller",
    );
}

#[test]
fn with_resolvers_result_context_selects_only_the_executing_function_realm() {
    let factory = between(
        PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE,
        "fn emit_current_function_promise_with_resolvers_result_allocation_context(",
        "pub(super) fn emit_install_promise_with_resolvers_result_prototype(",
    );
    for marker in [
        "self.current_env_local",
        "OBJECT_PROTOTYPE_GLOBAL_INDEX",
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET",
    ] {
        assert!(
            factory.contains(marker),
            "missing Realm authority: {marker}"
        );
    }
    assert!(!factory.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert_eq!(factory.matches("Instruction::Unreachable").count(), 3);
    assert!(
        factory.find("OBJECT_PROTOTYPE_GLOBAL_INDEX").unwrap()
            < factory.find("Instruction::Else").unwrap(),
        "the entry Object prototype is valid only in the zero-environment branch"
    );
    assert!(
        factory.find("let prototype_local").unwrap() < factory.find("let realm_local").unwrap()
    );
    assert!(
        factory.find("let realm_local").unwrap() < factory.find("let intrinsics_local").unwrap()
    );
    assert!(
        factory
            .find("release_temp_local(intrinsics_local)")
            .unwrap()
            < factory.find("release_temp_local(realm_local)").unwrap()
    );
}

#[test]
fn with_resolvers_result_installer_consumes_the_prototype_proof() {
    let installer = PROMISE_WITH_RESOLVERS_RESULT_ALLOCATION_SOURCE
        .split_once("fn emit_install_promise_with_resolvers_result_prototype(")
        .expect("Promise.withResolvers result prototype installer")
        .1
        .rsplit_once("\n    }\n}")
        .expect("Promise.withResolvers result prototype installer end")
        .0;
    assert!(installer.contains("context: PromiseWithResolversResultAllocationContext"));
    assert!(installer.contains("HEAP_PROTOTYPE_OFFSET"));
    assert!(installer.contains("HEAP_OBJECT_PROTOTYPE_TAG_OFFSET"));
    assert!(installer.contains("ValueKind::Object.tag()"));
    assert!(
        installer.find("HEAP_PROTOTYPE_OFFSET").unwrap()
            < installer
                .find("release_temp_local(context.prototype_local)")
                .unwrap()
    );
    assert!(!installer.contains("Result<"));
}

#[test]
fn with_resolvers_creates_the_capability_before_the_realm_owned_result() {
    let builtin = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_with_resolvers(",
        "pub(crate) fn emit_promise_try(",
    );
    let capability = builtin.find("emit_new_promise_capability(").unwrap();
    let raw_allocation = builtin
        .find("emit_alloc_plain_object_with_prototype(None, None, function)")
        .unwrap();
    let context = builtin
        .find("emit_current_function_promise_with_resolvers_result_allocation_context(function)")
        .unwrap();
    let installation = builtin
        .find("emit_install_promise_with_resolvers_result_prototype(")
        .unwrap();
    assert!(capability < raw_allocation);
    assert!(raw_allocation < context);
    assert!(context < installation);
    assert!(installation < builtin.find("\"promise\"").unwrap());
    assert_eq!(
        builtin
            .matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        1
    );
    assert!(!builtin.contains("OBJECT_PROTOTYPE_GLOBAL_INDEX"));
    assert!(!builtin.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(builtin.find("\"promise\"").unwrap() < builtin.find("\"resolve\"").unwrap());
    assert!(builtin.find("\"resolve\"").unwrap() < builtin.find("\"reject\"").unwrap());
}

#[test]
fn created_realm_fixture_separates_method_and_constructor_ownership() {
    for marker in [
        "borrowed Promise.withResolvers result object realm",
        "borrowed Promise.withResolvers constructor promise realm",
        "borrowed Promise.withResolvers resolve function realm",
        "borrowed Promise.withResolvers reject function realm",
        "entry Promise.withResolvers result object realm",
        "entry Promise.withResolvers constructor promise realm",
        "entry Promise.withResolvers resolve function realm",
        "entry Promise.withResolvers reject function realm",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing runtime witness: {marker}"
        );
    }
    assert!(CLI_FIXTURE.contains("otherPromise.withResolvers.call(Promise)"));
    assert!(CLI_FIXTURE.contains("Promise.withResolvers.call(otherPromise)"));
}
