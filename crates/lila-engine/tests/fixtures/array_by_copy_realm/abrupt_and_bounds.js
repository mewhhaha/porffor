const other = __lilaCreateRealm().global;
const LocalArray = Array;
const ForeignArray = other.Array;
const localMethods = [LocalArray.prototype.toReversed, LocalArray.prototype.with, LocalArray.prototype.toSpliced];
const foreignMethods = [ForeignArray.prototype.toReversed, ForeignArray.prototype.with, ForeignArray.prototype.toSpliced];
const localRangePrototype = RangeError.prototype;
const foreignRangePrototype = other.RangeError.prototype;
const localTypePrototype = TypeError.prototype;
const foreignTypePrototype = other.TypeError.prototype;
const ForeignError = other.Error;
const marker = new ForeignError('by-copy-marker');
function wrong() { throw 'public constructor observed'; }
globalThis.Array = wrong; other.Array = wrong;
globalThis.RangeError = wrong; other.RangeError = wrong;
globalThis.TypeError = wrong; other.TypeError = wrong;

function expectNative(action, prototype) {
  try { action(); } catch (error) {
    if (Object.getPrototypeOf(error) !== prototype) throw 'called-Realm native error';
    return;
  }
  throw 'missing native error';
}
function expectMarker(action, expectedTrace, trace) {
  const previous = {};
  let result = previous;
  let caught = false;
  try { result = action(); } catch (error) {
    if (error !== marker || Object.getPrototypeOf(error) !== ForeignError.prototype) throw 'original foreign marker';
    caught = true; trace.push('catch');
  } finally { trace.push('finally'); }
  if (!caught || result !== previous || trace.join(',') !== expectedTrace) throw 'abrupt before result publication';
}

let bounds = 0;
for (let direction = 0; direction < 2; direction++) {
  const methods = direction === 0 ? localMethods : foreignMethods;
  const rangePrototype = direction === 0 ? localRangePrototype : foreignRangePrototype;
  const typePrototype = direction === 0 ? localTypePrototype : foreignTypePrototype;
  const Source = direction === 0 ? ForeignArray : LocalArray;
  let lengthGets = 0;
  let indexGets = 0;
  const oversized = {get length() { lengthGets++; return 4294967296; },
    get 0() { indexGets++; throw 'oversized index Get'; }};
  expectNative(() => methods[0].call(oversized), rangePrototype);
  expectNative(() => methods[1].call(oversized, 0, 1), rangePrototype);
  expectNative(() => methods[2].call(oversized, 0, 0), rangePrototype);
  if (lengthGets !== 3 || indexGets !== 0) throw 'ArrayCreate length bounds before Gets';
  bounds += 3;
  let indexCoercions = 0;
  const outOfRange = {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'index conversion hint';
    indexCoercions++; return 0;
  }};
  expectNative(() => methods[1].call({length: 0}, outOfRange, {}), rangePrototype);
  if (indexCoercions !== 1) throw 'with empty index coerced once';
  bounds++;
  const hugeTrace = [];
  const huge = {get length() { hugeTrace.push('length'); return 9007199254740991; },
    get 0() { throw 'unsafe result indexed Get'; }};
  const start = {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'start conversion hint'; hugeTrace.push('start'); return 0;
  }};
  const skip = {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'skip conversion hint'; hugeTrace.push('skip'); return 0;
  }};
  expectNative(() => methods[2].call(huge, start, skip, 'insert'), typePrototype);
  if (hugeTrace.join(',') !== 'length,start,skip') throw 'safe-length bound follows operand conversions';
  bounds++;

  const lengthTrace = [];
  const throwingLength = {get length() { lengthTrace.push('length'); throw marker; },
    get 0() { throw 'Get after length throw'; }};
  expectMarker(() => methods[0].call(throwingLength), 'length,catch,finally', lengthTrace);
  const reverseTrace = [];
  const source = new Source(3);
  source[0] = 10; source[1] = 20; source[2] = 30;
  const reverseProxy = new Proxy(source, {
    get(target, key, receiver) {
      reverseTrace.push(key);
      if (key === '1') throw marker;
      return Reflect.get(target, key, receiver);
    },
    set() { throw 'source write'; },
    deleteProperty() { throw 'source delete'; }
  });
  expectMarker(() => methods[0].call(reverseProxy), 'length,2,1,catch,finally', reverseTrace);
  if (source.length !== 3 || source[0] !== 10 || source[1] !== 20 || source[2] !== 30) throw 'reverse abrupt source intact';
  const withTrace = [];
  const throwingIndex = {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'index hint'; withTrace.push('index'); throw marker;
  }};
  const withSource = {get length() { withTrace.push('length'); return 2; },
    get 0() { throw 'Get after index throw'; }};
  expectMarker(() => methods[1].call(withSource, throwingIndex, {}), 'length,index,catch,finally', withTrace);
  const withGetTrace = [];
  const withGetSource = {get length() { withGetTrace.push('length'); return 3; },
    get 0() { withGetTrace.push('0'); return 1; },
    get 1() { throw 'replaced Get on abrupt path'; },
    get 2() { withGetTrace.push('2'); throw marker; }};
  expectMarker(() => methods[1].call(withGetSource, 1, {}), 'length,0,2,catch,finally', withGetTrace);
  const skipTrace = [];
  const skipSource = {get length() { skipTrace.push('length'); return 3; },
    get 0() { throw 'Get after skip conversion throw'; }};
  const throwingSkip = {[Symbol.toPrimitive](hint) {
    if (hint !== 'number') throw 'skip hint'; skipTrace.push('skip'); throw marker;
  }};
  expectMarker(() => methods[2].call(skipSource, 1, throwingSkip), 'length,skip,catch,finally', skipTrace);
  const spliceTrace = [];
  const spliceSource = {get length() { spliceTrace.push('length'); return 3; },
    get 0() { spliceTrace.push('0'); return 'kept'; },
    get 1() { throw 'deleted Get on abrupt path'; },
    get 2() { spliceTrace.push('2'); throw marker; }};
  expectMarker(() => methods[2].call(spliceSource, 1, 1, {}), 'length,0,2,catch,finally', spliceTrace);
}
if (bounds !== 10) throw 'both called-Realm bound cohorts';
print('array-by-copy-abrupt:ok');
262;
