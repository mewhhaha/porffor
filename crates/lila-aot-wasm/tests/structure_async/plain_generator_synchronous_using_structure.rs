const IR_SOURCE: &str = include_str!("../../../lila-ir/src/ir.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../../lila-ir/src/analysis.rs");
const LOWERING_SOURCE: &str = include_str!("../../../lila-ir/src/lowering.rs");
const MODULE_LOWERING_SOURCE: &str =
    include_str!("../../../lila-ir/src/lowering/module_execution.rs");
const ASYNC_LOWERING_SOURCE: &str =
    include_str!("../../../lila-ir/src/lowering/async_disposable.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../../src/control_flow.rs");
const FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_using_plain_generator_lifecycle.js");
const CONTRACT: &str = include_str!(
    "../../../../docs/rust-rewrite/contracts/plain-generator-synchronous-using-scope.md"
);
const EXACT_TEST262: &str = include_str!(
    "../../../../test262/vendor/test262/test/language/statements/using/initializer-disposed-at-end-of-generatorbody.js"
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
    let admission = bounded(
        MODULE_LOWERING_SOURCE,
        "    pub(super) fn admit_sync_disposable_scope_owner(",
        "    pub(super) fn lower_module_instantiation_boundary(",
    );
    assert!(admission.contains("owner.sync_disposable_scope_owner()"));
    assert!(admission.contains("matches!("));
    assert!(admission.contains("FunctionProtocolIr::ModuleActivation"));
    assert!(admission.contains("FunctionProtocolIr::AsyncModuleActivation"));
    assert!(admission.contains("owner.parent_owner_id.as_deref()"));
    assert!(admission.contains("return None;"));

    let lower = bounded(
        LOWERING_SOURCE,
        "    fn lower_using_declaration(",
        "    fn sync_disposable_scope_execution(",
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
        "    fn sync_disposable_scope_execution(",
        "    fn hoist_root_statement_items(",
    );
    for marker in [
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
        ASYNC_LOWERING_SOURCE,
        "    pub(super) fn finish_disposable_scopes(",
        "    pub(super) fn lower_await_using_declaration(",
    );
    assert!(finish.contains("LoweredDisposableScopeIr::Sync"));
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
