const other = __lilaCreateRealm().global;
const LocalIterator = Iterator;
const ForeignIterator = other.Iterator;
const localFrom = LocalIterator.from;
const foreignFrom = ForeignIterator.from;
const getPrototypeOf = Object.getPrototypeOf;
const getDescriptor = Object.getOwnPropertyDescriptor;
const defineProperty = Object.defineProperty;
const create = Object.create;
const localWrapperPrototype = getPrototypeOf(localFrom({next() { return 0; }}));
const foreignWrapperPrototype = getPrototypeOf(foreignFrom({next() { return 0; }}));
const localNext = localWrapperPrototype.next;
const foreignNext = foreignWrapperPrototype.next;
const localReturn = localWrapperPrototype.return;
const foreignReturn = foreignWrapperPrototype.return;
const localTypePrototype = TypeError.prototype;
const foreignTypePrototype = other.TypeError.prototype;
const localStringPrototype = String.prototype;
const foreignStringPrototype = other.String.prototype;
const ForeignError = other.Error;
const marker = new ForeignError('iterator-from-marker');
function wrong() { throw 'public intrinsic constructor'; }
globalThis.Iterator = wrong; other.Iterator = wrong;
globalThis.Object = wrong; other.Object = wrong;
globalThis.TypeError = wrong; other.TypeError = wrong;

function expectNative(action, prototype) {
  try { action(); } catch (error) {
    if (getPrototypeOf(error) !== prototype) throw 'called-Realm Iterator TypeError';
    return;
  }
  throw 'missing Iterator TypeError';
}
function expectMarker(action, trace, expected) {
  const previous = {};
  let result = previous;
  let caught = false;
  try { result = action(); } catch (error) {
    if (error !== marker || getPrototypeOf(error) !== ForeignError.prototype) throw 'original foreign abrupt identity';
    caught = true; trace.push('catch');
  } finally { trace.push('finally'); }
  if (!caught || result !== previous || trace.join(',') !== expected) throw 'Iterator abrupt before publication';
}
function restore(prototype, key, descriptor) {
  if (descriptor === undefined) delete prototype[key];
  else defineProperty(prototype, key, descriptor);
}

