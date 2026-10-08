function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'unobserved numeric hook'; }
const define = Object.defineProperty;

function numberEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const trace = [];
  const input = {get [Symbol.toPrimitive]() {
    trace.push('get');
    return function(hint) {
      check(this === input && hint === 'number' && arguments.length === 1, 'Number ToPrimitive input');
      trace.push('call'); kind = function() { return 7; };
      shape = {value: Symbol.for('number effect')}; elements[0] = function() { return 9; };
      return '12';
    };
  }};
  const result = Number(input);
  check(result === 12 && trace.join(',') === 'get,call' && kind() === 7 &&
    typeof shape.value === 'symbol' && elements[0]() === 9, 'Number invalidates captured facts');
  const boxedTrace = [];
  const boxed = new Number({valueOf() { boxedTrace.push('valueOf'); return 17; }});
  check(boxed.valueOf() === 17 && boxedTrace.join(',') === 'valueOf', 'Number Construct still converts input');
}
function bigintEffects() {
  let kind = 1, shape = {value: 1}, elements = [1];
  const input = {[Symbol.toPrimitive](hint) {
    check(hint === 'number', 'BigInt conversion hint'); kind = function() { return 7; };
    shape = {value: Symbol.for('bigint effect')}; elements[0] = function() { return 9; }; return '17';
  }};
  check(BigInt(input) === 17n && kind() === 7 && typeof shape.value === 'symbol' && elements[0]() === 9,
    'BigInt conversion invalidates captured facts');
  const trace = [];
  let bitsKind = 1, valueElements = [1];
  const bits = {valueOf() { trace.push('bits'); bitsKind = function() { return 7; }; return 3; }};
  const value = {valueOf() { trace.push('value'); valueElements[0] = function() { return 9; }; return 7n; }};
  check(BigInt.asIntN(bits, value) === -1n && trace.join(',') === 'bits,value' && bitsKind() === 7 &&
    valueElements[0]() === 9, 'asIntN converts bits before BigInt value');
  let unsignedShape = {value: 1};
  check(BigInt.asUintN(3, {valueOf() { unsignedShape = {value: Symbol.for('unsigned effect')}; return -1n; }}) === 7n &&
    typeof unsignedShape.value === 'symbol', 'asUintN observes value conversion');
  let zeroReads = 0;
  check(BigInt.asIntN(0, {valueOf() { zeroReads++; return 7n; }}) === 0n && zeroReads === 1,
    'zero bits does not skip the BigInt conversion');
}
function globalPredicateEffects() {
  let kind = 1, elements = [1];
  const finite = isFinite({valueOf() { kind = function() { return 7; }; return '12'; }});
  const nan = isNaN({valueOf() { elements[0] = function() { return 9; }; return 'not a number'; }});
  check(finite && nan && kind() === 7 && elements[0]() === 9, 'global predicates coerce unlike Number predicates');
}
function formatterEffects() {
  let fixedKind = 1, exponentialShape = {value: 1}, precisionElements = [1], radixKind = 1, bigRadixKind = 1;
  const fixed = (12.5).toFixed({valueOf() { fixedKind = function() { return 7; }; return 2; }});
  const exponential = (12).toExponential({valueOf() {
    exponentialShape = {value: Symbol.for('exponential effect')}; return 1;
  }});
  const precision = (12.5).toPrecision({valueOf() { precisionElements[0] = function() { return 9; }; return 4; }});
  const radix = (255).toString({valueOf() { radixKind = function() { return 7; }; return 16; }});
  const bigRadix = (255n).toString({valueOf() { bigRadixKind = function() { return 7; }; return 16; }});
  check(fixed === '12.50' && exponential === '1.2e+1' && precision === '12.50' && radix === 'ff' && bigRadix === 'ff' &&
    fixedKind() === 7 && typeof exponentialShape.value === 'symbol' && precisionElements[0]() === 9 &&
    radixKind() === 7 && bigRadixKind() === 7, 'formatting arguments invalidate their captured facts');
  let numberReads = 0, bigintReads = 0, numberKind = 1, bigintShape = {value: 1};
  const numberLocale = (12).toLocaleString('en', {get useGrouping() {
    numberReads++; numberKind = function() { return 7; }; return false;
  }});
  const bigintLocale = (12n).toLocaleString('en', {get useGrouping() {
    bigintReads++; bigintShape = {value: Symbol.for('locale effect')}; return false;
  }});
  check(numberLocale === '12' && bigintLocale === '12' && numberReads === 1 && bigintReads === 1 &&
    numberKind() === 7 && typeof bigintShape.value === 'symbol', 'intrinsic locale formatting observes options');
}
function noncoercingPartition() {
  let gets = 0;
  const input = new Proxy({}, {get() { gets++; throw 'type predicates do not Get'; }});
  check(!Number.isInteger(input) && !Number.isSafeInteger(input) && !Number.isFinite(input) && !Number.isNaN(input),
    'Number predicates use the input type only');
  const numberBox = new Number(7), bigintBox = Object(7n);
  numberBox[Symbol.toPrimitive] = poison; numberBox.toString = poison;
  bigintBox[Symbol.toPrimitive] = poison; bigintBox.toString = poison;
  check(numberBox.valueOf() === 7 && bigintBox.valueOf() === 7n, 'valueOf reads internal primitive slots');
  const random = Math.random(input);
  check(random >= 0 && random < 1 && Atomics.pause(0, input) === undefined && gets === 0,
    'random and pause ignore extra operands without coercion');
}
function acquiredAliasAndSpread() {
  const trace = [], savedNumber = Number;
  const input = {valueOf() { trace.push('coerce'); return 21; }};
  const receiver = {};
  const callable = new Proxy(savedNumber, {apply(target, rawThis, args) {
    trace.push('apply');
    check(rawThis === receiver && args.length === 3 && args[0] === input && args[1] === 23 && args[2] === 29,
      'numeric alias sees original receiver and full argv');
    return Reflect.apply(target, rawThis, args);
  }});
  define(receiver, 'saved', {configurable: true, get() { trace.push('callee'); return callable; }});
  function first() {
    trace.push('first'); define(receiver, 'saved', {value: poison, configurable: true}); return input;
  }
  let position = 0, closes = 0;
  const iterator = {};
  const next = new Proxy(function() {}, {apply(target, rawThis, args) {
    check(rawThis === iterator && args.length === 0, 'spread cached next invocation');
    trace.push('next' + position); define(iterator, 'next', {value: poison, configurable: true});
    if (position++ === 0) return {done: false, value: 23};
    return {done: true, get value() { throw 'terminal spread value'; }};
  }});
  define(iterator, 'next', {configurable: true, get() { trace.push('next.get'); return next; }});
  define(iterator, 'return', {get() { closes++; throw 'normal spread does not close'; }});
  const source = {get [Symbol.iterator]() {
    trace.push('iterator.get'); return function() { trace.push('iterator.call'); return iterator; };
  }};
  const result = receiver.saved(first(), ...source, (trace.push('last'), 29));
  check(result === 21 && closes === 0 &&
    trace.join(',') === 'callee,first,iterator.get,iterator.call,next.get,next0,next1,last,apply,coerce',
    'callee acquisition and full argument evaluation precede numeric coercion');
}
numberEffects(); bigintEffects(); globalPredicateEffects(); formatterEffects();
noncoercingPartition(); acquiredAliasAndSpread();
print('numeric-conversions:ok');
262;
