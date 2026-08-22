const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const ASYNC_LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering/async_disposable.rs");
const IR_TEST_SOURCE: &str = include_str!("../../lila-ir/src/lib.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_await_using_for_of_lifecycle.js");
const CLI_TEST_SOURCE: &str = include_str!("../../lila-cli/tests/cli/resource_management.rs");
const TEST262_RUNNER_SOURCE: &str = include_str!("../../lila-test262/src/lib.rs");
const KNOWN_FAILURES: &str = include_str!("../../lila-cli/tests/known-failures.tsv");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/plain-async-function-for-of-await-using.md");
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

const EXACT_FILES: [(&str, &str); 5] = [
    (
        "language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-each-iteration-of-forofstatement.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-each-iteration-of-forofstatement.js"
        ),
    ),
    (
        "language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-each-iteration-of-forofstatement.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-each-iteration-of-forofstatement.js"
        ),
    ),
    (
        "language/statements/for-of/head-await-using-bound-names-fordecl-tdz.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/for-of/head-await-using-bound-names-fordecl-tdz.js"
        ),
    ),
    (
        "language/statements/await-using/syntax/await-using-invalid-assignment-statement-body-for-of.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/await-using/syntax/await-using-invalid-assignment-statement-body-for-of.js"
        ),
    ),
    (
        "language/statements/await-using/syntax/await-using-valid-for-await-using-of-of.js",
        include_str!(
            "../../../test262/vendor/test262/test/language/statements/await-using/syntax/await-using-valid-for-await-using-of-of.js"
        ),
    ),
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

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker: {marker}"));
        cursor += offset + marker.len();
    }
}

#[test]
fn ir_head_makes_the_repeating_async_capability_and_iterator_record_indivisible() {
    assert!(IR_SOURCE.contains(
        "#[must_use = \"a plain-async for-of async DisposeCapability must be attached to its head\"]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AsyncFunctionAsyncDisposableForOfCapabilityIr"
    ));
    let capability = bounded(
        IR_SOURCE,
        "pub struct AsyncFunctionAsyncDisposableForOfCapabilityIr {",
        "/// One immutable async-disposable resource binding",
    );
    for marker in [
        "binding_name: String",
        "finalizer: AsyncDisposableFinalizerPlanIr",
        "pub(crate) fn new(",
        "pub fn binding_name(&self) -> &str",
        "pub fn finalizer(&self) -> &AsyncDisposableFinalizerPlanIr",
    ] {
        assert!(capability.contains(marker), "{marker}");
    }
    assert!(!capability.contains("pub binding_name"));
    assert!(!capability.contains("pub finalizer"));
    assert!(!capability.contains("Copy"));

    assert!(IR_SOURCE.contains(
        "#[must_use = \"an async-disposable for-of head must be attached to its iterator loop\"]\n#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AsyncDisposableForOfHeadIr"
    ));
    let head = bounded(
        IR_SOURCE,
        "pub struct AsyncDisposableForOfHeadIr {",
        "/// The exhaustive head domain of the generic iterator protocol path.",
    );
    for marker in [
        "binding_name: String",
        "capability: AsyncFunctionAsyncDisposableForOfCapabilityIr",
        "record: IteratorRecordIr",
        "pub(crate) fn new(",
        "pub fn binding_name(&self) -> &str",
        "pub fn capability(&self) -> &AsyncFunctionAsyncDisposableForOfCapabilityIr",
        "pub fn record(&self) -> &IteratorRecordIr",
    ] {
        assert!(head.contains(marker), "{marker}");
    }
    assert!(!head.contains("pub binding_name"));
    assert!(!head.contains("pub capability"));
    assert!(!head.contains("pub record"));
    assert!(!head.contains("Copy"));

    let heads = bounded(
        IR_SOURCE,
        "pub enum ForOfIteratorHeadIr {",
        "/// The runtime Environment Record lifecycle owned by a resumable loop.",
    );
    assert!(heads.contains("AsyncDisposable(AsyncDisposableForOfHeadIr)"));
    assert!(heads.contains("SyncDisposable(SyncDisposableForOfHeadIr)"));
    let async_head = bounded(heads, "AsyncDisposable(AsyncDisposableForOfHeadIr)", "}");
    assert!(!async_head.contains("async_plan"));
    assert!(!async_head.contains("protocol"));

    let statements = bounded(IR_SOURCE, "    ForOfArray {", "    ForInArray {");
    assert_eq!(statements.matches("head: ForOfAssignmentIr").count(), 2);
    let iterator = bounded(statements, "    ForOfIterator {", "    },");
    assert!(iterator.contains("head: ForOfIteratorHeadIr"));
    assert!(!iterator.contains("async_plan:"));
    assert!(!iterator.contains("protocol:"));
}

