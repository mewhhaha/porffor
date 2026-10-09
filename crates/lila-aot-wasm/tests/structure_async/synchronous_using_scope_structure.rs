const IR_SOURCE: &str = include_str!("../../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../../lila-ir/src/lowering.rs");
const ASYNC_LOWERING_SOURCE: &str =
    include_str!("../../../lila-ir/src/lowering/async_disposable.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../../src/control_flow.rs");
const FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_using_synchronous_scope.js");
const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/synchronous-using-scope-ir.md");

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
        ASYNC_LOWERING_SOURCE,
        "pub(super) enum LoweredStatementListItemIr {",
        "impl LoweredStatementListItemIr {",
    );
    assert!(marker.contains("SyncDisposableScope {"));
    assert!(marker.contains("execution: SyncDisposableScopeExecutionIr"));
    assert!(marker.contains("resources: SyncDisposableResourcesIr"));
    assert!(!LOWERING_SOURCE.contains("enum LoweredStatementListItemIr {"));

    let finish = bounded(
        ASYNC_LOWERING_SOURCE,
        "    pub(super) fn finish_disposable_scopes(",
        "    pub(super) fn lower_await_using_declaration(",
    );
    assert!(finish.contains("for (mut prefix, scope) in segments.into_iter().rev()"));
    assert!(finish.contains("LoweredDisposableScopeIr::Sync"));
    assert!(finish.contains("StatementIr::SyncDisposableScope"));
    assert!(finish.contains("body: suffix"));
    assert!(!finish.contains("StatementIr::TryFinally"));
    assert!(!finish.contains("StatementIr::Lexical"));
}

