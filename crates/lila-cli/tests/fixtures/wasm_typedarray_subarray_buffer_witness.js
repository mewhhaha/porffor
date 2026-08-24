function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label + ": " + actual;
}

function captureError(callback, label) {
  try {
    callback();
  } catch (error) {
    return error;
  }
  throw label + " did not throw";
}

function assertErrorPrototype(callback, expectedPrototype, label) {
  var error = captureError(callback, label);
  if (Object.getPrototypeOf(error) !== expectedPrototype) {
    throw label + " wrong error Realm";
  }
}

var fixedBuffer = new ArrayBuffer(12, { maxByteLength: 12 });
var fixedSource = new Uint16Array(fixedBuffer, 2, 4);
fixedSource.set([10, 20, 30, 40]);
var fixedResult = fixedSource.subarray(1, 3);
assertSame(fixedResult.buffer, fixedBuffer, "fixed buffer");
assertSame(fixedResult.byteOffset, 4, "fixed byte offset");
assertSame(fixedResult.length, 2, "fixed length");
assertSame(Object.getPrototypeOf(fixedResult), Uint16Array.prototype, "fixed element kind");
assertSame(fixedResult[0], 20, "fixed first element");
assertSame(fixedResult[1], 30, "fixed second element");
fixedBuffer.resize(6);
assertSame(fixedResult.length, 0, "fixed result out of bounds");
fixedBuffer.resize(12);
assertSame(fixedResult.length, 2, "fixed result regrown");

var trackingBuffer = new ArrayBuffer(10, { maxByteLength: 14 });
var trackingSource = new Uint16Array(trackingBuffer, 2);
var trackingResult = trackingSource.subarray(1);
assertSame(trackingResult.byteOffset, 4, "tracking byte offset");
assertSame(trackingResult.length, 3, "tracking initial length");
trackingBuffer.resize(9);
assertSame(trackingResult.length, 2, "tracking odd-byte shrink floor");
trackingBuffer.resize(13);
assertSame(trackingResult.length, 4, "tracking odd-byte growth floor");
var fixedFromTracking = trackingSource.subarray(1, 99);
assertSame(fixedFromTracking.length, 4, "explicit end creates fixed result");
trackingBuffer.resize(14);
assertSame(fixedFromTracking.length, 4, "fixed result does not track growth");

var order = [];
var orderedSource = new Uint8Array([1, 2, 3, 4]);
orderedSource.constructor = {
  [Symbol.species]: function(buffer, byteOffset, length) {
    order.push("species");
    assertSame(buffer, orderedSource.buffer, "species buffer");
    assertSame(byteOffset, 1, "species byte offset");
    assertSame(length, 2, "species length");
    return new Uint8Array(buffer, byteOffset, length);
  }
};
var orderedResult = orderedSource.subarray(
  {
    valueOf: function() {
      order.push("begin");
      return 1;
    }
  },
  {
    valueOf: function() {
      order.push("end");
      return 3;
    }
  }
);
assertSame(order.join(","), "begin,end,species", "coercion and species order");
assertSame(orderedResult[0], 2, "ordered result first element");
assertSame(orderedResult[1], 3, "ordered result second element");

var outOfBoundsBuffer = new ArrayBuffer(8, { maxByteLength: 8 });
var outOfBounds = new Uint16Array(outOfBoundsBuffer, 2, 3);
outOfBoundsBuffer.resize(0);
var outOfBoundsBeginCalls = 0;
var restoredFromOutOfBounds = outOfBounds.subarray(
  {
    valueOf: function() {
      outOfBoundsBeginCalls = outOfBoundsBeginCalls + 1;
      outOfBoundsBuffer.resize(8);
      return 1;
    }
  },
  1
);
assertSame(outOfBoundsBeginCalls, 1, "out-of-bounds begin coercion");
assertSame(restoredFromOutOfBounds.byteOffset, 2, "out-of-bounds stored byte offset");
assertSame(restoredFromOutOfBounds.length, 0, "out-of-bounds zero length snapshot");

var detachedBuffer = new ArrayBuffer(4);
var detached = new Uint8Array(detachedBuffer, 1, 2);
__lilaDetachArrayBuffer(detachedBuffer);
var detachedOrder = [];
assertErrorPrototype(function() {
  detached.subarray(
    {
      valueOf: function() {
        detachedOrder.push("begin");
        return 0;
      }
    },
    {
      valueOf: function() {
        detachedOrder.push("end");
        return 0;
      }
    }
  );
}, TypeError.prototype, "detached default constructor");
assertSame(detachedOrder.join(","), "begin,end", "detached coercion order");

var customDetachedBuffer = new ArrayBuffer(4);
var customDetached = new Uint8Array(customDetachedBuffer, 1, 2);
var customDetachedResult = new Uint8Array(0);
var customDetachedSpeciesCalls = 0;
customDetached.constructor = {
  [Symbol.species]: function(buffer, byteOffset, length) {
    customDetachedSpeciesCalls = customDetachedSpeciesCalls + 1;
    assertSame(buffer, customDetachedBuffer, "custom detached buffer");
    assertSame(byteOffset, 1, "custom detached stored byte offset");
    assertSame(length, 0, "custom detached zero length");
    return customDetachedResult;
  }
};
__lilaDetachArrayBuffer(customDetachedBuffer);
assertSame(customDetached.subarray(0, 0), customDetachedResult, "custom detached result");
assertSame(customDetachedSpeciesCalls, 1, "custom detached species reached");

var other = __lilaCreateRealm().global;
var otherSubarray = other.Uint8Array.prototype.subarray;
var entryDetached = new Uint8Array(1);
__lilaDetachArrayBuffer(entryDetached.buffer);
assertErrorPrototype(function() {
  otherSubarray.call(entryDetached, 0);
}, TypeError.prototype, "entry constructor owns detached error");

var bigintSource = new BigUint64Array(3);
var bigintResult = bigintSource.subarray(1);
assertSame(Object.getPrototypeOf(bigintResult), BigUint64Array.prototype, "BigInt element kind");
assertSame(bigintResult.length, 2, "BigInt length");

967;
