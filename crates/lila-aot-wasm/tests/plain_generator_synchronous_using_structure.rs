const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../lila-ir/src/analysis.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_using_plain_generator_lifecycle.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/plain-generator-synchronous-using-scope.md");
const EXACT_TEST262: &str = include_str!(
    "../../../test262/vendor/test262/test/language/statements/using/initializer-disposed-at-end-of-generatorbody.js"
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
fn ir_requires_one_closed_execution_owner_and_private_generator_capability() {
    let statement = bounded(
        IR_SOURCE,
        "    SyncDisposableScope {",
        "    ParameterInitialization {",
    );
    assert!(statement.contains("execution: SyncDisposableScopeExecutionIr"));
    assert!(statement.contains("resources: SyncDisposableResourcesIr"));
    assert!(statement.contains("body: BlockIr"));

    let owner = bounded(
        IR_SOURCE,
        "pub enum SyncDisposableScopeExecutionIr {",
        "/// The hidden activation binding for one plain-generator DisposeCapability.",
    );
    assert!(owner.contains("Immediate"));
    assert!(owner.contains("PlainGenerator(PlainGeneratorSyncDisposableCapabilityIr)"));
    assert!(owner.contains("AsyncFunction(AsyncFunctionSyncDisposableCapabilityIr)"));
    assert!(owner.contains("AsyncGenerator(AsyncGeneratorSyncDisposableCapabilityIr)"));
    assert!(!owner.contains("Option<"));
    assert!(!owner.contains("bool"));

    assert!(IR_SOURCE.contains(
        "#[must_use = \"a plain-generator synchronous DisposeCapability must be attached to its scope\"]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct PlainGeneratorSyncDisposableCapabilityIr {\n    binding_name: String,\n}"
    ));
    let capability = bounded(
        IR_SOURCE,
        "pub struct PlainGeneratorSyncDisposableCapabilityIr {",
        "/// A declaration-ordered, statically non-empty synchronous resource list.",
    );
    assert!(capability.contains("pub(crate) fn new(binding_name: String)"));
    assert!(capability.contains("pub fn binding_name(&self) -> &str"));
    assert!(!capability.contains("impl Copy"));
    assert!(!capability.contains("pub binding_name"));
}

#[test]
fn lowering_selects_and_allocates_the_owner_before_any_resource_initializer() {
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
    assert!(lower.contains("Some((\n            execution,"));

    let owner = bounded(
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
        "self.alloc_suspension_owned_binding(",
        "\"generator.dispose.capability.\"",
        "PlainGeneratorSyncDisposableCapabilityIr::new(binding_name)",
        "\"async.generator.dispose.capability.\"",
        "AsyncGeneratorSyncDisposableCapabilityIr::new(binding_name)",
    ] {
        assert!(owner.contains(marker), "missing owner boundary: {marker}");
    }
    assert_before(
        owner,
        "self.alloc_suspension_owned_binding(",
        "PlainGeneratorSyncDisposableCapabilityIr::new(binding_name)",
    );
    assert!(!owner.contains("_ =>"));
    assert_eq!(
        LOWERING_SOURCE
            .matches("PlainGeneratorSyncDisposableCapabilityIr::new(")
            .count(),
        1
    );

    let finish = bounded(
        LOWERING_SOURCE,
        "    fn finish_sync_disposable_scopes(",
        "    fn statement_list_ends_in_return(",
    );
    assert!(finish.contains("execution,"));
    assert!(finish.contains("StatementIr::SyncDisposableScope"));
    assert!(!finish.contains("StatementIr::TryFinally"));
}

#[test]
fn backend_exhaustively_selects_local_or_activation_backed_capability_storage() {
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
        "Self::PlainGenerator(_) => FunctionExecutionKind::Generator",
        "Self::PlainGenerator(_) => HEAP_GENERATOR_RESUME_STATE_OFFSET",
        "Self::PlainGenerator(_) => SyncDisposeCompletionContinuation::Dispatch",
        "Self::AsyncFunction(_) => FunctionExecutionKind::Async",
        "Self::AsyncGenerator(_) => FunctionExecutionKind::AsyncGenerator",
    ] {
        assert!(witnesses.contains(marker), "missing AOT witness: {marker}");
    }
    assert!(!witnesses.contains("derive(Clone"));
    assert!(!witnesses.contains("derive(Copy"));
    assert!(!CONTROL_FLOW_SOURCE.contains("impl Copy for ActivationSyncDisposeCapabilityStorage"));
    assert!(
        !CONTROL_FLOW_SOURCE.contains("impl Copy for ActiveActivationSyncDisposeCapabilityLocals")
    );
    assert!(!CONTROL_FLOW_SOURCE
        .contains("impl Copy for DetachedActivationSyncDisposeCapabilityLocals"));

    let dispatch = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_sync_disposable_scope(",
        "    fn compile_immediate_sync_disposable_scope(",
    );
    assert!(dispatch.contains("SyncDisposableScopeExecutionIr::Immediate =>"));
    assert!(dispatch.contains("SyncDisposableScopeExecutionIr::PlainGenerator(capability) =>"));
    assert!(dispatch.contains("ActivationSyncDisposeOwner::PlainGenerator(capability)"));
    assert!(dispatch.contains("SyncDisposableScopeExecutionIr::AsyncFunction(capability) =>"));
    assert!(dispatch.contains("ActivationSyncDisposeOwner::AsyncFunction(capability)"));
    assert!(dispatch.contains("SyncDisposableScopeExecutionIr::AsyncGenerator(capability) =>"));
    assert!(dispatch.contains("ActivationSyncDisposeOwner::AsyncGenerator(capability)"));
    assert!(dispatch.contains("compile_activation_sync_disposable_scope("));
    assert!(!dispatch.contains("_ =>"));
}

