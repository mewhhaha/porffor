use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_intl_realm(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .expect("Intl Realm regression must execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion,
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())],
        "{source}",
    );
}

#[test]
fn created_realms_publish_the_complete_represented_intl_namespace() {
    assert_intl_realm(
        r#"
var first = __lilaCreateRealm().global;
var second = __lilaCreateRealm().global;
for (var foreign of [first, second]) {
  if (typeof foreign.Intl !== 'object' || foreign.Intl === Intl ||
      Object.getPrototypeOf(foreign.Intl) !== foreign.Object.prototype) throw 'namespace Realm';
  var namespace = Object.getOwnPropertyDescriptor(foreign, 'Intl');
  if (!namespace.writable || namespace.enumerable || !namespace.configurable) throw 'namespace descriptor';
  if (Object.getOwnPropertyNames(foreign.Intl).join(',') !== Object.getOwnPropertyNames(Intl).join(',')) throw 'namespace members';
  if (Object.prototype.toString.call(foreign.Intl) !== '[object Intl]') throw 'namespace tag';
  for (var name of ['getCanonicalLocales','Locale','DateTimeFormat']) {
    var local = foreign.Intl[name], original = Intl[name];
    var descriptor = Object.getOwnPropertyDescriptor(foreign.Intl, name);
    if (typeof local !== 'function' || local === original ||
        Object.getPrototypeOf(local) !== foreign.Function.prototype ||
        local.name !== original.name || local.length !== original.length ||
        !descriptor.writable || descriptor.enumerable || !descriptor.configurable) throw 'namespace callable';
    if (name === 'getCanonicalLocales') continue;
    var prototype = local.prototype;
    if (prototype === original.prototype || Object.getPrototypeOf(prototype) !== foreign.Object.prototype ||
        prototype.constructor !== local) throw 'prototype Realm';
    var ownPrototype = Object.getOwnPropertyDescriptor(local, 'prototype');
    if (ownPrototype.writable || ownPrototype.enumerable || ownPrototype.configurable) throw 'constructor prototype flags';
    var keys = Object.getOwnPropertyNames(prototype);
    if (keys.join(',') !== Object.getOwnPropertyNames(original.prototype).join(',')) throw 'prototype members';
    for (var key of keys) {
      if (key === 'constructor') continue;
      var property = Object.getOwnPropertyDescriptor(prototype,key);
      var counterpart = Object.getOwnPropertyDescriptor(original.prototype,key);
      var callable = property.get === undefined ? property.value : property.get;
      var source = counterpart.get === undefined ? counterpart.value : counterpart.get;
      if (typeof callable !== 'function' || callable === source ||
          Object.getPrototypeOf(callable) !== foreign.Function.prototype ||
          callable.name !== source.name || callable.length !== source.length ||
          property.enumerable !== counterpart.enumerable || property.configurable !== counterpart.configurable ||
          property.writable !== counterpart.writable || property.set !== counterpart.set) throw 'prototype callable';
    }
  }
  var supported = foreign.Intl.DateTimeFormat.supportedLocalesOf;
  if (supported === Intl.DateTimeFormat.supportedLocalesOf ||
      Object.getPrototypeOf(supported) !== foreign.Function.prototype) throw 'static callable Realm';
}
if (first.Intl === second.Intl || first.Intl.Locale === second.Intl.Locale ||
    first.Intl.Locale.prototype === second.Intl.Locale.prototype) throw 'fresh Realm intrinsics';
print(true);
"#,
    );
}

#[test]
fn intl_constructor_fallback_uses_the_new_target_realm() {
    assert_intl_realm(
        r#"
var foreign = __lilaCreateRealm().global;
var bound = foreign.Array.bind(null);
for (var name of ['Locale','DateTimeFormat']) {
  var constructor = Intl[name], other = foreign.Intl[name];
  var args = name === 'Locale' ? ['en'] : ['en-US', {timeZone:'UTC'}];
  var direct = Reflect.construct(other,args);
  if (Object.getPrototypeOf(direct) !== other.prototype) throw 'direct constructor Realm';
  for (var prototype of [undefined, null, false, 0, 'prototype', Symbol('prototype')]) {
    Object.defineProperty(bound, 'prototype', {value:prototype, writable:true, configurable:true});
    var value = Reflect.construct(constructor,args,bound);
    if (Object.getPrototypeOf(value) !== other.prototype) throw 'bound newTarget Realm';
    var proxy = new Proxy(new Proxy(bound,{}),{});
    value = Reflect.construct(constructor,args,proxy);
    if (Object.getPrototypeOf(value) !== other.prototype) throw 'proxy newTarget Realm';
  }
}
var plain = foreign.Intl.DateTimeFormat('en-US',{timeZone:'UTC'});
if (Object.getPrototypeOf(plain) !== foreign.Intl.DateTimeFormat.prototype) throw 'plain formatter call Realm';
print(true);
"#,
    );
}

