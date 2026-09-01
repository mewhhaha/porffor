const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_proxy_delete_property.js");
const CLI_REGISTRATION: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/proxy-delete-traversal.md");
const TASK: &str = include_str!("../../../tasks/11-proxy-reflect-metaobject.md");

macro_rules! witness {
    ($path:literal) => {
        (
            $path,
            include_str!(concat!("../../../test262/vendor/test262/test/", $path)),
        )
    };
}

const VENDORED_WITNESSES: [(&str, &str); 3] = [
    witness!("built-ins/Proxy/deleteProperty/trap-is-missing-target-is-proxy.js"),
    witness!("built-ins/Proxy/deleteProperty/trap-is-null-target-is-proxy.js"),
    witness!("built-ins/Proxy/deleteProperty/trap-is-undefined-target-is-proxy.js"),
];

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn delete_traversal() -> &'static str {
    bounded(
        OBJECTS_SOURCE,
        "pub(crate) fn emit_object_delete(",
        "pub(crate) fn emit_delete_ordinary_by_tag(",
    )
}

#[test]
fn one_runtime_loop_replaces_source_generated_proxy_depth() {
    let traversal = delete_traversal();

    assert_eq!(
        traversal
            .matches("Instruction::Loop(BlockType::Empty)")
            .count(),
        1
    );
    for marker in [
        "const INSPECT_CURRENT_TARGET: i64 = 0;",
        "const DELETE_COMPLETE: i64 = 1;",
        "const FOLLOW_PROXY_TARGET: i64 = 2;",
        "let current_payload_local = self.reserve_temp_local();",
        "let current_tag_local = self.reserve_temp_local();",
        "let traversal_state_local = self.reserve_temp_local();",
    ] {
        assert!(
            traversal.contains(marker),
            "missing traversal marker: {marker}"
        );
    }
    assert!(!OBJECTS_SOURCE.contains("emit_object_delete_with_depth"));
    assert!(!traversal.contains("proxy_depth"));
    assert!(!traversal.contains("self.emit_object_delete("));
}

#[test]
fn nullish_traps_advance_the_typed_target_and_normal_paths_exit_once() {
    let traversal = delete_traversal();

    assert_before(
        traversal,
        "self.emit_load_live_proxy_slots(",
        "self.emit_object_read_without_throw_propagation(",
    );
    assert_before(
        traversal,
        "self.emit_object_read_without_throw_propagation(",
        "self.emit_propagate_throw_from_locals_if_needed(",
    );
    assert_before(
        traversal,
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.emit_is_callable_i32(",
    );
    assert_before(
        traversal,
        "self.emit_function_or_proxy_call_with_throw_propagation(",
        "self.emit_proxy_delete_invariant_check(",
    );
    assert_before(
        traversal,
        "self.emit_proxy_delete_invariant_check(",
        "Instruction::I64Const(DELETE_COMPLETE)",
    );

    let nullish_fallback = traversal
        .split_once("Instruction::I32Or")
        .expect("nullish trap classification")
        .1;
    assert_before(
        nullish_fallback,
        "Instruction::LocalGet(target_payload_local)",
        "Instruction::LocalSet(current_payload_local)",
    );
    assert_before(
        nullish_fallback,
        "Instruction::LocalGet(target_tag_local)",
        "Instruction::LocalSet(current_tag_local)",
    );
    assert_before(
        nullish_fallback,
        "Instruction::LocalSet(current_tag_local)",
        "Instruction::I64Const(FOLLOW_PROXY_TARGET)",
    );
    assert!(nullish_fallback.contains("Instruction::Br(1)"));
    assert!(traversal.contains(
        "self.emit_delete_ordinary_by_tag(\n            current_payload_local,\n            current_tag_local,"
    ));
}

#[test]
fn focused_fixture_crosses_six_nullish_proxy_targets_in_order() {
    assert_eq!(
        FIXTURE
            .matches("deepDeleteProxy = new Proxy(deepDeleteProxy, nullishDeleteHandler(")
            .count(),
        6
    );
    for marker in [
        "deepDeleteOrder = deepDeleteOrder * 10 + marker;",
        "deepDeleteOrder = deepDeleteOrder * 10 + 7;",
        "deepDeleteOrder !== 6543217",
        "deepDeleteCalls !== 1",
        "Object.prototype.hasOwnProperty.call(\n  deepDeleteTarget,\n  \"forwardedAcrossSixProxies\"",
    ] {
        assert!(FIXTURE.contains(marker), "missing fixture marker: {marker}");
    }
    assert!(CLI_REGISTRATION
        .contains("fn run_wasm_backend_succeeds_for_supported_proxy_delete_property_fixture()"));
    assert!(CLI_REGISTRATION.contains("fixture_path(\"wasm_proxy_delete_property.js\")"));
    assert!(CONTRACT.contains("six nullish forwarding handlers"));
    assert!(TASK.contains("six nullish forwarding handlers"));
}

#[test]
fn exact_nested_target_witnesses_retain_six_default_executions() {
    assert_eq!(VENDORED_WITNESSES.len(), 3);
    for (path, source) in VENDORED_WITNESSES {
        assert!(path.contains("target-is-proxy"));
        assert!(source.contains("features: [Proxy, Reflect]"));
        for single_mode in [
            "flags: [module]",
            "flags: [noStrict]",
            "flags: [onlyStrict]",
        ] {
            assert!(
                !source.contains(single_mode),
                "{path} must retain two modes"
            );
        }
    }
    assert!(CONTRACT.contains("three physical files and six executions"));
}
