const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const FIXTURE: &str = include_str!("../../lila-cli/tests/fixtures/wasm_using_classic_for_head.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/synchronous-using-classic-for.md");
const VENDORED_WITNESSES: [&str; 5] = [
    include_str!(
        "../../../test262/vendor/test262/test/language/statements/using/syntax/using-for-statement.js"
    ),
    include_str!(
        "../../../test262/vendor/test262/test/language/statements/using/syntax/using-invalid-assignment-next-expression-for.js"
    ),
    include_str!(
        "../../../test262/vendor/test262/test/language/statements/using/syntax/using-outer-inner-using-bindings.js"
    ),
    include_str!(
        "../../../test262/vendor/test262/test/language/statements/using/initializer-disposed-at-end-of-forstatement.js"
    ),
    include_str!(
        "../../../test262/vendor/test262/test/language/statements/using/initializer-disposed-if-subsequent-initializer-throws-in-forstatement-head.js"
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
fn branch_completion_survives_finalizers_and_is_consumed_at_its_target() {
    let dispatch = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn emit_dispatch_branch_completion(",
        "    /// Every frame a pending Break completion",
    );
    let (through_finalizer, at_final_target) = dispatch
        .split_once("            } else {")
        .expect("branch dispatch must distinguish an intervening finalizer");

    assert!(through_finalizer.contains(
        "if let Some(finalizer) = self.active_finally_target_for_branch(*branch_target)"
    ));
    assert!(through_finalizer.contains("self.emit_branch_to_target(finalizer, function)"));
    assert!(!through_finalizer.contains("CompletionKind::Normal"));
    assert!(at_final_target.contains("self.set_completion_kind(CompletionKind::Normal, function)"));
    assert_before(
        at_final_target,
        "self.set_completion_kind(CompletionKind::Normal, function)",
        "self.emit_branch_to_target(*branch_target, function)",
    );
}

#[test]
fn closed_initializer_keeps_the_classic_for_as_the_direct_control_owner() {
    let initializer = bounded(
        IR_SOURCE,
        "pub enum ForInitIr {",
        "pub struct VarDeclaratorIr {",
    );
    assert!(initializer.contains("SyncDisposable(SyncDisposableResourcesIr)"));
    assert!(!initializer.contains("SyncDisposable(Vec<SyncDisposableResourceIr>)"));

    let lexical_init = bounded(
        LOWERING_SOURCE,
        "    fn lower_for_lexical_init(",
        "    /// Lowers a classic `for (using",
    );
    assert!(lexical_init.contains("LexicalDeclaration::Using(list)"));
    assert!(lexical_init.contains(".map(ForInitIr::SyncDisposable)"));

    let resource_init = bounded(
        LOWERING_SOURCE,
        "    fn lower_for_sync_disposable_init(",
        "    /// `for (let [a, b] = x;",
    );
    assert!(resource_init.contains("Option<SyncDisposableResourcesIr>"));
    assert!(resource_init.contains("SyncDisposableResourcesIr::new(first, resources.collect())"));
    assert!(!resource_init.contains("StatementIr::Block"));
    assert!(!resource_init.contains("ForInitIr::Lexical"));

    let loop_lowering = bounded(
        LOWERING_SOURCE,
        "    fn lower_for_loop(",
        "    /// The resume state a plain `async function` body",
    );
    assert!(loop_lowering.contains("StatementIr::For {\n                init,"));
    assert!(!loop_lowering.contains("StatementIr::Block(Box::new(StatementIr::For"));
    assert!(CONTRACT.contains("The containing node remains `StatementIr::For`"));
}

#[test]
fn backend_nests_continue_inside_one_disposal_capability_and_restores_after_it() {
    let dispatch = bounded(
        CONTROL_FLOW_SOURCE,
        "    pub(crate) fn compile_for(",
        "    fn compile_sync_disposable_for(",
    );
    assert!(dispatch.contains("if let Some(ForInitIr::SyncDisposable(resources)) = init"));
    assert!(dispatch.contains("return self.compile_sync_disposable_for("));
    assert_eq!(
        dispatch
            .matches("self.compile_classic_for_test(test, break_frame, function)?")
            .count(),
        1,
        "ordinary classic for must share the abrupt test boundary"
    );
    assert_eq!(
        dispatch
            .matches("self.compile_classic_for_update(update, function)?")
            .count(),
        1,
        "ordinary classic for must share the abrupt update boundary"
    );

    let test = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_classic_for_test(",
        "    fn compile_classic_for_update(",
    );
    for boundary in [
        "self.compile_truthy_i32(test, function)?",
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.result_local",
        "self.result_tag_local",
        "function.instruction(&Instruction::I32Eqz)",
        "self.emit_branch_if_to_target(false_target, function)",
    ] {
        assert!(test.contains(boundary), "missing test boundary: {boundary}");
    }
    assert_before(
        test,
        "self.emit_propagate_throw_from_locals_if_needed(",
        "function.instruction(&Instruction::I32Eqz)",
    );
    assert_before(
        test,
        "function.instruction(&Instruction::I32Eqz)",
        "self.emit_branch_if_to_target(false_target, function)",
    );

    let update = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_classic_for_update(",
        "    fn compile_sync_disposable_for(",
    );
    for boundary in [
        "self.compile_expr_payload(update, function)?",
        "function.instruction(&Instruction::Drop)",
        "self.emit_propagate_throw_from_locals_if_needed(",
        "self.result_local",
        "self.result_tag_local",
    ] {
        assert!(
            update.contains(boundary),
            "missing update boundary: {boundary}"
        );
    }
    assert_before(
        update,
        "function.instruction(&Instruction::Drop)",
        "self.emit_propagate_throw_from_locals_if_needed(",
    );

    let lifecycle = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_sync_disposable_for(",
        "    pub(crate) fn compile_switch(",
    );
    for boundary in [
        "debug_assert!(!resources.is_empty())",
        "!environment.per_iteration_slots.is_empty()",
        "if let Some(environment) = &runtime_environment",
        "self.emit_enter_lexical_environment(environment, function)?",
        "self.initialize_sync_disposable_resource_bindings(resources, function)",
        "self.finally_stack.push(disposal_frame)",
        "self.compile_sync_disposable_resource(resource, locals, function)",
        "self.compile_classic_for_test(test, disposal_frame, function)?",
        "let continue_frame = self.open_frame(ControlFrameKind::Block, function)",
        "self.push_labels(labels, break_frame, Some(continue_frame))",
        "self.compile_statement(body, function)",
        "self.compile_classic_for_update(update, function)?",
        "self.capture_pending_sync_dispose_completion(function)",
        "SyncDisposeCompletionContinuation::Dispatch",
        "self.emit_branch_to_target(break_frame, function)",
        "self.end_lexical_environment_scope()",
    ] {
        assert!(
            lifecycle.contains(boundary),
            "missing lifecycle boundary: {boundary}"
        );
    }

    assert_before(
        lifecycle,
        "self.emit_enter_lexical_environment(environment, function)?",
        "self.initialize_sync_disposable_resource_bindings(resources, function)",
    );
    assert_before(
        lifecycle,
        "self.initialize_sync_disposable_resource_bindings(resources, function)",
        "self.reserve_sync_disposable_resource_locals(function)",
    );
    assert_before(
        lifecycle,
        "self.finally_stack.push(disposal_frame)",
        "self.compile_sync_disposable_resource(resource, locals, function)",
    );
    assert_before(
        lifecycle,
        "self.compile_sync_disposable_resource(resource, locals, function)",
        "self.compile_classic_for_test(test, disposal_frame, function)?",
    );
    assert_before(
        lifecycle,
        "self.finally_stack.push(disposal_frame)",
        "let continue_frame = self.open_frame(ControlFrameKind::Block, function)",
    );
    assert_before(
        lifecycle,
        "self.push_labels(labels, break_frame, Some(continue_frame))",
        "self.compile_statement(body, function)",
    );
    assert_before(
        lifecycle,
        "self.compile_statement(body, function)",
        "self.compile_classic_for_update(update, function)?",
    );
    assert_before(
        lifecycle,
        "self.compile_classic_for_update(update, function)?",
        "function.branch_to_label(loop_frame.label)",
    );
    assert_before(
        lifecycle,
        "self.finally_stack.pop()",
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
        "self.emit_branch_to_target(break_frame, function)",
    );
    assert_before(
        lifecycle,
        "self.emit_branch_to_target(break_frame, function)",
        "self.end_lexical_environment_scope()",
    );
}

#[test]
fn temp_budget_keeps_all_acquired_entries_live_across_the_largest_loop_phase() {
    let statement_budget = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn count_statement_temp_locals(",
        "pub(crate) fn count_for_init_temp_locals(",
    );
    assert!(statement_budget.contains("Some(ForInitIr::SyncDisposable(resources))"));
    assert!(statement_budget.contains("test_temps.max(update_temps).max(body_temps)"));
    assert!(statement_budget.contains("count_sync_disposable_resources_temp_locals("));

    let resource_budget = bounded(
        PLANNING_SOURCE,
        "fn count_sync_disposable_resources_temp_locals(",
        "pub(crate) fn count_expr_temp_locals(",
    );
    assert!(resource_budget.contains("resources.len() * 5"));
    assert!(resource_budget.contains(".max(active_scope_temps)"));
    assert!(resource_budget.contains(".max(SYNC_DISPOSABLE_SCOPE_COMPLETION_TEMP_LOCALS)"));
}

#[test]
fn focused_oracle_covers_the_exact_vendored_boundaries_without_broad_claims() {
    for witness in [
        "for (using of = null; ; ) break",
        "immutable update TypeError",
        "outer binding restored",
        "later binding getter observes TDZ",
        "normal LIFO second",
        "subsequent initializer once",
        "suppression error order",
        "labelled continue reached update",
    ] {
        assert!(FIXTURE.contains(witness), "missing CLI witness: {witness}");
    }
    for (source, witness) in VENDORED_WITNESSES.into_iter().zip([
        "for (using of = null;;) break;",
        "for (using i = null; i === null; i = {",
        "outer using binding unchanged",
        "Initialized value is disposed at end of ForStatement",
        "Initialized value is disposed at end of FunctionBody",
    ]) {
        assert!(
            source.contains(witness),
            "missing vendored boundary: {witness}"
        );
    }
    assert!(CONTRACT.contains("initializer-disposed-at-end-of-forstatement.js"));
    assert!(CONTRACT
        .contains("initializer-disposed-if-subsequent-initializer-throws-in-forstatement-head.js"));
    assert!(CONTRACT.contains("two files and four executions"));
    assert!(CONTRACT.contains("does not claim the complete 78-file"));
}