#[test]
fn intl_prototype_selection_preserves_explicit_objects_and_abrupt_order() {
    assert_intl_realm(
        r#"
var foreign = __lilaCreateRealm().global;
var bound = foreign.Array.bind(null);
for (var prototype of [{}, [], function prototype() {}]) {
  var log = '', pair = Proxy.revocable(bound, {get(target,key) {
    if (key === 'prototype') {log += 'prototype;'; pair.revoke(); return prototype;}
    return Reflect.get(target,key);
  }});
  var locale = Reflect.construct(Intl.Locale,[{toString(){log += 'tag;'; return 'en';}}],pair.proxy);
  if (Object.getPrototypeOf(locale) !== prototype || log !== 'prototype;tag;') throw 'explicit prototype avoids Realm lookup';
}
var touched = false, pair = Proxy.revocable(bound, {get(target,key) {
  if (key === 'prototype') {pair.revoke(); return undefined;}
  return Reflect.get(target,key);
}}), received;
try {
  Reflect.construct(Intl.Locale,[{toString(){touched=true; return 'en';}}],pair.proxy);
} catch (error) {received=error;}
if (!(received instanceof TypeError) || received instanceof foreign.TypeError || touched) throw 'revoked Realm resolution order';
var sentinel = {}, target = new Proxy(bound,{get(){throw sentinel;}});
received=undefined;
try {Reflect.construct(Intl.Locale,[{toString(){touched=true; return 'en';}}],target);}
catch (error) {received=error;}
if (received !== sentinel || touched) throw 'prototype getter completion';
print(true);
"#,
    );
}

#[test]
fn locale_coercion_and_getter_errors_follow_the_active_builtin_realm() {
    assert_intl_realm(
        r#"
var foreign = __lilaCreateRealm().global;
var constructor = foreign.Intl.Locale;
for (var options of [null, {calendar:Symbol('calendar')}, {calendar:{toString(){return {};},valueOf(){return {};}}}]) {
  var received=undefined;
  try {new constructor('en',options);} catch (error) {received=error;}
  if (!(received instanceof foreign.TypeError) || received instanceof TypeError) throw 'constructor TypeError Realm';
}
for (var options of [{language:'abcd'}, {calendar:'ab'}, {variants:'1901-1901'}]) {
  var received=undefined;
  try {new constructor('en',options);} catch (error) {received=error;}
  if (!(received instanceof foreign.RangeError) || received instanceof RangeError) throw 'constructor RangeError Realm';
}
var boxed=false;
Object.defineProperty(foreign.String.prototype,'language',{configurable:true,get(){
  boxed=Object.getPrototypeOf(this)===foreign.String.prototype; return 'de';
}});
if (new constructor('en','options').language !== 'de' || !boxed) throw 'options boxing Realm';
var locale = new constructor('en-1901-u-ca-gregory');
for (var key of ['language','script','region','baseName','calendar','collation','firstDayOfWeek','hourCycle','caseFirst','numeric','numberingSystem','variants']) {
  var getter=Object.getOwnPropertyDescriptor(constructor.prototype,key).get;
  var received=undefined;
  try {getter.call(new Proxy(locale,{}));} catch (error) {received=error;}
  if (!(received instanceof foreign.TypeError) || received instanceof TypeError) throw 'getter TypeError Realm';
}
if (Intl.Locale.prototype.toString.call(locale) !== 'en-1901-u-ca-gregory') throw 'cross-Realm Locale slots';
print(true);
"#,
    );
}

#[test]
fn intl_fallback_reads_intrinsics_after_public_namespace_replacement() {
    assert_intl_realm(
        r#"
var foreign=__lilaCreateRealm().global;
var localePrototype=foreign.Intl.Locale.prototype;
var formatter=foreign.Intl.DateTimeFormat;
var formatterPrototype=formatter.prototype;
var target=foreign.Array.bind(null);
foreign.Intl.Locale=function Replacement(){};
foreign.Intl.DateTimeFormat=function Replacement(){};
Object.defineProperty(foreign,'Intl',{get(){throw 'mutable global read';},configurable:true});
var locale=Reflect.construct(Intl.Locale,['de'],target);
if (Object.getPrototypeOf(locale)!==localePrototype || locale.language!=='de') throw 'stored Locale intrinsic';
var value=Reflect.construct(Intl.DateTimeFormat,['en-US',{timeZone:'UTC'}],target);
if (Object.getPrototypeOf(value)!==formatterPrototype ||
    Object.getPrototypeOf(formatter())!==formatterPrototype) throw 'stored formatter intrinsic';
print(true);
"#,
    );
}
