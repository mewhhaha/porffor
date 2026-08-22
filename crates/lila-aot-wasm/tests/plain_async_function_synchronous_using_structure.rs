const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../lila-ir/src/analysis.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_using_plain_async_function_lifecycle.js");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/plain-async-function-synchronous-using-scope.md"
);
const EXACT_TEST262: &str = include_str!(
    "../../../test262/vendor/test262/test/language/statements/using/initializer-disposed-at-end-of-asyncfunctionbody.js"
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
fn ir_requires_a_distinct_private_async_function_capability() {
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
        "#[must_use = \"a plain-async-function synchronous DisposeCapability must be attached to its scope\"]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AsyncFunctionSyncDisposableCapabilityIr {\n    binding_name: String,\n}"
    ));
    let capability = bounded(
        IR_SOURCE,
        "pub struct AsyncFunctionSyncDisposableCapabilityIr {",
        "/// A declaration-ordered, statically non-empty synchronous resource list.",
    );
    assert!(capability.contains("pub(crate) fn new(binding_name: String)"));
    assert!(capability.contains("pub fn binding_name(&self) -> &str"));
    assert!(!capability.contains("impl Copy"));
    assert!(!capability.contains("pub binding_name"));
}

#[test]
fn analysis_and_lowering_mint_one_suspension_owned_async_capability() {
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
    assert_before(
        lower,
        "let execution = self.sync_disposable_scope_execution(owner);",
        "for variable in list",
    );
    assert_before(
        lower,
        "for variable in list",
        "self.lower_expression(initializer)",
    );

    let selection = bounded(
        LOWERING_SOURCE,
        "    fn sync_disposable_scope_execution(",
        "    fn hoist_root_statement_items(",
    );
    for marker in [
        "SyncDisposableScopeOwnerPlan::Immediate =>",
        "SyncDisposableScopeOwnerPlan::PlainGenerator =>",
        "SyncDisposableScopeOwnerPlan::AsyncFunction =>",
        "SyncDisposableScopeOwnerPlan::AsyncGenerator =>",
        "self.alloc_suspension_owned_binding(",
        "\"async.dispose.capability.\"",
        "AsyncFunctionSyncDisposableCapabilityIr::new(binding_name)",
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
        "\"async.dispose.capability.\"",
        "AsyncFunctionSyncDisposableCapabilityIr::new(binding_name)",
    );
    assert!(!selection.contains("_ =>"));
    assert_eq!(
        LOWERING_SOURCE
            .matches("AsyncFunctionSyncDisposableCapabilityIr::new(")
            .count(),
        1
    );
}

#[test]
fn backend_owner_exhaustively_selects_async_state_and_completion_authority() {
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
        "PlainGenerator(&'a PlainGeneratorSyncDisposableCapabilityIr)",
        "AsyncFunction(&'a AsyncFunctionSyncDisposableCapabilityIr)",
        "Self::AsyncFunction(_) => FunctionExecutionKind::Async",
        "Self::AsyncFunction(_) => HEAP_ASYNC_RESUME_STATE_OFFSET",
        "Self::AsyncFunction(_) => SyncDisposeCompletionContinuation::DispatchAsyncFunction",
        "Self::AsyncGenerator(_) => FunctionExecutionKind::AsyncGenerator",
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
        "ActivationSyncDisposeOwner::PlainGenerator(capability)",
        "SyncDisposableScopeExecutionIr::AsyncFunction(capability) =>",
        "ActivationSyncDisposeOwner::AsyncFunction(capability)",
        "SyncDisposableScopeExecutionIr::AsyncGenerator(capability) =>",
        "ActivationSyncDisposeOwner::AsyncGenerator(capability)",
        "compile_activation_sync_disposable_scope(",
    ] {
        assert!(dispatch.contains(marker), "missing AOT dispatch: {marker}");
    }
    assert!(!dispatch.contains("_ =>"));
}

