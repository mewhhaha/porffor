use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_trace(source: &str, marker: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let observed = Engine::new(RealmBuilder::new().build())
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
        .expect("property flow controls compile and execute through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{observed:?}"
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine(marker.into())],
        "{observed:?}"
    );
}

#[test]
fn optional_primitive_methods_and_factories_keep_their_original_receivers() {
    assert_trace(
        r#"
const symbol = Symbol('receiver');
String.prototype.q = function () { 'use strict'; return this === 'z'; };
Number.prototype.q = function () { 'use strict'; return this === 3; };
Boolean.prototype.q = function () { 'use strict'; return this === true; };
BigInt.prototype.q = function () { 'use strict'; return this === 3n; };
Symbol.prototype.q = function () { 'use strict'; return this === symbol; };
if (!'z'?.q() || !(3)?.q()) throw 'literal receiver was boxed';
function makeString() { return 'z'; }
function makeNumber() { return 3; }
function makeBoolean() { return true; }
function makeBigInt() { return 3n; }
function makeSymbol() { return symbol; }
if (!makeString?.().q() || !makeNumber?.().q() || !makeBoolean?.().q() ||
    !makeBigInt?.().q() || !makeSymbol?.().q()) throw 'factory receiver was lost';
let effects = 0;
if (null?.q(effects++) !== undefined || effects !== 0) throw 'optional cutoff';

const array = [1];
array.join = function () { if (this !== array) throw 'join receiver'; return 17; };
array.toString = function () { if (this !== array) throw 'toString receiver'; return 17; };
array.reverse = function () { if (this !== array) throw 'reverse receiver'; return 17; };
if (array.join() !== 17 || array.toString() !== 17 || array.reverse() !== 17) throw 'own array method';
let compound = '';
compound += String.fromCodePoint.apply(null, [65, 66]);
if (compound !== 'AB') throw 'compound string value';
print('property-receivers');
"#,
        "property-receivers",
    );
}

#[test]
fn property_effects_preserve_live_globals_function_identity_and_constructor_arguments() {
    assert_trace(
        r#"
var value = 1;
function read() { return this.value; }
if (read(value = 's') + 1 !== 's1') throw 'default this before argument effects';
function changed() {}
function preserved() {}
changed.value = 1;
preserved.value = 1;
changed.marker = 0;
if (preserved.value + 1 !== 2) throw 'distinct function property';
let objectLengthCalls = 0;
const objectLengthHolder = { value: 1 };
Object.defineProperty([], 'length', {
    __proto__: null,
    value: { valueOf() { objectLengthCalls++; objectLengthHolder.value = 'object'; return 0; } }
});
if (objectLengthCalls !== 2 || objectLengthHolder.value + 1 !== 'object1') throw 'Object ArraySetLength coercion';
let reflectLengthCalls = 0;
const reflectLengthHolder = { value: 1 };
if (!Reflect.defineProperty([], 'length', {
    __proto__: null,
    value: { valueOf() { reflectLengthCalls++; reflectLengthHolder.value = 'reflect'; return 0; } }
})) throw 'Reflect ArraySetLength status';
if (reflectLengthCalls !== 2 || reflectLengthHolder.value + 1 !== 'reflect1') throw 'Reflect ArraySetLength coercion';
function DescriptorTarget() {}
const uncoercedDescriptorValue = { valueOf() { throw 'ordinary descriptor value coerced'; } };
Object.defineProperty(DescriptorTarget, 'ordinary', { __proto__: null, value: uncoercedDescriptorValue });
if (DescriptorTarget.ordinary !== uncoercedDescriptorValue) throw 'ordinary descriptor value';
let setterCalls = 0;
Object.defineProperty(Function.prototype, 'observedWrite', {
    configurable: true,
    set: function () {
        if (this !== changed) throw 'setter receiver';
        setterCalls++;
        preserved.value = 'setter';
    }
});
changed.observedWrite = 7;
if (setterCalls !== 1 || preserved.value + 1 !== 'setter1') throw 'inherited setter effect';
function Constructor(value) { this.value = value; }
function construct(flag) { let Target = flag ? Constructor : Array; return new Target(7); }
if (construct(true).value !== 7 || construct(false).length !== 7) throw 'constructor argument';
function write(flag) {
    let left = { p: 0, leftOnly: 0 };
    let right = { p: 0, rightOnly: 0 };
    (flag ? left : right).p = 's';
    return (1).toString();
}
if (write(true) !== '1' || write(false) !== '1') throw 'unrelated Number method';

const parent = { inherited: 19 };
function create(flag) { return Object.create(flag ? null : parent); }
if (Object.getPrototypeOf(create(true)) !== null ||
    Object.getPrototypeOf(create(false)) !== parent || create(false).inherited !== 19) throw 'mixed prototype';
let descriptorReads = 0, assigned;
const descriptor = {
    __proto__: null,
    marker: 1,
    get set() {
        if (this !== descriptor || this.marker !== 1) throw 'descriptor getter receiver';
        descriptorReads++;
        return function (value) { assigned = value; };
    }
};
const subject = {};
Object.defineProperty(subject, 'value', descriptor);
subject.value = 23;
if (descriptorReads !== 1 || assigned !== 23) throw 'descriptor setter';

const originalSymbol = Symbol;
function freshSymbol(label) { return Symbol(label); }
const first = freshSymbol('first'), second = freshSymbol('second');
const keyed = { [first]: 11, [second]: 12 };
if (first === second || keyed[first] !== 11 || keyed[second] !== 12 ||
    Reflect.ownKeys(keyed).length !== 2) throw 'distinct Symbol keys';
const trigger = { get value() { globalThis.Symbol = function () { return 'replacement-key'; }; return 0; } };
trigger.value;
const replaced = freshSymbol('after');
const liveKey = { [replaced]: 29 };
if (replaced !== 'replacement-key' || liveKey['replacement-key'] !== 29) throw 'mutable Symbol global';
globalThis.Symbol = originalSymbol;
print('property-effects');
"#,
        "property-effects",
    );
}

#[test]
fn joined_collection_accessors_and_async_iterator_symbol_reads_remain_live() {
    assert_trace(
        r#"
function readSize(flag, map, set) { return (flag ? map : set).size ||= 1; }
const map = new Map(), set = new Set();
if (readSize(true, map, set) !== 1 || readSize(false, map, set) !== 1) throw 'empty size';
map.set('first', 1);
map.set('second', 2);
set.add('first');
set.add('second');
set.add('third');
if (readSize(true, map, set) !== 2 || readSize(false, map, set) !== 3) throw 'truthy size';
function strictSize(value) { 'use strict'; return value.size ||= 1; }
let rejected = false;
try { strictSize(new Map()); } catch (error) { rejected = error instanceof TypeError; }
if (!rejected) throw 'strict readonly size';
let iteratorCalls = 0;
String.prototype[Symbol.asyncIterator] = function () {
    'use strict';
    if (this !== 'source') throw 'async primitive receiver was boxed';
    iteratorCalls++;
    let count = 0;
    return { next() { return Promise.resolve({ value: 31, done: count++ !== 0 }); } };
};
String.prototype['Symbol.asyncIterator'] = function () { throw 'ordinary string property used'; };
async function collect() {
    let total = 0;
    for await (const value of 'source') total += value;
    return total;
}
collect().then(function (total) {
    if (total !== 31 || iteratorCalls !== 1) throw 'async iterator result';
    print('property-hooks');
});
"#,
        "property-hooks",
    );
}
