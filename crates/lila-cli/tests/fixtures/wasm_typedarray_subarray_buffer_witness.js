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

function assertSubarraySpeciesArguments(actualArguments, expectedArguments, label) {
  assertSame(actualArguments.length, expectedArguments.length, label + " count");
  assertSame(actualArguments[0], expectedArguments[0], label + " buffer");
  assertSame(actualArguments[1], expectedArguments[1], label + " byte offset");
  assertSame(
    Object.getOwnPropertyDescriptor(actualArguments, "2") !== undefined,
    expectedArguments.length === 3,
    label + " new length presence"
  );
  assertSame(actualArguments[2], expectedArguments[2], label + " new length");
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

var fixedNumberSpeciesSource = new Uint16Array([10, 20, 30, 40]);
var fixedNumberSpeciesArguments;
fixedNumberSpeciesSource.constructor = {
  [Symbol.species]: function(buffer, byteOffset, length) {
    fixedNumberSpeciesArguments = arguments;
    return new Uint16Array(buffer, byteOffset, length);
  }
};
fixedNumberSpeciesSource.subarray(1);
assertSubarraySpeciesArguments(
  fixedNumberSpeciesArguments,
  [fixedNumberSpeciesSource.buffer, 2, 3],
  "fixed Number species arguments"
);

var trackingNumberSpeciesBuffer = new ArrayBuffer(10, { maxByteLength: 14 });
var trackingNumberSpeciesSource = new Uint16Array(trackingNumberSpeciesBuffer, 2);
var trackingNumberSpeciesArguments;
trackingNumberSpeciesSource.constructor = {
  [Symbol.species]: function(buffer, byteOffset, length) {
    trackingNumberSpeciesArguments = arguments;
    return new Uint16Array(buffer, byteOffset, length);
  }
};
trackingNumberSpeciesSource.subarray(1);
assertSubarraySpeciesArguments(
  trackingNumberSpeciesArguments,
  [trackingNumberSpeciesBuffer, 4],
  "tracking Number omitted-end species arguments"
);
trackingNumberSpeciesSource.subarray(1, 3);
assertSubarraySpeciesArguments(
  trackingNumberSpeciesArguments,
  [trackingNumberSpeciesBuffer, 4, 2],
  "tracking Number explicit-end species arguments"
);

var fixedBigIntSpeciesSource = new BigUint64Array(4);
var fixedBigIntSpeciesArguments;
fixedBigIntSpeciesSource.constructor = {
  [Symbol.species]: function(buffer, byteOffset, length) {
    fixedBigIntSpeciesArguments = arguments;
    return new BigUint64Array(buffer, byteOffset, length);
  }
};
fixedBigIntSpeciesSource.subarray(1);
assertSubarraySpeciesArguments(
  fixedBigIntSpeciesArguments,
  [fixedBigIntSpeciesSource.buffer, 8, 3],
  "fixed BigInt species arguments"
);

var trackingBigIntSpeciesBuffer = new ArrayBuffer(32, { maxByteLength: 64 });
var trackingBigIntSpeciesSource = new BigUint64Array(trackingBigIntSpeciesBuffer, 8);
var trackingBigIntSpeciesArguments;
trackingBigIntSpeciesSource.constructor = {
  [Symbol.species]: function(buffer, byteOffset, length) {
    trackingBigIntSpeciesArguments = arguments;
    return new BigUint64Array(buffer, byteOffset, length);
  }
};
trackingBigIntSpeciesSource.subarray(1);
assertSubarraySpeciesArguments(
  trackingBigIntSpeciesArguments,
  [trackingBigIntSpeciesBuffer, 16],
  "tracking BigInt omitted-end species arguments"
);
trackingBigIntSpeciesSource.subarray(1, 3);
assertSubarraySpeciesArguments(
  trackingBigIntSpeciesArguments,
  [trackingBigIntSpeciesBuffer, 16, 2],
  "tracking BigInt explicit-end species arguments"
);

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

var detachedSpeciesSource = new Uint8Array(1);
var detachedSpeciesResult = new Uint8Array(1);
var detachedResultSpeciesCalls = 0;
detachedSpeciesSource.constructor = {
  [Symbol.species]: function() {
    detachedResultSpeciesCalls = detachedResultSpeciesCalls + 1;
    return detachedSpeciesResult;
  }
};
__lilaDetachArrayBuffer(detachedSpeciesResult.buffer);
assertErrorPrototype(function() {
  otherSubarray.call(detachedSpeciesSource, 0);
}, other.TypeError.prototype, "species detached result validation");
assertSame(detachedResultSpeciesCalls, 1, "detached result species reached");

var outOfBoundsSpeciesBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var outOfBoundsSpeciesResult = new Uint8Array(outOfBoundsSpeciesBuffer, 2, 2);
var outOfBoundsSpeciesSource = new Uint8Array(1);
var outOfBoundsResultSpeciesCalls = 0;
outOfBoundsSpeciesSource.constructor = {
  [Symbol.species]: function() {
    outOfBoundsResultSpeciesCalls = outOfBoundsResultSpeciesCalls + 1;
    outOfBoundsSpeciesBuffer.resize(1);
    return outOfBoundsSpeciesResult;
  }
};
assertErrorPrototype(function() {
  otherSubarray.call(outOfBoundsSpeciesSource, 0);
}, other.TypeError.prototype, "species out-of-bounds result validation");
assertSame(outOfBoundsResultSpeciesCalls, 1, "out-of-bounds result species reached");

var bigintSource = new BigUint64Array(3);
var bigintResult = bigintSource.subarray(1);
assertSame(Object.getPrototypeOf(bigintResult), BigUint64Array.prototype, "BigInt element kind");
assertSame(bigintResult.length, 2, "BigInt length");

967;
