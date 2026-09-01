const ATOMICS_SOURCE: &str = include_str!("../src/builtins/atomics.rs");
const WAIT_ASYNC_RESULT_SOURCE: &str = include_str!("../src/builtins/atomics/wait_async_result.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/binary_data.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_atomics_wait_async_created_realm.js");

fn result_object_prototype() -> &'static str {
    WAIT_ASYNC_RESULT_SOURCE
        .split_once("fn emit_atomics_wait_async_result_object_prototype(")
        .expect("Atomics.waitAsync result Object prototype")
        .1
        .split_once("fn emit_atomics_wait_async_return_object(")
        .expect("Atomics.waitAsync result Object prototype end")
        .0
}

fn synchronous_result() -> &'static str {
    WAIT_ASYNC_RESULT_SOURCE
        .split_once("fn emit_atomics_wait_async_return_object(")
        .expect("Atomics.waitAsync synchronous result")
        .1
        .split_once("fn emit_atomics_wait_async_return_promise(")
        .expect("Atomics.waitAsync synchronous result end")
        .0
}

fn asynchronous_result() -> &'static str {
    WAIT_ASYNC_RESULT_SOURCE
        .split_once("fn emit_atomics_wait_async_return_promise(")
        .expect("Atomics.waitAsync asynchronous result")
        .1
        .split_once("}\n}")
        .expect("Atomics.waitAsync asynchronous result end")
        .0
}

fn wait_async_body() -> &'static str {
    ATOMICS_SOURCE
        .split_once("fn emit_atomics_wait_async(")
        .expect("Atomics.waitAsync body")
        .1
        .split_once("pub(crate) fn emit_drain_atomics_wait_async_timeouts(")
        .expect("Atomics.waitAsync body end")
        .0
}

fn entry_realm_atomics_installer() -> &'static str {
    BOOTSTRAP_SOURCE
        .split_once("pub(crate) fn init_atomics_object(")
        .expect("entry-Realm Atomics installer")
        .1
        .split_once("pub(crate) fn init_typed_array_intrinsic(")
        .expect("entry-Realm Atomics installer end")
        .0
}

fn created_realm_atomics_installer() -> &'static str {
    HOST_SOURCE
        .split_once(
            "        self.emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)?;\n        function.instruction(&Instruction::LocalSet(atomics_object_local));",
        )
        .expect("created-Realm Atomics installer")
        .1
        .split_once(
            "        self.emit_function_value_payload_in_realm(\n            &promise_meta,",
        )
        .expect("created-Realm Atomics installer end")
        .0
}

#[test]
fn wait_async_result_object_prototype_requires_defining_realm_intrinsics() {
    let object_prototype = result_object_prototype();
    let proof_type = WAIT_ASYNC_RESULT_SOURCE
        .split_once("#[must_use = \"Atomics.waitAsync result Object prototype must be consumed\"]")
        .expect("Atomics.waitAsync result Object prototype proof")
        .1
        .split_once("impl<'a> FunctionBuilder<'a>")
        .expect("Atomics.waitAsync result Object prototype proof end")
        .0;

    assert!(proof_type.contains("struct AtomicsWaitAsyncResultObjectPrototypeLocal(u32);"));
    assert!(!proof_type.contains("Clone"));
    assert!(!proof_type.contains("Copy"));
    assert_eq!(
        WAIT_ASYNC_RESULT_SOURCE
            .matches("struct AtomicsWaitAsyncResultObjectPrototypeLocal(u32);")
            .count(),
        1
    );
    assert!(!WAIT_ASYNC_RESULT_SOURCE.contains("AtomicsWaitAsyncResultKind"));
    assert!(!WAIT_ASYNC_RESULT_SOURCE.contains("AtomicsWaitAsyncResultRealmContext"));
    assert!(!object_prototype
        .contains("emit_current_function_realm_intrinsic_promise_allocation_context(function)"));
    for required_slot in [
        "HEAP_FUNCTION_DEFINING_REALM_OFFSET",
        "HEAP_REALM_INTRINSICS_OFFSET",
        "HEAP_REALM_INTRINSICS_OBJECT_PROTOTYPE_OFFSET",
    ] {
        assert!(
            object_prototype.contains(required_slot),
            "missing {required_slot}"
        );
    }
    assert_eq!(
        object_prototype.matches("Instruction::Unreachable").count(),
        4
    );
    assert!(!object_prototype.contains("CURRENT_REALM_GLOBAL_INDEX"));
    assert!(!object_prototype.contains("OBJECT_PROTOTYPE_GLOBAL_INDEX"));
    assert!(!object_prototype.contains("PROMISE_PROTOTYPE_GLOBAL_INDEX"));

    let promise_context = PROMISE_SOURCE
        .split_once(
            "pub(crate) fn emit_current_function_realm_intrinsic_promise_allocation_context(",
        )
        .expect("current-function Promise allocation context")
        .1
        .split_once("fn emit_current_function_realm_promise_allocation_context(")
        .expect("current-function Promise allocation context end")
        .0;
    assert!(promise_context.contains("LocalGet(self.current_env_local)"));
    assert!(promise_context.contains("HEAP_FUNCTION_DEFINING_REALM_OFFSET"));
    assert!(promise_context.contains("HEAP_REALM_INTRINSICS_PROMISE_PROTOTYPE_OFFSET"));
    assert_eq!(
        promise_context.matches("Instruction::Unreachable").count(),
        4
    );
    assert!(!promise_context.contains("CURRENT_REALM_GLOBAL_INDEX"));
}

