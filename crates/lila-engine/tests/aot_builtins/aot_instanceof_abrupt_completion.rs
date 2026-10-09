use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_script_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
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
        .unwrap_or_else(|error| panic!("instanceof script failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

// InstanceofOperator step 3 calls @@hasInstance. Its abrupt completion must
// reach the enclosing user handler, not return from the current function.
#[test]
fn a_throwing_has_instance_method_is_caught_and_runs_finally() {
    assert_script_true(
        r#"
var marker = {}, trace = '';
var target = { [Symbol.hasInstance]() { trace += 'h'; throw marker; } };
var caught;
try { ({}) instanceof target; trace += 'x'; }
catch (error) { caught = error; trace += 'c'; }
finally { trace += 'f'; }
caught === marker && trace === 'hcf';
"#,
    );
}

// OrdinaryHasInstance step 4 reads C.prototype through a Proxy trap, and step 5
// throws a TypeError for a non-object prototype. Both run after the intrinsic
// Function.prototype[@@hasInstance] call, inside a user function's handler.
#[test]
fn ordinary_has_instance_errors_reach_the_function_handler() {
    assert_script_true(
        r#"
var marker = {};
var proxy = new Proxy(function () {}, {
  get(target, key) { if (key === 'prototype') throw marker; return target[key]; }
});
function nonObjectPrototype() {}
nonObjectPrototype.prototype = 1;
const arrow = () => {};
function check(constructor) {
  try { ({}) instanceof constructor; return 'none'; }
  catch (error) { return error; }
}
var fromProxy = check(proxy);
var fromPrimitive = check(nonObjectPrototype);
var fromArrow = check(arrow);
fromProxy === marker
  && fromPrimitive instanceof TypeError
  && fromArrow instanceof TypeError
  && ((function* () {})() instanceof (function* () {})) === false;
"#,
    );
}
