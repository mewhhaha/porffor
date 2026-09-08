use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_stack_realm_trace(source: &str, expected: &[&str]) {
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
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("cross-realm AsyncDisposableStack methods execute in Wasm");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
            .collect::<Vec<_>>(),
        "source:\n{source}",
    );
}

#[test]
fn disposal_capabilities_follow_the_method_realm_for_empty_invalid_and_awaited_receivers() {
    assert_stack_realm_trace(
        r#"
var other = __lilaCreateRealm().global;
var ForeignPromise = other.Promise;
var dispose = other.AsyncDisposableStack.prototype.disposeAsync;
other.Promise = function () { throw "must use intrinsic Promise"; };
var empty = dispose.call(new AsyncDisposableStack());
var invalid = dispose.call({});
var stack = new AsyncDisposableStack();
stack.defer(function () {
  return { then: function (resolve) {
    print(Object.getPrototypeOf(resolve) === other.Function.prototype);
    resolve();
  } };
});
var awaited = dispose.call(stack);
print(empty instanceof ForeignPromise && invalid instanceof ForeignPromise && awaited instanceof ForeignPromise);
var local = AsyncDisposableStack.prototype.disposeAsync.call(new other.AsyncDisposableStack());
print(local instanceof Promise && !(local instanceof ForeignPromise));
empty.then(function (value) { print("empty:" + value); });
invalid.then(function () { print("invalid fulfilled"); }, function (error) {
  print(error instanceof other.TypeError && !(error instanceof TypeError));
});
awaited.then(function (value) { print("awaited:" + value); });
void 0;
"#,
        &[
            "true",
            "true",
            "true",
            "empty:undefined",
            "true",
            "awaited:undefined",
        ],
    );
}

#[test]
fn moved_stacks_and_synchronous_errors_follow_the_borrowed_method_realm() {
    assert_stack_realm_trace(
        r#"
var other = __lilaCreateRealm().global;
var methods = other.AsyncDisposableStack.prototype;
var stack = new AsyncDisposableStack();
var calls = 0;
stack.defer(function () { calls++; });
Object.defineProperty(stack, "constructor", { get: function () { throw "must not read constructor"; } });
var moved = methods.move.call(stack);
print(Object.getPrototypeOf(moved) === methods && moved !== stack);
print(stack.disposed && !moved.disposed && calls === 0);
var local = AsyncDisposableStack.prototype.move.call(new other.AsyncDisposableStack());
print(Object.getPrototypeOf(local) === AsyncDisposableStack.prototype);
try { methods.use.call(stack, null); } catch (error) {
  print(error instanceof other.ReferenceError && !(error instanceof ReferenceError));
}
try { methods.defer.call(new AsyncDisposableStack(), null); } catch (error) {
  print(error instanceof other.TypeError && !(error instanceof TypeError));
}
var getter = Object.getOwnPropertyDescriptor(methods, "disposed").get;
try { getter.call({}); } catch (error) { print(error instanceof other.TypeError); }
moved.disposeAsync().then(function () { print("calls:" + calls); });
void 0;
"#,
        &["true", "true", "true", "true", "true", "true", "calls:1"],
    );
}

#[test]
fn suppression_keeps_the_disposal_method_realm_across_rejected_awaits() {
    assert_stack_realm_trace(
        r#"
var other = __lilaCreateRealm().global;
var ForeignSuppressedError = other.SuppressedError;
other.SuppressedError = function () { throw "must use intrinsic SuppressedError"; };
var first = {};
var second = {};
var third = {};
var order = "";
var stack = new AsyncDisposableStack();
stack.defer(function () { order += "1"; return Promise.reject(first); });
stack.defer(function () { order += "2"; throw second; });
stack.defer(function () { order += "3"; return Promise.reject(third); });
var result = other.AsyncDisposableStack.prototype.disposeAsync.call(stack);
print(result instanceof other.Promise);
result.then(function () { print("unexpected fulfillment"); }, function (error) {
  print(order);
  print(error instanceof ForeignSuppressedError && !(error instanceof SuppressedError));
  print(error.error === first);
  print(error.suppressed instanceof ForeignSuppressedError);
  print(error.suppressed.error === second && error.suppressed.suppressed === third);
});
void 0;
"#,
        &["true", "321", "true", "true", "true", "true"],
    );
}

#[test]
fn synchronous_fallback_discards_returns_and_keeps_its_acquisition_realm() {
    assert_stack_realm_trace(
        r#"
var other = __lilaCreateRealm().global;
var realmReads = 0;
Object.defineProperty(other.Promise.prototype, "constructor", {
  configurable: true,
  get: function () { realmReads++; return other.Promise; }
});
var returnedThenReads = 0;
var poison = { get then() { returnedThenReads++; throw "must discard return"; } };
var rejected = Promise.reject("ignored returned rejection");
rejected.catch(function () {});
Object.defineProperty(rejected, "then", {
  get: function () { returnedThenReads++; throw "must discard returned Promise"; }
});
var calls = 0;
var receiverMatches = false;
var asyncAwaits = 0;
var fallbackReads = 0;
var stack = new AsyncDisposableStack();
stack.use({
  [Symbol.asyncDispose]: function () {
    return { then: function (resolve) { asyncAwaits++; resolve(); } };
  },
  get [Symbol.dispose]() { fallbackReads++; throw "must prefer async method"; }
});
var use = other.AsyncDisposableStack.prototype.use;
use.call(stack, { [Symbol.dispose]: function () { calls++; return poison; } });
var resource = { [Symbol.dispose]: function () {
  calls++;
  receiverMatches = this === resource;
  return rejected;
} };
use.call(stack, resource);
print(realmReads === 0);
stack.disposeAsync().then(function () {
  print(calls + ":" + returnedThenReads + ":" + asyncAwaits + ":" + fallbackReads + ":" + receiverMatches);
  print(realmReads > 0);
}, function () { print("unexpected rejection"); });
void 0;
"#,
        &["true", "2:0:1:0:true", "true"],
    );
}

#[test]
fn synchronous_fallback_throws_suspend_before_the_next_disposer() {
    assert_stack_realm_trace(
        r#"
var marker = {};
var order = "";
var stack = new AsyncDisposableStack();
stack.defer(function () { order += "last;"; });
stack.use({ [Symbol.dispose]: function () { order += "throw;"; throw marker; } });
var result = stack.disposeAsync();
print(order === "throw;");
result.then(function () { print("unexpected fulfillment"); }, function (error) {
  print(error === marker && order === "throw;last;");
});
void 0;
"#,
        &["true", "true"],
    );
}

#[test]
fn null_and_undefined_resources_preserve_the_explicit_await() {
    assert_stack_realm_trace(
        r#"
function check(value) {
  var stack = new AsyncDisposableStack();
  var sequence = [];
  stack.use(value);
  return Promise.all([
    Promise.resolve().then(function () {}).then(function () { sequence.push("before"); }),
    stack.disposeAsync().then(function () { sequence.push("dispose"); }),
    Promise.resolve().then(function () {}).then(function () { sequence.push("after"); })
  ]).then(function () { print(sequence.join(",")); });
}
check(null).then(function () { return check(undefined); });
void 0;
"#,
        &["before,dispose,after", "before,dispose,after"],
    );
}
