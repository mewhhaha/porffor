use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_object(source: &str) {
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
        .expect("Object control executes through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
}

#[test]
fn property_index_keeps_collision_chains_and_string_symbol_equality() {
    assert_object(
        r#"
var object = Object.create(null);
// These different UTF-16 strings share the low ten bits of the key hash.
var keys = ['prop-98', 'prop-105', 'prop-120', 'prop-142'];
for (var i = 0; i < keys.length; i++) object[keys[i]] = i;
assert(delete object[keys[0]] && object[keys[3]] === 3, 'lookup crosses a deleted collision head');
object[keys[2]] = 42;
assert(Object.keys(object).join(',') === keys.slice(1).join(','), 'update preserves ordered slot');
for (var i = 0; i < 130; i++) object['filler' + i] = i;
assert(object[keys[1]] === 1 && object[keys[2]] === 42 && object[keys[3]] === 3, 'rehash preserves colliding entries');
assert(delete object[keys[1]] && object[keys[3]] === 3, 'deleted collision middle stays traversable after growth');
object[keys[0]] = 10; object[keys[1]] = 11;
var names = Object.getOwnPropertyNames(object);
assert(names.length === 134 && names.slice(-2).join(',') === keys[0] + ',' + keys[1], 'tombstone reuse does not change re-addition order');
assert(object[keys[0]] === 10 && object[keys[1]] === 11 && !Object.hasOwn(object, 'missing'), 'exact key match after reuse');
var text = 'prefix\uD83D\uDE00\uD800';
object[text] = 23;
assert(object['prefix' + String.fromCharCode(0xD83D, 0xDE00, 0xD800)] === 23, 'equal UTF16 content finds one entry');
object['prefix\uD83D\uDE00\uFFFD'] = 24;
assert(object[text] === 23 && object['prefix\uD83D\uDE00\uFFFD'] === 24, 'lone surrogate remains distinct from replacement character');
var first = Symbol('same'), second = Symbol('same'), third = Symbol('same'), token = {};
object[first] = token; object[second] = 2;
var map = new Map([[first, 1], [second, 2], [third, 3]]);
object[third] = 3;
gc();
assert(object[first] === token && object[second] === 2 && object[third] === 3, 'Symbol identity survives property-first and collection-first hashing');
assert(map.get(first) === 1 && map.get(second) === 2 && map.get(third) === 3, 'property hashing preserves collection identity');
assert(delete object[first] && !Object.hasOwn(object, first) && object[second] === 2, 'Symbol deletion remains separate');
object[first] = token;
var symbols = Object.getOwnPropertySymbols(object);
assert(symbols.length === 3 && symbols[0] === second && symbols[1] === third && symbols[2] === first, 'Symbol re-addition order');
Object.defineProperty(object, keys[3], {value: 30, writable: false, configurable: false});
assert(!Reflect.set(object, keys[3], 40) && !Reflect.deleteProperty(object, keys[3]) && object[keys[3]] === 30, 'indexed descriptor retains attributes');
"#,
    );
}

#[test]
fn property_growth_preserves_order_descriptors_and_reentrant_gc_roots() {
    assert_object(
        r#"
var object = Object.create(null), token = {}, first = Symbol('same'), second = Symbol('same');
object.before = token;
object[first] = token;
for (var i = 0; i < 129; i++) object['key' + i] = i;
object[second] = 2;
object['2'] = 'two'; object['1'] = 'one'; object['01'] = 'named';
object['\uD83D\uDE00\uD800'] = token;
var names = Object.getOwnPropertyNames(object);
assert(names.slice(0, 4).join(',') === '1,2,before,key0', 'numeric keys precede insertion-ordered names');
assert(object[first] === token && object[second] === 2 && object['\uD83D\uDE00\uD800'] === token, 'distinct symbols and exact UTF16 keys');
for (var i = 0; i < 129; i++) assert(object['key' + i] === i, 'old entries survive each growth');
var snapshot = Object.getOwnPropertyDescriptor(object, 'key64');
object.key64 = token;
assert(snapshot.value === 64 && object.key64 === token, 'descriptor snapshot remains separate from stored descriptor');
assert(delete object.before && delete object.key0 && delete object.key128 && delete object[first], 'delete ordered entries');
object.before = token; object.key0 = 1000; object[first] = token;
names = Object.getOwnPropertyNames(object);
assert(names.slice(-2).join(',') === 'before,key0' && names.indexOf('key128') === -1, 're-added strings append after surviving names');
var symbols = Object.getOwnPropertySymbols(object);
assert(symbols.length === 2 && symbols[0] === second && symbols[1] === first, 're-added symbol appends after surviving symbols');
var reads = 0;
Object.defineProperty(object, 'key63', {get: function() {
    reads++;
    for (var j = 0; j < 140; j++) object['grown' + j] = j;
    gc();
    return token;
}, configurable: true});
assert(object.key63 === token && reads === 1, 'one getter survives table replacement and collection');
assert(Object.getOwnPropertyDescriptor(object, 'key63').get !== undefined && reads === 1, 'descriptor reads do not invoke getter');
Object.defineProperty(object, 'fixed', {value: undefined});
assert(Object.hasOwn(object, 'fixed') && !Reflect.set(object, 'fixed', 9), 'all-false descriptor remains present');
Object.preventExtensions(object);
assert(!Reflect.defineProperty(object, 'absent', {value: 1}) && !Object.hasOwn(object, 'absent'), 'spare capacity cannot bypass extensibility');
assert(object.before === token && object.key64 === token, 'rooted values remain live');
"#,
    );
}

#[test]
fn define_properties_converts_all_before_applying_and_retains_partial_presence() {
    assert_object(
        r#"
var token = {}, target = {existing: 1}, reads = 0;
var bag = {
    get first() { reads++; return {value: token}; },
    get last() { reads++; throw token; }
};
try { Object.defineProperties(target, bag); throw new Error('missing conversion error'); }
catch (error) { assert(error === token, 'whole original throw'); }
assert(reads === 2 && !Object.hasOwn(target, 'first') && target.existing === 1, 'no definition before all conversions');

var first = Object.create(null);
first.value = token;
bag = {
    get first() { return first; },
    get last() { first.value = 'changed'; return {value: 31}; }
};
Object.defineProperties(target, bag);
assert(target.first === token && target.last === 31, 'retained converted value, no replay');
var setter = function(value) {};
Object.defineProperty(target, 'accessor', {get: function() {return 1;}, set: setter, enumerable: true, configurable: true});
var replacement = function() {return token;};
Object.defineProperties(target, {existing: {value: 2}, accessor: {get: replacement}});
var data = Object.getOwnPropertyDescriptor(target, 'existing');
var accessor = Object.getOwnPropertyDescriptor(target, 'accessor');
assert(data.value === 2 && data.writable && data.enumerable && data.configurable, 'absent data flags retain current attributes');
assert(accessor.get === replacement && accessor.set === setter && accessor.enumerable && accessor.configurable, 'absent accessor fields retain current attributes');
"#,
    );
}

#[test]
fn proxy_descriptor_target_observations_precede_trap_result_conversion() {
    assert_object(
        r#"
var order = [], token = {};
var target = new Proxy({}, {
    getOwnPropertyDescriptor: function() {order.push('target:own'); return undefined;},
    isExtensible: function() {order.push('target:extensible'); return true;}
});
var attributes = {
    get value() {order.push('descriptor:value'); return token;},
    configurable: true
};
var proxy = new Proxy(target, {
    getOwnPropertyDescriptor: function() {order.push('trap'); return attributes;}
});
var result = Object.getOwnPropertyDescriptor(proxy, 'answer');
assert(order.join(',') === 'trap,target:own,target:extensible,descriptor:value', 'complete step order');
assert(result !== attributes && result.value === token && Object.getPrototypeOf(result) === Object.prototype, 'fresh complete result');
order = [];
proxy = new Proxy(target, {
    getOwnPropertyDescriptor: function() {order.push('trap'); return undefined;}
});
assert(Object.getOwnPropertyDescriptor(proxy, 'absent') === undefined, 'absent result');
assert(order.join(',') === 'trap,target:own', 'absent target avoids extensibility');
"#,
    );
}

#[test]
fn object_to_locale_string_retains_primitive_receiver_and_borrowed_lookup_realm() {
    assert_object(
        r#"
var other = __lilaCreateRealm().global;
var original = Number.prototype.toString;
var foreignOriginal = other.Number.prototype.toString;
var mainSeen, foreignSeen;
try {
    Number.prototype.toString = function() {'use strict'; mainSeen = this; return 'main';};
    other.Number.prototype.toString = function() {'use strict'; foreignSeen = this; return 'foreign';};
    assert(Object.prototype.toLocaleString.call(7) === 'main', 'main GetV lookup');
    assert(other.Object.prototype.toLocaleString.call(7) === 'foreign', 'foreign GetV lookup');
    assert(mainSeen === 7 && foreignSeen === 7, 'Call receiver remains primitive');
} finally {
    Number.prototype.toString = original;
    other.Number.prototype.toString = foreignOriginal;
}

var token = {}, receiver = {};
Object.defineProperty(receiver, 'toString', {get: function() {throw token;}});
try { Object.prototype.toLocaleString.call(receiver); throw new Error('missing getter error'); }
catch (error) {assert(error === token, 'GetV original throw');}
"#,
    );
}

#[test]
fn object_construction_selects_new_target_and_keeps_null_prototype_definitions() {
    assert_object(
        r#"
var other = __lilaCreateRealm().global;
var token = {};
assert(Object(token) === token && new Object(token) === token, 'ordinary Object retains object argument');
assert(Object.getPrototypeOf(other.Object()) === other.Object.prototype, 'borrowed constructor defining Realm');
function Different() {}
var constructed = Reflect.construct(Object, [token], Different);
assert(constructed !== token && Object.getPrototypeOf(constructed) === Different.prototype, 'different newTarget ignores value');
var constructedForeign = Reflect.construct(Object, [token], other.Object);
assert(constructedForeign !== token && Object.getPrototypeOf(constructedForeign) === other.Object.prototype, 'foreign newTarget prototype');
var symbol = Symbol('field'), value = {};
var bag = Object.create(null);
bag.answer = {value: value, enumerable: true};
bag[symbol] = {value: token, configurable: true};
var object = Object.create(null, bag);
assert(Object.getPrototypeOf(object) === null && object.answer === value && object[symbol] === token, 'null prototype applies all definitions');
assert(Object.getOwnPropertyNames(object).join(',') === 'answer', 'names retain String keys');
assert(Object.getOwnPropertySymbols(object)[0] === symbol, 'symbols retain identity');
assert(Object.is(NaN, NaN) && !Object.is(0, -0) && Object.is(value, value), 'whole SameValue');

var separate = Object.create(null);
separate.answer = 9;
assert(object.answer === value && separate.answer === 9, 'allocated headers own independent property tables');
var parent = {};
assert(Reflect.setPrototypeOf(separate, parent) && Object.getPrototypeOf(separate) === parent, 'ordinary null header remains mutable');
assert(!Reflect.setPrototypeOf(Object.prototype, parent) && Object.getPrototypeOf(Object.prototype) === null, 'Object.prototype retains immutable null prototype');

var boxing = [
  [7, other.Number.prototype, other.Number.prototype.valueOf],
  [true, other.Boolean.prototype, other.Boolean.prototype.valueOf],
  ['\uD83D\uDE00\uD800', other.String.prototype, other.String.prototype.valueOf],
  [symbol, other.Symbol.prototype, other.Symbol.prototype.valueOf],
  [17n, other.BigInt.prototype, other.BigInt.prototype.valueOf]
];
for (var entry of boxing) {
  var boxed = other.Object(entry[0]);
  assert(Object.getPrototypeOf(boxed) === entry[1], 'each primitive uses the selected callable Realm prototype');
  assert(entry[2].call(boxed) === entry[0], 'boxing retains the whole original primitive');
  if (typeof entry[0] === 'string') {
    var length = Object.getOwnPropertyDescriptor(boxed, 'length');
    assert(length.value === 3 && !length.writable && !length.enumerable && !length.configurable, 'String boxing publishes exact UTF16 length attributes');
  }
}
for (var nullish of [null, undefined]) {
  try { other.Object.keys(nullish); throw new Error('missing foreign ToObject error'); }
  catch (error) {assert(error instanceof other.TypeError && !(error instanceof TypeError), 'ToObject retains explicit foreign error Realm');}
}
"#,
    );
}

#[test]
fn object_to_string_preserves_brands_tag_getters_and_utf16() {
    assert_object(
        r#"
var toString = Object.prototype.toString;
assert(toString.call(undefined) === '[object Undefined]' && toString.call(null) === '[object Null]', 'nullish before boxing');
assert(toString.call([]) === '[object Array]' && toString.call(function(){}) === '[object Function]', 'array and callable brands');
assert(toString.call(new Date(0)) === '[object Date]' && toString.call(Date.prototype) === '[object Object]', 'Date value slot');
assert(toString.call(new Number(2)) === '[object Number]' && toString.call(new String('x')) === '[object String]', 'primitive slots');
assert(toString.call(new Error()) === '[object Error]' && toString.call(/x/) === '[object RegExp]', 'Error and RegExp slots');
var text = 'custom\uD83D\uDE00\uD800', seen = 0, token = {};
var value = Object.create(null);
Object.defineProperty(value, Symbol.toStringTag, {get: function() {seen++; return text;}});
assert(toString.call(value) === '[object ' + text + ']' && seen === 1, 'one tag Get and exact UTF16');
Object.defineProperty(token, Symbol.toStringTag, {get: function() {throw token;}});
try { toString.call(token); throw new Error('missing tag error'); }
catch (error) {assert(error === token, 'whole tag getter Throw');}
var pair = Proxy.revocable([], {}), observed = 0;
pair.revoke();
try {toString.call(pair.proxy); throw new Error('missing revoked error');}
catch (error) {assert(error instanceof TypeError, 'IsArray rejects revoked Proxy');}
"#,
    );
}

#[test]
fn object_integrity_and_prototype_operations_keep_observation_order() {
    assert_object(
        r#"
var order = [], target = {x: 1};
function proxy() {
    return new Proxy(target, {
        preventExtensions: function(t) {order.push('prevent'); return Reflect.preventExtensions(t);},
        ownKeys: function(t) {order.push('keys'); return Reflect.ownKeys(t);},
        getOwnPropertyDescriptor: function(t, k) {order.push('own:' + k); return Reflect.getOwnPropertyDescriptor(t, k);},
        defineProperty: function(t, k, d) {order.push('define:' + k); return Reflect.defineProperty(t, k, d);}
    });
}
var sealed = proxy();
assert(Object.seal(sealed) === sealed, 'seal returns original target');
assert(order.join(',') === 'prevent,keys,define:x', 'seal does not acquire descriptors');
order = []; target = {x: 1};
var frozen = proxy();
assert(Object.freeze(frozen) === frozen, 'freeze returns original target');
assert(order.join(',') === 'prevent,keys,own:x,define:x', 'freeze owns ordered descriptor acquisition');
assert(Object.isFrozen(target) && !Object.getOwnPropertyDescriptor(target, 'x').writable, 'frozen data attributes');
assert(Object.seal(3) === 3 && Object.freeze(null) === null && !Object.isExtensible('x'), 'primitive integrity');
var primitiveSetter = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').set;
assert(primitiveSetter.call(3, {}) === undefined && primitiveSetter.call({}, 3) === undefined, 'legacy setter ignored primitive forms');
try {Object.setPrototypeOf(3, 3); throw new Error('missing invalid prototype');}
catch (error) {assert(error instanceof TypeError, 'function validates prototype on primitive target');}
var token = {}, hostile = new Proxy({}, {getPrototypeOf: function() {throw token;}});
try {Object.prototype.isPrototypeOf.call({}, hostile); throw new Error('missing prototype error');}
catch (error) {assert(error === token, 'whole getPrototypeOf Throw');}
"#,
    );
}

#[test]
fn shared_set_and_data_definition_preserve_receivers_flags_and_whole_throws() {
    assert_object(
        r#"
var token = {}, symbol = Symbol('field');
var literal = {answer: 1, [symbol]: token, answer: token};
var data = Object.getOwnPropertyDescriptor(literal, 'answer');
var symbolic = Object.getOwnPropertyDescriptor(literal, symbol);
assert(data.value === token && data.writable && data.enumerable && data.configurable, 'literal definition keeps all flags');
assert(symbolic.value === token && symbolic.writable && symbolic.enumerable && symbolic.configurable, 'whole Symbol key and value');
var elements = [token, , 3];
data = Object.getOwnPropertyDescriptor(elements, '0');
assert(data.value === token && data.writable && data.enumerable && data.configurable && !Object.hasOwn(elements, '1'), 'array literal definitions retain holes and flags');

var base = {answer: 1}, receiver = {}, order = [];
Object.defineProperty(receiver, 'answer', {value: 2, writable: true});
var observed = new Proxy(receiver, {
    getOwnPropertyDescriptor: function(t, k) {order.push('own:' + k); return Reflect.getOwnPropertyDescriptor(t, k);},
    defineProperty: function(t, k, d) {
        order.push('define:' + k);
        assert(d.value === token && !Object.hasOwn(d, 'writable') && !Object.hasOwn(d, 'enumerable') && !Object.hasOwn(d, 'configurable'), 'existing receiver gets only a value');
        return Reflect.defineProperty(t, k, d);
    }
});
assert(Reflect.set(base, 'answer', token, observed), 'receiver definition succeeds');
data = Object.getOwnPropertyDescriptor(receiver, 'answer');
assert(order.join(',') === 'own:answer,define:answer' && base.answer === 1 && data.value === token && data.writable && !data.enumerable && !data.configurable, 'receiver identity, order and absent flags');
var child = Object.create(base);
child.answer = token;
assert(child.answer === token && Object.hasOwn(child, 'answer') && base.answer === 1, 'prototype recursion retains receiver');
var transparent = new Proxy(new Proxy(child, {}), {});
transparent.answer = 4;
assert(child.answer === 4, 'nested Proxy fallback reaches shared Set');

var setterSeen;
Object.defineProperty(base, 'accessor', {set: function(value) {
    'use strict'; setterSeen = this; this.side = value; throw token;
}});
try { Reflect.set(base, 'accessor', token, child); throw new Error('missing setter throw'); }
catch (error) {assert(error === token && setterSeen === child && child.side === token, 'reentrant setter preserves receiver and original Throw');}
Object.defineProperty(base, 'fixed', {value: token});
assert((base.fixed = 9) === 9 && base.fixed === token && !Reflect.set(base, 'fixed', 9), 'sloppy and Reflect rejection retain their distinct normal results');
try { (function() {'use strict'; base.fixed = 9;})(); throw new Error('missing strict rejection'); }
catch (error) {assert(error instanceof TypeError, 'strict false result becomes TypeError');}
var refusing = new Proxy({}, {set: function() {return false;}});
assert((refusing.answer = token) === token && !Object.hasOwn(refusing, 'answer'), 'sloppy Proxy false keeps RHS');
try { (function() {'use strict'; refusing.answer = token;})(); throw new Error('missing strict Proxy rejection'); }
catch (error) {assert(error instanceof TypeError, 'strict Proxy false becomes TypeError');}

var indexed = new Uint8Array(1), conversions = 0;
indexed[0] = {valueOf: function() {conversions++; return 7;}};
assert(indexed[0] === 7 && conversions === 1, 'integer-indexed Set keeps conversion count');
var other = __lilaCreateRealm().global, foreignError = new other.TypeError('identity');
var hostile = new Proxy({}, {set: function() {throw foreignError;}});
try { hostile.answer = token; throw new Error('missing foreign throw'); }
catch (error) {assert(error === foreignError && error instanceof other.TypeError, 'shared helpers retain foreign thrown identity');}
"#,
    );
}

#[test]
fn own_keys_sort_sparse_unsigned_indices_and_preserve_string_symbol_order() {
    assert_object(
        r#"
var object = Object.create(null), first = Symbol('first'), second = Symbol('second');
object.z = 1; object[first] = 1;
for (var i = 256; i >= 0; i--) object[String(i)] = i;
object['4294967294'] = 'max index'; object['2147483648'] = 'high index';
object['4294967295'] = 'non-index'; object['01'] = 1; object['-0'] = 1;
object['\uD800'] = 1; object[second] = 2;
delete object['7']; object['7'] = 70;
delete object.z; object.z = 2;
var keys = Reflect.ownKeys(object);
assert(keys.length === 266, 'all and only own keys');
for (var i = 0; i < 257; i++) assert(keys[i] === String(i), 'ascending indices despite reverse insertion and re-addition');
assert(keys[257] === '2147483648' && keys[258] === '4294967294', 'unsigned sparse high indices');
assert(keys.slice(259, 264).join('|') === '4294967295|01|-0|\uD800|z', 'non-index strings retain insertion order');
assert(keys[264] === first && keys[265] === second, 'symbols follow strings in insertion order');
var reads = 0;
Object.defineProperty(object, '5', {get: function() { reads++; return 5; }});
assert(Reflect.ownKeys(object)[5] === '5' && reads === 0, 'key enumeration never invokes accessors');
assert(Reflect.ownKeys(Object.create(null)).length === 0, 'empty list');
assert(Reflect.ownKeys({'3': true}).join(',') === '3', 'single numeric key');
"#,
    );
}
