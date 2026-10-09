//! A program may replace any method of a built-in prototype. Static lowering
//! resolves a method call to a builtin — and takes the builtin's result kind —
//! only while it can prove the prototype still holds that builtin; otherwise
//! the call is an ordinary property read and call of whatever is installed.
//!
//! Every replacement below returns a value of a *different* kind than the
//! builtin, and the result is then combined with `+ 1`, so a stale static
//! result kind is observable (string concatenation instead of addition, or the
//! builtin's value instead of the replacement's).
use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_string(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .expect("replaced-builtin program must compile and execute through Wasm AOT");
    let completion = format!("string({expected})");
    assert!(
        outcome.note.contains(&completion),
        "expected completion {completion}, got {}",
        outcome.note
    );
}

/// After an opaque call `TypeError` is an arbitrary value, so its result may
/// be a function, an array or any object; nothing may claim
/// `Function.prototype.toString` for it.
#[test]
fn a_widened_receiver_resolves_no_function_prototype_to_string() {
    assert_wasm_string(
        r#"
function add(x, y) { return x + y; }
Error.prototype.toString = function () { return 8; };
let ok = add(2, 1) === 3 && true;
let s = TypeError("y").toString();
typeof s + "|" + (s + 1) + "|" + ok;
"#,
        "number|9|true",
    );
}

#[test]
fn replaced_function_prototype_apply_call_and_bind_are_called() {
    assert_wasm_string(
        r#"
function f() { return 1; }
Function.prototype.apply = function () { return "applied"; };
Function.prototype.call = function () { return "called"; };
Function.prototype.bind = function () { return 7; };
let a = f.apply(null, []);
let c = f.call(null);
let b = f.bind(null);
typeof a + "|" + (a + 1) + "|" + (c + 1) + "|" + typeof b + "|" + (b + 1);
"#,
        "string|applied1|called1|number|8",
    );
}

#[test]
fn replaced_function_prototype_apply_written_inside_a_called_function() {
    assert_wasm_string(
        r#"
function f() { return 1; }
function install() { Function.prototype.apply = function () { return "replaced"; }; }
install();
let r = f.apply(null, []);
typeof r + "|" + (r + 1);
"#,
        "string|replaced1",
    );
}

#[test]
fn replaced_function_prototype_to_string_is_called() {
    assert_wasm_string(
        r#"
Function.prototype.toString = function () { return 7; };
function f() { return 1; }
let r = f.toString();
typeof r + "|" + (r + 1);
"#,
        "number|8",
    );
}

#[test]
fn replaced_array_prototype_map_on_a_value_that_may_be_an_array() {
    assert_wasm_string(
        r#"
Array.prototype.map = function () { return "mapped"; };
function pick(flag) { return flag ? [1, 2] : undefined; }
let maybe = pick(true);
let r = maybe.map(function (x) { return x * 2; });
typeof r + "|" + (r + 1);
"#,
        "string|mapped1",
    );
}

#[test]
fn a_value_that_may_be_an_array_or_an_object_with_its_own_map() {
    assert_wasm_string(
        r#"
function pick(flag) { return flag ? [1, 2] : { map() { return 5; } }; }
let either = pick(false);
let r = either.map(function (x) { return x; });
typeof r + "|" + (r + 1);
"#,
        "number|6",
    );
}

#[test]
fn replaced_primitive_prototype_methods_are_called() {
    assert_wasm_string(
        r#"
String.prototype.toUpperCase = function () { return 5; };
String.prototype.split = function () { return 6; };
String.prototype.charCodeAt = function () { return "c"; };
Number.prototype.toFixed = function () { return 3; };
Boolean.prototype.valueOf = function () { return "v"; };
let u = "a".toUpperCase();
let s = "a,b".split(",");
let c = "abc".charCodeAt(0);
let n = (1.5).toFixed(1);
let v = true.valueOf();
(u + 1) + "|" + (s + 1) + "|" + (c + 1) + "|" + (n + 1) + "|" + (v + 1);
"#,
        "6|7|c1|4|v1",
    );
}

#[test]
fn replaced_string_method_is_called_from_a_function_body() {
    assert_wasm_string(
        r#"
function shout(s) { let r = s.toUpperCase(); return typeof r + "|" + (r + 1); }
String.prototype.toUpperCase = function () { return 5; };
shout("a");
"#,
        "number|6",
    );
}

#[test]
fn unmodified_builtin_methods_keep_their_results() {
    assert_wasm_string(
        r#"
function f(a, b) { return a + b; }
let applied = f.apply(null, [2, 3]);
let called = f.call(null, 4, 5);
let bound = f.bind(null, 1)(1);
let mapped = [1, 2].map(function (x) { return x * 2; }).join(",");
let text = "abc".toUpperCase() + "a,b".split(",").length + "abc".charCodeAt(1);
let fixed = (1.25).toFixed(1);
let errors = TypeError("y").toString() + "/" + new Error("x").toString();
let caught;
try { null.x; } catch (e) { caught = e.toString().slice(0, 9); }
[applied + 1, called, bound, mapped, text, fixed, errors, caught].join("|");
"#,
        "6|9|2|2,4|ABC298|1.3|TypeError: y/Error: x|TypeError",
    );
}
