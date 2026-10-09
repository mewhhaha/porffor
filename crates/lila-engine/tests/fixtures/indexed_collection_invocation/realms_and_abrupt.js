var $262 = { createRealm: __lilaCreateRealm };
const foreign = $262.createRealm().global;
const LocalArray = Array;
const ForeignArray = foreign.Array;
const LocalObject = Object;
const ForeignObject = foreign.Object;
const localArrayPrototype = LocalArray.prototype;
const foreignArrayPrototype = ForeignArray.prototype;
const localTypeErrorPrototype = TypeError.prototype;
const foreignTypeErrorPrototype = foreign.TypeError.prototype;
const localJoin = LocalArray.prototype.join;
const foreignJoin = ForeignArray.prototype.join;
const localMap = LocalArray.prototype.map;
const foreignMap = ForeignArray.prototype.map;
const localSorted = LocalArray.prototype.toSorted;
const foreignSorted = ForeignArray.prototype.toSorted;
const localFind = Int16Array.prototype.find;
const foreignFind = foreign.Int16Array.prototype.find;
const marker = new foreign.Error('indexed-collection marker');
function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'public or canonical method must not be selected'; }
const define = LocalObject.defineProperty;
const getPrototypeOf = LocalObject.getPrototypeOf;
Array = poison; foreign.Array = poison;
Object = poison; foreign.Object = poison;
TypeError = poison; foreign.TypeError = poison;

const foreignReceiver = LocalObject.create(ForeignObject.prototype);
foreignReceiver.length = 2; foreignReceiver[0] = 4; foreignReceiver[1] = 5;
foreignReceiver.alias = localMap; foreignReceiver.map = poison;
define(foreignReceiver, 'constructor', {get() { throw 'generic map must not use species'; }});
const localMapped = foreignReceiver.alias(value => value + 1);
check(getPrototypeOf(localMapped) === localArrayPrototype && localMapped[0] === 5 && localMapped[1] === 6,
  'local called method result Realm on foreign receiver');
const localReceiver = LocalObject.create(LocalObject.prototype);
localReceiver.length = 2; localReceiver[0] = 4; localReceiver[1] = 5;
localReceiver.alias = foreignMap; localReceiver.map = poison;
define(localReceiver, 'constructor', {get() { throw 'generic map must not use species'; }});
const foreignMapped = localReceiver.alias(value => value + 1);
check(getPrototypeOf(foreignMapped) === foreignArrayPrototype && foreignMapped[0] === 5 && foreignMapped[1] === 6,
  'foreign called method result Realm on local receiver');

function nativeErrors(joinMethod, sortedMethod, findMethod, errorPrototype) {
  let finalized = 0;
  let caught;
  try { Reflect.apply(joinMethod, null, [':']); }
  catch (error) { caught = error; }
  finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === errorPrototype, 'join native called Realm');
  const invalidSorted = {length: 1, 0: 4, alias: sortedMethod, toSorted: poison};
  caught = undefined;
  try { invalidSorted.alias(17); }
  catch (error) { caught = error; }
  finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === errorPrototype, 'toSorted native called Realm');
  const invalidFind = {length: 1, 0: 4, alias: findMethod, find: poison};
  caught = undefined;
  try { invalidFind.alias(() => true); }
  catch (error) { caught = error; }
  finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === errorPrototype && finalized === 3, 'TypedArray native brand called Realm');
}
nativeErrors(localJoin, localSorted, localFind, localTypeErrorPrototype);
nativeErrors(foreignJoin, foreignSorted, foreignFind, foreignTypeErrorPrototype);

const localCopyReceiver = LocalObject.create(ForeignObject.prototype);
localCopyReceiver.length = 2; localCopyReceiver[0] = 3; localCopyReceiver[1] = 1;
localCopyReceiver.alias = localSorted; localCopyReceiver.toSorted = poison;
const localCopy = localCopyReceiver.alias((a, b) => a - b);
check(getPrototypeOf(localCopy) === localArrayPrototype && localCopy[0] === 1 && localCopy[1] === 3 && localCopyReceiver[0] === 3,
  'local copy Realm and unchanged source');
const foreignCopyReceiver = LocalObject.create(LocalObject.prototype);
foreignCopyReceiver.length = 2; foreignCopyReceiver[0] = 3; foreignCopyReceiver[1] = 1;
foreignCopyReceiver.alias = foreignSorted; foreignCopyReceiver.toSorted = poison;
const foreignCopy = foreignCopyReceiver.alias((a, b) => a - b);
check(getPrototypeOf(foreignCopy) === foreignArrayPrototype && foreignCopy[0] === 1 && foreignCopy[1] === 3,
  'foreign copy Realm after public globals poisoned');

let trace = [];
let prior = 17;
let finalized = 0;
let result = 'unpublished';
let caught;
const inaccessible = {get alias() { trace.push('callee'); throw marker; }, join: poison};
function argument() { trace.push('argument'); return ':'; }
try { prior = 18; result = inaccessible.alias(argument()); }
catch (error) { caught = error; }
finally { finalized++; trace.push('finally'); }
check(caught === marker && prior === 18 && result === 'unpublished' && finalized === 1 &&
  trace.join(',') === 'callee,finally', 'original callee Get skips arguments');

trace = []; caught = undefined; result = 'unpublished';
const argumentReceiver = {get length() { throw 'native body before arguments'; }, alias: localJoin, join: poison};
function failingArgument() { trace.push('argument'); throw marker; }
try { result = argumentReceiver.alias(failingArgument()); }
catch (error) { caught = error; }
finally { trace.push('finally'); }
check(caught === marker && result === 'unpublished' && trace.join(',') === 'argument,finally', 'original argument abrupt skips native method');

