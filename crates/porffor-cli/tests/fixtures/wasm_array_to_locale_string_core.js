"use strict";

let failures = 0;

if (typeof Array.prototype.toLocaleString !== "function") failures |= 1;

let o = {
  length: 2,
  0: 7,
  1: {
    toLocaleString: function () {
      return "baz";
    }
  }
};
if (Array.prototype.toLocaleString.call(o) !== "7,baz") failures |= 2;

o = {};
if (Array.prototype.toLocaleString.call(o) !== "") failures |= 4;

let log = "";
let arr = {
  length: {
    valueOf: function () {
      log += "L";
      return 2;
    }
  },
  0: "x",
  1: "z"
};
if (Array.prototype.toLocaleString.call(arr) !== "x,z") failures |= 8;
if (log !== "L") failures |= 16;

if ([7, {
  toLocaleString: function () {
    return "baz";
  }
}].toLocaleString() !== "7,baz") failures |= 32;

let separator = ["", ""].toLocaleString();
Boolean.prototype.toLocaleString = function () {
  return typeof this;
};
if ([true, false].toLocaleString() !== "boolean" + separator + "boolean") failures |= 64;

let unique = {
  toString: function () {
    return "ignored";
  }
};
let testCases = [
  { label: "none", args: [] },
  { label: "undefined", args: [undefined] },
  { label: "string", args: ["ar"] },
  { label: "object", args: [unique] },
  { label: "pair", args: ["zh", unique] },
  { label: "extra", args: [unique, unique, unique] },
];

for (const { label, args } of testCases) {
  if ([undefined].toLocaleString(...args) !== "") failures |= 128;
  if ([null].toLocaleString(...args) !== "") failures |= 256;

  let spy = {
    toLocaleString: function (...receivedArgs) {
      let captured = "case:" + label;
      if (captured.length < 6) return "bad";
      return String(receivedArgs.length);
    }
  };
  if ([spy].toLocaleString(...args) !== "0") failures |= 512;
}

let rab = new ArrayBuffer(4, { maxByteLength: 8 });
let tracking = new Uint8Array(rab);
for (let i = 0; i < 4; i = i + 1) {
  tracking[i] = i + 1;
}
if (Array.prototype.toLocaleString.call(tracking) !== "1,2,3,4") failures |= 1024;
if (tracking.toLocaleString() !== "1,2,3,4") failures |= 8192;

rab.resize(2);
if (Array.prototype.toLocaleString.call(tracking) !== "1,2") failures |= 2048;
if (tracking.toLocaleString() !== "1,2") failures |= 16384;

let fixedRab = new ArrayBuffer(4, { maxByteLength: 8 });
let fixed = new Uint8Array(fixedRab, 0, 4);
fixed[0] = 1;
fixed[1] = 2;
fixed[2] = 3;
fixed[3] = 4;
fixedRab.resize(2);
if (Array.prototype.toLocaleString.call(fixed) !== "") failures |= 4096;
let fixedToLocaleStringThrew = false;
try {
  fixed.toLocaleString();
} catch (e) {
  fixedToLocaleStringThrew = e instanceof TypeError;
}
if (!fixedToLocaleStringThrew) failures |= 32768;

failures === 0;
