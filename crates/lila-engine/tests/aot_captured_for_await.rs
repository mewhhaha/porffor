use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_aot_trace(source: &str, expected: &[&str]) {
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
        .expect("captured for-await regression must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "completion: {:?}; source:\n{source}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn captured_const_heads_keep_distinct_cells_across_multiple_yields() {
    assert_aot_trace(
        r#"
var readers = [];
async function* stream() {
  for await (const value of [3, 7]) {
    readers.push(function () { return value; });
    yield value * 2;
    print("after:" + value + ":" + readers[readers.length - 1]());
    yield value + 1;
  }
  print("retained:" + readers[0]() + ":" + readers[1]());
}
var iterator = stream();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) { report(result); return iterator.next(); }).then(function (result) { report(result); return iterator.next(); }).then(function (result) { report(result); return iterator.next(); }).then(function (result) { report(result); return iterator.next(); }).then(report);

void 0;
"#,
        &[
            "6:false",
            "after:3:3",
            "4:false",
            "14:false",
            "after:7:7",
            "8:false",
            "retained:3:7",
            "undefined:true",
        ],
    );
}

#[test]
fn closure_writes_and_resumed_body_writes_share_the_same_let_cell() {
    assert_aot_trace(
        r#"
var readers = [];
var read;
var change;
async function* stream() {
  for await (let value of [3, 7]) {
    read = function () { return value; };
    change = function (next) { value = next; };
    readers.push(read);
    yield value;
    print("resumed:" + value + ":" + read());
    value++;
    yield read();
  }
  print("retained:" + readers[0]() + ":" + readers[1]());
}
var iterator = stream();
function report(result) { print(result.value + ":" + result.done); }

iterator.next().then(function (result) {
  report(result); change(30); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); change(70); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(report);

void 0;
"#,
        &[
            "3:false",
            "resumed:30:30",
            "31:false",
            "7:false",
            "resumed:70:70",
            "71:false",
            "retained:31:71",
            "undefined:true",
        ],
    );
}

#[test]
fn yielded_closures_and_shadowed_outer_binding_survive_loop_completion() {
    assert_aot_trace(
        r#"
var first;
var second;
async function* stream() {
  let value = 99;
  for await (const value of [2, 5]) {
    yield function () { return value; };
  }
  yield value;
}
var iterator = stream();
iterator.next().then(function (result) {
  first = result.value; print(first()); return iterator.next();
}).then(function (result) {
  second = result.value; print(second()); return iterator.next();
}).then(function (result) {
  print(result.value + ":" + result.done);
  print("retained:" + first() + ":" + second());
  return iterator.next();
}).then(function (result) { print(result.value + ":" + result.done); });

void 0;
"#,
        &["2", "5", "99:false", "retained:2:5", "undefined:true"],
    );
}

#[test]
fn queued_requests_and_interleaved_activations_keep_separate_environment_chains() {
    assert_aot_trace(
        r#"
var readers = [];
async function* stream(prefix, source) {
  for await (const value of source) {
    readers.push(function () { return prefix + value; });
    yield prefix + value;
  }
  yield prefix + "done";
}
var left = stream("L", [1, 2]);
var right = stream("R", [8]);
function report(result) { print(result.value + ":" + result.done); }

Promise.all([left.next(), left.next(), right.next()]).then(function (results) {
  report(results[0]); report(results[1]); report(results[2]);
  return left.next();
}).then(function (result) {
  report(result); return right.next();
}).then(function (result) {
  report(result);
  print("retained:" + readers[0]() + ":" + readers[1]() + ":" + readers[2]());
});

void 0;
"#,
        &[
            "L1:false",
            "L2:false",
            "R8:false",
            "Ldone:false",
            "Rdone:false",
            "retained:L1:R8:L2",
        ],
    );
}

#[test]
fn break_restores_parent_before_async_iterator_close_and_following_yield() {
    assert_aot_trace(
        r#"
var read;
var calls = 0;
var source = {
  [Symbol.asyncIterator]: function () { return this; },
  next: function () {
    calls++; print("next:" + calls);
    return Promise.resolve({ value: calls, done: false });
  },
  return: function () {
    print("close:" + read()); return Promise.resolve({ done: true });
  }
};
async function* stream() {
  var anchor = 40;
  for await (let value of source) {
    read = function () { return value; };
    yield value;
    value += 10;
    break;
  }
  print("after:" + anchor + ":" + read());
  yield anchor;
}
var iterator = stream();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) { report(result); return iterator.next(); }).then(function (result) { report(result); return iterator.next(); }).then(report);

void 0;
"#,
        &[
            "next:1",
            "1:false",
            "close:11",
            "after:40:11",
            "40:false",
            "undefined:true",
        ],
    );
}