#[test]
fn lowering_holds_the_iterator_roles_until_the_body_has_allocated_source_states() {
    assert!(ASYNC_LOWERING_SOURCE.contains(
        "#[must_use = \"a pending async-disposable for-of head must be finalized after its body\"]\npub(super) struct PendingAsyncDisposableForOfHeadIr"
    ));
    let pending = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) struct PendingAsyncDisposableForOfHeadIr {",
        "/// The private pre-finalizer owner proof",
    );
    for marker in [
        "entry_state: u32",
        "binding_name: String",
        "capability_binding_name: String",
        "record: IteratorRecordIr",
    ] {
        assert!(pending.contains(marker), "{marker}");
    }
    assert!(!pending.contains("Clone"));
    assert!(!pending.contains("Copy"));

    let admit = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) fn admit_async_disposable_for_of_head(",
        "pub(super) fn async_disposable_for_head(",
    );
    positions_in_order(
        admit,
        &[
            "if for_of.r#await()",
            "await using declaration in for-await-of",
            "contains(for_of.iterable(), ContainsSymbol::AwaitExpression)",
            "contains(for_of.iterable(), ContainsSymbol::YieldExpression)",
            "contains(for_of.body(), ContainsSymbol::AwaitExpression)",
            "contains(for_of.body(), ContainsSymbol::YieldExpression)",
            "source suspension in await using for-of loop",
            "let Binding::Identifier(identifier) = binding",
            "await using declaration binding pattern in for-of",
        ],
    );

    let begin = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) fn begin_async_disposable_for_of_head(",
        "/// Completes the repeating per-iteration finalizer",
    );
    positions_in_order(
        begin,
        &[
            "match self.async_disposable_scope_owner()",
            "AsyncDisposableScopeOwnerPlan::AsyncFunction => {}",
            "AsyncDisposableScopeOwnerPlan::AsyncGenerator =>",
            "AsyncDisposableScopeOwnerPlan::Generator =>",
            "AsyncDisposableScopeOwnerPlan::Ordinary =>",
            "let entry_state = self",
            "alloc_suspension_owned_binding(",
            "async.function.forof.await.dispose.capability.",
            "let record = IteratorRecordIr::new(",
            "self.alloc_iterator_slot()",
            "self.alloc_next_method_slot()",
            "self.alloc_done_slot()",
            "Some(PendingAsyncDisposableForOfHeadIr",
        ],
    );

    let finish = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) fn finish_async_disposable_for_of_head(",
        "/// Lowers one admitted async-owner `await using` declaration",
    );
    positions_in_order(
        finish,
        &[
            "let finalizer = self.allocate_async_disposable_finalizer(pending.entry_state)",
            "AsyncDisposableForOfHeadIr::new(",
            "pending.binding_name",
            "AsyncFunctionAsyncDisposableForOfCapabilityIr::new(",
            "pending.capability_binding_name",
            "finalizer",
            "pending.record",
        ],
    );

    let exhaustive_begin = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) fn begin_async_disposable_for_of_if_needed(",
        "/// Completes the repeating per-iteration finalizer",
    );
    assert!(exhaustive_begin
        .contains("LoweredForOfHeadKind::Assignment | LoweredForOfHeadKind::SyncDisposable"));
    assert!(exhaustive_begin.contains("LoweredForOfHeadKind::AsyncDisposable => self"));
    assert!(exhaustive_begin.contains("begin_async_disposable_for_of_head"));
    assert!(!exhaustive_begin.contains("_ =>"));

    let statement = bounded(
        ASYNC_LOWERING_SOURCE,
        "pub(super) fn async_disposable_for_of_statement(",
        "/// Lowers one admitted async-owner `await using` declaration",
    );
    positions_in_order(
        statement,
        &[
            "IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL",
            "StatementIr::ForOfIterator",
            "ForOfIteratorHeadIr::AsyncDisposable(head)",
        ],
    );
    assert!(!statement.contains("ASYNC_ITERATOR_PROTOCOL"));

    let lower = bounded(
        LOWERING_SOURCE,
        "fn lower_for_of_head(&mut self, for_of: &ForOfLoop) -> ForOfLoweringIr {",
        "fn lower_switch(&mut self, switch: &AstSwitch)",
    );
    positions_in_order(
        lower,
        &[
            "IterableLoopInitializer::AwaitUsing(binding)",
            "self.admit_async_disposable_for_of_head(for_of, binding)",
            "LoweredForOfHeadKind::AsyncDisposable",
            "self.lower_for_head_expression_with_tdz(mode, &name, for_of.iterable())",
            "self.begin_async_disposable_for_of_if_needed(head_kind, &storage_name)",
            "self.declare_binding(",
            "let (mut body, body_kind) = self.lower_loop_body(for_of.body())",
            "self.finish_async_disposable_for_of_head(pending)",
            "LoweredForOfHeadKind::AsyncDisposable => Self::async_disposable_for_of_statement(",
        ],
    );
    assert_eq!(
        lower
            .matches("&& head_kind == LoweredForOfHeadKind::Assignment")
            .count(),
        2,
        "Array and String specializations must reject resource heads"
    );

    assert!(IR_TEST_SOURCE
        .contains("fn plain_async_for_of_await_using_owns_repeating_iteration_capability()"));
    let ir_test = bounded(
        IR_TEST_SOURCE,
        "fn plain_async_for_of_await_using_owns_repeating_iteration_capability()",
        "fn synchronous_using_for_of_is_a_closed_generic_iterator_head()",
    );
    for marker in [
        "head: ForOfIteratorHeadIr::AsyncDisposable(head)",
        "lexical_environment: Some(environment)",
        "tdz_binding_names",
        "ExprIr::RuntimeThrow",
        "name: NativeErrorKind::ReferenceError",
        "iteration_environment",
        "capture.mode, BindingMode::Const",
        "name: NativeErrorKind::TypeError",
        "$async.function.forof.await.dispose.capability.",
        "record.iterator().as_str()",
        "record.next_method().as_str()",
        "record.done().as_str()",
        "let owned_names = [",
        "owned_names.iter().copied().collect::<BTreeSet<_>>().len(),",
        "every activation-backed role must own exactly one slot",
        "finalizer.entry_state() < finalizer.dispose_state()",
        "finalizer.dispose_state() < finalizer.resume_state()",
        "finalizer.resume_state() < finalizer.exit_state()",
        "source suspension in await using for-of loop",
    ] {
        assert!(ir_test.contains(marker), "{marker}");
    }
}

