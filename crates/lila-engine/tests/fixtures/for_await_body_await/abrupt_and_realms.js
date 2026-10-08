function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
const realm = __lilaCreateRealm();
const foreign = realm.global;
const LocalTypeErrorPrototype = TypeError.prototype;
const ForeignTypeErrorPrototype = foreign.TypeError.prototype;
const ForeignError = foreign.Error;
const marker = new ForeignError('body-marker');
const closeMarker = new ForeignError('close-marker');
const localTypeDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'TypeError');
const foreignTypeDescriptor = Object.getOwnPropertyDescriptor(foreign, 'TypeError');
const foreignConsume = realm.evalScript("async function consume(source) { for await (const value of source) { await 0; break; } return 42; } consume;");
function source(state, prototype) {
  const iterator = Object.create(prototype);
  iterator.next = new Proxy(function () {}, {
    apply(target, receiver, args) {
      same(receiver, iterator, 'abrupt next receiver');
      same(args.length, 0, 'abrupt next arguments');
      state.nexts++; state.events.push('next');
      if (state.mode === 'nextThrow') throw state.marker;
      if (state.mode === 'nextReject') return Promise.reject(state.marker);
      if (state.mode === 'primitiveNext') return Promise.resolve(7);
      return Promise.resolve({
        get done() {
          state.doneGets++; state.events.push('done');
          if (state.mode === 'doneThrow') throw state.marker;
          return state.nexts > 2;
        },
        get value() {
          state.valueGets++; state.events.push('value');
          if (state.mode === 'valueThrow') throw state.marker;
          return state.nexts;
        }
      });
    }
  });
  Object.defineProperty(iterator, 'return', {
    configurable: true,
    get() {
      state.returnGets++; state.events.push('get:return');
      if (state.closeMode === 'getThrow') throw closeMarker;
      if (state.closeMode === 'noncallable') return 7;
      return new Proxy(function () {}, {
        apply(target, receiver, args) {
          same(receiver, iterator, 'abrupt close receiver');
          same(args.length, 0, 'abrupt close arguments');
          state.closes++; state.events.push('close');
          if (state.closeMode === 'callThrow') throw closeMarker;
          if (state.closeMode === 'primitive') return Promise.resolve(7);
          if (state.closeMode === 'reject') return Promise.reject(closeMarker);
          return Promise.resolve({ done: true });
        }
      });
    }
  });
  return { [Symbol.asyncIterator]() { return iterator; } };
}
function state(mode, closeMode, original) {
  return { mode, closeMode, marker: original, events: [], nexts: 0,
    doneGets: 0, valueGets: 0, body: 0, returnGets: 0, closes: 0 };
}
async function escaping(iterable, state) {
  try {
    for await (const value of iterable) {
      state.body++; state.events.push('body');
      await Promise.reject(state.marker);
      throw 'rejected body resumed normally';
    }
  } finally {
    await 0;
    state.events.push('outer:finally');
  }
}
async function stepFailure(mode, expectedTrace, expectedDone, expectedValue) {
  const selected = state(mode, 'reject', marker);
  const iterable = source(selected, foreign.Object.prototype);
  let observed = closeMarker;
  try { await escaping(iterable, selected); }
  catch (error) { observed = error; selected.events.push('caught'); }
  same(observed, marker, 'original step failure identity');
  same(Object.getPrototypeOf(observed), ForeignError.prototype, 'original step failure Realm');
  same(selected.events.join(','), expectedTrace, 'step abrupt cutoff');
  same(selected.nexts, 1, 'failed step is not repeated');
  same(selected.doneGets, expectedDone, 'step done observation');
  same(selected.valueGets, expectedValue, 'step value observation');
  same(selected.body, 0, 'failed step never enters awaited body');
  same(selected.returnGets, 0, 'failed step does not Get return');
  same(selected.closes, 0, 'failed step does not close');
}
async function bodyFailure(closeMode, original, closes) {
  const selected = state('normal', closeMode, original);
  const iterable = source(selected, foreign.Object.prototype);
  let observed = closeMarker;
  try { await escaping(iterable, selected); }
  catch (error) { observed = error; selected.events.push('caught'); }
  same(observed, original, 'body rejection wins over all close failures');
  if (original === marker) {
    same(Object.getPrototypeOf(observed), ForeignError.prototype, 'body rejection Realm');
  }
  same(selected.events.join(','),
    'next,done,value,body,get:return,' + (closes ? 'close,' : '') + 'outer:finally,caught',
    'body close precedes outer awaited finally and catch');
  same(selected.nexts, 1, 'body rejection cannot request a following step');
  same(selected.body, 1, 'body is not replayed after rejection');
  same(selected.returnGets, 1, 'body rejection Gets return once');
  same(selected.closes, closes, 'body rejection close call count');
}
async function recovering(iterable, selected) {
  for await (const value of iterable) {
    try {
      selected.body++; selected.events.push('body:' + value);
      await Promise.reject(undefined);
      throw 'undefined rejection resumed normally';
    } catch (error) {
      same(error, undefined, 'caught undefined is an abrupt value');
      await 0;
      selected.events.push('catch:' + value);
    } finally {
      await 0;
      selected.events.push('finally:' + value);
    }
  }
  await 0;
  selected.events.push('tail');
}
async function localConsume(iterable) {
  for await (const value of iterable) { await 0; break; }
  return 42;
}
async function nativeFailure(consume, sourcePrototype, expectedPrototype, mode, closeMode) {
  const selected = state(mode, closeMode, marker);
  const iterable = source(selected, sourcePrototype);
  let observed = marker;
  try { await consume(iterable); }
  catch (error) { observed = error; }
  same(Object.getPrototypeOf(observed), expectedPrototype, 'current async execution Realm native error');
  same(observed === marker || observed === closeMarker, false, 'native error remains distinct from markers');
  same(selected.nexts, 1, 'native failure next call count');
  if (mode === 'primitiveNext') {
    same(selected.returnGets, 0, 'invalid next result does not Get return');
    same(selected.closes, 0, 'invalid next result does not close');
  } else {
    same(selected.returnGets, 1, 'native close error observes return once');
    same(selected.closes, closeMode === 'noncallable' ? 0 : 1, 'native close call count');
  }
}
async function run() {
  await stepFailure('nextThrow', 'next,outer:finally,caught', 0, 0);
  await stepFailure('nextReject', 'next,outer:finally,caught', 0, 0);
  await stepFailure('doneThrow', 'next,done,outer:finally,caught', 1, 0);
  await stepFailure('valueThrow', 'next,done,value,outer:finally,caught', 1, 1);
  await bodyFailure('getThrow', marker, 0);
  await bodyFailure('noncallable', marker, 0);
  await bodyFailure('callThrow', marker, 1);
  await bodyFailure('primitive', marker, 1);
  await bodyFailure('reject', marker, 1);
  await bodyFailure('reject', undefined, 1);
  const recovered = state('normal', 'reject', marker);
  await recovering(source(recovered, foreign.Object.prototype), recovered);
  same(recovered.events.join(','),
    'next,done,value,body:1,catch:1,finally:1,' +
    'next,done,value,body:2,catch:2,finally:2,next,done,tail',
    'caught body rejection continues from its own clause and next states');
  same(recovered.body, 2, 'each recoverable body entered once');
  same(recovered.returnGets, 0, 'caught rejection and natural done never Get return');
  same(recovered.closes, 0, 'caught rejection never closes');

  function poisoned() { throw 'mutable TypeError constructor observed'; }
  globalThis.TypeError = poisoned;
  foreign.TypeError = poisoned;
  try {
    await nativeFailure(localConsume, foreign.Object.prototype, LocalTypeErrorPrototype, 'primitiveNext', 'normal');
    await nativeFailure(foreignConsume, Object.prototype, ForeignTypeErrorPrototype, 'primitiveNext', 'normal');
    await nativeFailure(localConsume, foreign.Object.prototype, LocalTypeErrorPrototype, 'normal', 'noncallable');
    await nativeFailure(foreignConsume, Object.prototype, ForeignTypeErrorPrototype, 'normal', 'noncallable');
    await nativeFailure(localConsume, foreign.Object.prototype, LocalTypeErrorPrototype, 'normal', 'primitive');
    await nativeFailure(foreignConsume, Object.prototype, ForeignTypeErrorPrototype, 'normal', 'primitive');
  } finally {
    Object.defineProperty(globalThis, 'TypeError', localTypeDescriptor);
    Object.defineProperty(foreign, 'TypeError', foreignTypeDescriptor);
  }
}
run().then(function () { print('for-await-abrupt-realms:ok'); },
  function (error) { print('unexpected:' + String(error)); });
262;
