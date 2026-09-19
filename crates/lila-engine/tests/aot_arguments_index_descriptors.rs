use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_arguments_output(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    let source = format!(
        "function assert(condition, message) {{ if (!condition) throw new Error(message); }}\n{source}"
    );
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
        .expect("Arguments indexed descriptors must execute through Wasm AOT");
    assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("ok".into())],
        "{source}"
    );
}

#[test]
fn every_attribute_combination_remains_present_in_each_arguments_mode() {
    assert_arguments_output(
        r#"
function mapped(value) { return arguments; }
function unmapped() { return arguments; }
function strict(value) { 'use strict'; return arguments; }
var factories = [mapped, unmapped, strict];
for (var kind = 0; kind < factories.length; kind++) {
  for (var bits = 0; bits < 8; bits++) {
    for (var reflect = 0; reflect < 2; reflect++) {
      var source = factories[kind](10);
      var attributes = {
        value: 20,
        writable: (bits & 1) !== 0,
        enumerable: (bits & 2) !== 0,
        configurable: (bits & 4) !== 0
      };
      if (reflect) {
        assert(Reflect.defineProperty(source, '0', attributes), 'Reflect definition');
      } else {
        assert(Object.defineProperty(source, '0', attributes) === source, 'Object definition');
      }
      var descriptor = Object.getOwnPropertyDescriptor(source, '0');
      assert(descriptor !== undefined, 'own descriptor exists');
      assert(descriptor.value === 20 && source[0] === 20, 'stored value');
      assert(descriptor.writable === attributes.writable, 'writable');
      assert(descriptor.enumerable === attributes.enumerable, 'enumerable');
      assert(descriptor.configurable === attributes.configurable, 'configurable');
      assert(Object.hasOwn(source, '0'), 'own property presence');
      assert(Object.getOwnPropertyNames(source).indexOf('0') !== -1, 'own name');
      assert((Object.keys(source).indexOf('0') !== -1) === attributes.enumerable, 'enumeration');
    }
  }
}
print('ok');
"#,
    );
}

#[test]
fn non_writable_definitions_capture_then_detach_the_original_parameter_slot() {
    assert_arguments_output(
        r#"
function omitted(first, second, third) {
  second = 22;
  Object.defineProperty(arguments, '1', {
    writable: false, enumerable: false, configurable: false
  });
  var descriptor = Object.getOwnPropertyDescriptor(arguments, '1');
  assert(descriptor.value === 22 && arguments[1] === 22, 'snapshot the current mapped value');
  second = 33;
  assert(arguments[1] === 22 && second === 33, 'parameter mutation after detachment');
  assert(first === 1 && third === 3, 'nonzero parameter slot');
  assert(!Reflect.set(arguments, '1', 44), 'non-writable indexed assignment');
  assert(arguments[1] === 22 && second === 33, 'rejected write leaves both values');
}
function explicit(first, second, third) {
  assert(Reflect.defineProperty(arguments, '1', {
    value: undefined, writable: false, enumerable: false, configurable: false
  }), 'define an explicit undefined value');
  assert(second === undefined && arguments[1] === undefined, 'update the mapped parameter once');
  var descriptor = Object.getOwnPropertyDescriptor(arguments, '1');
  assert(descriptor !== undefined && descriptor.value === undefined, 'undefined remains present');
  second = 55;
  assert(arguments[1] === undefined && second === 55, 'undefined property stays detached');
  assert(first === 1 && third === 3, 'adjacent parameters remain intact');
}
function retained(first, second, third) {
  Object.defineProperty(arguments, '1', {enumerable: false, configurable: false});
  second = 66;
  assert(arguments[1] === 66, 'generic descriptor retains mapping');
  var caught = false;
  try { Object.defineProperty(arguments, '1', {get() { return 0; }}); }
  catch (error) { caught = error instanceof TypeError; }
  assert(caught, 'incompatible accessor is rejected');
  second = 77;
  assert(arguments[1] === 77, 'rejected descriptor leaves mapping intact');
  Object.defineProperty(arguments, '1', {writable: false});
  second = 88;
  assert(arguments[1] === 77 && first === 1 && third === 3, 'later detach retains the full slot');
}
omitted(1, 2, 3);
explicit(1, 2, 3);
retained(1, 2, 3);
print('ok');
"#,
    );
}

