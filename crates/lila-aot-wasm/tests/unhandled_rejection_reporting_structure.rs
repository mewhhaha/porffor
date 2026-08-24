const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const CLI_MAIN_SOURCE: &str = include_str!("../../lila-cli/tests/cli/main.rs");
const CLI_FUNCTIONS_SOURCE: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CONTRACT: &str = include_str!("../../../docs/rust-rewrite/contracts/main-job-checkpoint.md");
const NORMAL_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections.js");
const PRIMARY_THROW_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections_with_primary_throw.js"
);

fn reporter() -> &'static str {
    PROMISE_SOURCE
        .split_once("    pub(crate) fn emit_report_unhandled_rejection(")
        .expect("unhandled-rejection reporter should exist")
        .1
        .split_once("    pub(crate) fn emit_promise_constructor(")
        .expect("unhandled-rejection reporter should remain bounded")
        .0
}

#[test]
fn reporter_detaches_a_finite_fifo_and_prints_each_required_entry_once() {
    let reporter = reporter();

    assert_eq!(
        reporter
            .matches("emit_load_promise_state_strict(record_local, state_local, function)")
            .count(),
        1,
        "each candidate must cross the strict Promise-state boundary once"
    );
    assert_eq!(
        reporter
            .matches("HEAP_PROMISE_UNHANDLED_NEXT_OFFSET")
            .count(),
        2,
        "the old tail must be severed once and the FIFO cursor must advance through one link load"
    );
    assert_eq!(
        reporter.matches("Instruction::Br(2)").count(),
        0,
        "the retired first-unhandled early exit must not return"
    );
    assert_eq!(
        reporter
            .matches("Instruction::Call(HOST_PRINT_IMPORT_FUNCTION_INDEX)")
            .count(),
        1,
        "one emitted host call inside the loop must serve every diagnostic entry"
    );

    let snapshot_head = reporter
        .find("Instruction::LocalSet(snapshot_head_local)")
        .expect("the checkpoint head must be captured");
    let snapshot_tail = reporter
        .find("Instruction::LocalSet(snapshot_tail_local)")
        .expect("the checkpoint tail must be captured");
    let detach_head = reporter
        .find("Instruction::GlobalSet(\n            PROMISE_UNHANDLED_REJECTION_HEAD_GLOBAL_INDEX,")
        .expect("the live tracker head must be detached");
    let detach_tail = reporter
        .find("Instruction::GlobalSet(\n            PROMISE_UNHANDLED_REJECTION_TAIL_GLOBAL_INDEX,")
        .expect("the live tracker tail must be detached");
    let sever_snapshot_tail = reporter
        .find("self.store_i64_const_at_offset(\n            snapshot_tail_local,\n            HEAP_PROMISE_UNHANDLED_NEXT_OFFSET,")
        .expect("the detached snapshot tail must be severed");
    let strict_load = reporter
        .find("emit_load_promise_state_strict(record_local, state_local, function)")
        .expect("strict Promise-state load should exist");
    let handled_load = reporter
        .find("HEAP_PROMISE_IS_HANDLED_OFFSET")
        .expect("handled mark should be re-read");
    let oldest_selection = reporter
        .match_indices("Instruction::LocalSet(oldest_unhandled_local)")
        .nth(1)
        .map(|(position, _)| position)
        .expect("oldest unhandled candidate should be retained");
    let host_call = reporter
        .find("Instruction::Call(HOST_PRINT_IMPORT_FUNCTION_INDEX)")
        .expect("host diagnostic call should exist");
    let cursor_advance = reporter
        .match_indices("HEAP_PROMISE_UNHANDLED_NEXT_OFFSET")
        .nth(1)
        .map(|(position, _)| position)
        .expect("FIFO cursor advance should exist");

    assert!(snapshot_head < snapshot_tail);
    assert!(snapshot_tail < detach_head);
    assert!(detach_head < detach_tail);
    assert!(detach_tail < sever_snapshot_tail);
    assert!(sever_snapshot_tail < strict_load);
    assert!(strict_load < handled_load);
    assert!(handled_load < oldest_selection);
    assert!(oldest_selection < host_call);
    assert!(host_call < cursor_advance);
    assert_eq!(
        reporter
            .matches("PROMISE_UNHANDLED_REJECTION_HEAD_GLOBAL_INDEX")
            .count(),
        2,
        "the reporter may capture and detach the head, but must not clear a fresh reentrant FIFO"
    );
    assert_eq!(
        reporter
            .matches("PROMISE_UNHANDLED_REJECTION_TAIL_GLOBAL_INDEX")
            .count(),
        2,
        "the reporter may capture and detach the tail, but must not clear a fresh reentrant FIFO"
    );
}

