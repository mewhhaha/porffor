function sameValue(actual, expected, label) {
  if (actual !== expected) throw label + " expected " + expected + " got " + actual;
}

function throwsTypeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (e) {
    threw = e && e.name === "TypeError";
  }
  if (!threw) throw label + " did not throw TypeError";
}

let arrayKeys = [7, 8].keys();
let arrayKey0 = arrayKeys.next();
sameValue(arrayKey0.value, 0, "array key 0");
sameValue(arrayKey0.done, false, "array key 0 done");
let arrayKey1 = arrayKeys.next();
sameValue(arrayKey1.value, 1, "array key 1");
sameValue(arrayKey1.done, false, "array key 1 done");
sameValue(arrayKeys.next().done, true, "array keys exhausted");

let arrayEntries = [7, 8].entries();
let arrayEntry0 = arrayEntries.next().value;
sameValue(arrayEntry0[0], 0, "array entry 0 key");
sameValue(arrayEntry0[1], 7, "array entry 0 value");
let arrayEntry1 = arrayEntries.next().value;
sameValue(arrayEntry1[0], 1, "array entry 1 key");
sameValue(arrayEntry1[1], 8, "array entry 1 value");
sameValue(arrayEntries.next().done, true, "array entries exhausted");

let rab = new ArrayBuffer(4, { maxByteLength: 6 });
let write = new Uint8Array(rab);
for (let i = 0; i < 4; i++) write[i] = i + 10;

let fixed = new Uint8Array(rab, 0, 4);
let tracking = new Uint8Array(rab, 0);
let typedKeys = Array.prototype.keys.call(fixed);
sameValue(typedKeys.next().value, 0, "typed key 0");
sameValue(typedKeys.next().value, 1, "typed key 1");

let typedEntries = Array.prototype.entries.call(tracking);
let typedEntry0 = typedEntries.next().value;
sameValue(typedEntry0[0], 0, "typed entry 0 key");
sameValue(typedEntry0[1], 10, "typed entry 0 value");
let typedEntry1 = typedEntries.next().value;
sameValue(typedEntry1[0], 1, "typed entry 1 key");
sameValue(typedEntry1[1], 11, "typed entry 1 value");

rab.resize(3);
throwsTypeError(function () {
  Array.prototype.keys.call(fixed).next();
}, "fixed keys shrink");
throwsTypeError(function () {
  Array.prototype.entries.call(fixed).next();
}, "fixed entries shrink");

let trackingKeys = Array.prototype.keys.call(tracking);
sameValue(trackingKeys.next().value, 0, "tracking key 0 after shrink");
sameValue(trackingKeys.next().value, 1, "tracking key 1 after shrink");
sameValue(trackingKeys.next().value, 2, "tracking key 2 after shrink");
sameValue(trackingKeys.next().done, true, "tracking keys exhausted after shrink");

true;
