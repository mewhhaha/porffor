const marker = new (__lilaCreateRealm().global.Error)('with foreign marker');
function expectError(action, Prototype) {
  try { action(); } catch (error) {
    if (Object.getPrototypeOf(error) !== Prototype) throw 'wrong native error';
    return;
  }
  throw 'missing native error';
}
function expectMarker(action) {
  try { action(); } catch (error) {
    if (error !== marker) throw 'lost foreign abrupt identity';
    return;
  }
  throw 'missing abrupt';
}

const regrownBuffer = new ArrayBuffer(6, { maxByteLength: 12 });
const regrown = new Int16Array(regrownBuffer);
regrown.set([1, 2, 3]);
const trace = [];
const result = regrown.with(
  { valueOf() { trace.push('index'); regrownBuffer.resize(2); return -1; } },
  { valueOf() { trace.push('replacement'); regrownBuffer.resize(8); regrown.set([5, 6, 7, 8]); return 9; } }
);
if (trace.join() !== 'index,replacement' || result.length !== 3 || result.buffer.byteLength !== 6) throw 'captured length and order';
if (result[0] !== 5 || result[1] !== 6 || result[2] !== 9 || regrown.length !== 4 || regrown[2] !== 7) throw 'fresh reads after regrowth';

const shrunkBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
const shrunk = new Uint8Array(shrunkBuffer);
shrunk.set([1, 2, 3, 4]);
const shrunkTrace = [];
let prior = 'unchanged';
let finalizers = 0;
try {
  prior = shrunk.with(
    { valueOf() { shrunkTrace.push('index'); return -1; } },
    { valueOf() { shrunkTrace.push('replacement'); shrunkBuffer.resize(2); return 9; } }
  );
  throw 'missing fresh RangeError';
} catch (error) {
  if (Object.getPrototypeOf(error) !== RangeError.prototype) throw 'shrink error';
} finally { finalizers++; }
if (prior !== 'unchanged' || finalizers !== 1 || shrunkTrace.join() !== 'index,replacement' || shrunk.length !== 2) throw 'shrink completion';

let replacementCalls = 0;
expectMarker(function() {
  new Uint8Array([1]).with(
    { valueOf() { throw marker; } },
    { valueOf() { replacementCalls++; return 2; } }
  );
});
if (replacementCalls !== 0) throw 'index abrupt must precede replacement';

const abruptNumber = new Uint8Array([1, 2]);
const abruptTrace = [];
expectMarker(function() {
  abruptNumber.with(
    { valueOf() { abruptTrace.push('index'); return 1; } },
    { valueOf() { abruptTrace.push('replacement'); __lilaDetachArrayBuffer(abruptNumber.buffer); throw marker; } }
  );
});
if (abruptTrace.join() !== 'index,replacement' || abruptNumber.length !== 0) throw 'Number abrupt before live index';

const detachedNumber = new Uint8Array([1, 2]);
const detachedTrace = [];
expectError(function() {
  detachedNumber.with(
    { valueOf() { detachedTrace.push('index'); return 1; } },
    { valueOf() { detachedTrace.push('replacement'); __lilaDetachArrayBuffer(detachedNumber.buffer); return 7; } }
  );
}, RangeError.prototype);
if (detachedTrace.join() !== 'index,replacement') throw 'detachment index order';

const bigintBuffer = new ArrayBuffer(16, { maxByteLength: 32 });
const bigint = new BigInt64Array(bigintBuffer);
bigint.set([1n, 2n]);
const bigintTrace = [];
expectError(function() {
  bigint.with(
    { valueOf() { bigintTrace.push('index'); return 1; } },
    { valueOf() { bigintTrace.push('replacement'); bigintBuffer.resize(8); return 3; } }
  );
}, TypeError.prototype);
if (bigintTrace.join() !== 'index,replacement' || bigint.length !== 1) throw 'ToBigInt error before invalid index';

const detachedBigInt = new BigInt64Array([1n, 2n]);
const detachedBigIntTrace = [];
expectError(function() {
  detachedBigInt.with(
    { valueOf() { detachedBigIntTrace.push('index'); return 1; } },
    { valueOf() { detachedBigIntTrace.push('replacement'); __lilaDetachArrayBuffer(detachedBigInt.buffer); return 8n; } }
  );
}, RangeError.prototype);
if (detachedBigIntTrace.join() !== 'index,replacement') throw 'BigInt detachment after conversion';

const abruptBigInt = new BigInt64Array([1n, 2n]);
expectMarker(function() {
  abruptBigInt.with(1, { valueOf() { __lilaDetachArrayBuffer(abruptBigInt.buffer); throw marker; } });
});

let entryConversions = 0;
const entryDetached = new Uint8Array([1]);
__lilaDetachArrayBuffer(entryDetached.buffer);
expectError(function() {
  entryDetached.with(
    { valueOf() { entryConversions++; return 0; } },
    { valueOf() { entryConversions++; return 1; } }
  );
}, TypeError.prototype);
if (entryConversions !== 0) throw 'entry validation before conversion';
print('typed-array-same-type-views:ok');
262;
