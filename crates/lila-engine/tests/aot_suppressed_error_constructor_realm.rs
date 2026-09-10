use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_suppressed_error(source: &str) {
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
        .expect("SuppressedError construction must use the resolved new target realm");
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn foreign_new_targets_resolve_canonical_suppressed_error_prototypes() {
    assert_suppressed_error(
        r#"
var other = __lilaCreateRealm().global;
var prototype = other.SuppressedError.prototype;
var target = other.Function();
other.SuppressedError = function replacement() {};
var values = [undefined, null, true, '', Symbol(), -1];
var valid = true;
for (var value of values) {
  target.prototype = value;
  var error = Reflect.construct(SuppressedError, [7, 8, 'message'], target);
  valid = valid && Object.getPrototypeOf(error) === prototype &&
    error.error === 7 && error.suppressed === 8 && error.message === 'message';
}
var reads = 0;
var proxied = new Proxy(new Proxy(target.bind(null), {}), {get(target, key) {
  if (key === 'prototype') { ++reads; return undefined; }
  return target[key];
}});
var error = Reflect.construct(SuppressedError, [7, 8], proxied);
valid && Object.getPrototypeOf(error) === prototype && reads === 1;
"#,
    );
}

#[test]
fn calls_without_new_use_the_active_foreign_constructor() {
    assert_suppressed_error(
        r#"
var other = __lilaCreateRealm().global;
var constructor = other.SuppressedError;
var prototype = constructor.prototype;
other.SuppressedError = function replacement() {};
var direct = constructor(7, 8);
var called = constructor.call(null, 7, 8);
var applied = constructor.apply(null, [7, 8]);
Object.getPrototypeOf(direct) === prototype &&
  Object.getPrototypeOf(called) === prototype &&
  Object.getPrototypeOf(applied) === prototype &&
  direct.error === 7 && direct.suppressed === 8;
"#,
    );
}

#[test]
fn explicit_prototypes_and_abrupt_gets_are_preserved() {
    assert_suppressed_error(
        r#"
var prototype = {};
var target = new Proxy(function () {}, {get(target, key) {
  if (key === 'prototype') return prototype;
  return target[key];
}});
var error = Reflect.construct(SuppressedError, [7, 8], target);
var marker = {};
var throwing = new Proxy(function () {}, {get(target, key) {
  if (key === 'prototype') throw marker;
  return target[key];
}});
var caught = false;
try { Reflect.construct(SuppressedError, [7, 8], throwing); }
catch (error) { caught = error === marker; }
Object.getPrototypeOf(error) === prototype && caught;
"#,
    );
}
