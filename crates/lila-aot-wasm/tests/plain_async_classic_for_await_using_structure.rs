const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../lila-ir/src/analysis.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const ASYNC_LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering/async_disposable.rs");
const IR_TEST_SOURCE: &str = include_str!("../../lila-ir/src/lib.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_await_using_classic_for_lifecycle.js");
const CLI_TEST_SOURCE: &str = include_str!("../../lila-cli/tests/cli/resource_management.rs");
const TEST262_RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/plain-async-function-classic-for-await-using.md"
);
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

const EXACT_FILES: [(&str, &str); 4] = [
    (
        "language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-forstatement.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-forstatement.js"
        ),
    ),
    (
        "language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-forstatement.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-forstatement.js"
        ),
    ),
    (
        "language/statements/await-using/initializer-Symbol.asyncDispose-called-if-subsequent-initializer-throws-in-forstatement-head.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/await-using/initializer-Symbol.asyncDispose-called-if-subsequent-initializer-throws-in-forstatement-head.js"
        ),
    ),
    (
        "language/statements/await-using/initializer-Symbol.dispose-called-if-subsequent-initializer-throws-in-forstatement-head.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/await-using/initializer-Symbol.dispose-called-if-subsequent-initializer-throws-in-forstatement-head.js"
        ),
    ),
];

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
fn ir_owns_one_closed_nonempty_classic_for_async_dispose_capability() {
    let init_enum = bounded(
        IR_SOURCE,
        "pub enum ForInitIr {",
        "pub struct ForLexicalEnvironmentIr",
    );
    assert!(init_enum.contains("AsyncDisposable(AsyncDisposableForInitIr)"));
    assert!(!init_enum.contains("AsyncDisposable(AsyncDisposableResourcesIr)"));

    assert!(IR_SOURCE.contains(
        "#[must_use = \"an async-disposable classic-for initializer must be attached to its loop\"]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AsyncDisposableForInitIr"
    ));
    let init = bounded(
        IR_SOURCE,
        "pub struct AsyncDisposableForInitIr {",
        "impl AsyncDisposableResourcesIr",
    );
    for marker in [
        "capability: AsyncFunctionAsyncDisposableCapabilityIr",
        "resources: AsyncDisposableResourcesIr",
        "pub(crate) fn new(",
        "capability: AsyncFunctionAsyncDisposableCapabilityIr",
        "resources: AsyncDisposableResourcesIr",
        "pub fn capability(&self) -> &AsyncFunctionAsyncDisposableCapabilityIr",
        "pub fn resources(&self) -> &AsyncDisposableResourcesIr",
    ] {
        assert!(init.contains(marker), "{marker}");
    }
    assert!(!init.contains("pub capability"));
    assert!(!init.contains("pub resources"));
    assert!(!init.contains("Copy"));

    let resources = bounded(
        IR_SOURCE,
        "pub struct AsyncDisposableResourcesIr {",
        "pub struct AsyncDisposableForInitIr",
    );
    assert!(resources.contains("first: AsyncDisposableResourceIr"));
    assert!(resources.contains("rest: Vec<AsyncDisposableResourceIr>"));
    assert!(IR_SOURCE.contains("DoubleEndedIterator<Item = &AsyncDisposableResourceIr>"));
    assert!(IR_SOURCE.contains("pub fn is_empty(&self) -> bool {\n        false"));
}

