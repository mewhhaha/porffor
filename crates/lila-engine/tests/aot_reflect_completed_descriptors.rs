use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_definition_output(source: &str) {
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
        .expect("property definitions must execute through Wasm AOT");
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
fn private_data_and_accessor_forwarding_does_not_observe_inherited_descriptor_fields() {
    assert_definition_output(
        r#"
var define = Reflect.defineProperty;
var attributes = Object.create(null), target = {}, reads = 0, inherited = 0;
Object.defineProperty(attributes, 'value', {get() { reads++; return 7; }});
attributes.writable = true;
attributes.enumerable = true;
attributes.configurable = true;
var poison = Object.create(null);
poison.get = function() { inherited++; throw 'private descriptor escaped'; };
poison.configurable = true;
var direct, borrowed;
try {
  Object.defineProperty(Object.prototype, 'get', poison);
  Object.defineProperty(Object.prototype, 'set', poison);
  direct = Reflect.defineProperty(target, 'direct', attributes);
  borrowed = define(target, 'borrowed', attributes);
} finally {
  delete Object.prototype.get;
  delete Object.prototype.set;
}
assert(direct === true && borrowed === true, 'ordinary data definitions');
assert(reads === 2 && inherited === 0, 'only original descriptor reads');
assert(target.direct === 7 && target.borrowed === 7, 'data values');
var property = Object.getOwnPropertyDescriptor(target, 'direct');
assert(property.writable && property.enumerable && property.configurable, 'data attributes');

var getter = function() { return 11; };
attributes = Object.create(null);
attributes.get = getter;
attributes.set = undefined;
attributes.enumerable = true;
attributes.configurable = true;
var accessorResult;
try {
  Object.defineProperty(Object.prototype, 'value', poison);
  Object.defineProperty(Object.prototype, 'writable', poison);
  accessorResult = define(target, 'accessor', attributes);
} finally {
  delete Object.prototype.value;
  delete Object.prototype.writable;
}
property = Object.getOwnPropertyDescriptor(target, 'accessor');
assert(accessorResult === true && inherited === 0, 'ordinary accessor definition');
assert(property.get === getter && property.set === undefined, 'accessor identities');
assert(property.enumerable && property.configurable && target.accessor === 11, 'accessor attributes');
print('ok');
"#,
    );
}

#[test]
fn absent_and_null_proxy_traps_forward_only_the_converted_descriptor() {
    assert_definition_output(
        r#"
var target = {}, attributes = Object.create(null), reads = 0, traps = 0, inherited = 0;
Object.defineProperty(attributes, 'value', {get() { reads++; return 13; }});
attributes.writable = true;
attributes.enumerable = true;
attributes.configurable = true;
var inner = new Proxy(target, {defineProperty: null});
var outer = new Proxy(inner, {get defineProperty() { traps++; return undefined; }});
var poison = Object.create(null);
poison.get = function() { inherited++; throw undefined; };
poison.configurable = true;
var result;
try {
  Object.defineProperty(Object.prototype, 'get', poison);
  result = Reflect.defineProperty(outer, 'answer', attributes);
} finally {
  delete Object.prototype.get;
}
assert(result === true && target.answer === 13, 'nested unhandled Proxy definition');
assert(reads === 1 && traps === 1 && inherited === 0, 'no descriptor reconversion effects');
print('ok');
"#,
    );
}

#[test]
fn callable_proxy_traps_retain_descriptor_identity_and_the_method_realm_prototype() {
    assert_definition_output(
        r#"
var other = __lilaCreateRealm().global;
var methods = [Reflect.defineProperty, other.Reflect.defineProperty];
var prototypes = [Object.prototype, other.Object.prototype];
var replies = [true, false, undefined];
for (var realm = 0; realm < methods.length; realm++) {
  for (var reply = 0; reply < replies.length; reply++) {
    var retained, during, calls = 0, target = {}, handler;
    var attributes = Object.create(null);
    attributes.value = 17;
    attributes.writable = true;
    attributes.enumerable = true;
    attributes.configurable = true;
    handler = {defineProperty(actualTarget, key, descriptor) {
      calls++;
      assert(this === handler && actualTarget === target && key === 'answer', 'trap arguments');
      retained = descriptor;
      during = Object.getPrototypeOf(descriptor);
      return replies[reply];
    }};
    var proxy = new Proxy(target, handler);
    if (reply === 1) proxy = new Proxy(proxy, {});
    var result = methods[realm](proxy, 'answer', attributes);
    assert(result === (reply === 0) && calls === 1, 'truthy and false trap outcomes');
    assert(retained !== attributes && during === prototypes[realm], 'fresh method-Realm descriptor');
    assert(Object.getPrototypeOf(retained) === prototypes[realm], 'retained prototype');
    assert(retained.value === 17 && retained.writable && retained.enumerable && retained.configurable, 'retained fields');
    assert(!Object.hasOwn(retained, 'get') && !Object.hasOwn(retained, 'set'), 'absent accessor fields');
    assert(!Object.hasOwn(target, 'answer'), 'trap controls target mutation');
    Reflect.defineProperty({}, 'later', {value: 1});
    assert(Object.getPrototypeOf(retained) === prototypes[realm], 'later private forwarding preserves escaped object');
  }
}
print('ok');
"#,
    );
}

#[test]
fn actual_descriptor_and_proxy_hook_abrupts_keep_identity_and_observation_order() {
    assert_definition_output(
        r#"
var reasons = [{}, undefined];
for (var reason of reasons) {
  var log = '', observed = false, trapCalls = 0;
  var target = new Proxy({}, {defineProperty() { trapCalls++; return true; }});
  var key = {[Symbol.toPrimitive](hint) { log += hint + ','; return 'answer'; }};
  var attributes = Object.create(null);
  Object.defineProperty(attributes, 'enumerable', {get() { log += 'enumerable'; throw reason; }});
  try { Reflect.defineProperty(target, key, attributes); }
  catch (error) { observed = true; assert(error === reason, 'input getter abrupt identity'); }
  assert(observed && log === 'string,enumerable' && trapCalls === 0, 'key and descriptor order');

  observed = false;
  attributes = new Proxy({}, {has(object, property) {
    assert(property === 'enumerable', 'first descriptor presence check');
    throw reason;
  }});
  try { Reflect.defineProperty(target, 'answer', attributes); }
  catch (error) { observed = true; assert(error === reason, 'input has abrupt identity'); }
  assert(observed && trapCalls === 0, 'descriptor has before target trap');

  for (var site = 0; site < 2; site++) {
    var handler = site === 0
      ? {get defineProperty() { throw reason; }}
      : {defineProperty() { throw reason; }};
    var proxy = new Proxy(new Proxy({}, handler), {});
    observed = false;
    try { Reflect.defineProperty(proxy, 'answer', {value: 1}); }
    catch (error) { observed = true; assert(error === reason, 'target hook abrupt identity'); }
    assert(observed, 'target hook must throw');
  }

  var poison = Object.create(null);
  poison.get = function() { throw reason; };
  poison.configurable = true;
  observed = false;
  try {
    Object.defineProperty(Object.prototype, 'get', poison);
    try { Reflect.defineProperty({}, 'answer', {}); }
    catch (error) { observed = true; assert(error === reason, 'actual inherited descriptor field'); }
  } finally {
    delete Object.prototype.get;
  }
  assert(observed, 'real input inherits descriptor getters');
}
print('ok');
"#,
    );
}

#[test]
fn typed_array_value_coercion_preserves_exact_abrupts_and_successful_conversions() {
    assert_definition_output(
        r#"
var methods = [Reflect.defineProperty, Object.defineProperty];
var reasons = [{}, undefined], calls = 0;
for (var method of methods) {
  for (var bigint = 0; bigint < 2; bigint++) {
    for (var reason of reasons) {
      var target = bigint ? new BigInt64Array(1) : new Uint8Array(1);
      var observed = false;
      var value = {valueOf() { calls++; throw reason; }};
      try { method(new Proxy(new Proxy(target, {}), {}), '0', {value}); }
      catch (error) { observed = true; assert(error === reason, 'coercion abrupt identity'); }
      assert(observed && target[0] === (bigint ? 0n : 0), 'coercion throw is not rejection');
    }
  }
}
assert(calls === 8, 'each Number and BigInt coercion runs once');
var bytes = new Uint8Array(1), words = new BigInt64Array(1);
assert(Reflect.defineProperty(bytes, '0', {value: {valueOf() { return 257; }}}) === true, 'numeric value definition');
assert(Object.defineProperty(words, '0', {value: {valueOf() { return 257n; }}}) === words, 'BigInt value definition');
assert(bytes[0] === 1 && words[0] === 257n, 'converted element values');
var ordinaryValue = {valueOf() { throw 'not an element'; }};
assert(Reflect.defineProperty(bytes, 'label', {value: ordinaryValue}) === true, 'ordinary TypedArray property');
assert(bytes.label === ordinaryValue, 'ordinary property does not coerce');

var other = __lilaCreateRealm().global, observed = false;
try { other.Reflect.defineProperty(bytes, '0', {value: Symbol('number')}); }
catch (error) { observed = true; assert(error instanceof other.TypeError && !(error instanceof TypeError), 'borrowed Reflect coercion Realm'); }
assert(observed, 'Symbol Number conversion throws');
observed = false;
try { Reflect.defineProperty(words, '0', {value: 1}); }
catch (error) { observed = true; assert(error instanceof TypeError, 'BigInt content mismatch TypeError'); }
assert(observed, 'Number BigInt conversion throws');
print('ok');
"#,
    );
}

#[test]
fn ordinary_rejection_and_typed_array_index_rejection_remain_normal_false() {
    assert_definition_output(
        r#"
var calls = 0, value = {valueOf() { calls++; throw 'unexpected conversion'; }};
var target = new Uint8Array(1);
for (var key of ['-0', '-1', '0.5', '1', 'NaN', 'Infinity', '-Infinity']) {
  assert(Reflect.defineProperty(target, key, {value}) === false, 'invalid numeric index');
  var observed = false;
  try { Object.defineProperty(target, key, {value}); }
  catch (error) { observed = true; assert(error instanceof TypeError, 'Object rejects invalid index'); }
  assert(observed, 'Object rejection becomes TypeError');
}
for (var descriptor of [
  {get: undefined}, {set: undefined}, {value, writable: false},
  {value, enumerable: false}, {value, configurable: false}
]) {
  assert(Reflect.defineProperty(target, '0', descriptor) === false, 'incompatible index descriptor');
}
var observed = false;
try { Reflect.defineProperty(target, '1', {value, get: undefined}); }
catch (error) { observed = true; assert(error instanceof TypeError, 'invalid input descriptor throws'); }
assert(observed && calls === 0, 'descriptor validation and rejection precede value coercion');
assert(Reflect.defineProperty(target, '0', {}) === true, 'empty descriptor');
assert(Reflect.defineProperty(target, '0', {value: 5, writable: 1, enumerable: {}, configurable: 'yes'}) === true, 'normalized Boolean fields');
assert(target[0] === 5, 'accepted descriptor writes');
Object.preventExtensions(target);
assert(Reflect.defineProperty(target, '0', {value: 6}) === true && target[0] === 6, 'existing nonextensible index');
assert(Reflect.defineProperty(target, 'newName', {value: 7}) === false, 'nonextensible ordinary key');
var detached = new Uint8Array(1);
__lilaDetachArrayBuffer(detached.buffer);
assert(Reflect.defineProperty(detached, '0', {value}) === false && calls === 0, 'initially detached index');
var plain = {};
Object.defineProperty(plain, 'fixed', {value: 1});
assert(Reflect.defineProperty(plain, 'fixed', {value: 2}) === false && plain.fixed === 1, 'incompatible ordinary descriptor');
Object.preventExtensions(plain);
assert(Reflect.defineProperty(plain, 'missing', {value: 3}) === false, 'missing ordinary nonextensible property');
print('ok');
"#,
    );
}

#[test]
fn typed_array_definition_rechecks_the_buffer_after_value_coercion() {
    assert_definition_output(
        r#"
var methods = [Reflect.defineProperty, Object.defineProperty];
for (var index = 0; index < methods.length; index++) {
  var define = methods[index];
  for (var bigint = 0; bigint < 2; bigint++) {
    var target = bigint ? new BigInt64Array(1) : new Uint8Array(1), calls = 0;
    var result = define(target, '0', {value: {valueOf() {
      calls++;
      __lilaDetachArrayBuffer(target.buffer);
      return bigint ? 9n : 9;
    }}});
    assert(result === (index === 0 ? true : target), 'successful coercion after detachment');
    assert(calls === 1 && target.length === 0, 'detachment suppresses element write');
  }
  var buffer = new ArrayBuffer(2, {maxByteLength: 4});
  target = new Uint8Array(buffer, 0, 2);
  var result = define(target, '1', {value: {valueOf() { buffer.resize(0); return 5; }}});
  assert(result === (index === 0 ? true : target) && target.length === 0, 'resize after successful coercion');
  buffer.resize(2);
  assert(target[1] === 0, 'out-of-bounds conversion did not write');
  result = define(target, '1', {value: {valueOf() {
    buffer.resize(0);
    buffer.resize(2);
    return 6;
  }}});
  assert(result === (index === 0 ? true : target) && target[1] === 6, 'write uses refreshed storage');
}
print('ok');
"#,
    );
}

#[test]
fn object_definition_forwards_private_descriptors_through_nullish_proxy_layers() {
    assert_definition_output(
        r#"
var target = {}, reads = 0, inherited = 0, trapReads = 0;
var proxy = new Proxy(new Proxy(target, {defineProperty: null}), {
  get defineProperty() { trapReads++; return undefined; }
});
var attributes = Object.create(null);
Object.defineProperty(attributes, 'value', {get() { reads++; return 7; }});
attributes.writable = true;
attributes.enumerable = true;
attributes.configurable = true;
var poison = Object.create(null);
poison.get = function() { inherited++; throw 'private descriptor reread'; };
poison.configurable = true;
var result;
try {
  Object.defineProperty(Object.prototype, 'get', poison);
  Object.defineProperty(Object.prototype, 'set', poison);
  result = Object.defineProperty(proxy, 'value', attributes);
} finally {
  delete Object.prototype.get;
  delete Object.prototype.set;
}
assert(result === proxy && target.value === 7, 'Object keeps original target identity');
assert(reads === 1 && inherited === 0 && trapReads === 1, 'Object reads original descriptor once');
attributes = Object.create(null);
var getter = function() { return 19; };
attributes.get = getter;
attributes.set = undefined;
attributes.configurable = true;
try {
  Object.defineProperty(Object.prototype, 'value', poison);
  Object.defineProperty(Object.prototype, 'writable', poison);
  result = Object.defineProperty(proxy, 'computed', attributes);
} finally {
  delete Object.prototype.value;
  delete Object.prototype.writable;
}
assert(result === proxy && target.computed === 19 && inherited === 0, 'Object forwards accessor descriptor');
assert(Object.getOwnPropertyDescriptor(target, 'computed').get === getter, 'Object keeps accessor identity');
print('ok');
"#,
    );
}

#[test]
fn object_callable_traps_keep_exposed_descriptors_after_success_or_rejection() {
    assert_definition_output(
        r#"
var other = __lilaCreateRealm().global;
var methods = [Object.defineProperty, other.Object.defineProperty];
var prototypes = [Object.prototype, other.Object.prototype];
var errors = [TypeError, other.TypeError];
var replies = [true, false, undefined];
for (var realm = 0; realm < methods.length; realm++) {
  for (var reply = 0; reply < replies.length; reply++) {
    var retained, during, calls = 0, observed = false, result;
    var target = {}, attributes = Object.create(null);
    attributes.value = 23;
    attributes.configurable = true;
    var proxy = new Proxy(new Proxy(target, {defineProperty(actualTarget, key, descriptor) {
      calls++;
      retained = descriptor;
      during = Object.getPrototypeOf(descriptor);
      return replies[reply];
    }}), {});
    try { result = methods[realm](proxy, 'value', attributes); }
    catch (error) { observed = true; assert(error instanceof errors[realm], 'Object trap rejection Realm'); }
    assert(calls === 1 && retained !== attributes, 'one exposed Object descriptor');
    assert(observed === (reply !== 0), 'Object false and undefined trap returns reject');
    if (reply === 0) assert(result === proxy, 'Object successful trap returns original target');
    assert(during === prototypes[realm] && Object.getPrototypeOf(retained) === prototypes[realm], 'Object keeps exposed descriptor prototype');
    assert(retained.value === 23 && retained.configurable, 'Object keeps exposed descriptor fields');
    Object.defineProperty(new Proxy(new Proxy({}, {}), {}), 'later', {value: 1});
    assert(Object.getPrototypeOf(retained) === prototypes[realm], 'later Object forwarding cannot mutate retained descriptor');
  }
}
print('ok');
"#,
    );
}
