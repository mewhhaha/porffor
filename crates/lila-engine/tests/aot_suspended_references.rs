use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_aot_completion_trace(source: &str, expected: &[&str]) {
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
        .expect("completion regression must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn aot_completion_yield_assignment_pins_raw_reference_and_defers_key_coercion() {
    assert_aot_completion_trace(
        r#"
var target = {};
var original = target;
var key = {
  toString: function () { print("key-convert"); return "value"; }
};
function base() { print("base"); return target; }
function property() { print("key-evaluate"); return key; }
function right() { print("rhs"); return 5; }
async function* assign() {
  base()[property()] = yield right();
  print("assigned:" + original.value + ":" + target.value);
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result);
  target = {};
  key = "wrong";
  print("resume");
  return iterator.next(42);
}).then(report);
void 0;
"#,
        &[
            "base",
            "key-evaluate",
            "rhs",
            "5:false",
            "resume",
            "key-convert",
            "assigned:42:undefined",
            "undefined:true",
        ],
    );
}

#[test]
fn aot_completion_throw_resumption_does_not_commit_the_property_write() {
    assert_aot_completion_trace(
        r#"
var marker = {};
var writes = 0;
var target = { set value(value) { writes++; } };
var key = { toString: function () { print("unexpected-key-coercion"); return "value"; } };
async function* assign() {
  try {
    target[key] = yield 1;
    print("unreachable");
  } catch (error) {
    print("caught:" + (error === marker));
  } finally {
    print("finally");
  }
  print("writes:" + writes);
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result);
  return iterator.throw(marker);
}).then(report);
void 0;
"#,
        &[
            "1:false",
            "caught:true",
            "finally",
            "writes:0",
            "undefined:true",
        ],
    );
}

#[test]
fn aot_completion_resumed_property_write_preserves_strictness() {
    assert_aot_completion_trace(
        r#"
var target = {};
Object.defineProperty(target, "value", { value: 1, writable: false });
async function* assign() {
  "use strict";
  try {
    target.value = yield 1;
  } catch (error) {
    print("type:" + (error instanceof TypeError));
  }
  print("value:" + target.value);
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result);
  return iterator.next(7);
}).then(report);
void 0;
"#,
        &["1:false", "type:true", "value:1", "undefined:true"],
    );
}

#[test]
fn aot_completion_delegated_yield_assignment_uses_the_terminal_value() {
    assert_aot_completion_trace(
        r#"
var target = {};
async function* source() {
  yield 2;
  return 9;
}
async function* assign() {
  target.value = yield* source();
  print("assigned:" + target.value);
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result);
  print("before-done:" + target.value);
  return iterator.next(41);
}).then(report);
void 0;
"#,
        &[
            "2:false",
            "before-done:undefined",
            "assigned:9",
            "undefined:true",
        ],
    );
}

#[test]
fn aot_completion_return_resumption_skips_key_coercion_and_property_write() {
    assert_aot_completion_trace(
        r#"
var writes = 0;
var target = { set value(value) { writes++; } };
var key = { toString: function () { print("unexpected-key-coercion"); return "value"; } };
async function* assign() {
  try {
    target[key] = yield 1;
    print("unreachable");
  } finally {
    print("finally:" + writes);
  }
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result);
  return iterator.return(99);
}).then(function (result) {
  report(result);
  print("writes:" + writes);
  return iterator.next();
}).then(report);
void 0;
"#,
        &[
            "1:false",
            "finally:0",
            "99:true",
            "writes:0",
            "undefined:true",
        ],
    );
}

#[test]
fn aot_completion_rejected_yield_value_skips_key_coercion_and_property_write() {
    assert_aot_completion_trace(
        r#"
var marker = {};
var writes = 0;
var target = { set value(value) { writes++; } };
var key = { toString: function () { print("unexpected-key-coercion"); return "value"; } };
async function* assign() {
  try {
    target[key] = yield Promise.reject(marker);
    print("unreachable");
  } catch (error) {
    print("caught:" + (error === marker));
  } finally {
    print("finally");
  }
  print("writes:" + writes);
}
assign().next().then(function (result) {
  print(result.value + ":" + result.done);
});
void 0;
"#,
        &["caught:true", "finally", "writes:0", "undefined:true"],
    );
}

#[test]
fn synchronous_generators_also_defer_key_conversion_until_normal_completion() {
    assert_aot_completion_trace(
        r#"
var original = {};
var target = original;
var key = { toString: function () { print("coerce"); return "value"; } };
function* source() { yield 2; return 8; }
function* plain() { target[key] = yield 1; }
function* delegated() { target[key] = yield* source(); }
var a = plain();
print(a.next().value);
target = {};
a.next(7);
print(original.value + ":" + target.value);
var b = delegated();
print(b.next().value);
b.next();
print(target.value);
var c = plain();
print(c.next().value);
c.return(9);
print(target.value);
"#,
        &["1", "coerce", "7:undefined", "2", "coerce", "8", "1", "8"],
    );
}

