function check(value, label) {
  if (!value) throw label;
}

let includesDesc = Object.getOwnPropertyDescriptor(Array.prototype, "includes");
check(typeof Array.prototype.includes === "function", "includes function");
check(includesDesc.value === Array.prototype.includes, "includes desc value");
check(includesDesc.writable === true, "includes writable");
check(includesDesc.enumerable === false, "includes enumerable");
check(includesDesc.configurable === true, "includes configurable");

let includesLengthDesc = Object.getOwnPropertyDescriptor(Array.prototype.includes, "length");
check(Array.prototype.includes.length === 1, "includes length value");
check(includesLengthDesc.value === 1, "includes length desc value");
check(includesLengthDesc.writable === false, "includes length writable");
check(includesLengthDesc.enumerable === false, "includes length enumerable");
check(includesLengthDesc.configurable === true, "includes length configurable");

let includesNameDesc = Object.getOwnPropertyDescriptor(Array.prototype.includes, "name");
check(Array.prototype.includes.name === "includes", "includes name value");
check(includesNameDesc.value === "includes", "includes name desc value");
check(includesNameDesc.writable === false, "includes name writable");
check(includesNameDesc.enumerable === false, "includes name enumerable");
check(includesNameDesc.configurable === true, "includes name configurable");

let rab = new ArrayBuffer(4, { maxByteLength: 8 });
let tracking = new Uint8Array(rab);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}

let fixed = new Uint8Array(rab, 0, 4);
let fixedOffset = new Uint8Array(rab, 2, 2);
let trackingOffset = new Uint8Array(rab, 2);

check(Array.prototype.includes.call(fixed, 2), "fixed initial");
check(!Array.prototype.includes.call(fixed, undefined), "fixed no undefined");
check(Array.prototype.includes.call(fixed, 2, 1), "fixed from index");
check(!Array.prototype.includes.call(fixed, 2, 2), "fixed from index miss");
check(!Array.prototype.includes.call(fixedOffset, 2), "fixed offset miss");
check(Array.prototype.includes.call(fixedOffset, 4), "fixed offset hit");
check(Array.prototype.includes.call(tracking, 4), "tracking initial");
check(Array.prototype.includes.call(trackingOffset, 4), "tracking offset initial");

rab.resize(3);
check(!Array.prototype.includes.call(fixed, 2), "fixed shrink out");
check(!Array.prototype.includes.call(fixedOffset, 4), "fixed offset shrink out");
check(Array.prototype.includes.call(tracking, 2), "tracking shrink");
check(Array.prototype.includes.call(trackingOffset, 4), "tracking offset shrink");

rab.resize(1);
check(!Array.prototype.includes.call(tracking, 2), "tracking shrink miss");
check(!Array.prototype.includes.call(trackingOffset, 4), "tracking offset shrink miss");

rab.resize(6);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}
check(Array.prototype.includes.call(fixed, 2), "fixed grow");
check(!Array.prototype.includes.call(fixed, 8), "fixed grow stale");
check(Array.prototype.includes.call(tracking, 8), "tracking grow");
check(Array.prototype.includes.call(trackingOffset, 8), "tracking offset grow");

let coercedBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let coercedFixed = new Uint8Array(coercedBuffer, 0, 4);
coercedFixed[0] = 7;
let shrinkForUndefined = {
  valueOf: function () {
    coercedBuffer.resize(2);
    return 0;
  }
};
check(Array.prototype.includes.call(coercedFixed, undefined, shrinkForUndefined), "coerced undefined");

coercedBuffer.resize(4);
coercedFixed[0] = 7;
let shrinkForNumber = {
  valueOf: function () {
    coercedBuffer.resize(2);
    return 0;
  }
};
check(!Array.prototype.includes.call(coercedFixed, 7, shrinkForNumber), "coerced number");

let floatBuffer = new ArrayBuffer(32, { maxByteLength: 64 });
let floats = new Float64Array(floatBuffer);
floats[0] = -Infinity;
floats[1] = Infinity;
floats[2] = NaN;
floats[3] = -0;
check(Array.prototype.includes.call(floats, -Infinity), "negative infinity");
check(Array.prototype.includes.call(floats, Infinity), "positive infinity");
check(Array.prototype.includes.call(floats, NaN), "nan");
check(Array.prototype.includes.call(floats, 0), "same value zero");

true;
