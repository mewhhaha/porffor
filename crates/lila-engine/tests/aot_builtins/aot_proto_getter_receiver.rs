use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_proto_getter(source: &str, host_surface_policy: HostSurfacePolicy) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| {
            panic!("prototype getter receiver check failed: {error}\n{source}")
        });
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn prototype_getter_rejects_nullish_receivers_and_boxes_other_primitives() {
    assert_proto_getter(
        r#"
var get = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').get;
function rejects(receiver) {
  try { get.call(receiver); return false; }
  catch (error) { return error instanceof TypeError; }
}
var prototype = {};
var object = Object.create(prototype);
rejects(undefined) && rejects(null) &&
  get.call(object) === prototype && get.call(Object.create(null)) === null &&
  get.call(17) === Number.prototype && get.call(false) === Boolean.prototype &&
  get.call('value') === String.prototype && get.call(Symbol()) === Symbol.prototype &&
  get.call(1n) === BigInt.prototype && get.call(1n << 70n) === BigInt.prototype;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn prototype_getter_preserves_proxy_throws_and_uses_its_defining_error_realm() {
    assert_proto_getter(
        r#"
var other = __lilaCreateRealm().global;
var descriptor = Object.getOwnPropertyDescriptor(other.Object.prototype, '__proto__');
var get = descriptor.get;
var set = descriptor.set;
var local = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__');
function rejects(receiver) {
  try { get.call(receiver); return false; }
  catch (error) {
    return Object.getPrototypeOf(error) === other.TypeError.prototype &&
      error instanceof other.TypeError && !(error instanceof TypeError);
  }
}
var marker = {};
var calls = 0;
var proxy = new Proxy({}, { getPrototypeOf() { calls++; throw marker; } });
var received;
try { get.call(proxy); } catch (error) { received = error; }
var setterErrors = 0;
try { set.call(null, {}); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) setterErrors++; }
var immutable = Object.preventExtensions({});
try { set.call(immutable, {}); }
catch (error) { if (Object.getPrototypeOf(error) === other.TypeError.prototype) setterErrors++; }
var object = {};
var prototype = {};
var assigned = set.call(object, prototype);
rejects(undefined) && rejects(null) && calls === 1 && received === marker &&
  !descriptor.enumerable && descriptor.configurable &&
  get !== local.get && set !== local.set &&
  Object.getPrototypeOf(get) === other.Function.prototype &&
  Object.getPrototypeOf(set) === other.Function.prototype &&
  get.call(17) === other.Number.prototype &&
  get.call(false) === other.Boolean.prototype &&
  get.call('value') === other.String.prototype &&
  get.call(Symbol()) === other.Symbol.prototype &&
  get.call(1n) === other.BigInt.prototype &&
  get.call(1n << 70n) === other.BigInt.prototype &&
  setterErrors === 2 && assigned === undefined && get.call(object) === prototype;
"#,
        HostSurfacePolicy::Test262,
    );
}
