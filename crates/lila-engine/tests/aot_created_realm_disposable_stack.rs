use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn created_realms_publish_disposable_stack_and_own_constructor_fallbacks() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
var other = __lilaCreateRealm().global;
var second = __lilaCreateRealm().global;
var C = other.DisposableStack;
var prototype = C.prototype;
print(C !== DisposableStack && C !== second.DisposableStack);
print(Object.getPrototypeOf(C) === other.Function.prototype);
print(Object.getPrototypeOf(prototype) === other.Object.prototype && prototype.constructor === C);
var descriptor = Object.getOwnPropertyDescriptor(C, 'prototype');
print(!descriptor.writable && !descriptor.enumerable && !descriptor.configurable);
var methodRealms = true;
for (var name of ['adopt', 'defer', 'move', 'use', 'dispose']) {
  var method = Object.getOwnPropertyDescriptor(prototype, name);
  methodRealms = methodRealms && method.writable && !method.enumerable && method.configurable
    && Object.getPrototypeOf(method.value) === other.Function.prototype;
}
print(methodRealms);
print(prototype.dispose === prototype[Symbol.dispose]);
var disposed = Object.getOwnPropertyDescriptor(prototype, 'disposed');
print(disposed.set === undefined && !disposed.enumerable && disposed.configurable
  && Object.getPrototypeOf(disposed.get) === other.Function.prototype);
var stack = new C();
print(Object.getPrototypeOf(stack) === prototype && stack.disposed === false);
print(Object.prototype.toString.call(stack) === '[object DisposableStack]');
var entryMoved = DisposableStack.prototype.move.call(new C());
print(Object.getPrototypeOf(entryMoved) === DisposableStack.prototype);
var foreignMoved = prototype.move.call(new DisposableStack());
print(Object.getPrototypeOf(foreignMoved) === prototype);
var target = other.Function();
var fallback = true;
for (var value of [undefined, null, true, '', Symbol(), 1]) {
  target.prototype = value;
  stack = Reflect.construct(DisposableStack, [], target);
  fallback = fallback && Object.getPrototypeOf(stack) === prototype && stack.disposed === false;
}
print(fallback);
var entryTarget = Function();
entryTarget.prototype = null;
print(Object.getPrototypeOf(Reflect.construct(C, [], entryTarget)) === DisposableStack.prototype);
var custom = {};
target.prototype = custom;
print(Object.getPrototypeOf(Reflect.construct(C, [], target)) === custom);
var reads = 0;
var marker = {};
var proxy = new Proxy(target, { get(target, key) {
  if (key === 'prototype') { reads++; throw marker; }
  return target[key];
} });
try { Reflect.construct(C, [], proxy); } catch (error) { print(reads === 1 && error === marker); }
target.prototype = null;
other.DisposableStack = function () { throw 'mutable global constructor was used'; };
print(Object.getPrototypeOf(Reflect.construct(DisposableStack, [], target)) === prototype);
void 0;
"#;
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
        .expect("foreign DisposableStack construction executes in Wasm");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string()); 16],
    );
}
