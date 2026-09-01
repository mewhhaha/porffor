use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_combinator_algorithm_error_realm.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_promise_combinator_algorithm_error_realm.js");

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
fn promise_combinator_algorithm_error_realm_context_is_private_paired_and_non_copyable() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_combinator_algorithm_error_realm;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_combinator_algorithm_error_realm;"));
    assert!(!PROMISE_SOURCE.contains("promise_combinator_algorithm_error_realm::"));
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_combinator_algorithm_error_realm::",),
        0,
        "the algorithm-error Realm owner must have no import or re-export",
    );
    assert!(!PROMISE_SOURCE.contains("PromiseCombinatorAlgorithmErrorRealmContext"));
    assert_eq!(
        count_in_rust_sources(&source_root, "PromiseCombinatorAlgorithmErrorRealmContext",),
        6,
        "the private child must own every algorithm-error context type use",
    );

    let context = between(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE,
        "#[must_use = \"Promise combinator algorithmic error Realm context must be explicitly released\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert!(PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE.contains(
        "#[must_use = \"Promise combinator algorithmic error Realm context must be explicitly released\"]"
    ));
    assert!(context.contains("pub(super) struct PromiseCombinatorAlgorithmErrorRealmContext"));
    assert!(context.contains("type_error_prototype_local: u32"));
    assert!(context.contains("range_error_prototype_local: u32"));
    assert!(!context.contains("derive(Clone"));
    assert_eq!(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE
            .matches(
                "PromiseCombinatorAlgorithmErrorRealmContext {\n            type_error_prototype_local,",
            )
            .count(),
        1,
        "the private child must own the sole paired-context construction",
    );
    assert_eq!(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE
            .matches("realm.type_error_prototype_local")
            .count(),
        2,
    );
    assert_eq!(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE
            .matches("realm.range_error_prototype_local")
            .count(),
        2,
    );
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE.contains(&format!(
                "impl {capability} for PromiseCombinatorAlgorithmErrorRealmContext"
            ))
        );
    }
    assert!(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE
            .lines()
            .count()
            <= 150
    );

    for (method, parent_calls) in [
        ("emit_promise_combinator_algorithm_error_realm_context(", 3),
        ("emit_throw_promise_combinator_type_error(", 14),
        ("emit_throw_promise_combinator_range_error(", 1),
        (
            "release_promise_combinator_algorithm_error_realm_context(",
            3,
        ),
    ] {
        assert_eq!(
            PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE
                .matches(method)
                .count(),
            1,
            "the child must own the sole {method} definition",
        );
        assert_eq!(
            PROMISE_SOURCE.matches(method).count(),
            parent_calls,
            "the parent must retain the exact {method} caller census",
        );
    }
}

#[test]
fn promise_combinator_algorithm_error_realm_factory_uses_one_strict_intrinsic_catalog() {
    let factory = between(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE,
        "pub(super) fn emit_promise_combinator_algorithm_error_realm_context(",
        "pub(super) fn emit_throw_promise_combinator_type_error(",
    );
    for marker in [
        "self.current_env_local",
        "TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX",
        "RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX",
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "HEAP_REALM_INTRINSICS_TYPE_ERROR_PROTOTYPE_OFFSET",
        "HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(
            factory.contains(marker),
            "missing Realm authority: {marker}"
        );
    }
    assert!(!factory.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!factory.contains("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET"));
    assert!(!factory.contains("HEAP_FUNCTION_REALM_RANGE_ERROR_PROTOTYPE_OFFSET"));
    assert_eq!(factory.matches("Instruction::Unreachable").count(), 3);

    let entry_end = factory.find("Instruction::Else").unwrap();
    assert!(factory.find("TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX").unwrap() < entry_end);
    assert!(factory.find("RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX").unwrap() < entry_end);
    assert!(factory.find("HEAP_FUNCTION_DEFINING_REALM_OFFSET").unwrap() > entry_end);
    assert!(factory.find("HEAP_REALM_INTRINSICS_OFFSET").unwrap() > entry_end);

    let type_error_reservation = factory.find("let type_error_prototype_local").unwrap();
    let range_error_reservation = factory.find("let range_error_prototype_local").unwrap();
    let realm_reservation = factory.find("let realm_local").unwrap();
    let intrinsics_reservation = factory.find("let intrinsics_local").unwrap();
    assert!(type_error_reservation < range_error_reservation);
    assert!(range_error_reservation < realm_reservation);
    assert!(realm_reservation < intrinsics_reservation);
    assert!(
        factory
            .find("release_temp_local(intrinsics_local)")
            .unwrap()
            < factory.find("release_temp_local(realm_local)").unwrap()
    );
}

