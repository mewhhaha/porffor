use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_definition_realms(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    let observation = Engine::new(RealmBuilder::new().build())
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
        .expect("property definitions must compile and execute through Wasm AOT");
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
fn define_property_trap_descriptors_use_the_method_realm_and_preserve_fields() {
    assert_definition_realms(
        r#"
var other = __lilaCreateRealm().global;
var symbol = Symbol('field');
var value = {};
var expectedPrototype = Object.prototype;
var input = {value:value, writable:1, enumerable:1, configurable:1};
var descriptors = [];
var proxy = new Proxy({}, {
  defineProperty(target, key, descriptor) {
    if (Object.getPrototypeOf(descriptor) !== expectedPrototype) throw 'descriptor Realm';
    if (descriptor === input || key !== symbol || descriptor.value !== value) throw 'identity';
    if (Object.keys(descriptor).join(',') !== 'value,writable,enumerable,configurable') throw 'field order';
    for (var field of Object.keys(descriptor)) {
      var own = Object.getOwnPropertyDescriptor(descriptor, field);
      if (!own.writable || !own.enumerable || !own.configurable) throw 'field attributes';
    }
    if (descriptor.writable !== true || descriptor.enumerable !== true || descriptor.configurable !== true) throw 'ToBoolean';
    descriptors.push(descriptor);
    return true;
  }
});
if (Object.defineProperty(proxy, symbol, input) !== proxy) throw 'main return';
expectedPrototype = other.Object.prototype;
if (other.Object.defineProperty(proxy, symbol, input) !== proxy) throw 'foreign return';
if (descriptors.length !== 2 || descriptors[0] === descriptors[1]) throw 'fresh descriptors';
print('ok');
"#,
    );
}

#[test]
fn define_properties_preserves_partial_descriptors_and_single_ordered_reads() {
    assert_definition_realms(
        r#"
var other = __lilaCreateRealm().global;
var order = '';
var getter = function() { throw 'descriptor getter was invoked'; };
var partial = {get:getter};
var complete = {
  get enumerable() { order += 'e'; return true; },
  get configurable() { order += 'c'; return true; },
  get value() { order += 'v'; return 3; },
  get writable() { order += 'w'; return true; }
};
var seen = '';
var proxy = new Proxy({}, {
  defineProperty(target, key, descriptor) {
    if (Object.getPrototypeOf(descriptor) !== other.Object.prototype) throw 'descriptor Realm';
    if (order !== 'ecvw') throw 'conversion order';
    if (key === 'partial') {
      if (Object.keys(descriptor).join(',') !== 'get' || descriptor.get !== getter) throw 'absent fields';
    } else if (key === 'complete') {
      if (Object.keys(descriptor).join(',') !== 'value,writable,enumerable,configurable') throw 'complete fields';
      if (descriptor.value !== 3) throw 'value';
    } else throw 'key';
    seen += key + ';';
    return true;
  }
});
if (other.Object.defineProperties(proxy, {partial:partial, complete:complete}) !== proxy) throw 'return';
if (seen !== 'partial;complete;' || order !== 'ecvw') throw 'observation count';
print('ok');
"#,
    );
}

#[test]
fn define_property_type_errors_use_the_method_realm_for_exotic_rejections() {
    assert_definition_realms(
        r#"
var other = __lilaCreateRealm().global;
function rejected(name, target, key, descriptor) {
  var caught = false;
  try { other.Object.defineProperty(target, key, descriptor); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== other.TypeError.prototype) throw name + ':Realm';
    caught = true;
  }
  if (!caught) throw name + ':accepted';
}
rejected('false proxy', new Proxy({}, {defineProperty(){return false;}}), 'field', {value:1});
rejected('mixed descriptor', {}, 'field', {get:function(){}, value:1});
rejected('noncallable getter', {}, 'field', {get:1});
rejected('nonextensible object', Object.preventExtensions({}), 'field', {value:1});
var array = [1];
Object.defineProperty(array, '0', {configurable:false, writable:false});
rejected('array index', array, '0', {value:2});
rejected('array accessor length', [], 'length', {get:function(){}});
var readonly = [];
Object.defineProperty(readonly, 'length', {writable:false});
rejected('array length value', readonly, 'length', {value:1});
rejected('array length writable', readonly, 'length', {writable:true});
rejected('array length configurable', [], 'length', {configurable:true});
rejected('boxed string index', Object('x'), '0', {value:'y'});
var args = (function(value){return arguments;})(1);
Object.defineProperty(args, '0', {configurable:false, writable:false});
rejected('arguments index', args, '0', {value:2});
var emptyArgs = (function(){return arguments;})();
rejected('nonextensible arguments', Object.preventExtensions(emptyArgs), '0', {value:1});
var strictArgs = (function(){'use strict';return arguments;})();
rejected('arguments callee configurable', strictArgs, 'callee', {configurable:true});
rejected('arguments callee enumerable', strictArgs, 'callee', {enumerable:true});
rejected('arguments callee data', strictArgs, 'callee', {value:1});
rejected('arguments callee getter', strictArgs, 'callee', {get:function(){}});
var frozenCallee = (function(){return arguments;})();
Object.defineProperty(frozenCallee, 'callee', {writable:false, configurable:false});
rejected('arguments callee writable', frozenCallee, 'callee', {writable:true});
rejected('arguments callee value', frozenCallee, 'callee', {value:1});
print('ok');
"#,
    );
}

