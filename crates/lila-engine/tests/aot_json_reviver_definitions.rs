use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_reviver_output(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    let mut forms = vec![source.to_owned()];
    if source.contains("JSON.parse(") {
        forms.push(format!(
            "var __jsonParse = JSON.parse;\n{}",
            source.replace("JSON.parse(", "__jsonParse(")
        ));
    }
    for source in forms {
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
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
            .expect("JSON reviver definitions must execute through Wasm AOT");
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observation.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observation.completion
        );
        assert_eq!(
            observation.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{source}"
        );
    }
}

#[test]
fn standalone_reviver_roots_complete_property_definition_without_other_globals() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    for source in [
        "JSON.parse('{\"value\":7}', (key, value) => key === 'value' ? 8 : value).value;",
        "var parse = JSON.parse; parse('{\"value\":7}', (key, value) => key === 'value' ? 8 : value).value;",
    ] {
        let observation = Engine::new(RealmBuilder::new().build())
            .observe_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("standalone JSON reviver must execute through Wasm AOT");
        assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observation.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(8.0))),
            "{source}"
        );
        assert!(observation.output_events.is_empty());
    }
}

#[test]
fn reviver_cannot_overwrite_sparse_nonconfigurable_indexes() {
    assert_reviver_output(
        r#"
var injected, visits = 0;
var result = JSON.parse('{"first":0,"second":0}', function(key, value) {
  if (key === 'first') {
    injected = [];
    Object.defineProperty(injected, '100', {value: 7, writable: false, enumerable: true, configurable: false});
    this.second = injected;
    return value;
  }
  if (this === injected) { visits++; return key === '100' ? 8 : undefined; }
  return value;
});
var descriptor = Object.getOwnPropertyDescriptor(result.second, '100');
print(visits === 101, result.second === injected, descriptor.value === 7, !descriptor.writable, !descriptor.configurable, Object.keys(injected).join(',') === '100');
"#,
        "true true true true true true",
    );
}

#[test]
fn reviver_replaces_sparse_accessors_with_complete_data_descriptors() {
    assert_reviver_output(
        r#"
var injected, gets = 0, sets = 0;
var result = JSON.parse('{"first":0,"second":0}', function(key, value) {
  if (key === 'first') {
    injected = [];
    Object.defineProperty(injected, '100', {get: function() { gets++; return 7; }, set: function(v) { sets++; }, configurable: true});
    this.second = injected;
    return value;
  }
  if (this === injected) return key === '100' ? 8 : undefined;
  return value;
});
var descriptor = Object.getOwnPropertyDescriptor(result.second, '100');
print(descriptor.value === 8, descriptor.get === undefined, descriptor.set === undefined, descriptor.writable, descriptor.enumerable, descriptor.configurable, gets === 1, sets === 0);
"#,
        "true true true true true true true true",
    );
}

#[test]
fn reviver_ignores_rejected_recreation_on_nonextensible_arrays() {
    assert_reviver_output(
        r#"
var injected;
var result = JSON.parse('{"first":0,"second":0}', function(key, value) {
  if (key === 'first') { injected = [7]; Object.preventExtensions(injected); this.second = injected; return value; }
  if (this === injected) { delete injected[0]; return 8; }
  return value;
});
print(result.second === injected, !Object.hasOwn(injected, '0'), injected.length === 1, !Object.isExtensible(injected));
"#,
        "true true true true",
    );
}

#[test]
fn reviver_ignores_false_proxy_definitions_and_preserves_descriptor_attributes() {
    assert_reviver_output(
        r#"
var proxy, definitions = 0;
var result = JSON.parse('{"first":0,"second":0}', function(key, value) {
  if (key === 'first') {
    proxy = new Proxy({field: 7}, {defineProperty(target, key, descriptor) {
      if (key !== 'field' || descriptor.value !== 8 || !descriptor.writable || !descriptor.enumerable || !descriptor.configurable) throw 'descriptor';
      definitions++; return false;
    }});
    this.second = proxy;
  }
  return this === proxy ? 8 : value;
});
print(result.second === proxy, proxy.field === 7, definitions === 1);
"#,
        "true true true",
    );
}

#[test]
fn reviver_propagates_proxy_definition_throws_through_catch_and_finally() {
    assert_reviver_output(
        r#"
var reasons = [{}, undefined], caught = 0, finalized = 0;
for (var reason of reasons) {
  var proxy;
  try {
    JSON.parse('{"first":0,"second":0}', function(key, value) {
      if (key === 'first') { proxy = new Proxy({field: 7}, {defineProperty() { throw reason; }}); this.second = proxy; }
      return this === proxy ? 8 : value;
    });
  } catch (error) { if (error === reason) caught++; }
  finally { finalized++; }
}
print(caught === 2, finalized === 2);
"#,
        "true true",
    );
}

#[test]
fn reviver_ignores_nonextensible_objects_and_array_length_rejections() {
    assert_reviver_output(
        r#"
var object, array;
var result = JSON.parse('{"first":0,"second":0,"third":0}', function(key, value) {
  if (key === 'first') {
    object = Object.preventExtensions({field: 7});
    array = [7, 8]; this.second = object; this.third = array;
  }
  if (this === object) { delete object.field; return 9; }
  if (this === array) { array.length = 0; Object.defineProperty(array, 'length', {writable: false}); return 9; }
  return value;
});
print(result.second === object, !Object.hasOwn(object, 'field'), result.third === array, array.length === 0, !Object.hasOwn(array, '0'), !Object.hasOwn(array, '1'));
"#,
        "true true true true true true",
    );
}

#[test]
fn reviver_definitions_ignore_public_intrinsic_replacement_and_descriptor_prototypes() {
    assert_reviver_output(
        r#"
var define = Object.defineProperty, reflectDefine = Reflect.defineProperty, reads = 0, result;
define(Object.prototype, 'get', {get: function() { reads++; throw 'inherited descriptor field'; }, configurable: true});
try {
  result = JSON.parse('{"value":7}', function(key, value) {
    Object.defineProperty = function() { throw 'public Object definition'; };
    Reflect.defineProperty = function() { throw 'public Reflect definition'; };
    return key === 'value' ? 8 : value;
  });
} finally {
  Object.defineProperty = define; Reflect.defineProperty = reflectDefine; delete Object.prototype.get;
}
print(result.value === 8, reads === 0);
"#,
        "true true",
    );
}

#[test]
fn foreign_json_reviver_proxy_descriptors_and_invariant_errors_use_its_realm() {
    assert_reviver_output(
        r#"
var other = __lilaCreateRealm().global, proxy, calls = 0;
var parse = other.JSON.parse;
var result = parse('{"first":0,"second":0}', function(key, value) {
  if (key === 'first') {
    proxy = new Proxy({field: 7}, {defineProperty(target, key, descriptor) {
      if (Object.getPrototypeOf(descriptor) !== other.Object.prototype) throw 'descriptor Realm';
      calls++; return false;
    }}); this.second = proxy;
  }
  return value;
});
var rejected = false;
try {
  parse('{"first":0,"second":0}', function(key, value) {
    if (key === 'first') {
      var target = {}; Object.defineProperty(target, 'field', {value: 7, enumerable: true});
      proxy = new Proxy(target, {defineProperty() { return true; }}); this.second = proxy;
    }
    return value;
  });
} catch (error) { rejected = Object.getPrototypeOf(error) === other.TypeError.prototype; }
print(result.second.field === 7, calls === 1, rejected);
"#,
        "true true true",
    );
}
