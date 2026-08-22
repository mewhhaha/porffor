const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../lila-ir/src/analysis.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const ASYNC_LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering/async_disposable.rs");
const IR_TEST_SOURCE: &str = include_str!("../../lila-ir/src/lib.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_await_using_plain_async_function_lifecycle.js"
);
const CLI_TEST_SOURCE: &str = include_str!("../../lila-cli/tests/cli/resource_management.rs");
const TEST262_RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/plain-async-function-await-using-scope.md");
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

const EXACT_PATHS: [&str; 2] = [
    "language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-asyncfunctionbody.js",
    "language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-asyncfunctionbody.js",
];

macro_rules! plain_async_statement_list_inventory {
    ($($file:literal),+ $(,)?) => {
        const PLAIN_ASYNC_STATEMENT_LIST_FILES: [(&str, &str); 49] = [$(
            (
                $file,
                include_str!(concat!(
                    "../../../test262/vendor/test262/test/language/statements/await-using/",
                    $file
                )),
            ),
        )+];
    };
}

plain_async_statement_list_inventory!(
    "Symbol.asyncDispose-getter.js",
    "Symbol.asyncDispose-method-called-with-correct-this.js",
    "Symbol.asyncDispose-method-not-async.js",
    "Symbol.dispose-getter.js",
    "Symbol.dispose-method-called-with-correct-this.js",
    "await-using-Symbol.asyncDispose-allows-non-promise-return-value.js",
    "await-using-Symbol.asyncDispose-allows-promiselike-return-value.js",
    "await-using-allows-null-initializer.js",
    "await-using-allows-undefined-initializer.js",
    "await-using-does-not-imply-await-if-not-evaluated.js",
    "await-using-implies-await-if-evaluated.js",
    "block-local-closure-get-before-initialization.js",
    "block-local-use-before-initialization-in-declaration-statement.js",
    "block-local-use-before-initialization-in-prior-statement.js",
    "fn-name-arrow.js",
    "fn-name-class.js",
    "fn-name-cover.js",
    "fn-name-fn.js",
    "fn-name-gen.js",
    "function-local-closure-get-before-initialization.js",
    "function-local-use-before-initialization-in-declaration-statement.js",
    "function-local-use-before-initialization-in-prior-statement.js",
    "gets-initializer-Symbol.asyncDispose-property-once.js",
    "gets-initializer-Symbol.dispose-after-Symbol.asyncDispose-is-null.js",
    "gets-initializer-Symbol.dispose-after-Symbol.asyncDispose-is-undefined.js",
    "gets-initializer-Symbol.dispose-property-once.js",
    "gets-initializer-does-not-read-Symbol.dispose-if-Symbol.asyncDispose-exists.js",
    "global-closure-get-before-initialization.js",
    "global-use-before-initialization-in-declaration-statement.js",
    "global-use-before-initialization-in-prior-statement.js",
    "initializer-Symbol.asyncDispose-called-at-end-of-asyncfunctionbody.js",
    "initializer-Symbol.asyncDispose-called-at-end-of-block.js",
    "initializer-Symbol.asyncDispose-called-if-subsequent-initializer-throws.js",
    "initializer-Symbol.dispose-called-at-end-of-asyncfunctionbody.js",
    "initializer-Symbol.dispose-called-at-end-of-block.js",
    "initializer-Symbol.dispose-called-if-subsequent-initializer-throws.js",
    "multiple-resources-disposed-in-reverse-order.js",
    "puts-initializer-on-top-of-disposableresourcestack-multiple-bindings.js",
    "puts-initializer-on-top-of-disposableresourcestack-subsequent-usings.js",
    "throws-error-as-is-if-only-one-error-during-disposal.js",
    "throws-if-initializer-Symbol.asyncDispose-property-is-null.js",
    "throws-if-initializer-Symbol.asyncDispose-property-is-undefined.js",
    "throws-if-initializer-Symbol.asyncDispose-property-not-callable.js",
    "throws-if-initializer-Symbol.dispose-property-is-null.js",
    "throws-if-initializer-Symbol.dispose-property-is-undefined.js",
    "throws-if-initializer-Symbol.dispose-property-not-callable.js",
    "throws-if-initializer-missing-both-Symbol.asyncDispose-and-Symbol.dispose.js",
    "throws-if-initializer-not-object.js",
    "throws-suppressederror-if-multiple-errors-during-disposal.js",
);

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("start marker");
    let tail = &source[start..];
    let end = tail.find(end).expect("end marker");
    &tail[..end]
}

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..].find(marker).expect(marker);
        cursor += offset + marker.len();
    }
}

