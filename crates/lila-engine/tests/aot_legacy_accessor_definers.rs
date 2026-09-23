use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_accessors(source: &str) {
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
        .expect("accessor definers must compile and execute through Wasm AOT");
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
fn accessor_definers_preserve_the_other_accessor_and_install_symbol_properties() {
    assert_accessors(
        r#"
var target = {};
var symbol = Symbol('field');
var value = 1;
var getter = function(){ return value; };
var setter = function(next){ value = next; };
Object.defineProperty(target, symbol, {get:getter, enumerable:false, configurable:true});
if (target.__defineSetter__(symbol, setter) !== undefined) throw 'setter result';
var descriptor = Object.getOwnPropertyDescriptor(target, symbol);
if (descriptor.get !== getter || descriptor.set !== setter || !descriptor.enumerable || !descriptor.configurable) throw 'setter descriptor';
var replacement = function(){ return value + 1; };
if (target.__defineGetter__(symbol, replacement) !== undefined) throw 'getter result';
descriptor = Object.getOwnPropertyDescriptor(target, symbol);
if (descriptor.get !== replacement || descriptor.set !== setter) throw 'getter descriptor';
target[symbol] = 8;
if (target[symbol] !== 9) throw 'accessor behavior';
var dataTarget = {field:1};
dataTarget.__defineGetter__('field', getter);
if (Object.getOwnPropertyDescriptor(dataTarget, 'field').set !== undefined || dataTarget.field !== 8) throw 'data conversion';
if (Object.prototype.__defineGetter__.call(1, 'field', getter) !== undefined) throw 'Number boxing';
if (Object.prototype.__defineSetter__.call('x', 'extra', setter) !== undefined) throw 'String boxing';
if (Object.prototype.__defineGetter__.call(1n, 'field', getter) !== undefined) throw 'BigInt boxing';
print('ok');
"#,
    );
}

#[test]
fn receiver_and_callability_validation_precede_single_key_conversion() {
    assert_accessors(
        r#"
var defineGetter = Object.prototype.__defineGetter__;
var defineSetter = Object.prototype.__defineSetter__;
var conversions = 0;
var key = { [Symbol.toPrimitive](hint) { if (hint !== 'string') throw 'hint'; conversions++; return 'field'; } };
var getter = function(){return 3;};
for (var receiver of [undefined, null]) {
  var caught = false;
  try { defineGetter.call(receiver, key, getter); } catch (error) { if (!(error instanceof TypeError)) throw error; caught = true; }
  if (!caught || conversions !== 0) throw 'receiver order';
}
for (var invalid of [undefined, null, 0, true, 'x', Symbol(), {}]) {
  for (var define of [defineGetter, defineSetter]) {
    var caught = false;
    try { define.call({}, key, invalid); } catch (error) { if (!(error instanceof TypeError)) throw error; caught = true; }
    if (!caught || conversions !== 0) throw 'callability order';
  }
}
var target = {};
defineGetter.call(target, key, getter);
if (conversions !== 1 || target.field !== 3) throw 'conversion count';
var marker = {};
var throwingKey = { [Symbol.toPrimitive]() { throw marker; } };
var propagated = false;
try { defineSetter.call(target, throwingKey, getter); } catch (error) { if (error !== marker) throw error; propagated = true; }
if (!propagated) throw 'key abrupt completion';
print('ok');
"#,
    );
}

#[test]
fn internal_partial_descriptors_ignore_prototype_and_public_builtin_mutation() {
    assert_accessors(
        r#"
var defineGetter = Object.prototype.__defineGetter__;
var defineSetter = Object.prototype.__defineSetter__;
var originalDefineProperty = Object.defineProperty;
var getter = function(){return 4;};
var setter = function(){};
var target = {};
originalDefineProperty(target, 'field', {get:getter, configurable:true});
var getPoison = Object.create(null);
getPoison.value = 17;
getPoison.configurable = true;
var valuePoison = Object.create(null);
valuePoison.value = 19;
valuePoison.configurable = true;
originalDefineProperty(Object.prototype, 'get', getPoison);
originalDefineProperty(Object.prototype, 'value', valuePoison);
Object.defineProperty = function(){throw 'public defineProperty was consulted';};
var result = defineSetter.call(target, 'field', setter);
delete Object.prototype.get;
delete Object.prototype.value;
Object.defineProperty = originalDefineProperty;
if (result !== undefined) throw 'result';
var descriptor = Object.getOwnPropertyDescriptor(target, 'field');
if (descriptor.get !== getter || descriptor.set !== setter || !descriptor.enumerable || !descriptor.configurable) throw 'partial descriptor';
var callback = Proxy.revocable(function(){return 1;}, {});
callback.revoke();
defineGetter.call({}, 'revoked', callback.proxy);
print('ok');
"#,
    );
}

#[test]
fn proxy_descriptors_and_definition_errors_follow_the_borrowed_method_realm() {
    assert_accessors(
        r#"
var other = __lilaCreateRealm().global;
var defineGetter = other.Object.prototype.__defineGetter__;
var defineSetter = other.Object.prototype.__defineSetter__;
var objectPrototype = other.Object.prototype;
var typeErrorPrototype = other.TypeError.prototype;
other.Object = function(){throw 'public Object';};
other.TypeError = function(){throw 'public TypeError';};
var symbol = Symbol('field');
var getter = function(){return 7;};
var seen = 0;
var proxy = new Proxy({}, {
  defineProperty(target, key, descriptor) {
    seen++;
    if (key !== symbol || Object.getPrototypeOf(descriptor) !== objectPrototype) throw 'descriptor Realm/key';
    if (Object.keys(descriptor).join(',') !== 'get,enumerable,configurable') throw 'partial fields';
    if (descriptor.get !== getter || !descriptor.enumerable || !descriptor.configurable) throw 'descriptor values';
    return true;
  }
});
if (defineGetter.call(proxy, symbol, getter) !== undefined || seen !== 1) throw 'trap count/result';
function rejected(method, receiver, key, accessor) {
  var caught = false;
  try { method.call(receiver, key, accessor); }
  catch (error) { if (Object.getPrototypeOf(error) !== typeErrorPrototype) throw 'error Realm'; caught = true; }
  if (!caught) throw 'expected rejection';
}
rejected(defineGetter, null, 'x', getter);
rejected(defineSetter, {}, 'x', undefined);
rejected(defineGetter, Object.preventExtensions({}), 'x', getter);
rejected(defineGetter, new Proxy({}, {defineProperty(){return false;}}), 'x', getter);
var array = [1];
Object.defineProperty(array, '0', {configurable:false});
rejected(defineSetter, array, '0', getter);
rejected(defineGetter, 'x', '0', getter);
print('ok');
"#,
    );
}

#[test]
fn accessor_definers_have_native_metadata_and_no_constructor_protocol() {
    assert_accessors(
        r#"
for (var name of ['__defineGetter__', '__defineSetter__']) {
  var method = Object.prototype[name];
  var property = Object.getOwnPropertyDescriptor(Object.prototype, name);
  if (typeof method !== 'function' || !property.writable || property.enumerable || !property.configurable) throw 'method property';
  if (method.name !== name || method.length !== 2 || Object.prototype.hasOwnProperty.call(method, 'prototype')) throw 'method metadata';
  var length = Object.getOwnPropertyDescriptor(method, 'length');
  var title = Object.getOwnPropertyDescriptor(method, 'name');
  if (length.writable || length.enumerable || !length.configurable || title.writable || title.enumerable || !title.configurable) throw 'metadata attributes';
  if (!Function.prototype.toString.call(method).includes('[native code]')) throw 'native representation';
  var caught = false;
  try { Reflect.construct(method, []); } catch (error) { if (!(error instanceof TypeError)) throw error; caught = true; }
  if (!caught) throw 'constructor protocol';
}
var inherited = 0;
var prototype = {set field(value){inherited++;}};
var object = Object.create(prototype);
object.__defineGetter__('field', function(){return 2;});
if (inherited !== 0 || object.field !== 2) throw 'inherited setter';
print('ok');
"#,
    );
}
