const other = __lilaCreateRealm().global;
const LocalIterator = Iterator;
const ForeignIterator = other.Iterator;
const localFrom = LocalIterator.from;
const foreignFrom = ForeignIterator.from;
const localIteratorPrototype = LocalIterator.prototype;
const foreignIteratorPrototype = ForeignIterator.prototype;
const localObjectPrototype = Object.prototype;
const foreignObjectPrototype = other.Object.prototype;
const localStringPrototype = String.prototype;
const foreignStringPrototype = other.String.prototype;
const getPrototypeOf = Object.getPrototypeOf;
const getDescriptor = Object.getOwnPropertyDescriptor;
const defineProperty = Object.defineProperty;
const create = Object.create;
const same = Object.is;
const hasOwn = Object.prototype.hasOwnProperty;
const localWrapperPrototype = getPrototypeOf(localFrom({next() { return 0; }}));
const foreignWrapperPrototype = getPrototypeOf(foreignFrom({next() { return 0; }}));
const localNext = localWrapperPrototype.next;
const foreignNext = foreignWrapperPrototype.next;
const localReturn = localWrapperPrototype.return;
const foreignReturn = foreignWrapperPrototype.return;
function wrong() { throw 'public Iterator or Object constructor'; }
globalThis.Iterator = wrong; other.Iterator = wrong;
globalThis.Object = wrong; other.Object = wrong;

function checkDone(result, prototype) {
  if (getPrototypeOf(result) !== prototype || result.done !== true || result.value !== undefined) throw 'fresh called-Realm done result';
  if (!hasOwn.call(result, 'done') || !hasOwn.call(result, 'value')) throw 'own done fields';
}
function restore(prototype, key, descriptor) {
  if (descriptor === undefined) delete prototype[key];
  else defineProperty(prototype, key, descriptor);
}

