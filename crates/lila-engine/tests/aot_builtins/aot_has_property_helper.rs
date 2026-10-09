use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_has_property(source: &str) {
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
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("shared HasProperty executes through emitted Wasm");
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn has_property_preserves_throw_identity_and_nested_catch_destinations() {
    assert_has_property(
        r#"
var objectMarker = {};
var symbolMarker = Symbol('marker');
var getterProxy = new Proxy({}, { get has() { throw symbolMarker; } });
var trapProxy = new Proxy({}, { has() { throw objectMarker; } });
var caught = 0;
for (var index = 0; index < 2; index++) {
  try {
    if (index === 0) { 'x' in getterProxy; } else { 'x' in trapProxy; }
  } catch (error) {
    if (index === 0 && error === symbolMarker) caught++;
    if (index === 1 && error === objectMarker) caught++;
  }
}
caught === 2;
"#,
    );
}

#[test]
fn has_property_preserves_symbol_keys_and_absent_trap_target_dispatch() {
    assert_has_property(
        r#"
var key = Symbol('key');
var target = { [key]: 1 };
var seen = 0;
var handler = { has(object, property) {
  if (this === handler && object === target && property === key) seen++;
  return property === key;
} };
var inner = new Proxy(target, handler);
var outer = new Proxy(inner, {});
(key in outer) && !('missing' in outer) && seen === 1;
"#,
    );
}

#[test]
fn has_property_canonical_numeric_indices_stop_before_typed_array_prototypes() {
    assert_has_property(
        r#"
var typed = new Uint8Array(1);
var prototype = Object.create(Object.getPrototypeOf(typed));
prototype['-0'] = 1;
prototype['1'] = 1;
prototype['1.5'] = 1;
prototype['01'] = 1;
Object.setPrototypeOf(typed, prototype);
('0' in typed) && !('-0' in typed) && !('1' in typed)
  && !('1.5' in typed) && ('01' in typed);
"#,
    );
}

#[test]
fn has_property_errors_keep_the_executing_foreign_realm() {
    assert_has_property(
        r#"
var realm = __lilaCreateRealm();
realm.evalScript(`
  var count = 0;
  var nonCallable = new Proxy({}, { has: 1 });
  var revoked = Proxy.revocable({}, {}); revoked.revoke();
  var fixedTarget = {};
  Object.defineProperty(fixedTarget, 'x', { value: 1, configurable: false });
  var fixed = new Proxy(fixedTarget, { has() { return false; } });
  var sealed = new Proxy(Object.preventExtensions({ x: 1 }), { has() { return false; } });
  try { 'x' in nonCallable; } catch (error) { if (error instanceof TypeError) count++; }
  try { 'x' in revoked.proxy; } catch (error) { if (error instanceof TypeError) count++; }
  try { 'x' in fixed; } catch (error) { if (error instanceof TypeError) count++; }
  try { 'x' in sealed; } catch (error) { if (error instanceof TypeError) count++; }
  count === 4;
`);
"#,
    );
}

#[test]
fn has_property_does_not_clobber_script_completion_or_with_reference_selection() {
    assert_has_property(
        r#"
var completion = (0, eval)("23; var exists = ('x' in { x: 1 });");
var fallback = 3;
var scope = { fallback: 9, [Symbol.unscopables]: { fallback: true } };
var observed;
with (scope) { observed = eval('fallback'); }
completion === 23 && observed === 3;
"#,
    );
}
