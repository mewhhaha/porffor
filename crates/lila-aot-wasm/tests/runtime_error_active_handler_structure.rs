const ERROR_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const FIXTURE_SOURCE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_object_prevent_extensions_missing_writes.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/runtime-error-active-handler-routing.md");

fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("section start must exist");
    let tail = &source[start..];
    let end = tail.find(end).expect("section end must exist");
    &tail[..end]
}

#[test]
fn fresh_runtime_error_delegates_to_the_canonical_current_throw_router() {
    let wrapper = section(
        ERROR_SOURCE,
        "pub(crate) fn emit_throw_runtime_error_to_active_handler(",
        "pub(crate) fn emit_capture_throw_error_name(",
    );

    let create = wrapper
        .find("self.emit_throw_runtime_error(")
        .expect("wrapper must create the native error");
    let route = wrapper
        .find("self.emit_propagate_current_throw(function);")
        .expect("wrapper must delegate its published Throw completion");
    assert!(create < route);
    assert_eq!(wrapper.matches("emit_throw_runtime_error(").count(), 1);
    assert_eq!(
        wrapper
            .matches("emit_propagate_current_throw(function);")
            .count(),
        1
    );
    assert!(!wrapper.contains("is_main()"));
    assert!(!wrapper.contains("active_throw_target()"));
    assert!(!wrapper.contains("emit_return_current_completion(function)"));
}

#[test]
fn canonical_router_prefers_a_typed_active_target_and_returns_only_without_one() {
    let route = section(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn emit_propagate_current_throw(",
        "pub(crate) fn emit_break_current_completion_if_throw(",
    );

    let resolve = route
        .find("if let Some(target) = self.active_throw_target()")
        .expect("router must resolve the typed active target first");
    let branch = route
        .find("self.emit_branch_to_target(target, function);")
        .expect("Some(ControlTarget) must branch through the typed target");
    let fallback = route
        .find("self.emit_return_current_completion(function);")
        .expect("None must return the current Throw completion");
    assert!(resolve < branch && branch < fallback);
    assert!(route.contains("} else {"));
    assert!(!route.contains("is_main()"));
    assert!(!route.contains("Instruction::Br("));
}

#[test]
fn strict_array_index_failure_and_internal_catch_remain_the_consumer_contract() {
    let array_write = section(
        ARRAY_SOURCE,
        "pub(crate) fn emit_array_assignment_write(",
        "pub(crate) fn emit_array_inherited_index_set_state(",
    );
    assert!(array_write.contains(
        "self.emit_object_write_set_failure_else(\"Cannot assign to array index\", function)?;"
    ));

    assert!(FIXTURE_SOURCE.contains("function catchesStrictArrayIndexWriteInOwnBody(target)"));
    let internal_catch = section(
        FIXTURE_SOURCE,
        "function catchesStrictArrayIndexWriteInOwnBody(target)",
        "var internalStrictArrayIndexThrew =",
    );
    let strict = internal_catch
        .find("\"use strict\";")
        .expect("the caller function must create a strict Reference");
    let try_block = internal_catch
        .find("try {")
        .expect("the catch must belong to the same non-main function");
    let write = internal_catch
        .find("target[0] = \"strict-index-internal-catch\";")
        .expect("the failed array-index PutValue must occur inside the try");
    let catch = internal_catch
        .find("} catch (error) {")
        .expect("the same function must catch the fresh TypeError");
    assert!(strict < try_block && try_block < write && write < catch);
    assert!(internal_catch.contains("caught = error instanceof TypeError;"));
    assert!(FIXTURE_SOURCE.contains("&& internalStrictArrayIndexThrew === true"));

    let nested_finally = section(
        FIXTURE_SOURCE,
        "function catchesStrictArrayIndexWriteAfterNestedFinally(target)",
        "var internalStrictArrayIndexFinallyThrew =",
    );
    let nested_write = nested_finally
        .find("target[0] = \"strict-index-nested-finally\";")
        .expect("the failed PutValue must precede both finalizers");
    let inner_finally = nested_finally
        .find("trace += \"inner-finally,\";")
        .expect("the inner finalizer must run first");
    let outer_finally = nested_finally
        .find("trace += \"outer-finally,\";")
        .expect("the outer finalizer must run second");
    let outer_catch = nested_finally
        .find("trace += \"catch\";")
        .expect("the outer catch must receive the preserved Throw");
    assert!(nested_write < inner_finally);
    assert!(inner_finally < outer_finally);
    assert!(outer_finally < outer_catch);
    assert_eq!(nested_finally.matches("finally {").count(), 2);
    assert!(nested_finally.contains("caught instanceof TypeError"));
    assert!(nested_finally.contains("caught.message === \"Cannot assign to array index\""));
    assert!(FIXTURE_SOURCE.contains("&& internalStrictArrayIndexFinallyThrew === true"));

    assert!(CONTRACT.contains("built-ins/Object/preventExtensions/15.2.3.10-3-4.js"));
    assert!(CONTRACT.contains("reports `1/2` under Wasm AOT"));
    assert!(CONTRACT.contains("The complete `built-ins/Object/preventExtensions` leaf was `77/78`"));
    assert!(CONTRACT.contains("object-literal method `[[HomeObject]]`"));
    assert!(CONTRACT.contains("does not claim every throw/catch site"));
}
