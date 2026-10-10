function check(condition, label) { if (!condition) throw label; }
function next(iterator, input, expected, done) {
  var result = iterator.next(input);
  check(result.value === expected && result.done === done, 'unexpected-iterator-step');
  return result;
}
var whole = { marker: 7 }; whole.self = whole;
whole[Symbol.toPrimitive] = function () { throw 'unexpected-value-coercion'; };

// PropertyName converts before target acquisition. A Member Reference key
// stays raw through the property read and every default suspension.
var events = [], selected = Symbol('selected'), destination = {};
var patternKey = { [Symbol.toPrimitive]: function () { events.push('key'); return selected; } };
var memberKey = { [Symbol.toPrimitive]: function () { events.push('target-key'); return selected; } };
var source = {};
Object.defineProperty(source, selected, { enumerable: true, get: function () {
  events.push('get'); return undefined;
} });
Object.defineProperty(source, 'remaining', { enumerable: true, get: function () {
  events.push('rest-get'); return whole;
} });
function* ordered() {
  return ({[yield 'key']: (yield 'base')[yield 'raw'] = (yield 'd1', yield 'd2'), ...destination.rest} = source);
}
var iterator = ordered();
next(iterator, undefined, 'key', false);
next(iterator, patternKey, 'base', false);
check(events.join(',') === 'key', 'converted-pattern-key-first');
gc(); next(iterator, destination, 'raw', false);
check(events.join(',') === 'key', 'no-get-before-reference');
next(iterator, memberKey, 'd1', false);
check(events.join(',') === 'key,get', 'reference-before-get-default');
gc(); next(iterator, whole, 'd2', false);
check(events.join(',') === 'key,get', 'raw-target-key-before-put');
next(iterator, whole, source, true);
check(events.join(',') === 'key,get,target-key,rest-get', 'one-key-get-and-rest-order');
check(destination[selected] === whole && destination.rest.remaining === whole, 'whole-put-rest');
check(!Object.prototype.hasOwnProperty.call(destination.rest, selected), 'symbol-excluded');
check(Object.getPrototypeOf(destination.rest) === Object.prototype, 'assignment-rest-prototype');

// Nonundefined values skip the complete default region, including all yields.
function* lazy(input) {
  let {[yield 'choose']: value = (yield 'wrong1', yield 'wrong2'), [yield 'later']: second} = input;
  return [value, second];
}
iterator = lazy({ a: whole, b: 8 });
next(iterator, undefined, 'choose', false);
next(iterator, 'a', 'later', false);
var result = iterator.next('b');
check(result.done && result.value[0] === whole && result.value[1] === 8, 'lazy-default-whole-value');

// GetV's lookup object is boxed once; the original primitive is its receiver.
var primitiveReceiver;
Object.defineProperty(String.prototype, 'patternReceiver', { configurable: true, get: function () {
  'use strict'; primitiveReceiver = this; return whole;
} });
function* primitive(input) { const {[yield 'primitive-key']: value, ...rest} = input; return [value, rest]; }
iterator = primitive('ab'); next(iterator, undefined, 'primitive-key', false);
result = iterator.next('patternReceiver');
check(result.done && primitiveReceiver === 'ab' && result.value[0] === whole, 'raw-primitive-getv-receiver');
check(result.value[1][0] === 'a' && result.value[1][1] === 'b', 'boxed-primitive-rest');
check(Object.getPrototypeOf(result.value[1]) === Object.prototype, 'primitive-rest-prototype');
delete String.prototype.patternReceiver;

// Nullish ToObject happens before even the first pattern key is evaluated.
function* nullish(input) { let {[yield 'wrong-key']: value} = input; }
for (var nullishValue of [null, undefined]) {
  try { nullish(nullishValue).next(); throw 'missing-nullish-typeerror'; }
  catch (error) { check(error instanceof TypeError, 'nullish-before-key'); }
}

