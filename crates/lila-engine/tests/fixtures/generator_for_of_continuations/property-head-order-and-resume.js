function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + ' value');
  same(result.done, done, label + ' done');
}
const log = [];
const state = { steps: 0, nextGets: 0, closes: 0, writes: 0, stored: undefined };
const symbol = Symbol('second');
let currentKey = 'first';
let currentTarget;
const target = new Proxy({}, {
  get(object, key, receiver) {
    if (key === 'first' || key === symbol) throw 'simple head must not Get its property';
    return Reflect.get(object, key, receiver);
  },
  set(object, key, value, receiver) {
    same(receiver, target, 'original proxy receiver');
    same(key, currentKey, 'canonical key');
    log.push('set:' + value);
    ++state.writes;
    state.stored = value;
    iterator.next = function () { throw 'cached next was replaced'; };
    return true;
  }
});
currentTarget = target;
const rawKey = { [Symbol.toPrimitive](hint) {
  same(hint, 'string', 'property key hint');
  log.push('coerce');
  return currentKey;
} };
function base() { log.push('base'); return currentTarget; }
function key() { log.push('key'); return rawKey; }
function cachedNext() {
  same(this, iterator, 'cached next receiver');
  same(arguments.length, 0, 'cached next argc');
  const index = ++state.steps;
  log.push('next:' + index);
  return {
    get done() { log.push('done:' + index); return index > 2; },
    get value() { log.push('value:' + index); return index * 11; }
  };
}
let currentNext = cachedNext;
const iterator = {
  get next() { ++state.nextGets; log.push('get:next'); return currentNext; },
  set next(value) { log.push('replace:next'); currentNext = value; },
  return() { ++state.closes; log.push('close'); return {}; }
};
const source = { [Symbol.iterator]() { log.push('open'); return iterator; } };
function* walk(iterable) {
  for (base()[key()] of iterable) {
    let body = state.stored * 10;
    const readers = [() => state.stored, () => body];
    log.push('body');
    yield readers;
    ++body;
    log.push('resumed');
    yield readers;
  }
  return 'done';
}
const consumer = walk(source);
const first = consumer.next().value;
same(first[0](), 11, 'first incoming value');
same(first[1](), 110, 'first body cell');
same(log.join(','), 'open,get:next,next:1,done:1,value:1,base,key,coerce,set:11,replace:next,body', 'first head chronology');
state.stored = 700;
currentTarget = { first: 'other' };
currentKey = symbol;
step(consumer.next(), first, false, 'first body resumes');
same(first[0](), 700, 'resume cannot rewrite property');
same(first[1](), 111, 'first lexical body resumes');
same(state.writes, 1, 'resume no Set');
same(state.steps, 1, 'resume no iterator step');
same(log.join(','), 'open,get:next,next:1,done:1,value:1,base,key,coerce,set:11,replace:next,body,resumed', 'resume no base or key');
currentTarget = target;
const second = consumer.next().value;
same(second[0](), 22, 'next iteration property');
same(second[1](), 220, 'fresh body cell');
same(first[1](), 111, 'earlier body cell retained');
same(state.writes, 2, 'next entered iteration Set once');
same(state.nextGets, 1, 'next method cached across head effects');
same(log.join(','), 'open,get:next,next:1,done:1,value:1,base,key,coerce,set:11,replace:next,body,resumed,next:2,done:2,value:2,base,key,coerce,set:22,replace:next,body', 'second head chronology');
state.stored = 900;
step(consumer.next(), second, false, 'second body resumes');
step(consumer.next(), 'done', true, 'normal exhaustion');
same(state.stored, 900, 'terminal done cannot assign');
same(state.steps, 3, 'terminal step once');
same(state.closes, 0, 'normal exhaustion never closes');
same(log[log.length - 1], 'done:3', 'terminal value getter absent');
const completedLog = log.join(',');
step(consumer.next(), undefined, true, 'completed consumer');
same(log.join(','), completedLog, 'completed consumer has no property work');

let shadowTarget = {};
function* shadow(iterable) {
  for (shadowTarget.value of iterable) {
    let shadowTarget = { value: 99 };
    yield () => shadowTarget.value;
  }
  return shadowTarget.value;
}
const shadowConsumer = shadow([7]);
const shadowReader = shadowConsumer.next().value;
same(shadowTarget.value, 7, 'head writes outer target before body scope');
same(shadowReader(), 99, 'body captures its own shadow');
step(shadowConsumer.next(), 7, true, 'outer property returned');
same(shadowReader(), 99, 'body shadow retained after completion');

const shared = { value: 0 };
function* interleave(iterable) {
  for (shared.value of iterable) {
    yield () => shared.value;
    yield () => shared.value;
  }
  return shared.value;
}
const a = interleave([1, 2]);
const b = interleave([3, 4]);
const aFirst = a.next().value;
const bFirst = b.next().value;
same(aFirst(), 3, 'first capture observes other instance write');
same(bFirst(), 3, 'both instances share property target');
shared.value = 100;
same(a.next().value(), 100, 'first instance resume does not replay Set');
same(b.next().value(), 100, 'second instance resume does not replay Set');
same(a.next().value(), 2, 'next first entry writes once');
same(b.next().value(), 4, 'next second entry writes once');
shared.value = 200;
same(a.next().value(), 200, 'later first resume');
same(b.next().value(), 200, 'later second resume');
step(a.next(), 200, true, 'first instance completes');
step(b.next(), 200, true, 'second instance completes');
print('ok');
262;
