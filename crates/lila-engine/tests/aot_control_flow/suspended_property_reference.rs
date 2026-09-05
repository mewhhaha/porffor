use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
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
        .expect("suspended Reference regression must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "unexpected completion: {:?}",
        outcome.completion,
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn raw_key_and_base_are_pinned_until_normal_resume() {
    assert_trace(
        r#"
var target = {};
var original = target;
var spelling = "early";
var key = { toString: function () { print("coerce:" + spelling); return spelling; } };
function base() { print("base"); return target; }
function name() { print("key"); return key; }
function rhs() { print("rhs"); return 7; }
function* assign() { base()[name()] = yield rhs(); print("after"); }
var iterator = assign();
var first = iterator.next();
print(first.value + ":" + first.done);
target = {}; key = { toString: function () { throw "reevaluated"; } }; spelling = "late";
var last = iterator.next(42);
print(last.value + ":" + last.done);
print(original.late + ":" + original.early + ":" + target.late);
void 0;
"#,
        &[
            "base",
            "key",
            "rhs",
            "7:false",
            "coerce:late",
            "after",
            "undefined:true",
            "42:undefined:undefined",
        ],
    );
}

#[test]
fn throw_resume_skips_key_coercion_and_write() {
    assert_trace(
        r#"
var target = {}; var marker = {};
var key = { toString: function () { print("coerce"); return "p"; } };
function* assign() { try { target[key] = yield 1; print("after"); } finally { print("finally"); } }
var iterator = assign(); print(iterator.next().value);
try { iterator.throw(marker); } catch (error) { print(error === marker); }
print(target.p); print(iterator.next().done);
void 0;
"#,
        &["1", "finally", "true", "undefined", "true"],
    );
}

#[test]
fn return_resume_skips_write_across_suspending_finally() {
    assert_trace(
        r#"
var target = {};
var key = { toString: function () { print("coerce"); return "p"; } };
function* assign() {
  try { target[key] = yield 1; print("after"); }
  finally { print("finally"); yield 9; print("finally-end"); }
}
var iterator = assign(); print(iterator.next().value);
var pending = iterator.return(77); print(pending.value + ":" + pending.done);
var last = iterator.next(); print(last.value + ":" + last.done); print(target.p);
void 0;
"#,
        &[
            "1",
            "finally",
            "9:false",
            "finally-end",
            "77:true",
            "undefined",
        ],
    );
}

#[test]
fn nullish_base_throws_after_rhs_before_key_conversion() {
    assert_trace(
        r#"
var target = null;
var key = { toString: function () { print("coerce"); throw "wrong error"; } };
function* assign() { target[key] = yield 2; print("after"); }
var iterator = assign(); print(iterator.next().value);
try { iterator.next(8); } catch (error) { print(error instanceof TypeError); }
print(iterator.next().done);
void 0;
"#,
        &["2", "true", "true"],
    );
}

#[test]
fn key_expression_throw_precedes_rhs() {
    assert_trace(
        r#"
var marker = {}; var target = {};
function key() { print("key"); throw marker; }
function rhs() { print("rhs"); return 1; }
function* assign() { target[key()] = yield rhs(); print("after"); }
try { assign().next(); } catch (error) { print(error === marker); }
print(target.p);
void 0;
"#,
        &["key", "true", "undefined"],
    );
}

#[test]
fn coercion_throw_routes_through_generator_finally() {
    assert_trace(
        r#"
var marker = {}; var target = {};
var key = { toString: function () { print("coerce"); throw marker; } };
function* assign() { try { target[key] = yield 1; print("after"); } finally { print("finally"); } }
var iterator = assign(); print(iterator.next().value);
try { iterator.next(8); } catch (error) { print(error === marker); }
print(target.p); print(iterator.next().done);
void 0;
"#,
        &["1", "coerce", "finally", "true", "undefined", "true"],
    );
}

#[test]
fn symbol_key_keeps_its_tag_after_resume() {
    assert_trace(
        r#"
var target = {}; var symbol = Symbol("key");
var key = { [Symbol.toPrimitive]: function (hint) { print("hint:" + hint); return symbol; } };
function* assign() { target[key] = yield 3; }
var iterator = assign(); print(iterator.next().value);
print(iterator.next(19).done); print(target[symbol]); print(Object.keys(target).length);
void 0;
"#,
        &["3", "hint:string", "true", "19", "0"],
    );
}

#[test]
fn strict_failed_set_throws_after_key_coercion() {
    assert_trace(
        r#"
var target = {}; Object.defineProperty(target, "p", { value: 5, writable: false });
var key = { toString: function () { print("coerce"); return "p"; } };
function* assign() { "use strict"; target[key] = yield 1; print("after"); }
var iterator = assign(); print(iterator.next().value);
try { iterator.next(9); } catch (error) { print(error instanceof TypeError); }
print(target.p); print(iterator.next().done);
void 0;
"#,
        &["1", "coerce", "true", "5", "true"],
    );
}

#[test]
fn sloppy_failed_set_returns_normally() {
    assert_trace(
        r#"
var target = {}; Object.defineProperty(target, "p", { value: 5, writable: false });
var key = { toString: function () { print("coerce"); return "p"; } };
function* assign() { target[key] = yield 1; print("after"); }
var iterator = assign(); print(iterator.next().value);
print(iterator.next(9).done); print(target.p);
void 0;
"#,
        &["1", "coerce", "after", "true", "5"],
    );
}

#[test]
fn delegated_yield_uses_terminal_value_and_late_key() {
    assert_trace(
        r#"
var target = {}; var spelling = "early";
var key = { toString: function () { print("coerce:" + spelling); return spelling; } };
function* values() { yield 1; yield 2; return 43; }
function* assign() { target[key] = yield* values(); print("after"); }
var iterator = assign(); print(iterator.next().value); print(iterator.next().value);
spelling = "late"; print(iterator.next().done); print(target.late + ":" + target.early);
void 0;
"#,
        &["1", "2", "coerce:late", "after", "true", "43:undefined"],
    );
}

#[test]
fn delegate_handled_throw_can_finish_assignment_normally() {
    assert_trace(
        r#"
var target = {}; var marker = {};
var key = { toString: function () { print("coerce"); return "p"; } };
function* values() { try { yield 1; } catch (error) { print(error === marker); return 44; } }
function* assign() { target[key] = yield* values(); print("after"); }
var iterator = assign(); print(iterator.next().value);
print(iterator.throw(marker).done); print(target.p);
void 0;
"#,
        &["1", "true", "coerce", "after", "true", "44"],
    );
}

#[test]
fn delegate_return_followed_by_next_can_finish_assignment_normally() {
    assert_trace(
        r#"
var target = {};
var key = { toString: function () { print("coerce"); return "p"; } };
function* values() { try { yield 1; } finally { yield 9; } }
function* assign() { target[key] = yield* values(); print("after"); }
var iterator = assign(); print(iterator.next().value);
var pending = iterator.return(55); print(pending.value + ":" + pending.done);
var last = iterator.next(); print(last.value + ":" + last.done); print(target.p);
void 0;
"#,
        &["1", "9:false", "coerce", "after", "undefined:true", "55"],
    );
}

#[test]
fn delegate_immediate_return_does_not_commit_pending_assignment() {
    assert_trace(
        r#"
var target = {};
var key = { toString: function () { print("coerce"); return "p"; } };
function* values() { yield 1; return 9; }
function* assign() { target[key] = yield* values(); print("after"); }
var iterator = assign(); print(iterator.next().value);
var last = iterator.return(55); print(last.value + ":" + last.done); print(target.p);
void 0;
"#,
        &["1", "55:true", "undefined"],
    );
}

#[test]
fn rhs_throw_does_not_trigger_key_coercion() {
    assert_trace(
        r#"
var target = {}; var marker = {};
var key = { toString: function () { print("coerce"); return "p"; } };
function rhs() { print("rhs"); throw marker; }
function* assign() { target[key] = yield rhs(); print("after"); }
try { assign().next(); } catch (error) { print(error === marker); }
print(target.p);
void 0;
"#,
        &["rhs", "true", "undefined"],
    );
}
