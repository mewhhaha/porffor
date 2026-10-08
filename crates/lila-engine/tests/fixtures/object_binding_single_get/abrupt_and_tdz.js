function check(condition, name) { if (!condition) throw name; }
const foreign = $262.createRealm().global;
const marker = new foreign.Error('object binding original marker');
const markerPrototype = foreign.Error.prototype;
const prototype = Object.getPrototypeOf;
const trace = [];
let prior = 0, published = false;
function fallback() { trace.push('default'); prior = 7; throw marker; }
const source = {get first() { trace.push('first'); return undefined; },
  get second() { trace.push('unreached.second'); return 9; }};
let caught, finalized = 0;
try {
  const {first = fallback(), second} = source;
  published = true;
} catch (error) { caught = error; trace.push('catch'); }
finally { finalized++; trace.push('finally'); }
check(caught === marker && prototype(caught) === markerPrototype && prior === 7 && !published && finalized === 1 &&
  trace.join(',') === 'first,default,catch,finally', 'default throw preserves original identity, prior write and sibling cutoff');

let getterHits = 0, unusedDefault = 0;
const badGetter = {get item() { getterHits++; prior = 11; throw marker; }};
try { let {item = (unusedDefault++, 9)} = badGetter; throw 'unreached publication'; }
catch (error) { check(error === marker, 'getter original abrupt identity'); }
check(getterHits === 1 && unusedDefault === 0 && prior === 11, 'getter abrupt never evaluates default');

let native, laterGets = 0;
try { let {first = second, second} = {get second() { laterGets++; return 9; }}; }
catch (error) { native = error; }
check(native !== undefined && prototype(native) === ReferenceError.prototype && laterGets === 0,
  'forward sibling default retains TDZ and cuts off later Get');
{
  let gets = 0;
  const source = {get item() { gets++; return 17; }};
  const {item = item} = source;
  check(item === 17 && gets === 1, 'nonundefined Get skips an otherwise abrupt self-default');
}
for (const invalid of [null, undefined]) {
  let error;
  try { let {} = invalid; } catch (caught) { error = caught; }
  check(error !== undefined && prototype(error) === TypeError.prototype, 'empty object pattern still rejects nullish source');
}

const headTrace = [];
let nexts = 0, closes = 0, bodies = 0;
const badHead = {get item() { headTrace.push('get'); prior = 19; throw marker; }};
const iterator = {next() { nexts++; headTrace.push('next'); return {done: false, value: badHead}; },
  return() { closes++; headTrace.push('close'); throw 'close must not replace original throw'; }};
const iterable = {[Symbol.iterator]() { return iterator; }};
let headError;
try { for (const {item = (headTrace.push('unreached.default'), 3)} of iterable) { bodies++; } }
catch (error) { headError = error; headTrace.push('catch'); }
finally { headTrace.push('finally'); }
check(headError === marker && prior === 19 && nexts === 1 && closes === 1 && bodies === 0 &&
  headTrace.join(',') === 'next,get,close,catch,finally', 'head getter failure closes once before outer clauses and preserves throw');
print('object-binding-abrupt:ok');
262;
