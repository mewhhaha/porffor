let objectPrototypeBuffer = Reflect.construct(SharedArrayBuffer, [4], Object);
if (Object.getPrototypeOf(objectPrototypeBuffer) !== Object.prototype) {
  throw "Object NewTarget prototype";
}

let newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, "prototype", {
  get: function() {
    return Array.prototype;
  }
});
let arrayPrototypeBuffer = Reflect.construct(SharedArrayBuffer, [8], newTarget);
if (Object.getPrototypeOf(arrayPrototypeBuffer) !== Array.prototype) {
  throw "bound NewTarget prototype";
}

let primitivePrototypeTarget = function() {};
primitivePrototypeTarget.prototype = null;
let fallbackBuffer = Reflect.construct(
  SharedArrayBuffer,
  [16],
  primitivePrototypeTarget
);
if (Object.getPrototypeOf(fallbackBuffer) !== SharedArrayBuffer.prototype) {
  throw "primitive NewTarget prototype fallback";
}

let prototypeWasRead = false;
let poisonedNewTarget = Object.defineProperty(function() {}.bind(null), "prototype", {
  get: function() {
    prototypeWasRead = true;
    throw "poisoned NewTarget prototype";
  }
});
let invalidLengthThrewRangeError = false;
try {
  Reflect.construct(
    SharedArrayBuffer,
    [10, { maxByteLength: 0 }],
    poisonedNewTarget
  );
} catch (error) {
  invalidLengthThrewRangeError = error.constructor === RangeError;
}
if (prototypeWasRead) throw "NewTarget prototype read before length comparison";
if (!invalidLengthThrewRangeError) throw "byteLength/maxByteLength comparison";

123;