#[test]
fn exact_inventory_is_raw_and_the_consumer_oracle_pins_the_lifecycle_boundary() {
    for (path, source) in EXACT_FILES {
        assert!(source.contains("features: [explicit-resource-management]"));
        assert!(source.contains("await using"));
        assert!(!TEST262_RUNNER_SOURCE.contains(path), "rewritten: {path}");
        assert!(!KNOWN_FAILURES.contains(path), "known failure: {path}");
        assert!(CONTRACT.contains(path), "contract inventory: {path}");
        assert!(TASK.contains(path), "task inventory: {path}");
    }

    for marker in [
        "for (await using shadowed of [shadowed])",
        "for (await using of of [])",
        "acquire-tdz:ReferenceError",
        "binding initialized after acquisition",
        "genericIterable(",
        "normal disposal before next",
        "first fresh captured binding",
        "sync fallback return ignored",
        "continue disposes before next without close",
        "break disposal before close",
        "return disposal before close",
        "throw disposal before close",
        "current = null",
        "immutable assignment disposal before close",
        "later acquisition failure closes after prior disposal",
        "outer head binding after nested finalizer",
        "outer head binding survives nested finalizer",
        "nested LIFO suppression before close",
        "body suppression identity",
        "await-using-for-of:true",
    ] {
        assert!(FIXTURE.contains(marker), "{marker}");
    }
    assert!(!FIXTURE.contains("for await ("));
    assert!(!FIXTURE.contains("eval("));
    assert!(CLI_TEST_SOURCE.contains("fn wasm_await_using_for_of_lifecycle()"));
    assert!(CLI_TEST_SOURCE.contains("wasm_await_using_for_of_lifecycle.js"));

    let readme_status = bounded(
        README,
        "- The plain-async resource-loop batch now supports synchronous",
        "- The adjacent classic-`for` extension",
    );
    let task_status = bounded(
        TASK,
        "The adjacent batch gives a plain async function's synchronous `for-of`",
        "The next bounded source batch extends that same synchronous disposal lifecycle",
    );
    for status in [readme_status, task_status] {
        assert!(status.contains("`009219b28`"));
        assert!(status.contains("`0/10`"));
        assert!(status.contains("Runtime/NotImplemented"));
        assert!(status.contains("await using"));
        assert!(status.contains("declaration in for-of"));
        assert!(status.contains("cargo check --workspace"));
        assert!(status.contains("--all-targets"));
        assert!(status.contains("cargo xc"));
        assert!(status.contains("focused IR test"));
        assert!(status.contains("12.17s"));
        assert!(status.contains("bounded structure executable"));
        assert!(status.contains("`5/5`"));
        assert!(status.contains("`0.23s`"));
        assert!(status.contains("`14.25s`"));
        assert!(status.contains("`4/4`"));
        assert!(status.contains("37.83s"));
        assert!(status.contains("48.82s"));
        assert!(status.contains("`10/10`"));
        assert!(status.contains("zero unsupported"));
        assert!(status.contains("crash or bug outcomes"));
        assert!(status.contains("focused evidence only"));
        assert!(!status.contains("dry-written"));
        assert!(!status.contains("pending central"));
    }
    for exclusion in [
        "Module-only fresh-binding",
        "`for-await-of`",
        "async generators",
        "binding patterns",
        "dynamic source",
        "complete `await using` directory",
        "full pinned aggregate",
    ] {
        assert!(
            CONTRACT.contains(exclusion) || README.contains(exclusion) || TASK.contains(exclusion),
            "{exclusion}"
        );
    }
}