#[test]
fn generator_scope_publishes_once_retains_through_body_then_detaches_and_disposes_once() {
    let scope = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_activation_sync_disposable_scope(",
        "    fn initialize_sync_disposable_resource_bindings(",
    );
    for marker in [
        "owner.execution_kind()",
        "owner.binding_name()",
        "ActivationSyncDisposeOwner::PlainGenerator(_) =>",
        "Self::generator_statement_entry_state",
        "Self::generator_statement_exit_state",
        "emit_state_in_inclusive_range_i32(",
        "owner.resume_state_offset()",
        "initialize_activation_sync_dispose_capability(",
        "compile_generator_block_contents(body, entry_state, true, function)",
        "detach_activation_sync_dispose_capability(",
        "load_detached_activation_sync_disposable_resources(",
        "capture_pending_sync_dispose_completion(function)",
        "consume_sync_disposable_resources(",
        "owner.completion_continuation()",
        "release_detached_activation_sync_dispose_capability(detached)",
    ] {
        assert!(
            scope.contains(marker),
            "missing generator lifecycle: {marker}"
        );
    }
    assert_before(
        scope,
        "emit_state_in_inclusive_range_i32(",
        "initialize_activation_sync_dispose_capability(",
    );
    assert_before(
        scope,
        "initialize_activation_sync_dispose_capability(",
        "compile_generator_block_contents(body, entry_state, true, function)",
    );
    assert_before(
        scope,
        "compile_generator_block_contents(body, entry_state, true, function)",
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
    assert!(!scope.contains("reserve_sync_disposable_resource_locals(function)"));

    let initialize = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn initialize_activation_sync_dispose_capability(",
        "    fn append_activation_sync_disposable_resource(",
    );
    assert!(initialize.contains("storage: &ActivationSyncDisposeCapabilityStorage"));
    assert!(initialize.contains("DisposableStackState::Pending.word()"));
    assert_before(
        initialize,
        "self.write_binding_from_locals(storage.binding, capability.object, object_tag, function)",
        "for resource in resources.iter()",
    );
    assert_before(
        initialize,
        "self.compile_expr_to_locals(",
        "self.acquire_sync_disposable_resource_from_locals(",
    );
    assert_before(
        initialize,
        "self.acquire_sync_disposable_resource_from_locals(",
        "self.append_activation_sync_disposable_resource(",
    );
    assert_before(
        initialize,
        "self.append_activation_sync_disposable_resource(",
        "self.write_binding_from_locals(\n                resource_storage",
    );

    let append = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn append_activation_sync_disposable_resource(",
        "    fn detach_activation_sync_dispose_capability(",
    );
    assert_before(
        append,
        "LocalGet(resource.registered)",
        "HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET",
    );
    assert!(append.contains("DisposableStackEntryKind::Use.word()"));

    let detach = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn detach_activation_sync_dispose_capability(",
        "    fn load_detached_activation_sync_disposable_resources(",
    );
    assert!(detach.contains("storage: ActivationSyncDisposeCapabilityStorage"));
    assert_before(
        detach,
        "self.read_binding_to_locals(",
        "DisposableStackState::Disposed.word()",
    );
    assert_before(
        detach,
        "DisposableStackState::Disposed.word()",
        "HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,\n            0",
    );

    let load = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn load_detached_activation_sync_disposable_resources(",
        "    fn release_detached_activation_sync_dispose_capability(",
    );
    assert!(load.contains("index as i64"));
    assert!(load.contains("LocalGet(detached.entry_count)"));
    assert!(load.contains("Instruction::I64LtU"));
}

#[test]
fn planner_derives_nonoverlapping_active_body_and_detached_disposal_peaks() {
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
    assert!(count.contains("let acquisition_peak"));
    assert!(count.contains("let disposal_peak"));
    assert!(count.contains("acquisition_peak.max(disposal_peak).max(body_temps)"));
    assert!(!count.contains("_ =>"));
}

#[test]
fn durable_consumer_and_exact_two_execution_inventory_bound_the_claim() {
    for marker in [
        "normal before start",
        "normal while suspended",
        "normal completion LIFO",
        "return disposal",
        "injected throw identity",
        "acquisition failure disposal",
        "nested scope LIFO before outer completion",
        "outer SuppressedError",
        "suppressed exactly once",
    ] {
        assert!(
            FIXTURE.contains(marker),
            "missing fixture witness: {marker}"
        );
    }
    assert!(FIXTURE.contains("returned.return(42)"));
    assert!(FIXTURE.contains("thrown.throw(injectedError)"));
    assert!(FIXTURE.contains("combined.suppressed.suppressed, bodyError"));

    assert!(EXACT_TEST262.contains("features: [explicit-resource-management]"));
    assert!(EXACT_TEST262.contains("function * f()"));
    assert!(EXACT_TEST262.contains("wasDisposedBeforeGeneratorStarted"));
    assert!(EXACT_TEST262.contains("wasDisposedWhileSuspended"));
    assert!(EXACT_TEST262.contains("isDisposedAfterGeneratorCompleted"));
    assert!(!EXACT_TEST262.contains("flags:"));

    assert!(CONTRACT
        .contains("language/statements/using/initializer-disposed-at-end-of-generatorbody.js"));
    assert!(CONTRACT.contains("reports `0/2` under Wasm AOT"));
    assert!(CONTRACT.contains("complete `using` tree"));
    for exclusion in [
        "classic-`for`",
        "for-of",
        "async functions",
        "async generators",
        "`await using`",
        "modules",
        "dynamic source",
    ] {
        assert!(
            CONTRACT.contains(exclusion),
            "missing nonclaim: {exclusion}"
        );
    }
}
