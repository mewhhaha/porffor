let filterDesc = Object.getOwnPropertyDescriptor(Array.prototype, "filter");
if (filterDesc.value !== Array.prototype.filter) throw "filter descriptor value";
if (filterDesc.writable !== true) throw "filter descriptor writable";
if (filterDesc.enumerable !== false) throw "filter descriptor enumerable";
if (filterDesc.configurable !== true) throw "filter descriptor configurable";

let filterLengthDesc = Object.getOwnPropertyDescriptor(Array.prototype.filter, "length");
if (Array.prototype.filter.length !== 1) throw "filter length value";
if (filterLengthDesc.value !== 1) throw "filter length descriptor value";
if (filterLengthDesc.writable !== false) throw "filter length writable";
if (filterLengthDesc.enumerable !== false) throw "filter length enumerable";
if (filterLengthDesc.configurable !== true) throw "filter length configurable";

let filterNameDesc = Object.getOwnPropertyDescriptor(Array.prototype.filter, "name");
if (Array.prototype.filter.name !== "filter") throw "filter name value";
if (filterNameDesc.value !== "filter") throw "filter name descriptor value";
if (filterNameDesc.writable !== false) throw "filter name writable";
if (filterNameDesc.enumerable !== false) throw "filter name enumerable";
if (filterNameDesc.configurable !== true) throw "filter name configurable";

function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

function keepEven(value) {
  return value !== undefined && value % 2 === 0;
}

let rab = new ArrayBuffer(4, { maxByteLength: 8 });
let tracking = new Uint8Array(rab);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i;
}

let fixed = new Uint8Array(rab, 0, 4);
let fixedOffset = new Uint8Array(rab, 2, 2);
let trackingOffset = new Uint8Array(rab, 2);

if (!sameArray(Array.prototype.filter.call(fixed, keepEven), [0, 2])) {
  throw "fixed initial";
}
if (!sameArray(Array.prototype.filter.call(fixedOffset, keepEven), [2])) {
  throw "fixed offset initial";
}
if (!sameArray(Array.prototype.filter.call(tracking, keepEven), [0, 2])) {
  throw "tracking initial";
}
if (!sameArray(Array.prototype.filter.call(trackingOffset, keepEven), [2])) {
  throw "tracking offset initial";
}

rab.resize(3);
if (!sameArray(Array.prototype.filter.call(fixed, keepEven), [])) {
  throw "fixed shrink";
}
if (!sameArray(Array.prototype.filter.call(fixedOffset, keepEven), [])) {
  throw "fixed offset shrink";
}
if (!sameArray(Array.prototype.filter.call(tracking, keepEven), [0, 2])) {
  throw "tracking shrink";
}
if (!sameArray(Array.prototype.filter.call(trackingOffset, keepEven), [2])) {
  throw "tracking offset shrink";
}

rab.resize(1);
if (!sameArray(Array.prototype.filter.call(tracking, keepEven), [0])) {
  throw "tracking shrink one";
}
if (!sameArray(Array.prototype.filter.call(trackingOffset, keepEven), [])) {
  throw "tracking offset shrink one";
}

rab.resize(6);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i;
}
if (!sameArray(Array.prototype.filter.call(fixed, keepEven), [0, 2])) {
  throw "fixed grow";
}
if (!sameArray(Array.prototype.filter.call(tracking, keepEven), [0, 2, 4])) {
  throw "tracking grow";
}
if (!sameArray(Array.prototype.filter.call(trackingOffset, keepEven), [2, 4])) {
  throw "tracking offset grow";
}

let midBuffer = new ArrayBuffer(3, { maxByteLength: 4 });
let mid = new Uint8Array(midBuffer);
mid[0] = 10;
mid[1] = 11;
mid[2] = 12;
let seen = [];
let midResult = Array.prototype.filter.call(mid, function (value, index) {
  if (index === 0) {
    midBuffer.resize(2);
  }
  seen.push(value);
  return value % 2 === 0;
});

if (!sameArray(seen, [10, 11])) throw "mid seen";
if (!sameArray(midResult, [10])) throw "mid result";

true;