#[test]
fn backend_typestate_and_exhaustive_consumers_make_the_async_head_compile_visible() {
    let continuation = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"an async DisposeCapability continuation must be consumed by its finalizer\"]",
        "/// The two activation layouts that can own an asynchronous DisposeCapability.",
    );
    for marker in [
        "enum ActivationAsyncDisposeCompletionContinuation",
        "Scope",
        "ClassicFor",
        "ForOf(AsyncDisposableForOfCompletionContinuationLocals)",
        "enum AsyncDisposableForOfIterationEnvironment",
        "Absent",
        "Active",
        "struct AsyncDisposableForOfCompletionContinuationLocals",
        "iteration_environment: AsyncDisposableForOfIterationEnvironment",
        "continue_target: ControlTarget",
        "loop_target: ControlTarget",
    ] {
        assert!(continuation.contains(marker), "{marker}");
    }
    assert!(!continuation.contains("derive(Clone"));
    assert!(!continuation.contains("derive(Copy"));

    let owner = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"an async DisposeCapability owner must reach its consuming finalizer\"]",
        "/// The resumable execution owners that share the activation-backed",
    );
    assert!(owner.contains("AsyncFunctionForOf(&'a AsyncFunctionAsyncDisposableForOfCapabilityIr)"));
    for helper in [
        "fn binding_name(&self)",
        "fn finalizer(&self)",
        "const fn execution_kind(&self)",
        "const fn resume_state_offset(&self)",
        "const fn resume_payload_offset(&self)",
        "const fn resume_tag_offset(&self)",
    ] {
        let helper = bounded(owner, helper, "    }");
        assert!(helper.contains("Self::AsyncFunctionForOf"), "{helper}");
        assert!(!helper.contains("_ =>"));
    }
    assert!(!owner.contains("derive(Clone"));
    assert!(!owner.contains("derive(Copy"));

    let entry = bounded(
        CONTROL_FLOW_SOURCE,
        "fn async_statement_entry_state(statement: &StatementIr)",
        "fn async_statement_exit_state(statement: &StatementIr)",
    );
    assert!(entry.contains("head: ForOfIteratorHeadIr::AsyncDisposable(head)"));
    assert!(entry.contains("head.capability().finalizer().entry_state()"));
    let exit = bounded(
        CONTROL_FLOW_SOURCE,
        "fn async_statement_exit_state(statement: &StatementIr)",
        "fn labelled_async_disposable_for_finalizer(",
    );
    assert!(exit.contains("head: ForOfIteratorHeadIr::AsyncDisposable(head)"));
    assert!(exit.contains("head.capability().finalizer().exit_state()"));
    let labelled = bounded(
        CONTROL_FLOW_SOURCE,
        "fn labelled_async_disposable_for_finalizer(",
        "fn generator_statement_entry_state(",
    );
    assert!(labelled.contains("StatementIr::Labelled { statement, .. }"));
    assert!(labelled.contains("Self::labelled_async_disposable_for_finalizer(statement)"));
    assert!(labelled.contains("head: ForOfIteratorHeadIr::AsyncDisposable(head)"));
    assert!(labelled.contains("Some(head.capability().finalizer())"));
    assert!(labelled.contains("_ => None"));

    for (start, end) in [
        (
            "pub(crate) fn compile_statement(",
            "fn compile_return_position_expr(",
        ),
        (
            "pub(crate) fn compile_labelled_statement(",
            "pub(crate) fn compile_try_catch(",
        ),
    ] {
        let dispatch = bounded(CONTROL_FLOW_SOURCE, start, end);
        assert!(dispatch.contains("ForOfIteratorHeadIr::AsyncDisposable(head)"));
        assert!(dispatch.contains("self.compile_async_disposable_for_of_iterator("));
    }

    let planning_lexicals = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn count_statement_lexicals(statement: &StatementIr)",
        "pub(crate) fn count_statement_temp_locals(statement: &StatementIr)",
    );
    assert!(planning_lexicals.contains("ForOfIteratorHeadIr::AsyncDisposable(head)"));
    assert!(planning_lexicals.contains("(BindingMode::Const, head.binding_name())"));
    let planning_temps = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn count_statement_temp_locals(statement: &StatementIr)",
        "const SYNC_DISPOSABLE_SCOPE_COMPLETION_TEMP_LOCALS",
    );
    assert!(planning_temps.contains("ForOfIteratorHeadIr::AsyncDisposable(_)"));
    assert!(planning_temps.contains("ASYNC_DISPOSABLE_FOR_OF_PERSISTENT_TEMP_LOCALS"));
    assert!(planning_temps.contains("ACTIVATION_ASYNC_DISPOSE_WALKER_TEMP_LOCALS"));
    assert!(planning_temps.contains("ACTIVATION_ASYNC_DISPOSE_HELPER_TEMP_LOCALS"));
    assert!(planning_temps.contains("ASYNC_DISPOSABLE_FOR_OF_BINDING_RESTORE_TEMP_LOCALS"));
    assert!(PLANNING_SOURCE.contains(
        "const ASYNC_DISPOSABLE_FOR_OF_PERSISTENT_TEMP_LOCALS: usize = 1 + 7 * 2 + 1 + 2 * 4 + 5;"
    ));
    assert!(PLANNING_SOURCE
        .contains("const ASYNC_DISPOSABLE_FOR_OF_BINDING_RESTORE_TEMP_LOCALS: usize = 6;"));

    let data = bounded(
        DATA_SOURCE,
        "StatementIr::ForOfIterator {",
        "StatementIr::Switch {",
    );
    assert!(data.contains("ForOfIteratorHeadIr::AsyncDisposable(head)"));
    assert!(data.contains("\"Symbol.asyncDispose\""));
    assert!(data.contains("\"Symbol.dispose\""));
    assert!(data.contains("self.intern_string(head.binding_name())"));
    assert_eq!(
        EMIT_SOURCE
            .matches("head: ForOfIteratorHeadIr::AsyncDisposable(_)")
            .count(),
        2,
        "async-generator suspension and admission walkers must classify the head explicitly"
    );
    assert!(EMIT_SOURCE.contains("await using for-of requires a plain async function"));
    assert!(
        CONTRACT.contains("All closed head and owner domains are handled with exhaustive matches")
    );
}