trace = []; caught = undefined; result = 'unpublished';
const iterator = {};
let closes = 0;
define(iterator, 'return', {get() { closes++; throw 'spread must not close on value abrupt'; }});
define(iterator, 'next', {get() {
  trace.push('get-next');
  return new Proxy(function() {}, {apply(target, receiver, args) {
    check(receiver === iterator && args.length === 0, 'spread next original receiver');
    trace.push('next');
    return {get done() { trace.push('done'); return false; }, get value() { trace.push('value'); throw marker; }};
  }});
}});
const spread = {get [Symbol.iterator]() {
  trace.push('get-iterator');
  return function() { check(this === spread && arguments.length === 0, 'spread method receiver'); trace.push('open'); return iterator; };
}};
try { result = argumentReceiver.alias(argument(), ...spread); }
catch (error) { caught = error; }
finally { trace.push('finally'); }
check(caught === marker && result === 'unpublished' && closes === 0 &&
  trace.join(',') === 'argument,get-iterator,open,get-next,next,done,value,finally', 'real spread abrupt before method, no close');

trace = []; caught = undefined; result = 'unpublished';
const getterReceiver = {get length() { trace.push('length'); return 1; },
  get 0() { trace.push('get0'); throw marker; }, alias: foreignJoin, join: poison};
try { result = getterReceiver.alias(':'); }
catch (error) { caught = error; }
finally { trace.push('finally'); }
check(caught === marker && result === 'unpublished' && trace.join(',') === 'length,get0,finally', 'native method preserves original foreign getter throw');

trace = []; caught = undefined; result = 'unpublished';
const coercionReceiver = {get length() { trace.push('length'); return 1; },
  get 0() { throw 'element read after separator throw'; }, alias: localJoin, join: poison};
const separator = {toString() { trace.push('separator'); throw marker; }};
try { result = coercionReceiver.alias(separator); }
catch (error) { caught = error; }
finally { trace.push('finally'); }
check(caught === marker && result === 'unpublished' && trace.join(',') === 'length,separator,finally', 'separator original throw after length before element');

trace = []; caught = undefined; result = 'unpublished';
const sorting = {length: 2, 0: 3, 1: 1, alias: LocalArray.prototype.sort, sort: poison};
function comparator() { trace.push('compare'); throw marker; }
try { result = sorting.alias(comparator); }
catch (error) { caught = error; }
finally { trace.push('finally'); }
check(caught === marker && result === 'unpublished' && sorting[0] === 3 && sorting[1] === 1 &&
  trace.join(',') === 'compare,finally', 'comparator throw before writeback');

trace = []; caught = undefined; result = 'unpublished';
const mapping = new ForeignArray(1);
mapping.alias = localMap; mapping.map = poison;
define(mapping, '0', {get() { trace.push('get0'); return 4; }});
define(mapping, 'constructor', {get() {
  trace.push('constructor');
  return {get [Symbol.species]() {
    trace.push('species');
    return function Species(length) { trace.push('construct:' + length); return {}; };
  }};
}});
function callback() { trace.push('callback'); throw marker; }
try { result = mapping.alias(callback); }
catch (error) { caught = error; }
finally { trace.push('finally'); }
check(caught === marker && result === 'unpublished' && mapping.length === 1 &&
  trace.join(',') === 'constructor,species,construct:1,get0,callback,finally', 'species creation precedes original callback throw');

trace = []; caught = undefined; result = 'unpublished';
const speciesFailure = new LocalArray(1);
speciesFailure.alias = foreignMap; speciesFailure.map = poison;
define(speciesFailure, '0', {get() { throw 'element after species abrupt'; }});
speciesFailure.constructor = {get [Symbol.species]() { trace.push('species'); throw marker; }};
try { result = speciesFailure.alias(() => { throw 'callback after species abrupt'; }); }
catch (error) { caught = error; }
finally { trace.push('finally'); }
check(caught === marker && result === 'unpublished' && trace.join(',') === 'species,finally', 'species original abrupt cutoff');

trace = []; caught = undefined; result = 'unpublished';
const filling = {length: 2, 0: 1, alias: LocalArray.prototype.fill, fill: poison};
define(filling, '1', {configurable: true, set(value) { check(value === 9, 'fill setter value'); trace.push('set1'); throw marker; }});
try { result = filling.alias(9); }
catch (error) { caught = error; }
finally { trace.push('finally'); }
check(caught === marker && result === 'unpublished' && filling[0] === 9 && trace.join(',') === 'set1,finally', 'setter original throw retains prior writes');

trace = [];
const proxyTarget = {length: 2, 0: 'a', 1: 'b', join: poison};
const callProxy = new Proxy(foreignJoin, {apply(target, receiver, args) {
  check(target === foreignJoin && receiver === receiverProxy && args.length === 2 && args[1] === 19, 'Proxy apply actual receiver and argc');
  trace.push('apply'); return Reflect.apply(target, receiver, args);
}});
proxyTarget.alias = callProxy;
const receiverProxy = new Proxy(proxyTarget, {get(target, key, receiver) {
  check(target === proxyTarget && receiver === receiverProxy, 'Proxy Get actual receiver');
  trace.push('get:' + key); return Reflect.get(target, key, receiver);
}});
function proxyArgument() { trace.push('argument'); proxyTarget.alias = poison; return ':'; }
check(receiverProxy.alias(proxyArgument(), 19) === 'a:b' &&
  trace.join(',') === 'get:alias,argument,apply,get:length,get:0,get:1', 'retained callable Proxy after replacement and original Get once');
print('indexed-collection-abrupt:ok');
262;
