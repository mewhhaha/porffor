use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_delete(source: &str) {
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
        .unwrap_or_else(|error| panic!("delete failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn nullish_bases_evaluate_raw_keys_before_throwing_without_coercion() {
    assert_delete(
        r#"
function remove(base, key) { return delete base[key]; }
function strictRemove(base, key) { 'use strict'; return delete base[key]; }
var trace = [], marker = {}, caught = 0;
var key = { [Symbol.toPrimitive]() { trace.push('coerce'); return 'x'; } };
function rawKey() { trace.push('raw'); return key; }
function failKey() { trace.push('throw'); throw marker; }
function failBase() { trace.push('base'); throw marker; }
try { delete null[rawKey()]; } catch (e) { if (e instanceof TypeError) caught++; }
try { delete undefined[rawKey()]; } catch (e) { if (e instanceof TypeError) caught++; }
try { remove(null, rawKey()); } catch (e) { if (e instanceof TypeError) caught++; }
try { remove(undefined, rawKey()); } catch (e) { if (e instanceof TypeError) caught++; }
try { delete null[failKey()]; } catch (e) { if (e === marker) caught++; }
try { delete failBase()[rawKey()]; } catch (e) { if (e === marker) caught++; }
try { delete Object[0][0]; } catch (e) { if (e instanceof TypeError) caught++; }
try { delete null.x; } catch (e) { if (e instanceof TypeError) caught++; }
try { strictRemove(undefined, key); } catch (e) { if (e instanceof TypeError) caught++; }
caught === 9 && trace.join(',') === 'raw,raw,raw,raw,throw,base';
"#,
    );
}

#[test]
fn computed_numeric_keys_are_evaluated_once_for_every_object_dispatch() {
    assert_delete(
        r#"
var calls = 0;
function next() { calls++; return 1; }
function remove(base) { return delete base[next()]; }
function args() { return arguments; }
var array = [1,2,3], object = {1:2}, argumentsObject = args(1,2,3);
var view = new Uint8Array([1,2,3]);
var a = remove(array), b = remove(object), c = remove(argumentsObject), d = remove(view);
a && b && c && !d && calls === 4 && !(1 in array) && !(1 in object) &&
  !(1 in argumentsObject) && view[1] === 2;
"#,
    );
}

#[test]
fn primitive_string_deletion_respects_canonical_indices_and_utf16_length() {
    assert_delete(
        r#"
function remove(base, key) { return delete base[key]; }
function strictRemove(base, key) { 'use strict'; return delete base[key]; }
var text = 'abcde\u{1F600}z', caught = 0;
for (const key of ['0', '5', '6', '7', 'length', ['len', 'gth'].join('')]) {
  if (remove(text, key)) throw 'deleted string own property';
  try { strictRemove(text, key); } catch (e) { if (e instanceof TypeError) caught++; }
}
for (const key of ['8', '100', '-0', '00', '05', '5.0', '-1', '1.5', '4294967295', Symbol()]) {
  if (!remove(text, key) || !strictRemove(text, key)) throw 'absent property';
}
for (const base of [Object(text), [], new Proxy([], {})]) {
  const dynamicLength = ['len', 'gth'].join('');
  if (remove(base, dynamicLength)) throw 'deleted exotic length';
  try { strictRemove(base, dynamicLength); } catch (e) { if (e instanceof TypeError) caught++; }
}
var conversions = 0;
var key = { toString() { conversions++; return 'valueOf'; } };
var values = [true, 1, 1n, 1n << 70n, Symbol()];
for (const value of values) if (!remove(value, key)) throw 'primitive inherited property';
caught === 9 && conversions === 5 && delete 'Test262'[100] && !delete 'abcdef'[5];
"#,
    );
}

#[test]
fn proxy_delete_keeps_symbol_identity_and_strict_invariants() {
    assert_delete(
        r#"
var trace = [], symbol = Symbol(), target = {};
Object.defineProperty(target, 'fixed', { value: 1, configurable: false });
target[symbol] = 2;
var proxy = new Proxy(target, {
  deleteProperty(object, key) {
    trace.push(key === symbol ? 'symbol' : key);
    if (key === 'fixed') return false;
    return Reflect.deleteProperty(object, key);
  },
  get() { throw 'delete must not Get'; }
});
function remove(base, key) { return delete base[key]; }
function strictRemove(base, key) { 'use strict'; return delete base[key]; }
var first = remove(proxy, symbol), second = remove(proxy, 'fixed'), caught = false;
try { strictRemove(proxy, 'fixed'); } catch (e) { caught = e instanceof TypeError; }
first && !second && caught && !(symbol in target) && target.fixed === 1 &&
  trace.join(',') === 'symbol,fixed,fixed';
"#,
    );
}

#[test]
fn with_delete_selects_the_binding_without_reading_its_value() {
    assert_delete(
        r#"
var scope = {};
Object.defineProperty(scope, 'x', { get() { throw 'GetValue'; }, configurable: true });
var removed;
with (scope) { removed = delete (((x))); }
var inner = {x: 2}, outer = {x: 3};
var nested;
with (outer) { with (inner) { nested = delete x; } }
var lexical;
with (outer) { let x = 4; lexical = delete x; }
function capture(object) { with (object) { return function() { return delete (x); }; } }
var captured = capture(outer)();
removed && !('x' in scope) && nested && !('x' in inner) && !lexical && captured && !('x' in outer);
"#,
    );
}

#[test]
fn with_delete_preserves_unscopables_order_and_abrupt_completion() {
    assert_delete(
        r#"
var trace = [], marker = {}, target = {x: 1};
var proxy = new Proxy(target, {
  has(object, key) { if (key === 'x') trace.push('has'); return Reflect.has(object, key); },
  get(object, key) {
    if (key === Symbol.unscopables) { trace.push('unscopables'); return { get x() { trace.push('blocked'); return false; } }; }
    throw 'GetValue';
  },
  deleteProperty(object, key) { trace.push('delete'); return Reflect.deleteProperty(object, key); }
});
function remove(object) { with (object) { return delete x; } }
var removed = remove(proxy), received;
var abrupt = { x: 1, get [Symbol.unscopables]() { throw marker; } };
try { remove(abrupt); } catch (e) { received = e; }
removed && !('x' in target) && received === marker && abrupt.x === 1 &&
  trace.join(',') === 'has,unscopables,blocked,delete';
"#,
    );
}

#[test]
fn with_selected_delete_does_not_mark_the_global_fallback_absent() {
    assert_delete(
        r#"
globalThis.deleteFallback = 17;
var scope = {deleteFallback: 19};
var first;
with (scope) { first = delete deleteFallback; }
var retained = deleteFallback;
var blocked = {deleteFallback: 21, [Symbol.unscopables]: {deleteFallback: true}};
var second;
with (blocked) { second = delete deleteFallback; }
var creating = { createdDuringResolution: 1, get [Symbol.unscopables]() {
  globalThis.createdDuringResolution = 23;
  return {createdDuringResolution: true};
} };
var third;
with (creating) { third = delete createdDuringResolution; }
first && second && third && !('createdDuringResolution' in globalThis) && retained === 17 && !('deleteFallback' in scope) &&
  blocked.deleteFallback === 21 && !('deleteFallback' in globalThis);
"#,
    );
}

#[test]
fn parenthesized_eval_visible_identifiers_and_nonreferences_keep_their_semantics() {
    assert_delete(
        r#"
function remove(object) {
  eval('var local = 1');
  var localRemoved = delete (((local)));
  with (object) { return localRemoved && delete (((x))); }
}
var object = {x: 1}, calls = 0, marker = {}, caught;
function value() { calls++; return object; }
function fail() { throw marker; }
var nonreference = delete (0, object.x);
try { delete fail(); } catch (e) { caught = e; }
var removed = remove(object);
removed && nonreference && delete value() && calls === 1 && caught === marker && !('x' in object);
"#,
    );
}