#[test]
fn async_state_traversal_enters_only_the_async_owned_scope_body() {
    let async_entry = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn async_statement_entry_state(statement: &StatementIr) -> Option<u32> {",
        "    fn async_statement_exit_state(statement: &StatementIr) -> Option<u32> {",
    );
    assert!(async_entry.contains("SyncDisposableScopeExecutionIr::AsyncFunction(_)"));
    assert!(async_entry.contains("| SyncDisposableScopeExecutionIr::AsyncGenerator(_)"));
    assert!(async_entry.contains("body,"));
    assert!(async_entry.contains("find_map(Self::async_statement_entry_state)"));
    assert!(async_entry.contains(
        "SyncDisposableScopeExecutionIr::Immediate\n                    | SyncDisposableScopeExecutionIr::PlainGenerator(_)"
    ));

    let generator_entry = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn generator_statement_entry_state(statement: &StatementIr) -> Option<u32> {",
        "    fn generator_statement_exit_state(statement: &StatementIr) -> Option<u32> {",
    );
    assert!(generator_entry.contains(
        "execution: SyncDisposableScopeExecutionIr::PlainGenerator(_),\n                body,"
    ));
    assert!(generator_entry.contains("find_map(Self::generator_statement_entry_state)"));
    assert!(generator_entry.contains(
        "SyncDisposableScopeExecutionIr::Immediate\n                    | SyncDisposableScopeExecutionIr::AsyncFunction(_)\n                    | SyncDisposableScopeExecutionIr::AsyncGenerator(_)"
    ));
}

#[test]
fn async_scope_initializes_once_retains_through_await_then_disposes_before_dispatch() {
    let scope = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_activation_sync_disposable_scope(",
        "    fn initialize_sync_disposable_resource_bindings(",
    );
    for marker in [
        "owner.execution_kind()",
        "owner.binding_name()",
        "owned_env_slot(owner.binding_name())",
        "activation-backed synchronous DisposeCapability is missing its owned binding",
        "BindingStorage::EnvSlot { slot, hops: 0 }",
        "ActivationSyncDisposeCapabilityStorage { binding }",
        "ActivationSyncDisposeOwner::AsyncFunction(_) =>",
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
    assert_before(
        scope,
        "activation-backed synchronous DisposeCapability is missing its owned binding",
        "ActivationSyncDisposeCapabilityStorage { binding }",
    );
    assert!(!scope.contains("self.allocate_binding("));
    assert_before(
        scope,
        "emit_state_in_inclusive_range_i32(",
        "initialize_activation_sync_dispose_capability(",
    );
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
        "SyncDisposeCompletionContinuation::DispatchAsyncFunction =>",
    );
    assert_before(
        consume,
        "SyncDisposeCompletionContinuation::DispatchAsyncFunction =>",
        "self.emit_dispatch_async_completion(function)?",
    );
}

#[test]
fn planner_derives_one_shared_activation_capability_peak_exhaustively() {
    let constants = bounded(
        PLANNING_SOURCE,
        "const ACTIVATION_SYNC_DISPOSE_DETACHED_TEMP_LOCALS",
        "const SUPER_PROPERTY_MUTATION_PERSISTENT_TEMP_LOCALS",
    );
    assert!(constants.contains("usize = 5"));
    assert!(constants.contains("ACTIVATION_SYNC_DISPOSE_ACTIVE_TEMP_LOCALS: usize = 3 + 5"));

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
fn durable_consumer_and_exact_two_execution_inventory_bound_the_claim() {
    for marker in [
        "normal before call",
        "normal while suspended",
        "normal completion LIFO",
        "return disposal before resolution",
        "source throw disposal before rejection",
        "rejected await disposal",
        "acquisition failure disposal",
        "nested scopes",
        "outer SuppressedError",
        "suppressed exactly once",
        "using-plain-async-function:true",
    ] {
        assert!(
            FIXTURE.contains(marker),
            "missing fixture witness: {marker}"
        );
    }
    assert!(FIXTURE.contains("await normalPromise"));
    assert!(FIXTURE.contains("await Promise.reject(error)"));
    assert!(FIXTURE.contains("combined.suppressed.suppressed, bodyError"));
    assert!(!FIXTURE.contains("await using"));
    assert!(!FIXTURE.contains("async function*"));

    assert!(EXACT_TEST262.contains("flags: [async]"));
    assert!(EXACT_TEST262.contains("features: [explicit-resource-management]"));
    assert!(EXACT_TEST262.contains("async function f()"));
    assert!(EXACT_TEST262.contains("wasDisposedWhileSuspended1"));
    assert!(EXACT_TEST262.contains("wasDisposedWhileSuspended2"));
    assert!(EXACT_TEST262.contains("isDisposedAfterCompleted"));

    assert!(CONTRACT
        .contains("language/statements/using/initializer-disposed-at-end-of-asyncfunctionbody.js"));
    assert!(CONTRACT.contains("reports `0/2` under Wasm AOT"));
    assert!(CONTRACT.contains("complete `using` tree"));
    for exclusion in [
        "async generators",
        "classic-`for`",
        "for-of",
        "`await using`",
        "resource-initializer suspension",
        "modules",
        "dynamic source",
    ] {
        assert!(
            CONTRACT.contains(exclusion),
            "missing contract nonclaim: {exclusion}"
        );
    }
}
