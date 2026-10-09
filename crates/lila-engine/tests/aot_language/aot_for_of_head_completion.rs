use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_completion(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("for-of failed: {error}\n{source}"));
    assert!(
        outcome.note.contains(expected),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn identifier_assignment_heads_preserve_empty_body_completion() {
    assert_completion("var value; for (value of [1, 2]) {}", "undefined");
}

#[test]
fn property_assignment_heads_preserve_empty_body_completion() {
    assert_completion(
        "var target = {}; for (target.value of [1, 2]) {}",
        "undefined",
    );
}

#[test]
fn destructuring_assignment_heads_preserve_empty_body_completion() {
    assert_completion("var value; for ([value] of [[1], [2]]) {}", "undefined");
    assert_completion("var value; for ({value} of [{value: 1}]) {}", "undefined");
}

#[test]
fn if_continue_replaces_the_previous_iteration_value_with_undefined() {
    assert_completion(
        "var value; for (value of [1, 2]) { if (value === 2) continue; 23; }",
        "undefined",
    );
    assert_completion(
        "var target = {}; for (target.value of [1, 2]) { if (target.value === 2) continue; 23; }",
        "undefined",
    );
}

#[test]
fn statement_values_flow_into_bare_continue_completion() {
    assert_completion(
        "var value; for (value of [1, 2]) { 23; continue; }",
        "number(23)",
    );
    assert_completion(
        "var target = {}; for (target.value of [1, 2]) { 23; continue; }",
        "number(23)",
    );
}

#[test]
fn abrupt_assignment_heads_close_the_iterator_before_the_body() {
    assert_completion(
        r#"
var marker = {};
var closed = 0;
var bodies = 0;
var target = { set value(value) { throw marker; } };
var iterable = {
  [Symbol.iterator]() { return this; },
  next() { return { value: 1, done: false }; },
  return() { closed++; return {}; }
};
var caught;
try { for (target.value of iterable) bodies++; }
catch (error) { caught = error; }
caught === marker && closed === 1 && bodies === 0;
"#,
        "boolean(true)",
    );
}