#[test]
fn wait_async_branches_use_typed_object_prototype_and_create_enumerable_ordered_results() {
    let body = wait_async_body();
    let object_prototype = result_object_prototype();
    assert_eq!(
        body.matches("self.emit_atomics_wait_async_return_object(")
            .count(),
        2
    );
    assert_eq!(
        body.matches("self.emit_atomics_wait_async_return_promise(")
            .count(),
        1
    );
    assert!(!body.contains("result_realm_context"));

    let synchronous = synchronous_result();
    let asynchronous = asynchronous_result();
    for result in [synchronous, asynchronous] {
        assert_eq!(
            result
                .matches("self.emit_atomics_wait_async_result_object_prototype(function)")
                .count(),
            1
        );
        assert!(result.contains(
            "emit_alloc_plain_object_with_prototype(Some(object_prototype_local), None, function)"
        ));
        assert_eq!(
            result
                .matches("self.emit_object_define_local_data_with_flags(")
                .count(),
            2
        );
        assert_eq!(
            result
                .matches("true,\n            true,\n            true,")
                .count(),
            2
        );
        let async_offset = result.find("\"async\"").expect("async property");
        let value_offset = result.find("\"value\"").expect("value property");
        assert!(
            async_offset < value_offset,
            "async must be defined before value"
        );
        assert!(!result.contains("OBJECT_PROTOTYPE_GLOBAL_INDEX"));
        assert!(!result.contains("emit_object_define_bool_data"));
    }
    assert!(
        asynchronous.contains("promise_allocation_context,\n            promise_payload_local,")
    );
    assert!(!asynchronous.contains("emit_current_realm_promise_allocation_context"));
    assert!(!asynchronous.contains("PROMISE_PROTOTYPE_GLOBAL_INDEX"));
    assert!(!synchronous
        .contains("emit_current_function_realm_intrinsic_promise_allocation_context(function)"));
    assert_eq!(
        asynchronous
            .matches("emit_current_function_realm_intrinsic_promise_allocation_context(function)")
            .count(),
        1
    );
    assert_eq!(
        WAIT_ASYNC_RESULT_SOURCE
            .matches("emit_atomics_wait_async_result_object_prototype(")
            .count(),
        3
    );
    assert!(!WAIT_ASYNC_RESULT_SOURCE.contains("unreachable!("));

    let synchronous_context_offset = synchronous
        .find("emit_atomics_wait_async_result_object_prototype(function)")
        .expect("synchronous result context");
    assert!(
        synchronous.find("let value_tag_local").expect("work local") < synchronous_context_offset
    );
    let asynchronous_context_offset = asynchronous
        .find("emit_atomics_wait_async_result_object_prototype(function)")
        .expect("asynchronous result context");
    assert!(
        asynchronous
            .find("let value_tag_local")
            .expect("work local")
            < asynchronous_context_offset
    );
    let promise_context_offset = asynchronous
        .find("emit_current_function_realm_intrinsic_promise_allocation_context(function)")
        .expect("Promise allocation context");
    let promise_allocation_offset = asynchronous
        .find("self.emit_alloc_promise_with_prototype(")
        .expect("Promise allocation");
    let object_witness_release_offset = asynchronous
        .find("self.release_temp_local(object_prototype_local)")
        .expect("object witness release");
    assert!(asynchronous_context_offset < promise_context_offset);
    assert!(promise_context_offset < promise_allocation_offset);
    assert!(promise_allocation_offset < object_witness_release_offset);

    let object_reservation_offset = object_prototype
        .find("let object_prototype_local = self.reserve_temp_local();")
        .expect("object prototype reservation");
    let realm_reservation_offset = object_prototype
        .find("let realm_local = self.reserve_temp_local();")
        .expect("Realm scratch reservation");
    let intrinsics_reservation_offset = object_prototype
        .find("let intrinsics_local = self.reserve_temp_local();")
        .expect("intrinsics scratch reservation");
    let intrinsics_release_offset = object_prototype
        .find("self.release_temp_local(intrinsics_local)")
        .expect("intrinsics scratch release");
    let realm_release_offset = object_prototype
        .find("self.release_temp_local(realm_local)")
        .expect("Realm scratch release");
    assert!(object_reservation_offset < realm_reservation_offset);
    assert!(realm_reservation_offset < intrinsics_reservation_offset);
    assert!(intrinsics_reservation_offset < intrinsics_release_offset);
    assert!(intrinsics_release_offset < realm_release_offset);
}

