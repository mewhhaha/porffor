function assertRangeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof RangeError)) throw label + ": wrong error";
  }
  if (!threw) throw label + ": missing throw";
}

function assertRangeErrorAndUnchanged(array, expected, fn, label) {
  assertRangeError(fn, label);
  if (array.length !== expected) throw label + ": length changed";
}

function assertThrowsValue(fn, expected, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (error !== expected) throw label + ": wrong error";
  }
  if (!threw) throw label + ": missing throw";
}

let array = [];
array.length = 0;
if (array.length !== 0) throw "zero length";
array.length = -0;
if (array.length !== 0) throw "negative zero length";
let maximum = [];
maximum.length = 4294967295;
if (maximum.length !== 4294967295) throw "maximum length";

array = [];
let invalid = [-1, 1.5, NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY, undefined, 4294967296];
for (let i = 0; i < invalid.length; i = i + 1) {
  array.length = 9;
  let value = invalid[i];
  assertRangeErrorAndUnchanged(array, 9, function () {
    array.length = value;
  }, "invalid " + i);
}

array.length = 7;
let directRangeCaught = false;
try {
  array.length = -1;
} catch (error) {
  directRangeCaught = error instanceof RangeError;
}
if (!directRangeCaught || array.length !== 7) throw "direct range catch";

let directAbruptCaught = false;
let directAbrupt = {
  valueOf: function () {
    throw "direct conversion";
  }
};
try {
  array.length = directAbrupt;
} catch (error) {
  directAbruptCaught = error === "direct conversion";
}
if (!directAbruptCaught || array.length !== 7) throw "direct conversion catch";

array = [];
let conversions = 0;
let original = {
  valueOf: function () {
    conversions = conversions + 1;
    return 2;
  }
};
let assignmentResult = (array.length = original);
if (assignmentResult !== original) throw "assignment identity";
if (conversions !== 2) throw "assignment coercions";
if (array.length !== 2) throw "assignment length";

let wrappingConversions = 0;
let wrapping = {
  valueOf: function () {
    wrappingConversions = wrappingConversions + 1;
    return wrappingConversions === 1 ? 4294967296 : 0;
  }
};
array.length = wrapping;
if (wrappingConversions !== 2 || array.length !== 0) throw "uint32 positive wrap";

wrappingConversions = 0;
wrapping = {
  valueOf: function () {
    wrappingConversions = wrappingConversions + 1;
    return wrappingConversions === 1 ? -1 : 4294967295;
  }
};
array.length = wrapping;
if (wrappingConversions !== 2 || array.length !== 4294967295) throw "uint32 negative wrap";

let locked = [1];
Object.defineProperty(locked, "length", { writable: false });
let lockedConversions = 0;
let lockedRhs = {
  valueOf: function () {
    lockedConversions = lockedConversions + 1;
    return 0;
  }
};
let lockedAssignmentResult = (locked.length = lockedRhs);
if (lockedAssignmentResult !== lockedRhs || lockedConversions !== 0 || locked.length !== 1) {
  throw "locked sloppy set";
}
let lockedStrictCaught = false;
try {
  (function () {
    "use strict";
    locked.length = lockedRhs;
  })();
} catch (error) {
  lockedStrictCaught = error instanceof TypeError;
}
if (!lockedStrictCaught || lockedConversions !== 0 || locked.length !== 1) {
  throw "locked strict set";
}
if (Reflect.set(locked, "length", lockedRhs) !== false || lockedConversions !== 0) {
  throw "locked reflect set";
}

array.length = 6;
let firstAbrupt = {
  valueOf: function () {
    throw "first conversion";
  }
};
assertThrowsValue(function () {
  array.length = firstAbrupt;
}, "first conversion", "first abrupt");
if (array.length !== 6) throw "first abrupt length";

let secondConversions = 0;
let secondAbrupt = {
  valueOf: function () {
    secondConversions = secondConversions + 1;
    if (secondConversions === 2) throw "second conversion";
    return 3;
  }
};
assertThrowsValue(function () {
  array.length = secondAbrupt;
}, "second conversion", "second abrupt");
if (secondConversions !== 2) throw "second abrupt coercions";
if (array.length !== 6) throw "second abrupt length";

array.length = 8;
assertRangeErrorAndUnchanged(array, 8, function () {
  Object.defineProperty(array, "length", { value: -1 });
}, "define property invalid");
assertRangeErrorAndUnchanged(array, 8, function () {
  Object.defineProperty(array, "len" + "gth", { value: -1 });
}, "define property computed length invalid");
assertRangeErrorAndUnchanged(array, 8, function () {
  Reflect.defineProperty(array, "length", { value: 4294967296 });
}, "reflect define property invalid");

// ToPropertyDescriptor must use ToBoolean on the tagged descriptor values,
// rather than treating their raw representation as an integer boolean.
let descriptorFalsy = [false, 0, -0, NaN, "", null, undefined];
for (let i = 0; i < descriptorFalsy.length; i = i + 1) {
  if (!Reflect.defineProperty([], "length", { enumerable: descriptorFalsy[i] })) {
    throw "reflect length enumerable falsy " + i;
  }
}
let descriptorArray = [];
if (!Reflect.defineProperty(descriptorArray, "length", { configurable: NaN })) {
  throw "reflect length configurable NaN";
}
if (Reflect.defineProperty(descriptorArray, "length", { enumerable: "truthy" })) {
  throw "reflect length enumerable string";
}
if (Reflect.defineProperty(descriptorArray, "length", { writable: "" }) !== true) {
  throw "reflect length writable empty string";
}

function assertTypeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = error instanceof TypeError;
  }
  if (!threw) throw label;
}

// This must fail in ToPropertyDescriptor, before ArraySetLength can turn it
// into Reflect.defineProperty(false).
assertTypeError(function () {
  Reflect.defineProperty([], "length", { value: 0, get: 1 });
}, "reflect malformed length accessor");
assertTypeError(function () {
  Object.defineProperty([], "length", { value: 0, set: 1 });
}, "object malformed length accessor");

// An ordinary ToPrimitive failure is catchable at the assignment site; the
// conversion helper must not return from the whole Wasm function.
assertTypeError(function () {
  array.length = {
    valueOf: function () { return {}; },
    toString: function () { return {}; }
  };
}, "ordinary ToPrimitive length catch");

// Failed shrinking stops at the first non-configurable index, restores P + 1,
// and still applies a requested deferred writable:false transition.
let truncated = [0, 1, 2];
Object.defineProperty(truncated, 1, { configurable: false });
assertTypeError(function () {
  Object.defineProperty(truncated, "length", { value: 0, writable: false });
}, "non-configurable shrink");
let truncatedLength = Object.getOwnPropertyDescriptor(truncated, "length");
if (truncated.length !== 2 || truncatedLength.value !== 2 || truncatedLength.writable !== false) {
  throw "non-configurable shrink descriptor";
}

true;
