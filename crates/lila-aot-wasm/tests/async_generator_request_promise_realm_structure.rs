use std::fs;
use std::path::Path;

const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const CURRENT_FUNCTION_REALM_INTRINSIC_PROMISE_CAPABILITY_SOURCE: &str =
    include_str!("../src/builtins/promise/current_function_realm_intrinsic_promise_capability.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_async_execution_realm.js");

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
fn promise_constructor_is_a_traced_realm_intrinsic_in_both_bootstraps() {
    for marker in [
        "pub(crate) const HEAP_REALM_INTRINSICS_RECORD_SIZE: u64 = 424;",
        "pub(crate) const HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET: u64 = 416;",
        "name: \"%Promise%\"",
        "offset: HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET",
        "pointer: true",
    ] {
        assert!(
            HEAP_SOURCE.contains(marker),
            "missing heap marker: {marker}"
        );
    }
    assert!(FUNCTIONS_SOURCE.contains("PromiseConstructor,"));
    assert!(FUNCTIONS_SOURCE
        .contains("Self::PromiseConstructor => HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET"));
    assert!(BOOTSTRAP_SOURCE.contains(
        "PROMISE_CONSTRUCTOR_GLOBAL_INDEX,\n                NonArrayRealmIntrinsicSlot::PromiseConstructor,"
    ));

    let created_realm_promise = between(
        HOST_SOURCE,
        "        self.emit_function_value_payload_in_realm(\n            &promise_meta,",
        "        for builtin in PROMISE_PROTOTYPE_METHOD_PUBLICATIONS",
    );
    assert!(created_realm_promise.contains(
        "NonArrayRealmIntrinsicSlot::PromiseConstructor,\n            promise_constructor_local,"
    ));
    assert!(
        created_realm_promise
            .find("emit_set_function_prototype_data_with_flags(")
            .unwrap()
            < created_realm_promise
                .find("NonArrayRealmIntrinsicSlot::PromiseConstructor")
                .unwrap()
    );
}

#[test]
fn current_function_constructor_proof_has_one_consuming_capability_api() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod current_function_realm_intrinsic_promise_capability;\n")
            .count(),
        1,
    );
    assert!(
        !PROMISE_SOURCE.contains("pub mod current_function_realm_intrinsic_promise_capability;")
    );
    assert!(!PROMISE_SOURCE.contains("current_function_realm_intrinsic_promise_capability::"));
    assert!(!PROMISE_SOURCE.contains("CurrentFunctionRealmIntrinsicPromiseConstructor"));
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "current_function_realm_intrinsic_promise_capability::",
        ),
        0,
        "the capability proof owner must have no import or re-export",
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "CurrentFunctionRealmIntrinsicPromiseConstructor",
        ),
        4,
        "the private child must own every intrinsic constructor proof use",
    );
    assert_eq!(
        CURRENT_FUNCTION_REALM_INTRINSIC_PROMISE_CAPABILITY_SOURCE
            .matches(
                "CurrentFunctionRealmIntrinsicPromiseConstructor {\n            constructor_payload_local,\n        }",
            )
            .count(),
        1,
        "the private child must contain the sole constructor proof construction",
    );
    assert_eq!(
        CURRENT_FUNCTION_REALM_INTRINSIC_PROMISE_CAPABILITY_SOURCE
            .matches("constructor.constructor_payload_local")
            .count(),
        2,
        "only the consuming adapter may project the constructor payload",
    );
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !CURRENT_FUNCTION_REALM_INTRINSIC_PROMISE_CAPABILITY_SOURCE.contains(&format!(
                "impl {capability} for CurrentFunctionRealmIntrinsicPromiseConstructor"
            ))
        );
    }
    assert!(
        CURRENT_FUNCTION_REALM_INTRINSIC_PROMISE_CAPABILITY_SOURCE
            .lines()
            .count()
            <= 120
    );

    let declaration = between(
        CURRENT_FUNCTION_REALM_INTRINSIC_PROMISE_CAPABILITY_SOURCE,
        "#[must_use = \"intrinsic Promise constructor must be consumed by capability allocation\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert!(declaration.contains("struct CurrentFunctionRealmIntrinsicPromiseConstructor"));
    assert!(declaration.contains("constructor_payload_local: u32"));
    assert!(!declaration.contains("derive("));

    let factory = between(
        CURRENT_FUNCTION_REALM_INTRINSIC_PROMISE_CAPABILITY_SOURCE,
        "pub(crate) fn emit_current_function_realm_intrinsic_promise_constructor(",
        "pub(crate) fn emit_new_current_function_realm_intrinsic_promise_capability(",
    );
    for marker in [
        "self.current_env_local",
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "HEAP_REALM_INTRINSICS_PROMISE_CONSTRUCTOR_OFFSET",
        "Instruction::Unreachable",
    ] {
        assert!(factory.contains(marker), "missing factory marker: {marker}");
    }
    assert!(!factory.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!factory.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));
    assert!(
        factory.find("let constructor_payload_local").unwrap()
            < factory.find("let realm_local").unwrap()
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

    let consumer = between(
        CURRENT_FUNCTION_REALM_INTRINSIC_PROMISE_CAPABILITY_SOURCE,
        "pub(crate) fn emit_new_current_function_realm_intrinsic_promise_capability(",
        "\n}",
    );
    assert!(consumer.contains("constructor: CurrentFunctionRealmIntrinsicPromiseConstructor"));
    assert!(consumer.contains("let result = self.emit_new_promise_capability("));
    assert!(
        consumer
            .find("release_temp_local(constructor_tag_local)")
            .unwrap()
            < consumer
                .find("release_temp_local(constructor.constructor_payload_local)")
                .unwrap()
    );
    assert!(
        consumer
            .find("release_temp_local(constructor.constructor_payload_local)")
            .unwrap()
            < consumer.rfind("result").unwrap()
    );
}

