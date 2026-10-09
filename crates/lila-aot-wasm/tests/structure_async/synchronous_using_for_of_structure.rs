const IR_SOURCE: &str = include_str!("../../../lila-ir/src/ir.rs");
const FOR_OF_LOWERING_SOURCE: &str = include_str!("../../../lila-ir/src/lowering/for_of.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../../src/control_flow.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../../src/expressions.rs");
const FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_using_for_of_lifecycle.js");
const CONTRACT: &str =
    include_str!("../../../../docs/rust-rewrite/contracts/synchronous-using-for-of.md");
const VENDORED_WITNESSES: [&str; 3] = [
    include_str!(
        "../../../../test262/vendor/test262/test/language/statements/for-of/head-using-bound-names-fordecl-tdz.js"
    ),
    include_str!(
        "../../../../test262/vendor/test262/test/language/statements/for-of/head-using-fresh-binding-per-iteration.js"
    ),
    include_str!(
        "../../../../test262/vendor/test262/test/language/statements/using/syntax/using-invalid-assignment-statement-body-for-of.js"
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

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later, "{earlier} must precede {later}");
}

#[test]
fn closed_head_forces_resources_onto_the_generic_synchronous_protocol() {
    let heads = bounded(
        IR_SOURCE,
        "pub struct ForOfAssignmentIr {",
        "/// The runtime Environment Record lifecycle owned by a resumable loop.",
    );
    assert!(heads.contains("pub mode: BindingMode"));
    assert!(heads.contains("pub name: String"));
    assert!(heads.contains("pub struct SyncDisposableForOfHeadIr {\n    binding_name: String,"));
    assert!(heads.contains("pub(crate) fn new(binding_name: String) -> Self"));
    assert!(heads.contains("pub fn binding_name(&self) -> &str"));
    assert!(heads.contains("pub enum ForOfIteratorHeadIr {"));
    assert!(heads.contains("Assignment {"));
    assert!(heads.contains("binding: ForOfAssignmentIr"));
    assert!(heads.contains("async_plan: Option<AsyncForOfIteratorPlanIr>"));
    assert!(heads.contains("protocol: IteratorProtocolWitness"));
    assert!(heads.contains("SyncDisposable(SyncDisposableForOfHeadIr)"));

    assert!(!IR_SOURCE.contains("    ForOfArray {"));
    assert!(!IR_SOURCE.contains("    ForOfString {"));
    let iterator = bounded(IR_SOURCE, "    ForOfIterator {", "    ForInArray {");
    assert!(iterator.contains("head: ForOfIteratorHeadIr"));
    assert!(!iterator.contains("async_plan:"));
    assert!(!iterator.contains("protocol:"));
}

#[test]
fn lowering_keeps_tdz_and_protocol_decisions_at_the_closed_head_boundary() {
    let lowering = bounded(
        FOR_OF_LOWERING_SOURCE,
        "    fn lower_for_of_head(&mut self, for_of: &ForOfLoop) -> ForOfLoweringIr {",
        "        ForOfLoweringIr::new(statement, body_kind, protocol)",
    );
    for boundary in [
        "IterableLoopInitializer::Using(Binding::Identifier(identifier))",
        "LoweredForOfHeadKind::SyncDisposable",
        "BindingMode::Const",
        "self.lower_for_head_expression_with_tdz(mode, &name, for_of.iterable())",
        "match head_kind",
        "ForOfIteratorHeadIr::SyncDisposable(",
        "IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL",
    ] {
        assert!(
            lowering.contains(boundary),
            "missing lowering boundary: {boundary}"
        );
    }
    let compact_lowering = lowering
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    assert!(compact_lowering.contains("SyncDisposableForOfHeadIr::new(storage_name,)"));
    assert_eq!(
        lowering
            .matches("&& head_kind == LoweredForOfHeadKind::Assignment")
            .count(),
        0,
        "ordinary for-of heads must not select an iterator-protocol bypass"
    );
    assert_before(
        lowering,
        "self.lower_for_head_expression_with_tdz(mode, &name, for_of.iterable())",
        "ForOfIteratorHeadIr::SyncDisposable(",
    );
}

#[test]
fn comma_operands_route_abrupt_completion_before_the_right_operand() {
    assert_eq!(
        EXPRESSIONS_SOURCE
            .matches("ExprIr::Comma { lhs, rhs } => {")
            .count(),
        1
    );
    let arm = bounded(
        EXPRESSIONS_SOURCE,
        "ExprIr::Comma { lhs, rhs } => {",
        "ExprIr::MaterializeBinding",
    );
    assert_before(
        arm,
        "self.compile_expr_to_value(lhs, &value, function)?;",
        "value.clear(function);",
    );
    assert_before(
        arm,
        "value.clear(function);",
        "self.compile_expr_to_value(rhs, output, function)?;",
    );
    // Every recursive expression evaluation now routes Throw before returning;
    // the sole comma arm inherits that boundary before evaluating its RHS.
    let publisher = bounded(
        EXPRESSIONS_SOURCE,
        "pub(crate) fn compile_expr_to_value(",
        "fn compile_identifier_to_value(",
    );
    assert!(publisher.trim_end().ends_with(
        "self.emit_propagate_current_throw_if_needed(function);\n        Ok(())\n    }"
    ));
}

