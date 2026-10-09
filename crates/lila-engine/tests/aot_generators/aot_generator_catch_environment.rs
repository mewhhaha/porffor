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
        .expect("generator catch must compile and execute through Wasm AOT");
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
fn a_simple_catch_retains_its_binding_and_reader_identity() {
    assert_trace(
        r#"
function* values() {
  try { throw 2; }
  catch(value) {
    let read=()=>value;
    yield read;
    value=4;
    yield read;
    return read;
  }
}
let iterator=values();
let first=iterator.next().value;
print(first());
let second=iterator.next().value;
print(second===first);
print(first());
let last=iterator.next();
print(last.done && last.value===first);
"#,
        &["2", "true", "4", "true"],
    );
}

#[test]
fn catch_sibling_blocks_resume_their_own_captured_cells() {
    assert_trace(
        r#"
function* values() {
  try { throw [1]; }
  catch([parameter, readParameter=()=>parameter]) {
    { let value=2, read=()=>value; yield read; value=3; }
    { let value=4, read=()=>value; yield read; value=5; }
    parameter=6;
    yield readParameter;
  }
}
let iterator=values();
let first=iterator.next().value;
print(first());
let second=iterator.next().value;
print(first()+':'+second());
let third=iterator.next().value;
print(second()+':'+third());
print(iterator.next().done);
"#,
        &["2", "3:4", "5:6", "true"],
    );
}

#[test]
fn parameter_default_body_and_catch_cells_keep_distinct_owners() {
    assert_trace(
        r#"
function* values(value=2, readParameter=()=>value) {
  var value=10;
  let readBody=()=>value;
  try { throw [3]; }
  catch([caught, readCatch=()=>caught]) {
    yield readParameter()+':'+readBody()+':'+readCatch();
    value=20; caught=4;
    yield readParameter()+':'+readBody()+':'+readCatch();
  }
}
let iterator=values();
print(iterator.next().value);
print(iterator.next().value);
print(iterator.next().done);
"#,
        &["2:10:3", "2:20:4", "true"],
    );
}

#[test]
fn delegated_yields_save_the_catch_environment_chain() {
    assert_trace(
        r#"
let change;
function* values() {
  try { throw [2]; }
  catch([value, read=()=>value]) {
    change=()=>value=4;
    yield* [read(),read()];
    yield read();
  }
}
let iterator=values();
print(iterator.next().value);
change();
print(iterator.next().value);
print(iterator.next().value);
print(iterator.next().done);
"#,
        &["2", "2", "4", "true"],
    );
}

#[test]
fn return_and_throw_restore_captured_finalizer_cells() {
    assert_trace(
        r#"
function* values() {
  try { throw [2]; }
  catch([value, read=()=>value]) {
    yield read;
    value=4;
    yield read;
  } finally {
    let value=7, read=()=>value;
    yield read;
    value=8;
    print('finally:'+read());
  }
}
let returned=values();
print(returned.next().value());
print(returned.return(9).value());
let result=returned.next();
print(result.value+':'+result.done);
let thrown=values(), marker={};
print(thrown.next().value());
print(thrown.throw(marker).value());
try { thrown.next(); } catch(error) { print(error===marker); }
"#,
        &[
            "2",
            "7",
            "finally:8",
            "9:true",
            "2",
            "7",
            "finally:8",
            "true",
        ],
    );
}

#[test]
fn interleaved_generators_keep_separate_catch_environments() {
    assert_trace(
        r#"
function* values(initial) {
  try { throw [initial]; }
  catch([value, read=()=>value]) {
    yield read;
    value++;
    yield read;
  }
}
let first=values(1), second=values(10);
let readFirst=first.next().value, readSecond=second.next().value;
print(readFirst()+':'+readSecond());
second.next();
print(readFirst()+':'+readSecond());
first.next();
print(readFirst()+':'+readSecond());
print(first.next().done && second.next().done);
"#,
        &["1:10", "1:11", "2:11", "true"],
    );
}
