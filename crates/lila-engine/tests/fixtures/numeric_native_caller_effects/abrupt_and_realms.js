var $262 = { createRealm: __lilaCreateRealm };
function check(condition, name) { if (!condition) throw name; }
function poison() { throw 'public numeric globals or unreachable phase'; }
const define = Object.defineProperty;
const descriptor = Object.getOwnPropertyDescriptor;
const getPrototypeOf = Object.getPrototypeOf;
const apply = Reflect.apply;
const construct = Reflect.construct;
const LocalNumber = Number, LocalBigInt = BigInt, LocalMath = Math, LocalAtomics = Atomics;
const localFinite = isFinite, localNaN = isNaN;
const localFixed = Number.prototype.toFixed, localNumberString = Number.prototype.toString;
const localBigString = BigInt.prototype.toString, localLocale = Number.prototype.toLocaleString;
const localType = TypeError.prototype, localRange = RangeError.prototype;
const foreign = $262.createRealm().global;
const ForeignNumber = foreign.Number, ForeignBigInt = foreign.BigInt;
const ForeignMath = foreign.Math, ForeignAtomics = foreign.Atomics;
const ForeignObject = foreign.Object;
const foreignFinite = foreign.isFinite, foreignNaN = foreign.isNaN;
const foreignFixed = ForeignNumber.prototype.toFixed, foreignNumberString = ForeignNumber.prototype.toString;
const foreignBigString = ForeignBigInt.prototype.toString, foreignLocale = ForeignNumber.prototype.toLocaleString;
const foreignType = foreign.TypeError.prototype, foreignRange = foreign.RangeError.prototype;
const marker = new foreign.Error('numeric original marker');
const foreignError = foreign.Error.prototype;
const originalGlobals = [];
for (const realm of [globalThis, foreign]) {
  for (const key of ['Number', 'BigInt', 'Math', 'Atomics', 'isFinite', 'isNaN', 'TypeError', 'RangeError']) {
    originalGlobals.push([realm, key, descriptor(realm, key)]);
  }
}
function expectMarker(action, trace, prefix) {
  let result = 'unpublished', caught;
  try { const value = action(); result = value; }
  catch (error) { caught = error; trace.push('catch'); }
  finally { trace.push('finally'); }
  check(caught === marker && getPrototypeOf(caught) === foreignError && result === 'unpublished' &&
    trace.join(',') === prefix + ',catch,finally', 'original foreign marker, phase cutoff and finally');
}
function expectNative(action, prototype) {
  let result = 'unpublished', caught, finalized = 0;
  try { const value = action(); result = value; }
  catch (error) { caught = error; }
  finally { finalized++; }
  check(caught !== undefined && getPrototypeOf(caught) === prototype && result === 'unpublished' && finalized === 1,
    'native error belongs to the called numeric function Realm');
}

