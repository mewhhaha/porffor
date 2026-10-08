use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_modes(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for prefix in ["", "\"use strict\";\n"] {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{prefix}{source}"),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("actual compiled async switch");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(matches!(observed.completion, ObservedCompletion::Normal(_)));
        assert_eq!(
            observed.output_events,
            expected
                .iter()
                .map(|line| HostOutputEvent::PrintLine((*line).into()))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn eager_selectors_stop_at_first_match_and_default_middle_does_not_preempt_it() {
    assert_modes(
        r#"
async function run() {
  switch (2) {
    case (print('test0'), 0): print('wrong0'); break;
    default: print('wrongdefault'); break;
    case (print('test2'), 2): await 0; print('body2');
    case (print('wrongtest3'), 3): await 0; print('body3'); break;
  }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["test0", "test2", "called", "body2", "body3", "done"],
    );
}

#[test]
fn default_fallback_waits_for_later_selectors_then_falls_through_without_retesting() {
    assert_modes(
        r#"
async function run() {
  switch (9) {
    case (print('test0'), 0): print('wrong0'); break;
    default: await 0; print('default');
    case (print('test2'), 2): await 0; print('tail');
  }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["test0", "test2", "called", "default", "tail", "done"],
    );
}

#[test]
fn no_match_commits_exit_and_does_not_run_skipped_awaits() {
    assert_modes(
        r#"
async function run() {
  switch (9) { case 0: await 0; print('wrong0'); case 1: await 0; print('wrong1'); }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "done"],
    );
}

#[test]
fn awaited_discriminant_uses_outer_binding_and_saved_case_cell_is_not_recreated() {
    assert_modes(
        r#"
async function run() {
  let value = 9;
  switch (await value) {
    case 9:
      let value = 1;
      const read = () => value;
      await 0; print(read());
      value = 2; await 0; print(read()); break;
  }
  await 0; print(value);
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "1", "2", "9"],
    );
}

#[test]
fn selectors_run_after_shared_case_tdz_instantiation() {
    assert_modes(
        r#"
async function run() {
  let value = 1;
  try { switch (await value) { case value: let value = 2; await 0; print('wrong'); } }
  catch (error) { print(error instanceof ReferenceError); }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "true", "done"],
    );
}

#[test]
fn early_switch_break_bypasses_case_continuations_but_reaches_following_await() {
    assert_modes(
        r#"
async function run() {
  switch (0) { default: break; await print('wrong'); }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "done"],
    );
}

#[test]
fn switch_break_waits_for_awaited_finally_before_leaving_case_environment() {
    assert_modes(
        r#"
async function run() {
  switch (0) { default: try { await 0; print('try'); break; } finally { await 0; print('finally'); } }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "try", "finally", "done"],
    );
}

#[test]
fn awaited_finalizer_return_overrides_pending_switch_break() {
    assert_modes(
        r#"
async function run() {
  switch (0) { default: try { await 0; break; } finally { await 0; return 'override'; } }
  print('wrong');
}
run().then(value => print(value), error => print('error:' + error)); print('called');
"#,
        &["called", "override"],
    );
}

#[test]
fn ancestor_label_break_unwinds_caseblock_and_commits_ancestor_exit() {
    assert_modes(
        r#"
async function run() {
  outer: { switch (0) { default: { let value = 2; await 0; print(value); break outer; } } print('wrong'); }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "2", "done"],
    );
}

#[test]
fn labelled_switch_break_from_awaited_finally_keeps_its_owner_exit() {
    assert_modes(
        r#"
async function run() {
  chosen: switch (0) { default: try { await 0; break chosen; } finally { await 0; print('finally'); } }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "finally", "done"],
    );
}

#[test]
fn awaited_while_child_preserves_its_continue_reset_and_switch_fallthrough() {
    assert_modes(
        r#"
async function run() {
  switch (0) { default:
    let count = 0;
    while (await (++count < 4)) { if (count === 1) continue; print(count); break; }
    await 0; print('case');
  }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "2", "case", "done"],
    );
}

#[test]
fn nested_switch_break_does_not_skip_outer_case_continuation() {
    assert_modes(
        r#"
async function run() {
  switch (0) { default:
    switch (1) { default: await 0; print('inner'); break; }
    await 0; print('outer');
  }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "inner", "outer", "done"],
    );
}

#[test]
fn rejection_retains_identity_and_runs_finally_before_following_await() {
    assert_modes(
        r#"
async function run() {
  const marker = {};
  try { switch (0) { default: await Promise.reject(marker); print('wrong'); } }
  catch (error) { print(error === marker); }
  finally { await 0; print('finally'); }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "true", "finally", "done"],
    );
}

#[test]
fn unvisited_case_lexical_remains_tdz_in_a_retained_closure() {
    assert_modes(
        r#"
async function run() {
  let read;
  switch (0) { case 0: read = () => later; await 0; break; case 1: let later = 9; }
  try { read(); } catch (error) { print(error instanceof ReferenceError); }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "true", "done"],
    );
}

#[test]
fn nested_sync_using_disposes_once_after_await_before_switch_break_exit() {
    assert_modes(
        r#"
async function run() {
  switch (0) { default: {
    using resource = { [Symbol.dispose]() { print('dispose'); } };
    await 0; print('body'); break;
  } }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "body", "dispose", "done"],
    );
}

#[test]
fn eager_switch_keeps_existing_synchronous_loop_behavior() {
    assert_modes(
        r#"
async function run() {
  let index = 0;
  while (index < 2) { switch (index) { default: print(index); break; } ++index; }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["0", "1", "called", "done"],
    );
}

#[test]
fn nested_function_awaits_have_their_own_activation_inside_a_resumable_case() {
    assert_modes(
        r#"
async function run() {
  switch (0) { default:
    const helper = async () => { await 0; print('inner'); };
    await 0; print('outer'); await helper(); break;
  }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "outer", "inner", "done"],
    );
}

#[test]
fn branch_sensitive_selector_implicit_suspension_and_ancestor_loop_shapes_are_typed_refusals() {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for source in [
        "async function run(flag) { switch (0) { case flag[await 0] &&= await 0: break; } } run();",
        "async function run() { switch (0) { default: { await using resource = null; break; } } } run();",
        "async function run() { switch (0) { default: for await (const value of []) {} } } run();",
        "async function run() { outer: while (true) { switch (0) { default: await 0; continue outer; } } } run();",
        "async function run() { while (true) { switch (0) { default: try { break; } finally {} } } } run();",
    ] {
        for prefix in ["", "\"use strict\";\n"] {
            let engine=Engine::new(RealmBuilder::new().build());
            let unit=engine.compile_script(&format!("{prefix}{source}"),CompileOptions::default()).expect("complete source owns selectors, implicit phases and ancestor loops");
            let artifact=engine.emit_wasm(&unit).expect("real JS -> Wasm owner dispatch");
            assert!(artifact.bytes.starts_with(b"\0asm"));
        }
    }
}

#[test]
fn awaited_selectors_stop_at_first_match_and_do_not_run_fallthrough_selectors() {
    assert_modes(
        r#"
async function run() {
  switch ((print('discriminant'), 2)) {
    case await (print('test0'), 0): print('wrong0'); break;
    default: print('wrongdefault'); break;
    case await (print('test2'), 2): print('body2'); await 0; print('resumed2');
    case (print('wrongtest3'), 3): print('body3'); break;
  }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &[
            "discriminant",
            "test0",
            "called",
            "test2",
            "body2",
            "resumed2",
            "body3",
            "done",
        ],
    );
}

#[test]
fn awaited_selection_default_and_no_match_have_distinct_body_entries() {
    assert_modes(
        r#"
async function run() {
  switch (9) {
    case await (print('test0'), 0): print('wrong0'); break;
    default: print('default'); await 0;
    case await (print('test2'), 2): print('tail'); break;
  }
  switch (8) {
    case await (print('no-match0'), 0): print('wrong body0'); break;
    case await (print('no-match1'), 1): print('wrong body1'); break;
  }
  print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &[
            "test0",
            "called",
            "test2",
            "default",
            "tail",
            "no-match0",
            "no-match1",
            "done",
        ],
    );
}

#[test]
fn selection_keeps_original_discriminant_identity_across_operand_mutation() {
    assert_modes(
        r#"
let selected = {};
const original = selected;
let reads = 0;
const source = { get value() { reads++; return selected; } };
async function run() {
  switch (source.value) {
    case await (selected = {}, original): print('kept original'); break;
    default: print('wrong re-read');
  }
  print(reads); print(selected === original);
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "kept original", "1", "false"],
    );
}

#[test]
fn selector_await_keeps_the_caseblock_environment_and_outer_discriminant_scope() {
    assert_modes(
        r#"
let label = 'outer';
let saved;
async function run() {
  switch (await (print(label), 1)) {
    case await (saved = () => label, 1):
      let label = 'case'; print(saved()); await 0; print(saved()); break;
    default: print('wrongdefault');
  }
  try {
    switch (0) { case (await 0, hidden): break; default: let hidden = 0; }
  } catch (error) { print(error instanceof ReferenceError); }
  print(label);
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["outer", "called", "case", "case", "true", "outer"],
    );
}

#[test]
fn selector_rejection_and_labelled_break_keep_nested_finally_completion() {
    assert_modes(
        r#"
const marker = {};
async function run() {
  try {
    switch (1) { case await Promise.reject(marker): print('wrong'); break; }
  } catch (error) { print(error === marker); }
  finally { await 0; print('rejection finally'); }
  chosen: switch (1) {
    case await 1:
      try { try { print('body'); break chosen; }
            finally { await 0; print('inner finally'); } }
      finally { await 0; print('outer finally'); }
    default: print('wrong default');
  }
  await 0; print('done');
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &[
            "called",
            "true",
            "rejection finally",
            "body",
            "inner finally",
            "outer finally",
            "done",
        ],
    );
}

#[test]
fn selector_call_gets_and_holds_its_method_receiver_before_argument_await() {
    assert_modes(
        r#"
const receiver = {
  get method() {
    print('get method');
    return function (value) { print(this === receiver); return value; };
  }
};
async function run() {
  switch (2) {
    case receiver.method(await (print('argument'), 2)): print('selected'); break;
    default: print('wrong default');
  }
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["get method", "argument", "called", "true", "selected"],
    );
}

#[test]
fn failed_selector_writes_supply_the_next_selectors_value_facts() {
    assert_modes(
        r#"
async function run() {
  var x = 1;
  switch (3) {
    case (x = 'abc', await 0): print('wrong first'); break;
    case x.length: print('selected'); break;
    default: print('wrong default');
  }
  print(x.length);
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "selected", "3"],
    );
}

#[test]
fn default_and_no_match_observe_writes_from_all_failed_selectors() {
    assert_modes(
        r#"
async function run() {
  var x = 1;
  switch (99) {
    case await 0: print('wrong first'); break;
    default: print(x.length); break;
    case (x = 'abcd', await 1): print('wrong last');
  }
  var y = 1;
  switch (99) {
    case (y = 'after', await 0): print('wrong no-match'); break;
  }
  print(y.length);
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "4", "5"],
    );
}

#[test]
fn case_body_facts_join_direct_selection_and_awaited_fallthrough() {
    assert_modes(
        r#"
async function run() {
  var x = 1;
  switch (0) {
    case await 0: x = 'abc'; await 0;
    case 1: print(x.length); break;
  }
  x = 1;
  switch (1) {
    case await 0: x = 'unvisited';
    case 1: print(x + 1); break;
  }
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "3", "2"],
    );
}

#[test]
fn lexical_value_facts_do_not_leak_from_skipped_selectors_or_bodies() {
    assert_modes(
        r#"
async function run() {
  let x = 1;
  switch (0) {
    case await 0: print(typeof x); print(x); break;
    case (x = 'unvisited', await 1): print('wrong selector'); break;
  }
  x = 1;
  switch (0) {
    case await 0: x = 'abc'; await 0;
    case 1: print(x.length); break;
  }
  x = 1;
  switch (99) {
    case await 0: x = 'unvisited body'; break;
  }
  print(typeof x); print(x);
}
run().catch(error => print('error:' + error)); print('called');
"#,
        &["called", "number", "1", "3", "number", "1"],
    );
}

#[test]
fn ternary_and_logical_selector_prefixes_own_only_the_reached_branches() {
    assert_modes(
        r#"
const trace = [];
function record(name, value) { trace.push(name); return value; }
function skipped() { throw 'unreached selector arm'; }
async function run() {
  switch (record('discriminant', 2)) {
    case record('and-left', 0) && await skipped(): throw 'wrong first body';
    default: throw 'default must wait';
    case record('ternary-test', true) ? await record('ternary-value', 1) : await skipped():
      throw 'wrong second body';
    case record('nullish-left', null) ?? (record('or-left', false) || await record('match-value', 2)):
      trace.push('matched'); await 0; trace.push('resumed');
    case await skipped(): trace.push('fallthrough'); break;
  }
  if (trace.join(',') !== 'discriminant,and-left,ternary-test,ternary-value,called,nullish-left,or-left,match-value,matched,resumed,fallthrough')
    throw 'selector prefix order';
  print('selectors:ok');
}
run().catch(error => print('unexpected:' + error)); trace.push('called');
"#,
        &["selectors:ok"],
    );
}

#[test]
fn logical_assignment_selectors_get_once_and_put_before_matching() {
    assert_modes(
        r#"
const trace = [];
async function run() {
  let stored = 0;
  const target = { get value() { trace.push('get'); return stored; }, set value(v) { trace.push('set'); stored = v; } };
  switch (target.value ||= await 2) { case 2: trace.push('body'); break; default: throw 'wrong discriminant'; }
  let selector = 0;
  switch (3) { case selector ||= await 3: trace.push('case'); break; case (trace.push('wrong later selector'), 4): throw 'wrong match'; }
  if (stored !== 2 || selector !== 3 || trace.join(',') !== 'get,caller,set,body,case') throw 'logical selector Reference/order';
  print('logical-selectors:ok');
}
run().catch(error => print('error:' + error)); trace.push('caller');
"#,
        &["logical-selectors:ok"],
    );
}