let forwarded = 0;
for (let direction = 0; direction < 2; direction++) {
  const from = direction === 0 ? localFrom : foreignFrom;
  const otherFrom = direction === 0 ? foreignFrom : localFrom;
  const fromReceiver = direction === 0 ? ForeignIterator : LocalIterator;
  const nextMethod = direction === 0 ? localNext : foreignNext;
  const returnMethod = direction === 0 ? localReturn : foreignReturn;
  const wrapperPrototype = direction === 0 ? localWrapperPrototype : foreignWrapperPrototype;
  const iteratorPrototype = direction === 0 ? localIteratorPrototype : foreignIteratorPrototype;
  const objectPrototype = direction === 0 ? localObjectPrototype : foreignObjectPrototype;
  const sourceObjectPrototype = direction === 0 ? foreignObjectPrototype : localObjectPrototype;
  const stringPrototype = direction === 0 ? localStringPrototype : foreignStringPrototype;
  const symbol = Symbol('forwarded');
  const opaque = {get done() { throw 'wrapper inspected done'; }, get value() { throw 'wrapper inspected value'; }};
  const values = [undefined, null, false, 0, -0, NaN, 8n, 'primitive', symbol, opaque];
  const trace = [];
  const underlying = create(sourceObjectPrototype);
  let nextCalls = 0;
  const nextTarget = function() { throw 'direct next target'; };
  const next = new Proxy(nextTarget, {apply(target, receiver, args) {
    if (target !== nextTarget || receiver !== underlying || args.length !== 0) throw 'wrapper next Call receiver and argc0';
    trace.push('next.apply'); return values[nextCalls++];
  }});
  defineProperty(underlying, 'next', {configurable: true, get() { trace.push('next.get'); return next; }});
  const methodTarget = function() { throw 'direct iterator method'; };
  const method = new Proxy(methodTarget, {apply(target, receiver, args) {
    if (target !== methodTarget || receiver !== iterable || args.length !== 0) throw 'original object receiver and argc0';
    trace.push('iterator.apply'); return underlying;
  }});
  const iterable = {get [Symbol.iterator]() {
    if (this !== iterable) throw 'original getter receiver';
    trace.push('iterator.get'); return method;
  }};
  const wrapper = from.call(fromReceiver, iterable);
  if (wrapper === underlying || getPrototypeOf(wrapper) !== wrapperPrototype) throw 'defining-Realm wrapper';
  if (trace.join(',') !== 'iterator.get,iterator.apply,next.get') throw 'one method Get then cached next Get';
  defineProperty(underlying, 'next', {configurable: true, get() { throw 'cached next reread'; }});
  for (let i = 0; i < values.length; i++) {
    if (!same(nextMethod.call(wrapper, 'ignored', 'also ignored'), values[i])) throw 'arbitrary next Call result';
    forwarded++;
  }
  if (nextCalls !== values.length || trace[2] !== 'next.get' || trace.length !== 3 + values.length) throw 'cached Proxy next count';

  let returnGets = 0;
  let returnCalls = 0;
  const returnTarget = function() { throw 'direct return target'; };
  const returnProxy = new Proxy(returnTarget, {apply(target, receiver, args) {
    if (target !== returnTarget || receiver !== underlying || args.length !== 0) throw 'wrapper return Call receiver and argc0';
    return values[returnCalls++];
  }});
  defineProperty(underlying, 'return', {configurable: true, get() {
    returnGets++;
    if (returnGets === 1) return undefined;
    if (returnGets === 2) return null;
    return returnProxy;
  }});
  const absent = returnMethod.call(wrapper, 'ignored');
  const nullish = returnMethod.call(wrapper, 'ignored');
  checkDone(absent, objectPrototype); checkDone(nullish, objectPrototype);
  if (absent === nullish) throw 'fresh nullable return results';
  for (let i = 0; i < values.length; i++) {
    if (!same(returnMethod.call(wrapper, 'ignored'), values[i])) throw 'arbitrary return Call result';
    forwarded++;
  }
  if (returnGets !== values.length + 2 || returnCalls !== values.length) throw 'return Get remains fresh';

  const identityTrace = [];
  const identity = new Proxy(create(iteratorPrototype), {
    get(target, key, receiver) {
      if (receiver !== identity) throw 'identity receiver';
      if (key === Symbol.iterator) { identityTrace.push('iterator.get'); return null; }
      if (key === 'next') { identityTrace.push('next.get'); return 1; }
      throw 'identity unexpected Get';
    },
    getPrototypeOf() { identityTrace.push('prototype'); return iteratorPrototype; }
  });
  if (from(identity) !== identity || identityTrace.join(',') !== 'iterator.get,next.get,prototype') throw 'next acquired before identity without wrap';
  for (const absentMethod of [undefined, null]) {
    let methodGets = 0;
    let cachedGets = 0;
    const direct = {get [Symbol.iterator]() { methodGets++; return absentMethod; },
      get next() { cachedGets++; return function() { return 'direct'; }; }};
    const directWrapper = from(direct);
    if (directWrapper === direct || nextMethod.call(directWrapper) !== 'direct' || methodGets !== 1 || cachedGets !== 1) throw 'nullish direct Object fallback';
  }

  const originalIterator = getDescriptor(stringPrototype, Symbol.iterator);
  const primitive = 'primitive-source';
  const stringTrace = [];
  const stringUnderlying = {get next() {
    stringTrace.push('next.get');
    return new Proxy(function() {}, {apply(target, receiver, args) {
      if (receiver !== stringUnderlying || args.length !== 0) throw 'String wrapper next Call';
      stringTrace.push('next.apply'); return 'string-result';
    }});
  }};
  const stringMethod = new Proxy(function() {}, {apply(target, receiver, args) {
    if (receiver !== primitive || args.length !== 0) throw 'primitive String original receiver';
    stringTrace.push('iterator.apply'); return stringUnderlying;
  }});
  defineProperty(stringPrototype, Symbol.iterator, {configurable: true, get: function() {
    'use strict';
    if (this !== primitive) throw 'primitive String getter this';
    stringTrace.push('iterator.get'); return stringMethod;
  }});
  const stringWrapper = from.call(fromReceiver, primitive);
  if (getPrototypeOf(stringWrapper) !== wrapperPrototype || nextMethod.call(stringWrapper, 9) !== 'string-result') throw 'String wrapper result';
  restore(stringPrototype, Symbol.iterator, originalIterator);
  if (stringTrace.join(',') !== 'iterator.get,iterator.apply,next.get,next.apply') throw 'String lookup and Call once';

  const oppositeUnderlying = {next() {
    if (this !== oppositeUnderlying || arguments.length !== 0) throw 'borrowed next receiver';
    return 'borrowed';
  }, return: null};
  const oppositeWrapper = otherFrom(oppositeUnderlying);
  if (nextMethod.call(oppositeWrapper, 1) !== 'borrowed') throw 'wrapper methods accept other-Realm records';
  const borrowedDone = returnMethod.call(oppositeWrapper, 1);
  checkDone(borrowedDone, objectPrototype);
}
if (forwarded !== 40) throw 'both Realm primitive and object result forwarding';
print('iterator-from-realms:ok');
262;
