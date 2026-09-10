use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_aggregate_error(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
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
        .expect("AggregateError construction must use the resolved new target realm");
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn dynamic_function_new_targets_use_canonical_aggregate_error_prototypes() {
    assert_aggregate_error(
        r#"
var target = Function();
target.prototype = null;
var direct = Reflect.construct(AggregateError, [[]], target);
var reads = 0;
var proxied = new Proxy(new Proxy(target.bind(null), {}), {
  get(target, key) {
    if (key === 'prototype') { ++reads; return undefined; }
    return target[key];
  }
});
var indirect = Reflect.construct(AggregateError, [[]], proxied);
Object.getPrototypeOf(direct) === AggregateError.prototype &&
  Object.getPrototypeOf(indirect) === AggregateError.prototype && reads === 1;
"#,
    );
}

#[test]
fn foreign_new_target_realm_keeps_its_intrinsic_after_global_replacement() {
    assert_aggregate_error(
        r#"
var other = __lilaCreateRealm().global;
var prototype = other.AggregateError.prototype;
var target = other.Function();
target.prototype = 1;
other.AggregateError = function replacement() {};
var error = Reflect.construct(AggregateError, [[]], new Proxy(target.bind(null), {}));
Object.getPrototypeOf(error) === prototype && prototype !== AggregateError.prototype;
"#,
    );
}

#[test]
fn prototype_getters_preserve_objects_and_abrupt_completions() {
    assert_aggregate_error(
        r#"
var target = Function();
var prototype = {};
var objectTarget = new Proxy(target, {get(target, key) {
  if (key === 'prototype') return prototype;
  return target[key];
}});
var error = Reflect.construct(AggregateError, [[]], objectTarget);
var marker = {};
var throwingTarget = new Proxy(target, {get(target, key) {
  if (key === 'prototype') throw marker;
  return target[key];
}});
var caught = false;
try { Reflect.construct(AggregateError, [[]], throwingTarget); }
catch (error) { caught = error === marker; }
Object.getPrototypeOf(error) === prototype && caught;
"#,
    );
}

#[test]
fn calls_without_new_use_the_active_foreign_constructor() {
    assert_aggregate_error(
        r#"
var other = __lilaCreateRealm().global;
var constructor = other.AggregateError;
var prototype = constructor.prototype;
other.AggregateError = function replacement() {};
var direct = constructor([]);
var called = constructor.call(null, []);
var applied = constructor.apply(null, [[]]);
Object.getPrototypeOf(direct) === prototype &&
  Object.getPrototypeOf(called) === prototype &&
  Object.getPrototypeOf(applied) === prototype &&
  prototype !== AggregateError.prototype;
"#,
    );
}
