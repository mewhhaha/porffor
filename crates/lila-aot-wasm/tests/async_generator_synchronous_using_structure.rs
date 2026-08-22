const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../lila-ir/src/analysis.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const TEST262_RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_using_async_generator_lifecycle.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/async-generator-synchronous-using-scope.md");
const EXACT_PATH: &str =
    "language/statements/using/initializer-disposed-at-end-of-asyncgeneratorbody.js";
const EXACT_TEST262: &str = include_str!(
    "../../../test262/vendor/test262/test/language/statements/using/initializer-disposed-at-end-of-asyncgeneratorbody.js"
);

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
fn ir_requires_a_distinct_private_async_generator_capability() {
    let owner = bounded(
        IR_SOURCE,
        "pub enum SyncDisposableScopeExecutionIr {",
        "/// The hidden activation binding for one plain-generator DisposeCapability.",
    );
    for marker in [
        "Immediate",
        "PlainGenerator(PlainGeneratorSyncDisposableCapabilityIr)",
        "AsyncFunction(AsyncFunctionSyncDisposableCapabilityIr)",
        "AsyncGenerator(AsyncGeneratorSyncDisposableCapabilityIr)",
    ] {
        assert!(owner.contains(marker), "missing execution owner: {marker}");
    }
    assert!(!owner.contains("Option<"));
    assert!(!owner.contains("bool"));
    assert!(!owner.contains("_ =>"));

    assert!(IR_SOURCE.contains(
        "#[must_use = \"an async-generator synchronous DisposeCapability must be attached to its scope\"]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AsyncGeneratorSyncDisposableCapabilityIr {\n    binding_name: String,\n}"
    ));
    let capability = bounded(
        IR_SOURCE,
        "pub struct AsyncGeneratorSyncDisposableCapabilityIr {",
        "/// A declaration-ordered, statically non-empty synchronous resource list.",
    );
    assert!(capability.contains("pub(crate) fn new(binding_name: String)"));
    assert!(capability.contains("pub fn binding_name(&self) -> &str"));
    assert!(!capability.contains("impl Copy"));
    assert!(!capability.contains("pub binding_name"));
}

#[test]
fn lowering_selects_owner_and_rejects_initializer_suspension_before_lowering() {
    let analyzed_owner = bounded(
        ANALYSIS_SOURCE,
        "pub(crate) enum SyncDisposableScopeOwnerPlan {",
        "#[derive(Debug, Clone)]\npub(crate) struct Analysis",
    );
    for marker in [
        "Immediate",
        "PlainGenerator",
        "AsyncFunction",
        "AsyncGenerator",
        "FunctionExecutionKind::Ordinary => SyncDisposableScopeOwnerPlan::Immediate",
        "FunctionExecutionKind::Generator => SyncDisposableScopeOwnerPlan::PlainGenerator",
        "FunctionExecutionKind::Async => SyncDisposableScopeOwnerPlan::AsyncFunction",
        "FunctionExecutionKind::AsyncGenerator => SyncDisposableScopeOwnerPlan::AsyncGenerator",
    ] {
        assert!(
            analyzed_owner.contains(marker),
            "missing analyzed owner boundary: {marker}"
        );
    }
    assert!(!analyzed_owner.contains("_ =>"));

    let lower = bounded(
        LOWERING_SOURCE,
        "    fn lower_using_declaration(",
        "    /// Selects the only legal lifetime for an ordinary statement-list `using`.",
    );
    for marker in [
        "let owner = self.sync_disposable_scope_owner()",
        "owner == SyncDisposableScopeOwnerPlan::AsyncGenerator",
        "ContainsSymbol::AwaitExpression",
        "ContainsSymbol::YieldExpression",
        "suspension inside an async-generator using initializer",
        "let execution = self.sync_disposable_scope_execution(owner)",
        "for variable in list",
        "self.lower_expression(initializer)",
    ] {
        assert!(lower.contains(marker), "missing lowering guard: {marker}");
    }
    assert_before(
        lower,
        "owner == SyncDisposableScopeOwnerPlan::AsyncGenerator",
        "let execution = self.sync_disposable_scope_execution(owner)",
    );
    assert_before(
        lower,
        "suspension inside an async-generator using initializer",
        "self.lower_expression(initializer)",
    );
    assert_before(
        lower,
        "let execution = self.sync_disposable_scope_execution(owner)",
        "for variable in list",
    );

    let selection = bounded(
        LOWERING_SOURCE,
        "    fn sync_disposable_scope_owner(",
        "    fn hoist_root_statement_items(",
    );
    for marker in [
        "function.sync_disposable_scope_owner()",
        "SyncDisposableScopeOwnerPlan::Immediate =>",
        "SyncDisposableScopeOwnerPlan::PlainGenerator =>",
        "SyncDisposableScopeOwnerPlan::AsyncFunction =>",
        "SyncDisposableScopeOwnerPlan::AsyncGenerator =>",
        "\"async.generator.dispose.capability.\"",
        "AsyncGeneratorSyncDisposableCapabilityIr::new(binding_name)",
    ] {
        assert!(
            selection.contains(marker),
            "missing owner selection: {marker}"
        );
    }
    assert_before(
        selection,
        "\"async.generator.dispose.capability.\"",
        "AsyncGeneratorSyncDisposableCapabilityIr::new(binding_name)",
    );
    assert!(!selection.contains("_ =>"));
    assert_eq!(
        LOWERING_SOURCE
            .matches("AsyncGeneratorSyncDisposableCapabilityIr::new(")
            .count(),
        1
    );
}