#[test]
fn ir_owns_a_nonempty_async_resource_domain_and_closed_finalizer_states() {
    let statement = bounded(
        IR_SOURCE,
        "AsyncDisposableScope {",
        "ParameterInitialization {",
    );
    assert!(statement.contains("execution: AsyncDisposableScopeExecutionIr"));
    assert!(statement.contains("resources: AsyncDisposableResourcesIr"));
    assert!(statement.contains("body: BlockIr"));

    let resource = bounded(
        IR_SOURCE,
        "pub struct AsyncDisposableResourceIr {",
        "pub struct AsyncDisposableResourcesIr {",
    );
    assert!(resource.contains("binding_name: String"));
    assert!(resource.contains("initializer: TypedExpr"));
    assert!(resource.contains("pub(crate) fn new("));
    assert!(resource.contains("pub fn binding_name(&self) -> &str"));
    assert!(resource.contains("pub fn initializer(&self) -> &TypedExpr"));
    assert!(!resource.contains("pub binding_name"));
    assert!(!resource.contains("SyncDisposableResourceIr"));

    let resources = bounded(
        IR_SOURCE,
        "pub struct AsyncDisposableResourcesIr {",
        "pub struct AsyncDisposableFinalizerPlanIr {",
    );
    assert!(resources.contains("first: AsyncDisposableResourceIr"));
    assert!(resources.contains("rest: Vec<AsyncDisposableResourceIr>"));
    assert!(resources.contains("pub(crate) fn new("));
    assert!(resources.contains("DoubleEndedIterator<Item = &AsyncDisposableResourceIr>"));
    assert!(resources.contains("pub fn is_empty(&self) -> bool {\n        false"));

    assert!(IR_SOURCE.contains(
        "#[must_use = \"an async-dispose finalizer plan must be attached to its capability\"]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AsyncDisposableFinalizerPlanIr"
    ));
    let finalizer = bounded(
        IR_SOURCE,
        "pub struct AsyncDisposableFinalizerPlanIr {",
        "pub struct AsyncFunctionAsyncDisposableCapabilityIr {",
    );
    for role in ["entry_state", "dispose_state", "resume_state", "exit_state"] {
        assert!(finalizer.contains(&format!("{role}: u32")));
        assert!(finalizer.contains(&format!("pub fn {role}(&self) -> u32")));
    }
    assert!(finalizer.contains(
        "entry_state < dispose_state\n                && dispose_state < resume_state\n                && resume_state < exit_state"
    ));
    assert!(!finalizer.contains("Copy"));

    assert!(IR_SOURCE.contains(
        "#[must_use = \"a plain-async-function async DisposeCapability must be attached to its scope\"]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AsyncFunctionAsyncDisposableCapabilityIr"
    ));
    let capability = bounded(
        IR_SOURCE,
        "pub struct AsyncFunctionAsyncDisposableCapabilityIr {",
        "impl SyncDisposableResourcesIr",
    );
    assert!(capability.contains("binding_name: String"));
    assert!(capability.contains("finalizer: AsyncDisposableFinalizerPlanIr"));
    assert!(capability.contains("pub(crate) fn new("));
    assert!(capability.contains("pub fn binding_name(&self) -> &str"));
    assert!(capability.contains("pub fn finalizer(&self) -> &AsyncDisposableFinalizerPlanIr"));
    assert!(!capability.contains("Copy"));
}

