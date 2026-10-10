use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

const STA: &str = include_str!("../../../../test262/vendor/test262/harness/sta.js");
const ASSERT: &str = include_str!("../../../../test262/vendor/test262/harness/assert.js");
const NATIVE_MATCHER: &str =
    include_str!("../../../../test262/vendor/test262/harness/nativeFunctionMatcher.js");
const INTRINSICS: &str =
    include_str!("../../../../test262/vendor/test262/harness/wellKnownIntrinsicObjects.js");
const PINNED_NATIVE_FUNCTIONS: &str = include_str!(
    "../../../../test262/vendor/test262/test/built-ins/Function/prototype/toString/built-in-function-object.js"
);

fn assert_wasm_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("native function source controls compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn unchanged_pinned_intrinsic_traversal_uses_complete_native_matcher_in_both_modes() {
    for directive in ["", "\"use strict\";\n"] {
        assert_wasm_true(&format!(
            "{directive}{STA}\n{ASSERT}\n{NATIVE_MATCHER}\n{INTRINSICS}\n{PINNED_NATIVE_FUNCTIONS}\ntrue;"
        ));
    }
}

#[test]
fn native_sources_retain_initial_names_without_observing_mutable_name_properties() {
    let controls = r#"
var toSource = Function.prototype.toString;
var legacy = Object.getOwnPropertyDescriptor(RegExp, "input");
var alias = Object.getOwnPropertyDescriptor(RegExp, "$_");
assert.sameValue(legacy.get, alias.get);
assert.sameValue(legacy.set, alias.set);
assert.sameValue(legacy.get.name, "get input");
assert.sameValue(legacy.set.name, "set input");
assert.sameValue(toSource.call(legacy.get), "function get input() { [native code] }");
assert.sameValue(toSource.call(legacy.set), "function set input() { [native code] }");
var species = Object.getOwnPropertyDescriptor(Array, Symbol.species).get;
assert.sameValue(species.name, "get [Symbol.species]");
assert.sameValue(toSource.call(species), "function get [Symbol.species]() { [native code] }");
assertNativeFunction(species);
var callables = [legacy.get, legacy.set, Array.prototype.map, species];
var nameReads = 0;
for (var index = 0; index < callables.length; index++) {
  var callable = callables[index];
  var initialSource = toSource.call(callable);
  assertNativeFunction(callable);
  Object.defineProperty(callable, "name", {value: "changed " + index, configurable: true});
  assert.sameValue(callable.name, "changed " + index);
  assert.sameValue(toSource.call(callable), initialSource);
  Object.defineProperty(callable, "name", {
    get: function() { nameReads++; throw new Test262Error("toString read public name"); },
    configurable: true
  });
  assert.sameValue(toSource.call(callable), initialSource);
  assertNativeFunction(callable);
  assert.sameValue(delete callable.name, true);
  assert.sameValue(toSource.call(callable), initialSource);
}
assert.sameValue(nameReads, 0);
true;
"#;
    for directive in ["", "\"use strict\";\n"] {
        assert_wasm_true(&format!(
            "{directive}{STA}\n{ASSERT}\n{NATIVE_MATCHER}\n{controls}"
        ));
    }
}

#[test]
fn legacy_regexp_aliases_share_one_canonical_accessor_per_captured_slot() {
    let controls = r#"
var pairs = [['input','$_'],['lastMatch','$&'],['lastParen','$+'],['leftContext','$`'],['rightContext',"$'"]];
var previous;
for (var index = 0; index < pairs.length; index++) {
  var canonical = pairs[index][0], alias = pairs[index][1];
  var first = Object.getOwnPropertyDescriptor(RegExp, canonical);
  var second = Object.getOwnPropertyDescriptor(RegExp, alias);
  assert.sameValue(first.get, second.get);
  assert.sameValue(first.set, second.set);
  assert.sameValue(first.get.name, 'get ' + canonical);
  assert.sameValue(Function.prototype.toString.call(first.get), 'function get ' + canonical + '() { [native code] }');
  assertNativeFunction(first.get);
  if (previous !== undefined) assert.notSameValue(first.get, previous);
  previous = first.get;
  if (canonical === 'input') {
    assert.sameValue(first.set.name, 'set input');
    assert.sameValue(Function.prototype.toString.call(first.set), 'function set input() { [native code] }');
    assertNativeFunction(first.set);
  } else assert.sameValue(first.set, undefined);
}
for (var capture = 1; capture <= 9; capture++) {
  var name = '$' + capture;
  var descriptor = Object.getOwnPropertyDescriptor(RegExp, name);
  assert.sameValue(descriptor.get.name, 'get ' + name);
  assert.sameValue(Function.prototype.toString.call(descriptor.get), 'function get ' + name + '() { [native code] }');
  assertNativeFunction(descriptor.get);
  assert.notSameValue(descriptor.get, previous);
  previous = descriptor.get;
}
true;
"#;
    for directive in ["", "'use strict';\n"] {
        assert_wasm_true(&format!(
            "{directive}{STA}\n{ASSERT}\n{NATIVE_MATCHER}\n{controls}"
        ));
    }
}