#[test]
fn backend_disposes_and_awaits_before_choosing_next_or_iterator_close() {
    let compile = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn compile_async_disposable_for_of_iterator(",
        "pub(crate) fn compile_for_of_iterator(",
    );
    for marker in [
        "await using for-of head requires a plain async function",
        "ActivationAsyncDisposeOwner::AsyncFunctionForOf(head.capability())",
        "await using for-of Iterator is missing its activation-owned binding",
        "await using for-of NextMethod is missing its activation-owned binding",
        "await using for-of Done is missing its activation-owned binding",
        "await using for-of DisposeCapability is missing its activation-owned binding",
        "AsyncDisposableForOfIterationEnvironment::Active",
        "AsyncDisposableForOfIterationEnvironment::Absent",
    ] {
        assert!(compile.contains(marker), "{marker}");
    }
    positions_in_order(
        compile,
        &[
            "self.emit_async_state_in_range(",
            "self.emit_enter_for_in_of_tdz_scope(",
            "self.compile_expr_to_locals(",
            "self.emit_leave_for_in_of_tdz_scope(",
            "self.strings.property_key_symbol_payload(\"Symbol.iterator\")",
            "self.write_binding_from_locals(\n            iterator_storage",
            "self.write_binding_from_locals(next_storage",
            "self.write_binding_from_locals(done_storage",
            "let break_frame = self.open_frame(ControlFrameKind::Block",
            "let loop_frame = self.open_frame(ControlFrameKind::Loop",
            "self.emit_function_or_proxy_call_leave_throw_completion(\n            next_payload_local",
            "self.strings.payload(\"done\")",
            "self.strings.payload(\"value\")",
            "self.emit_enter_lexical_environment(environment, function)",
            "activation_owned_binding_storage(owner.binding_name())",
            "A nested implicit await-using finalizer resumes source execution",
            "finalizer.entry_state()",
            "Instruction::I64GtU",
            "finalizer.dispose_state()",
            "Instruction::I64LtU",
            "Instruction::I32And",
            "self.restore_async_disposable_for_of_binding(&capability_storage, storage, function)",
            "self.read_binding_to_locals(\n            iteration_iterator_storage",
            "let continue_frame = self.open_frame(ControlFrameKind::Block",
            "let disposal_frame = self.open_frame(ControlFrameKind::Block",
            "self.initialize_binding_uninitialized(storage, function)",
            "self.initialize_empty_activation_async_dispose_capability(",
            "self.reset_async_disposable_resource_locals(&acquired, function)",
            "self.acquire_async_disposable_resource_from_locals(&acquired, function)",
            "self.append_activation_async_disposable_resource(&capability, &acquired, function)",
            "self.write_binding_from_locals(\n            storage",
            "self.release_active_activation_async_dispose_capability(capability)",
            "Resume those states through the body without reacquiring",
            "self.emit_async_state_in_range(",
            "finalizer.entry_state()",
            "finalizer.dispose_state()",
            "self.push_labels(labels, break_frame, Some(continue_frame))",
            "self.compile_statement(body, function)",
            "Do not expose the now",
            "self.loop_stack.pop()",
            "self.begin_async_dispose_pending_completion(function)",
            "self.begin_activation_async_dispose_capability(",
            "self.consume_activation_async_dispose_capability(",
            "ActivationAsyncDisposeCompletionContinuation::ForOf(",
            "AsyncDisposableForOfCompletionContinuationLocals",
        ],
    );

    let restore = bounded(
        CONTROL_FLOW_SOURCE,
        "fn restore_async_disposable_for_of_binding(",
        "fn reserve_async_disposable_resource_locals(",
    );
    positions_in_order(
        restore,
        &[
            "self.read_binding_to_locals(\n            capability_storage.binding",
            "HEAP_OBJECT_BOXED_PAYLOAD_OFFSET",
            "HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET",
            "HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET",
            "HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET",
            "self.write_binding_from_locals(\n            binding_storage",
            "self.release_temp_local(value_tag_local)",
            "self.release_temp_local(value_payload_local)",
            "self.release_temp_local(entry_local)",
            "self.release_temp_local(record_local)",
            "self.release_temp_local(object_tag_local)",
            "self.release_temp_local(object_local)",
        ],
    );
    assert_eq!(restore.matches("self.reserve_temp_local()").count(), 6);

    let finish = bounded(
        CONTROL_FLOW_SOURCE,
        "fn finish_async_disposable_for_of_iteration(",
        "fn finish_async_dispose_pending_completion(",
    );
    positions_in_order(
        finish,
        &[
            "match continuation.iteration_environment",
            "AsyncDisposableForOfIterationEnvironment::Absent => {}",
            "AsyncDisposableForOfIterationEnvironment::Active =>",
            "self.emit_leave_lexical_environment(function)",
            "COMPLETION_KIND_CONTINUE",
            "continuation.continue_target.frame",
            "self.set_completion_kind(CompletionKind::Normal, function)",
            "COMPLETION_KIND_NORMAL",
            "self.emit_set_async_resume_state(activation_local, finalizer.entry_state()",
            "LocalSet(continuation.state_local)",
            "self.emit_branch_to_target(continuation.loop_target, function)",
            "self.emit_set_async_resume_state(activation_local, finalizer.exit_state()",
            "self.save_current_completion(",
            "COMPLETION_KIND_THROW",
            "self.emit_iterator_close_preserving_current_throw(",
            "Instruction::Else",
            "self.emit_iterator_close(",
            "self.emit_dispatch_activation_async_dispose_completion(owner, function)",
        ],
    );
    assert!(!finish.contains("_ =>"));
}
