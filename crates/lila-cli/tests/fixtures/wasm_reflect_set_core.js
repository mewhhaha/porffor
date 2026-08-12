var failures = 0;

function fail(bit) {
  failures = failures + bit;
}

var reflectSetDesc = Object.getOwnPropertyDescriptor(Reflect, "set");
if (typeof Reflect.set !== "function") fail(1);
if (reflectSetDesc.value !== Reflect.set) fail(2);
if (reflectSetDesc.writable !== true) fail(4);
if (reflectSetDesc.enumerable !== false) fail(8);
if (reflectSetDesc.configurable !== true) fail(16);

var lengthDesc = Object.getOwnPropertyDescriptor(Reflect.set, "length");
if (Reflect.set.length !== 3) fail(32);
if (lengthDesc.value !== 3) fail(64);
if (lengthDesc.writable !== false) fail(128);
if (lengthDesc.enumerable !== false) fail(256);
if (lengthDesc.configurable !== true) fail(512);

var nameDesc = Object.getOwnPropertyDescriptor(Reflect.set, "name");
if (Reflect.set.name !== "set") fail(1024);
if (nameDesc.value !== "set") fail(2048);
if (nameDesc.writable !== false) fail(4096);
if (nameDesc.enumerable !== false) fail(8192);
if (nameDesc.configurable !== true) fail(16384);

var target = {};
if (Reflect.set(target, "alpha", 41) !== true) fail(32768);
if (target.alpha !== 41) fail(65536);

var receiver = {};
if (Reflect.set(target, "beta", 42, receiver) !== true) fail(131072);
if (target.beta !== undefined) fail(262144);
if (receiver.beta !== 42) fail(524288);

var sym = Symbol("s");
if (Reflect.set(target, sym, 43) !== true) fail(1048576);
if (target[sym] !== 43) fail(2097152);

if (Reflect.set({}, "primitiveReceiver", 44, "receiver") !== false) {
  fail(4194304);
}

var frozenTarget = {};
Object.defineProperty(frozenTarget, "locked", {
  writable: false,
  value: 1
});
if (Reflect.set(frozenTarget, "locked", 2) !== false) fail(8388608);
if (frozenTarget.locked !== 1) fail(16777216);

var lockedReceiverTarget = {};
var lockedReceiver = {};
Object.defineProperty(lockedReceiver, "slot", {
  writable: false,
  value: 3
});
if (Reflect.set(lockedReceiverTarget, "slot", 4, lockedReceiver) !== false) {
  fail(33554432);
}
if (lockedReceiverTarget.slot !== undefined) fail(67108864);
if (lockedReceiver.slot !== 3) fail(134217728);

var accessorReceiverTarget = {};
var accessorReceiver = {};
function receiverSetter(value) {}
Object.defineProperty(accessorReceiver, "slot", {
  set: receiverSetter
});
if (Reflect.set(accessorReceiverTarget, "slot", 5, accessorReceiver) !== false) {
  fail(268435456);
}
var accessorReceiverDesc = Object.getOwnPropertyDescriptor(accessorReceiver, "slot");
if (accessorReceiverDesc.set !== receiverSetter) fail(536870912);
if (accessorReceiverTarget.slot !== undefined) fail(1073741824);

var accessorTarget = {};
var accessorCallCount = 0;
var accessorThis;
var accessorValue;
var explicitReceiver = {};
Object.defineProperty(accessorTarget, "slot", {
  set: function(value) {
    accessorCallCount = accessorCallCount + 1;
    accessorThis = this;
    accessorValue = value;
  }
});
if (Reflect.set(accessorTarget, "slot", 6, explicitReceiver) !== true) {
  fail(2147483648);
}
if (accessorCallCount !== 1) fail(4294967296);
if (accessorThis !== explicitReceiver) fail(8589934592);
if (accessorValue !== 6) fail(17179869184);

var nonwritableLengthSource = [];
Object.defineProperty(nonwritableLengthSource, "length", { writable: false });
var lengthReceiver = {};
if (Reflect.set(nonwritableLengthSource, "length", 1, lengthReceiver) !== false) {
  fail(35184372088832);
}
if (Object.prototype.hasOwnProperty.call(lengthReceiver, "length")) {
  fail(70368744177664);
}

