function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + ' value');
  same(result.done, done, label + ' done');
}
const nativeTypeErrorPrototype = TypeError.prototype;
const getPrototypeOf = Object.getPrototypeOf;
const foreign = __lilaCreateRealm();
const headMarker = new foreign.global.Object();
const closeMarker = new foreign.global.Object();
const strictMode = (function () { return this; })() === undefined;
function source(state, closeThrows) {
  const iterator = {
    next() { ++state.next; state.log.push('next'); return { value: 9, done: false }; },
    get return() {
      ++state.returnGets;
      state.log.push('get:return');
      return function () {
        same(this, iterator, 'head failure close receiver');
        same(arguments.length, 0, 'head failure close argc');
        ++state.close;
        state.log.push('close');
        if (closeThrows) throw closeMarker;
        return {};
      };
    }
  };
  return { [Symbol.iterator]() { return iterator; } };
}
function makeConsumer(mode, state) {
  const target = new Proxy({}, {
    get() { throw 'plain property head cannot Get'; },
    set(object, name, value, receiver) {
      state.log.push('set');
      same(name, 'field', 'failure key');
      same(value, 9, 'failure incoming value');
      same(receiver, target, 'failure raw receiver');
      if (mode === 'set') throw headMarker;
      return mode !== 'false';
    }
  });
  const rawKey = { [Symbol.toPrimitive](hint) {
    same(hint, 'string', 'failure key hint');
    state.log.push('coerce');
    if (mode === 'coerce') throw headMarker;
    return 'field';
  } };
  function base() {
    state.log.push('base');
    if (mode === 'base') throw headMarker;
    return mode === 'null' ? null : target;
  }
  function key() {
    state.log.push('key');
    if (mode === 'key') throw headMarker;
    return rawKey;
  }
  function* walk(iterable) {
    try {
      for (base()[key()] of iterable) {
        ++state.body;
        state.log.push('body');
        yield 'body';
      }
    } catch (error) {
      state.error = error;
      state.log.push('catch');
    } finally {
      state.log.push('finally');
      yield 'cleanup';
      state.log.push('finalized');
    }
    return 'done';
  }
  return walk(source(state, true));
}
function checkFailure(mode, prefix, nativeError) {
  const state = { next: 0, close: 0, returnGets: 0, body: 0, error: undefined, log: [] };
  const consumer = makeConsumer(mode, state);
  step(consumer.next(), 'cleanup', false, mode + ' reaches outer cleanup');
  if (nativeError) same(getPrototypeOf(state.error), nativeTypeErrorPrototype, mode + ' native source Realm');
  else same(state.error, headMarker, mode + ' original foreign throw');
  same(state.error === closeMarker, false, mode + ' head Throw wins over close Throw');
  same(state.log.join(','), prefix + ',get:return,close,catch,finally', mode + ' exact close chronology');
  same(state.body, 0, mode + ' body not entered');
  same(state.next, 1, mode + ' next once');
  same(state.returnGets, 1, mode + ' return Get once');
  same(state.close, 1, mode + ' close before yielding outer finally');
  step(consumer.next(), 'done', true, mode + ' finalizer completed');
  step(consumer.next(), undefined, true, mode + ' remains completed');
  same(state.log.join(','), prefix + ',get:return,close,catch,finally,finalized', mode + ' no replay');
}
checkFailure('base', 'next,base', false);
checkFailure('key', 'next,base,key', false);
checkFailure('coerce', 'next,base,key,coerce', false);
checkFailure('set', 'next,base,key,coerce,set', false);
checkFailure('null', 'next,base,key', true);
if (strictMode) {
  checkFailure('false', 'next,base,key,coerce,set', true);
} else {
  const state = { next: 0, close: 0, returnGets: 0, body: 0, error: undefined, log: [] };
  const consumer = makeConsumer('false', state);
  step(consumer.next(), 'body', false, 'sloppy false Set enters body');
  same(state.body, 1, 'sloppy false Set body once');
  same(state.error, undefined, 'sloppy false Set does not throw');
  step(consumer.return('stop'), 'cleanup', false, 'sloppy false Set closes before outer cleanup');
  same(state.error, closeMarker, 'close Throw replaces injected Return');
  same(state.log.join(','), 'next,base,key,coerce,set,body,get:return,close,catch,finally', 'sloppy false Set close chronology');
  step(consumer.next(), 'done', true, 'sloppy outer cleanup completed');
  same(state.close, 1, 'sloppy false Set closes once');
}

let baseCalls = 0;
let closes = 0;
const iteratorMarker = new foreign.global.Object();
function baseAfterStep() { ++baseCalls; return {}; }
function* afterStep(iterable) { for (baseAfterStep().field of iterable) { yield 'unreached'; } }
const badStep = { [Symbol.iterator]() {
  return {
    next() { throw iteratorMarker; },
    return() { ++closes; return {}; }
  };
} };
let caught;
try { afterStep(badStep).next(); } catch (error) { caught = error; }
same(caught, iteratorMarker, 'iterator failure original throw');
same(baseCalls, 0, 'iterator failure before head evaluation');
same(closes, 0, 'iterator failure does not enter close owner');
print('ok');
262;
