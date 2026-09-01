function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function checkResult(result, expectedValue, expectedDone, label) {
  check(result.value, expectedValue, label + " value");
  check(result.done, expectedDone, label + " done");
}

function checkTypeError(thunk, label) {
  var error;
  try {
    thunk();
  } catch (caught) {
    error = caught;
  }
  if (!(error instanceof TypeError)) throw label;
}

var arrayLike = { 0: "left", 1: "right", length: 2 };

var genericKeys = Array.prototype.keys.call(arrayLike);
checkResult(genericKeys.next(), 0, false, "generic keys first");
checkResult(genericKeys.next(), 1, false, "generic keys second");
checkResult(genericKeys.next(), undefined, true, "generic keys complete");

var genericValues = Array.prototype.values.call(arrayLike);
checkResult(genericValues.next(), "left", false, "generic values first");
checkResult(genericValues.next(), "right", false, "generic values second");

for (var invalidKind of [NaN, -1, 0.5, 1.5, 2.9]) {
  var corruptedIterator = Array.prototype.values.call(arrayLike);
  corruptedIterator["$ArrayIterator.kind"] = invalidKind;
  checkTypeError(function () {
    corruptedIterator.next();
  }, "generic iterator accepted invalid kind " + invalidKind);
}

var genericEntries = Array.prototype.entries.call(arrayLike);
var genericEntry = genericEntries.next();
check(genericEntry.done, false, "generic entries done");
check(genericEntry.value[0], 0, "generic entries key");
check(genericEntry.value[1], "left", "generic entries value");

var typed = new Uint8Array([7, 9]);
var borrowedEntry = Array.prototype.entries.call(typed).next();
check(borrowedEntry.done, false, "borrowed typed entry done");
check(borrowedEntry.value[0], 0, "borrowed typed entry key");
check(borrowedEntry.value[1], 7, "borrowed typed entry value");

var typedKeys = Uint8Array.prototype.keys.call(typed);
checkResult(typedKeys.next(), 0, false, "typed keys first");
checkResult(typedKeys.next(), 1, false, "typed keys second");

var typedValues = Uint8Array.prototype.values.call(typed);
checkResult(typedValues.next(), 7, false, "typed values first");
checkResult(typedValues.next(), 9, false, "typed values second");
checkResult(typedValues.next(), undefined, true, "typed values complete");

var typedEntry = Uint8Array.prototype.entries.call(typed).next();
check(typedEntry.done, false, "typed entries done");
check(typedEntry.value[0], 0, "typed entries key");
check(typedEntry.value[1], 7, "typed entries value");

checkTypeError(function () {
  Uint8Array.prototype.keys.call(arrayLike);
}, "typed keys accepted array-like");
checkTypeError(function () {
  Uint8Array.prototype.values.call(arrayLike);
}, "typed values accepted array-like");
checkTypeError(function () {
  Uint8Array.prototype.entries.call(arrayLike);
}, "typed entries accepted array-like");

262;
