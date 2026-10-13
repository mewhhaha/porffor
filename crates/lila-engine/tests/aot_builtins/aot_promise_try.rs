use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_promise_try(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "\"use strict\";\n"] {
        let source = format!(
            "{directive}function require(value, label) {{ if (!value) throw label; }}\n{source}"
        );
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("Promise.try executes through Wasm: {error}"));
        assert!(
            matches!(&observed.completion, ObservedCompletion::Normal(_)),
            "{observed:?}"
        );
        assert_eq!(
            observed.output_events,
            [HostOutputEvent::PrintLine("promise-try:ok".into())]
        );
    }
}

#[test]
fn matching_constructor_promises_keep_identity_without_resolve_lookup() {
    assert_promise_try(
        r#"
var regular = Promise.resolve(11);
var resolve = Promise.resolve;
var constructorCalls = 0;
class SubPromise extends Promise {
  constructor(executor) { constructorCalls++; super(executor); }
}
var derived = SubPromise.resolve(23);
require(constructorCalls === 1, 'sentinel constructor');
Object.defineProperty(Promise, 'resolve', { get: function() { throw 'observable Promise.resolve'; } });
Object.defineProperty(SubPromise, 'resolve', { get: function() { throw 'observable subclass resolve'; } });
require(Promise.try(function() { return regular; }) === regular, 'regular identity');
require(SubPromise.try(function() { return derived; }) === derived, 'subclass identity');
require(constructorCalls === 1, 'no extra subclass construction');
// PromiseResolve can return an existing Promise even for a non-constructor
// Object when that Promise's live constructor property matches the Object.
for (var receiver of [{}, () => {}]) {
  Object.defineProperty(regular, 'constructor', { value: receiver, configurable: true });
  require(Promise.try.call(receiver, function() { return regular; }) === regular, 'Object try identity fast path');
  require(resolve.call(receiver, regular) === regular, 'Object resolve identity fast path');
}
print('promise-try:ok');
"#,
    );
}

#[test]
fn callback_precedes_constructor_and_settlement_on_both_completion_paths() {
    assert_promise_try(
        r#"
var events = [];
var promise = {};
var marker = {};
var resolved;
var rejected;
function Capability(executor) {
  events.push('construct');
  executor(function(value) { 'use strict'; require(this === undefined, 'resolve receiver'); events.push('resolve'); resolved = value; },
           function(value) { 'use strict'; require(this === undefined, 'reject receiver'); events.push('reject'); rejected = value; });
  return promise;
}
var result = Promise.try.call(Capability, function(a, b) {
  'use strict';
  require(this === undefined && arguments.length === 2 && a === 7 && b === 9, 'callback arguments');
  events.push('callback');
  return marker;
}, 7, 9);
require(result === promise && resolved === marker && rejected === undefined, 'fulfilled capability');
require(events.join('|') === 'callback|construct|resolve', 'fulfillment order');
events = [];
resolved = undefined;
result = Promise.try.call(Capability, function() { events.push('callback'); throw marker; });
require(result === promise && rejected === marker && resolved === undefined, 'rejected capability');
require(events.join('|') === 'callback|construct|reject', 'rejection order');
print('promise-try:ok');
"#,
    );
}

#[test]
fn live_constructor_get_and_receiver_validation_follow_callback_effects() {
    assert_promise_try(
        r#"
var promise = Promise.resolve();
var marker = {};
var events = [];
Object.defineProperty(promise, 'constructor', { get: function() { events.push('constructor Get'); throw marker; } });
var caught;
try { Promise.try(function() { events.push('callback'); return promise; }); } catch (error) { caught = error; }
require(caught === marker && events.join('|') === 'callback|constructor Get', 'live abrupt constructor Get');
var count = 0;
for (var receiver of [{}, () => {}]) {
  caught = undefined;
  try { Promise.try.call(receiver, function() { count++; return 1; }); } catch (error) { caught = error; }
  require(caught instanceof TypeError, 'Object constructor rejection');
}
require(count === 2, 'Object callback effects before validation');
for (var receiver of [undefined, null, true, 1, '', Symbol()]) {
  caught = undefined;
  try { Promise.try.call(receiver, function() { count++; }); } catch (error) { caught = error; }
  require(caught instanceof TypeError, 'primitive receiver rejection');
}
require(count === 2, 'primitive callback excluded');
// Borrowed builtins still allocate their synchronous receiver TypeError in the
// defining Realm, independently of the receiver constructor or callback Realm.
var other = __lilaCreateRealm().global;
caught = undefined;
try { other.Promise.try.call(null, function() { count++; }); } catch (error) { caught = error; }
require(caught instanceof other.TypeError && !(caught instanceof TypeError), 'borrowed receiver error Realm');
require(count === 2, 'borrowed primitive receiver excludes callback');
print('promise-try:ok');
"#,
    );
}

#[test]
fn differing_constructors_wrap_and_settlement_throws_remain_synchronous() {
    assert_promise_try(
        r#"
class SubPromise extends Promise {}
var derived = SubPromise.resolve(17);
var result = Promise.try(function() { return derived; });
require(result !== derived && Object.getPrototypeOf(result) === Promise.prototype, 'differing constructor wrapping');
var delivered = false;
result.then(function(value) { require(value === 17, 'wrapped fulfillment'); delivered = true; });
var original = {};
var thrown = {};
var events = [];
function ThrowingConstructor(executor) { events.push('construct'); throw thrown; }
var caught;
try { Promise.try.call(ThrowingConstructor, function() { events.push('callback'); throw original; }); } catch (error) { caught = error; }
require(caught === thrown && events.join('|') === 'callback|construct', 'constructor throw supersedes callback rejection');
function ThrowingSettlement(executor) {
  events.push('construct');
  executor(function() { events.push('resolve'); throw thrown; }, function() { events.push('reject'); throw thrown; });
}
events = [];
caught = undefined;
try { Promise.try.call(ThrowingSettlement, function() { events.push('callback'); return original; }); } catch (error) { caught = error; }
require(caught === thrown && events.join('|') === 'callback|construct|resolve', 'resolve throw');
events = [];
caught = undefined;
try { Promise.try.call(ThrowingSettlement, function() { events.push('callback'); throw original; }); } catch (error) { caught = error; }
require(caught === thrown && events.join('|') === 'callback|construct|reject', 'reject throw');
result.then(function() { require(delivered, 'jobs completed'); print('promise-try:ok'); });
"#,
    );
}
