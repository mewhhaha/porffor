let typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
let values = typedArrayPrototype.values;
let keys = typedArrayPrototype.keys;
let entries = typedArrayPrototype.entries;

if (typedArrayPrototype[Symbol.iterator] !== values) throw "iterator alias";
for (let methodName of ["values", "keys", "entries"]) {
  let descriptor = Object.getOwnPropertyDescriptor(typedArrayPrototype, methodName);
  if (descriptor === undefined) throw methodName + " descriptor";
  if (descriptor.value !== typedArrayPrototype[methodName]) throw methodName + " value";
  if (descriptor.writable !== true) throw methodName + " writable";
  if (descriptor.enumerable !== false) throw methodName + " enumerable";
  if (descriptor.configurable !== true) throw methodName + " configurable";
  if (typedArrayPrototype[methodName].name !== methodName) throw methodName + " name";
  if (typedArrayPrototype[methodName].length !== 0) throw methodName + " length";
}

let sample = new Uint16Array([10, 20, 30]);
let valueIterator = values.call(sample);
if (Object.getPrototypeOf(valueIterator) !== Object.getPrototypeOf([].values())) throw "iterator prototype";
if (valueIterator[Symbol.iterator]() !== valueIterator) throw "iterator identity";
if (valueIterator.next().value !== 10) throw "first value";
sample[1] = 25;
if (valueIterator.next().value !== 25) throw "current value";
if (valueIterator.next().value !== 30) throw "third value";
if (valueIterator.next().done !== true) throw "values done";

let keyIterator = keys.call(sample);
if (keyIterator.next().value !== 0) throw "first key";
if (keyIterator.next().value !== 1) throw "second key";

let entryIterator = entries.call(sample);
let firstEntry = entryIterator.next().value;
if (firstEntry[0] !== 0 || firstEntry[1] !== 10) throw "first entry";
let secondEntry = entryIterator.next().value;
if (secondEntry[0] !== 1 || secondEntry[1] !== 25) throw "second entry";

let bigintIterator = new BigInt64Array([1n, -2n]).values();
if (bigintIterator.next().value !== 1n) throw "bigint first";
if (bigintIterator.next().value !== -2n) throw "bigint second";

for (let invalidReceiver of [{}, [], Object.create(sample)]) {
  __porfAssertThrows(TypeError, function () { values.call(invalidReceiver); });
  __porfAssertThrows(TypeError, function () { keys.call(invalidReceiver); });
  __porfAssertThrows(TypeError, function () { entries.call(invalidReceiver); });
}

let detachedBeforeCall = new Uint8Array([1]);
__porfDetachArrayBuffer(detachedBeforeCall.buffer);
__porfAssertThrows(TypeError, function () { values.call(detachedBeforeCall); });

let detachedAfterCall = new Uint8Array([1]);
let detachedIterator = values.call(detachedAfterCall);
__porfDetachArrayBuffer(detachedAfterCall.buffer);
__porfAssertThrows(TypeError, function () { detachedIterator.next(); });

let growBuffer = new ArrayBuffer(4, { maxByteLength: 6 });
let tracking = new Uint8Array(growBuffer);
tracking[0] = 1;
tracking[1] = 2;
tracking[2] = 3;
tracking[3] = 4;
let growingIterator = tracking.values();
if (growingIterator.next().value !== 1) throw "grow first";
if (growingIterator.next().value !== 2) throw "grow second";
growBuffer.resize(6);
if (growingIterator.next().value !== 3) throw "grow third";
if (growingIterator.next().value !== 4) throw "grow fourth";
if (growingIterator.next().value !== 0) throw "grow fifth";
if (growingIterator.next().value !== 0) throw "grow sixth";

let shrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let fixed = new Uint8Array(shrinkBuffer, 0, 4);
let shrinkingIterator = fixed.keys();
if (shrinkingIterator.next().value !== 0) throw "shrink first";
shrinkBuffer.resize(3);
__porfAssertThrows(TypeError, function () { shrinkingIterator.next(); });
__porfAssertThrows(TypeError, function () { fixed.keys(); });

let exhaustedBuffer = new ArrayBuffer(2, { maxByteLength: 4 });
let exhaustedView = new Uint8Array(exhaustedBuffer, 1, 1);
let exhaustedIterator = exhaustedView.values();
exhaustedIterator.next();
if (exhaustedIterator.next().done !== true) throw "initial exhausted";
exhaustedBuffer.resize(0);
if (exhaustedIterator.next().done !== true) throw "remains exhausted";

123;