#[test]
fn backend_owner_exhaustively_selects_async_generator_authority() {
    let witnesses = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"an activation-backed DisposeCapability binding must reach its consuming detach path\"]",
        "#[must_use = \"a synchronous iterator head must consume its iteration lifecycle\"]",
    );
    for marker in [
        "struct ActivationSyncDisposeCapabilityStorage",
        "struct ActiveActivationSyncDisposeCapabilityLocals",
        "struct DetachedActivationSyncDisposeCapabilityLocals",
        "enum ActivationSyncDisposeOwner<'a>",
        "AsyncGenerator(&'a AsyncGeneratorSyncDisposableCapabilityIr)",
        "Self::AsyncGenerator(_) => FunctionExecutionKind::AsyncGenerator",
        "Self::AsyncGenerator(_) => HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET",
        "Self::AsyncGenerator(_) => SyncDisposeCompletionContinuation::DispatchAsyncGenerator",
    ] {
        assert!(
            witnesses.contains(marker),
            "missing AOT authority: {marker}"
        );
    }
    assert!(!witnesses.contains("derive(Clone"));
    assert!(!witnesses.contains("derive(Copy"));
    assert!(!witnesses.contains("_ =>"));

    let dispatch = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_sync_disposable_scope(",
        "    fn compile_immediate_sync_disposable_scope(",
    );
    for marker in [
        "SyncDisposableScopeExecutionIr::Immediate =>",
        "SyncDisposableScopeExecutionIr::PlainGenerator(capability) =>",
        "SyncDisposableScopeExecutionIr::AsyncFunction(capability) =>",
        "SyncDisposableScopeExecutionIr::AsyncGenerator(capability) =>",
        "ActivationSyncDisposeOwner::AsyncGenerator(capability)",
        "compile_activation_sync_disposable_scope(",
    ] {
        assert!(dispatch.contains(marker), "missing AOT dispatch: {marker}");
    }
    assert!(!dispatch.contains("_ =>"));
}

