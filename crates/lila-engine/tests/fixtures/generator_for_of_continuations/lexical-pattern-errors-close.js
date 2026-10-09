function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
function step(result, value, done, label) {
  same(result.value, value, label + ' value');
  same(result.done, done, label + ' done');
}
const foreign = __lilaCreateRealm();
const marker = new foreign.global.Object();
const closeError = new foreign.global.Object();
const nativeTypeError = TypeError.prototype;
const nativeReferenceError = ReferenceError.prototype;
function source(value, log, closeThrows) {
  const iterator = {
    next() { log.push('next'); return { value, done: false }; },
    get return() {
      log.push('outer:get:return');
      return function () {
        same(this, iterator, 'outer close receiver');
        same(arguments.length, 0, 'outer close arguments');
        log.push('outer:close');
        if (closeThrows) throw closeError;
        return {};
      };
    }
  };
  return { [Symbol.iterator]() { log.push('open'); return iterator; } };
}
function* objectGetter(iterable, log) {
  try { for (const { value } of iterable) { log.push('body'); yield value; } }
  finally { log.push('finally'); yield 'cleanup'; log.push('after'); }
}
function* objectDefault(iterable, log) {
  function fail() { log.push('default'); throw marker; }
  try { for (let { value = fail() } of iterable) { log.push('body'); yield value; } }
  finally { log.push('finally'); yield 'cleanup'; log.push('after'); }
}
function* objectKey(iterable, log) {
  function fail() { log.push('key'); throw marker; }
  try { for (const { [fail()]: value } of iterable) { log.push('body'); yield value; } }
  finally { log.push('finally'); yield 'cleanup'; log.push('after'); }
}
function finishFailure(consumer, log, prefix, label) {
  step(consumer.next(), 'cleanup', false, label + ' enters outer finally');
  same(log.join(','), prefix + ',outer:get:return,outer:close,finally', label + ' closes before finally');
  let caught;
  let finalized = 0;
  try { consumer.next(); } catch (error) { caught = error; } finally { ++finalized; }
  same(caught, marker, label + ' original marker survives close Throw');
  same(finalized, 1, label + ' caller finally once');
  same(log.join(','), prefix + ',outer:get:return,outer:close,finally,after', label + ' full chronology');
  step(consumer.next(), undefined, true, label + ' completed');
}
let log = [];
const getterValue = { get value() { log.push('get'); throw marker; } };
finishFailure(objectGetter(source(getterValue, log, true), log), log, 'open,next,get', 'Get abrupt');
log = [];
const undefinedValue = { get value() { log.push('get'); return undefined; } };
finishFailure(objectDefault(source(undefinedValue, log, true), log), log, 'open,next,get,default', 'default abrupt');
log = [];
finishFailure(objectKey(source({}, log, true), log), log, 'open,next,key', 'computed key abrupt');

log = [];
const innerIterator = {
  next() { log.push('inner:next'); return { value: undefined, done: false }; },
  get return() {
    log.push('inner:get:return');
    return function () { log.push('inner:close'); throw closeError; };
  }
};
const inner = { [Symbol.iterator]() { log.push('inner:open'); return innerIterator; } };
function* nested(iterable, trace) {
  function fail() { trace.push('default'); throw marker; }
  try { for (const [value = fail()] of iterable) { trace.push('body'); yield value; } }
  finally { trace.push('finally'); yield 'cleanup'; trace.push('after'); }
}
finishFailure(nested(source(inner, log, true), log), log,
  'open,next,inner:open,inner:next,default,inner:get:return,inner:close', 'inner BindingInitialization abrupt');

let emptyClosed = 0;
const emptyInnerIterator = {
  next() { throw 'empty array binding cannot step'; },
  return() { ++emptyClosed; return {}; }
};
function* empty(iterable) { for (const [] of iterable) { yield 'body'; } }
const emptyConsumer = empty([{ [Symbol.iterator]() { return emptyInnerIterator; } }]);
step(emptyConsumer.next(), 'body', false, 'empty array still acquires and closes inner iterator');
same(emptyClosed, 1, 'empty array close once');
step(emptyConsumer.next(), undefined, true, 'empty binding exhausts');

log = [];
const nullConsumer = objectGetter(source(null, log, false), log);
step(nullConsumer.next(), 'cleanup', false, 'null object pattern closes before cleanup');
let nullError;
try { nullConsumer.next(); } catch (error) { nullError = error; }
same(Object.getPrototypeOf(nullError), nativeTypeError, 'null ObjectBindingPattern native TypeError');
same(log.join(','), 'open,next,outer:get:return,outer:close,finally,after', 'null never enters body');

function* tdzDefault(iterable) { for (let [first = later, later] of iterable) { yield first; } }
log = [];
let tdzError;
try { tdzDefault(source([undefined, 2], log, false)).next(); } catch (error) { tdzError = error; }
same(Object.getPrototypeOf(tdzError), nativeReferenceError, 'later pattern cell remains TDZ');
same(log.join(','), 'open,next,outer:get:return,outer:close', 'TDZ initialization failure closes outer iterator');
let opened = 0;
function build(read) { read(); return { [Symbol.iterator]() { ++opened; return {}; } }; }
function* tdzIterable() { for (const [value] of build(() => value)) { yield value; } }
let iterableError;
try { tdzIterable().next(); } catch (error) { iterableError = error; }
same(Object.getPrototypeOf(iterableError), nativeReferenceError, 'head names are TDZ during iterable evaluation');
same(opened, 0, 'iterable abrupt occurs before iterator acquisition');

for (const phase of ['next', 'done', 'value']) {
  const trace = [];
  const iterator = {
    next() {
      trace.push('next');
      if (phase === 'next') throw marker;
      return {
        get done() { trace.push('done'); if (phase === 'done') throw marker; return false; },
        get value() { trace.push('value'); throw marker; }
      };
    },
    return() { trace.push('close'); return {}; }
  };
  const consumer = objectGetter({ [Symbol.iterator]() { return iterator; } }, trace);
  step(consumer.next(), 'cleanup', false, phase + ' step abrupt enters outer finally');
  let error;
  try { consumer.next(); } catch (value) { error = value; }
  same(error, marker, phase + ' original step error');
  same(trace.join(','), (phase === 'next' ? 'next' : phase === 'done' ? 'next,done' : 'next,done,value') + ',finally,after', phase + ' no IteratorClose or head work');
}
print('ok');
262;
