use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_wasm_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for prefix in ["", "'use strict';\n"] {
        let engine = Engine::new(RealmBuilder::new().build());
        let outcome = engine
            .run_script(
                &format!("{prefix}{source}"),
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Arguments iterator mutation must compile and execute through Wasm AOT");
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }
}

#[test]
fn arguments_iterator_is_one_real_non_enumerable_own_symbol_property() {
    assert_wasm_true(
        r#"
function mapped(a, b) { return arguments; }
function unmapped(a, b) { 'use strict'; return arguments; }
function nonSimple(a = 0) { return arguments; }
var factories = [mapped, unmapped, nonSimple];
var ok = true;
for (var n = 0; n < factories.length; n++) {
  var args = factories[n](2, 3);
  var descriptor = Object.getOwnPropertyDescriptor(args, Symbol.iterator);
  var symbols = Object.getOwnPropertySymbols(args);
  var keys = Reflect.ownKeys(args);
  ok = ok && descriptor.value === Array.prototype.values &&
    descriptor.writable && !descriptor.enumerable && descriptor.configurable &&
    symbols.length === 1 && symbols[0] === Symbol.iterator &&
    keys.length === 5 && keys[4] === Symbol.iterator && Object.keys(args).join(',') === '0,1';
  args['Symbol.iterator'] = 17;
  ok = ok && args['Symbol.iterator'] === 17 && args[Symbol.iterator] === descriptor.value &&
    Object.getOwnPropertySymbols(args).length === 1;
}
ok;
"#,
    );
}

#[test]
fn replacement_persists_in_get_descriptor_and_iteration() {
    assert_wasm_true(
        r#"
function mapped(a, b) { return arguments; }
function unmapped(a, b) { 'use strict'; return arguments; }
var factories = [mapped, unmapped];
var original = Array.prototype.values;
var calls = 0;
var receiver;
function replacement() { calls++; receiver = this; return original.call(this); }
var ok = true;
for (var n = 0; n < factories.length; n++) {
  var args = factories[n](4, 5);
  args[Symbol.iterator] = replacement;
  var descriptor = Object.getOwnPropertyDescriptor(args, Symbol.iterator);
  var collected = '';
  for (var value of args) collected += value + ':';
  ok = ok && args[Symbol.iterator] === replacement && Reflect.get(args, Symbol.iterator) === replacement &&
    descriptor.value === replacement && descriptor.writable && !descriptor.enumerable && descriptor.configurable &&
    collected === '4:5:' && receiver === args;
}
ok && calls === 2 && Array.prototype.values === original;
"#,
    );
}

#[test]
fn deletion_persists_and_recreation_uses_normal_assignment_attributes() {
    assert_wasm_true(
        r#"
function exercise() {
  var original = arguments[Symbol.iterator];
  var initiallyPresent = Symbol.iterator in arguments;
  var removed = delete arguments[Symbol.iterator];
  var absent = !Object.hasOwn(arguments, Symbol.iterator) && !(Symbol.iterator in arguments) &&
    Object.getOwnPropertyDescriptor(arguments, Symbol.iterator) === undefined &&
    arguments[Symbol.iterator] === undefined && Object.getOwnPropertySymbols(arguments).length === 0;
  var threw = false;
  try { for (var value of arguments) {} } catch (error) { threw = error instanceof TypeError; }
  arguments[Symbol.iterator] = original;
  var descriptor = Object.getOwnPropertyDescriptor(arguments, Symbol.iterator);
  return initiallyPresent && removed && absent && threw && (Symbol.iterator in arguments) &&
    descriptor.value === original && descriptor.writable && descriptor.enumerable && descriptor.configurable &&
    Object.getOwnPropertySymbols(arguments).length === 1;
}
function strictExercise() {
  'use strict';
  var original = arguments[Symbol.iterator];
  var removed = delete arguments[Symbol.iterator];
  var absent = !Object.hasOwn(arguments, Symbol.iterator) && !(Symbol.iterator in arguments) &&
    Object.getOwnPropertyDescriptor(arguments, Symbol.iterator) === undefined && arguments[Symbol.iterator] === undefined;
  arguments[Symbol.iterator] = original;
  var descriptor = Object.getOwnPropertyDescriptor(arguments, Symbol.iterator);
  return removed && absent && descriptor.enumerable && descriptor.value === original;
}
exercise(1, 2) && strictExercise(3, 4);
"#,
    );
}

