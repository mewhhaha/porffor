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
