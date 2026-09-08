use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_generator_trace(source: &str, expected: &[&str]) {
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
        .expect("generator loop must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn generator_loop_lexicals_survive_resume_and_remain_per_activation() {
    assert_generator_trace(
        r#"
function* values() {
  for (let index = 0; index < 4; index++) {
    let value = index * 2;
    yield value;
    print("resumed:" + value);
  }
}
function report(result) { print(result.value + ":" + result.done); }
var left = values();
var right = values();
report(left.next());
report(left.next());
report(right.next());
report(left.next());
report(left.next());
report(left.next());
report(right.next());
"#,
        &[
            "0:false",
            "resumed:0",
            "2:false",
            "0:false",
            "resumed:2",
            "4:false",
            "resumed:4",
            "6:false",
            "resumed:6",
            "undefined:true",
            "resumed:0",
            "2:false",
        ],
    );
}

#[test]
fn conditional_generator_loop_skips_iterations_and_resumes_after_selected_branch_yield() {
    assert_generator_trace(
        r#"
function* oddValues() {
  for (var index = 0; index < 4; index++) {
    let value = index * 10;
    print("before:" + index);
    if (index % 2) {
      yield value;
      print("branch:" + value);
    }
    print("after:" + value);
  }
  yield "tail";
}
function report(result) { print(result.value + ":" + result.done); }
var iterator = oddValues();
report(iterator.next());
report(iterator.next());
report(iterator.next());
report(iterator.next());
"#,
        &[
            "before:0",
            "after:0",
            "before:1",
            "10:false",
            "branch:10",
            "after:10",
            "before:2",
            "after:20",
            "before:3",
            "30:false",
            "branch:30",
            "after:30",
            "tail:false",
            "undefined:true",
        ],
    );
}

#[test]
fn generator_branch_lexicals_keep_their_shadowed_slots_across_resume() {
    assert_generator_trace(
        r#"
function* values(chooseLeft) {
  let value = 99;
  if (chooseLeft) {
    const value = 7;
    yield value;
    print("left:" + value);
  } else {
    let value = 8;
    yield value;
    value++;
    print("right:" + value);
  }
  return value;
}
function report(result) { print(result.value + ":" + result.done); }
var left = values(true);
var right = values(false);
report(left.next());
report(right.next());
report(left.next());
report(right.next());
"#,
        &[
            "7:false", "8:false", "left:7", "99:true", "right:9", "99:true",
        ],
    );
}

#[test]
fn conditional_while_generator_consumes_return_and_throw_before_continuing_iteration() {
    assert_generator_trace(
        r#"
function* values() {
  var index = 0;
  while (index < 4) {
    let value = index++;
    if (value % 2 === 0) { print("skip:" + value); }
    else { yield value; print("resumed:" + value); }
    print("end:" + value);
  }
}
function report(result) { print(result.value + ":" + result.done); }
var returned = values();
report(returned.next());
report(returned.return(91));
report(returned.next());
var thrown = values();
report(thrown.next());
var marker = {};
try { thrown.throw(marker); } catch (error) { print("same:" + (error === marker)); }
report(thrown.next());
"#,
        &[
            "skip:0",
            "end:0",
            "1:false",
            "91:true",
            "undefined:true",
            "skip:0",
            "end:0",
            "1:false",
            "same:true",
            "undefined:true",
        ],
    );
}

#[test]
fn array_from_async_consumes_and_maps_every_synchronous_generator_iteration() {
    assert_generator_trace(
        r#"
function* values() {
  for (let index = 0; index < 4; index++) { yield index * 2; }
}
Array.fromAsync({ [Symbol.iterator]: values }, function (value, index) {
  return value * index;
}).then(function (result) {
  print(result.join(","));
  return Array.fromAsync({ [Symbol.asyncIterator]: values }, function (value, index) {
    return Promise.resolve(value + index);
  });
}).then(function (result) { print(result.join(",")); });
void 0;
"#,
        &["0,2,8,18", "0,3,6,9"],
    );
}

#[test]
fn array_from_async_array_like_path_preserves_try_block_lexicals_across_await() {
    assert_generator_trace(
        r#"
(async function () {
  const iteratorPrototype = Object.getPrototypeOf([].values());
  const originalNext = iteratorPrototype.next;
  try {
    iteratorPrototype.next = function () { throw "array iterator was used"; };
    const expected = [0, 1, 2];
    const input = { length: 3, 0: 0, 1: 1, 2: 2 };
    const output = await Array.fromAsync(input);
    print(output.join(",") + ":" + (output.join(",") === expected.join(",")));
  } finally {
    iteratorPrototype.next = originalNext;
  }
  print("restored:" + (iteratorPrototype.next === originalNext));
})();
void 0;
"#,
        &["0,1,2:true", "restored:true"],
    );
}