#[test]
fn backend_consumes_each_closed_head_and_disposes_before_loop_continue_or_close() {
    let head_witness = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"a synchronous iterator head must consume its iteration lifecycle\"]",
        "#[must_use = \"a synchronous for-of iteration must finish assignment or disposal\"]",
    );
    assert!(head_witness.contains("pub(crate) enum SyncForOfIteratorHead<'a>"));
    assert!(head_witness.contains("Assignment(&'a ForOfAssignmentIr)"));
    assert!(head_witness.contains("head: &'a SyncDisposableForOfHeadIr"));
    assert!(head_witness.contains("body: SynchronousLoopBodyIr<'a>"));
    assert!(!head_witness.contains("derive(Clone"));
    assert!(!head_witness.contains("derive(Copy"));

    let lifecycle_witness = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"a synchronous for-of iteration must finish assignment or disposal\"]",
        "#[must_use = \"a sync disposal continuation must be consumed after completion restoration\"]",
    );
    assert!(lifecycle_witness.contains("enum SyncForOfIterationLifecycleLocals<'a>"));
    assert!(lifecycle_witness.contains("acquired: AcquiredSyncDisposableResourceLocals"));
    assert!(!lifecycle_witness.contains("derive(Clone"));
    assert!(!lifecycle_witness.contains("derive(Copy"));

    let continuation = bounded(
        CONTROL_FLOW_SOURCE,
        "enum SyncDisposeCompletionContinuation {",
        "fn innermost_target(",
    );
    assert!(continuation.contains("Dispatch"));
    assert!(continuation.contains("DeferToIteratorClose"));

    let consumer = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn consume_sync_disposable_resources(",
        "    pub(crate) fn compile_try_catch_finally(",
    );
    assert!(consumer.contains("self.completion().copy_from(&pending.completion, function)"));
    assert!(consumer.contains("match continuation"));
    assert!(consumer.contains(
        "SyncDisposeCompletionContinuation::Dispatch => {\n                self.emit_dispatch_current_completion(function)?"
    ));
    assert!(consumer.contains("SyncDisposeCompletionContinuation::DeferToIteratorClose => {}"));
    assert_before(
        consumer,
        "self.completion().copy_from(&pending.completion, function)",
        "match continuation",
    );

    for (start, end) in [
        (
            "    pub(crate) fn compile_statement(",
            "    pub(crate) fn compile_labelled_statement(",
        ),
        (
            "    pub(crate) fn compile_labelled_statement(",
            "    pub(crate) fn compile_try_catch(",
        ),
    ] {
        let dispatch = bounded(CONTROL_FLOW_SOURCE, start, end);
        assert!(dispatch.contains("match head"));
        assert!(dispatch.contains("ForOfIteratorHeadIr::Assignment {"));
        assert!(dispatch.contains("ForOfIteratorHeadIr::SyncDisposable(head)"));
        assert!(dispatch.contains("SyncForOfIteratorHead::Assignment(binding)"));
        assert!(dispatch.contains("SyncForOfIteratorHead::SyncDisposable {"));
        assert!(dispatch.contains("SynchronousLoopBodyIr::new(body)"));
        assert!(dispatch.contains("self.compile_for_of_iterator("));
    }

    let lifecycle = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_for_of_iterator(",
        "    pub(crate) fn emit_copy_data_properties_into(",
    );
    for boundary in [
        "let lifecycle = match head",
        "SyncForOfIterationLifecycleLocals::Assignment(binding)",
        "SyncForOfIterationLifecycleLocals::SyncDisposable {",
        "self.reserve_sync_disposable_resource_locals(function)",
        "self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?",
        "self.compile_expr_to_value(iterable, &source, function)?",
        "self.emit_leave_for_in_of_tdz_scope(environment, function)",
        "self.emit_enter_lexical_environment(environment, function)?",
        "let continue_frame = self.open_frame(ControlFrameKind::Block, function)",
        "self.loop_stack.push(LoopTargets { continue_frame })",
        "self.finally_stack.push(finalizer)",
        "self.initialize_binding_uninitialized(binding, function)",
        "self.reset_sync_disposable_resource_locals(acquired, function)",
        "self.finally_stack.push(disposal)",
        "self.compile_sync_disposable_resource_from_locals(binding, acquired, function)?",
        "self.push_labels(labels, break_frame, Some(continue_frame))",
        "self.compile_statement(body, function)?",
        "self.capture_pending_sync_dispose_completion(function)",
        "self.consume_sync_disposable_resources(",
        "SyncDisposeCompletionContinuation::DeferToIteratorClose",
        "pending.copy_from(self.completion(), function)",
        "self.emit_leave_lexical_environment(function)",
        "CompletionKind::Continue.code()",
        "pending.target().load(function)",
        "pending.set_normal(pending.value(), function)",
        "CompletionKind::Normal.code()",
        "self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?",
        "self.emit_dispatch_current_completion(function)?",
        "self.emit_branch_to_target(loop_frame, function)",
    ] {
        assert!(
            lifecycle.contains(boundary),
            "missing backend boundary: {boundary}"
        );
    }
    for (earlier, later) in [
        (
            "self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?",
            "self.compile_expr_to_value(iterable, &source, function)?",
        ),
        (
            "self.compile_expr_to_value(iterable, &source, function)?",
            "self.emit_leave_for_in_of_tdz_scope(environment, function)",
        ),
        (
            "self.emit_enter_lexical_environment(environment, function)?",
            "self.initialize_binding_uninitialized(binding, function)",
        ),
        (
            "self.finally_stack.push(finalizer)",
            "self.initialize_binding_uninitialized(binding, function)",
        ),
        (
            "self.finally_stack.push(disposal)",
            "self.compile_sync_disposable_resource_from_locals(binding, acquired, function)?",
        ),
        (
            "self.compile_sync_disposable_resource_from_locals(binding, acquired, function)?",
            "self.compile_statement(body, function)?",
        ),
        (
            "self.compile_statement(body, function)?",
            "self.capture_pending_sync_dispose_completion(function)",
        ),
        (
            "self.capture_pending_sync_dispose_completion(function)",
            "self.consume_sync_disposable_resources(",
        ),
        (
            "SyncDisposeCompletionContinuation::DeferToIteratorClose",
            "pending.copy_from(self.completion(), function)",
        ),
        (
            "pending.copy_from(self.completion(), function)",
            "self.emit_leave_lexical_environment(function)",
        ),
        (
            "self.emit_leave_lexical_environment(function)",
            "CompletionKind::Continue.code()",
        ),
        (
            "CompletionKind::Continue.code()",
            "pending.target().load(function)",
        ),
        (
            "pending.target().load(function)",
            "pending.set_normal(pending.value(), function)",
        ),
        (
            "pending.set_normal(pending.value(), function)",
            "self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?",
        ),
        (
            "self.emit_sync_iterator_close(&iterator, &pending, &closed, function)?",
            "self.emit_dispatch_current_completion(function)?",
        ),
        (
            "self.emit_dispatch_current_completion(function)?",
            "self.emit_branch_to_target(loop_frame, function)",
        ),
    ] {
        assert_before(lifecycle, earlier, later);
    }
    assert_eq!(
        lifecycle.matches("self.emit_sync_iterator_close(").count(),
        1
    );
}

