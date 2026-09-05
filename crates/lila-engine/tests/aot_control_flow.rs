use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("control-flow regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn throwing_getter_inside_a_loop_and_switch_reaches_the_outer_handler() {
    assert_wasm_true(
        r#"
var marker = {};
var visits = 0;
var caught = 0;
var object = { get value() { throw marker; } };
try {
  for (var i = 0; i < 3; i++) {
    visits++;
    switch (i) {
      case 0:
        if (visits === 1) object.value;
        break;
      default:
        visits += 100;
    }
  }
} catch (error) {
  if (error === marker) caught++;
}
visits === 1 && caught === 1;
"#,
    );
}

#[test]
fn labeled_continue_and_break_run_finally_once_per_exit() {
    assert_wasm_true(
        r#"
var visits = 0;
var finalizers = 0;
outer: for (var i = 0; i < 3; i++) {
  for (var j = 0; j < 3; j++) {
    try {
      visits++;
      if (i === 0) continue outer;
      break outer;
    } finally {
      finalizers++;
    }
  }
}
visits === 2 && finalizers === 2 && i === 1;
"#,
    );
}

#[test]
fn finally_can_replace_return_or_throw_completion() {
    assert_wasm_true(
        r#"
var marker = {};
var count = 0;
function replaceReturn() {
  try { return 1; }
  finally { count++; return 2; }
}
function replaceThrow() {
  try { throw marker; }
  finally { count++; return 3; }
}
function replaceWithThrow() {
  try { return 4; }
  finally { count++; throw marker; }
}
var first = replaceReturn();
var second = replaceThrow();
var caught = false;
try { replaceWithThrow(); }
catch (error) { caught = error === marker; }
first === 2 && second === 3 && caught && count === 3;
"#,
    );
}

#[test]
fn throwing_argument_stops_later_arguments_and_the_call() {
    assert_wasm_true(
        r#"
var marker = {};
var order = 0;
function first() { order = order * 10 + 1; return 1; }
function second() { order = order * 10 + 2; throw marker; }
function third() { order = order * 10 + 3; return 3; }
function target(a, b, c) { order = 999; }
var caught = false;
try {
  for (var i = 0; i < 2; i++) {
    if (i === 0) target(first(), second(), third());
  }
} catch (error) {
  caught = error === marker;
}
caught && order === 12;
"#,
    );
}

#[test]
fn sibling_control_regions_keep_their_own_exit_targets() {
    assert_wasm_true(
        r#"
var marker = {};
var score = 0;
first: {
  for (var i = 0; i < 2; i++) {
    if (i === 0) { score += 1; break first; }
  }
  score += 1000;
}
second: {
  try {
    for (var j = 0; j < 2; j++) {
      switch (j) {
        case 0: score += 2; throw marker;
        default: score += 1000;
      }
    }
  } catch (error) {
    if (error === marker) { score += 4; break second; }
  } finally {
    score += 8;
  }
  score += 1000;
}
score === 15;
"#,
    );
}
