function check(condition, label) {
  if (!condition) throw new Error(label);
}
function step(iterator, input, value, done, label) {
  const result = iterator.next(input);
  check(result.value === value && result.done === done, label);
}
let calls = [];
function seen(label, value) { calls.push(label); return value; }
function* choose() {
  return (yield seen('selector', 'choose'))
    ? (yield seen('then-first', 'one'), yield seen('then-last', 'two'))
    : (yield seen('else-first', 'three'), yield seen('else-last', 'four'));
}
let iterator = choose();
step(iterator, undefined, 'choose', false, 'selector entry');
step(iterator, true, 'one', false, 'selected first');
gc();
step(iterator, 10, 'two', false, 'selected second');
step(iterator, 20, 20, true, 'selected result');
check(calls.join(',') === 'selector,then-first,then-last', 'complete lazy then');
calls = [];
iterator = choose();
step(iterator, undefined, 'choose', false, 'selector again');
step(iterator, false, 'three', false, 'other first');
step(iterator, 30, 'four', false, 'other second');
step(iterator, 40, 40, true, 'other result');
check(calls.join(',') === 'selector,else-first,else-last', 'complete lazy else');

function* lazy(left) {
  return left || (yield seen('right-first', 'right-one'), yield seen('right-last', 'right-two'));
}
const original = { marker: 262 };
calls = [];
iterator = lazy(original);
step(iterator, undefined, original, true, 'original skipped value');
check(calls.length === 0, 'skipped multiple yield arm');
iterator = lazy(0);
step(iterator, undefined, 'right-one', false, 'lazy first');
step(iterator, 1, 'right-two', false, 'lazy second');
step(iterator, original, original, true, 'whole lazy result');

function* delegated() {
  return (yield 'outer-choice') ? (true ? yield* [11, 12] : yield 99) : yield 100;
}
iterator = delegated();
step(iterator, undefined, 'outer-choice', false, 'nested selector');
step(iterator, true, 11, false, 'delegate first');
gc();
step(iterator, undefined, 12, false, 'delegate second');
step(iterator, undefined, undefined, true, 'delegate normal completion');

const thrown = {};
const returned = {};
function* abrupt() {
  try {
    return (yield 'pick') ? (yield 'first', yield 'last') : yield 'skipped';
  } catch (error) {
    check(error === thrown, 'caught whole marker');
    return (yield 'caught-choice') ? (yield 'caught-first', yield 'caught-last') : 0;
  } finally {
    yield 'finally';
  }
}
iterator = abrupt();
step(iterator, undefined, 'pick', false, 'throw selector');
step(iterator, true, 'first', false, 'throw selected');
let result = iterator.throw(thrown);
check(result.value === 'caught-choice' && !result.done, 'caught new selector');
step(iterator, true, 'caught-first', false, 'catch first');
gc();
step(iterator, 1, 'caught-last', false, 'catch last');
step(iterator, original, 'finally', false, 'normal pending finally');
step(iterator, undefined, original, true, 'normal pending whole value');
iterator = abrupt();
step(iterator, undefined, 'pick', false, 'return selector');
step(iterator, false, 'skipped', false, 'return selected');
result = iterator.return(returned);
check(result.value === 'finally' && !result.done, 'return yielding finally');
gc();
step(iterator, undefined, returned, true, 'pending whole Return');
print('generator-complete-regions:ok');