#[test]
fn key_conversion_and_setter_throws_reach_the_async_handler_without_losing_rhs() {
    assert_aot_completion_trace(
        r#"
var marker = {};
var key = { toString: function () { print("convert"); throw marker; } };
var target = { set value(value) { print("set:" + value); throw marker; } };
async function* convert() {
  try { target[key] = yield 1; print("unreachable"); }
  catch (error) { print("convert-caught:" + (error === marker)); }
  finally { print("convert-finally"); }
}
async function* setter() {
  try { target.value = yield 2; print("unreachable"); }
  catch (error) { print("set-caught:" + (error === marker)); }
  finally { print("set-finally"); }
}
var a = convert();
var b = setter();
a.next().then(function (result) {
  print(result.value); return a.next(41);
}).then(function (result) {
  print(result.done); return b.next();
}).then(function (result) {
  print(result.value); return b.next(42);
}).then(function (result) { print(result.done); });
void 0;
"#,
        &[
            "1",
            "convert",
            "convert-caught:true",
            "convert-finally",
            "true",
            "2",
            "set:42",
            "set-caught:true",
            "set-finally",
            "true",
        ],
    );
}

#[test]
fn nullish_base_rejects_after_rhs_without_key_coercion() {
    assert_aot_completion_trace(
        r#"
var key = { toString: function () { print("unreachable-key"); return "value"; } };
function base() { print("base"); return null; }
function right() { print("rhs"); return 1; }
async function* assign() {
  try { base()[key] = yield right(); }
  catch (error) { print("type:" + (error instanceof TypeError)); }
}
var a = assign();
a.next().then(function (result) {
  print(result.value); return a.next(42);
}).then(function (result) { print(result.done); });
void 0;
"#,
        &["base", "rhs", "1", "type:true", "true"],
    );
}

#[test]
fn delegated_return_runs_the_finalizer_without_coercing_or_assigning() {
    assert_aot_completion_trace(
        r#"
var target = {};
var key = { toString: function () { print("unreachable-key"); return "value"; } };
async function* source() { try { yield 1; } finally { print("inner-finally"); } }
async function* assign() {
  try { target[key] = yield* source(); print("unreachable"); }
  finally { print("outer-finally:" + target.value); }
}
var a = assign();
a.next().then(function (result) {
  print(result.value); return a.return(99);
}).then(function (result) { print(result.value + ":" + result.done); });
void 0;
"#,
        &["1", "inner-finally", "outer-finally:undefined", "99:true"],
    );
}

#[test]
fn symbol_keys_and_sloppy_failed_sets_survive_async_resumption() {
    assert_aot_completion_trace(
        r#"
var target = {};
var symbol = Symbol("key");
var key = { [Symbol.toPrimitive]: function (hint) { print(hint); return symbol; } };
var frozen = Object.freeze({ value: 3 });
async function* assign() {
  target[key] = yield 1;
  print("symbol:" + target[symbol]);
  frozen.value = yield 2;
  print("frozen:" + frozen.value);
}
var a = assign();
a.next().then(function (result) {
  print(result.value); return a.next(42);
}).then(function (result) {
  print(result.value); return a.next(99);
}).then(function (result) { print(result.done); });
void 0;
"#,
        &["1", "string", "symbol:42", "2", "frozen:3", "true"],
    );
}

#[test]
fn delegated_completion_converts_the_original_raw_key_once() {
    assert_aot_completion_trace(
        r#"
var target = {};
var raw = { name: "old", toString: function () { print("convert:" + this.name); return this.name; } };
var key = raw;
function* source() { yield 1; yield 2; return 42; }
async function* assign() {
  target[key] = yield* source();
  print("assigned:" + target.old + ":" + target.changed + ":" + target.wrong);
}
var a = assign();
a.next().then(function (result) {
  print(result.value);
  raw.name = "changed";
  key = "wrong";
  return a.next();
}).then(function (result) {
  print(result.value); return a.next();
}).then(function (result) { print(result.done); });
void 0;
"#,
        &[
            "1",
            "2",
            "convert:changed",
            "assigned:undefined:42:undefined",
            "true",
        ],
    );
}

#[test]
fn queued_requests_and_interleaved_generators_keep_separate_references() {
    assert_aot_completion_trace(
        r#"
var left = {};
var right = {};
async function* assign(target) {
  target.first = yield 1;
  target.second = yield 2;
}
var a = assign(left);
var b = assign(right);
a.next();
b.next();
var a1 = a.next(11);
var b1 = b.next(21);
var a2 = a.next(12);
var b2 = b.next(22);
a1.then(function (result) { print("a1:" + result.value); return a2; })
.then(function (result) { print("a2:" + result.done); return b1; })
.then(function (result) { print("b1:" + result.value); return b2; })
.then(function (result) {
  print("b2:" + result.done);
  print(left.first + ":" + left.second + ":" + right.first + ":" + right.second);
});
void 0;
"#,
        &["a1:2", "a2:true", "b1:2", "b2:true", "11:12:21:22"],
    );
}

