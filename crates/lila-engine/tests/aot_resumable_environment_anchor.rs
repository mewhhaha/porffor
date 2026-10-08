use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("resumable environments compile and execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}; output: {:?}",
            observed.completion,
            observed.output_events
        );
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
fn simple_parameter_generators_reattach_original_nested_cells_to_their_own_invocation() {
    assert_modes(
        r#"
function make(seed) {
  let outer = seed;
  return function* values(initial) {
    const readOuter = () => outer;
    {
      let value = initial;
      const read = () => value;
      yield read;
      gc();
      value = value + outer;
      yield read;
    }
    return readOuter;
  };
}
const first = make(10)(1), second = make(100)(2);
const readFirst = first.next().value, readSecond = second.next().value;
gc();
print(readFirst() + ':' + readSecond());
print(first.next().value === readFirst);
print(readFirst() + ':' + readSecond());
print(second.next().value === readSecond);
print(readFirst() + ':' + readSecond());
const endFirst = first.next(), endSecond = second.next();
print(endFirst.done && endSecond.done);
print(endFirst.value() + ':' + endSecond.value());
"#,
        &["1:2", "true", "11:2", "true", "11:102", "true", "10:100"],
    );
}

#[test]
fn parameterless_generators_restore_catch_and_finalizer_chains_for_injected_whole_completions() {
    assert_modes(
        r#"
function make(seed) {
  let outer = seed;
  return function* values() {
    try { throw {value: outer}; }
    catch (caught) {
      const read = () => caught;
      yield read;
      yield read;
    } finally {
      let marker = {value: outer};
      marker.self = marker;
      const read = () => marker;
      yield read;
      gc();
      marker.value = marker.value + 1;
      yield read;
    }
  };
}
const returned = make(7)(), returnedToken = {kind: 'return'};
print(returned.next().value().value);
const readReturnFinally = returned.return(returnedToken).value;
gc();
print(readReturnFinally().value);
print(returned.next().value === readReturnFinally);
print(readReturnFinally().value + ':' + (readReturnFinally().self === readReturnFinally()));
const returnEnd = returned.next();
print(returnEnd.done && returnEnd.value === returnedToken);
const thrown = make(20)(), thrownToken = {kind: 'throw'};
print(thrown.next().value().value);
const readThrowFinally = thrown.throw(thrownToken).value;
gc();
print(readThrowFinally().value);
print(thrown.next().value === readThrowFinally);
print(readThrowFinally().value + ':' + (readThrowFinally().self === readThrowFinally()));
try { thrown.next(); } catch (error) { print(error === thrownToken); }
"#,
        &[
            "7", "7", "true", "8:true", "true", "20", "20", "true", "21:true", "true",
        ],
    );
}

#[test]
fn plain_async_functions_keep_the_defining_parent_and_nested_cells_across_await() {
    assert_modes(
        r#"
function make(seed) {
  let outer = seed;
  return async function run(initial) {
    {
      let inner = initial;
      const read = () => outer + inner;
      await 0;
      gc();
      outer = outer + 10;
      inner = inner + 20;
      const identity = read;
      await 0;
      print(read === identity);
      print(read());
      return read;
    }
  };
}
make(2)(3).then(read => print(read()), error => print('wrong:' + error));
print('called');
"#,
        &["called", "true", "35", "35"],
    );
}
