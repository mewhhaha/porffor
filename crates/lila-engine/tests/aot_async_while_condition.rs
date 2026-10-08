use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_modes(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compilation");
    for prefix in ["", "\"use strict\";\n"] {
        let source = format!("{prefix}{source}");
        let result = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("real JS -> Wasm async while");
        assert_eq!(result.backend_used, ExecutionBackend::WasmAot);
        assert!(matches!(result.completion, ObservedCompletion::Normal(_)));
        assert_eq!(
            result.output_events,
            expected
                .iter()
                .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn every_back_edge_evaluates_the_condition_again_without_repeating_operand_effects() {
    assert_modes(
        r#"
async function run() {
  let calls = 0;
  while (await (++calls < 3)) { print('body' + calls); }
  print('done' + calls);
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["body1", "body2", "done3"],
    );
}

#[test]
fn continue_and_break_run_their_eager_finally_before_selecting_the_next_phase() {
    assert_modes(
        r#"
async function run() {
  let calls = 0;
  while (await (++calls < 5)) {
    try {
      if (calls === 1) continue;
      if (calls === 3) break;
      print('body' + calls);
    } finally { print('finally' + calls); }
  }
  print('done' + calls);
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["finally1", "body2", "finally2", "finally3", "done3"],
    );
}

#[test]
fn rejected_condition_retains_identity_and_runs_the_surrounding_finally() {
    assert_modes(
        r#"
async function run() {
  const marker = {};
  try {
    while (await Promise.reject(marker)) { print('unreachable'); }
  } catch (error) { print(error === marker); }
  finally { print('finally'); }
  print('done');
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["true", "finally", "done"],
    );
}

#[test]
fn condition_operand_throw_follows_the_same_surrounding_completion_route() {
    assert_modes(
        r#"
async function run() {
  const marker = {};
  function condition() { throw marker; }
  try { while (await condition()) { print('unreachable'); } }
  catch (error) { print(error === marker); }
  finally { print('finally'); }
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["true", "finally"],
    );
}

#[test]
fn a_replaced_promise_is_read_on_every_iteration() {
    assert_modes(
        r#"
async function run() {
  const values = [];
  let p = Promise.resolve().then(() => {
    p = Promise.resolve().then(() => {
      p = Promise.resolve().then(() => { values.push(3); return false; });
      values.push(2); return true;
    });
    values.push(1); return true;
  });
  while (await p) {}
  print(values.join(','));
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["1,2,3"],
    );
}

#[test]
fn nested_condition_awaits_and_labelled_continue_and_break_remain_ordered() {
    assert_modes(
        r#"
async function run() {
  let calls = 0;
  outer: while (await await (++calls <= 3)) {
    if (calls === 1) continue outer;
    print('body' + calls);
    break outer;
  }
  print('done' + calls);
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["body2", "done2"],
    );
}

#[test]
fn zero_iterations_and_return_from_the_body_leave_no_active_condition() {
    assert_modes(
        r#"
async function run() {
  while (await false) { print('unreachable'); }
  try { while (await true) { return 'returned'; } }
  finally { print('finally'); }
  print('unreachable');
}
run().then(value => print(value), error => print('error:' + error));
void 0;
"#,
        &["finally", "returned"],
    );
}

#[test]
fn a_condition_prefix_repeats_once_per_iteration_and_not_on_reaction_resume() {
    assert_modes(
        r#"
async function run() {
  let calls = 0;
  while ((print('condition' + calls), await (++calls < 3))) {
    print('body' + calls);
  }
  print('done' + calls);
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &[
            "condition0",
            "body1",
            "condition1",
            "body2",
            "condition2",
            "done3",
        ],
    );
}

#[test]
fn even_an_awaited_primitive_resumes_after_the_call_returns() {
    assert_modes(
        r#"
async function run() {
  let calls = 0;
  while (await (++calls < 2)) { print('body'); }
  print('done');
}
run().then(() => print('fulfilled'), error => print('error:' + error));
print('called');
"#,
        &["called", "body", "done", "fulfilled"],
    );
}

#[test]
fn each_eager_body_keeps_its_own_captured_block_binding() {
    assert_modes(
        r#"
async function run() {
  const readers = [];
  let calls = 0;
  while (await (++calls <= 2)) {
    const value = calls;
    readers.push(() => value);
  }
  print(readers[0]() + ',' + readers[1]());
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["1,2"],
    );
}

#[test]
fn a_body_finally_can_replace_continue_with_return() {
    assert_modes(
        r#"
async function run() {
  while (await true) {
    try { continue; }
    finally { return 'returned'; }
  }
  print('unreachable');
}
run().then(value => print(value), error => print('error:' + error));
void 0;
"#,
        &["returned"],
    );
}

#[test]
fn a_body_finally_can_replace_break_with_a_surrounding_throw() {
    assert_modes(
        r#"
async function run() {
  const marker = {};
  try {
    while (await true) {
      try { break; }
      finally { throw marker; }
    }
    print('unreachable');
  } catch (error) { print(error === marker); }
  finally { print('finally'); }
  print('done');
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["true", "finally", "done"],
    );
}

#[test]
fn labelled_block_commits_its_exit_before_the_following_await() {
    assert_modes("async function run() { outer: { while (await true) { print('body'); break outer; } } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "body", "done", "fulfilled"]);
}

#[test]
fn labelled_block_resumes_after_an_earlier_await() {
    assert_modes("async function run() { await 0; outer: { while (await true) { print('body'); break outer; } } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "body", "done", "fulfilled"]);
}

#[test]
fn early_labelled_break_commits_exit_without_evaluating_the_condition() {
    assert_modes("async function run() { outer: { print('entered'); break outer; while ((print('bypassed-condition'), await true)) { print('bypassed-body'); } } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["entered", "called", "done", "fulfilled"]);
}

#[test]
fn nested_condition_under_label_reuses_branch_and_eager_finally_owners() {
    assert_modes("async function run(flag) { outer: { try { if (flag) { while (await true) { print('body'); break outer; } } } finally { print('finally'); } } await 0; print('done'); } run(true).then(() => print('fulfilled')); print('called');", &["called", "body", "finally", "done", "fulfilled"]);
}

#[test]
fn labelled_try_commits_its_exit_after_the_awaited_finalizer() {
    assert_modes("async function run() { outer: try { while (await true) { print('body'); break outer; } } finally { print('finally'); await 0; print('finalized'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "body", "finally", "finalized", "done", "fulfilled"]);
}

#[test]
fn a_skipped_labelled_branch_advances_to_the_following_await() {
    assert_modes("async function run(flag) { outer: if (flag) { while ((print('bypassed-condition'), await true)) { print('bypassed-body'); break outer; } } await 0; print('done'); } run(false).then(() => print('fulfilled')); print('called');", &["called", "done", "fulfilled"]);
}

#[test]
fn normal_label_completion_keeps_post_loop_effects_and_back_edges() {
    assert_modes("async function run() { let calls = 0; outer: { print('entered'); while (await (++calls <= 2)) { print('body' + calls); } print('region-done'); } await 0; print('done' + calls); } run().then(() => print('fulfilled')); print('called');", &["entered", "called", "body1", "body2", "region-done", "done3", "fulfilled"]);
}

#[test]
fn a_resumed_early_break_bypasses_a_later_condition_and_prefix() {
    assert_modes("async function run() { outer: { await 0; print('resumed'); break outer; while ((print('bypassed-condition'), await true)) {} } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "resumed", "done", "fulfilled"]);
}

#[test]
fn a_bypassing_break_waits_for_finalization_before_the_label_epilogue() {
    assert_modes("async function run() { outer: try { print('entered'); break outer; while ((print('bypassed-condition'), await true)) {} } finally { print('finally'); await 0; print('finalized'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["entered", "finally", "called", "finalized", "done", "fulfilled"]);
}

#[test]
fn an_outer_label_break_bypasses_inner_epilogues_after_awaited_finalization() {
    assert_modes("async function run() { outer: { inner: { try { while (await true) { print('body'); break outer; } } finally { print('finally'); await 0; print('finalized'); } print('bypassed-inner'); } print('bypassed-outer'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "body", "finally", "finalized", "done", "fulfilled"]);
}

#[test]
fn awaited_finalizer_return_replaces_a_label_break_without_running_its_tail() {
    assert_modes("async function run() { outer: try { while (await true) { break outer; } } finally { await 0; return 'returned'; } await 0; print('bypassed'); } run().then(value => print(value)); print('called');", &["called", "returned"]);
}

#[test]
fn awaited_finalizer_throw_replaces_a_label_break_and_preserves_identity() {
    assert_modes("async function run() { const marker = {}; try { outer: try { while (await true) { break outer; } } finally { await 0; throw marker; } print('bypassed'); } catch (error) { print(error === marker); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "true", "done", "fulfilled"]);
}

#[test]
fn a_rejected_labelled_condition_keeps_the_surrounding_catch_and_finally() {
    assert_modes("async function run() { const marker = {}; try { outer: { while (await Promise.reject(marker)) { print('bypassed-body'); } print('bypassed-tail'); } } catch (error) { print(error === marker); } finally { await 0; print('finally'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "true", "finally", "done", "fulfilled"]);
}

#[test]
fn an_owning_label_restores_and_unwinds_its_captured_block_environment() {
    assert_modes("async function run() { let reader; outer: { const value = 'captured'; reader = () => value; await 0; while (await true) { break outer; } print('bypassed'); } await 0; print(reader()); } run().then(() => print('fulfilled')); print('called');", &["called", "captured", "fulfilled"]);
}

#[test]
fn a_label_inside_finally_preserves_the_pending_outer_label_break() {
    assert_modes("async function run() { outer: try { break outer; while (await false) {} } finally { inner: { while (await true) { print('inner'); break inner; } } await 0; print('finalized'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "inner", "finalized", "done", "fulfilled"]);
}

#[test]
fn independent_nested_functions_remain_opaque_to_the_label_owner_census() {
    assert_modes("async function run() { outer: { const helper = async function() { while (await true) { break; } return 'inner'; }; helper().then(value => print(value)); break outer; } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "done", "inner", "fulfilled"]);
}

#[test]
fn ancestor_loop_continue_resumes_the_owning_test_phase() {
    assert_modes("async function run() { let i=0; ancestor: while (i++ < 2) { region: { while (await false) {} } print(i); continue ancestor; } print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "1", "2", "done", "fulfilled"]);
}

#[test]
fn ancestor_loop_break_waits_for_awaited_finalization() {
    assert_modes("async function run() { ancestor: while (true) { region: { try { while (await false) {} } finally { await 0; print('finalized'); break ancestor; } } } print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "finalized", "done", "fulfilled"]);
}

#[test]
fn implicit_disposal_body_reports_a_compiler_gap() {
    assert_modes(
        "async function run() { while (await true) { await using resource = null; break; } } run();",
        &[],
    );
}

#[test]
fn implicit_for_await_body_reports_a_compiler_gap() {
    // This formerly refused implicit iterator body now has a complete owner.
    assert_modes(
        "async function run() { while (await true) { for await (const value of []) {} break; } } run();",
        &[],
    );
}

#[test]
fn implicit_await_using_iterator_head_reports_a_compiler_gap() {
    assert_modes(
        "async function run() { while (await true) { for (await using resource of []) {} break; } } run();",
        &[],
    );
}

#[test]
fn implicit_await_using_classic_for_head_reports_a_compiler_gap() {
    assert_modes(
        "async function run() { while (await true) { for (await using resource = null; false;) {} break; } } run();",
        &[],
    );
}

#[test]
fn condition_under_switch_case_reaches_the_following_await() {
    assert_modes("async function run() { switch (0) { default: while (await true) { print('body'); break; } } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "body", "done", "fulfilled"]);
}

#[test]
fn early_switch_break_before_condition_commits_its_own_exit() {
    assert_modes("async function run() { switch (0) { default: break; while (await true) { print('bypassed'); } } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "done", "fulfilled"]);
}

#[test]
fn independent_helper_await_does_not_suspend_the_directly_labelled_eager_body() {
    assert_modes(
        r#"
async function run() {
  outer: inner: while (await true) {
    const helper = async function() { await 0; return 'inner'; };
    helper().then(value => print(value));
    break outer;
  }
  print('done');
}
run().then(() => print('fulfilled'), error => print('error:' + error));
"#,
        &["done", "fulfilled", "inner"],
    );
}

#[test]
fn a_direct_await_label_advances_to_its_following_await() {
    assert_modes("async function run() { outer: await 0; await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "done", "fulfilled"]);
}

#[test]
fn a_block_await_label_restores_its_body_cell_before_the_following_await() {
    assert_modes("async function run() { outer: { const value = 'cell'; print('entered'); await 0; print(value); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["entered", "called", "cell", "done", "fulfilled"]);
}

#[test]
fn a_labelled_if_await_resumes_before_its_matching_break() {
    assert_modes("async function run(flag) { outer: if (flag) { await 0; print('resumed'); break outer; } await 0; print('done'); } run(true).then(() => print('fulfilled')); print('called');", &["called", "resumed", "done", "fulfilled"]);
}

#[test]
fn a_skipped_labelled_if_does_not_evaluate_its_await_operand() {
    assert_modes("async function run(flag) { outer: if (flag) { await print('bypassed'); } await 0; print('done'); } run(false).then(() => print('fulfilled')); print('called');", &["called", "done", "fulfilled"]);
}

#[test]
fn a_labelled_try_await_keeps_break_and_awaited_finally_order() {
    assert_modes("async function run() { outer: try { await 0; print('resumed'); break outer; } finally { await 0; print('finalized'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "resumed", "finalized", "done", "fulfilled"]);
}

#[test]
fn an_early_break_bypasses_the_first_ordinary_await_operand() {
    assert_modes("async function run() { outer: { print('entered'); break outer; await print('bypassed'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["entered", "called", "done", "fulfilled"]);
}

#[test]
fn a_resumed_break_bypasses_the_later_ordinary_await_operand() {
    assert_modes("async function run() { outer: { await 0; print('resumed'); break outer; await print('bypassed'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "resumed", "done", "fulfilled"]);
}

#[test]
fn a_bypassed_ordinary_await_waits_for_the_awaited_finalizer() {
    assert_modes("async function run() { outer: try { print('entered'); break outer; await print('bypassed'); } finally { print('finally'); await 0; print('finalized'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["entered", "finally", "called", "finalized", "done", "fulfilled"]);
}

#[test]
fn an_inner_ordinary_await_label_keeps_a_pending_outer_break() {
    assert_modes("async function run() { outer: try { break outer; await print('bypassed'); } finally { inner: { await 0; print('inner'); break inner; await print('bypassed-inner'); } await 0; print('finalized'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "inner", "finalized", "done", "fulfilled"]);
}

#[test]
fn ordinary_await_rejection_in_a_label_preserves_catch_identity_and_finally() {
    assert_modes("async function run() { const marker = {}; try { outer: { await Promise.reject(marker); print('bypassed'); } } catch (error) { print(error === marker); } finally { await 0; print('finally'); } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "true", "finally", "done", "fulfilled"]);
}

#[test]
fn switch_case_ordinary_await_reaches_the_following_label_exit() {
    assert_modes("async function run() { outer: { switch (0) { default: await 0; print('case'); } } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "case", "done", "fulfilled"]);
}

#[test]
fn an_early_switch_break_cannot_hide_an_ordinary_await() {
    assert_modes("async function run() { outer: { switch (0) { default: break; await print('bypassed'); } } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "done", "fulfilled"]);
}

#[test]
fn switch_discriminant_await_is_owned_but_suspended_selector_references_remain_unowned() {
    assert_modes("async function run() { outer: { switch (await 0) { default: print('case'); break; } } await 0; print('done'); } run().then(() => print('fulfilled')); print('called');", &["called", "case", "done", "fulfilled"]);
    assert_modes(
        "async function run(target) { outer: { switch (0) { case target[await 0] &&= await 0: break; } } await 0; } run({});",
        &[],
    );
}

#[test]
fn switch_implicit_disposal_and_iterator_awaits_remain_unowned() {
    for source in [
        "async function run() { outer: { switch (0) { default: { await using resource = null; break; } } } await 0; } run();",
        "async function run() { outer: { switch (0) { default: for await (const value of []) {} break; } } await 0; } run();",
    ] {
        assert_modes(source, &[]);
    }
}

#[test]
fn directly_labelled_awaited_body_while_reaches_the_following_await() {
    assert_modes("async function run() { let n = 0; outer: inner: while (n++ < 1) { await 0; print('body'); } await 0; print('done' + n); } run().then(() => print('fulfilled')); print('called');", &["called", "body", "done2", "fulfilled"]);
}

#[test]
fn directly_labelled_awaited_body_for_reaches_the_following_await() {
    assert_modes("async function run() { let n = 0; outer: inner: for (; n < 2; ++n) { await 0; print('body' + n); } await 0; print('done' + n); } run().then(() => print('fulfilled')); print('called');", &["called", "body0", "body1", "done2", "fulfilled"]);
}

#[test]
fn restartable_conditions_preserve_receivers_skipped_calls_and_fresh_class_environments() {
    assert_modes(
        r#"
async function run() {
  let trace = '', calls = 0;
  const object = { get method() {
    trace += 'g';
    return function(value) { trace += (this === object ? 'r' : 'x') + value; return ++calls < 2; };
  } };
  while (object?.[await 'method'](await calls)) { trace += 'b'; }
  print(trace);
  let touched = 0;
  while (null?.(await (++touched))) { touched += 100; }
  print(touched);
  let keyCalls = 0;
  const target = { value: true };
  while (target[await (keyCalls++, 'value')] &&= await false) { print('bypassed'); }
  print(keyCalls + ':' + target.value);
  let iteration = 0;
  const captures = [];
  while (class Named { [await (captures.push(() => Named), 'key')]() {} }) {
    print(captures[iteration]().name);
    if (++iteration === 2) break;
  }
  print(captures[0]() !== captures[1]());
  print('done');
}
run().catch(error => print('error:' + error));
void 0;
"#,
        &["gr0bgr1", "0", "1:false", "Named", "Named", "true", "done"],
    );
}
