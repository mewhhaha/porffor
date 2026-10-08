const trace = [];
let skippedCalls = 0;
let skippedJobs = 0;
function skipped() {
  skippedCalls++;
  Promise.resolve().then(() => { skippedJobs++; });
  throw 'skipped yield operand';
}
function check(result, value, done) {
  if (!Object.is(result.value, value) || result.done !== done) throw 'iterator result';
}
let selectorReads = 0;
let flag = true;
const selector = {get value() { selectorReads++; trace.push('selector'); return flag; }};
function token(arm) { trace.push(arm); return arm + '-token'; }
function* choose() {
  const value = selector.value
    ? (yield token('then'))
    : (yield token('else'));
  trace.push('chosen');
  return value;
}
const first = choose();
check(first.next('ignored'), 'then-token', false);
flag = false;
check(first.next(17), 17, true);
check(first.next(), undefined, true);
const second = choose();
check(second.next(), 'else-token', false);
flag = true;
check(second.next(undefined), undefined, true);
if (selectorReads !== 2 || trace.join(',') !== 'selector,then,chosen,selector,else,chosen') throw 'selector replay or selected arm';

function* andValue(value) { const result = value && (yield skipped()); return result; }
function* orValue(value) { let result = value || (yield skipped()); return result; }
function* coalesceValue(value) { var result = value ?? (yield skipped()); return result; }
const htmlDda = __lilaCreateHTMLDDA();
for (const value of [-0, false, '', NaN, 0n, htmlDda]) check(andValue(value).next(), value, true);
const object = {};
const symbol = Symbol('retained');
for (const value of [object, symbol, 3, 'saved']) check(orValue(value).next(), value, true);
for (const value of [-0, false, '', NaN, 0n, htmlDda, object, symbol]) check(coalesceValue(value).next(), value, true);
function* selectedAnd() { const value = true && (yield 'and'); return value; }
function* selectedOr() { let value = false || (yield 'or'); return value; }
function* selectedNull() { var value = null ?? (yield 'null'); return value; }
for (const generator of [selectedAnd, selectedOr, selectedNull]) {
  const iterator = generator();
  const before = iterator.next();
  if (before.done !== false) throw 'selected arm did not yield';
  check(iterator.next(-0), -0, true);
}
function* sequential() {
  const a = true ? (yield 'one') : 0;
  var b = null ?? (yield 'two');
  return a + b;
}
const sequence = sequential();
check(sequence.next(), 'one', false);
check(sequence.next(10), 'two', false);
check(sequence.next(20), 30, true);

let methodReads = 0;
const target = {get method() {
  methodReads++;
  return function(value) { if (this !== target) throw 'lost call receiver'; return value + 1; };
}};
function* invoke() { return target.method(false || (yield 'argument')); }
const call = invoke();
check(call.next(), 'argument', false);
Object.defineProperty(target, 'method', {value() { throw 'replacement callee'; }, configurable: true});
check(call.next(40), 41, true);
if (methodReads !== 1 || skippedCalls !== 0) throw 'retained callee or skipped operand';
Promise.resolve().then(() => {
  if (skippedJobs !== 0) throw 'skipped branch scheduled work';
  print('generator-values:ok');
});