#[test]
fn reference_operand_throws_prevent_rhs_evaluation() {
    assert_aot_completion_trace(
        r#"
var marker = {};
function base() { print("base"); throw marker; }
function key() { print("key"); throw marker; }
function right() { print("unreachable-rhs"); return 1; }
async function* first() {
  try { base()[key()] = yield right(); }
  catch (error) { print("base-caught:" + (error === marker)); }
}
async function* second() {
  var target = {};
  try { target[key()] = yield right(); }
  catch (error) { print("key-caught:" + (error === marker)); }
}
first().next().then(function (result) {
  print(result.done); return second().next();
}).then(function (result) { print(result.done); });
void 0;
"#,
        &[
            "base",
            "base-caught:true",
            "true",
            "key",
            "key-caught:true",
            "true",
        ],
    );
}

#[test]
fn delegated_strict_assignment_failure_runs_catch_and_finally() {
    assert_aot_completion_trace(
        r#"
var target = Object.freeze({ value: 3 });
function* source() { yield 1; return 42; }
async function* assign() {
  "use strict";
  try { target.value = yield* source(); print("unreachable"); }
  catch (error) { print("type:" + (error instanceof TypeError)); }
  finally { print("finally:" + target.value); }
}
var a = assign();
a.next().then(function (result) {
  print(result.value); return a.next();
}).then(function (result) { print(result.done); });
void 0;
"#,
        &["1", "type:true", "finally:3", "true"],
    );
}

#[test]
fn delegated_missing_return_runs_finally_and_retires_pending_reference() {
    assert_aot_completion_trace(
        r#"
var object = {};
var key = { toString: function () { print("wrong-coercion"); return "value"; } };
var source = { next: function () { return { value: 1, done: false }; } };
source[Symbol.iterator] = function () { return this; };
async function* assign() {
  try { object[key] = yield* source; }
  finally { print("finally"); yield Promise.resolve(7); print("finally-resumed"); }
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result); return iterator.return(99);
}).then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); print("assigned:" + object.value);
});
void 0;
"#,
        &[
            "1:false",
            "finally",
            "7:false",
            "finally-resumed",
            "99:true",
            "assigned:undefined",
        ],
    );
}

#[test]
fn delegated_rejection_enters_catch_and_allows_a_fresh_yield() {
    assert_aot_completion_trace(
        r#"
var marker = {};
var object = {};
var key = { toString: function () { print("wrong-coercion"); return "value"; } };
var source = { next: function () { return Promise.reject(marker); } };
source[Symbol.asyncIterator] = function () { return this; };
async function* assign() {
  try { object[key] = yield* source; }
  catch (error) { print("caught:" + (error === marker)); yield Promise.resolve(8); }
  finally { print("finally"); }
  print("assigned:" + object.value);
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result); return iterator.next();
}).then(report);
void 0;
"#,
        &[
            "caught:true",
            "8:false",
            "finally",
            "assigned:undefined",
            "undefined:true",
        ],
    );
}

#[test]
fn delegated_getter_failure_retires_the_record_before_a_handler_yields() {
    assert_aot_completion_trace(
        r#"
var marker = {};
var object = {};
var key = { toString: function () { print("wrong-coercion"); return "value"; } };
var source = { next: function () {
  return { get done() { throw marker; }, get value() { print("wrong-value"); return 0; } };
} };
source[Symbol.asyncIterator] = function () { return this; };
async function* assign() {
  try { object[key] = yield* source; }
  catch (error) { print("caught:" + (error === marker)); yield Promise.resolve(9); }
  finally { print("finally"); }
  print("assigned:" + object.value);
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result); return iterator.next();
}).then(report);
void 0;
"#,
        &[
            "caught:true",
            "9:false",
            "finally",
            "assigned:undefined",
            "undefined:true",
        ],
    );
}

#[test]
fn delegated_throw_recovery_commits_only_the_terminal_value() {
    assert_aot_completion_trace(
        r#"
var marker = {};
var object = {};
var key = { toString: function () { print("coerce"); return "value"; } };
var source = {
  next: function () { return Promise.resolve({ value: 1, done: false }); },
  throw: function (error) {
    print("handled:" + (error === marker));
    return Promise.resolve({ value: 44, done: true });
  }
};
source[Symbol.asyncIterator] = function () { return this; };
async function* assign() {
  try { object[key] = yield* source; print("assigned:" + object.value); }
  finally { print("finally"); }
}
var iterator = assign();
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result); return iterator.throw(marker);
}).then(report);
void 0;
"#,
        &[
            "1:false",
            "handled:true",
            "coerce",
            "assigned:44",
            "finally",
            "undefined:true",
        ],
    );
}
