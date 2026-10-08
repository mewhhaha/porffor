const numeric = new Float64Array([NaN, 0, -0, 2, -1, NaN, -0, 0]);
const sorted = numeric.toSorted();
if (sorted[0] !== -1 || !Object.is(sorted[1], -0) || !Object.is(sorted[2], -0) ||
    !Object.is(sorted[3], 0) || !Object.is(sorted[4], 0) || sorted[5] !== 2 ||
    !Number.isNaN(sorted[6]) || !Number.isNaN(sorted[7])) throw 'numeric default order';
if (!Number.isNaN(numeric[0]) || !Object.is(numeric[2], -0)) throw 'toSorted source unchanged';
const stable = new Float64Array([3, -0, 2, 0]).toSorted(function() { return NaN; });
if (stable[0] !== 3 || !Object.is(stable[1], -0) || stable[2] !== 2 || !Object.is(stable[3], 0)) throw 'stable equal comparison';
const bigint = new BigInt64Array([3n, -1n, 2n, 0n]).toSorted();
if (bigint[0] !== -1n || bigint[1] !== 0n || bigint[2] !== 2n || bigint[3] !== 3n) throw 'BigInt numeric order';
const bigintStable = new BigUint64Array([3n, 1n, 2n]).toSorted(function() { return 0; });
if (bigintStable[0] !== 3n || bigintStable[1] !== 1n || bigintStable[2] !== 2n) throw 'BigInt stable comparison';

const foreign = __lilaCreateRealm().global;
const marker = new foreign.Error('comparison coercion marker');
const abruptSource = new Int16Array([3, 2, 1]);
const trace = [];
let calls = 0;
let coercions = 0;
const comparator = new Proxy(function() { throw 'Proxy apply must own Call'; }, {
  apply(target, thisArg, argumentsList) {
    if (thisArg !== undefined || argumentsList.length !== 2) throw 'comparator Call operands';
    calls++;
    trace.push('call');
    return { valueOf() {
      coercions++;
      trace.push('number');
      if (coercions === 1) {
        __lilaDetachArrayBuffer(abruptSource.buffer);
        return argumentsList[0] - argumentsList[1];
      }
      throw marker;
    } };
  }
});
let assigned = 'prior';
try { assigned = abruptSource.sort(comparator); }
catch (error) { if (error !== marker) throw 'lost comparison abrupt identity'; trace.push('catch'); }
finally { trace.push('finally'); }
if (assigned !== 'prior' || calls !== 2 || coercions !== 2 || abruptSource.length !== 0 ||
    trace.join() !== 'call,number,call,number,catch,finally') throw 'continued comparison and abrupt completion';

const detached = new Int16Array([4, 3, 2, 1]);
let detachedCalls = 0;
const detachedResult = detached.sort(function(left, right) {
  detachedCalls++;
  if (detachedCalls === 1) __lilaDetachArrayBuffer(detached.buffer);
  return left - right;
});
if (detachedResult !== detached || detached.length !== 0 || detachedCalls <= 1) throw 'non-abrupt detachment must keep comparing';

const resizedBuffer = new ArrayBuffer(8, { maxByteLength: 12 });
const resized = new Int16Array(resizedBuffer);
resized.set([4, 3, 2, 1]);
let resizeCalls = 0;
const resizedResult = resized.sort(function(left, right) {
  resizeCalls++;
  if (resizeCalls === 1) resizedBuffer.resize(2);
  if (resizeCalls === 2) { resizedBuffer.resize(10); resized[4] = 77; }
  return left - right;
});
if (resizeCalls <= 1 || resizedResult !== resized || resized.length !== 5 ||
    resized[0] !== 1 || resized[1] !== 2 || resized[2] !== 3 || resized[3] !== 4 || resized[4] !== 77) throw 'fresh writeback after resize';

const copySource = new Int16Array([3, 2, 1]);
let copyCalls = 0;
const copy = copySource.toSorted(function(left, right) {
  copyCalls++;
  if (copyCalls === 1) __lilaDetachArrayBuffer(copySource.buffer);
  return left - right;
});
if (copySource.length !== 0 || copyCalls <= 1 || copy[0] !== 1 || copy[1] !== 2 || copy[2] !== 3) throw 'toSorted private snapshot';

const ForeignArray = foreign.Array;
const foreignArrayPrototype = ForeignArray.prototype;
const foreignToSorted = foreignArrayPrototype.toSorted;
foreign.Array = function() { throw 'public foreign Array called'; };
const gets = [];
const parent = { get 1() { return 'a'; } };
const source = Object.create(parent);
source.length = 3;
source[0] = 'b';
const arrayLike = new Proxy(source, {
  get(target, key, receiver) {
    if (receiver !== arrayLike) throw 'Array Get receiver';
    gets.push(key);
    return Reflect.get(target, key, receiver);
  },
  has() { throw 'toSorted must read through holes'; }
});
const arrayResult = foreignToSorted.call(arrayLike);
if (gets.join() !== 'length,0,1,2' || Object.getPrototypeOf(arrayResult) !== foreignArrayPrototype ||
    arrayResult[0] !== 'a' || arrayResult[1] !== 'b' || arrayResult[2] !== undefined ||
    !Object.prototype.hasOwnProperty.call(arrayResult, '2') || source[0] !== 'b') throw 'borrowed Array toSorted holes';
const foreignSource = new ForeignArray(3);
foreignSource[0] = 3;
foreignSource[2] = 1;
const localResult = Array.prototype.toSorted.call(foreignSource);
if (Object.getPrototypeOf(localResult) !== Array.prototype || localResult[0] !== 1 ||
    localResult[1] !== 3 || localResult[2] !== undefined || !Object.prototype.hasOwnProperty.call(localResult, '2')) throw 'reverse Array borrowing direction';
print('typed-array-same-type-sort:ok');
262;
