//! 10.5.5 Proxy `[[GetOwnProperty]]` as `Object.getOwnPropertyDescriptor`
//! and `Reflect.getOwnPropertyDescriptor` observe it.
//!
//! The trap result used to be returned as-is, with only its own
//! `configurable`/`writable` data fields consulted. It must instead be
//! converted with ToPropertyDescriptor (observable HasProperty/Get reads),
//! completed, validated against the target, and republished as a fresh object.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

const HELPERS: &str = r#"
function describe(descriptor) {
  if (descriptor === undefined) return 'undefined';
  if (Object.getPrototypeOf(descriptor) !== Object.prototype) throw 'descriptor prototype';
  var parts = [];
  var keys = Object.keys(descriptor);
  for (var index = 0; index < keys.length; index++) {
    var value = descriptor[keys[index]];
    parts.push(keys[index] + '=' + (typeof value === 'function' ? 'fn:' + value.name : String(value)));
  }
  return parts.join(',');
}
function typeErrorOf(action) {
  try { action(); } catch (error) { return error instanceof TypeError ? 'TypeError' : 'other:' + error; }
  return 'no error';
}
function trapReturning(target, result) {
  return new Proxy(target, { getOwnPropertyDescriptor: function() { return result; } });
}
"#;

fn assert_prints(body: &str, expected: &[&str]) {
    let source = format!("{HELPERS}{body}");
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
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
        .expect("Proxy descriptor fixture compiles and executes through Wasm AOT");
    assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "{body}"
    );
}

#[test]
fn partial_trap_results_complete_into_fresh_descriptor_objects() {
    assert_prints(
        r#"
var partial = {value: 1, configurable: true};
var proxy = trapReturning({}, partial);
var first = Object.getOwnPropertyDescriptor(proxy, 'x');
var second = Reflect.getOwnPropertyDescriptor(proxy, 'x');
print(first === partial, second === partial, first === second);
print(describe(first));
print(JSON.stringify(second));
print(describe(Object.getOwnPropertyDescriptor(trapReturning({}, {configurable: true}), 'x')));
function getter() { return 1; }
print(describe(Object.getOwnPropertyDescriptor(trapReturning({}, {get: getter, configurable: true}), 'x')));
print(describe(Object.getOwnPropertyDescriptor(
  trapReturning({}, Object.create({value: 'inherited', enumerable: 1, configurable: 'yes'})), 'x')));
var callable = function carrier() {};
callable.configurable = true;
callable.value = 'from a function';
print(describe(Object.getOwnPropertyDescriptor(trapReturning({}, callable), 'x')));
var array = [];
array.configurable = true;
array.writable = 0;
print(describe(Object.getOwnPropertyDescriptor(trapReturning({}, array), 'x')));
print(describe(Object.getOwnPropertyDescriptor(trapReturning({}, undefined), 'x')));
print(Object.keys(Object.getOwnPropertyDescriptors(trapReturning({}, partial))).length);
"#,
        &[
            "false false false",
            "value=1,writable=false,enumerable=false,configurable=true",
            "{\"value\":1,\"writable\":false,\"enumerable\":false,\"configurable\":true}",
            "value=undefined,writable=false,enumerable=false,configurable=true",
            "get=fn:getter,set=undefined,enumerable=false,configurable=true",
            "value=inherited,writable=false,enumerable=true,configurable=true",
            "value=from a function,writable=false,enumerable=false,configurable=true",
            "value=undefined,writable=false,enumerable=false,configurable=true",
            "undefined",
            "0",
        ],
    );
}