#[test]
fn entry_and_created_realm_atomics_functions_are_self_backed() {
    let entry = entry_realm_atomics_installer();
    let created = created_realm_atomics_installer();

    assert!(entry.contains("for builtin in ATOMICS_PUBLICATION_ORDER"));
    assert_eq!(
        entry.matches("self.emit_function_value_payload(").count(),
        1
    );
    assert_eq!(entry.matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET").count(), 1);
    assert!(entry.contains("self.emit_function_value_payload(&meta, function)?;"));
    assert!(entry.contains(
        "method_payload_local,\n                HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n                method_payload_local,"
    ));
    assert!(!entry.contains("self.emit_object_define_function_data("));
    let entry_materialization = entry
        .find("self.emit_function_value_payload(&meta, function)?;")
        .expect("entry Atomics function materialization");
    let entry_self_backing = entry
        .find("HEAP_FUNCTION_ENV_HANDLE_OFFSET")
        .expect("entry Atomics self environment");
    let entry_publication = entry
        .find("self.emit_object_define_local_data(")
        .expect("entry Atomics function publication");
    assert!(entry_materialization < entry_self_backing);
    assert!(entry_self_backing < entry_publication);

    assert!(created.contains("for builtin in ATOMICS_PUBLICATION_ORDER"));
    assert_eq!(
        created
            .matches("self.emit_function_value_payload_in_realm(")
            .count(),
        1
    );
    assert_eq!(
        created.matches("HEAP_FUNCTION_ENV_HANDLE_OFFSET").count(),
        1
    );
    assert!(created.contains("self.emit_function_value_payload_in_realm("));
    assert!(created.contains(
        "method_payload_local,\n                HEAP_FUNCTION_ENV_HANDLE_OFFSET,\n                method_payload_local,"
    ));
}

#[test]
fn focused_fixture_covers_three_nonblocking_created_realm_result_branches() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_preserves_created_realm_atomics_wait_async_results()"));
    assert!(CLI_TESTS.contains("wasm_atomics_wait_async_created_realm.js"));
    assert_eq!(CLI_FIXTURE.matches("waitAsync(view,").count(), 3);
    assert_eq!(CLI_FIXTURE.matches("other.Atomics.notify(view,").count(), 1);
    assert!(!CLI_FIXTURE.contains("Atomics.wait("));
    for marker in [
        "not-equal result",
        "timeout-zero result",
        "async result",
        "async Promise prototype",
        "async not entry-Realm Promise",
        "async Promise method Realm",
        "async writable",
        "async enumerable",
        "async configurable",
        "value writable",
        "value enumerable",
        "value configurable",
        "created-waitAsync:",
        "immediate notification outcome",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture control: {marker}"
        );
    }
    assert!(CLI_FIXTURE.contains("Object.keys(result).join(\",\")"));
    assert!(CLI_FIXTURE.contains("\"async,value\""));
}
