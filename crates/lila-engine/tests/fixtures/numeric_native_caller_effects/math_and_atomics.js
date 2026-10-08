function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'unobserved numeric phase'; }
const define = Object.defineProperty;

function mathCoercions() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const trace = [];
  const base = {valueOf() { trace.push('base'); kind = function() { return 7; }; return 2; }};
  const exponent = {valueOf() { trace.push('exponent'); shape = {value: Symbol.for('pow effect')}; return 3; }};
  check(Math.pow(base, exponent) === 8 && trace.join(',') === 'base,exponent' && kind() === 7 &&
    typeof shape.value === 'symbol', 'binary Math converts left then right and invalidates captured facts');
  check(Math.abs({valueOf() { elements[0] = function() { return 9; }; return -3; }}) === 3 && elements[0]() === 9,
    'unary Math observes ToNumber');
  let imulKind = 1;
  const imulTrace = [];
  check(Math.imul({valueOf() { imulTrace.push('left'); imulKind = function() { return 7; }; return 4294967295; }},
    {valueOf() { imulTrace.push('right'); return 5; }}) === -5 && imulKind() === 7 &&
    imulTrace.join(',') === 'left,right', 'imul converts operands before Uint32 multiplication');
  let roundingKind = 1;
  check(Math.f16round({valueOf() { roundingKind = function() { return 7; }; return 1.5; }}) === 1.5 && roundingKind() === 7,
    'half-precision rounding is also coercing');

  const variadic = [];
  let variadicKind = 1;
  function evaluated(name, value) { variadic.push('arg.' + name); return value; }
  const low = {valueOf() { variadic.push('coerce.low'); variadicKind = function() { return 7; }; return 2; }};
  const high = {valueOf() { variadic.push('coerce.high'); return 7; }};
  const maximum = Math.max(evaluated('low', low), evaluated('high', high));
  check(maximum === 7 && variadicKind() === 7 && variadic.join(',') === 'arg.low,arg.high,coerce.low,coerce.high',
    'variadic arguments evaluate completely before ordered ToNumber');
  const nanTrace = [];
  const minimum = Math.min(NaN, {valueOf() { nanTrace.push('after.nan'); return 3; }});
  const infinite = Math.hypot(Infinity, {valueOf() { nanTrace.push('after.infinity'); return 4; }});
  check(Number.isNaN(minimum) && infinite === Infinity && nanTrace.join(',') === 'after.nan,after.infinity',
    'NaN and Infinity do not skip later operand coercions');

  const savedAbs = Math.abs, aliasTrace = [], receiver = {};
  const input = {valueOf() { aliasTrace.push('coerce'); return -4; }};
  const alias = new Proxy(savedAbs, {apply(target, rawThis, args) {
    aliasTrace.push('apply'); check(rawThis === receiver && args.length === 3 && args[0] === input &&
      args[1] === 31 && args[2] === 37, 'Math alias raw receiver and ignored operands');
    return Reflect.apply(target, rawThis, args);
  }});
  define(receiver, 'saved', {configurable: true, get() { aliasTrace.push('callee'); return alias; }});
  const aliasResult = receiver.saved((aliasTrace.push('first'), input),
    (define(receiver, 'saved', {value: poison, configurable: true}), aliasTrace.push('extra1'), 31),
    (aliasTrace.push('extra2'), 37));
  check(aliasResult === 4 && aliasTrace.join(',') === 'callee,first,extra1,extra2,apply,coerce',
    'Math alias keeps acquired callee until all ignored arguments finish');
}
function preciseIteration() {
  const trace = [];
  let kind = 1, shape = {value: 1}, elements = [1], position = 0, closes = 0;
  const iterator = {};
  const next = new Proxy(function() {}, {apply(target, rawThis, args) {
    check(rawThis === iterator && args.length === 0, 'sumPrecise cached next receiver and argc');
    const index = position++; trace.push('next' + index);
    define(iterator, 'next', {value: poison, configurable: true});
    return {get done() { trace.push('done' + index); return index === 2; },
      get value() { trace.push('value' + index); if (index === 2) throw 'terminal value unread';
        elements[0] = function() { return 9; }; return index + 1; }};
  }});
  define(iterator, 'next', {configurable: true, get() {
    trace.push('next.get'); shape = {value: Symbol.for('iterator effect')}; return next;
  }});
  define(iterator, 'return', {get() { closes++; throw 'normal sumPrecise no close'; }});
  const iterable = {get [Symbol.iterator]() {
    trace.push('iterator.get'); kind = function() { return 7; };
    return new Proxy(function() {}, {apply(target, rawThis, args) {
      check(rawThis === iterable && args.length === 0, 'sumPrecise iterator method invocation');
      trace.push('iterator.apply'); return iterator;
    }});
  }};
  const result = Math.sumPrecise(iterable);
  check(result === 3 && kind() === 7 && typeof shape.value === 'symbol' && elements[0]() === 9 && closes === 0 &&
    trace.join(',') === 'iterator.get,iterator.apply,next.get,next0,done0,value0,next1,done1,value1,next2,done2',
    'sumPrecise observes iteration rather than coercing its numeric values');
}
function atomicPreparation() {
  const buffer = new SharedArrayBuffer(8), view = new Int32Array(buffer);
  view[0] = 3;
  let kind = 1, shape = {value: 1}, elements = [1];
  const trace = [];
  const index = {valueOf() { trace.push('index'); kind = function() { return 7; }; return 0; }};
  const expected = {valueOf() { trace.push('expected'); shape = {value: Symbol.for('atomic effect')}; return 3; }};
  const replacement = {valueOf() { trace.push('replacement'); elements[0] = function() { return 9; }; return 7; }};
  const before = Atomics.compareExchange(view, index, expected, replacement);
  check(before === 3 && view[0] === 7 && view.buffer === buffer && kind() === 7 &&
    typeof shape.value === 'symbol' && elements[0]() === 9 && trace.join(',') === 'index,expected,replacement',
    'compareExchange converts index, expected and replacement before the atomic operation');
  let storeKind = 1, loadKind = 1;
  check(Atomics.store(view, 0, {valueOf() { storeKind = function() { return 7; }; return 9; }}) === 9 && storeKind() === 7,
    'store value conversion invalidates facts');
  check(Atomics.load(view, {valueOf() { loadKind = function() { return 7; }; return 0; }}) === 9 && loadKind() === 7,
    'load index conversion invalidates facts');
  const bigBuffer = new SharedArrayBuffer(8), bigView = new BigInt64Array(bigBuffer);
  let bigKind = 1;
  check(Atomics.store(bigView, 0, {valueOf() { bigKind = function() { return 7; }; return 17n; }}) === 17n &&
    bigView[0] === 17n && bigView.buffer === bigBuffer && bigKind() === 7, 'BigInt atomic values use ToBigInt');

  const notifyTrace = [];
  let notifyKind = 1;
  check(Atomics.notify(view, {valueOf() { notifyTrace.push('index'); return 0; }},
    {valueOf() { notifyTrace.push('count'); notifyKind = function() { return 7; }; return 0; }}) === 0 &&
    notifyKind() === 7 && notifyTrace.join(',') === 'index,count', 'notify converts count with no waiters');
  const waitTrace = [];
  let timeoutKind = 1;
  check(Atomics.wait(view, {valueOf() { waitTrace.push('index'); return 0; }},
    {valueOf() { waitTrace.push('expected'); return 9; }},
    {valueOf() { waitTrace.push('timeout'); timeoutKind = function() { return 7; }; return 0; }}) === 'timed-out' &&
    timeoutKind() === 7 && waitTrace.join(',') === 'index,expected,timeout', 'wait observes hooks with zero timeout');
  const asyncTrace = [];
  let asyncKind = 1;
  const timed = Atomics.waitAsync(view, {valueOf() { asyncTrace.push('index'); return 0; }},
    {valueOf() { asyncTrace.push('expected'); return 9; }},
    {valueOf() { asyncTrace.push('timeout'); asyncKind = function() { return 7; }; return 0; }});
  const unequal = Atomics.waitAsync(view, 0, 10, 0);
  check(!timed.async && timed.value === 'timed-out' && !unequal.async && unequal.value === 'not-equal' &&
    asyncKind() === 7 && asyncTrace.join(',') === 'index,expected,timeout', 'waitAsync zero timeout has no async waiter lifecycle');
  let lockKind = 1;
  check(Atomics.isLockFree({valueOf() { lockKind = function() { return 7; }; return 4; }}) && lockKind() === 7,
    'isLockFree coerces the requested size');
}
mathCoercions(); preciseIteration(); atomicPreparation();
print('numeric-math-atomics:ok');
262;
