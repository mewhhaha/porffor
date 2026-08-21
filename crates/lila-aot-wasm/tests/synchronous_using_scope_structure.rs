const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_using_synchronous_scope.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/synchronous-using-scope-ir.md");

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
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later, "{earlier} must precede {later}");
}

#[test]
fn ir_owns_one_statically_nonempty_synchronous_dispose_capability() {
    let statement = bounded(
        IR_SOURCE,
        "    SyncDisposableScope {",
        "    ParameterInitialization {",
    );
    assert!(statement.contains("resources: SyncDisposableResourcesIr"));
    assert!(statement.contains("body: BlockIr"));
    assert!(!statement.contains("Vec<SyncDisposableResourceIr>"));

    let resources = bounded(
        IR_SOURCE,
        "pub struct SyncDisposableResourcesIr {",
        "#[derive(Debug, Clone, PartialEq, Eq)]\npub enum AnnexBFunctionCopyTargetIr",
    );
    assert!(resources.contains("first: SyncDisposableResourceIr"));
    assert!(resources.contains("rest: Vec<SyncDisposableResourceIr>"));
    assert!(resources.contains("pub(crate) fn new("));
    assert!(resources.contains("first: SyncDisposableResourceIr"));
    assert!(resources.contains("impl DoubleEndedIterator"));
    assert!(resources.contains("pub fn is_empty(&self) -> bool {\n        false"));
}

#[test]
fn lowering_nests_reached_suffixes_without_generic_finally_or_double_initialization() {
    let marker = bounded(
        LOWERING_SOURCE,
        "enum LoweredStatementListItemIr {",
        "impl LoweredStatementListItemIr {",
    );
    assert!(marker.contains("SyncDisposableResources(SyncDisposableResourcesIr)"));

    let finish = bounded(
        LOWERING_SOURCE,
        "    fn finish_sync_disposable_scopes(",
        "    fn lower_block_function_declarations(",
    );
    assert!(finish.contains("for (mut prefix, resources) in segments.into_iter().rev()"));
    assert!(finish.contains("prefix.push(StatementIr::SyncDisposableScope"));
    assert!(finish.contains("body: suffix"));
    assert!(!finish.contains("StatementIr::TryFinally"));
    assert!(!finish.contains("StatementIr::Lexical"));
}

#[test]
fn acquisition_publishes_only_after_validation_then_initializes_the_binding() {
    let acquire = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_sync_disposable_resource(",
        "    fn capture_pending_sync_dispose_completion(",
    );
    for boundary in [
        "using declaration resource is not an object",
        "property_key_symbol_payload(\"Symbol.dispose\")",
        "using declaration resource has no [Symbol.dispose] method",
        "using declaration [Symbol.dispose] method is not callable",
        "LocalSet(locals.registered)",
        "self.write_binding_from_locals(",
    ] {
        assert!(
            acquire.contains(boundary),
            "missing acquisition boundary: {boundary}"
        );
    }
    assert_before(
        acquire,
        "compile_expr_to_locals(",
        "property_key_symbol_payload",
    );
    assert_before(
        acquire,
        "using declaration [Symbol.dispose] method is not callable",
        "LocalSet(locals.registered)",
    );
    assert_before(
        acquire,
        "LocalSet(locals.registered)",
        "self.write_binding_from_locals(",
    );
    assert_eq!(acquire.matches("LocalSet(locals.registered)").count(), 1);

    for witness in [
        "dispose method acquired once",
        "dispose getter observes TDZ",
        "TDZ resource disposed after initialization",
        "subsequent initializer identity",
    ] {
        assert!(
            FIXTURE.contains(witness),
            "missing acquisition witness: {witness}"
        );
    }
}

