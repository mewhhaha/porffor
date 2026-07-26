function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertThrows(expectedConstructor, callback, label) {
  try {
    callback();
  } catch (error) {
    assertSame(error.constructor, expectedConstructor, label + " constructor");
    return;
  }
  throw label + " did not throw";
}

var typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
var typedArrayConstructor = Object.getPrototypeOf(Uint8Array);
var slice = typedArrayPrototype.slice;
var descriptor = Object.getOwnPropertyDescriptor(typedArrayPrototype, "slice");
assertSame(descriptor.value, slice, "descriptor value");
assertSame(descriptor.writable, true, "descriptor writable");
assertSame(descriptor.enumerable, false, "descriptor enumerable");
assertSame(descriptor.configurable, true, "descriptor configurable");
assertSame(slice.name, "slice", "function name");
assertSame(slice.length, 2, "function length");

var speciesDescriptor = Object.getOwnPropertyDescriptor(
  typedArrayConstructor,
  Symbol.species
);
assertSame(typeof speciesDescriptor.get, "function", "species getter");
assertSame(speciesDescriptor.set, undefined, "species setter");
assertSame(speciesDescriptor.enumerable, false, "species enumerable");
assertSame(speciesDescriptor.configurable, true, "species configurable");
assertSame(
  typedArrayConstructor[Symbol.species],
  typedArrayConstructor,
  "typed array intrinsic species"
);
assertSame(Uint8Array[Symbol.species], Uint8Array, "concrete typed array species");

var source = new Uint8Array([10, 20, 30, 40]);
var order = [];
var target = new Uint16Array(3);
var constructor = {};
Object.defineProperty(constructor, Symbol.species, {
  get: function() {
    order.push("species");
    return function(length) {
      order.push("construct");
      assertSame(length, 2, "species length");
      return target;
    };
  }
});
source.constructor = constructor;
var result = source.slice(
  {
    valueOf: function() {
      order.push("start");
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
assertSame(result, target, "species result");
assertSame(order.join(","), "start,end,species,construct", "observable ordering");
assertSame(target[0], 20, "converted first value");
assertSame(target[1], 30, "converted second value");
assertSame(target[2], 0, "species target remainder");
assertSame(source[0], 10, "source remains unchanged");

var overlapSource = new Uint8Array([10, 20, 30, 40, 50, 60]);
overlapSource.constructor = {};
overlapSource.constructor[Symbol.species] = function() {
  return new Uint8Array(overlapSource.buffer, 2);
};
var overlapResult = overlapSource.slice(1, 4);
assertSame(overlapResult.length, 4, "overlap target length");
assertSame(overlapResult[0], 20, "overlap first value");
assertSame(overlapResult[1], 20, "overlap second value");
assertSame(overlapResult[2], 20, "overlap third value");
assertSame(overlapResult[3], 60, "overlap fourth value");

var bitBuffer = new ArrayBuffer(4);
new Uint32Array(bitBuffer)[0] = 2141266757;
var floatResult = new Float32Array(bitBuffer).slice();
assertSame(new Uint32Array(floatResult.buffer)[0], 2141266757, "same type copies bytes");

var resizableBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var trackingSource = new Uint8Array(resizableBuffer);
trackingSource.set([1, 2, 3, 4]);
var shrunkResult = trackingSource.slice({
  valueOf: function() {
    resizableBuffer.resize(0);
    return 1;
  }
});
assertSame(shrunkResult.length, 3, "shrink keeps initial result length");
assertSame(shrunkResult[0], 0, "shrink leaves first target element zero");
assertSame(shrunkResult[2], 0, "shrink leaves last target element zero");

resizableBuffer.resize(4);
var fixedSource = new Uint8Array(resizableBuffer, 0, 4);
assertThrows(TypeError, function() {
  fixedSource.slice({
    valueOf: function() {
      resizableBuffer.resize(0);
      return 0;
    }
  });
}, "fixed source becomes out of bounds");

var speciesResizableBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
var resizeDuringSpeciesConstruction = false;
class ResizingUint8Array extends Uint8Array {
  constructor(...params) {
    super(...params);
    if (resizeDuringSpeciesConstruction) {
      speciesResizableBuffer.resize(2);
    }
  }
}
var speciesFixedSource = new ResizingUint8Array(speciesResizableBuffer, 0, 4);
resizeDuringSpeciesConstruction = true;
assertThrows(TypeError, function() {
  speciesFixedSource.slice();
}, "species resize makes fixed source out of bounds");
assertSame(speciesResizableBuffer.byteLength, 2, "species constructor ran");

var bigintSource = new BigInt64Array([1n, -2n]);
var bigintTarget = new BigUint64Array(2);
bigintSource.constructor = {};
bigintSource.constructor[Symbol.species] = function() {
  return bigintTarget;
};
assertSame(bigintSource.slice(), bigintTarget, "bigint species result");
assertSame(bigintTarget[0], 1n, "bigint first value");
assertSame(bigintTarget[1], 18446744073709551614n, "bigint wrapped value");

var wrongContentType = new Uint8Array([1]);
wrongContentType.constructor = {};
wrongContentType.constructor[Symbol.species] = function() {
  return new BigInt64Array(1);
};
assertThrows(TypeError, function() {
  wrongContentType.slice();
}, "content type mismatch");

var tooSmall = new Uint8Array([1, 2]);
tooSmall.constructor = {};
tooSmall.constructor[Symbol.species] = function() {
  return new Uint8Array(1);
};
assertThrows(TypeError, function() {
  tooSmall.slice();
}, "species target too small");

var detachedBuffer = new ArrayBuffer(2);
var detachedSource = new Uint8Array(detachedBuffer);
detachedBuffer.transfer();
assertThrows(TypeError, function() {
  detachedSource.slice();
}, "detached source");

assertThrows(TypeError, function() {
  slice.call({}, 0);
}, "invalid receiver");

true;
