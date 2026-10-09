function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + ' value');
  same(result.done, done, label + ' done');
}
const primitiveKey = Symbol('primitive head');
let primitiveWrites = 0;
let primitiveValue;
let primitiveReceiver;
const primitiveSetter = new Proxy(function (value) {
  'use strict';
  same(this, 'primitive', 'setter unboxed receiver');
  ++primitiveWrites;
  primitiveValue = value;
  primitiveReceiver = this;
}, {
  apply(target, receiver, args) {
    same(receiver, 'primitive', 'callable Proxy setter raw receiver');
    same(args.length, 1, 'setter argument count');
    return Reflect.apply(target, receiver, args);
  }
});
Object.defineProperty(String.prototype, primitiveKey, {
  configurable: true,
  get() { throw 'plain head cannot read inherited getter'; },
  set: primitiveSetter
});
function* primitiveWalk(source) {
  for ('primitive'[primitiveKey] of source) {
    yield primitiveValue;
    yield primitiveReceiver;
  }
  return primitiveValue;
}
try {
  const consumer = primitiveWalk([4, 5]);
  step(consumer.next(), 4, false, 'primitive first entry');
  primitiveValue = 700;
  step(consumer.next(), 'primitive', false, 'primitive first resume');
  same(primitiveValue, 700, 'primitive resume no setter');
  same(primitiveWrites, 1, 'primitive setter once on first entry');
  step(consumer.next(), 5, false, 'primitive second entry');
  primitiveValue = 900;
  step(consumer.next(), 'primitive', false, 'primitive second resume');
  step(consumer.next(), 900, true, 'primitive normal exhaustion');
  same(primitiveWrites, 2, 'primitive setter once per incoming value');
} finally {
  delete String.prototype[primitiveKey];
}

const foreign = __lilaCreateRealm();
const throwMarker = new foreign.global.Object();
const closeMarker = new foreign.global.Object();
function makeWalk(mode, closeThrows, state) {
  const target = new Proxy({}, {
    get() { throw 'finalizer head cannot Get'; },
    set(object, name, value, receiver) {
      same(receiver, target, 'finalizer head receiver');
      same(name, 'field', 'finalizer head key');
      ++state.writes;
      state.stored = value;
      state.log.push('set:' + value);
      return true;
    }
  });
  function base() { ++state.bases; state.log.push('base'); return target; }
  function key() { ++state.keys; state.log.push('key'); return 'field'; }
  const iterator = {
    next() {
      same(this, iterator, 'finalizer next receiver');
      ++state.next;
      state.log.push('next:' + state.next);
      return { value: state.next, done: false };
    },
    get return() {
      ++state.returnGets;
      state.log.push('get:return');
      return function () {
        same(this, iterator, 'finalizer close receiver');
        same(arguments.length, 0, 'finalizer close argc');
        ++state.close;
        state.log.push('close');
        if (closeThrows) throw closeMarker;
        return {};
      };
    }
  };
  const source = { [Symbol.iterator]() { return iterator; } };
  function* walk() {
    for (base()[key()] of source) {
      try {
        try {
          state.log.push('body');
          yield 'body';
          state.log.push('resumed');
          if (mode === 'continue' && state.writes === 1) continue;
          if (mode === 'break' || mode === 'continue') break;
          if (mode === 'return') return 'returned';
          if (mode === 'throw') throw throwMarker;
        } finally {
          state.log.push('inner');
          yield 'inner';
          state.log.push('inner-done');
        }
      } finally {
        state.log.push('outer');
        yield 'outer';
        state.log.push('outer-done');
      }
    }
    return 'loop-done';
  }
  return walk();
}
function state() {
  return { writes: 0, bases: 0, keys: 0, next: 0, returnGets: 0, close: 0, stored: 0, log: [] };
}
function pendingFinalizers(consumer, observed, mode, first) {
  const writes = observed.writes;
  const next = observed.next;
  step(first, 'inner', false, mode + ' inner finalizer');
  same(observed.close, 0, mode + ' no close before inner finalizer');
  observed.stored = 800;
  step(consumer.next(), 'outer', false, mode + ' outer finalizer');
  same(observed.stored, 800, mode + ' resume cannot replay property Set');
  same(observed.writes, writes, mode + ' pending setter count');
  same(observed.bases, writes, mode + ' pending base count');
  same(observed.keys, writes, mode + ' pending key count');
  same(observed.next, next, mode + ' pending next count');
  same(observed.returnGets, 0, mode + ' no close Get during finalizers');
  same(observed.close, 0, mode + ' no close during finalizers');
}
for (const mode of ['break', 'return', 'throw', 'injected-return', 'injected-throw', 'return-close-throws']) {
  const observed = state();
  const consumer = makeWalk(mode === 'return-close-throws' ? 'return' : mode, mode === 'throw' || mode === 'injected-throw' || mode === 'return-close-throws', observed);
  step(consumer.next(), 'body', false, mode + ' body entry');
  const first = mode === 'injected-return' ? consumer.return('injected') :
    mode === 'injected-throw' ? consumer.throw(throwMarker) : consumer.next();
  pendingFinalizers(consumer, observed, mode, first);
  const throws = mode === 'throw' || mode === 'injected-throw' || mode === 'return-close-throws';
  if (throws) {
    let caught;
    try { consumer.next(); } catch (error) { caught = error; }
    same(caught, mode === 'return-close-throws' ? closeMarker : throwMarker, mode + ' selected completion identity');
  } else {
    step(consumer.next(), mode === 'break' ? 'loop-done' : mode === 'return' ? 'returned' : 'injected', true, mode + ' selected completion');
  }
  same(observed.writes, 1, mode + ' completed head once');
  same(observed.next, 1, mode + ' completed iterator step once');
  same(observed.returnGets, 1, mode + ' close Get once');
  same(observed.close, 1, mode + ' close once');
  const resumed = mode === 'injected-return' || mode === 'injected-throw' ? '' : ',resumed';
  same(observed.log.join(','), 'next:1,base,key,set:1,body' + resumed + ',inner,inner-done,outer,outer-done,get:return,close', mode + ' full chronology');
  step(consumer.next(), undefined, true, mode + ' remains completed');
  same(observed.close, 1, mode + ' completed next no repeat close');
}
const continued = state();
const continuing = makeWalk('continue', false, continued);
step(continuing.next(), 'body', false, 'continue first entry');
pendingFinalizers(continuing, continued, 'continue', continuing.next());
step(continuing.next(), 'body', false, 'local continue next entry');
same(continued.close, 0, 'local continue never closes');
same(continued.writes, 2, 'local continue reevaluates head at next entry');
same(continued.stored, 2, 'local continue incoming next value');
same(continued.log.join(','), 'next:1,base,key,set:1,body,resumed,inner,inner-done,outer,outer-done,next:2,base,key,set:2,body', 'local continue chronology');
pendingFinalizers(continuing, continued, 'break after continue', continuing.next());
step(continuing.next(), 'loop-done', true, 'break after continue');
same(continued.returnGets, 1, 'break after continue close Get once');
same(continued.close, 1, 'break after continue closes once');
same(continued.writes, 2, 'break after continue no replay');
print('ok');
262;