#[test]
fn trap_result_fields_are_read_after_target_checks_in_to_property_descriptor_order() {
    assert_prints(
        r#"
var log = [];
var fields = {enumerable: true, configurable: true, value: 2};
var observed = new Proxy(fields, {
  has: function(target, key) { log.push('has:' + key); return key in target; },
  get: function(target, key) { log.push('get:' + String(key)); return target[key]; }
});
var target = new Proxy({}, {
  getOwnPropertyDescriptor: function() { log.push('target getOwnPropertyDescriptor'); return undefined; },
  isExtensible: function(t) { log.push('target isExtensible'); return Reflect.isExtensible(t); }
});
var proxy = new Proxy(target, {
  getOwnPropertyDescriptor: function(t, key) {
    log.push('trap:' + key + ':' + (t === target));
    return observed;
  }
});
var descriptor = Object.getOwnPropertyDescriptor(proxy, 'k');
print(log.join(' '));
print(describe(descriptor));

var order = [];
var accessors = {
  get enumerable() { order.push('enumerable'); return 1; },
  get configurable() { order.push('configurable'); return 1; },
  get value() { order.push('value'); return 'v'; },
  get writable() { order.push('writable'); return 0; }
};
print(describe(Object.getOwnPropertyDescriptor(trapReturning({}, accessors), 'x')), order.join(','));
"#,
        &[
            "trap:k:true target getOwnPropertyDescriptor target isExtensible has:enumerable get:enumerable has:configurable get:configurable has:value get:value has:writable has:get has:set",
            "value=2,writable=false,enumerable=true,configurable=true",
            "value=v,writable=false,enumerable=true,configurable=true enumerable,configurable,value,writable",
        ],
    );
}

#[test]
fn trap_results_that_break_proxy_invariants_throw_type_errors() {
    assert_prints(
        r#"
function getOwn(target, result) {
  return function() { Object.getOwnPropertyDescriptor(trapReturning(target, result), 'x'); };
}
function frozenData(value, writable) {
  var target = {};
  Object.defineProperty(target, 'x', {value: value, writable: writable, enumerable: true, configurable: false});
  return target;
}
function g1() {}
function g2() {}
var accessorTarget = {};
Object.defineProperty(accessorTarget, 'x', {get: g1, enumerable: false, configurable: false});
var sealedEmpty = Object.preventExtensions({});
var sealedConfigurable = Object.preventExtensions({x: 1});

var cases = [
  ['number result', getOwn({}, 1)],
  ['null result', getOwn({}, null)],
  ['string result', getOwn({}, 'x')],
  ['undefined for non-configurable', getOwn(frozenData(1, true), undefined)],
  ['undefined on non-extensible', getOwn(sealedConfigurable, undefined)],
  ['new property on non-extensible', getOwn(sealedEmpty, {value: 1, configurable: true})],
  ['non-configurable for missing', getOwn({}, {value: 1, configurable: false})],
  ['non-configurable for configurable', getOwn({x: 1}, {value: 1, writable: true, enumerable: true, configurable: false})],
  ['configurable for non-configurable', getOwn(frozenData(1, true), {value: 1, writable: true, enumerable: true, configurable: true})],
  ['enumerable mismatch', getOwn(frozenData(1, true), {value: 1, writable: true, enumerable: false, configurable: false})],
  ['accessor for data', getOwn(frozenData(1, true), {get: g1, enumerable: true, configurable: false})],
  ['data for accessor', getOwn(accessorTarget, {value: 1, enumerable: false, configurable: false})],
  ['other getter', getOwn(accessorTarget, {get: g2, enumerable: false, configurable: false})],
  ['setter added', getOwn(accessorTarget, {get: g1, set: g2, enumerable: false, configurable: false})],
  ['frozen value changed', getOwn(frozenData(1, false), {value: 2, writable: false, enumerable: true, configurable: false})],
  ['frozen -0 for +0', getOwn(frozenData(0, false), {value: -0, writable: false, enumerable: true, configurable: false})],
  ['frozen reported writable', getOwn(frozenData(1, false), {value: 1, writable: true, enumerable: true, configurable: false})],
  ['writable reported non-writable', getOwn(frozenData(1, true), {value: 1, writable: false, enumerable: true, configurable: false})],
  ['uncallable getter', getOwn({}, {get: 1, configurable: true})],
  ['mixed descriptor', getOwn({}, {get: g1, value: 1, configurable: true})],
  ['revoked', function() {
    var pair = Proxy.revocable({}, {});
    pair.revoke();
    Object.getOwnPropertyDescriptor(pair.proxy, 'x');
  }],
  ['uncallable trap', function() {
    Object.getOwnPropertyDescriptor(new Proxy({}, {getOwnPropertyDescriptor: 1}), 'x');
  }]
];
for (var index = 0; index < cases.length; index++) {
  print(cases[index][0] + ': ' + typeErrorOf(cases[index][1]));
}

// The same checks accept every compatible report.
print(describe(Object.getOwnPropertyDescriptor(trapReturning(sealedEmpty, undefined), 'x')));
print(describe(Object.getOwnPropertyDescriptor(
  trapReturning(frozenData(NaN, false), {value: NaN, writable: false, enumerable: true, configurable: false}), 'x')));
print(describe(Object.getOwnPropertyDescriptor(
  trapReturning(accessorTarget, {get: g1, configurable: false}), 'x')));
print(describe(Reflect.getOwnPropertyDescriptor(
  trapReturning(frozenData(1, true), {value: 5, writable: true, enumerable: true, configurable: false}), 'x')));
"#,
        &[
            "number result: TypeError",
            "null result: TypeError",
            "string result: TypeError",
            "undefined for non-configurable: TypeError",
            "undefined on non-extensible: TypeError",
            "new property on non-extensible: TypeError",
            "non-configurable for missing: TypeError",
            "non-configurable for configurable: TypeError",
            "configurable for non-configurable: TypeError",
            "enumerable mismatch: TypeError",
            "accessor for data: TypeError",
            "data for accessor: TypeError",
            "other getter: TypeError",
            "setter added: TypeError",
            "frozen value changed: TypeError",
            "frozen -0 for +0: TypeError",
            "frozen reported writable: TypeError",
            "writable reported non-writable: TypeError",
            "uncallable getter: TypeError",
            "mixed descriptor: TypeError",
            "revoked: TypeError",
            "uncallable trap: TypeError",
            "undefined",
            "value=NaN,writable=false,enumerable=true,configurable=false",
            "get=fn:g1,set=undefined,enumerable=false,configurable=false",
            "value=5,writable=true,enumerable=true,configurable=false",
        ],
    );
}