#[test]
fn state_walkers_include_async_generator_body_but_never_generator_offsets() {
    let async_entry = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn async_statement_entry_state(statement: &StatementIr) -> Option<u32> {",
        "    fn async_statement_exit_state(statement: &StatementIr) -> Option<u32> {",
    );
    assert!(async_entry.contains("SyncDisposableScopeExecutionIr::AsyncFunction(_)"));
    assert!(async_entry.contains("| SyncDisposableScopeExecutionIr::AsyncGenerator(_)"));
    assert!(async_entry.contains("find_map(Self::async_statement_entry_state)"));

    let generator_entry = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn generator_statement_entry_state(statement: &StatementIr) -> Option<u32> {",
        "    fn generator_statement_exit_state(statement: &StatementIr) -> Option<u32> {",
    );
    assert!(generator_entry.contains(
        "execution: SyncDisposableScopeExecutionIr::PlainGenerator(_),\n                body,"
    ));
    assert!(generator_entry.contains("find_map(Self::generator_statement_entry_state)"));
    assert!(generator_entry.contains("| SyncDisposableScopeExecutionIr::AsyncGenerator(_)"));

    let suspension_scan = bounded(
        EMIT_SOURCE,
        "fn async_generator_contains_suspension(",
        "fn async_generator_dispatcher_unsupported_feature(",
    );
    assert!(
        suspension_scan.contains("execution: SyncDisposableScopeExecutionIr::AsyncGenerator(_)")
    );
    assert!(suspension_scan.contains("async_generator_contains_suspension(statement, suspension)"));

    let preflight = bounded(
        EMIT_SOURCE,
        "fn async_generator_dispatcher_unsupported_feature(",
        "pub(crate) fn async_generator_for_await_is_transparent_yield(",
    );
    assert!(preflight.contains("execution: SyncDisposableScopeExecutionIr::AsyncGenerator(_)"));
    assert!(preflight.contains("find_map(async_generator_dispatcher_unsupported_feature)"));
    assert!(!preflight.contains("Some(\"synchronous using scopes\")"));
}

#[test]
fn async_generator_scope_disposes_before_request_dispatch_and_queue_drain() {
    let scope = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_activation_sync_disposable_scope(",
        "    fn initialize_sync_disposable_resource_bindings(",
    );
    for marker in [
        "owner.execution_kind()",
        "owner.binding_name()",
        "BindingStorage::EnvSlot { slot, hops: 0 }",
        "ActivationSyncDisposeOwner::AsyncGenerator(_) =>",
        "Self::async_statement_entry_state",
        "Self::async_statement_exit_state",
        "owner.resume_state_offset()",
        "emit_state_in_inclusive_range_i32(",
        "initialize_activation_sync_dispose_capability(",
        "compile_async_block_contents(",
        "detach_activation_sync_dispose_capability(",
        "load_detached_activation_sync_disposable_resources(",
        "capture_pending_sync_dispose_completion(function)",
        "set_completion_kind(CompletionKind::Normal, function)",
        "consume_sync_disposable_resources(",
        "owner.completion_continuation()",
        "release_detached_activation_sync_dispose_capability(detached)",
    ] {
        assert!(scope.contains(marker), "missing lifecycle marker: {marker}");
    }
    assert!(!scope.contains("self.allocate_binding("));
    assert_before(
        scope,
        "initialize_activation_sync_dispose_capability(",
        "compile_async_block_contents(",
    );
    assert_before(
        scope,
        "compile_async_block_contents(",
        "detach_activation_sync_dispose_capability(",
    );
    assert_before(
        scope,
        "detach_activation_sync_dispose_capability(",
        "load_detached_activation_sync_disposable_resources(",
    );
    assert_before(
        scope,
        "load_detached_activation_sync_disposable_resources(",
        "capture_pending_sync_dispose_completion(function)",
    );
    assert_before(
        scope,
        "capture_pending_sync_dispose_completion(function)",
        "consume_sync_disposable_resources(",
    );
    assert_before(
        scope,
        "consume_sync_disposable_resources(",
        "release_detached_activation_sync_dispose_capability(detached)",
    );
    assert!(!scope.contains("emit_push_async_pending_completion("));

    let consume = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn consume_sync_disposable_resources(",
        "    pub(crate) fn compile_try_catch_finally(",
    );
    assert_before(
        consume,
        "self.restore_saved_completion(",
        "SyncDisposeCompletionContinuation::DispatchAsyncGenerator =>",
    );
    assert_before(
        consume,
        "SyncDisposeCompletionContinuation::DispatchAsyncGenerator =>",
        "self.emit_dispatch_async_generator_completion(function)",
    );
}