#[test]
fn lowering_holds_an_unfinished_owner_until_test_update_and_body_are_lowered() {
    let owner = bounded(
        ANALYSIS_SOURCE,
        "pub(crate) enum AsyncDisposableScopeOwnerPlan {",
        "#[derive(Debug, Clone)]",
    );
    for variant in ["Ordinary", "Generator", "AsyncFunction", "AsyncGenerator"] {
        assert!(owner.contains(variant));
    }

    assert!(ASYNC_LOWERING_SOURCE.contains(
        "#[must_use = \"a pending async-disposable classic-for initializer must be finalized\"]\npub(super) struct PendingAsyncDisposableForInitIr"
    ));
    let pending = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) struct PendingAsyncDisposableForInitIr {",
        "/// The private pre-finalizer owner proof",
    );
    for marker in [
        "entry_state: u32",
        "binding_name: String",
        "resources: AsyncDisposableResourcesIr",
    ] {
        assert!(pending.contains(marker));
    }
    assert!(!pending.contains("Clone"));
    assert!(!pending.contains("Copy"));

    let lower = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) fn lower_async_disposable_for_init(",
        "pub(super) fn finish_async_disposable_for_init(",
    );
    positions_in_order(
        lower,
        &[
            "suspension inside an await using classic-for initializer",
            "match self.async_disposable_scope_owner()",
            "AsyncDisposableScopeOwnerPlan::AsyncFunction => {}",
            "AsyncDisposableScopeOwnerPlan::AsyncGenerator =>",
            "AsyncDisposableScopeOwnerPlan::Generator =>",
            "AsyncDisposableScopeOwnerPlan::Ordinary =>",
            "let entry_state = self",
            "alloc_suspension_owned_binding(",
            "async.function.for.await.dispose.capability.",
            "let init = self.lower_expression(initializer)",
            "InitializedBinding::without_creation",
            "into_async_disposable_resource(self)",
            "AsyncDisposableResourcesIr::new(first, resources.collect())",
        ],
    );

    let for_loop = bounded(
        LOWERING_SOURCE,
        "fn lower_for_loop(&mut self, for_loop: &ForLoop)",
        "fn plain_async_entry_state(&self)",
    );
    positions_in_order(
        for_loop,
        &[
            "pending_async_disposable_init = self.lower_async_disposable_for_init(list)",
            "let lexical_environment = self.lower_for_lexical_environment",
            "let test = for_loop.condition().map(|expr| self.lower_expression(expr))",
            "let update = for_loop",
            "let (body, body_kind) = self.lower_loop_body(for_loop.body())",
            "self.finish_async_disposable_for_init(pending)",
            "StatementIr::For {",
        ],
    );
    assert!(for_loop.contains("suspension inside an await using classic-for loop"));
    assert!(for_loop.contains("Some(ForInitIr::AsyncDisposable(_)) =>"));

    assert!(IR_TEST_SOURCE
        .contains("fn plain_async_classic_for_await_using_owns_closed_initializer_capability()"));
    let ir_test = bounded(
        IR_TEST_SOURCE,
        "fn plain_async_classic_for_await_using_owns_closed_initializer_capability()",
        "fn synchronous_using_for_of_is_a_closed_generic_iterator_head()",
    );
    for marker in [
        "StatementIr::Labelled { labels, statement }",
        "StatementIr::For {",
        "init: Some(ForInitIr::AsyncDisposable(init))",
        "lexical_environment: Some(environment)",
        "assert_eq!(init.resources().len(), 2)",
        "assert!(!init.resources().is_empty())",
        "assert!(environment.per_iteration_slots.is_empty())",
        "assert!(finalizer.entry_state() < finalizer.dispose_state())",
        "assert!(finalizer.dispose_state() < finalizer.resume_state())",
        "assert!(finalizer.resume_state() < finalizer.exit_state())",
    ] {
        assert!(ir_test.contains(marker), "{marker}");
    }
}

