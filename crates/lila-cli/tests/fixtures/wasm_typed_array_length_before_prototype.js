// TypedArray(length) validates the length before touching the prototype:
// ToIndex(length) precedes AllocateTypedArray, so an invalid length throws
// RangeError even when reading newTarget.prototype would throw first.
var newTarget = function () {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get: function () {
    throw "PROTO-READ";
  }
});

function throwsRangeError(fn, label) {
  try {
    fn();
  } catch (error) {
    if (error instanceof RangeError) return;
    throw label + ": wrong error";
  }
  throw label + ": no throw";
}

throwsRangeError(function () {
  Reflect.construct(Uint8Array, [-1], newTarget);
}, "negative");

throwsRangeError(function () {
  Reflect.construct(Uint8Array, [9007199254740993], newTarget);
}, "huge");

// A valid length still reaches the prototype read.
var protoRead = false;
var observingTarget = function () {}.bind(null);
Object.defineProperty(observingTarget, "prototype", {
  get: function () {
    protoRead = true;
    return Uint8Array.prototype;
  }
});
var ta = Reflect.construct(Uint8Array, [4], observingTarget);
if (!protoRead) throw "prototype not read";
if (ta.length !== 4) throw "length";

ta.length;