#[test]
fn noncopyable_completion_is_captured_walked_in_reverse_folded_and_restored_once() {
    let witnesses = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"a captured using-scope completion must be restored and dispatched\"]",
        "fn innermost_target(",
    );
    assert!(witnesses.contains("struct PendingSyncDisposeCompletionLocals"));
    assert!(witnesses.contains("struct AcquiredSyncDisposableResourceLocals"));
    assert!(!witnesses.contains("derive(Clone, Copy)"));
    assert!(!CONTROL_FLOW_SOURCE.contains("impl Copy for PendingSyncDisposeCompletionLocals"));
    assert!(!CONTROL_FLOW_SOURCE.contains("impl Copy for AcquiredSyncDisposableResourceLocals"));

    let scope = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_sync_disposable_scope(",
        "    fn reserve_sync_disposable_resource_locals(",
    );
    assert!(scope.contains("debug_assert!(!resources.is_empty())"));
    assert_before(
        scope,
        "self.finally_stack.push",
        "self.compile_sync_disposable_resource(",
    );
    assert_before(
        scope,
        "self.compile_sync_disposable_resource(",
        "self.compile_block_contents(body",
    );
    assert_before(
        scope,
        "self.compile_block_contents(body",
        "self.capture_pending_sync_dispose_completion(function)",
    );
    assert_before(
        scope,
        "self.capture_pending_sync_dispose_completion(function)",
        "self.set_completion_kind(CompletionKind::Normal",
    );
    assert_before(
        scope,
        "self.set_completion_kind(CompletionKind::Normal",
        "self.consume_sync_disposable_resources(pending, acquired",
    );

    let walk = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn consume_sync_disposable_resources(",
        "    pub(crate) fn compile_try_catch_finally(",
    );
    assert!(walk.contains("pending: PendingSyncDisposeCompletionLocals"));
    assert!(walk.contains("acquired: Vec<AcquiredSyncDisposableResourceLocals>"));
    assert!(walk.contains("for resource in acquired.iter().rev()"));
    assert!(walk.contains("emit_function_or_proxy_call_leave_throw_completion("));
    assert!(walk.contains("emit_alloc_suppressed_error_instance_from_locals("));
    assert_eq!(
        walk.matches("self.set_completion_kind(CompletionKind::Normal")
            .count(),
        2
    );
    assert!(walk.contains(
        "Instruction::LocalSet(pending.aux));\n            self.set_completion_kind(CompletionKind::Normal"
    ));
    let after_throw_capture = walk
        .split_once("Instruction::LocalSet(pending.aux));")
        .expect("throw completion capture")
        .1;
    assert_before(
        after_throw_capture,
        "self.set_completion_kind(CompletionKind::Normal",
        "emit_alloc_suppressed_error_instance_from_locals(",
    );
    assert_before(
        walk,
        "for resource in acquired.iter().rev()",
        "emit_alloc_suppressed_error_instance_from_locals(",
    );
    assert_before(
        walk,
        "LocalGet(pending.kind)",
        "emit_alloc_suppressed_error_instance_from_locals(",
    );
    assert_before(
        walk,
        "emit_alloc_suppressed_error_instance_from_locals(",
        "LocalSet(pending.payload)",
    );
    assert_before(
        walk,
        "LocalSet(pending.payload)",
        "self.restore_saved_completion(",
    );
    assert_before(
        walk,
        "self.restore_saved_completion(",
        "self.emit_dispatch_current_completion(function)",
    );
    assert_eq!(walk.matches("self.restore_saved_completion(").count(), 1);
    assert_eq!(
        walk.matches("self.emit_dispatch_current_completion(function)")
            .count(),
        1
    );
    assert_before(
        walk,
        "self.emit_dispatch_current_completion(function)",
        "self.release_temp_local(prototype_local)",
    );
    assert_before(
        walk,
        "self.release_temp_local(prototype_local)",
        "self.release_temp_local(call_result_payload_local)",
    );
    assert_before(
        walk,
        "self.release_temp_local(call_result_payload_local)",
        "self.release_temp_local(pending.aux)",
    );
    assert_before(
        walk,
        "self.release_temp_local(pending.aux)",
        "self.release_temp_local(pending.payload)",
    );
    assert_before(
        walk,
        "self.release_temp_local(pending.payload)",
        "for resource in acquired.into_iter().rev()",
    );
    assert_before(
        walk,
        "for resource in acquired.into_iter().rev()",
        "self.release_sync_disposable_resource_locals(resource)",
    );
    let release = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn release_sync_disposable_resource_locals(",
        "    fn compile_sync_disposable_resource(",
    );
    assert_before(release, "resource.method_tag", "resource.method_payload");
    assert_before(release, "resource.method_payload", "resource.value_tag");
    assert_before(release, "resource.value_tag", "resource.value_payload");
    assert_before(release, "resource.value_payload", "resource.registered");

    for witness in [
        "single error identity",
        "return completion preserved",
        "disposal replaces return",
        "all disposers continue",
        "outer SuppressedError",
        "inner suppressed",
    ] {
        assert!(
            FIXTURE.contains(witness),
            "missing completion witness: {witness}"
        );
    }
    assert!(CONTRACT.contains("This includes normal, throw, return,"));
    assert!(CONTRACT.contains("break, and continue completions."));
}
