use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_legacy_formatter(source: &str) {
    crate::configure_compilation_jobs();
    for prefix in ["", "'use strict';\n"] {
        let source = format!("{prefix}{source}");
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
            .expect("legacy Intl constructor control must execute through Wasm AOT");
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
}

#[test]
fn legacy_formatter_identity_descriptors_and_realm_symbol_ownership() {
    assert_legacy_formatter(
        r#"
var foreign = __lilaCreateRealm().global;
var constructors = [Intl.DateTimeFormat, Intl.NumberFormat,
                    foreign.Intl.DateTimeFormat, foreign.Intl.NumberFormat];
var localKey;
var foreignKey;
for (var i = 0; i < constructors.length; i++) {
  var C = constructors[i];
  var wrapper = Object.create(C.prototype);
  if (C.call(wrapper, 'en', {timeZone:'UTC'}) !== wrapper) throw 'wrapper identity';
  var keys = Object.getOwnPropertySymbols(wrapper);
  if (keys.length !== 1 || keys[0].description !== 'IntlLegacyConstructedSymbol') throw 'fallback symbol';
  var key = keys[0];
  var descriptor = Object.getOwnPropertyDescriptor(wrapper, key);
  if (descriptor.writable || descriptor.enumerable || descriptor.configurable) throw 'fallback attributes';
  var initialized = descriptor.value;
  if (initialized === wrapper || Object.getPrototypeOf(initialized) !== C.prototype) throw 'fresh hidden formatter';
  if (wrapper.format !== initialized.format || wrapper.resolvedOptions().locale !== initialized.resolvedOptions().locale) throw 'unwrap';
  if (typeof wrapper.format(0) !== 'string') throw 'bound formatter';
  if (i === 0) localKey = key;
  if (i === 1 && key !== localKey) throw 'services must share local symbol';
  if (i === 2) foreignKey = key;
  if (i === 3 && key !== foreignKey) throw 'services must share foreign symbol';
  if (i >= 2 && key === localKey) throw 'different Realms must not share symbol';
  var branded = new C('en', {timeZone:'UTC'});
  var originalFormat = branded.format;
  var originalLocale = branded.resolvedOptions().locale;
  if (C.call(branded, 'de', {timeZone:'UTC'}) !== branded) throw 'branded call identity';
  if (Object.getOwnPropertySymbols(branded)[0] !== key) throw 'branded fallback key';
  if (branded.format !== originalFormat || branded.resolvedOptions().locale !== originalLocale) throw 'branded receiver must bypass fallback';
  if (Object.getOwnPropertyDescriptor(branded,key).value.resolvedOptions().locale === originalLocale) throw 'hidden initialization';
  for (var receiver of [undefined, null, 1, 'x', {}]) {
    var ordinary = C.call(receiver, 'en', {timeZone:'UTC'});
    if (ordinary === receiver || Object.getPrototypeOf(ordinary) !== C.prototype || Object.getOwnPropertySymbols(ordinary).length !== 0) throw 'noninstance call';
  }
  var fresh = new C('en', {timeZone:'UTC'});
  if (Object.getOwnPropertySymbols(fresh).length !== 0) throw 'new must not chain';
  function Alternate() {}
  var alternate = Reflect.construct(C, ['en', {timeZone:'UTC'}], Alternate);
  if (Object.getPrototypeOf(alternate) !== Alternate.prototype || Object.getOwnPropertySymbols(alternate).length !== 0) throw 'newTarget must not chain';
}
print(true);
"#,
    );
}

#[test]
fn legacy_formatter_chain_initializes_before_proxy_hooks_and_ignores_has_instance() {
    assert_legacy_formatter(
        r#"
for (var C of [Intl.DateTimeFormat, Intl.NumberFormat]) {
  var marker = {};
  var trace = '';
  var target = Object.create(C.prototype);
  var proxy = new Proxy(target, {
    getPrototypeOf: function(t) { trace += 'p'; return Reflect.getPrototypeOf(t); },
    defineProperty: function(t, key, descriptor) {
      trace += 'd';
      if (typeof key !== 'symbol' || key.description !== 'IntlLegacyConstructedSymbol' ||
          descriptor.writable || descriptor.enumerable || descriptor.configurable ||
          Object.getPrototypeOf(descriptor.value) !== C.prototype) throw 'define descriptor';
      throw marker;
    }
  });
  var abruptOptions = {get localeMatcher() { trace += 'o'; throw marker; }};
  try { C.call(proxy, [], abruptOptions); throw 'lost options throw'; }
  catch (error) { if (error !== marker || trace !== 'o') throw 'options must precede proxy'; }
  trace = '';
  var options = {get localeMatcher() { trace += 'o'; return 'lookup'; }};
  try { C.call(proxy, [], options); throw 'lost define throw'; }
  catch (error) { if (error !== marker || trace !== 'opd') throw 'chain observation order'; }
  trace = '';
  var prototypeThrow = new Proxy(target, {getPrototypeOf: function() { trace += 'p'; throw marker; }});
  try { C.call(prototypeThrow, [], options); throw 'lost prototype throw'; }
  catch (error) { if (error !== marker || trace !== 'op') throw 'prototype abrupt order'; }
  trace = '';
  var falseDefine = new Proxy(target, {
    getPrototypeOf: function(t) { trace += 'p'; return Reflect.getPrototypeOf(t); },
    defineProperty: function() { trace += 'd'; return false; }
  });
  try { C.call(falseDefine, [], options); throw 'accepted false define'; }
  catch (error) { if (!(error instanceof TypeError) || trace !== 'opd') throw 'DefinePropertyOrThrow'; }
  var wrapper = Object.create(C.prototype);
  Object.defineProperty(C, Symbol.hasInstance, {configurable:true, get:function() {throw marker;}});
  try {
    if (C.call(wrapper, 'en', {timeZone:'UTC'}) !== wrapper) throw 'OrdinaryHasInstance chain';
    var getter = Object.getOwnPropertyDescriptor(C.prototype, 'format').get;
    if (typeof getter.call(wrapper)(0) !== 'string' || typeof C.prototype.resolvedOptions.call(wrapper).locale !== 'string') throw 'OrdinaryHasInstance unwrap';
  } finally { delete C[Symbol.hasInstance]; }
  try { C.call(wrapper, [], options); throw 'accepted existing nonconfigurable fallback'; }
  catch (error) { if (!(error instanceof TypeError)) throw error; }
}
print(true);
"#,
    );
}

#[test]
fn legacy_formatter_unwrap_uses_original_get_and_parts_ranges_keep_brand_checks() {
    assert_legacy_formatter(
        r#"
for (var C of [Intl.DateTimeFormat, Intl.NumberFormat]) {
  var wrapper = Object.create(C.prototype);
  C.call(wrapper, 'en', {timeZone:'UTC'});
  var key = Object.getOwnPropertySymbols(wrapper)[0];
  var initialized = wrapper[key];
  var getter = Object.getOwnPropertyDescriptor(C.prototype, 'format').get;
  var trace = '';
  var accessor = Object.create(C.prototype);
  var proxy;
  Object.defineProperty(accessor, key, {get:function() {
    trace += 'a';
    if (this !== proxy) throw 'Get receiver';
    return initialized;
  }});
  proxy = new Proxy(accessor, {get:function(t, k, r) {
    if (k === key) {
      trace += 'g';
      if (r !== proxy) throw 'proxy Get receiver';
    }
    return Reflect.get(t, k, r);
  }});
  if (C.prototype.resolvedOptions.call(proxy).locale !== initialized.resolvedOptions().locale || trace !== 'ga') throw 'resolvedOptions ordinary Get';
  trace = '';
  if (getter.call(proxy) !== initialized.format || trace !== 'ga') throw 'format ordinary Get';
  var marker = {};
  var abrupt = new Proxy(wrapper, {get:function(t,k,r) { if (k === key) throw marker; return Reflect.get(t,k,r); }});
  for (var operation of [getter, C.prototype.resolvedOptions]) {
    try { operation.call(abrupt); throw 'lost fallback Get throw'; }
    catch (error) { if (error !== marker) throw 'changed Get abrupt'; }
  }
  var direct = new Proxy(wrapper, {
    getPrototypeOf:function() {throw marker;},
    get:function() {throw marker;}
  });
  var poison = {valueOf:function() {throw marker;}};
  for (var method of [C.prototype.formatToParts, C.prototype.formatRange, C.prototype.formatRangeToParts]) {
    try { method.call(direct, poison, poison); throw 'accepted legacy parts or range receiver'; }
    catch (error) { if (!(error instanceof TypeError)) throw 'parts/range observed fallback or endpoints'; }
    try { method.call(wrapper, poison, poison); throw 'accepted wrapper internal slot'; }
    catch (error) { if (!(error instanceof TypeError)) throw 'wrapper parts/range order'; }
  }
  var invalid = Object.create(C.prototype);
  Object.defineProperty(invalid, key, {value:1});
  for (var receiver of [undefined, null, true, 1, 'x', Symbol('x'), 1n, {}, Object.create(C.prototype), invalid]) {
    for (var operation of [getter, C.prototype.resolvedOptions]) {
      try { operation.call(receiver); throw 'accepted invalid formatter'; }
      catch (error) { if (!(error instanceof TypeError)) throw error; }
    }
  }
}
print(true);
"#,
    );
}

#[test]
fn legacy_formatter_constructor_and_method_realms_use_intrinsics() {
    assert_legacy_formatter(
        r#"
var foreign = __lilaCreateRealm().global;
var locals = [Intl.DateTimeFormat, Intl.NumberFormat];
var foreigners = [foreign.Intl.DateTimeFormat, foreign.Intl.NumberFormat];
var foreignTypeErrorPrototype = foreign.TypeError.prototype;
foreign.Intl = null;
for (var i = 0; i < locals.length; i++) {
  var local = locals[i];
  var other = foreigners[i];
  var localGetter = Object.getOwnPropertyDescriptor(local.prototype, 'format').get;
  var foreignGetter = Object.getOwnPropertyDescriptor(other.prototype, 'format').get;
  var localWrapper = Object.create(local.prototype);
  var fresh = other.call(localWrapper, 'en', {timeZone:'UTC'});
  if (fresh === localWrapper || Object.getPrototypeOf(fresh) !== other.prototype || Object.getOwnPropertySymbols(localWrapper).length !== 0) throw 'constructor Realm instance test';
  var foreignWrapper = Object.create(other.prototype);
  if (other.call(foreignWrapper, 'en', {timeZone:'UTC'}) !== foreignWrapper) throw 'mutable public Intl';
  var foreignKey = Object.getOwnPropertySymbols(foreignWrapper)[0];
  if (typeof foreignGetter.call(foreignWrapper)(0) !== 'string') throw 'foreign method owner';
  try { localGetter.call(foreignWrapper); throw 'local method accepted foreign wrapper'; }
  catch (error) { if (Object.getPrototypeOf(error) !== TypeError.prototype) throw 'local receiver error Realm'; }
  local.call(localWrapper, 'en', {timeZone:'UTC'});
  var localKey = Object.getOwnPropertySymbols(localWrapper)[0];
  if (localKey === foreignKey) throw 'cross-Realm key collision';
  try { other.prototype.resolvedOptions.call(localWrapper); throw 'foreign method accepted local wrapper'; }
  catch (error) { if (Object.getPrototypeOf(error) !== foreignTypeErrorPrototype) throw 'foreign receiver error Realm'; }
  var brandedForeign = new other('en', {timeZone:'UTC'});
  if (typeof localGetter.call(brandedForeign)(0) !== 'string') throw 'branded cross-Realm receiver';
}
print(true);
"#,
    );
}
