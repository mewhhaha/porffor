function check(value, label) { if (!value) throw new Error(label); }
function step(iterator, input, expected, done, label) {
  const result = iterator.next(input);
  check(result.value === expected && result.done === done, label);
}
function* lexical() {
  const {selected = 7} = yield (() => selected);
  return selected;
}
let iterator = lexical();
let result = iterator.next();
const probe = result.value;
check(!result.done && typeof probe === 'function', 'escaped lexical probe');
let tdz = false;
try { probe(); } catch (error) { tdz = error instanceof ReferenceError; }
check(tdz, 'same source binding stays in TDZ during suspension');
gc();
step(iterator, {}, 7, true, 'default initializes resumed source binding');
check(probe() === 7, 'escaped closure sees initialized cell');

let log = [];
function key() { log.push('key'); return 'selected'; }
function fallback() { log.push('default'); return 23; }
function* objectPattern() {
  const {[key()]: selected = fallback(), ...rest} = (yield 'first', yield 'source');
  return [selected, rest];
}
const source = { get selected() { log.push('get'); return undefined; }, other: 29 };
iterator = objectPattern();
step(iterator, undefined, 'first', false, 'complete initializer first');
step(iterator, 1, 'source', false, 'complete initializer second');
check(log.length === 0, 'pattern effects await Normal initializer result');
gc();
result = iterator.next(source);
check(result.done && result.value[0] === 23 && result.value[1].other === 29, 'default and rest');
check(log.join(',') === 'key,get,default', 'one Get and source order');

let nextCalls = 0;
let closeCalls = 0;
const iterable = {
  [Symbol.iterator]() {
    return {
      next() { nextCalls++; return {value: 31, done: false}; },
      return() { closeCalls++; return {}; }
    };
  }
};
function* arrayPattern() { let [first] = yield 'array'; return first; }
iterator = arrayPattern();
step(iterator, undefined, 'array', false, 'array initializer');
check(nextCalls === 0 && closeCalls === 0, 'iterator not acquired before resume');
gc();
step(iterator, iterable, 31, true, 'binding iterator value');
check(nextCalls === 1 && closeCalls === 1, 'binding IteratorClose exactly once');

function* variable() {
  const before = () => first;
  var [first] = yield before;
  return before;
}
iterator = variable();
result = iterator.next();
const varProbe = result.value;
check(varProbe() === undefined, 'var hoisting has no TDZ');
gc();
result = iterator.next([37]);
check(result.done && result.value === varProbe && varProbe() === 37, 'var source cell survives resume');

const thrown = {};
const returned = {};
let observedGet = 0;
const untouched = { get selected() { observedGet++; return 41; } };
function* abrupt() {
  try { const {selected} = yield untouched; return selected; }
  finally { yield 'finally'; }
}
iterator = abrupt();
step(iterator, undefined, untouched, false, 'throw initializer');
result = iterator.throw(thrown);
check(result.value === 'finally' && !result.done, 'Throw passes through yielding finally');
gc();
let caught;
try { iterator.next(); } catch (error) { caught = error; }
check(caught === thrown && observedGet === 0, 'Throw skips binding initialization and keeps whole value');
iterator = abrupt();
step(iterator, undefined, untouched, false, 'return initializer');
result = iterator.return(returned);
check(result.value === 'finally' && !result.done, 'Return passes through yielding finally');
gc();
step(iterator, undefined, returned, true, 'pending whole Return');
check(observedGet === 0, 'Return skips binding initialization');
print('generator-pattern-initializers:ok');
