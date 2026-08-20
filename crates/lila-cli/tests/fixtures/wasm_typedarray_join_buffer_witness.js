let typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
let join = typedArrayPrototype.join;

if (new Uint8Array([1, 2, 3]).join("-") !== "1-2-3") {
  throw "ordinary join";
}
if (new BigInt64Array([1n, -2n, 3n]).join("|") !== "1|-2|3") {
  throw "BigInt join";
}

function assertTypeErrorPrototype(thunk, prototype, label) {
  let error;
  try {
    thunk();
  } catch (caught) {
    error = caught;
  }
  if (error === undefined) throw label + " missing throw";
  if (Object.getPrototypeOf(error) !== prototype) {
    throw label + " wrong TypeError realm";
  }
}

let separatorCalls = 0;
let unusedSeparator = {
  toString: function () {
    separatorCalls = separatorCalls + 1;
    return ".";
  },
};
assertTypeErrorPrototype(function () {
  join.call({}, unusedSeparator);
}, TypeError.prototype, "invalid receiver");
if (separatorCalls !== 0) throw "invalid receiver separator order";

let detached = new Uint8Array([1]);
__lilaDetachArrayBuffer(detached.buffer);
assertTypeErrorPrototype(function () {
  join.call(detached, unusedSeparator);
}, TypeError.prototype, "detached receiver");
if (separatorCalls !== 0) throw "detached separator order";

let fixedBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let fixed = new Uint8Array(fixedBuffer, 0, 4);
fixed[0] = 1;
fixed[1] = 2;
fixed[2] = 3;
fixed[3] = 4;
fixedBuffer.resize(3);
assertTypeErrorPrototype(function () {
  join.call(fixed, unusedSeparator);
}, TypeError.prototype, "fixed out of bounds");
if (separatorCalls !== 0) throw "out-of-bounds separator order";
fixedBuffer.resize(4);
if (fixed.join() !== "1,2,3,0") throw "fixed regrow";

let trackingBuffer = new ArrayBuffer(4, { maxByteLength: 6 });
let tracking = new Uint8Array(trackingBuffer);
tracking[0] = 1;
tracking[1] = 2;
tracking[2] = 3;
tracking[3] = 4;
trackingBuffer.resize(6);
if (tracking.join() !== "1,2,3,4,0,0") throw "tracking grow";
trackingBuffer.resize(2);
if (tracking.join() !== "1,2") throw "tracking shrink";

let oddBuffer = new ArrayBuffer(4, { maxByteLength: 5 });
let odd = new Uint16Array(oddBuffer);
odd[0] = 7;
odd[1] = 9;
oddBuffer.resize(5);
if (odd.join(":") !== "7:9") throw "odd-byte element floor";

let separatorBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let separatorFixed = new Uint8Array(separatorBuffer, 0, 4);
let shrinkingSeparator = {
  toString: function () {
    separatorBuffer.resize(2);
    return ".";
  },
};
if (separatorFixed.join(shrinkingSeparator) !== "...") {
  throw "fixed separator shrink snapshot";
}

let trackingSeparatorBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let separatorTracking = new Uint8Array(trackingSeparatorBuffer);
separatorTracking[0] = 1;
separatorTracking[1] = 2;
separatorTracking[2] = 3;
separatorTracking[3] = 4;
let trackingShrinkingSeparator = {
  toString: function () {
    trackingSeparatorBuffer.resize(2);
    return ".";
  },
};
if (separatorTracking.join(trackingShrinkingSeparator) !== "1.2..") {
  throw "tracking separator shrink snapshot";
}

let other = __lilaCreateRealm().global;
let otherJoin = other.Uint8Array.prototype.join;

let otherDetached = new other.Uint8Array(1);
__lilaDetachArrayBuffer(otherDetached.buffer);
assertTypeErrorPrototype(function () {
  otherJoin.call(otherDetached);
}, other.TypeError.prototype, "other method and receiver");

let entryDetachedForOtherJoin = new Uint8Array(1);
__lilaDetachArrayBuffer(entryDetachedForOtherJoin.buffer);
assertTypeErrorPrototype(function () {
  otherJoin.call(entryDetachedForOtherJoin);
}, other.TypeError.prototype, "other method entry receiver");

let otherDetachedForEntryJoin = new other.Uint8Array(1);
__lilaDetachArrayBuffer(otherDetachedForEntryJoin.buffer);
assertTypeErrorPrototype(function () {
  join.call(otherDetachedForEntryJoin);
}, TypeError.prototype, "entry method other receiver");

let otherFixedBuffer = new other.ArrayBuffer(2, { maxByteLength: 2 });
let otherFixed = new other.Uint8Array(otherFixedBuffer, 0, 2);
ArrayBuffer.prototype.resize.call(otherFixedBuffer, 1);
assertTypeErrorPrototype(function () {
  otherJoin.call(otherFixed);
}, other.TypeError.prototype, "other out of bounds");

123;
