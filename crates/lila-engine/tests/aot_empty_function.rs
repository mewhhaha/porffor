use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn run_boolean(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("empty Function construction should execute through Wasm");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn empty_function_allocates_an_ordinary_callable_with_correct_descriptors() {
    run_boolean(
        r#"
var called = Function();
var constructed = new Function();
var forwarded = Function.call(null);
var length = Object.getOwnPropertyDescriptor(called, "length");
var name = Object.getOwnPropertyDescriptor(called, "name");
var prototype = Object.getOwnPropertyDescriptor(called, "prototype");
var instance = new called();
called !== constructed && called !== forwarded
  && called() === undefined && constructed() === undefined && forwarded() === undefined
  && typeof instance === "object" && Object.getPrototypeOf(instance) === called.prototype
  && called.prototype.constructor === called
  && called.name === "anonymous" && called.length === 0
  && Function.prototype.toString.call(called) === "function anonymous(\n) {\n\n}"
  && length.writable === false && length.enumerable === false && length.configurable === true
  && name.writable === false && name.enumerable === false && name.configurable === true
  && prototype.writable === true && prototype.enumerable === false && prototype.configurable === false;
"#,
    );
}

#[test]
fn empty_function_preserves_the_active_realm_for_instances_and_array_construction() {
    run_boolean(
        r#"
var other = __lilaCreateRealm().global;
var called = other.Function();
var constructed = new other.Function();
var originalPrototype = called.prototype;
called.prototype = null;
constructed.prototype = null;
var from = Array.from.call(constructed, []);
var of = Array.of.call(called);
Object.getPrototypeOf(called) === other.Function.prototype
  && Object.getPrototypeOf(constructed) === other.Function.prototype
  && Object.getPrototypeOf(originalPrototype) === other.Object.prototype
  && typeof new called() === "object"
  && Object.getPrototypeOf(new called()) === other.Object.prototype
  && Object.getPrototypeOf(new constructed()) === other.Object.prototype
  && Object.getPrototypeOf(from) === other.Object.prototype && from.length === 0
  && Object.getPrototypeOf(of) === other.Object.prototype && of.length === 0;
"#,
    );
}

#[test]
fn empty_function_separates_new_target_prototype_from_active_constructor_realm() {
    run_boolean(
        r#"
var other = __lilaCreateRealm().global;
var target = other.Function();
target.prototype = null;
var created = Reflect.construct(Function, [], target);
var reads = 0;
var sentinel = {};
var proxy = new Proxy(target, {
  get: function (target, key) {
    if (key === "prototype") {
      reads++;
      throw sentinel;
    }
    return target[key];
  }
});
var caught;
try { Reflect.construct(Function, [], proxy); } catch (error) { caught = error; }
Object.getPrototypeOf(created) === other.Function.prototype
  && Object.getPrototypeOf(created.prototype) === Object.prototype
  && created() === undefined && reads === 1 && caught === sentinel;
"#,
    );
}