#[test]
fn nested_proxies_forward_or_trap_at_every_level_and_engine_callers_see_the_completion() {
    assert_prints(
        r#"
var log = [];
var inner = new Proxy({x: 1}, {
  getOwnPropertyDescriptor: function(t, key) {
    log.push('inner');
    return Reflect.getOwnPropertyDescriptor(t, key);
  }
});
var forwarding = new Proxy(inner, {});
var nullTrap = new Proxy(forwarding, {getOwnPropertyDescriptor: null});
print(describe(Object.getOwnPropertyDescriptor(nullTrap, 'x')), log.join(','));
log = [];
var outer = new Proxy(inner, {
  getOwnPropertyDescriptor: function(t, key) {
    log.push('outer');
    return {value: 1, writable: true, enumerable: true, configurable: true};
  }
});
print(describe(Object.getOwnPropertyDescriptor(outer, 'x')), log.join(','));

// [[GetOwnProperty]] callers see the completed descriptor: an accessor-valued
// `enumerable` on the trap result counts, and an absent one is false.
var source = new Proxy({a: 1, b: 2, c: 3}, {
  getOwnPropertyDescriptor: function(t, key) {
    if (key === 'c') return {value: t[key], configurable: true};
    return {get enumerable() { return key === 'a'; }, configurable: true, value: t[key]};
  }
});
print(Object.keys(source).join(','), JSON.stringify(Object.entries(source)), JSON.stringify(Object.assign({}, source)));
var hasOwn = new Proxy({}, {getOwnPropertyDescriptor: function() { return {value: 0, configurable: true}; }});
print(Object.prototype.hasOwnProperty.call(hasOwn, 'anything'), Object.hasOwn(hasOwn, 'anything'));
"#,
        &[
            "value=1,writable=true,enumerable=true,configurable=true inner",
            "value=1,writable=true,enumerable=true,configurable=true outer,inner",
            "a [[\"a\",1]] {\"a\":1}",
            "true true",
        ],
    );
}

#[test]
fn descriptors_come_from_the_executing_builtins_realm() {
    assert_prints(
        r#"
var other = __lilaCreateRealm().global;
var proxy = new Proxy({}, {getOwnPropertyDescriptor: function() { return {value: 1, configurable: true}; }});
var local = Object.getOwnPropertyDescriptor(proxy, 'x');
var foreign = other.Object.getOwnPropertyDescriptor(proxy, 'x');
print(Object.getPrototypeOf(local) === Object.prototype, Object.getPrototypeOf(foreign) === other.Object.prototype);
var bad = new Proxy({}, {getOwnPropertyDescriptor: function() { return 1; }});
try { other.Object.getOwnPropertyDescriptor(bad, 'x'); } catch (error) {
  print(error instanceof other.TypeError, error instanceof TypeError);
}
"#,
        &["true true", "true false"],
    );
}
