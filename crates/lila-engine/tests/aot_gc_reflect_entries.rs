use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_reflect(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let source = format!(
        "function assert(value, message) {{ if (!value) throw new Error(message); }}\n{source}"
    );
    let observed = Engine::new(RealmBuilder::new().build())
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
        .expect("Reflect control executes through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
}

#[test]
fn explicit_undefined_receivers_and_symbol_keys_retain_whole_identity() {
    assert_reflect(
        r#"
var setterThis, setterValue;
var target = Object.create(null);
Object.defineProperty(target, 'answer', {
    get: function() { 'use strict'; return this; },
    set: function(value) { 'use strict'; setterThis = this; setterValue = value; },
    enumerable: true, configurable: true
});
assert(Reflect.get(target, 'answer') === target, 'omitted receiver uses target');
assert(Reflect.get(target, 'answer', undefined) === undefined, 'explicit receiver stays undefined');
assert(Reflect.set(target, 'answer', 7, undefined) === true, 'setter succeeds');
assert(setterThis === undefined && setterValue === 7, 'strict setter receiver and value');
var symbol = Symbol('retained'), token = {}, hooks = 0;
var key = { [Symbol.toPrimitive]: function(hint) { hooks++; assert(hint === 'string', 'key hint'); return symbol; } };
assert(Reflect.defineProperty(target, key, {value: token, configurable: true}), 'Symbol definition');
assert(hooks === 1 && Reflect.get(target, symbol) === token, 'single conversion and reference identity');
var descriptor = Reflect.getOwnPropertyDescriptor(target, symbol);
assert(descriptor.value === token && descriptor.configurable && !descriptor.writable, 'complete descriptor');
var keys = Reflect.ownKeys(target);
assert(keys.length === 2 && keys[0] === 'answer' && keys[1] === symbol, 'own key order and Symbol identity');
var data = {value: 3};
assert(Reflect.set(data, 'value', 9, undefined) === false && data.value === 3, 'primitive data receiver rejected');
"#,
    );
}

#[test]
fn target_and_new_target_validation_precede_observable_argument_or_key_reads() {
    assert_reflect(
        r#"
var reads = 0, sentinel = {};
var list = {get length() { reads++; throw sentinel; }};
function caught(call, Type) {
    try { call(); throw new Error('missing exception'); }
    catch (error) { assert(error instanceof Type, 'correct error class'); }
}
caught(function() { Reflect.apply(null, undefined, list); }, TypeError);
caught(function() { Reflect.construct(function Target() {}, list, undefined); }, TypeError);
assert(reads === 0, 'invalid callable/newTarget before list');
try { Reflect.apply(function() {}, undefined, list); throw new Error('missing length throw'); }
catch (error) { assert(error === sentinel && reads === 1, 'valid call observes original length throw'); }
var key = {[Symbol.toPrimitive]: function() { reads++; throw sentinel; }};
caught(function() { Reflect.get(null, key); }, TypeError);
caught(function() { Reflect.defineProperty(null, key, {}); }, TypeError);
assert(reads === 1, 'object rejection before key conversion');
try { Reflect.deleteProperty({}, key); throw new Error('missing key throw'); }
catch (error) { assert(error === sentinel && reads === 2, 'valid target retains key throw'); }
var order = [], attributes = { get value() { order.push('value'); return sentinel; } };
key = {[Symbol.toPrimitive]: function() { order.push('key'); return 'item'; }};
var target = {};
assert(Reflect.defineProperty(target, key, attributes), 'definition succeeds');
assert(order.join(',') === 'key,value' && target.item === sentinel, 'key before descriptor and whole value');
"#,
    );
}

#[test]
fn private_descriptor_recursion_preserves_the_executing_native_realm() {
    assert_reflect(
        r#"
var other = __lilaCreateRealm().global;
var methods = [Reflect.getOwnPropertyDescriptor, other.Reflect.getOwnPropertyDescriptor];
var prototypes = [Object.prototype, other.Object.prototype];
var types = [TypeError, other.TypeError];
for (var index = 0; index < methods.length; index++) {
    var token = {}, object = {value: token};
    var descriptor = methods[index](object, 'value');
    assert(Object.getPrototypeOf(descriptor) === prototypes[index], 'fresh descriptor method Realm');
    assert(descriptor.value === token, 'retained descriptor value');
    var proxy = new Proxy(object, {getOwnPropertyDescriptor: function() { return 1; }});
    try { methods[index](proxy, 'value'); throw new Error('missing Proxy error'); }
    catch (error) { assert(error instanceof types[index], 'internal Proxy error method Realm'); }
}
var foreignKeys = other.Reflect.ownKeys({a: 1});
assert(Object.getPrototypeOf(foreignKeys) === other.Array.prototype && foreignKeys[0] === 'a', 'ownKeys Array method Realm');
"#,
    );
}
