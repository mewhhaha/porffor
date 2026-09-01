use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE: &str =
    include_str!("../src/builtins/promise/promise_try_callback_type_error.rs");
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
fn promise_try_callback_type_error_proof_is_private_and_one_shot() {
    let proof = between(
        PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE,
        "#[must_use = \"Promise.try callback TypeError prototype must be consumed\"]",
        "impl<'a> FunctionBuilder<'a>",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("\nmod promise_try_callback_type_error;\n")
            .count(),
        1,
    );
    assert!(!PROMISE_SOURCE.contains("pub mod promise_try_callback_type_error;"));
    assert!(!PROMISE_SOURCE.contains("promise_try_callback_type_error::"));
    assert!(!PROMISE_SOURCE.contains("PromiseTryCallbackTypeErrorPrototypeLocal"));
    assert!(proof.contains("pub(super) struct PromiseTryCallbackTypeErrorPrototypeLocal(u32);"));
    assert!(proof.contains("(u32);"));
    assert!(!proof.contains("#[derive"));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq"] {
        assert!(
            !PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE.contains(&format!(
                "impl {capability} for PromiseTryCallbackTypeErrorPrototypeLocal"
            )),
            "PromiseTryCallbackTypeErrorPrototypeLocal must not acquire manual {capability}",
        );
    }
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "PromiseTryCallbackTypeErrorPrototypeLocal"),
        4,
        "the private child must own every Promise.try callback TypeError proof use",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "promise_try_callback_type_error::"),
        0,
        "the Promise.try callback TypeError owner must have no import or re-export",
    );
    assert_eq!(
        PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE
            .matches("PromiseTryCallbackTypeErrorPrototypeLocal(prototype_local)")
            .count(),
        1,
        "only the private child may construct the raw prototype proof",
    );
    assert_eq!(
        PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE
            .matches("prototype.0")
            .count(),
        2,
        "only the consuming child method may project and release the raw prototype",
    );
    assert_eq!(
        PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE
            .matches("emit_load_promise_try_callback_type_error_prototype(")
            .count(),
        1,
        "the private child must own the sole proof factory",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_load_promise_try_callback_type_error_prototype(")
            .count(),
        1,
        "Promise.try must remain the sole proof factory caller",
    );
    assert_eq!(
        PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE
            .matches("emit_throw_promise_try_non_callable_callback(")
            .count(),
        1,
        "the private child must own the sole proof consumer",
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("emit_throw_promise_try_non_callable_callback(")
            .count(),
        1,
        "Promise.try must remain the sole proof consumer caller",
    );
}

#[test]
fn promise_try_callback_type_error_proof_uses_only_the_executing_function_snapshot() {
    let factory = between(
        PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE,
        "fn emit_load_promise_try_callback_type_error_prototype(",
        "pub(super) fn emit_throw_promise_try_non_callable_callback(",
    );
    for marker in [
        "self.current_env_local",
        "TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX",
        "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET",
    ] {
        assert!(
            factory.contains(marker),
            "missing Realm authority: {marker}"
        );
    }
    assert!(!factory.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!factory.contains("PROMISE_CONSTRUCTOR_GLOBAL_INDEX"));
    assert!(!factory.contains("HEAP_FUNCTION_DEFINING_REALM_OFFSET"));
    assert_eq!(factory.matches("Instruction::Unreachable").count(), 1);
    assert!(
        factory.find("TYPE_ERROR_PROTOTYPE_GLOBAL_INDEX").unwrap()
            < factory.find("Instruction::Else").unwrap(),
        "the entry TypeError prototype is valid only for the zero environment"
    );
    assert!(
        factory
            .find("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET")
            .unwrap()
            > factory.find("Instruction::Else").unwrap(),
        "a self-backed Promise.try must load its published snapshot"
    );
}

#[test]
fn promise_try_callback_type_error_consumer_releases_its_proof() {
    let consumer = PROMISE_TRY_CALLBACK_TYPE_ERROR_SOURCE
        .split_once("fn emit_throw_promise_try_non_callable_callback(")
        .expect("Promise.try callback TypeError proof consumer")
        .1
        .rsplit_once("\n    }\n}")
        .expect("Promise.try callback TypeError proof consumer end")
        .0;
    assert!(consumer.contains("prototype: PromiseTryCallbackTypeErrorPrototypeLocal"));
    assert!(consumer.contains("emit_throw_runtime_error_with_prototype_local("));
    assert!(consumer.contains("TYPE_ERROR_NAME"));
    assert!(consumer.contains("\"value is not callable\""));
    assert!(
        consumer
            .find("emit_throw_runtime_error_with_prototype_local(")
            .unwrap()
            < consumer.find("release_temp_local(prototype.0)").unwrap()
    );
    assert!(consumer.trim_end().ends_with("result"));
}

#[test]
fn promise_try_preserves_capability_and_argument_order_before_rejecting_invalid_callbacks() {
    let builtin = between(
        PROMISE_SOURCE,
        "pub(crate) fn emit_promise_try(",
        "#[allow(clippy::too_many_arguments)]",
    );
    let capability = builtin.find("emit_new_promise_capability(").unwrap();
    let callback = builtin.find("emit_builtin_arg_to_locals(0").unwrap();
    let argv_allocation = builtin
        .find("emit_alloc_array_payload_with_length(")
        .unwrap();
    let argv_copy = builtin.find("self.emit_array_write(").unwrap();
    let callable = builtin.find("emit_is_callable_i32(").unwrap();
    let generic_call = builtin
        .find("emit_function_or_proxy_call_with_argv_leave_throw_completion(")
        .unwrap();
    let invalid_callback = builtin
        .find("emit_load_promise_try_callback_type_error_prototype(function)")
        .unwrap();
    let settle_selection = builtin
        .find("Instruction::I64Const(COMPLETION_KIND_THROW)")
        .unwrap();

    assert!(capability < callback);
    assert!(callback < argv_allocation);
    assert!(argv_allocation < argv_copy);
    assert!(argv_copy < callable);
    assert!(callable < generic_call);
    assert!(generic_call < invalid_callback);
    assert!(invalid_callback < settle_selection);
    assert_eq!(
        builtin
            .matches("emit_function_or_proxy_call_with_argv_leave_throw_completion(")
            .count(),
        1
    );
    let callback_dispatch = between(
        builtin,
        "self.emit_is_callable_i32(",
        "function.instruction(&Instruction::LocalGet(self.completion_local));",
    );
    assert!(callback_dispatch.contains("Instruction::If(BlockType::Empty)"));
    assert!(callback_dispatch.contains("Instruction::Else"));
    assert!(callback_dispatch.contains("emit_throw_promise_try_non_callable_callback("));
    assert!(!callback_dispatch.contains("emit_return_current_completion"));
    assert!(!callback_dispatch.contains("CURRENT_REALM_GLOBAL_INDEX"));
}

#[test]
fn promise_internal_callback_fixture_observes_the_borrowed_try_error_realm() {
    for marker in [
        "other.Promise.try(0)",
        "Promise.try callback TypeError realm",
        "Promise.try callback rejection checkpoint",
        "promise-internal-callback-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing runtime witness: {marker}"
        );
    }
    assert!(
        CLI_FIXTURE
            .find("Promise.try callback rejection checkpoint")
            .unwrap()
            < CLI_FIXTURE
                .find("promise-internal-callback-realm:ok")
                .unwrap(),
        "the success sentinel must be gated by the Promise.try rejection reaction"
    );
}
