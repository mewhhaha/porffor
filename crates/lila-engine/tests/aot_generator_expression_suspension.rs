//! Synchronous generator `yield`s nested inside expressions, destructuring
//! assignment patterns, return operands and declarations, compiled to Wasm and
//! executed through Wasmtime. Each case prints an exact trace of the
//! specification's evaluation order across the suspension.

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
        .expect("generator suspension regression must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "source:\n{source}\ncompletion: {:?}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

/// An iterable whose iterator logs every protocol call into `log`.
const LOGGING_ITERABLE: &str = r#"
var log = [];
function iterable(values, returnBehaviour) {
  var result = {};
  result[Symbol.iterator] = function () {
    log.push("get-iterator");
    var index = 0;
    return {
      next: function () {
        log.push("next");
        return index < values.length
          ? { done: false, value: values[index++] }
          : { done: true, value: undefined };
      },
      return: function () {
        log.push("return");
        return returnBehaviour();
      }
    };
  };
  return result;
}
"#;

#[test]
fn array_element_target_reference_suspends_before_the_iterator_step() {
    assert_aot_trace(
        &format!(
            r#"{LOGGING_ITERABLE}
var x = {{}};
var vals = iterable([33], function () {{ return {{}}; }});
var result;
function* g() {{
  result = [ x[(log.push("key"), yield "k")] ] = vals;
  log.push("after:" + (result === vals) + ":" + x.prop);
}}
var it = g();
print(it.next().value);
log.push("resume");
it.next("prop");
print(log.join(","));
"#
        ),
        &["k", "get-iterator,key,resume,next,return,after:true:33"],
    );
}

#[test]
fn array_element_initializer_runs_only_for_undefined_and_resumes_its_value() {
    assert_aot_trace(
        r#"
var x, y, z;
function* g(vals) {
  [ x = yield "x", y = yield "y", z ] = vals;
  print("assigned:" + x + ":" + y + ":" + z);
}
var it = g([undefined, 2, 3]);
print(it.next().value);
var last = it.next(10);
print(last.done);
"#,
        &["x", "assigned:10:2:3", "true"],
    );
}

#[test]
fn return_resumption_closes_the_suspended_pattern_iterator() {
    assert_aot_trace(
        &format!(
            r#"{LOGGING_ITERABLE}
var unreachable = 0;
function* g() {{
  [ {{}}[yield] ] = iterable([1], function () {{ return {{}}; }});
  unreachable += 1;
}}
var it = g();
it.next();
var result = it.return(888);
print(result.value + ":" + result.done + ":" + unreachable);
print(log.join(","));
"#
        ),
        &["888:true:0", "get-iterator,return"],
    );
}

#[test]
fn return_resumption_propagates_an_abrupt_or_non_object_close() {
    assert_aot_trace(
        &format!(
            r#"{LOGGING_ITERABLE}
function* throwing() {{
  [ {{}} = yield ] = iterable([undefined], function () {{ throw new Error("boom"); }});
}}
var first = throwing();
first.next();
try {{ first.return(1); print("no-throw"); }} catch (error) {{ print("caught:" + error.message); }}
function* primitive() {{
  [ {{}} = yield ] = iterable([undefined], function () {{ return null; }});
}}
var second = primitive();
second.next();
try {{ second.return(2); print("no-throw"); }} catch (error) {{ print("type:" + (error instanceof TypeError)); }}
print(log.join(","));
"#
        ),
        &[
            "caught:boom",
            "type:true",
            "get-iterator,next,return,get-iterator,next,return",
        ],
    );
}

#[test]
fn throw_resumption_closes_quietly_and_rethrows_the_original_error() {
    assert_aot_trace(
        &format!(
            r#"{LOGGING_ITERABLE}
function* g() {{
  [ {{}}[yield] ] = iterable([1], function () {{ throw new Error("ignored"); }});
}}
var it = g();
it.next();
try {{ it.throw(new Error("original")); }} catch (error) {{ print(error.message); }}
print(log.join(","));
"#
        ),
        &["original", "get-iterator,return"],
    );
}

#[test]
fn an_exhausted_iterator_is_not_closed_after_a_suspending_default() {
    assert_aot_trace(
        &format!(
            r#"{LOGGING_ITERABLE}
var a, b;
function* g() {{
  [a, b = yield "default"] = iterable([1], function () {{ return {{}}; }});
  log.push("end:" + a + ":" + b);
}}
var it = g();
it.next();
it.next(5);
print(log.join(","));
"#
        ),
        &["get-iterator,next,next,end:1:5"],
    );
}

#[test]
fn rest_and_nested_patterns_resume_inside_the_pattern() {
    assert_aot_trace(
        r#"
var x = {}, y, z;
function* g() {
  [...x[yield "rest-key"]] = [1, 2];
  [[y = yield "nested"]] = [[]];
  [...{ length: z = yield "never" }] = [7];
  print(x.k.join("|") + ":" + y + ":" + z);
}
var it = g();
print(it.next().value);
print(it.next("k").value);
it.next(9);
"#,
        &["rest-key", "nested", "1|2:9:1"],
    );
}

#[test]
fn object_patterns_evaluate_key_target_get_and_initializer_in_order() {
    assert_aot_trace(
        r#"
var log = [];
var obj = {};
var source = { get p() { log.push("get"); return 9; } };
var x, z;
function* g() {
  ({ [(log.push("key"), yield "k")]: obj[(log.push("target"), "q")] } = source);
  ({ x = yield "x" } = {});
  ({ b: [z = yield "z"] } = { b: [] });
  log.push("after:" + obj.q + ":" + x + ":" + z);
}
var it = g();
print(it.next().value);
log.push("resume");
print(it.next("p").value);
print(it.next(2).value);
it.next(3);
print(log.join(","));
"#,
        &["k", "x", "z", "key,resume,target,get,after:9:2:3"],
    );
}

#[test]
fn staged_operands_keep_values_evaluated_before_the_suspension() {
    assert_aot_trace(
        r#"
var n = 1;
var x = 1;
function* g() {
  let sum = n + (yield "sum");
  x += yield "compound";
  print(sum + ":" + x + ":" + typeof (yield "typeof"));
}
var it = g();
print(it.next().value);
n = 100;
print(it.next(2).value);
x = 50;
print(it.next(2).value);
it.next("text");
"#,
        &["sum", "compound", "typeof", "3:3:string"],
    );
}

#[test]
fn private_in_and_computed_object_keys_resume_as_operands() {
    assert_aot_trace(
        r#"
class C {
  #field;
  static *isNameIn() { return #field in (yield); }
}
var first = C.isNameIn();
first.next();
print(first.next(new C()).value);
var second = C.isNameIn();
second.next();
print(second.next({}).value);
function* keys() {
  let o = { [yield 9]: 9, get [yield "getter"]() { return "g"; } };
  print(o.a + ":" + o.b);
}
var it = keys();
print(it.next().value);
print(it.next("a").value);
it.next("b");
"#,
        &["true", "false", "9", "getter", "9:g"],
    );
}

#[test]
fn template_substitutions_convert_before_the_next_substitution_suspends() {
    assert_aot_trace(
        r#"
function* g() { print(`a${yield 1}b${yield 2}c`); }
var it = g();
it.next();
it.next({ toString: function () { print("to-string"); return "X"; } });
it.next("Y");
"#,
        &["to-string", "aXbYc"],
    );
}