#[test]
fn borrowed_definition_uses_intrinsics_after_public_constructor_bindings_change() {
    assert_definition_realms(
        r#"
var other = __lilaCreateRealm().global;
var define = other.Object.defineProperty;
var objectPrototype = other.Object.prototype;
var typeErrorPrototype = other.TypeError.prototype;
other.Object = function(){throw 'public Object lookup';};
other.TypeError = function(){throw 'public TypeError lookup';};
var seen = false;
var proxy = new Proxy({}, {
  defineProperty(target, key, descriptor) {
    if (Object.getPrototypeOf(descriptor) !== objectPrototype) throw 'intrinsic Object';
    seen = true;
    return false;
  }
});
var caught = false;
try { define(proxy, 'field', {value:3}); }
catch (error) {
  if (Object.getPrototypeOf(error) !== typeErrorPrototype) throw 'intrinsic TypeError';
  caught = true;
}
if (!seen || !caught) throw 'missing operation';
print('ok');
"#,
    );
}

#[test]
fn public_field_trap_descriptors_follow_the_class_realm_in_both_directions() {
    assert_definition_realms(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
var symbol = Symbol('field');
other.symbol = symbol;
var seen = 0;
other.receiver = new Proxy({}, {
  defineProperty(target, key, descriptor) {
    if (Object.getPrototypeOf(descriptor) !== other.Object.prototype) throw 'foreign class descriptor Realm';
    if (key !== symbol || descriptor.value !== 3) throw 'foreign class field';
    if (!descriptor.writable || !descriptor.enumerable || !descriptor.configurable) throw 'attributes';
    seen++;
    return true;
  }
});
var Derived = realm.evalScript('(class extends (class { constructor() { return globalThis.receiver; } }) { [globalThis.symbol] = 3; })');
new Derived();
if (seen !== 1) throw 'foreign class trap count';
other.expectedPrototype = Object.prototype;
other.seen = 0;
var receiver = realm.evalScript('new Proxy({}, { defineProperty(target, key, descriptor) { if (Object.getPrototypeOf(descriptor) !== globalThis.expectedPrototype) throw "main class descriptor Realm"; if (key !== "field" || descriptor.value !== 5) throw "main class field"; globalThis.seen++; return true; } })');
class MainBase { constructor() { return receiver; } }
class MainDerived extends MainBase { field = 5; }
new MainDerived();
if (other.seen !== 1) throw 'main class trap count';
print('ok');
"#,
    );
}

#[test]
fn foreign_public_field_rejections_keep_the_class_realm_and_stop_initialization() {
    assert_definition_realms(
        r#"
var realm = __lilaCreateRealm();
var other = realm.global;
other.later = 0;
var Derived = realm.evalScript('(class extends (class { constructor() { return globalThis.receiver; } }) { field = 3; later = globalThis.later++; })');
function rejected(receiver) {
  other.receiver = receiver;
  var caught = false;
  try { new Derived(); }
  catch (error) {
    if (Object.getPrototypeOf(error) !== other.TypeError.prototype) throw 'class error Realm';
    caught = true;
  }
  if (!caught || other.later !== 0) throw 'class failure ordering';
}
rejected(new Proxy({}, {defineProperty(){return false;}}));
rejected(Object.preventExtensions({}));
var marker = {};
other.receiver = new Proxy({}, {defineProperty(){throw marker;}});
var propagated = false;
try { new Derived(); } catch (error) { propagated = error === marker; }
if (!propagated || other.later !== 0) throw 'trap abrupt completion';
print('ok');
"#,
    );
}