let deferred = 0;
for (let direction = 0; direction < 2; direction++) {
  const from = direction === 0 ? localFrom : foreignFrom;
  const nextMethod = direction === 0 ? localNext : foreignNext;
  const returnMethod = direction === 0 ? localReturn : foreignReturn;
  const typePrototype = direction === 0 ? localTypePrototype : foreignTypePrototype;
  const stringPrototype = direction === 0 ? localStringPrototype : foreignStringPrototype;
  for (const invalid of [undefined, null, 1, true, 1n, Symbol('invalid')]) {
    expectNative(() => from(invalid), typePrototype);
  }
  let laterHooks = 0;
  const noncallable = new Proxy({}, {
    get(target, key) { if (key === Symbol.iterator) return 1; laterHooks++; throw 'next before callable check'; },
    getPrototypeOf() { laterHooks++; throw 'prototype before callable check'; }
  });
  expectNative(() => from(noncallable), typePrototype);
  const primitiveResult = new Proxy({}, {
    get(target, key) { if (key === Symbol.iterator) return function() { return 'primitive-result'; }; laterHooks++; throw 'next before Object result check'; },
    getPrototypeOf() { laterHooks++; throw 'prototype before Object result check'; }
  });
  if (laterHooks !== 0) throw 'invalid acquisition skips later hooks';

  const originalIterator = getDescriptor(stringPrototype, Symbol.iterator);
  const originalNext = getDescriptor(stringPrototype, 'next');
  let stringGets = 0;
  let stringNextGets = 0;
  defineProperty(stringPrototype, 'next', {configurable: true, get() { stringNextGets++; throw 'primitive next Get'; }});
  expectNative(() => from(primitiveResult), typePrototype);
  for (const absent of [undefined, null]) {
    defineProperty(stringPrototype, Symbol.iterator, {configurable: true, get: function() {
      'use strict';
      if (this !== 'absent-string') throw 'absent String original getter receiver';
      stringGets++; return absent;
    }});
    expectNative(() => from('absent-string'), typePrototype);
  }
  restore(stringPrototype, Symbol.iterator, originalIterator);
  restore(stringPrototype, 'next', originalNext);
  if (stringGets !== 2 || stringNextGets !== 0 || laterHooks !== 0) throw 'String nullish fallback and method result Object gates before next';

  for (const fault of ['iterator.get', 'iterator.apply', 'next.get', 'prototype']) {
    const trace = [];
    const candidate = new Proxy({}, {
      get(target, key, receiver) {
        if (receiver !== candidate || key !== 'next') throw 'candidate next lookup';
        trace.push('next.get'); if (fault === 'next.get') throw marker; return function() {};
      },
      getPrototypeOf() { trace.push('prototype'); throw marker; }
    });
    const method = new Proxy(function() {}, {apply(target, receiver, args) {
      if (receiver !== input || args.length !== 0) throw 'acquisition Call original receiver';
      trace.push('iterator.apply'); if (fault === 'iterator.apply') throw marker; return candidate;
    }});
    const input = {get [Symbol.iterator]() {
      if (this !== input) throw 'acquisition Get receiver';
      trace.push('iterator.get'); if (fault === 'iterator.get') throw marker; return method;
    }};
    const operations = ['iterator.get', 'iterator.apply', 'next.get', 'prototype'];
    let expected = '';
    for (const operation of operations) {
      expected += (expected === '' ? '' : ',') + operation;
      if (operation === fault) break;
    }
    expectMarker(() => from(input), trace, expected + ',catch,finally');
  }
  const stringTrace = [];
  defineProperty(stringPrototype, Symbol.iterator, {configurable: true, get: function() {
    'use strict';
    if (this !== 'throwing-string') throw 'throwing String original receiver';
    stringTrace.push('iterator.get'); throw marker;
  }});
  expectMarker(() => from('throwing-string'), stringTrace, 'iterator.get,catch,finally');
  restore(stringPrototype, Symbol.iterator, originalIterator);

  let nextGets = 0;
  const deferredInput = {get [Symbol.iterator]() { return null; }, get next() { nextGets++; return 1; }};
  const deferredWrapper = from(deferredInput);
  if (nextGets !== 1 || deferredWrapper === deferredInput) throw 'arbitrary cached next acquisition';
  defineProperty(deferredInput, 'next', {configurable: true, value() { throw 'cached noncallable replaced'; }});
  expectNative(() => nextMethod.call(deferredWrapper), typePrototype);
  if (nextGets !== 1) throw 'deferred noncallability without reread';
  deferred++;
  const nextTrace = [];
  const nextUnderlying = {next: new Proxy(function() {}, {apply(target, receiver, args) {
    if (receiver !== nextUnderlying || args.length !== 0) throw 'next abrupt Call receiver';
    nextTrace.push('next.apply'); throw marker;
  }})};
  const nextWrapper = from(nextUnderlying);
  expectMarker(() => nextMethod.call(nextWrapper, 9), nextTrace, 'next.apply,catch,finally');
  for (const fault of ['return.get', 'return.apply']) {
    const trace = [];
    const underlying = {next() {}, get return() {
      trace.push('return.get'); if (fault === 'return.get') throw marker;
      return new Proxy(function() {}, {apply(target, receiver, args) {
        if (receiver !== underlying || args.length !== 0) throw 'return abrupt Call receiver';
        trace.push('return.apply'); throw marker;
      }});
    }};
    const wrapper = from(underlying);
    const expected = fault === 'return.get' ? 'return.get,catch,finally' : 'return.get,return.apply,catch,finally';
    expectMarker(() => returnMethod.call(wrapper, 9), trace, expected);
  }

  const revokedMethod = Proxy.revocable(function() {}, {});
  revokedMethod.revoke();
  expectNative(() => from({[Symbol.iterator]: revokedMethod.proxy}), typePrototype);
  const revokedNext = Proxy.revocable(function() {}, {});
  const revokedNextWrapper = from({next: revokedNext.proxy});
  revokedNext.revoke();
  expectNative(() => nextMethod.call(revokedNextWrapper), typePrototype);
  const revokedReturn = Proxy.revocable(function() {}, {});
  revokedReturn.revoke();
  const revokedReturnWrapper = from({next() {}, return: revokedReturn.proxy});
  expectNative(() => returnMethod.call(revokedReturnWrapper), typePrototype);
  let returnGets = 0;
  const noncallableReturnWrapper = from({next() {}, get return() { returnGets++; return 1; }});
  expectNative(() => returnMethod.call(noncallableReturnWrapper), typePrototype);
  if (returnGets !== 1) throw 'noncallable return one Get';

  let traps = 0;
  const validWrapper = from({next() {}});
  const proxyWrapper = new Proxy(validWrapper, {
    get() { traps++; throw marker; },
    has() { traps++; throw marker; },
    getOwnPropertyDescriptor() { traps++; throw marker; },
    getPrototypeOf() { traps++; throw marker; }
  });
  for (const invalid of [undefined, null, 1, {}, create(validWrapper), proxyWrapper]) {
    expectNative(() => nextMethod.call(invalid), typePrototype);
    expectNative(() => returnMethod.call(invalid), typePrototype);
  }
  if (traps !== 0) throw 'internal slot check without Proxy traps';
}
if (deferred !== 2) throw 'both Realm deferred next';
print('iterator-from-abrupt:ok');
262;