#[test]
fn lowering_selects_the_plain_async_owner_before_minting_one_finalizer() {
    let owner = bounded(
        ANALYSIS_SOURCE,
        "pub(crate) enum AsyncDisposableScopeOwnerPlan {",
        "#[derive(Debug, Clone)]",
    );
    for variant in ["Ordinary", "Generator", "AsyncFunction", "AsyncGenerator"] {
        assert!(owner.contains(variant));
    }
    for mapping in [
        "FunctionExecutionKind::Ordinary => AsyncDisposableScopeOwnerPlan::Ordinary",
        "FunctionExecutionKind::Generator => AsyncDisposableScopeOwnerPlan::Generator",
        "FunctionExecutionKind::Async => AsyncDisposableScopeOwnerPlan::AsyncFunction",
        "FunctionExecutionKind::AsyncGenerator => AsyncDisposableScopeOwnerPlan::AsyncGenerator",
    ] {
        assert!(owner.contains(mapping));
    }

    let lower = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) fn lower_await_using_declaration(",
        "fn async_disposable_scope_owner(&self)",
    );
    positions_in_order(
        lower,
        &[
            "suspension inside an await using initializer",
            "let owner = self.async_disposable_scope_owner()",
            "match owner",
            "AsyncDisposableScopeOwnerPlan::AsyncFunction\n            | AsyncDisposableScopeOwnerPlan::AsyncGenerator => {}",
            "let entry_state = self",
            "let execution = match owner",
            "AsyncDisposableScopeOwnerPlan::AsyncFunction =>",
            "alloc_suspension_owned_binding(",
            "async.function.async.dispose.capability.",
            "let init = self.lower_expression(initializer)",
            "into_async_disposable_resource(self)",
            "AsyncDisposableResourcesIr::new(first, resources.collect())",
        ],
    );
    assert!(lower.contains("AsyncDisposableScopeOwnerPlan::Ordinary =>"));
    assert!(lower.contains("AsyncDisposableScopeOwnerPlan::Generator =>"));
    assert!(lower.contains("PendingAsyncDisposableScopeExecutionIr::AsyncGenerator"));
    assert!(!lower.contains("SyncDisposableResourcesIr::new"));

    let finish = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) fn finish_disposable_scopes(",
        "pub(super) fn lower_await_using_declaration(",
    );
    positions_in_order(
        finish,
        &[
            "for (mut prefix, scope) in segments.into_iter().rev()",
            "LoweredDisposableScopeIr::Async(scope) =>",
            "let dispose_state = self",
            "let resume_state = dispose_state",
            "let exit_state = resume_state",
            "self.current_async_resume_state = Some(exit_state)",
            "AsyncDisposableFinalizerPlanIr::new(",
            "StatementIr::AsyncDisposableScope",
        ],
    );
    assert!(finish.matches("checked_add(1)").count() >= 3);
    assert!(IR_TEST_SOURCE
        .contains("fn plain_async_function_await_using_owns_closed_finalizer_states()"));
    assert!(LOWERING_SOURCE.contains("mod async_disposable;"));
}

