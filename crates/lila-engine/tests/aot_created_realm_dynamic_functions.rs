use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

#[test]
fn created_realms_own_distinct_dynamic_function_and_iterator_intrinsics() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
var first = __lilaCreateRealm().global;
var second = __lilaCreateRealm().global;
var target1 = first.Function();
var target2 = second.Function();
target1.prototype = null;
target2.prototype = null;
var G = (function* () {}).constructor;
var AG = (async function* () {}).constructor;
var firstG = Object.getPrototypeOf(Reflect.construct(G, [], target1)).constructor;
var secondG = Object.getPrototypeOf(Reflect.construct(G, [], target2)).constructor;
var firstAG = Object.getPrototypeOf(Reflect.construct(AG, [], target1)).constructor;
print(firstG !== secondG && firstG !== G && secondG !== G);
print(Object.getPrototypeOf(firstG) === first.Function && Object.getPrototypeOf(secondG) === second.Function);
print(Object.getPrototypeOf(firstG.prototype) === first.Function.prototype);
print(Object.getPrototypeOf(secondG.prototype) === second.Function.prototype);
var generatorPrototype = firstG.prototype.prototype;
var asyncGeneratorPrototype = firstAG.prototype.prototype;
var asyncIteratorPrototype = Object.getPrototypeOf(asyncGeneratorPrototype);
print(Object.getPrototypeOf(generatorPrototype) === first.Iterator.prototype);
print(Object.getPrototypeOf(asyncIteratorPrototype) === first.Object.prototype);
print(Object.getPrototypeOf(generatorPrototype.next) === first.Function.prototype);
print(Object.getPrototypeOf(asyncGeneratorPrototype.next) === first.Function.prototype);
print(generatorPrototype.next !== secondG.prototype.prototype.next);
print(generatorPrototype.constructor === firstG.prototype && asyncGeneratorPrototype.constructor === firstAG.prototype);
var descriptor = Object.getOwnPropertyDescriptor(firstG.prototype, "constructor");
print(!descriptor.writable && !descriptor.enumerable && descriptor.configurable);
print(generatorPrototype[Symbol.toStringTag] === "Generator" && asyncGeneratorPrototype[Symbol.toStringTag] === "AsyncGenerator");
var receiver = {};
print(asyncIteratorPrototype[Symbol.asyncIterator].call(receiver) === receiver);
print(Object.getPrototypeOf(asyncIteratorPrototype[Symbol.asyncIterator]) === first.Function.prototype);
print(Object.getPrototypeOf(G.prototype) === Function.prototype && Object.getPrototypeOf(AG.prototype) === Function.prototype);
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
        .expect("created-realm intrinsic graph executes in Wasm");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string()); 15],
    );
}

#[test]
fn foreign_async_dispose_promises_use_the_method_realm_for_every_completion() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
var other = __lilaCreateRealm().global;
var target = other.Function();
target.prototype = null;
var AG = (async function* () {}).constructor;
var foreignAG = Object.getPrototypeOf(Reflect.construct(AG, [], target)).constructor;
var asyncIteratorPrototype = Object.getPrototypeOf(foreignAG.prototype.prototype);
var dispose = asyncIteratorPrototype[Symbol.asyncDispose];
var ForeignPromise = other.Promise;
var EntryPromise = Promise;
other.Promise = function () { throw 'foreign global Promise was read'; };
globalThis.Promise = function () { throw 'entry global Promise was read'; };
var marker = {};
var calls = 0;
var absent = dispose.call({});
var asynchronous = dispose.call({
  return() { calls++; return { then(resolve) { calls++; resolve(19); } }; }
});
var rejected = dispose.call({ return() { calls++; throw marker; } });
var invalid = dispose.call(null);
print(absent instanceof ForeignPromise && asynchronous instanceof ForeignPromise);
print(rejected instanceof ForeignPromise && invalid instanceof ForeignPromise);
print(!(absent instanceof EntryPromise) && Object.getPrototypeOf(asynchronous) === ForeignPromise.prototype);
EntryPromise.all([
  absent.then(function (value) { return value === undefined; }),
  asynchronous.then(function (value) { return value === undefined; }),
  rejected.then(function () { return false; }, function (error) { return error === marker; }),
  invalid.then(function () { return false; }, function (error) {
    return error instanceof other.TypeError && !(error instanceof TypeError);
  })
]).then(function (results) {
  print(results.join(':'));
  print('calls:' + calls);
});
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
        .expect("foreign asyncDispose executes in Wasm");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        ["true", "true", "true", "true:true:true:true", "calls:3"]
            .map(|line| HostOutputEvent::PrintLine(line.to_string()))
            .to_vec(),
    );
}
