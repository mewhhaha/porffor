const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_using_for_of_lifecycle.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/synchronous-using-for-of.md");
const VENDORED_WITNESSES: [&str; 3] = [
    include_str!(
        "../../../test262/vendor/test262/test/language/statements/for-of/head-using-bound-names-fordecl-tdz.js"
    ),
    include_str!(
        "../../../test262/vendor/test262/test/language/statements/for-of/head-using-fresh-binding-per-iteration.js"
    ),
    include_str!(
        "../../../test262/vendor/test262/test/language/statements/using/syntax/using-invalid-assignment-statement-body-for-of.js"
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

    let statements = bounded(IR_SOURCE, "    ForOfArray {", "    ForInArray {");
    assert_eq!(
        statements.matches("head: ForOfAssignmentIr").count(),
        2,
        "only Array and String index walks accept an ordinary assignment head"
    );
    let iterator = bounded(statements, "    ForOfIterator {", "    },");
    assert!(iterator.contains("head: ForOfIteratorHeadIr"));
    assert!(!iterator.contains("async_plan:"));
    assert!(!iterator.contains("protocol:"));
}

#[test]
fn lowering_keeps_tdz_and_specialization_decisions_at_the_closed_head_boundary() {
    let lowering = bounded(
        LOWERING_SOURCE,
        "    fn lower_for_of_head(&mut self, for_of: &ForOfLoop) -> ForOfLoweringIr {",
        "    fn lower_for_init(&mut self, init: &ForLoopInitializer) -> Option<ForInitIr> {",
    );
    for boundary in [
        "IterableLoopInitializer::Using(Binding::Identifier(identifier))",
        "LoweredForOfHeadKind::SyncDisposable",
        "BindingMode::Const",
        "self.lower_for_head_expression_with_tdz(mode, &name, for_of.iterable())",
        "&& head_kind == LoweredForOfHeadKind::Assignment",
        "match head_kind",
        "ForOfIteratorHeadIr::SyncDisposable(",
        "SyncDisposableForOfHeadIr::new(storage_name)",
        "IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL",
    ] {
        assert!(
            lowering.contains(boundary),
            "missing lowering boundary: {boundary}"
        );
    }
    assert_eq!(
        lowering
            .matches("&& head_kind == LoweredForOfHeadKind::Assignment")
            .count(),
        2,
        "both Array and String specializations must reject resource heads"
    );
    assert_before(
        lowering,
        "self.lower_for_head_expression_with_tdz(mode, &name, for_of.iterable())",
        "ForOfIteratorHeadIr::SyncDisposable(",
    );
}

#[test]
fn comma_operands_route_abrupt_completion_before_the_right_operand() {
    let sites = EXPRESSIONS_SOURCE
        .match_indices("ExprIr::Comma { lhs, rhs } => {")
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(
        sites.len(),
        2,
        "both ordinary comma emitters must be pinned"
    );

    for (site, rhs) in sites.into_iter().zip([
        "self.compile_expr_payload(rhs, function)?;",
        "self.compile_expr_to_locals(rhs, payload_local, tag_local, function)?;",
    ]) {
        let arm = EXPRESSIONS_SOURCE[site..]
            .split_once("            ExprIr::MaterializeBinding")
            .expect("comma arm must end before MaterializeBinding")
            .0;
        assert_before(
            arm,
            "self.compile_expr_to_locals(",
            "self.emit_propagate_throw_from_locals_if_needed(",
        );
        assert_before(arm, "self.emit_propagate_throw_from_locals_if_needed(", rhs);
    }
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
    assert!(head_witness.contains("SyncDisposable(&'a SyncDisposableForOfHeadIr)"));
    assert!(!head_witness.contains("derive(Clone"));
    assert!(!head_witness.contains("derive(Copy"));

    let lifecycle_witness = bounded(
        CONTROL_FLOW_SOURCE,
        "#[must_use = \"a synchronous for-of iteration must finish assignment or disposal\"]",
        "#[derive(Clone, Copy)]\nenum SyncDisposeCompletionContinuation",
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
    assert!(consumer.contains("self.restore_saved_completion("));
    assert!(consumer.contains("match continuation"));
    assert!(consumer.contains(
        "SyncDisposeCompletionContinuation::Dispatch => {\n                self.emit_dispatch_current_completion(function)?;"
    ));
    assert!(consumer.contains("SyncDisposeCompletionContinuation::DeferToIteratorClose => {}"));
    assert_before(
        consumer,
        "self.restore_saved_completion(",
        "match continuation",
    );

    for (start, end) in [
        (
            "    pub(crate) fn compile_statement(",
            "    fn compile_return_position_expr(",
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
        assert!(dispatch.contains("SyncForOfIteratorHead::SyncDisposable(head)"));
        assert!(dispatch.contains("self.compile_for_of_iterator("));
    }

    let lifecycle = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_for_of_iterator(",
        "    pub(crate) fn compile_object_destructure_to_locals(",
    );
    for boundary in [
        "let lifecycle = match head",
        "SyncForOfIterationLifecycleLocals::Assignment(binding)",
        "SyncForOfIterationLifecycleLocals::SyncDisposable {",
        "self.reserve_sync_disposable_resource_locals(function)",
        "self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?",
        "self.compile_expr_to_locals(",
        "self.emit_leave_for_in_of_tdz_scope(environment, function)",
        "self.emit_enter_lexical_environment(environment, function)?",
        "let continue_frame = self.open_frame(ControlFrameKind::Block, function)",
        "self.loop_stack.push(LoopTargets { continue_frame })",
        "self.finally_stack.push(finally_frame)",
        "self.initialize_binding_uninitialized(storage, function)",
        "self.reset_sync_disposable_resource_locals(acquired, function)",
        "self.finally_stack.push(disposal_frame)",
        "self.compile_sync_disposable_resource_from_locals(storage, acquired, function)?",
        "self.push_labels(labels, break_frame, Some(continue_frame))",
        "self.compile_statement(body, function)?",
        "self.capture_pending_sync_dispose_completion(function)",
        "self.consume_sync_disposable_resources(",
        "SyncDisposeCompletionContinuation::DeferToIteratorClose",
        "self.save_current_completion(",
        "COMPLETION_KIND_CONTINUE",
        "self.emit_leave_lexical_environment(function)",
        "self.emit_iterator_close_condition_i32(",
        "self.emit_iterator_close_preserving_current_throw(",
        "self.emit_iterator_close(",
        "self.emit_dispatch_current_completion(function)?",
        "function.branch_to_label(loop_frame.label)",
    ] {
        assert!(
            lifecycle.contains(boundary),
            "missing backend boundary: {boundary}"
        );
    }

    assert_before(
        lifecycle,
        "self.emit_enter_for_in_of_tdz_scope(mode, environment, function)?",
        "self.compile_expr_to_locals(",
    );
    assert_before(
        lifecycle,
        "self.compile_expr_to_locals(",
        "self.emit_leave_for_in_of_tdz_scope(environment, function)",
    );
    assert_before(
        lifecycle,
        "self.emit_enter_lexical_environment(environment, function)?",
        "self.initialize_binding_uninitialized(storage, function)",
    );
    assert_before(
        lifecycle,
        "self.finally_stack.push(finally_frame)",
        "self.initialize_binding_uninitialized(storage, function)",
    );
    assert_before(
        lifecycle,
        "self.finally_stack.push(disposal_frame)",
        "self.compile_sync_disposable_resource_from_locals(storage, acquired, function)?",
    );
    assert_before(
        lifecycle,
        "self.compile_sync_disposable_resource_from_locals(storage, acquired, function)?",
        "self.compile_statement(body, function)?",
    );
    assert_before(
        lifecycle,
        "self.compile_statement(body, function)?",
        "self.capture_pending_sync_dispose_completion(function)",
    );
    assert_before(
        lifecycle,
        "self.capture_pending_sync_dispose_completion(function)",
        "self.consume_sync_disposable_resources(",
    );
    assert_before(
        lifecycle,
        "self.consume_sync_disposable_resources(",
        "SyncDisposeCompletionContinuation::DeferToIteratorClose",
    );
    assert_before(
        lifecycle,
        "SyncDisposeCompletionContinuation::DeferToIteratorClose",
        "self.save_current_completion(",
    );
    assert_before(
        lifecycle,
        "COMPLETION_KIND_CONTINUE",
        "self.emit_iterator_close_condition_i32(",
    );
    assert_before(
        lifecycle,
        "SyncDisposeCompletionContinuation::DeferToIteratorClose",
        "self.emit_leave_lexical_environment(function)",
    );
    assert_before(
        lifecycle,
        "self.save_current_completion(",
        "self.emit_leave_lexical_environment(function)",
    );
    assert_before(
        lifecycle,
        "self.emit_leave_lexical_environment(function)",
        "COMPLETION_KIND_CONTINUE",
    );
    assert_before(
        lifecycle,
        "self.emit_leave_lexical_environment(function)",
        "self.emit_iterator_close_condition_i32(",
    );
    assert_before(
        lifecycle,
        "self.emit_iterator_close_condition_i32(",
        "self.emit_dispatch_current_completion(function)?",
    );
    assert_before(
        lifecycle,
        "self.emit_dispatch_current_completion(function)?",
        "function.branch_to_label(loop_frame.label)",
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