#[test]
fn backend_keeps_continue_inside_and_routes_every_terminal_edge_through_disposal() {
    assert!(CONTROL_FLOW_SOURCE.contains(
        "Self::labelled_async_disposable_for_finalizer(statement)\n                    .map(AsyncDisposableFinalizerPlanIr::entry_state)"
    ));
    assert!(CONTROL_FLOW_SOURCE.contains(
        "Self::labelled_async_disposable_for_finalizer(statement)\n                    .map(AsyncDisposableFinalizerPlanIr::exit_state)"
    ));
    let labelled = bounded(
        CONTROL_FLOW_SOURCE,
        "fn labelled_async_disposable_for_finalizer(",
        "fn generator_statement_entry_state(",
    );
    assert!(labelled.contains("StatementIr::Labelled { statement, .. } =>"));
    assert!(labelled.contains("Self::labelled_async_disposable_for_finalizer(statement)"));
    assert!(labelled.contains("init: Some(ForInitIr::AsyncDisposable(init))"));
    assert!(labelled.contains("_ => None"));
    let compile_for = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn compile_for(",
        "fn compile_classic_for_test(",
    );
    assert!(compile_for.contains("if let Some(ForInitIr::AsyncDisposable(init)) = init"));
    assert!(compile_for.contains("return self.compile_async_disposable_for("));

    let compile = bounded(
        CONTROL_FLOW_SOURCE,
        "fn compile_async_disposable_for(",
        "fn compile_sync_disposable_for(",
    );
    for marker in [
        "meta.protocol.execution_kind() == FunctionExecutionKind::Async",
        "await using for head cannot own per-iteration bindings",
        "debug_assert!(!resources.is_empty())",
        "ActivationAsyncDisposeOwner::AsyncFunction(init.capability())",
        "activation_owned_binding_storage(owner.binding_name())",
        "async DisposeCapability is missing its activation-owned binding",
    ] {
        assert!(compile.contains(marker), "{marker}");
    }
    let activation_storage = bounded(
        CONTROL_FLOW_SOURCE,
        "fn activation_owned_binding_storage(&self, name: &str)",
        "fn initialize_async_disposable_resource_bindings(",
    );
    positions_in_order(
        activation_storage,
        &[
            "self.owned_env_slot(name)",
            "BindingStorage::EnvSlot",
            "slot",
            "hops: self.environment_depth",
        ],
    );
    positions_in_order(
        compile,
        &[
            "let break_frame = self.open_frame(ControlFrameKind::Block, function)",
            "finalizer.entry_state()",
            "finalizer.exit_state()",
            "self.emit_enter_lexical_environment(environment, function)",
            "activation_owned_binding_storage(owner.binding_name())",
            "let disposal_frame = self.open_frame(ControlFrameKind::Block, function)",
            "self.finally_stack.push(disposal_frame)",
            "finalizer.entry_state()",
            "finalizer.dispose_state()",
            "initialize_async_disposable_resource_bindings(resources, function)",
            "initialize_activation_async_dispose_capability(&storage, resources, function)",
            "let loop_frame = self.open_frame(ControlFrameKind::Loop, function)",
            "compile_classic_for_test(test, disposal_frame, function)",
            "let continue_frame = self.open_frame(ControlFrameKind::Block, function)",
            "self.push_labels(labels, break_frame, Some(continue_frame))",
            "self.compile_statement(body, function)",
            "compile_classic_for_update(update, function)",
            "function.branch_to_label(loop_frame.label)",
            "self.finally_stack.pop()",
            "begin_async_dispose_pending_completion(function)",
            "self.set_completion_kind(CompletionKind::Normal, function)",
            "begin_activation_async_dispose_capability(",
            "consume_activation_async_dispose_capability(",
            "ActivationAsyncDisposeCompletionContinuation::ClassicFor",
        ],
    );
    assert!(!compile.contains("emit_leave_lexical_environment(function)"));
}

#[test]
fn backend_reconstructs_the_loop_environment_and_leaves_it_before_dispatch() {
    assert!(CONTROL_FLOW_SOURCE.contains(
        "#[must_use = \"an async DisposeCapability continuation must be consumed by its finalizer\"]\nenum ActivationAsyncDisposeCompletionContinuation"
    ));
    let continuation = bounded(
        CONTROL_FLOW_SOURCE,
        "enum ActivationAsyncDisposeCompletionContinuation {",
        "/// The two activation layouts",
    );
    assert!(continuation.contains("Scope"));
    assert!(continuation.contains("ClassicFor {"));
    assert!(continuation.contains("lexical_environment: ClassicForAsyncDisposeLexicalEnvironment"));
    assert!(continuation.contains("break_target: ControlTarget"));
    assert!(continuation.contains("enum ClassicForAsyncDisposeLexicalEnvironment"));
    assert!(continuation.contains("Absent"));
    assert!(continuation.contains("Active"));
    assert!(!continuation.contains("Clone"));
    assert!(!continuation.contains("Copy"));

    let consume = bounded(
        CONTROL_FLOW_SOURCE,
        "fn consume_activation_async_dispose_capability(",
        "fn finish_async_dispose_pending_completion(",
    );
    positions_in_order(
        consume,
        &[
            "for entry_kind in ActivationAsyncDisposeEntryKind::ALL",
            "emit_set_async_resume_state(activation_local, finalizer.resume_state()",
            "emit_activation_async_dispose_await_reactions",
            "finish_async_dispose_pending_completion(pending, function)",
            "emit_set_async_resume_state(activation_local, finalizer.exit_state()",
            "ActivationAsyncDisposeCompletionContinuation::ClassicFor",
            "ClassicForAsyncDisposeLexicalEnvironment::Absent => {}",
            "ClassicForAsyncDisposeLexicalEnvironment::Active =>",
            "self.emit_leave_lexical_environment(function)",
            "emit_dispatch_activation_async_dispose_completion(owner, function)",
            "self.emit_branch_to_target(break_target, function)",
        ],
    );
    assert_eq!(
        consume
            .matches("for entry_kind in ActivationAsyncDisposeEntryKind::ALL")
            .count(),
        1
    );
    assert!(!consume.contains("_ =>"));

    let compile = bounded(
        CONTROL_FLOW_SOURCE,
        "fn compile_async_disposable_for(",
        "fn compile_sync_disposable_for(",
    );
    assert!(compile.contains("Async body re-entry reconstructs lexical environments"));
    assert!(compile.contains("The original loop record remains reachable by"));
    assert!(compile.contains("closures created before disposal"));
    positions_in_order(
        compile,
        &[
            "if let Some(environment) = &runtime_environment",
            "self.emit_enter_lexical_environment(environment, function)",
            "activation_owned_binding_storage(owner.binding_name())",
        ],
    );
    assert!(!compile.contains("HEAP_ASYNC_ENV_OFFSET"));

    let planning = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn count_statement_temp_locals(statement: &StatementIr)",
        "pub(crate) fn count_for_init_temp_locals(init: &ForInitIr)",
    );
    positions_in_order(
        planning,
        &[
            "let test_temps",
            "let update_temps",
            "let body_temps",
            "Some(ForInitIr::AsyncDisposable(init))",
            "count_async_disposable_scope_temp_locals(",
            "init.resources()",
            "test_temps.max(update_temps).max(body_temps)",
        ],
    );
    assert!(DATA_SOURCE.contains("ForInitIr::AsyncDisposable(init) =>"));
    assert!(DATA_SOURCE.contains("for resource in init.resources().iter()"));
}

