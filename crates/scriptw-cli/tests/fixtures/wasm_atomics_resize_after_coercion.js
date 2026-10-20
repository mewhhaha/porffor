function check(condition, label) {
  if (!condition) throw new Error(label);
}
function expectError(kind, action, label) {
  var seen;
  try { action(); } catch (error) { seen = error; }
  check(seen instanceof kind, label);
}

// All nine integer operations observe the backing state after their own
// argument coercions. A fixed view can be OOB even when the selected index
// remains in the backing buffer; a tracking view rejects the removed index.
var operations = [Atomics.load, Atomics.store, Atomics.add, Atomics.sub,
                  Atomics.and, Atomics.or, Atomics.xor, Atomics.exchange,
                  Atomics.compareExchange];
for (var operation of operations) {
  var buffer = new ArrayBuffer(8, { maxByteLength: 16 });
  var fixed = new Int32Array(buffer, 0, 2);
  var index = { valueOf: function () { buffer.resize(4); return 0; } };
  expectError(TypeError, function () { operation(fixed, index, 1, 2); }, 'fixed OOB after index');

  buffer = new ArrayBuffer(8, { maxByteLength: 16 });
  var tracking = new Int32Array(buffer);
  index = { valueOf: function () { buffer.resize(4); return 1; } };
  expectError(RangeError, function () { operation(tracking, index, 1, 2); }, 'tracking removed index');

  buffer = new ArrayBuffer(8, { maxByteLength: 16 });
  fixed = new Int32Array(buffer, 0, 2);
  index = { valueOf: function () { buffer.transfer(); return 0; } };
  expectError(TypeError, function () { operation(fixed, index, 1, 2); }, 'detached after index');
}

for (var operation of operations.slice(1)) {
  var buffer = new ArrayBuffer(8, { maxByteLength: 16 });
  var fixed = new Int32Array(buffer, 0, 2);
  var value = { valueOf: function () { buffer.resize(4); return 1; } };
  expectError(TypeError, function () { operation(fixed, 0, value, 2); }, 'fixed OOB after value');

  buffer = new ArrayBuffer(16, { maxByteLength: 32 });
  var bigTracking = new BigInt64Array(buffer);
  value = { valueOf: function () { buffer.resize(8); return 1n; } };
  expectError(RangeError, function () { operation(bigTracking, 1, value, 2n); }, 'BigInt removed index');
}

// The replacement is coerced before final revalidation, and an abrupt
// coercion keeps its own throw rather than becoming a view error.
var buffer = new ArrayBuffer(8, { maxByteLength: 16 });
var tracking = new Int32Array(buffer);
var order = '';
expectError(RangeError, function () {
  Atomics.compareExchange(tracking,
    { valueOf: function () { order += 'i'; return 1; } },
    { valueOf: function () { order += 'v'; return 0; } },
    { valueOf: function () { order += 'r'; buffer.resize(4); return 9; } });
}, 'replacement revalidation');
check(order === 'ivr', 'all coercions precede final observation');
var marker = {};
buffer = new ArrayBuffer(8, { maxByteLength: 16 });
tracking = new Int32Array(buffer);
var thrown;
try {
  Atomics.store(tracking, 1, { valueOf: function () { buffer.resize(4); throw marker; } });
} catch (error) { thrown = error; }
check(thrown === marker, 'abrupt conversion identity');

// Revalidation compares the approved absolute starting byte, not a second
// floored element-length check. A partial trailing element retains start byte 4.
buffer = new ArrayBuffer(8, { maxByteLength: 16 });
tracking = new Int32Array(buffer);
Atomics.store(tracking, 1, { valueOf: function () { buffer.resize(5); return 7; } });
check(buffer.byteLength === 5, 'partial trailing element start remains approved');

buffer = new ArrayBuffer(8, { maxByteLength: 32 });
tracking = new Int32Array(buffer);
check(Atomics.store(tracking, 1, { valueOf: function () { buffer.resize(16); return 17; } }) === 17,
      'growth preserves converted return');
check(Atomics.load(tracking, 1) === 17, 'growth refreshes current data pointer');
var shared = new SharedArrayBuffer(8, { maxByteLength: 16 });
var sharedView = new Int32Array(shared);
check(Atomics.store(sharedView, 1, { valueOf: function () { shared.grow(16); return 23; } }) === 23,
      'shared growth');
check(Atomics.load(sharedView, 1) === 23, 'shared grown value');
348;