#[test]
fn iterator_descriptor_changes_and_inherited_accessors_use_the_receiver() {
    assert_wasm_true(
        r#"
function mapped() { return arguments; }
function unmapped() { 'use strict'; return arguments; }
var factories = [mapped, unmapped];
var original = Array.prototype.values;
var getters = 0;
var receiver;
var alternate = {};
function getter() { getters++; receiver = this; return original; }
var ok = true;
for (var n = 0; n < factories.length; n++) {
  var args = factories[n](6, 7);
  Object.defineProperty(args, Symbol.iterator, {value: original, writable: false, enumerable: true, configurable: true});
  var descriptor = Object.getOwnPropertyDescriptor(args, Symbol.iterator);
  ok = ok && !descriptor.writable && descriptor.enumerable && descriptor.configurable &&
    !Reflect.set(args, Symbol.iterator, function() {});
  Object.defineProperty(args, Symbol.iterator, {get: getter, enumerable: false, configurable: true});
  descriptor = Object.getOwnPropertyDescriptor(args, Symbol.iterator);
  ok = ok && descriptor.get === getter && descriptor.set === undefined && !('value' in descriptor) && getters === n * 3;
  ok = ok && Reflect.get(args, Symbol.iterator, alternate) === original && receiver === alternate;
  var collected = '';
  for (var value of args) collected += value + ':';
  ok = ok && collected === '6:7:' && receiver === args;
  delete args[Symbol.iterator];
  var prototype = {};
  Object.defineProperty(prototype, Symbol.iterator, {get: getter, configurable: true});
  Object.setPrototypeOf(args, prototype);
  ok = ok && !Object.hasOwn(args, Symbol.iterator) && (Symbol.iterator in args) &&
    Reflect.get(args, Symbol.iterator, alternate) === original && receiver === alternate &&
    Object.getOwnPropertySymbols(args).length === 0;
}
ok && getters === 6;
"#,
    );
}

#[test]
fn creation_captures_the_original_intrinsic_before_and_after_prototype_mutation() {
    assert_wasm_true(
        r#"
function mapped() { return arguments; }
function unmapped() { 'use strict'; return arguments; }
var original = Array.prototype.values;
var beforeMapped = mapped(1);
var beforeUnmapped = unmapped(2);
var valuesDescriptor = Object.getOwnPropertyDescriptor(Array.prototype, 'values');
var iteratorDescriptor = Object.getOwnPropertyDescriptor(Array.prototype, Symbol.iterator);
var getterCalls = 0;
function poison() { getterCalls++; throw new Error('mutable prototype consulted'); }
var ok;
try {
  Object.defineProperty(Array.prototype, 'values', {get: poison, configurable: true});
  Object.defineProperty(Array.prototype, Symbol.iterator, {get: poison, configurable: true});
  var afterMapped = mapped(3);
  var afterUnmapped = unmapped(4);
  ok = beforeMapped[Symbol.iterator] === original && beforeUnmapped[Symbol.iterator] === original &&
    afterMapped[Symbol.iterator] === original && afterUnmapped[Symbol.iterator] === original &&
    Object.getOwnPropertyDescriptor(afterMapped, Symbol.iterator).value === original &&
    Object.getOwnPropertyDescriptor(afterUnmapped, Symbol.iterator).value === original && getterCalls === 0;
} finally {
  Object.defineProperty(Array.prototype, 'values', valuesDescriptor);
  Object.defineProperty(Array.prototype, Symbol.iterator, iteratorDescriptor);
}
ok && getterCalls === 0;
"#,
    );
}

#[test]
fn created_realm_arguments_capture_that_realms_original_function_identity() {
    assert_wasm_true(
        r#"
var other = __lilaCreateRealm();
var original = other.global.Array.prototype.values;
other.global.originalIterator = original;
other.global.Array.prototype.values = function() { throw new Error('replacement values'); };
other.global.Array.prototype[Symbol.iterator] = function() { throw new Error('replacement iterator'); };
var result = other.evalScript("function mapped() { return arguments; } function unmapped() { 'use strict'; return arguments; } var a = mapped(8); var b = unmapped(9); a[Symbol.iterator] === originalIterator && b[Symbol.iterator] === originalIterator && Object.getOwnPropertyDescriptor(a, Symbol.iterator).value === originalIterator && Object.getOwnPropertyDescriptor(b, Symbol.iterator).value === originalIterator && Object.getPrototypeOf(a[Symbol.iterator]) === Function.prototype;");
result === true && original !== Array.prototype.values;
"#,
    );
}

#[test]
fn deleted_own_iterator_uses_inherited_symbol_and_never_the_string_alias() {
    assert_wasm_true(
        r#"
function mapped() { return arguments; }
function unmapped() { 'use strict'; return arguments; }
var factories = [mapped, unmapped];
var original = Array.prototype.values;
var getterCalls = 0;
var stringReads = 0;
var receiver;
function inherited() { getterCalls++; receiver = this; return original; }
function wrongString() { stringReads++; throw new Error('string iterator alias consulted'); }
var ok = true;
for (var n = 0; n < factories.length; n++) {
  var args = factories[n](8, 9);
  delete args[Symbol.iterator];
  var prototype = {};
  Object.defineProperty(prototype, Symbol.iterator, {get: inherited, configurable: true});
  Object.defineProperty(prototype, 'Symbol.iterator', {get: wrongString, configurable: true});
  Object.setPrototypeOf(args, prototype);
  var collected = '';
  for (var value of args) collected += value + ':';
  ok = ok && collected === '8:9:' && receiver === args && getterCalls === n + 1 && stringReads === 0 &&
    !Object.hasOwn(args, Symbol.iterator);
  delete prototype[Symbol.iterator];
  Object.defineProperty(prototype, 'Symbol.iterator', {value: original, configurable: true});
  var threw = false;
  try { for (var value of args) {} } catch (error) { threw = error instanceof TypeError; }
  ok = ok && threw && args[Symbol.iterator] === undefined && args['Symbol.iterator'] === original &&
    !(Symbol.iterator in args) && Object.getOwnPropertySymbols(args).length === 0;
}
ok && getterCalls === 2 && stringReads === 0;
"#,
    );
}