#[test]
fn promise_combinators_have_exact_three_factory_fifteen_borrow_three_release_lifecycle() {
    let race = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_race(",
        "fn emit_promise_keyed_reject_current_throw(",
    );
    let keyed = between(
        PROMISE_SOURCE,
        "fn emit_promise_keyed(",
        "pub(crate) fn emit_promise_all(",
    );
    let standard = between(
        PROMISE_SOURCE,
        "fn emit_promise_combinator(",
        "pub(crate) fn emit_promise_resolving_function(",
    );
    let combinators = [race, keyed, standard];

    assert_eq!(
        combinators
            .iter()
            .map(|builtin| builtin
                .matches("emit_promise_combinator_algorithm_error_realm_context(function)")
                .count())
            .sum::<usize>(),
        3
    );
    assert_eq!(
        combinators
            .iter()
            .map(|builtin| builtin.matches("&algorithm_error_realm").count())
            .sum::<usize>(),
        15
    );
    assert_eq!(
        combinators
            .iter()
            .map(|builtin| builtin
                .matches("release_promise_combinator_algorithm_error_realm_context(algorithm_error_realm)")
                .count())
            .sum::<usize>(),
        3
    );
    assert_eq!(race.matches("&algorithm_error_realm").count(), 6);
    assert_eq!(keyed.matches("&algorithm_error_realm").count(), 2);
    assert_eq!(standard.matches("&algorithm_error_realm").count(), 7);

    for builtin in combinators {
        assert!(!builtin.contains("self.emit_throw_runtime_error("));
        let resolve_get = builtin
            .find("emit_object_read_without_throw_propagation(")
            .unwrap();
        let abrupt_route = builtin
            .find(
                if builtin.contains("emit_promise_keyed_reject_current_throw(") {
                    "emit_promise_keyed_reject_current_throw("
                } else {
                    "emit_promise_combinator_reject_current_throw("
                },
            )
            .unwrap();
        let context = builtin
            .find("emit_promise_combinator_algorithm_error_realm_context(function)")
            .unwrap();
        let callable = builtin.find("self.emit_is_callable_i32(").unwrap();
        assert!(resolve_get < abrupt_route);
        assert!(abrupt_route < context);
        assert!(context < callable);
    }
}

#[test]
fn promise_combinator_error_consumers_use_only_the_typed_prototypes() {
    let consumers = between(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE,
        "pub(super) fn emit_throw_promise_combinator_type_error(",
        "\n}",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_throw_promise_combinator_type_error(")
            .count(),
        14,
        "the parent must retain fourteen TypeError branches"
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_throw_promise_combinator_range_error(")
            .count(),
        1,
        "the parent must retain the structural max-length RangeError branch"
    );
    assert!(consumers.contains("realm.type_error_prototype_local"));
    assert!(consumers.contains("realm.range_error_prototype_local"));
    assert_eq!(
        consumers
            .matches("emit_throw_runtime_error_with_prototype_local(")
            .count(),
        2
    );
    assert!(!consumers.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!consumers.contains("TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX"));
    assert!(!consumers.contains("RANGE_ERROR_PROTOTYPE_GLOBAL_INDEX"));

    let release = between(
        PROMISE_COMBINATOR_ALGORITHM_ERROR_REALM_SOURCE,
        "pub(super) fn release_promise_combinator_algorithm_error_realm_context(",
        "\n}",
    );
    assert!(
        release
            .find("release_temp_local(realm.range_error_prototype_local)")
            .unwrap()
            < release
                .find("release_temp_local(realm.type_error_prototype_local)")
                .unwrap()
    );

    let static_settle = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_static_settle(",
        "pub(crate) fn emit_promise_try(",
    );
    assert_eq!(
        static_settle
            .matches("self.emit_throw_runtime_error(")
            .count(),
        1
    );
}

#[test]
fn created_realm_fixture_covers_all_six_combinator_method_identities() {
    for marker in [
        "other.Promise.all.call(Promise, null)",
        "other.Promise.allSettled.call(Promise, null)",
        "other.Promise.allKeyed.call(Promise, 0)",
        "other.Promise.allSettledKeyed.call(Promise, 0)",
        "other.Promise.any.call(Promise, null)",
        "other.Promise.race.call(Promise, null)",
        "other.TypeError.prototype",
        "promise-combinator-algorithm-error-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing runtime witness: {marker}"
        );
    }
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_uses_created_realm_promise_combinator_algorithm_errors()"));
    assert!(CLI_TESTS.contains("wasm_promise_combinator_algorithm_error_realm.js"));
}