// Every original lexical target starts in TDZ, including later nested targets.
function* tdz(input) { let {[yield 'tdz-key']: first = (yield 'tdz-default', later), inner: [later]} = input; }
iterator = tdz({ inner: [4] }); next(iterator, undefined, 'tdz-key', false);
next(iterator, 'first', 'tdz-default', false);
try { iterator.next(); throw 'missing-later-tdz'; }
catch (error) { check(error instanceof ReferenceError, 'later-nested-tdz-after-resume'); }

function* nested(input) {
  const {[yield 'outer-key']: {[yield 'inner-key']: value = yield 'nested-default'}, array: [item]} = input;
  return [value, item];
}
var closed = 0;
var iterable = { [Symbol.iterator]: function () { return {
  next: function () { return { value: 19, done: false }; },
  return: function () { closed++; return {}; }
}; } };
iterator = nested({ inner: {}, array: iterable });
next(iterator, undefined, 'outer-key', false); next(iterator, 'inner', 'inner-key', false);
gc(); next(iterator, 'value', 'nested-default', false);
result = iterator.next(whole);
check(result.done && result.value[0] === whole && result.value[1] === 19 && closed === 1, 'nested-object-eager-array-close');

// Original anonymous defaults keep their actual NamedEvaluation identity.
function* named(input) {
  const {[yield 'named-key']: namedFunction = function () {}, arrow = () => {},
    namedClass = class { static observed = this.name; }, explicit = function Explicit() {},
    explicitClass = class Named { static own = Named === this; }} = input;
  return [namedFunction, arrow, namedClass, explicit, explicitClass];
}
iterator = named({}); next(iterator, undefined, 'named-key', false);
result = iterator.next('function');
check(result.done && result.value[0].name === 'namedFunction' && result.value[1].name === 'arrow', 'default-function-arrow-names');
check(result.value[2].name === 'namedClass' && result.value[2].observed === 'namedClass', 'class-name-before-static-initialization');
check(result.value[3].name === 'Explicit' && result.value[4].name === 'Named' && result.value[4].own, 'explicit-definition-names-preserved');
class Parent {}
function* heritageDefault(input) { const {[yield 'class-key']: Selected = class extends (yield 'class-base') { static observed = this.name; }} = input; return Selected; }
iterator = heritageDefault({}); next(iterator, undefined, 'class-key', false);
next(iterator, 'selected', 'class-base', false); gc(); result = iterator.next(Parent);
check(result.done && result.value.name === 'Selected' && result.value.observed === 'Selected', 'suspended-class-default-name');
function* inferredClassTdz() { const {C = class { static seen = typeof C; }, x = yield 'unreached'} = {}; }
try { inferredClassTdz().next(); throw 'missing-class-default-tdz'; }
catch (error) { check(error instanceof ReferenceError, 'inferred-class-label-is-not-an-inner-binding'); }
function* inferredClassOuter() { var C = 17; var {C = class { static seen = C; }, x = yield 'outer-class'} = {}; return C; }
iterator = inferredClassOuter(); next(iterator, undefined, 'outer-class', false);
result = iterator.next(); check(result.done && result.value.seen === 17, 'anonymous-class-default-observes-original-outer-binding');

// Every var-pattern name is initialized before the body runs. A redeclaration
// preserves an earlier value while defaults and nested writes retain its cell.
function* hoistedVarObject(input) {
  var previous = 17;
  var read = () => [previous, selected, nested, rest];
  yield read;
  var {previous = previous + 1, x: selected = yield 'new-object-var', box: {nested}, ...rest} = input;
  return read;
}
iterator = hoistedVarObject({box: {nested: whole}, tail: whole});
var readObjectVars = iterator.next().value;
var beforeObjectVars = readObjectVars();
check(beforeObjectVars[0] === 17 && beforeObjectVars[1] === undefined && beforeObjectVars[2] === undefined && beforeObjectVars[3] === undefined, 'object-var-names-initialized-before-pattern');
next(iterator, undefined, 'new-object-var', false);
gc(); result = iterator.next(whole);
var afterObjectVars = readObjectVars();
check(result.done && result.value === readObjectVars && afterObjectVars[0] === 18 && afterObjectVars[1] === whole && afterObjectVars[2] === whole && afterObjectVars[3].tail === whole, 'object-var-captures-retain-resumed-writes');
function* skippedVarDefault() { var {x = yield 'wrong-var-default'} = {x: whole}; return x; }
next(skippedVarDefault(), undefined, whole, true);
function* yieldedVarObject() { var {x, ...rest} = yield 'var-source'; return [x, rest]; }
iterator = yieldedVarObject(); next(iterator, undefined, 'var-source', false);
result = iterator.next({x: whole, tail: whole});
check(result.done && result.value[0] === whole && result.value[1].tail === whole, 'object-var-names-from-suspended-source');