#[test]
fn continue_leaves_one_environment_without_closing_the_iterator() {
    assert_aot_trace(
        r#"
var readers = [];
var calls = 0;
var closes = 0;
var source = {
  [Symbol.asyncIterator]: function () { return this; },
  next: function () { calls++; return Promise.resolve({value: calls, done: calls > 2}); },
  return: function () { closes++; return Promise.resolve({done: true}); }
};
async function* stream() {
  for await (let value of source) {
    readers.push(function () { return value; });
    yield value;
    value += 10;
    continue;
  }
  print("retained:" + readers[0]() + ":" + readers[1]() + ":closes:" + closes);
}
var iterator = stream();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) { report(result); return iterator.next(); }).then(function (result) { report(result); return iterator.next(); }).then(report);

void 0;
"#,
        &[
            "1:false",
            "2:false",
            "retained:11:12:closes:0",
            "undefined:true",
        ],
    );
}

#[test]
fn throw_resumption_restores_parent_and_preserves_throw_over_close_rejection() {
    assert_aot_trace(
        r#"
var marker = {};
var read;
var source = {
  [Symbol.asyncIterator]: function () { return this; },
  next: function () { return Promise.resolve({value: 6, done: false}); },
  return: function () { print("close:" + read()); return Promise.reject("close-error"); }
};
async function* stream() {
  var anchor = 40;
  try {
    for await (const value of source) {
      read = function () { return value; };
      yield value;
      print("unreachable");
    }
  } catch (error) {
    print("caught:" + (error === marker) + ":" + anchor);
    yield anchor;
  }
  print("after:" + anchor + ":" + read());
}
var iterator = stream();
function report(result) { print(result.value + ":" + result.done); }

iterator.next().then(function (result) {
  report(result); return iterator.throw(marker);
}).then(function (result) {
  report(result); return iterator.next();
}).then(report);

void 0;
"#,
        &[
            "6:false",
            "close:6",
            "caught:true:40",
            "40:false",
            "after:40:6",
            "undefined:true",
        ],
    );
}

#[test]
fn return_resumption_closes_once_then_runs_suspending_finally_in_parent() {
    assert_aot_trace(
        r#"
var read;
var source = {
  [Symbol.asyncIterator]: function () { return this; },
  next: function () { return Promise.resolve({value: 6, done: false}); },
  return: function () { print("close:" + read()); return Promise.resolve({done: true}); }
};
async function* stream() {
  var anchor = 40;
  try {
    for await (const value of source) {
      read = function () { return value; };
      yield value;
    }
  } finally {
    print("finally:" + anchor + ":" + read());
    yield anchor;
    print("finished:" + anchor);
  }
}
var iterator = stream();
function report(result) { print(result.value + ":" + result.done); }

iterator.next().then(function (result) {
  report(result); return iterator.return(Promise.resolve(9));
}).then(function (result) {
  report(result); return iterator.next();
}).then(report);

void 0;
"#,
        &[
            "6:false",
            "close:6",
            "finally:40:6",
            "40:false",
            "finished:40",
            "9:true",
        ],
    );
}

#[test]
fn rejected_yield_closes_the_current_iteration_before_catch_suspends() {
    assert_aot_trace(
        r#"
var marker = {};
var read;
var source = {
  [Symbol.asyncIterator]: function () { return this; },
  next: function () { return Promise.resolve({value: 8, done: false}); },
  return: function () { print("close:" + read()); return Promise.resolve({done: true}); }
};
async function* stream() {
  var anchor = 40;
  try {
    for await (const value of source) {
      read = function () { return value; };
      yield Promise.reject(marker);
    }
  } catch (error) {
    print("caught:" + (error === marker) + ":" + read());
    yield anchor;
  }
  print("after:" + anchor);
}
var iterator = stream();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) { report(result); return iterator.next(); }).then(report);

void 0;
"#,
        &[
            "close:8",
            "caught:true:8",
            "40:false",
            "after:40",
            "undefined:true",
        ],
    );
}

#[test]
fn synchronous_body_throw_before_first_yield_leaves_the_new_cell_once() {
    assert_aot_trace(
        r#"
var marker = {};
var read;
var source = {
  [Symbol.asyncIterator]: function () { return this; },
  next: function () { return Promise.resolve({value: 8, done: false}); },
  return: function () { print("close:" + read()); return Promise.resolve({done: true}); }
};
async function* stream() {
  var anchor = 40;
  try {
    for await (const value of source) {
      read = function () { return value; };
      throw marker;
      yield value;
    }
  } catch (error) {
    print("caught:" + (error === marker) + ":" + anchor);
    yield anchor;
  }
  print("after:" + anchor + ":" + read());
}
var iterator = stream();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) { report(result); return iterator.next(); }).then(report);

void 0;
"#,
        &[
            "close:8",
            "caught:true:40",
            "40:false",
            "after:40:8",
            "undefined:true",
        ],
    );
}