#[test]
fn backend_typestates_and_closed_entry_kinds_own_the_async_lifecycle() {
    for declaration in [
        "struct ActivationAsyncDisposeCapabilityStorage",
        "struct ActiveActivationAsyncDisposeCapabilityLocals",
        "struct AcquiredAsyncDisposableResourceLocals",
        "struct DisposingActivationAsyncDisposeCapability",
        "struct ActiveAsyncDisposePendingCompletion",
    ] {
        assert!(CONTROL_FLOW_SOURCE.contains(declaration));
    }
    let typestates = bounded(
        CONTROL_FLOW_SOURCE,
        "struct ActivationAsyncDisposeCapabilityStorage",
        "enum ActivationSyncDisposeOwner",
    );
    assert_eq!(typestates.matches("#[must_use").count(), 6);
    assert!(typestates.contains("enum ActivationAsyncDisposeOwner<'a>"));
    assert!(!typestates.contains("Clone"));
    assert!(!typestates.contains("Copy"));

    let state = bounded(
        HEAP_SOURCE,
        "pub(crate) enum ActivationAsyncDisposeCapabilityState",
        "pub(crate) enum ActivationAsyncDisposeEntryKind",
    );
    for variant in ["Pending", "Disposing", "Disposed"] {
        assert!(state.contains(variant));
    }
    assert!(!state.contains("_ =>"));

    let kinds = bounded(
        HEAP_SOURCE,
        "pub(crate) enum ActivationAsyncDisposeEntryKind",
        "impl AsyncDisposableStackEntryKind",
    );
    for variant in ["Empty", "AsyncMethod", "SyncFallbackMethod"] {
        assert!(kinds.contains(variant));
    }
    assert!(kinds.contains("pub(crate) const ALL: [Self; 3]"));
    assert!(!kinds.contains("_ =>"));

    let compile = bounded(
        CONTROL_FLOW_SOURCE,
        "fn compile_async_disposable_scope(",
        "fn initialize_async_disposable_resource_bindings(",
    );
    assert!(compile.contains("ActivationAsyncDisposeOwner::from_execution(execution)"));
    assert!(compile.contains("meta.protocol.execution_kind() == owner.execution_kind()"));
    assert!(compile.contains("owned_env_slot(owner.binding_name())"));
    assert!(!compile.contains("allocate_binding(owner.binding_name"));
    positions_in_order(
        compile,
        &[
            "finalizer.entry_state()",
            "finalizer.exit_state()",
            "self.finally_stack.push(disposal_frame)",
            "finalizer.entry_state()",
            "finalizer.dispose_state()",
            "initialize_activation_async_dispose_capability",
            "compile_async_block_contents(",
            "begin_async_dispose_pending_completion",
            "begin_activation_async_dispose_capability",
            "consume_activation_async_dispose_capability",
        ],
    );
}

#[test]
fn acquisition_registers_before_binding_and_fallback_stays_distinct() {
    let initialize = bounded(
        CONTROL_FLOW_SOURCE,
        "fn initialize_activation_async_dispose_capability(",
        "fn reserve_async_disposable_resource_locals(",
    );
    positions_in_order(
        initialize,
        &[
            "ActivationAsyncDisposeCapabilityState::Pending.word()",
            "write_binding_from_locals(storage.binding, capability.object, object_tag, function)",
            "for resource in resources.iter()",
            "compile_expr_to_locals(",
            "emit_propagate_throw_from_locals_if_needed(",
            "acquire_async_disposable_resource_from_locals",
            "append_activation_async_disposable_resource",
            "write_binding_from_locals(\n                resource_storage",
        ],
    );

    let acquire = bounded(
        CONTROL_FLOW_SOURCE,
        "fn acquire_async_disposable_resource_from_locals(",
        "fn append_activation_async_disposable_resource(",
    );
    positions_in_order(
        acquire,
        &[
            "emit_is_nullish_tag_i32",
            "Symbol.asyncDispose",
            "emit_propagate_throw_from_locals_if_needed",
            "emit_is_nullish_tag_i32",
            "Symbol.dispose",
            "ActivationAsyncDisposeEntryKind::SyncFallbackMethod",
            "ActivationAsyncDisposeEntryKind::AsyncMethod",
        ],
    );
    assert!(acquire.contains("method is not callable"));
    assert!(acquire.contains("resource has no disposal method"));
}