// Rest observes the original own-key order and excludes retained Symbol keys.
events = [];
var restSymbol = Symbol('rest');
var proxy = new Proxy({ x: undefined, tail: whole, [restSymbol]: 11 }, {
  ownKeys: function (target) { events.push('keys'); return Reflect.ownKeys(target); },
  getOwnPropertyDescriptor: function (target, key) { events.push('descriptor'); return Reflect.getOwnPropertyDescriptor(target, key); },
  get: function (target, key, receiver) { events.push(key === restSymbol ? 'symbol' : key); return Reflect.get(target, key, receiver); }
});
function* proxyRest(input) { let {[yield 'proxy-key']: selectedValue = yield 'proxy-default', ...rest} = input; return rest; }
iterator = proxyRest(proxy); next(iterator, undefined, 'proxy-key', false);
next(iterator, 'x', 'proxy-default', false); check(events.join(',') === 'x', 'proxy-get-before-default');
gc(); result = iterator.next(whole);
check(result.done && result.value.tail === whole && result.value[restSymbol] === 11, 'proxy-rest-whole');
check(Object.getPrototypeOf(result.value) === Object.prototype, 'proxy-rest-prototype');
check(events.join(',') === 'x,keys,descriptor,tail,descriptor,symbol', 'rest-exclusion-and-trap-order');

// Member Put nullish refusal happens after default, before raw key coercion.
var coerced = 0;
var badKey = { [Symbol.toPrimitive]: function () { coerced++; return 'x'; } };
function* nullishTarget() { ({x: (yield 'null-base')[yield 'null-key'] = yield 'null-default'} = {}); }
iterator = nullishTarget(); next(iterator, undefined, 'null-base', false);
next(iterator, null, 'null-key', false); next(iterator, badKey, 'null-default', false);
try { iterator.next(whole); throw 'missing-target-typeerror'; }
catch (error) { check(error instanceof TypeError && coerced === 0, 'put-nullish-before-key-conversion'); }

