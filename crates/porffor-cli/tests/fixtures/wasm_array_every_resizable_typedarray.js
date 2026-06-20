let everyDesc = Object.getOwnPropertyDescriptor(Array.prototype, "every");
if (everyDesc.value !== Array.prototype.every) throw "every descriptor value";
if (everyDesc.writable !== true) throw "every descriptor writable";
if (everyDesc.enumerable !== false) throw "every descriptor enumerable";
if (everyDesc.configurable !== true) throw "every descriptor configurable";

let everyLengthDesc = Object.getOwnPropertyDescriptor(Array.prototype.every, "length");
if (Array.prototype.every.length !== 1) throw "every length value";
if (everyLengthDesc.value !== 1) throw "every length descriptor value";
if (everyLengthDesc.writable !== false) throw "every length writable";
if (everyLengthDesc.enumerable !== false) throw "every length enumerable";
if (everyLengthDesc.configurable !== true) throw "every length configurable";

let everyNameDesc = Object.getOwnPropertyDescriptor(Array.prototype.every, "name");
if (Array.prototype.every.name !== "every") throw "every name value";
if (everyNameDesc.value !== "every") throw "every name descriptor value";
if (everyNameDesc.writable !== false) throw "every name writable";
if (everyNameDesc.enumerable !== false) throw "every name enumerable";
if (everyNameDesc.configurable !== true) throw "every name configurable";

function all(array, predicate) {
  return Array.prototype.every.call(array, predicate);
}

function even(value) {
  return value % 2 === 0;
}

function belowSix(value) {
  return value < 6;
}

let rab = new ArrayBuffer(4, { maxByteLength: 8 });
let tracking = new Uint8Array(rab);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}

let fixed = new Uint8Array(rab, 0, 4);
let fixedOffset = new Uint8Array(rab, 2, 2);
let trackingOffset = new Uint8Array(rab, 2);

if (all(fixed, belowSix)) throw "fixed initial false";
if (!all(fixed, even)) throw "fixed initial true";
if (all(fixedOffset, belowSix)) throw "fixed offset initial false";
if (!all(fixedOffset, even)) throw "fixed offset initial true";
if (all(tracking, belowSix)) throw "tracking initial false";
if (!all(tracking, even)) throw "tracking initial true";
if (all(trackingOffset, belowSix)) throw "tracking offset initial false";
if (!all(trackingOffset, even)) throw "tracking offset initial true";

rab.resize(3);
if (!all(fixed, belowSix)) throw "fixed shrink out";
if (!all(fixedOffset, belowSix)) throw "fixed offset shrink out";
if (!all(tracking, belowSix)) throw "tracking shrink";
if (!all(trackingOffset, belowSix)) throw "tracking offset shrink";

rab.resize(1);
if (!all(fixed, belowSix)) throw "fixed shrink one";
if (!all(fixedOffset, belowSix)) throw "fixed offset shrink one";
if (!all(tracking, belowSix)) throw "tracking shrink one";
if (!all(trackingOffset, belowSix)) throw "tracking offset shrink one";

rab.resize(6);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}

if (all(fixed, belowSix)) throw "fixed grow false";
if (!all(fixed, even)) throw "fixed grow true";
if (all(fixedOffset, belowSix)) throw "fixed offset grow false";
if (!all(fixedOffset, even)) throw "fixed offset grow true";
if (all(tracking, belowSix)) throw "tracking grow false";
if (!all(tracking, even)) throw "tracking grow true";
if (all(trackingOffset, belowSix)) throw "tracking offset grow false";
if (!all(trackingOffset, even)) throw "tracking offset grow true";

let midBuffer = new ArrayBuffer(3, { maxByteLength: 4 });
let mid = new Uint8Array(midBuffer);
mid[0] = 10;
mid[1] = 12;
mid[2] = 14;
let seen = [];
let midResult = all(mid, function (value, index) {
  if (index === 0) {
    midBuffer.resize(2);
  }
  seen.push(value);
  return value % 2 === 0;
});

midResult === true && seen.length === 2 && seen[0] === 10 && seen[1] === 12;
