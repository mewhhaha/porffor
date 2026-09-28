use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
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
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("arguments allocation must preserve Wasm AOT execution");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}: {}\n{source}",
        outcome.completion,
        outcome.note,
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "{source}",
    );
}

#[test]
fn repeated_sloppy_calls_defer_unobserved_arguments() {
    assert_trace(
        r#"
function add(left, right) { return left + right; }
var result = 0;
for (var i = 0; i < 4000000; i++) result = add(result, 1);
print(result, add.arguments === null);
"#,
        &["4000000 true"],
    );
}

#[test]
fn deferred_arguments_keep_mapping_identity_and_escape() {
    assert_trace(
        r#"
function inspect(f) { return Object.getOwnPropertyDescriptor(f, 'arguments').value; }
function owner(first, second) {
  first = 7;
  var a = inspect(owner);
  print(a[0], a[1], a === owner.arguments, a.callee === owner);
  first = 9;
  print(a[0]);
  a[0] = 11;
  print(first);
  return a;
}
var saved = owner(3, 5);
print(saved[0], saved[1], owner.arguments === null);
function duplicate(x, x) {
  x = 19;
  return inspect(duplicate);
}
var duplicated = duplicate(13, 17);
print(duplicated[0], duplicated[1]);
var missing = duplicate(23);
print(missing[0], missing.length);
"#,
        &["7 5 true true", "9", "11", "11 5 true", "13 19", "23 1"],
    );
}

#[test]
fn recursive_frames_restore_values_after_normal_and_abrupt_completion() {
    assert_trace(
        r#"
var escaped = [];
function f(depth) {
  if (depth === 0) {
    escaped.push(f.arguments);
    throw 29;
  }
  var before = f.arguments;
  try { f(depth - 1); } catch (e) { print(e); }
  print(before === f.arguments, f.arguments[0]);
  escaped.push(f.arguments);
}
f(1);
print(escaped[0][0], escaped[1][0], escaped[0] !== escaped[1], f.arguments === null);
function pending(depth) {
  if (depth) pending(0);
  print(pending.arguments[0]);
}
pending(1);
print(pending.arguments === null);
"#,
        &["29", "true 1", "0 1 true true", "0", "1", "true"],
    );
}

#[test]
fn all_descriptor_observations_materialize_before_validation() {
    assert_trace(
        r#"
function attempt(f) {
  try { Object.defineProperty(f, 'arguments', {value: null}); return false; }
  catch (e) { return e instanceof TypeError; }
}
function definition(first) {
  print(attempt(definition));
  var d = Object.getOwnPropertyDescriptors(definition).arguments;
  print(d.value[0], d.writable, d.enumerable, d.configurable, 'get' in d);
  print(Reflect.defineProperty(definition, 'arguments', {value: d.value}));
}
definition(31);
function proxyGet(first) {
  var p = new Proxy(proxyGet, {get: function() { return null; }});
  try { p.arguments; print(false); } catch (e) { print(e instanceof TypeError); }
}
proxyGet(37);
function inherited(first) {
  var child = Object.create(inherited);
  print(child.arguments[0], new Proxy(inherited, {}).arguments === child.arguments);
}
inherited(41);
"#,
        &[
            "true",
            "31 false false false false",
            "true",
            "true",
            "41 true",
        ],
    );
}

#[test]
fn materialization_uses_the_defining_realm() {
    assert_trace(
        r#"
var other = __lilaCreateRealm();
other.evalScript('function foreign(value, callback) { return callback(foreign); }');
var saved = other.global.foreign(43, function (f) { return f.arguments; });
print(saved[0], Object.getPrototypeOf(saved) === other.global.Object.prototype,
  Object.getPrototypeOf(saved) !== Object.prototype, other.global.foreign.arguments === null);
"#,
        &["43 true true true"],
    );
}