try {
  const argumentTrace = [];
  const uncoerced = {valueOf() { argumentTrace.push('unreached.coerce'); return 7; }};
  expectMarker(() => LocalNumber((argumentTrace.push('first'), uncoerced),
    (argumentTrace.push('ignored'), (() => { throw marker; })())), argumentTrace, 'first,ignored');

  let numericKind = 1;
  const numericTrace = [];
  expectMarker(() => LocalNumber({get [Symbol.toPrimitive]() {
    numericTrace.push('get'); numericKind = function() { return 7; }; throw marker;
  }}), numericTrace, 'get');
  check(numericKind() === 7, 'ToPrimitive getter prior write survives throw');
  const primitiveTrace = [];
  let primitiveShape = {value: 1};
  const primitiveInput = {[Symbol.toPrimitive]: new Proxy(function() {}, {apply(target, rawThis, args) {
    check(rawThis === primitiveInput && args.length === 1 && args[0] === 'number', 'conversion Proxy apply');
    primitiveTrace.push('apply'); primitiveShape = {value: Symbol.for('primitive effect')}; throw marker;
  }})};
  expectMarker(() => ForeignBigInt(primitiveInput), primitiveTrace, 'apply');
  check(typeof primitiveShape.value === 'symbol', 'BigInt primitive hook prior shape write survives');

  const bitsTrace = [];
  let bitsElements = [1];
  const unreadBigint = {valueOf() { bitsTrace.push('unreached.value'); return 7n; }};
  expectMarker(() => LocalBigInt.asIntN({valueOf() {
    bitsTrace.push('bits'); bitsElements[0] = function() { return 9; }; throw marker;
  }}, unreadBigint), bitsTrace, 'bits');
  check(bitsElements[0]() === 9, 'bits failure skips BigInt input but retains effects');

  const mathTrace = [];
  const unreadExponent = {valueOf() { mathTrace.push('unreached.exponent'); return 3; }};
  expectMarker(() => ForeignMath.pow({valueOf() { mathTrace.push('base'); throw marker; }}, unreadExponent),
    mathTrace, 'base');
  const localeTrace = [];
  expectMarker(() => apply(localLocale, 7, ['en', {get useGrouping() {
    localeTrace.push('options'); throw marker;
  }}]), localeTrace, 'options');

  const view = new Int32Array(new SharedArrayBuffer(4));
  const foreignView = new foreign.Int32Array(new foreign.SharedArrayBuffer(4));
  view[0] = 5;
  const atomicTrace = [];
  const unreadAtomicValue = {valueOf() { atomicTrace.push('unreached.value'); return 9; }};
  expectMarker(() => LocalAtomics.store(view, {valueOf() { atomicTrace.push('index'); throw marker; }}, unreadAtomicValue),
    atomicTrace, 'index');
  check(view[0] === 5, 'atomic index failure makes no write');
  const waitTrace = [];
  let waitKind = 1;
  expectMarker(() => ForeignAtomics.wait(view, {valueOf() { waitTrace.push('index'); return 0; }},
    {valueOf() { waitTrace.push('expected'); return 5; }},
    {valueOf() { waitTrace.push('timeout'); waitKind = function() { return 7; }; throw marker; }}),
    waitTrace, 'index,expected,timeout');
  check(waitKind() === 7 && view[0] === 5, 'timeout abrupt completion precedes any wait lifecycle');

  let protocolCloses = 0;
  for (let stage = 0; stage < 3; stage++) {
    const trace = [];
    const iterator = {get next() {
      trace.push('next.get');
      return function() {
        trace.push('next'); if (stage === 0) throw marker;
        return {get done() { trace.push('done'); if (stage === 1) throw marker; return false; },
          get value() { trace.push('value'); throw marker; }};
      };
    }, get return() { protocolCloses++; throw 'protocol abrupt must not close'; }};
    const iterable = {get [Symbol.iterator]() {
      trace.push('iterator.get'); return function() { trace.push('iterator.call'); return iterator; };
    }};
    const prefix = stage === 0 ? 'iterator.get,iterator.call,next.get,next' : stage === 1 ?
      'iterator.get,iterator.call,next.get,next,done' : 'iterator.get,iterator.call,next.get,next,done,value';
    expectMarker(() => LocalMath.sumPrecise(iterable), trace, prefix);
  }
  check(protocolCloses === 0, 'sumPrecise protocol abrupt completions never Get return');

  for (const row of originalGlobals) define(row[0], row[1], {value: poison, configurable: true, writable: true});
  // Each direction borrows saved callees, while native errors retain the callee's Realm.
  for (let direction = 0; direction < 2; direction++) {
    const number = direction === 0 ? ForeignNumber : LocalNumber;
    const bigint = direction === 0 ? ForeignBigInt : LocalBigInt;
    const math = direction === 0 ? ForeignMath : LocalMath;
    const atomics = direction === 0 ? ForeignAtomics : LocalAtomics;
    const finite = direction === 0 ? foreignFinite : localFinite;
    const nan = direction === 0 ? foreignNaN : localNaN;
    const fixed = direction === 0 ? foreignFixed : localFixed;
    const numberString = direction === 0 ? foreignNumberString : localNumberString;
    const bigString = direction === 0 ? foreignBigString : localBigString;
    const locale = direction === 0 ? foreignLocale : localLocale;
    const type = direction === 0 ? foreignType : localType;
    const range = direction === 0 ? foreignRange : localRange;
    const numericReceiver = direction === 0 ? new LocalNumber(7) : new ForeignNumber(7);
    const bigintReceiver = direction === 0 ? Object(7n) : new ForeignObject(7n);
    const operationView = direction === 0 ? view : foreignView;
    let conversions = 0;
    const input = direction === 0 ? {} : new ForeignObject();
    input.valueOf = function() { conversions++; return 7; };
    check(number(input) === 7 && bigint(input) === 7n && conversions === 2, 'saved numeric constructors survive global poison');
    const symbol = Symbol('numeric native error');
    expectNative(() => number(symbol), type);
    expectNative(() => bigint(undefined), type);
    expectNative(() => bigint(1.5), range);
    expectNative(() => finite(symbol), type);
    expectNative(() => nan(symbol), type);
    expectNative(() => math.sqrt(symbol), type);
    expectNative(() => bigint.asUintN(-1, 7n), range);
    expectNative(() => apply(fixed, numericReceiver, [-1]), range);
    expectNative(() => apply(numberString, numericReceiver, [1]), range);
    expectNative(() => apply(bigString, bigintReceiver, [37]), range);
    const optionsTrace = [];
    let optionsKind = 1;
    expectMarker(() => apply(locale, numericReceiver, ['en', {get useGrouping() {
      optionsTrace.push('options'); optionsKind = function() { return 7; }; throw marker;
    }}]), optionsTrace, 'options');
    check(optionsKind() === 7, 'borrowed locale options preserve original marker and prior effects');
    let digitReads = 0;
    const digits = {valueOf() { digitReads++; throw marker; }};
    expectNative(() => apply(fixed, {}, [digits]), type);
    expectNative(() => apply(bigString, {}, [digits]), type);
    check(digitReads === 0, 'Number and BigInt brands precede formatter argument conversion');
    let constructorReads = 0;
    const bigintInput = {valueOf() { constructorReads++; throw marker; }};
    expectNative(() => construct(bigint, [bigintInput]), type);
    check(constructorReads === 0, 'BigInt rejects Construct before input coercion');
    let atomicReads = 0;
    const atomicIndex = {valueOf() { atomicReads++; throw marker; }};
    expectNative(() => atomics.load({}, atomicIndex), type);
    check(atomicReads === 0, 'atomic brand validation precedes index coercion');
    expectNative(() => atomics.load(operationView, -1), range);
    expectNative(() => atomics.store(operationView, 0, symbol), type);
    const closeTrace = [];
    let elementCoercions = 0;
    const nonNumber = {valueOf() { elementCoercions++; return 1; }};
    const rejectedIterator = {next() { closeTrace.push('next'); return {done: false, value: nonNumber}; },
      get return() { closeTrace.push('return.get'); throw marker; }};
    const rejectedIterable = {[Symbol.iterator]() { return rejectedIterator; }};
    expectNative(() => math.sumPrecise(rejectedIterable), type);
    check(elementCoercions === 0 && closeTrace.join(',') === 'next,return.get',
      'sumPrecise rejects non-Number without coercion and preserves its TypeError during close');
  }
} finally {
  for (const row of originalGlobals) {
    if (row[2] === undefined) delete row[0][row[1]];
    else define(row[0], row[1], row[2]);
  }
}
print('numeric-abrupt-realms:ok');
262;
