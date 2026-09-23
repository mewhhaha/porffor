//! EvaluateGeneratorBody / EvaluateAsyncGeneratorBody create the instance with
//! `OrdinaryCreateFromConstructor(functionObject, default)` after
//! FunctionDeclarationInstantiation. The instance prototype is therefore read
//! from `functionObject.prototype` at call time, after parameter
//! initialization, and a non-Object value selects the intrinsic of the
//! generator function's own realm.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_all_true(source: &str, host_surface_policy: HostSurfacePolicy, expected_lines: usize) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("generator instance prototype selection executes in Wasm");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    let lines = outcome
        .output_events
        .iter()
        .map(|event| match event {
            HostOutputEvent::PrintLine(line) => line.as_str(),
            other => panic!("unexpected host output event: {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), expected_lines, "{lines:?}");
    for (index, line) in lines.iter().enumerate() {
        assert_eq!(*line, "true", "check {index} failed: {lines:?}");
    }
}

#[test]
fn generator_instances_read_the_function_prototype_at_call_time() {
    assert_all_true(
        r#"
var generatorPrototype = Object.getPrototypeOf(function* () {}).prototype;
var asyncGeneratorPrototype = Object.getPrototypeOf(async function* () {}).prototype;

function* replaced() { yield 1; }
var original = replaced.prototype;
var replacement = { marker: 1 };
replaced.prototype = replacement;
var replacedInstance = replaced();
print(Object.getPrototypeOf(replacedInstance) === replacement && original !== replacement);
// The instance is still a genuine generator even though it no longer
// inherits `next`.
print(generatorPrototype.next.call(replacedInstance).value === 1);

async function* asyncReplaced() {}
var asyncReplacement = {};
asyncReplaced.prototype = asyncReplacement;
print(Object.getPrototypeOf(asyncReplaced()) === asyncReplacement);

function* nulled() {}
nulled.prototype = null;
print(Object.getPrototypeOf(nulled()) === generatorPrototype);

async function* asyncNulled() {}
asyncNulled.prototype = null;
print(Object.getPrototypeOf(asyncNulled()) === asyncGeneratorPrototype);

var primitives = [undefined, 0, "text", true, Symbol("s"), 1n];
var primitiveFallbacks = true;
for (var index = 0; index < primitives.length; index++) {
  var primitive = primitives[index];
  var sync = function* () {};
  var asyncSync = async function* () {};
  sync.prototype = primitive;
  asyncSync.prototype = primitive;
  primitiveFallbacks = primitiveFallbacks
    && Object.getPrototypeOf(sync()) === generatorPrototype
    && Object.getPrototypeOf(asyncSync()) === asyncGeneratorPrototype;
}
print(primitiveFallbacks);

function* definedPrimitive() {}
Object.defineProperty(definedPrimitive, "prototype", { value: 7 });
print(Object.getPrototypeOf(definedPrimitive()) === generatorPrototype);

function* callablePrototype() {}
var callable = function () {};
callablePrototype.prototype = callable;
var callableInstance = callablePrototype();
print(Object.getPrototypeOf(callableInstance) === callable && typeof callableInstance === "object");

class Holder {
  *method() {}
  static *staticMethod() {}
  async *asyncMethod() {}
}
var holder = new Holder();
var methodPrototype = {};
Holder.prototype.method.prototype = methodPrototype;
Holder.staticMethod.prototype = null;
Holder.prototype.asyncMethod.prototype = 3;
print(Object.getPrototypeOf(holder.method()) === methodPrototype);
print(Object.getPrototypeOf(Holder.staticMethod()) === generatorPrototype);
print(Object.getPrototypeOf(holder.asyncMethod()) === asyncGeneratorPrototype);

var literal = { *method() {}, async *asyncMethod() {} };
var literalPrototype = {};
literal.method.prototype = literalPrototype;
literal.asyncMethod.prototype = null;
print(Object.getPrototypeOf(literal.method()) === literalPrototype);
print(Object.getPrototypeOf(literal.asyncMethod()) === asyncGeneratorPrototype);
"#,
        HostSurfacePolicy::Product,
        13,
    );
}

#[test]
fn generator_instance_prototype_follows_parameter_initialization() {
    assert_all_true(
        r#"
var generatorPrototype = Object.getPrototypeOf(function* () {}).prototype;
var asyncGeneratorPrototype = Object.getPrototypeOf(async function* () {}).prototype;

var effects = "";
var installed = { marker: "during" };
function* during(value = (effects += "p", during.prototype = installed), ...rest) {
  effects += "b";
  yield rest.length;
}
var duringInstance = during(undefined, 1, 2);
print(effects === "p" && Object.getPrototypeOf(duringInstance) === installed);
print(generatorPrototype.next.call(duringInstance).value === 2 && effects === "pb");

function* topology(value = (topology.prototype = null)) { yield value; }
print(Object.getPrototypeOf(topology()) === generatorPrototype);

var bodyStarted = false;
var stream = async function* (value = (stream.prototype = null)) { bodyStarted = true; };
var oldPrototype = stream.prototype;
var streamInstance = stream();
print(Object.getPrototypeOf(streamInstance) === asyncGeneratorPrototype
  && Object.getPrototypeOf(streamInstance) !== oldPrototype
  && !bodyStarted);

var asyncInstalled = {};
async function* asyncDuring([first], { second } = (asyncDuring.prototype = asyncInstalled, { second: 2 })) {}
print(Object.getPrototypeOf(asyncDuring([1])) === asyncInstalled);

var abruptBody = false;
var abruptPrototype = {};
function* abrupt(value = (abrupt.prototype = abruptPrototype, (() => { throw "sync"; })())) {
  abruptBody = true;
}
var syncThrown;
try { abrupt(); } catch (error) { syncThrown = error; }
print(syncThrown === "sync" && !abruptBody && abrupt.prototype === abruptPrototype);

var asyncAbruptBody = false;
async function* asyncAbrupt(value = (() => { throw "async"; })()) { asyncAbruptBody = true; }
var asyncThrown;
try { asyncAbrupt(); } catch (error) { asyncThrown = error; }
print(asyncThrown === "async" && !asyncAbruptBody);
"#,
        HostSurfacePolicy::Product,
        7,
    );
}

#[test]
fn generator_instance_prototype_fallback_uses_the_generator_function_realm() {
    assert_all_true(
        r#"
var other = __lilaCreateRealm().global;
var target = other.Function();
target.prototype = null;
var GeneratorFunction = Object.getPrototypeOf(function* () {}).constructor;
var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;
var OtherGeneratorFunction = Object.getPrototypeOf(Reflect.construct(GeneratorFunction, [], target)).constructor;
var OtherAsyncGeneratorFunction = Object.getPrototypeOf(Reflect.construct(AsyncGeneratorFunction, [], target)).constructor;
var generatorPrototype = GeneratorFunction.prototype.prototype;
var asyncGeneratorPrototype = AsyncGeneratorFunction.prototype.prototype;
var otherGeneratorPrototype = OtherGeneratorFunction.prototype.prototype;
var otherAsyncGeneratorPrototype = OtherAsyncGeneratorFunction.prototype.prototype;
print(otherGeneratorPrototype !== generatorPrototype && otherAsyncGeneratorPrototype !== asyncGeneratorPrototype);

var foreign = new OtherGeneratorFunction();
foreign.prototype = null;
print(Object.getPrototypeOf(foreign()) === otherGeneratorPrototype);

var foreignAsync = new OtherAsyncGeneratorFunction();
foreignAsync.prototype = 1;
print(Object.getPrototypeOf(foreignAsync()) === otherAsyncGeneratorPrototype);

// The function is created by the entry GeneratorFunction, so its [[Realm]] is
// the entry realm even though its [[Prototype]] came from the foreign newTarget.
var entryRealmFunction = Reflect.construct(GeneratorFunction, [], target);
entryRealmFunction.prototype = null;
print(Object.getPrototypeOf(entryRealmFunction()) === generatorPrototype);
"#,
        HostSurfacePolicy::Test262,
        4,
    );
}