#[test]
fn diagnostic_conversion_is_explicit_and_cannot_replace_the_primary_completion() {
    let reporter = reporter();

    assert!(reporter.contains("emit_symbol_descriptive_string_to_local("));
    assert!(reporter.contains("store_call_results_to("));
    assert!(reporter.contains("UNHANDLED_REJECTION_TOSTRING_THROWN_MESSAGE"));
    assert!(
        !reporter.contains("emit_value_to_string_payload("),
        "the propagating ToString wrapper would overwrite or return past the primary completion"
    );

    let save_name = reporter
        .find("Instruction::LocalSet(saved_throw_error_name_local)")
        .expect("primary error name should be saved");
    let conversion = reporter
        .find("store_call_results_to(")
        .expect("conversion result tuple should be inspected without propagation");
    let marker = reporter
        .find("UNHANDLED_REJECTION_TOSTRING_THROWN_MESSAGE")
        .expect("conversion failure should have visible fallback evidence");
    let restore_name = reporter
        .rfind("Instruction::LocalGet(saved_throw_error_name_local)")
        .expect("primary error name should be restored");
    let promotion = reporter
        .rfind("self.set_completion_kind_with_aux(CompletionKind::Throw, -1, function)")
        .expect("the oldest rejection should be promoted after diagnostics");

    assert!(save_name < conversion);
    assert!(conversion < marker);
    assert!(marker < restore_name);
    assert!(restore_name < promotion);
}

#[test]
fn heap_modules_own_the_unhandled_diagnostic_print_import() {
    let authority = EMIT_SOURCE
        .split_once("let uses_host_print = ")
        .expect("host-print import authority should exist")
        .1
        .split_once(';')
        .expect("host-print import authority should be one expression")
        .0;

    assert!(authority.contains("uses_heap"));
    assert!(authority.contains("compiled_host_builtins.contains(&HostBuiltinId::Print)"));
    assert!(
        !NORMAL_FIXTURE.contains("print(") && !PRIMARY_THROW_FIXTURE.contains("print("),
        "the public fixtures must prove reporting without source-level print reachability"
    );
}

#[test]
fn main_export_routes_the_checkpoint_after_drain_and_registers_public_cli_tests() {
    assert_eq!(
        EMIT_SOURCE
            .matches("self.emit_report_unhandled_rejection(&mut function)?;")
            .count(),
        1,
        "the product main export must retain one rejection checkpoint call"
    );
    let main_checkpoint = EMIT_SOURCE
        .rsplit_once("if self.is_main() && self.uses_heap {")
        .expect("the heap-backed main-export checkpoint should exist")
        .1
        .split_once("        assert!(")
        .expect("the checkpoint should remain bounded by the local-planner assertion")
        .0;
    let final_job_drain = main_checkpoint
        .rfind("self.emit_drain_promise_jobs(&mut function)?;")
        .expect("Promise jobs must drain before rejection reporting");
    let rejection_report = main_checkpoint
        .find("self.emit_report_unhandled_rejection(&mut function)?;")
        .expect("the main export must route through rejection reporting");
    assert!(final_job_drain < rejection_report);

    assert_eq!(CLI_MAIN_SOURCE.matches("mod functions;").count(), 1);
    for (test_name, fixture_name) in [
        (
            "run_wasm_backend_reports_every_unhandled_rejection_in_fifo_order",
            "wasm_multiple_unhandled_rejections.js",
        ),
        (
            "run_wasm_backend_reports_all_rejections_without_replacing_a_primary_throw",
            "wasm_multiple_unhandled_rejections_with_primary_throw.js",
        ),
    ] {
        let registration = format!("#[test]\nfn {test_name}() {{");
        assert_eq!(
            CLI_FUNCTIONS_SOURCE.matches(&registration).count(),
            1,
            "public CLI regression lost test registration: {test_name}"
        );
        assert_eq!(
            CLI_FUNCTIONS_SOURCE.matches(fixture_name).count(),
            1,
            "public CLI regression lost fixture routing: {fixture_name}"
        );
    }
}

#[test]
fn contract_and_public_fixtures_pin_values_ordering_and_failure_precedence() {
    for marker in [
        "wasm_multiple_unhandled_rejections.js",
        "wasm_multiple_unhandled_rejections_with_primary_throw.js",
        "SymbolDescriptiveString",
        "unhandled rejection diagnostic ToString threw",
        "FIFO order",
        "finite FIFO snapshot",
        "recursively rejecting",
        "print_line_utf8",
        "emit_track_unhandled_rejection",
        "emit_script_with_forced_builtins",
        "realm-owned",
    ] {
        assert!(CONTRACT.contains(marker), "contract lost marker: {marker}");
    }
    for marker in [
        "checkpoint-first",
        "checkpoint-handled",
        "checkpoint-second",
        "checkpoint-symbol",
        "checkpoint-conversion",
        "checkpoint-reentrant",
        "Promise.reject(recursivelyRejected)",
        "checkpoint-third",
    ] {
        assert!(
            NORMAL_FIXTURE.contains(marker),
            "normal-completion fixture lost witness: {marker}"
        );
    }
    for marker in [
        "primary-first",
        "primary-handled",
        "primary-second",
        "primary-conversion",
        "primary-third",
        "primary-script-failure",
    ] {
        assert!(
            PRIMARY_THROW_FIXTURE.contains(marker),
            "primary-throw fixture lost witness: {marker}"
        );
    }
}
