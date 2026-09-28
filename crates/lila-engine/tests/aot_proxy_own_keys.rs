use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
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
        .expect("Proxy ownKeys must execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}: {}\n{source}",
        outcome.completion,
        outcome.note,
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "{source}",
    );
}

#[test]
fn target_checks_and_list_getters_are_observable() {
    assert_trace(
        include_str!("fixtures/proxy-own-keys-observations.js"),
        &[
            "outer extensible keys desc:a desc:b TypeError",
            "true",
            "0 1 2 true",
            "true",
        ],
    );
}

#[test]
fn list_conversion_finishes_before_duplicate_checks_and_preserves_abrupt_values() {
    assert_trace(
        r#"
function keysFrom(keys) {
  return Reflect.ownKeys(new Proxy({}, {ownKeys() { return keys; }}));
}
var symbol = Symbol('inherited');
var gets = 0;
var hole = new Array(1);
Object.setPrototypeOf(hole, {get 0() { gets++; return symbol; }});
print(keysFrom(hole)[0] === symbol, gets);
var mutable = ['first', 'second'];
Object.defineProperty(mutable, '1', {get() { mutable[0] = 'changed'; mutable.push('ignored'); return 'second'; }});
print(keysFrom(mutable).join(' '));
var sentinel = {};
var seen = [];
var duplicate = {length: 3, get 0() { seen.push('0'); return 'same'; },
  get 1() { seen.push('1'); return 'same'; }, get 2() { seen.push('2'); throw sentinel; }};
try { keysFrom(duplicate); }
catch (error) { print(error === sentinel, seen.join(' ')); }
seen = [];
var invalid = {length: 2, get 0() { seen.push('0'); return 1; }, get 1() { seen.push('1'); return 'key'; }};
try { keysFrom(invalid); }
catch (error) { print(error instanceof TypeError, seen.join(' ')); }
var huge = {length: 9007199254740991, get 0() { throw sentinel; }};
try { keysFrom(huge); }
catch (error) { print(error === sentinel); }
var lengthGets = 0;
try { keysFrom({get length() { lengthGets++; throw sentinel; }}); }
catch (error) { print(error === sentinel, lengthGets); }
var lookups = [];
var observed = new Proxy(['a'], {get(target, key, receiver) { lookups.push(key); return Reflect.get(target, key, receiver); }});
print(keysFrom(observed).join(' '), lookups.join(' '));
"#,
        &[
            "true 1",
            "first second",
            "true 0 1 2",
            "true 0",
            "true",
            "true 1",
            "a length 0",
        ],
    );
}

#[test]
fn all_descriptor_observations_precede_invariant_errors() {
    assert_trace(
        r#"
var base = {};
Object.defineProperty(base, 'a', {value: 1});
Object.defineProperty(base, 'b', {value: 2});
var stops = ['extensible', 'keys', 'descriptor'];
for (var i = 0; i < stops.length; i++) {
  var stop = stops[i];
  var sentinel = {};
  var trace = [];
  var target = new Proxy(base, {
    isExtensible(t) { trace.push('extensible'); if (stop === 'extensible') throw sentinel; return true; },
    ownKeys(t) { trace.push('keys'); if (stop === 'keys') throw sentinel; return ['a', 'b']; },
    getOwnPropertyDescriptor(t, key) {
      trace.push(key);
      if (stop === 'descriptor' && key === 'b') throw sentinel;
      return Reflect.getOwnPropertyDescriptor(t, key);
    }
  });
  try { Reflect.ownKeys(new Proxy(target, {ownKeys() { return []; }})); }
  catch (error) { print(error === sentinel, trace.join(' ')); }
}
var revocable = Proxy.revocable({}, {});
revocable.revoke();
try { Reflect.ownKeys(new Proxy(revocable.proxy, {ownKeys() { return []; }})); }
catch (error) { print(error instanceof TypeError); }
"#,
        &[
            "true extensible",
            "true extensible keys",
            "true extensible keys a b",
            "true",
        ],
    );
}

#[test]
fn exotic_targets_and_nested_proxies_require_exact_keys_when_non_extensible() {
    assert_trace(
        r#"
function rejects(target, keys) {
  try { Reflect.ownKeys(new Proxy(target, {ownKeys() { return keys; }})); }
  catch (error) { return error instanceof TypeError; }
  return false;
}
var targets = [new Uint8Array(2), new String('ab'), new Map(), function() {}];
for (var i = 0; i < targets.length; i++) {
  var target = targets[i];
  var keys = Reflect.ownKeys(target);
  print(rejects(target, keys.concat(['extra'])) === false);
  Object.preventExtensions(target);
  print(rejects(target, keys) === false, rejects(target, keys.concat(['extra'])));
  var nested = new Proxy(target, {});
  print(rejects(nested, keys) === false, rejects(nested, keys.concat(['extra'])));
  if (keys.length) print(rejects(nested, keys.slice(1)));
}
"#,
        &[
            "true",
            "true true",
            "true true",
            "true",
            "true",
            "true true",
            "true true",
            "true",
            "true",
            "true true",
            "true true",
            "true",
            "true true",
            "true true",
            "true",
        ],
    );
}
