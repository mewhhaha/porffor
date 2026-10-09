use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("async conditional must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}\noutput: {:?}\nsource:\n{source}",
        outcome.completion,
        outcome.output_events
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
            .collect::<Vec<_>>(),
        "source:\n{source}"
    );
}

#[test]
fn both_branches_resume_without_repeating_the_condition() {
    assert_trace(
        r#"
async function choose(flag) {
  let checks=0;
  function condition() { checks++; return flag; }
  if(condition()) {
    let value=await (flag=false, 1);
    return value+1+':'+checks;
  } else {
    let value=await (flag=true, 4);
    return value+1+':'+checks;
  }
}
async function run() {
  print(await choose(true));
  print(await choose(false));
}
run().catch(error=>print('unexpected:'+error));
"#,
        &["2:1", "5:1"],
    );
}

#[test]
fn constant_number_conditions_retain_their_selected_async_branch() {
    assert_trace(
        r#"
async function taken() {
  if((1<<0)===1) { let value=await 1; return value+1; }
  else { let value=await 4; return value+1; }
}
async function untaken() {
  if((1<<0)!==1) { let value=await 1; return value+1; }
  else { let value=await 4; return value+1; }
}
async function run() { print(await taken()); print(await untaken()); }
run().catch(error=>print('unexpected:'+error));
"#,
        &["2", "5"],
    );
}

#[test]
fn empty_and_non_suspending_branches_join_before_later_awaits() {
    assert_trace(
        r#"
async function empty(flag) {
  if(flag) {} else await 4;
  return await 9;
}
async function missing(flag) {
  if(flag) await 4;
  return await 7;
}
async function eager(flag) {
  if(flag) return 2;
  else await 4;
  return await 5;
}
async function run() {
  print(await empty(true)); print(await empty(false));
  print(await missing(true)); print(await missing(false));
  print(await eager(true)); print(await eager(false));
}
run().catch(error=>print('unexpected:'+error));
"#,
        &["9", "9", "7", "7", "2", "5"],
    );
}

#[test]
fn condition_awaits_run_once_before_the_selected_branch() {
    assert_trace(
        r#"
async function choose(flag) {
  let trace='';
  function condition() { trace+='C'; return flag; }
  await (trace+='P');
  if(await condition()) { await (trace+='T'); }
  else { await (trace+='E'); }
  await (trace+='J');
  return trace;
}
async function run() { print(await choose(true)); print(await choose(false)); }
run().catch(error=>print('unexpected:'+error));
"#,
        &["PCTJ", "PCEJ"],
    );
}

#[test]
fn nested_conditions_and_awaits_keep_each_branches_effect_order() {
    assert_trace(
        r#"
async function choose(first, second) {
  let trace='P';
  await 0;
  if(first) {
    trace+='A'; await 0;
    if(second) { trace+='B'; await 0; trace+='b'; }
    else { trace+='C'; await 0; trace+='c'; }
    await 0; trace+='a';
  } else { trace+='D'; await 0; trace+='d'; }
  trace+='J'; await 0; return trace;
}
async function run() {
  print(await choose(true,true));
  print(await choose(true,false));
  print(await choose(false,true));
}
run().catch(error=>print('unexpected:'+error));
"#,
        &["PABbaJ", "PACcaJ", "PDdJ"],
    );
}

#[test]
fn captured_branch_cells_keep_their_identity_and_invocation() {
    assert_trace(
        r#"
async function choose(flag) {
  if(flag) {
    let value=1;
    let read=()=>value;
    await 0; value=2;
    return {read, increment(){value++;}};
  } else {
    let value=3;
    let read=()=>value;
    await 0; value=4;
    return {read, increment(){value++;}};
  }
}
async function run() {
  let first=await choose(true), second=await choose(false);
  print(first.read()+':'+second.read());
  first.increment(); second.increment(); second.increment();
  print(first.read()+':'+second.read());
}
run().catch(error=>print('unexpected:'+error));
"#,
        &["2:4", "3:6"],
    );
}

#[test]
fn nested_captured_blocks_restore_the_saved_environment_chain() {
    assert_trace(
        r#"
async function choose(flag, initial=10) {
  let readInner;
  if(flag) {
    let outer=initial, readOuter=()=>outer;
    await 0;
    {
      let inner=1;
      readInner=()=>inner+outer;
      await 0; inner=2; outer=20;
    }
    await 0; outer=30;
    return readOuter()+':'+readInner();
  } else {
    let outer=initial, readOuter=()=>outer;
    {
      let inner=4;
      readInner=()=>inner+outer;
      await 0; inner=5; outer=40;
    }
    await 0; outer=50;
    return readOuter()+':'+readInner();
  }
}
async function run() { print(await choose(true)); print(await choose(false)); }
run().catch(error=>print('unexpected:'+error));
"#,
        &["30:32", "50:55"],
    );
}