class Holder {
  #value;
  *fill(input) { return ({x: (yield 'private-base').#value = yield 'private-default'} = input); }
  read() { return this.#value; }
}
var holder = new Holder(), privateSource = {};
iterator = holder.fill(privateSource); next(iterator, undefined, 'private-base', false);
next(iterator, holder, 'private-default', false); gc(); next(iterator, whole, privateSource, true);
check(holder.read() === whole, 'original-private-write');
iterator = holder.fill({}); next(iterator, undefined, 'private-base', false);
next(iterator, {}, 'private-default', false);
try { iterator.next(whole); throw 'missing-brand-error'; }
catch (error) { check(error instanceof TypeError, 'brand-check-after-default'); }

// A selected binding survives delegated done:false and each classic iteration
// captures a distinct real head cell.
var delegatedIndex = 0;
var delegated = { [Symbol.iterator]: function () { return { next: function () {
  delegatedIndex++; return delegatedIndex === 1 ? { value: 'delegate', done: false } : { value: whole, done: true };
} }; } };
function* delegateDefault() { var value; ({x: value = yield* delegated} = {}); return value; }
iterator = delegateDefault(); next(iterator, undefined, 'delegate', false); gc(); next(iterator, undefined, whole, true);
function* head(input) { for (let {[yield 'head-key']: i = yield 'head-default', box: {step}} = input; i < 2; i++) { yield () => [i, step]; } }
iterator = head({ box: {step: 3} }); next(iterator, undefined, 'head-key', false);
next(iterator, 'i', 'head-default', false);
var first = iterator.next(0).value; gc(); var second = iterator.next().value;
check(first()[0] === 0 && second()[0] === 1 && first()[1] === 3 && second()[1] === 3, 'distinct-head-closure-cells');
check(iterator.next().done, 'head-completes');

// Injected whole completions cross yielding finalizers without performing Put.
destination.pending = 5;
function* interrupted() { try { ({x: destination.pending = yield 'pending-default'} = {}); } finally { yield 'pending-finally'; } }
iterator = interrupted(); next(iterator, undefined, 'pending-default', false);
result = iterator.return(whole); check(result.value === 'pending-finally' && !result.done, 'return-finalizer');
gc(); next(iterator, undefined, whole, true); check(destination.pending === 5, 'no-put-after-return');
iterator = interrupted(); next(iterator, undefined, 'pending-default', false);
result = iterator.throw(whole); check(result.value === 'pending-finally' && !result.done, 'throw-finalizer');
try { iterator.next(); throw 'missing-whole-throw'; }
catch (error) { check(error === whole && destination.pending === 5, 'whole-throw-no-put'); }
function* caught() { var value = 0; try { ({x: value = yield 'caught-default'} = {}); } catch (error) { yield error; } ({x: value = yield 'fresh-default'} = {}); return value; }
iterator = caught(); next(iterator, undefined, 'caught-default', false);
result = iterator.throw(whole); check(result.value === whole && !result.done, 'caught-whole-reference-abrupt');
next(iterator, undefined, 'fresh-default', false); gc(); next(iterator, 23, 23, true);
// A borrowed main-Realm next must allocate suspended foreign rest in the
// generator's execution Realm, retaining getters and whole Symbol values.
var foreignRealm = __lilaCreateRealm();
var foreignObjectPrototype = foreignRealm.global.Object.prototype;
var foreignFactory = foreignRealm.evalScript('function* copy(input) { const {[yield "foreign-key"]: selected = yield "foreign-default", ...rest} = input; yield rest; return [selected, rest]; } copy;');
var mainNext = Object.getPrototypeOf((function* () {})()).next;
var foreignEvents = [], foreignInput = {};
Object.defineProperty(foreignInput, 'x', { enumerable: true, get: function () {
  foreignEvents.push('x'); return undefined;
} });
Object.defineProperty(foreignInput, 'tail', { enumerable: true, get: function () {
  foreignEvents.push('tail'); return whole;
} });
foreignInput[restSymbol] = whole;
iterator = foreignFactory(foreignInput);
result = mainNext.call(iterator);
check(!result.done && result.value === 'foreign-key', 'foreign-key-suspension');
result = mainNext.call(iterator, 'x');
check(!result.done && result.value === 'foreign-default' && foreignEvents.join(',') === 'x', 'foreign-get-before-default');
gc(); result = mainNext.call(iterator, whole);
var foreignRest = result.value;
check(!result.done && Object.getPrototypeOf(foreignRest) === foreignObjectPrototype, 'foreign-rest-execution-realm');
check(Object.getPrototypeOf(foreignRest) !== Object.prototype, 'foreign-rest-differs-from-caller-realm');
check(foreignRest.tail === whole && foreignRest[restSymbol] === whole, 'foreign-rest-retains-whole-values');
check(!Object.prototype.hasOwnProperty.call(foreignRest, 'x') && foreignEvents.join(',') === 'x,tail', 'foreign-rest-exclusion-get-order');
gc(); result = mainNext.call(iterator);
check(result.done && result.value[0] === whole && result.value[1] === foreignRest, 'foreign-rest-retained-through-next-yield');
print('generator-object-patterns:ok');
