use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_reflect(source: &str) {
    assert_reflect_mode(source, false);
}

fn assert_reflect_mode(source: &str, strict: bool) {
    scriptw_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let source = format!(
        "{}function assert(value, message) {{ if (!value) throw new Error(message); }}\n{source}",
        if strict { "\"use strict\";\n" } else { "" }
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
fn ordinary_data_reads_remain_live_after_key_effects_and_accessor_replacement() {
    for strict in [false, true] {
        assert_reflect_mode(
            r#"
var token = {}, symbol = Symbol('same'), otherSymbol = Symbol('same');
var proto = { inherited: token }, object = Object.create(proto);
object.value = token; object[symbol] = -23n; object[otherSymbol] = 3;
object['x\uD800'] = symbol;
assert(Reflect.get(object, 'value', undefined) === token, 'data ignores receiver');
assert(object[symbol] === -23n && object[otherSymbol] === 3, 'whole Symbol and BigInt values');
assert(object['x\uD800'] === symbol && object['x\uFFFD'] === undefined, 'exact UTF16 key');
var conversions = 0, key = {[Symbol.toPrimitive]: function(hint) {
    conversions++; assert(hint === 'string', 'key hint');
    Object.defineProperty(object, 'value', {value: symbol, configurable: true});
    gc(); return 'value';
}};
assert(Reflect.get(object, key, token) === symbol && conversions === 1, 'fresh data after coercion');
var receiver = {}, reads = 0;
Object.defineProperty(object, 'value', {get: function() {
    'use strict'; reads++; assert(this === receiver, 'accessor receiver');
    Object.defineProperty(object, 'value', {value: token, configurable: true});
    gc(); return symbol;
}, configurable: true});
assert(Reflect.get(object, 'value', receiver) === symbol && reads === 1, 'accessor replacement');
assert(object.value === token && reads === 1, 'fresh data after getter');
assert(delete object.value && Reflect.get(object, 'value', receiver) === undefined, 'deleted own data');
Object.defineProperty(proto, 'value', {get: function() { 'use strict'; return this; }, configurable: true});
assert(Reflect.get(object, 'value', receiver) === receiver, 'inherited getter retains receiver');
var sentinel = {};
Object.defineProperty(proto, 'value', {get: function() { throw sentinel; }, configurable: true});
try { object.value; throw new Error('missing inherited throw'); } catch (error) { assert(error === sentinel, 'original abrupt result'); }
object[symbol] = token; Object.freeze(object);
assert(object[symbol] === token && object.inherited === token, 'frozen own and inherited reads');
var traps = 0, proxy = new Proxy(object, {get: function(target, key, receiver) {
    traps++; return Reflect.get(target, key, receiver);
}});
assert(proxy[symbol] === token && traps === 1, 'Proxy trap remains observable');
var boxed = Object('ab'); boxed.extra = token;
assert(boxed[0] === 'a' && boxed.length === 2 && boxed.extra === token, 'String exotic keeps virtual properties');
var typed = new Uint8Array([5]);
assert(typed[0] === 5 && typed['-0'] === undefined && typed[1] === undefined, 'integer-indexed exotic remains distinct');
true;
"#,
            strict,
        );
    }
}

#[test]
fn primitive_string_length_reads_preserve_utf16_keys_receivers_and_abrupt_effects() {
    for strict in [false, true] {
        assert_reflect_mode(
            r#"
function read(value, key) { return value[key]; }
var text = 'x\uD83D\uDE00\uD800', calls = 0, sentinel = {};
Object.defineProperty(Object.prototype, 'length', {get: function() { calls++; throw sentinel; }, configurable: true});
assert(read(text, 'length') === 4 && calls === 0, 'own UTF16 length precedes prototype');
assert(read('', 'length') === 0 && read('\uD800', 'length') === 1, 'empty and lone surrogate lengths');
var keyReads = 0, key = {[Symbol.toPrimitive]: function(hint) {
    keyReads++; assert(hint === 'string', 'key hint'); gc(); return 'length';
}};
assert(read(text, key) === 4 && keyReads === 1, 'one coercion before own read');
key = {[Symbol.toPrimitive]: function() { keyReads++; throw sentinel; }};
try { read(text, key); throw new Error('missing key throw'); } catch (error) { assert(error === sentinel && keyReads === 2, 'coercion abrupt cutoff'); }
var symbol = Symbol('length'), receiver;
Object.defineProperty(String.prototype, symbol, {get: function() { 'use strict'; receiver = this; return sentinel; }, configurable: true});
assert(read(text, symbol) === sentinel && receiver === text, 'inherited strict receiver stays primitive');
Object.defineProperty(String.prototype, 'length\uD800', {get: function() { 'use strict'; return this; }, configurable: true});
assert(read(text, 'length\uD800') === text, 'exact key equality');
assert(read(text, 0) === 'x' && read(text, 3) === '\uD800' && read(text, 4) === undefined, 'indexed fallback keeps UTF16');
var boxed = Object(text), trapReads = 0;
var proxy = new Proxy(boxed, {get: function(target, key, receiver) { trapReads++; return Reflect.get(target, key, receiver); }});
assert(Reflect.get(boxed, 'length', sentinel) === 4, 'boxed String data read');
assert(proxy.length === 4 && trapReads === 1, 'boxed Proxy trap is observable');
for (var value of [null, undefined]) {
    try { read(value, 'length'); throw new Error('missing nullish throw'); }
    catch (error) { assert(error instanceof TypeError, 'nullish target still rejects'); }
}
delete Object.prototype.length;
true;
"#,
            strict,
        );
    }
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
var other = __scriptwCreateRealm().global;
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

#[test]
fn array_argument_lists_preserve_live_index_reads_and_initial_length() {
    for strict in [false, true] {
        assert_reflect_mode(
            r#"
function collect(...values) { return values; }
var token = {}, symbol = Symbol('argument'), big = 17n;
var source = [token, symbol, big];
var copied = Reflect.apply(collect, undefined, source);
assert(copied.length === 3 && copied[0] === token && copied[1] === symbol && copied[2] === big, 'whole values');
source[0] = 99;
assert(copied[0] === token, 'retained argument value does not change with descriptor');
assert(collect.apply(null, source)[0] === 99, 'later application observes mutation');
function Target(...values) { this.values = values; }
var constructed = Reflect.construct(Target, [token, symbol, big]);
assert(constructed.values[0] === token && constructed.values[1] === symbol && constructed.values[2] === big, 'construct shares whole List');
var trace = [], list = [0, 1, 2], prototype = Object.create(Array.prototype);
Object.defineProperty(prototype, '1', {get: function() { trace.push('hole'); assert(this === list, 'inherited receiver'); return symbol; }});
Object.setPrototypeOf(list, prototype);
Object.defineProperty(list, '0', {get: function() { trace.push('first'); delete list[1]; list[2] = big; list.push(4); return token; }});
copied = Reflect.apply(collect, null, list);
assert(copied.length === 3 && copied[0] === token && copied[1] === symbol && copied[2] === big, 'initial length and live later indices');
assert(trace.join(',') === 'first,hole', 'index order and exactly one getter');
var called = false, sentinel = {}, abrupt = [token, 0, big];
Object.defineProperty(abrupt, '1', {get: function() { throw sentinel; }});
Object.defineProperty(abrupt, '2', {get: function() { called = true; return big; }});
try { Reflect.apply(function() { called = true; }, null, abrupt); throw new Error('missing throw'); }
catch (error) { assert(error === sentinel && !called, 'no later Get or call after original throw'); }
trace = [];
var proxy = new Proxy([token, symbol], {get: function(target, key, receiver) { trace.push(key); return Reflect.get(target, key, receiver); }});
copied = Reflect.apply(collect, null, proxy);
assert(trace.join(',') === 'length,0,1' && copied[0] === token && copied[1] === symbol, 'Proxy retained generic ordered Gets');
var shrunk = [token, symbol, big];
Object.defineProperty(shrunk, '0', {get: function() { shrunk.length = 1; return token; }});
copied = collect.apply(null, shrunk);
assert(copied.length === 3 && copied[0] === token && copied[1] === undefined && copied[2] === undefined, 'length fixed before shrinking getter');
assert(Reflect.apply(String.prototype.charCodeAt, '\ud800', [0]) === 55296, 'UTF16 code unit preserved');
"#,
            strict,
        );
    }
}
