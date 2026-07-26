function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

var numeric = new Uint8Array([0, 0, 0, 0]);
assertSame(numeric.set([257, 2], 1), undefined, "return value");
assertSame(numeric[0], 0, "numeric prefix");
assertSame(numeric[1], 1, "numeric conversion");
assertSame(numeric[2], 2, "numeric second value");

var overlap = new Uint8Array([1, 2, 3, 4]);
overlap.set(overlap.subarray(0, 3), 1);
assertSame(overlap[0], 1, "overlap first value");
assertSame(overlap[1], 1, "overlap snapshot one");
assertSame(overlap[2], 2, "overlap snapshot two");
assertSame(overlap[3], 3, "overlap snapshot three");

var bigint = new BigInt64Array(2);
bigint.set([5n, -6n]);
assertSame(bigint[0], 5n, "bigint first value");
assertSame(bigint[1], -6n, "bigint second value");

var order = 0;
var orderedSource = {
  get length() {
    assertSame(order, 1, "length ordering");
    order = 2;
    return 1;
  },
  get 0() {
    assertSame(order, 2, "element ordering");
    order = 3;
    return 9;
  }
};
numeric.set(orderedSource, {
  valueOf: function() {
    assertSame(order, 0, "offset ordering");
    order = 1;
    return 0;
  }
});
assertSame(order, 3, "observable ordering");
assertSame(numeric[0], 9, "ordered source value");

var shared = new Uint8Array(new SharedArrayBuffer(2));
shared.set([42, 1]);
assertSame(shared[0], 42, "shared first value");
assertSame(shared[1], 1, "shared second value");

var negativeOffsetThrew = false;
try {
  numeric.set([], -1);
} catch (error) {
  negativeOffsetThrew = error instanceof RangeError;
}
assertSame(negativeOffsetThrew, true, "negative offset");

var contentTypeThrew = false;
try {
  numeric.set(bigint);
} catch (error) {
  contentTypeThrew = error instanceof TypeError;
}
assertSame(contentTypeThrew, true, "content type mismatch");

var detachedBuffer = new ArrayBuffer(2);
var detachedTarget = new Uint8Array(detachedBuffer);
var detachedDuringOffsetThrew = false;
try {
  detachedTarget.set([1], {
    valueOf: function() {
      detachedBuffer.transfer();
      return 0;
    }
  });
} catch (error) {
  detachedDuringOffsetThrew = error instanceof TypeError;
}
assertSame(detachedDuringOffsetThrew, true, "detached during offset coercion");

true;
