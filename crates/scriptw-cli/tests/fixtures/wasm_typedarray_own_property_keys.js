function assertKeys(actual, expected, label) {
  if (actual.length !== expected.length) throw label + " length";
  for (var i = 0; i < expected.length; i = i + 1) {
    if (actual[i] !== expected[i]) throw label + " key " + i;
  }
}

function integerKeys(length) {
  var keys = [];
  for (var i = 0; i < length; i = i + 1) keys.push(String(i));
  return keys;
}

function assertOrdinaryKeyOrdering(view, label) {
  var symbol = Symbol(label);
  view.visible = 1;
  Object.defineProperty(view, "hidden", {
    value: 2,
    enumerable: false
  });
  view[symbol] = 3;

  var names = integerKeys(view.length);
  names.push("visible");
  names.push("hidden");
  assertKeys(Object.getOwnPropertyNames(view), names, label + " names");

  names.push(symbol);
  assertKeys(Reflect.ownKeys(view), names, label + " reflect");
  if (Object.getOwnPropertyDescriptor(view, "buffer") !== undefined) {
    throw label + " own buffer";
  }
}

assertOrdinaryKeyOrdering(new Uint8Array(3), "numeric");
assertOrdinaryKeyOrdering(new BigInt64Array(2), "bigint");

var numericBuffer = new ArrayBuffer(8);
var numericSubarray = new Uint16Array(numericBuffer).subarray(2);
assertKeys(Reflect.ownKeys(numericSubarray), integerKeys(2), "numeric subarray");
if (numericSubarray.buffer !== numericBuffer) throw "numeric subarray buffer";
if (numericSubarray.byteOffset !== 4) throw "numeric subarray byte offset";

var bigintBuffer = new ArrayBuffer(32);
var bigintSubarray = new BigUint64Array(bigintBuffer).subarray(2);
assertKeys(Reflect.ownKeys(bigintSubarray), integerKeys(2), "bigint subarray");
if (bigintSubarray.buffer !== bigintBuffer) throw "bigint subarray buffer";
if (bigintSubarray.byteOffset !== 16) throw "bigint subarray byte offset";

var trackingBuffer = new ArrayBuffer(4, { maxByteLength: 6 });
var tracking = new Uint8Array(trackingBuffer, 1);
assertKeys(Reflect.ownKeys(tracking), integerKeys(3), "tracking initial");
trackingBuffer.resize(6);
assertKeys(Reflect.ownKeys(tracking), integerKeys(5), "tracking grow");
trackingBuffer.resize(2);
assertKeys(Reflect.ownKeys(tracking), integerKeys(1), "tracking shrink");
trackingBuffer.resize(0);
assertKeys(Reflect.ownKeys(tracking), [], "tracking out of bounds");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 6 });
var fixed = new Uint8Array(fixedBuffer, 1, 2);
assertKeys(Reflect.ownKeys(fixed), integerKeys(2), "fixed initial");
fixedBuffer.resize(6);
assertKeys(Reflect.ownKeys(fixed), integerKeys(2), "fixed grow");
fixedBuffer.resize(2);
assertKeys(Reflect.ownKeys(fixed), [], "fixed out of bounds");

true;