var writableLengthSource = [];
var ordinaryLengthReceiver = {};
if (Reflect.set(writableLengthSource, "length", 1, ordinaryLengthReceiver) !== true) {
  fail(140737488355328);
}
if (ordinaryLengthReceiver.length !== 1) fail(281474976710656);

var nonwritableLengthReceiver = [];
Object.defineProperty(nonwritableLengthReceiver, "length", { writable: false });
if (Reflect.set(writableLengthSource, "length", 1, nonwritableLengthReceiver) !== false) {
  fail(562949953421312);
}
if (nonwritableLengthReceiver.length !== 0) fail(1125899906842624);

var threw = false;
try {
  Reflect.set(null, "x", 1);
} catch (error) {
  if (!(error instanceof TypeError)) fail(34359738368);
  threw = true;
}
if (!threw) fail(68719476736);

var trapCalls = 0;
var seenTarget = false;
var seenKey = false;
var seenValue = false;
var seenReceiver = false;
var proxy = new Proxy(target, {
  set(t, k, v, r) {
    trapCalls = trapCalls + 1;
    if (t === target) seenTarget = true;
    if (k === "viaProxy") seenKey = true;
    if (v === 55) seenValue = true;
    if (r === proxy) seenReceiver = true;
    t[k] = v + 1;
    return "truthy";
  }
});

if (Reflect.set(proxy, "viaProxy", 55) !== true) fail(8796093022208);
if (target.viaProxy !== 56) fail(137438953472);
if (trapCalls !== 1) fail(274877906944);
if (seenTarget !== true) fail(549755813888);
if (seenKey !== true) fail(1099511627776);
if (seenValue !== true) fail(2199023255552);
if (seenReceiver !== true) fail(4398046511104);

var nonExtensibleArray = [1, 2];
Object.preventExtensions(nonExtensibleArray);
var existingIndexSet = Reflect.set(nonExtensibleArray, "0", 3);
var newIndexSet = Reflect.set(nonExtensibleArray, "2", 4);
var namedArraySet = Reflect.set(nonExtensibleArray, "named", 5);
var arraySymbolKey = Symbol("non-extensible-array");
var symbolArraySet = Reflect.set(nonExtensibleArray, arraySymbolKey, 6);
var sparseArray = [];
sparseArray[1000001] = 1;
Object.preventExtensions(sparseArray);
var existingSparseSet = Reflect.set(sparseArray, "1000001", 2);
var newSparseSet = Reflect.set(sparseArray, "1000002", 3);
if (
  existingIndexSet !== true ||
  nonExtensibleArray[0] !== 3 ||
  newIndexSet !== false ||
  nonExtensibleArray.length !== 2 ||
  namedArraySet !== false ||
  Object.prototype.hasOwnProperty.call(nonExtensibleArray, "named") ||
  symbolArraySet !== false ||
  Object.prototype.hasOwnProperty.call(nonExtensibleArray, arraySymbolKey) ||
  existingSparseSet !== true ||
  sparseArray[1000001] !== 2 ||
  newSparseSet !== false ||
  sparseArray[1000002] !== undefined ||
  sparseArray.length !== 1000002
) {
  fail(2251799813685248);
}

var descriptorArray = [1];
Object.defineProperty(descriptorArray, "0", {
  value: 1,
  writable: true,
  enumerable: false,
  configurable: false
});
var descriptorIndexSet = Reflect.set(descriptorArray, "0", 2);
var preservedIndexDesc = Object.getOwnPropertyDescriptor(descriptorArray, "0");
var fixedLengthArray = [];
Object.defineProperty(fixedLengthArray, "length", { writable: false });
if (
  descriptorIndexSet !== true ||
  descriptorArray[0] !== 2 ||
  preservedIndexDesc.writable !== true ||
  preservedIndexDesc.enumerable !== false ||
  preservedIndexDesc.configurable !== false ||
  Reflect.set(fixedLengthArray, "0", 1) !== false ||
  fixedLengthArray.length !== 0
) {
  fail(4503599627370496);
}

failures === 0;
