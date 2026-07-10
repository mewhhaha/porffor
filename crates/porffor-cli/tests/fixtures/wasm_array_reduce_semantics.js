function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

let sparse = [, 2, , 4];
Array.prototype[0] = 1;
Array.prototype[2] = 3;
let sparseForward = sparse.reduce(function (accumulator, value) {
  return accumulator + value;
});
let sparseReverse = sparse.reduceRight(function (accumulator, value) {
  return accumulator * 10 + value;
});
delete Array.prototype[0];
delete Array.prototype[2];

let snapshot = [1, 2, 3];
let snapshotResult = snapshot.reduce(function (accumulator, value, index) {
  if (index === 0) snapshot.push(4);
  return accumulator + value;
}, 0);

let generic = { 0: 7, 2: 9, length: 3 };
let genericResult = Array.prototype.reduce.call(generic, function (accumulator, value) {
  return accumulator + value;
}, 0);

function reduceArgumentsLength(first, second) {
  Object.defineProperty(arguments, "length", {
    get: function () {
      return 1;
    },
    configurable: true
  });
  return Array.prototype.reduce.call(arguments, function (accumulator, value) {
    return accumulator + value;
  }, 0);
}
let argumentsLengthResult = reduceArgumentsLength(4, 8);

let lengthReadBeforeCallableCheck = false;
let orderedReceiver = { 0: 1 };
Object.defineProperty(orderedReceiver, "length", {
  get: function () {
    lengthReadBeforeCallableCheck = true;
    return 1;
  },
  configurable: true
});
let orderedTypeError = throwsTypeError(function () {
  Array.prototype.reduce.call(orderedReceiver, null);
});

let sawUndefinedThis = false;
[1].reduce(function () {
  "use strict";
  sawUndefinedThis = this === undefined;
  return 0;
}, 0);

let initialUndefined = [].reduce(function () {
  throw "must not call reducer";
}, undefined);
let emptyThrows = throwsTypeError(function () {
  [].reduce(function () {});
}) && throwsTypeError(function () {
  [].reduceRight(function () {});
});

let callbackThrow = false;
try {
  [1].reduce(function () {
    throw new Error("callback abrupt");
  }, 0);
} catch (error) {
  callbackThrow = true;
}

let overrideArray = [1];
overrideArray.reduce = function () {
  return 42;
};
let overrideRead = overrideArray.reduce;
let overrideOk = overrideRead() === 42 &&
  overrideArray.reduce(function () { return 0; }) === 42;

let customPrototype = {
  reduce: function () {
    return 43;
  }
};
let customPrototypeArray = [1];
Object.setPrototypeOf(customPrototypeArray, customPrototype);
let customPrototypeOk = customPrototypeArray.reduce(function () { return 0; }) === 43;

function reduceInheritedArguments(value) {
  delete arguments[0];
  Object.prototype[0] = 9;
  try {
    return Array.prototype.reduce.call(arguments, function (accumulator, item) {
      return accumulator + item;
    }, 0);
  } finally {
    delete Object.prototype[0];
  }
}
let inheritedArgumentsOk = reduceInheritedArguments(1) === 9;

function reduceArgumentsAccessor(value) {
  let receiver = arguments;
  Object.defineProperty(arguments, "0", {
    get: function () {
      return this === receiver ? 9 : 0;
    },
    configurable: true
  });
  return Array.prototype.reduce.call(arguments, function (accumulator, item) {
    return accumulator + item;
  }, 0);
}
let argumentsAccessorOk = reduceArgumentsAccessor(1) === 9;

let argumentsGetterThrow = {};
let argumentsGetterThrowOk = false;
function reduceThrowingArgumentsAccessor(value) {
  Object.defineProperty(arguments, "0", {
    get: function () {
      throw argumentsGetterThrow;
    },
    configurable: true
  });
  return Array.prototype.reduce.call(arguments, function (accumulator, item) {
    return accumulator + item;
  }, 0);
}
try {
  reduceThrowingArgumentsAccessor(1);
} catch (error) {
  argumentsGetterThrowOk = error === argumentsGetterThrow;
}

let fakeBuffer = {
  $ArrayBufferByteLength: 8,
  $ArrayBufferDataPtr: 8
};
let typedSlotSpoof = {
  0: 5,
  length: 1,
  $TypedArrayViewedArrayBuffer: fakeBuffer,
  $TypedArrayByteOffset: 0,
  $TypedArrayByteLength: 8,
  $TypedArrayBytesPerElement: 8,
  $TypedArrayElementKind: 1,
  $TypedArrayLengthTracking: false
};
let typedSlotSpoofOk = Array.prototype.reduce.call(
  typedSlotSpoof,
  function (accumulator, item) { return accumulator + item; },
  0
) === 5;

let typed = new Uint8Array([1, 2, 3]);
let typedArrayOk = Array.prototype.reduce.call(
  typed,
  function (accumulator, item) { return accumulator + item; },
  0
) === 6;

let typedShadow = new Uint8Array([1, 2]);
typedShadow.$TypedArrayViewedArrayBuffer = {};
typedShadow.$TypedArrayByteLength = 1;
typedShadow.$TypedArrayBytesPerElement = 2;
let typedShadowOk = Array.prototype.reduce.call(
  typedShadow,
  function (accumulator, item) { return accumulator + item; },
  0
) === 3;

let typedOwnLength = new Uint8Array([1, 2]);
Object.defineProperty(typedOwnLength, "length", { value: 1 });
let typedOwnLengthOk = Array.prototype.reduce.call(
  typedOwnLength,
  function (accumulator, item) { return accumulator + item; },
  0
) === 1;

let typedCustomPrototype = new Uint8Array([1, 2]);
Object.setPrototypeOf(typedCustomPrototype, { length: 1, 0: 9 });
let typedCustomPrototypeOk = Array.prototype.reduce.call(
  typedCustomPrototype,
  function (accumulator, item) { return accumulator + item; },
  0
) === 1;

let checks = 0;
if (sparseForward === 10) checks = checks + 1;
if (sparseReverse === 4321) checks = checks + 2;
if (snapshotResult === 6) checks = checks + 4;
if (snapshot.length === 4) checks = checks + 8;
if (genericResult === 16) checks = checks + 16;
if (argumentsLengthResult === 4) checks = checks + 1024;
if (initialUndefined === undefined) checks = checks + 32;
if (emptyThrows) checks = checks + 64;
if (callbackThrow) checks = checks + 128;
if (lengthReadBeforeCallableCheck && orderedTypeError) checks = checks + 256;
if (sawUndefinedThis) checks = checks + 512;
checks === 2047 && overrideOk && customPrototypeOk && inheritedArgumentsOk &&
  argumentsAccessorOk && typedSlotSpoofOk && typedArrayOk && typedShadowOk &&
  argumentsGetterThrowOk && typedOwnLengthOk && typedCustomPrototypeOk;
