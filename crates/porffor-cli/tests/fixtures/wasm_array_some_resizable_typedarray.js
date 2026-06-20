let someDesc = Object.getOwnPropertyDescriptor(Array.prototype, "some");
if (someDesc.value !== Array.prototype.some) throw "some descriptor value";
if (someDesc.writable !== true) throw "some descriptor writable";
if (someDesc.enumerable !== false) throw "some descriptor enumerable";
if (someDesc.configurable !== true) throw "some descriptor configurable";

let someLengthDesc = Object.getOwnPropertyDescriptor(Array.prototype.some, "length");
if (Array.prototype.some.length !== 1) throw "some length value";
if (someLengthDesc.value !== 1) throw "some length descriptor value";
if (someLengthDesc.writable !== false) throw "some length writable";
if (someLengthDesc.enumerable !== false) throw "some length enumerable";
if (someLengthDesc.configurable !== true) throw "some length configurable";

let someNameDesc = Object.getOwnPropertyDescriptor(Array.prototype.some, "name");
if (Array.prototype.some.name !== "some") throw "some name value";
if (someNameDesc.value !== "some") throw "some name descriptor value";
if (someNameDesc.writable !== false) throw "some name writable";
if (someNameDesc.enumerable !== false) throw "some name enumerable";
if (someNameDesc.configurable !== true) throw "some name configurable";

let rab = new ArrayBuffer(4, { maxByteLength: 8 });
let tracking = new Uint8Array(rab);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}

if (!Array.prototype.some.call(tracking, function (value) { return value === 4; })) {
  throw "initial true";
}
if (Array.prototype.some.call(tracking, function (value) { return value > 9; })) {
  throw "initial false";
}

rab.resize(2);
if (!Array.prototype.some.call(tracking, function (value) { return value === 2; })) {
  throw "shrink true";
}
if (Array.prototype.some.call(tracking, function (value) { return value === 4; })) {
  throw "shrink stale";
}

rab.resize(6);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}
if (!Array.prototype.some.call(tracking, function (value) { return value === 10; })) {
  throw "grow true";
}

let fixed = new Uint8Array(rab, 2, 2);
rab.resize(3);
if (Array.prototype.some.call(fixed, function () { return true; })) {
  throw "fixed out";
}

rab.resize(4);
if (!Array.prototype.some.call(fixed, function (value) { return value === 4; })) {
  throw "fixed in";
}

let midBuffer = new ArrayBuffer(3, { maxByteLength: 4 });
let mid = new Uint8Array(midBuffer);
mid[0] = 10;
mid[1] = 11;
mid[2] = 12;
let seen = [];
let result = Array.prototype.some.call(mid, function (value, index) {
  if (index === 0) {
    midBuffer.resize(2);
  }
  seen.push(value);
  return false;
});

if (result !== false) throw "mid result";
if (seen.length !== 2) throw "mid seen length";
if (seen[0] !== 10) throw "mid seen 0";
if (seen[1] !== 11) throw "mid seen 1";

true;