#[test]
fn branch_return_throw_and_await_rejection_run_finally_once() {
    assert_trace(
        r#"
let marker={};
async function choose(flag) {
  try {
    if(flag) { await 0; return 2; }
    else { await 0; throw marker; }
  } finally { print('finally:'+flag); await 0; }
}
async function rejected(flag) {
  try {
    if(flag) await Promise.reject(marker);
    else await Promise.reject(marker);
    print('unreachable');
  } finally { print('rejection-finally:'+flag); }
}
async function run() {
  print(await choose(true));
  try { await choose(false); } catch(error) { print(error===marker); }
  try { await rejected(true); } catch(error) { print(error===marker); }
  try { await rejected(false); } catch(error) { print(error===marker); }
}
run().catch(error=>print('unexpected:'+error));
"#,
        &[
            "finally:true",
            "2",
            "finally:false",
            "true",
            "rejection-finally:true",
            "true",
            "rejection-finally:false",
            "true",
        ],
    );
}

#[test]
fn labels_inside_resumed_branches_target_the_same_control_frame() {
    assert_trace(
        r#"
async function choose(flag) {
  if(flag) {
    await 0;
    exit: { if(flag) break exit; throw 'missed then label'; }
    return 2;
  } else {
    await 0;
    exit: { if(!flag) break exit; throw 'missed else label'; }
    return 4;
  }
}
async function run() { print(await choose(true)); print(await choose(false)); }
run().catch(error=>print('unexpected:'+error));
"#,
        &["2", "4"],
    );
}

#[test]
fn independently_suspended_invocations_keep_their_parameter_and_branch_cells() {
    assert_trace(
        r#"
let firstRelease, secondRelease;
let firstReady=new Promise(resolve=>firstRelease=resolve);
let secondReady=new Promise(resolve=>secondRelease=resolve);
async function choose(flag, ready) {
  if(flag) {
    let value=1, read=()=>flag+':'+value;
    await ready; value=2; return read();
  } else {
    let value=3, read=()=>flag+':'+value;
    await ready; value=4; return read();
  }
}
async function run() {
  let first=choose(true,firstReady), second=choose(false,secondReady);
  secondRelease(); print(await second);
  firstRelease(); print(await first);
}
run().catch(error=>print('unexpected:'+error));
"#,
        &["false:4", "true:2"],
    );
}

#[test]
fn branch_continuations_compose_with_captured_for_of_iteration_cells() {
    assert_trace(
        r#"
async function choose(flag, extra=1) {
  if(flag) {
    const readers=[];
    for(let value of [3,7]) {
      readers.push(()=>value);
      await 0;
    }
    await 0;
    return readers[0]()+readers[1]()+extra;
  } else { await 0; return extra; }
}
async function run() { print(await choose(true)); print(await choose(false)); }
run().catch(error=>print('unexpected:'+error));
"#,
        &["11", "1"],
    );
}

#[test]
fn sibling_blocks_restore_only_their_active_captured_cells() {
    assert_trace(
        r#"
async function choose(flag) {
  let reads=[];
  if(flag) {
    { let value=1; reads.push(()=>value); await 0; value=2; }
    { let value=3; reads.push(()=>value); await 0; value=4; }
  } else {
    { let value=5; reads.push(()=>value); await 0; value=6; }
    { let value=7; reads.push(()=>value); await 0; value=8; }
  }
  await 0;
  return reads[0]()+':'+reads[1]();
}
async function run() { print(await choose(true)); print(await choose(false)); }
run().catch(error=>print('unexpected:'+error));
"#,
        &["2:4", "6:8"],
    );
}

#[test]
fn resumed_catches_keep_simple_and_pattern_parameter_cells() {
    assert_trace(
        r#"
async function pattern(flag) {
  let outside=9;
  try { throw [2]; }
  catch([value, read=()=>value, readOutside=()=>outside]) {
    let outside=100;
    if(flag) { await 0; value=4; }
    else { await 0; value=5; }
    return read()+readOutside();
  }
}
async function simple(flag) {
  try { throw 2; }
  catch(value) {
    let read=()=>value;
    if(flag) { await 0; value=4; }
    else { await 0; value=5; }
    await 0; return read;
  } finally { await 0; print('finally:'+flag); }
}
async function run() {
  print(await pattern(true)); print(await pattern(false));
  let first=await simple(true), second=await simple(false);
  print(first()+':'+second());
}
run().catch(error=>print('unexpected:'+error));
"#,
        &["13", "14", "finally:true", "finally:false", "4:5"],
    );
}

#[test]
fn parameter_default_closures_and_body_variables_keep_separate_cells() {
    assert_trace(
        r#"
async function choose(flag, value=2, readParameter=()=>value) {
  var value=10;
  let readBody=()=>value;
  if(flag) { await 0; value=20; }
  else { await 0; value=30; }
  await 0;
  return readParameter()+':'+readBody();
}
async function run() { print(await choose(true)); print(await choose(false)); }
run().catch(error=>print('unexpected:'+error));
"#,
        &["2:20", "2:30"],
    );
}

#[test]
fn reentrant_async_calls_keep_their_invocation_roots() {
    assert_trace(
        r#"
async function choose(depth) {
  let readDepth=()=>depth;
  if(depth) {
    let value=depth, readValue=()=>value;
    value+=await choose(depth-1);
    await 0;
    return readDepth()+readValue();
  } else {
    let value=1, readValue=()=>value;
    await 0; value=2;
    return readDepth()+readValue();
  }
}
choose(2).then(print, error=>print('unexpected:'+error));
"#,
        &["8"],
    );
}
