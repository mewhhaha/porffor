use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn run_boolean(source: &str) {
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
        .expect("bound function metadata should execute through Wasm");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn bound_functions_define_own_metadata_and_preserve_it_when_chained() {
    run_boolean(
        r#"
function target(a, b, c) { return a + b + c; }
var bound = target.bind(null, 1);
var chained = bound.bind(null, 2);
var length = Object.getOwnPropertyDescriptor(bound, "length");
var name = Object.getOwnPropertyDescriptor(bound, "name");
var correct = bound.length === 2 && bound.name === "bound target"
  && chained.length === 1 && chained.name === "bound bound target"
  && chained(3) === 6
  && length.value === 2 && length.writable === false
  && length.enumerable === false && length.configurable === true
  && name.value === "bound target" && name.writable === false
  && name.enumerable === false && name.configurable === true
  && Object.getOwnPropertyDescriptor(bound, "prototype") === undefined;
Object.defineProperty(bound, "length", {value: 9});
Object.defineProperty(bound, "name", {value: "renamed"});
correct && bound.length === 9 && bound.name === "renamed";
"#,
    );
}

#[test]
fn bound_function_length_preserves_large_numbers_and_handles_numeric_edges() {
    run_boolean(
        r#"
function target() {}
function lengthWith(value, count) {
  Object.defineProperty(target, "length", {value: value});
  if (count === 0) return target.bind().length;
  return target.bind(null, 1, 2).length;
}
lengthWith(4294967296, 2) === 4294967294
  && lengthWith(9007199254740991, 2) === 9007199254740989
  && lengthWith(Infinity, 0) === Infinity
  && lengthWith(Infinity, 2) === Infinity
  && lengthWith(-Infinity, 2) === 0
  && lengthWith(NaN, 0) === 0
  && 1 / lengthWith(-0, 0) === Infinity
  && lengthWith(3.66, 0) === 3
  && lengthWith(3.66, 2) === 1
  && lengthWith(1, 2) === 0
  && 1 / lengthWith(-0.77, 0) === Infinity;
"#,
    );
}

#[test]
fn bind_observes_metadata_getters_in_order_and_propagates_their_exceptions() {
    run_boolean(
        r#"
function target() {}
var reads = "";
var sentinel = {};
var correctReceiver = true;
Object.defineProperty(target, "length", {
  get: function () {
    reads += "length;";
    correctReceiver = correctReceiver && this === target;
    return 3;
  }
});
Object.defineProperty(target, "name", {
  get: function () {
    reads += "name;";
    correctReceiver = correctReceiver && this === target;
    return "observable";
  }
});
var bound = target.bind(null, 1);
var ordered = reads === "length;name;" && correctReceiver
  && bound.length === 2 && bound.name === "bound observable";
Object.defineProperty(target, "length", {
  get: function () { reads += "throw-length;"; throw sentinel; }
});
var lengthError;
try { target.bind(); } catch (error) { lengthError = error; }
var lengthAbrupt = lengthError === sentinel
  && reads === "length;name;throw-length;";
Object.defineProperty(target, "length", {value: 1});
Object.defineProperty(target, "name", {
  get: function () { reads += "throw-name;"; throw sentinel; }
});
var nameError;
try { target.bind(); } catch (error) { nameError = error; }
ordered && lengthAbrupt && nameError === sentinel
  && reads === "length;name;throw-length;throw-name;";
"#,
    );
}

#[test]
fn bind_ignores_inherited_length_and_nonprimitive_metadata_without_coercion() {
    run_boolean(
        r#"
var bind = Function.prototype.bind;
var coercions = 0;
var inheritedLengthReads = 0;
var inheritedNameReads = 0;
var metadata = {
  valueOf: function () { coercions++; return 4; },
  toString: function () { coercions++; return "coerced"; }
};
function target() {}
Object.defineProperty(target, "length", {value: metadata});
Object.defineProperty(target, "name", {value: metadata});
var nonprimitive = target.bind();
var first = nonprimitive.length === 0 && nonprimitive.name === "bound ";
delete target.length;
delete target.name;
var inherited = {
  get length() { inheritedLengthReads++; throw new Error("inherited length"); },
  get name() { inheritedNameReads++; return "inherited"; }
};
Object.setPrototypeOf(target, inherited);
var bound = bind.call(target, null);
var inheritedCorrect = bound.length === 0 && bound.name === "bound inherited"
  && Object.getPrototypeOf(bound) === inherited;
Object.setPrototypeOf(target, null);
var nullPrototype = bind.call(target, null);
first && inheritedCorrect && coercions === 0
  && inheritedLengthReads === 0 && inheritedNameReads === 1
  && Object.getPrototypeOf(nullPrototype) === null
  && nullPrototype.length === 0 && nullPrototype.name === "bound ";
"#,
    );
}

#[test]
fn bind_observes_proxy_prototype_before_metadata_and_keeps_an_abrupt_value() {
    run_boolean(
        r#"
var trace = "";
var prototype = {};
var abrupt = {};
function target(a, b) {}
var proxy = new Proxy(target, {
  getPrototypeOf: function () { trace += "p"; return prototype; },
  getOwnPropertyDescriptor: function (object, key) {
    if (key === "length") trace += "o";
    return Reflect.getOwnPropertyDescriptor(object, key);
  },
  get: function (object, key, receiver) {
    if (key === "length") trace += "l";
    if (key === "name") trace += "n";
    return Reflect.get(object, key, receiver);
  }
});
var bound = Function.prototype.bind.call(proxy, null, 1);
var correct = trace === "poln" && Object.getPrototypeOf(bound) === prototype
  && bound.length === 1 && bound.name === "bound target";
trace = "";
var throwing = new Proxy(target, {
  getPrototypeOf: function () { trace += "p"; throw abrupt; },
  getOwnPropertyDescriptor: function () { trace += "o"; throw "late descriptor"; },
  get: function () { trace += "g"; throw "late Get"; }
});
var observed;
try { Function.prototype.bind.call(throwing, null); } catch (error) { observed = error; }
correct = correct && observed === abrupt && trace === "p";
trace = "";
var descriptorThrowing = new Proxy(target, {
  getPrototypeOf: function () { trace += "p"; return prototype; },
  getOwnPropertyDescriptor: function () { trace += "o"; throw abrupt; },
  get: function () { trace += "g"; throw "late Get"; }
});
observed = undefined;
try { Function.prototype.bind.call(descriptorThrowing, null); } catch (error) { observed = error; }
correct && observed === abrupt && trace === "po";
"#,
    );
}

#[test]
fn bound_arguments_keep_reference_identity_and_exact_primitive_this() {
    run_boolean(
        r#"
var symbol = Symbol("argument");
var object = {};
var bigint = 18446744073709551615n;
function target(first, second, third) {
  "use strict";
  return this === 17 && first === symbol && second === object && third === bigint
    && arguments.length === 3 && arguments[0] === symbol
    && arguments[1] === object && arguments[2] === bigint;
}
var first = target.bind(17, symbol);
var second = first.bind(99, object);
second.call({}, bigint);
"#,
    );
}

#[test]
fn bound_construction_replaces_only_its_own_new_target_identity() {
    run_boolean(
        r#"
var symbol = Symbol("argument");
var object = {};
function Target(first, second) {
  this.first = first;
  this.second = second;
  this.seen = new.target;
}
function Other() {}
var first = Target.bind(null, symbol);
var second = first.bind({}, object);
var ordinary = new second();
var explicit = Reflect.construct(second, [], Other);
ordinary.first === symbol && ordinary.second === object && ordinary.seen === Target
  && Object.getPrototypeOf(ordinary) === Target.prototype
  && explicit.first === symbol && explicit.second === object && explicit.seen === Other
  && Object.getPrototypeOf(explicit) === Other.prototype;
"#,
    );
}

#[test]
fn proxy_revoker_uses_its_capture_ignores_this_and_is_idempotent() {
    run_boolean(
        r#"
var pair = Proxy.revocable({ value: 7 }, {});
var unrelated = new Proxy({ value: 9 }, {});
var revoke = pair.revoke;
var metadata = revoke.name === "" && revoke.length === 0
  && Object.getPrototypeOf(revoke) === Function.prototype
  && Object.getOwnPropertyDescriptor(revoke, "prototype") === undefined;
var first = revoke.call(unrelated);
var second = revoke.call(null);
var caught;
try { pair.proxy.value; } catch (error) { caught = error; }
metadata && first === undefined && second === undefined
  && caught instanceof TypeError && unrelated.value === 9;
"#,
    );
}
