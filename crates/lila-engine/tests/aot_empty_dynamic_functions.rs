use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_empty_function_trace(source: &str, expected: &[&str]) {
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
        .expect("empty derived function must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn empty_generator_functions_are_fresh_nonconstructable_callable_generators() {
    assert_empty_function_trace(
        r#"
var C = (function* () {}).constructor;
print(Object.getPrototypeOf(C) === Function && Object.getPrototypeOf(C.prototype) === Function.prototype);
var left = C();
var right = new C();
var forwarded = C.call(null);
var descriptor = Object.getOwnPropertyDescriptor(left, "prototype");
print(left !== right && left !== forwarded && left.prototype !== right.prototype);
print(Object.getPrototypeOf(left) === C.prototype);
print(Object.getPrototypeOf(left.prototype) === C.prototype.prototype);
print(left.name + ":" + left.length + ":" + descriptor.writable + ":" + descriptor.enumerable + ":" + descriptor.configurable);
print(Function.prototype.toString.call(left) === "function* anonymous(\n) {\n\n}");
var iterator = left();
print(Object.getPrototypeOf(iterator) === left.prototype);
var first = iterator.next();
var second = iterator.next();
print(first.done + ":" + first.value + ":" + second.done + ":" + second.value);
var returned = right().return(91);
print(returned.value + ":" + returned.done);
var marker = {};
try { forwarded().throw(marker); } catch (error) { print(error === marker); }
try { new left(); } catch (error) { print(error instanceof TypeError); }
"#,
        &[
            "true",
            "true",
            "true",
            "true",
            "anonymous:0:true:false:false",
            "true",
            "true",
            "true:undefined:true:undefined",
            "91:true",
            "true",
            "true",
        ],
    );
}

#[test]
fn empty_async_functions_return_fresh_promises_and_have_no_instance_prototype() {
    assert_empty_function_trace(
        r#"
var C = (async function () {}).constructor;
var left = C();
var right = new C();
print(left !== right && Object.getPrototypeOf(left) === C.prototype && Object.getPrototypeOf(C.prototype) === Function.prototype);
print(left.name + ":" + left.length + ":" + Object.prototype.hasOwnProperty.call(left, "prototype"));
print(Function.prototype.toString.call(left) === "async function anonymous(\n) {\n\n}");
var first = left();
var second = right();
print(first !== second && first instanceof Promise && second instanceof Promise);
try { new left(); } catch (error) { print(error instanceof TypeError); }
first.then(function (value) { print("first:" + value); });
second.then(function (value) { print("second:" + value); });
void 0;
"#,
        &[
            "true",
            "anonymous:0:false",
            "true",
            "true",
            "true",
            "first:undefined",
            "second:undefined",
        ],
    );
}

#[test]
fn empty_async_generator_functions_execute_next_return_and_throw_requests() {
    assert_empty_function_trace(
        r#"
var C = (async function* () {}).constructor;
var left = C();
var right = new C();
var descriptor = Object.getOwnPropertyDescriptor(left, "prototype");
print(left !== right && left.prototype !== right.prototype && Object.getPrototypeOf(C.prototype) === Function.prototype);
print(Object.getPrototypeOf(left) === C.prototype);
print(Object.getPrototypeOf(left.prototype) === C.prototype.prototype);
print(left.name + ":" + left.length + ":" + descriptor.writable + ":" + descriptor.enumerable + ":" + descriptor.configurable);
print(Function.prototype.toString.call(left) === "async function* anonymous(\n) {\n\n}");
var iterator = left();
print(Object.getPrototypeOf(iterator) === left.prototype);
try { new left(); } catch (error) { print(error instanceof TypeError); }
iterator.next().then(function (result) {
  print("next:" + result.value + ":" + result.done);
  return right().return(Promise.resolve(91));
}).then(function (result) {
  print("return:" + result.value + ":" + result.done);
  var marker = {};
  return left().throw(marker).then(function () { print("wrong resolution"); }, function (error) {
    print("throw:" + (error === marker));
  });
});
void 0;
"#,
        &[
            "true",
            "true",
            "true",
            "anonymous:0:true:false:false",
            "true",
            "true",
            "true",
            "next:undefined:true",
            "return:91:true",
            "throw:true",
        ],
    );
}

#[test]
fn empty_derived_functions_separate_active_realm_from_new_target_prototype() {
    assert_empty_function_trace(
        r#"
var other = __lilaCreateRealm().global;
var target = other.Function();
target.prototype = null;
var G = (function* () {}).constructor;
var A = (async function () {}).constructor;
var AG = (async function* () {}).constructor;
var transportedG = Reflect.construct(G, [], target);
var transportedA = Reflect.construct(A, [], target);
var transportedAG = Reflect.construct(AG, [], target);
var foreignG = Object.getPrototypeOf(transportedG).constructor;
var foreignA = Object.getPrototypeOf(transportedA).constructor;
var foreignAG = Object.getPrototypeOf(transportedAG).constructor;
print(Object.getPrototypeOf(foreignG) === other.Function && Object.getPrototypeOf(foreignG.prototype) === other.Function.prototype);
print(Object.getPrototypeOf(foreignA) === other.Function && Object.getPrototypeOf(foreignA.prototype) === other.Function.prototype);
print(Object.getPrototypeOf(foreignAG) === other.Function && Object.getPrototypeOf(foreignAG.prototype) === other.Function.prototype);
print(Object.getPrototypeOf(transportedG.prototype) === G.prototype.prototype);
print(Object.getPrototypeOf(transportedAG.prototype) === AG.prototype.prototype);
var generator = foreignG();
var asyncFunction = foreignA();
var asyncGenerator = foreignAG();
print(Object.getPrototypeOf(generator.prototype) === foreignG.prototype.prototype);
print(Object.getPrototypeOf(asyncGenerator.prototype) === foreignAG.prototype.prototype);
print(asyncFunction() instanceof other.Promise);
print(asyncGenerator().next() instanceof other.Promise);
var result = generator().next();
print(result.done && result.value === undefined);
var customPrototype = {};
function CustomTarget() {}
CustomTarget.prototype = customPrototype;
print(Object.getPrototypeOf(Reflect.construct(G, [], CustomTarget)) === customPrototype);
var sentinel = {};
var reads = 0;
var proxy = new Proxy(CustomTarget, {
  get: function (target, key) {
    if (key === "prototype") { reads++; throw sentinel; }
    return target[key];
  }
});
try { Reflect.construct(AG, [], proxy); } catch (error) { print(reads === 1 && error === sentinel); }
void 0;
"#,
        &[
            "true", "true", "true", "true", "true", "true", "true", "true", "true", "true", "true",
            "true",
        ],
    );
}