#[test]
fn undefined_and_default_descriptors_shadow_prototypes_without_growing_length() {
    assert_arguments_output(
        r#"
function mapped(value) { return arguments; }
function strict(value) { 'use strict'; return arguments; }
var factories = [mapped, strict];
for (var kind = 0; kind < factories.length; kind++) {
  var source = factories[kind](1), reads = 0;
  assert(delete source[0], 'delete the initial index');
  assert(Object.getOwnPropertyDescriptor(source, '0') === undefined, 'deleted descriptor is absent');
  var prototype = Object.create(Object.getPrototypeOf(source));
  Object.defineProperty(prototype, '0', {get() { reads++; return 9; }});
  Object.setPrototypeOf(source, prototype);
  assert(source[0] === 9 && reads === 1, 'deleted index uses the prototype');
  Object.defineProperty(source, '0', {value: undefined});
  assert(source[0] === undefined && Reflect.get(source, '0') === undefined, 'own undefined shadows');
  assert(reads === 1 && Object.hasOwn(source, '0'), 'no inherited getter call');
  assert(Reflect.defineProperty(source, '5', {}), 'default descriptor on a new index');
  var descriptor = Object.getOwnPropertyDescriptor(source, '5');
  assert(descriptor !== undefined && descriptor.value === undefined, 'default undefined exists');
  assert(!descriptor.writable && !descriptor.enumerable && !descriptor.configurable, 'default attributes');
  assert(source.length === 1, 'indexed extent does not change observable length');
  assert(Reflect.ownKeys(source).indexOf('5') !== -1, 'extended index is an own key');
  assert(Object.hasOwn(Object.getOwnPropertyDescriptors(source), '5'), 'descriptor collection');
  source.length = 0;
  assert(Object.hasOwn(source, '0') && Object.hasOwn(source, '5'), 'length does not delete own indexes');
  Object.preventExtensions(source);
  assert(Reflect.defineProperty(source, '0', {value: undefined}), 'SameValue on a non-extensible object');
  assert(!Reflect.defineProperty(source, '6', {}), 'a missing index remains absent on rejection');
  assert(Object.getOwnPropertyDescriptor(source, '6') === undefined, 'no rejected descriptor publication');
}
print('ok');
"#,
    );
}

#[test]
fn non_configurable_data_preserves_same_value_rules_and_failed_mutations() {
    assert_arguments_output(
        r#"
function source() { return arguments; }
var values = [undefined, NaN, 0, -0, 20, {}];
for (var index = 0; index < values.length; index++) {
  var value = values[index], args = source(value);
  Object.defineProperty(args, '0', {
    value: value, writable: false, enumerable: false, configurable: false
  });
  assert(Reflect.defineProperty(args, '0', {value: value}), 'Reflect SameValue succeeds');
  assert(Object.defineProperty(args, '0', {value: value}) === args, 'Object SameValue succeeds');
  var replacement = index === 2 ? -0 : index === 3 ? 0 : 21;
  assert(!Reflect.defineProperty(args, '0', {value: replacement}), 'Reflect incompatible value');
  var caught = false;
  try { Object.defineProperty(args, '0', {value: replacement}); }
  catch (error) { caught = error instanceof TypeError; }
  assert(caught, 'Object incompatible value');
  assert(!Reflect.defineProperty(args, '0', {writable: true}), 'cannot make writable');
  assert(!Reflect.defineProperty(args, '0', {enumerable: true}), 'cannot make enumerable');
  assert(!Reflect.defineProperty(args, '0', {configurable: true}), 'cannot make configurable');
  assert(!Reflect.deleteProperty(args, '0'), 'cannot delete');
  var descriptor = Object.getOwnPropertyDescriptor(args, '0');
  assert(descriptor !== undefined && Object.is(descriptor.value, value), 'failed definitions preserve value');
  assert(Object.is(args[0], value), 'indexed value remains unchanged');
  assert(!descriptor.writable && !descriptor.enumerable && !descriptor.configurable, 'attributes unchanged');
}
print('ok');
"#,
    );
}

#[test]
fn accessor_conversion_and_deletion_keep_distinct_presence_lifecycles() {
    assert_arguments_output(
        r#"
function mapped(first, second) {
  var gets = 0, getter = function() { gets++; return 5; };
  Object.defineProperty(arguments, '1', {get: getter, configurable: true, enumerable: false});
  var descriptor = Object.getOwnPropertyDescriptor(arguments, '1');
  assert(descriptor.get === getter && gets === 0, 'accessor materialization does not invoke it');
  second = 6;
  assert(arguments[1] === 5 && gets === 1, 'accessor detached the parameter');
  Object.defineProperty(arguments, '1', {
    value: 7, writable: false, enumerable: false, configurable: false
  });
  descriptor = Object.getOwnPropertyDescriptor(arguments, '1');
  assert(descriptor.value === 7 && !Object.hasOwn(descriptor, 'get'), 'accessor becomes own data');
  assert(arguments[1] === 7 && gets === 1 && second === 6, 'conversion does not call or remap');
  assert(!Reflect.deleteProperty(arguments, '1'), 'non-configurable data survives delete');
  assert(delete arguments[0], 'a configurable index is deleted');
  assert(Object.getOwnPropertyDescriptor(arguments, '0') === undefined, 'deletion clears the descriptor');
  assert(!Object.hasOwn(arguments, '0'), 'deletion clears own presence');
  arguments[0] = 8;
  descriptor = Object.getOwnPropertyDescriptor(arguments, '0');
  assert(descriptor.value === 8 && descriptor.writable && descriptor.enumerable && descriptor.configurable,
    'receiver-side assignment creates a normal own property');
  assert(first === 1, 'assignment does not recreate the deleted parameter map');
}
mapped(1, 2);
print('ok');
"#,
    );
}