#[test]
fn planner_groups_all_activation_backed_owners_exhaustively() {
    let count = bounded(
        PLANNING_SOURCE,
        "fn count_sync_disposable_scope_temp_locals(",
        "pub(crate) fn count_expr_temp_locals(",
    );
    assert!(count.contains("SyncDisposableScopeExecutionIr::Immediate =>"));
    assert!(count.contains("SyncDisposableScopeExecutionIr::PlainGenerator(_)"));
    assert!(count.contains("| SyncDisposableScopeExecutionIr::AsyncFunction(_)"));
    assert!(count.contains("| SyncDisposableScopeExecutionIr::AsyncGenerator(_) =>"));
    assert!(count.contains("ACTIVATION_SYNC_DISPOSE_ACTIVE_TEMP_LOCALS"));
    assert!(count.contains("ACTIVATION_SYNC_DISPOSE_DETACHED_TEMP_LOCALS"));
    assert!(count.contains("acquisition_peak.max(disposal_peak).max(body_temps)"));
    assert!(!count.contains("_ =>"));
}

#[test]
fn exact_inventory_and_durable_consumer_bound_the_claim() {
    for marker in [
        "normal before start",
        "normal while yielded",
        "normal while awaiting",
        "normal completion LIFO",
        "external return disposal",
        "external throw disposal",
        "rejected await disposal",
        "acquisition failure disposal",
        "nested inner disposal",
        "outer SuppressedError",
        "disposal before settlement and drain",
        "reentrant disposal and drain",
        "reentrant exactly once",
        "using-async-generator:true",
    ] {
        assert!(
            FIXTURE.contains(marker),
            "missing fixture witness: {marker}"
        );
    }
    assert!(FIXTURE.contains("reentrantRequest = reentrant.next().then("));
    assert!(FIXTURE.contains("combined.suppressed.suppressed, bodyError"));
    assert!(!FIXTURE.contains("await using"));

    assert!(EXACT_TEST262.contains("flags: [async]"));
    assert!(EXACT_TEST262.contains("features: [explicit-resource-management]"));
    assert!(EXACT_TEST262.contains("async function * f()"));
    assert!(EXACT_TEST262.contains("wasDisposedBeforeAsyncGeneratorStarted"));
    assert!(EXACT_TEST262.contains("wasDisposedWhileSuspendedForYield"));
    assert!(EXACT_TEST262.contains("wasDisposedWhileSuspendedForAwait"));
    assert!(EXACT_TEST262.contains("isDisposedAfterCompleted"));
    assert!(!TEST262_RUNNER_SOURCE.contains(EXACT_PATH));
    assert!(!KNOWN_FAILURES.contains(EXACT_PATH));

    assert!(CONTRACT.contains(EXACT_PATH));
    assert!(CONTRACT.contains("reports `0/2` under Wasm AOT"));
    assert!(CONTRACT.contains("no Wasm-AOT source rewrite, mask, or backlog entry"));
    assert!(CONTRACT.contains("complete `using` tree"));
    for exclusion in [
        "`await using`",
        "async disposers",
        "resource-initializer suspension",
        "resource loop heads",
        "modules",
        "dynamic source",
        "nonlinear async-generator forms",
    ] {
        assert!(
            CONTRACT.contains(exclusion),
            "missing contract nonclaim: {exclusion}"
        );
    }
}