#[test]
fn finalizer_awaits_empty_and_async_results_but_discards_sync_fallback_returns() {
    let begin = bounded(
        CONTROL_FLOW_SOURCE,
        "fn begin_activation_async_dispose_capability(",
        "fn fold_error_into_async_dispose_pending_completion(",
    );
    positions_in_order(
        begin,
        &[
            "ActivationAsyncDisposeCapabilityState::Disposing.word()",
            "HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET",
            "HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET",
            "HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET",
            "finalizer.dispose_state()",
        ],
    );

    let consume = bounded(
        CONTROL_FLOW_SOURCE,
        "fn consume_activation_async_dispose_capability(",
        "fn finish_async_dispose_pending_completion(",
    );
    positions_in_order(
        consume,
        &[
            "finalizer.resume_state()",
            "emit_load_activation_async_dispose_resume_is_throw",
            "fold_error_into_async_dispose_pending_completion",
            "finalizer.dispose_state()",
            "I64Sub",
            "HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET",
            "for entry_kind in ActivationAsyncDisposeEntryKind::ALL",
            "ActivationAsyncDisposeEntryKind::Empty => {}",
            "ActivationAsyncDisposeEntryKind::AsyncMethod =>",
            "ActivationAsyncDisposeEntryKind::SyncFallbackMethod =>",
            "emit_rejected_intrinsic_promise_from_error",
            "emit_set_async_resume_state(activation_local, finalizer.resume_state()",
            "emit_activation_async_dispose_await_reactions",
            "emit_return_current_completion",
            "ActivationAsyncDisposeCapabilityState::Disposed.word()",
            "finish_async_dispose_pending_completion",
            "finalizer.exit_state()",
            "emit_dispatch_activation_async_dispose_completion",
        ],
    );
    assert!(consume.contains("ValueKind::Undefined.tag()"));
    assert_eq!(
        consume
            .matches("for entry_kind in ActivationAsyncDisposeEntryKind::ALL")
            .count(),
        1
    );
    assert!(!consume.contains("_ =>"));

    let planning = bounded(
        PLANNING_SOURCE,
        "fn count_async_disposable_scope_temp_locals(",
        "pub(crate) fn count_expr_temp_locals(",
    );
    for marker in [
        "ACTIVATION_ASYNC_DISPOSE_ACTIVE_TEMP_LOCALS",
        "ACTIVATION_ASYNC_DISPOSE_WALKER_TEMP_LOCALS",
        "ACTIVATION_ASYNC_DISPOSE_HELPER_TEMP_LOCALS",
        "acquisition_peak.max(disposal_peak).max(body_temps)",
    ] {
        assert!(planning.contains(marker));
    }
}

#[test]
fn exact_inventory_and_durable_fixture_bound_the_claim() {
    assert_eq!(PLAIN_ASYNC_STATEMENT_LIST_FILES.len(), 49);
    for (file, source) in PLAIN_ASYNC_STATEMENT_LIST_FILES {
        assert!(
            source.contains("explicit-resource-management"),
            "unexpected inventory file {file}"
        );
    }

    for path in EXACT_PATHS {
        assert!(PLAIN_ASYNC_STATEMENT_LIST_FILES
            .iter()
            .any(|(file, _)| path.ends_with(file)));
        assert!(!TEST262_RUNNER_SOURCE.contains(path));
        assert!(!KNOWN_FAILURES.contains(path));
        assert!(CONTRACT.contains(path));
        assert!(TASK.contains(path));
    }

    for marker in [
        "same(fallback.thenReads(), 0, \"fallback thenable ignored\")",
        "acquisition:tdz:ReferenceError",
        "empty:after:false",
        "unreachable:after:true",
        "first waits for second",
        "body error identity",
        "disposer rejection identity",
        "outer suppressed error",
        "await-using-plain-async:true",
    ] {
        assert!(FIXTURE.contains(marker), "missing fixture marker {marker}");
    }
    assert!(CLI_TEST_SOURCE.contains("fn wasm_await_using_plain_async_function_lifecycle()"));
    assert!(README.contains("plain-async-function `await using` batch is implemented"));
    assert!(TASK.contains("plain-async-function `await using` batch is implemented"));
    assert!(README.contains("exact Test262 paths are now\n  `4/4`"));
    assert!(TASK.contains("exact Test262 paths are now `4/4`"));
    for nonclaim in [
        "Async generators",
        "resource loop heads",
        "modules",
        "dynamic source",
        "suspension inside an initializer",
        "nonlinear async control flow",
    ] {
        assert!(README.contains(nonclaim));
        assert!(TASK.contains(nonclaim));
    }
    assert!(CONTRACT.contains("all 49 plain-async statement-list files"));
    assert!(CONTRACT.contains("complete `await using` directory"));
}