#[test]
fn fixture_and_current_failure_cohort_bound_the_claim() {
    for witness in [
        "head binding TDZ",
        "first fresh captured binding",
        "continue disposal before next without close",
        "outer continue disposal before close",
        "break disposal before close",
        "return disposal before close",
        "throw disposal before close",
        "disposer throw before close",
        "acquisition failure disposes then closes",
        "using binding immutable",
        "iterable[Symbol.iterator]",
    ] {
        assert!(FIXTURE.contains(witness), "missing CLI witness: {witness}");
    }
    for (source, witness) in VENDORED_WITNESSES.into_iter().zip([
        "for (using x of [x])",
        "creates a fresh binding per iteration",
        "for (using x of [null]) { x = { [Symbol.dispose]() { } }; }",
    ]) {
        assert!(
            source.contains(witness),
            "missing vendored boundary: {witness}"
        );
    }
    for path in [
        "language/statements/for-of/head-using-bound-names-fordecl-tdz.js",
        "language/statements/for-of/head-using-fresh-binding-per-iteration.js",
        "language/statements/using/syntax/using-invalid-assignment-statement-body-for-of.js",
    ] {
        assert!(CONTRACT.contains(path), "missing contract witness: {path}");
    }
    assert!(CONTRACT.contains("three files and six executions"));
    assert!(CONTRACT.contains("exactly one BindingIdentifier"));
    assert!(CONTRACT.contains("ordinary element-access assignment head"));
    assert!(CONTRACT.contains("for-await-of"));
    assert!(CONTRACT.contains("integrated current-SHA checkpoint is green"));
    assert!(CONTRACT.contains("6/6 sloppy/strict Wasm-AOT executions"));
}