#[test]
fn all_three_request_methods_use_the_executing_function_realm() {
    let entry_publication = between(
        BOOTSTRAP_SOURCE,
        "pub(crate) fn init_async_generator_prototype(",
        "pub(crate) fn init_async_function_intrinsics(",
    );
    for marker in [
        "StandardBuiltinId::AsyncGeneratorPrototypeNext",
        "StandardBuiltinId::AsyncGeneratorPrototypeReturn",
        "StandardBuiltinId::AsyncGeneratorPrototypeThrow",
    ] {
        assert!(
            entry_publication.contains(marker),
            "missing entry publication marker: {marker}"
        );
    }
    assert!(entry_publication.contains("self.emit_function_value_payload(&method_meta, function)?"));
    assert!(entry_publication
        .contains("HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n                payload_local,"));
    assert!(entry_publication.contains("self.emit_object_define_local_data("));
    assert!(!entry_publication.contains("emit_object_define_function_data("));
    assert!(
        entry_publication
            .find("HEAP_FUNCTION_ENV_HANDLE_OFFSET")
            .unwrap()
            < entry_publication
                .find("self.emit_object_define_local_data(")
                .unwrap()
    );

    let request_arm = between(
        STANDARD_SOURCE,
        "StandardBuiltinId::AsyncGeneratorPrototypeNext\n            | StandardBuiltinId::AsyncGeneratorPrototypeReturn\n            | StandardBuiltinId::AsyncGeneratorPrototypeThrow => {",
        "StandardBuiltinId::ArrayIteratorNext => {",
    );
    assert!(request_arm.contains("emit_current_function_realm_intrinsic_promise_constructor("));
    assert!(request_arm.contains("emit_new_current_function_realm_intrinsic_promise_capability("));
    assert!(!request_arm.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));
    assert!(!request_arm.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!request_arm.contains("constructor_payload_local"));
    assert!(!request_arm.contains("constructor_tag_local"));
    assert!(!request_arm.contains("emit_async_generator_execution_realm_context_from_activation"));
    assert!(
        request_arm
            .find("emit_new_current_function_realm_intrinsic_promise_capability(")
            .unwrap()
            < request_arm
                .find("HEAP_OBJECT_INTERNAL_BRAND_OFFSET")
                .unwrap()
    );

    let dependency = between(
        PLANNING_SOURCE,
        "StandardBuiltinId::PromiseConstructor\n            | StandardBuiltinId::PromisePrototypeThen",
        "self.standard_roots\n                    .insert(StandardBuiltinId::PromiseSpeciesGetter);",
    );
    for builtin in [
        "AsyncGeneratorPrototypeNext",
        "AsyncGeneratorPrototypeReturn",
        "AsyncGeneratorPrototypeThrow",
    ] {
        assert!(
            dependency.contains(builtin),
            "missing planner dependency: {builtin}"
        );
    }
    assert!(dependency.contains(".insert(StandardBuiltinId::PromiseConstructor)"));
}

#[test]
fn finite_fixture_distinguishes_job_realm_from_request_method_realm() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_uses_async_function_realms_for_promises_and_reactions()"));
    for marker in [
        "other.Array.prototype.map.bind",
        "generator.next",
        "async-generator request Promise defining Realm",
        "invalid async-generator request Promise defining Realm",
        "invalid async-generator request TypeError defining Realm",
        "async-execution-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker: {marker}"
        );
    }
    assert!(!CLI_FIXTURE.contains("request ownership deferred"));
    assert!(!CLI_FIXTURE.contains("Atomics"));
    assert!(!CLI_FIXTURE.contains("waitAsync"));
}