#[test]
fn acquisition_publishes_only_after_validation_then_initializes_the_binding() {
    let evaluate = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_sync_disposable_resource(",
        "    fn emit_control_flow_type_error(",
    );
    assert_before(
        evaluate,
        "self.compile_expr_to_value(",
        "self.emit_propagate_current_throw_if_needed(function)",
    );
    assert_before(
        evaluate,
        "self.emit_propagate_current_throw_if_needed(function)",
        "self.compile_sync_disposable_resource_from_locals(binding, locals, function)",
    );
    let acquire = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn acquire_sync_disposable_resource_from_locals(",
        "    fn compile_sync_disposable_resource_from_locals(",
    );
    for boundary in [
        "self.compile_nullish_tagged_i32(resource.value.tag(), function)",
        "RuntimeErrorMessage::USING_DECLARATION_RESOURCE_IS_NOT_AN_OBJECT",
        "lila_ir::WellKnownSymbol::Dispose",
        "PropertyKeyLocals::from_symbol(schema, &symbol, function)",
        "self.emit_object_read(&resource.value, &resource.value, &key, &pending, function)",
        "self.emit_propagate_current_throw_if_needed(function)",
        "resource.method.copy_from(pending.value(), function)",
        "RuntimeErrorMessage::USING_DECLARATION_RESOURCE_HAS_NO_SYMBOL_DISPOSE_METHOD",
        "self.emit_is_callable_i32(&resource.method, function)",
        "RuntimeErrorMessage::USING_DECLARATION_SYMBOL_DISPOSE_METHOD_IS_NOT_CALLABLE",
        "resource.registered.store(function)",
    ] {
        assert!(
            acquire.contains(boundary),
            "missing acquisition boundary: {boundary}"
        );
    }
    assert_before(
        acquire,
        "USING_DECLARATION_RESOURCE_IS_NOT_AN_OBJECT",
        "WellKnownSymbol::Dispose",
    );
    assert_before(
        acquire,
        "self.emit_object_read(",
        "self.emit_propagate_current_throw_if_needed(function)",
    );
    assert_before(
        acquire,
        "USING_DECLARATION_SYMBOL_DISPOSE_METHOD_IS_NOT_CALLABLE",
        "resource.registered.store(function)",
    );
    assert_eq!(
        acquire
            .matches("resource.registered.store(function)")
            .count(),
        1
    );
    assert_eq!(acquire.matches("self.emit_object_read(").count(), 1);
    let initialize = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_sync_disposable_resource_from_locals(",
        "    fn capture_pending_sync_dispose_completion(",
    );
    assert_before(
        initialize,
        "self.acquire_sync_disposable_resource_from_locals(resource, function)",
        "self.write_binding_from_locals(binding, &resource.value, function)",
    );
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
    assert!(CONTROL_FLOW_SOURCE.contains(
        "#[must_use = \"a captured using-scope completion must be restored and dispatched\"]\nstruct PendingSyncDisposeCompletionLocals"
    ));
    assert!(CONTROL_FLOW_SOURCE.contains(
        "#[must_use = \"an acquired using resource must be consumed by reverse disposal\"]\nstruct AcquiredSyncDisposableResourceLocals"
    ));
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
        "self.consume_sync_disposable_resources(",
    );

    let walk = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn consume_sync_disposable_resources(",
        "    pub(crate) fn compile_try_catch_finally(",
    );
    assert!(walk.contains("pending: PendingSyncDisposeCompletionLocals"));
    assert!(walk.contains("resources: Vec<AcquiredSyncDisposableResourceLocals>"));
    assert!(walk.contains("for resource in resources.iter().rev()"));
    assert!(walk.contains("self.emit_function_or_proxy_call_with_argv("));
    assert!(walk.contains("self.emit_alloc_suppressed_error_instance("));
    assert_before(
        walk,
        "resource.registered.store(function)",
        "self.emit_function_or_proxy_call_with_argv(",
    );
    let call = bounded(walk, "self.emit_function_or_proxy_call_with_argv(", ")?;");
    assert_before(call, "&resource.method", "&resource.value");
    assert_before(call, "&resource.value", "&arguments");
    assert_before(
        walk,
        "called.kind().load(function)",
        "pending.completion.kind().load(function)",
    );
    assert_before(
        walk,
        "pending.completion.kind().load(function)",
        "self.emit_alloc_suppressed_error_instance(",
    );
    let suppress = bounded(walk, "self.emit_alloc_suppressed_error_instance(", ")?;");
    assert_before(suppress, "called.value()", "pending.completion.value()");
    assert_before(
        walk,
        "self.emit_alloc_suppressed_error_instance(",
        "pending.completion.set_throw(combined.value(), function)",
    );
    assert!(walk.contains("pending.completion.copy_from(&called, function)"));
    assert!(
        !walk.contains("self.set_completion_kind("),
        "the separate called/combined completions must not overwrite the parked completion"
    );
    let restore = "self.completion().copy_from(&pending.completion, function)";
    assert_eq!(walk.matches(restore).count(), 1);
    assert_before(
        walk,
        "pending.completion.set_throw(combined.value(), function)",
        restore,
    );
    assert_before(walk, restore, "pending.completion.clear(function)");
    assert_before(
        walk,
        "pending.completion.clear(function)",
        "for resource in resources.into_iter().rev()",
    );
    assert_before(
        walk,
        "for resource in resources.into_iter().rev()",
        "self.release_sync_disposable_resource_locals(resource, function)",
    );
    assert_before(
        walk,
        "self.release_sync_disposable_resource_locals(resource, function)",
        "match continuation",
    );
    assert_eq!(
        walk.matches("self.emit_dispatch_current_completion(function)")
            .count(),
        1
    );
    let release = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn release_sync_disposable_resource_locals(",
        "    fn compile_sync_disposable_resource(",
    );
    assert!(release.contains("resource: AcquiredSyncDisposableResourceLocals"));
    assert_before(
        release,
        "resource.method.clear(function)",
        "resource.value.clear(function)",
    );
    assert_before(
        release,
        "resource.value.clear(function)",
        "release_i32_local(resource.registered, function)",
    );
    let capture = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn capture_pending_sync_dispose_completion(",
        "    fn consume_sync_disposable_resources(",
    );
    assert!(capture.contains("completion.copy_from(self.completion(), function)"));
    assert!(capture.contains("PendingSyncDisposeCompletionLocals { completion }"));

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