#[test]
fn exact_inventory_and_durable_fixture_bound_the_verified_claim() {
    for (path, source) in EXACT_FILES {
        assert!(source.contains("features: [explicit-resource-management]"));
        assert!(source.contains("flags: [async]"));
        assert!(!TEST262_RUNNER_SOURCE.contains(path));
        assert!(!KNOWN_FAILURES.contains(path));
        assert!(CONTRACT.contains(path));
        assert!(TASK.contains(path));
    }

    for marker in [
        "await using directBinding = direct.value",
        "fallbackBinding = fallback.value",
        "normal direct before disposal",
        "break:dispose",
        "label-break:dispose",
        "continue:body:1:0",
        "return:dispose",
        "throw:dispose",
        "test:abrupt",
        "update:abrupt",
        "acquire:later-tdz:ReferenceError",
        "acquire:first-dispose",
        "outer suppression error",
        "body suppression identity",
        "capture:after:outer:capture:1",
        "sync fallback return ignored",
        "await-using-classic-for:true",
    ] {
        assert!(FIXTURE.contains(marker), "{marker}");
    }
    let without_declarations = FIXTURE.replace("await using", "");
    assert!(!without_declarations.contains("await "));
    assert!(!without_declarations.contains("yield "));
    assert!(!FIXTURE.contains("for await ("));
    assert!(!FIXTURE.contains("eval("));
    assert!(CONTRACT.contains("repeated/nonlinear execution"));
    assert!(CLI_TEST_SOURCE.contains("fn wasm_await_using_classic_for_lifecycle()"));
    assert!(CLI_TEST_SOURCE.contains("wasm_await_using_classic_for_lifecycle.js"));

    for status in [README, TASK] {
        assert!(status.contains("plain-async classic"));
        assert!(status.contains("`await using`"));
        assert!(status.contains("`bca90f2ff9`"));
        assert!(status.contains("`0/8`"));
        assert!(status.contains("Runtime/NotImplemented"));
        assert!(status.contains("`8/8`"));
        assert!(status.contains("zero unsupported"));
        assert!(status.contains("or bug outcomes"));
        assert!(status.contains("Labelled"));
        assert!(status.contains("label chain ending directly"));
    }
    for exclusion in [
        "Async generators",
        "ordinary",
        "generator owners",
        "modules",
        "dynamic source",
        "binding patterns",
        "`for-of`",
        "source suspension",
        "complete `await using` directory",
        "outer labelled-block",
        "enclosing-loop",
        "resource-loop node",
        "full pinned aggregate",
    ] {
        assert!(
            CONTRACT.contains(exclusion) || README.contains(exclusion) || TASK.contains(exclusion),
            "{exclusion}"
        );
    }
}
