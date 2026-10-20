function sameArray(actual, expected, label) {
  if (actual.length !== expected.length) throw label + " length " + actual.length;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) throw label + " value " + i;
  }
}

let rab = new ArrayBuffer(4, { maxByteLength: 8 });
let tracking = new Uint8Array(rab);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}

sameArray(Array.prototype.map.call(tracking, function (value) { return value + 1; }), [1, 3, 5, 7], "initial");

rab.resize(2);
sameArray(Array.prototype.map.call(tracking, function (value) { return value + 1; }), [1, 3], "shrink");

rab.resize(6);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}
sameArray(Array.prototype.map.call(tracking, function (value) { return value + 1; }), [1, 3, 5, 7, 9, 11], "grow");

let fixed = new Uint8Array(rab, 2, 2);
rab.resize(3);
sameArray(Array.prototype.map.call(fixed, function (value) { return value + 1; }), [], "fixed out");

rab.resize(4);
// The byte discarded by the shrink is zero-filled when the buffer grows again.
sameArray(Array.prototype.map.call(fixed, function (value) { return value + 1; }), [5, 1], "fixed in");

let midBuffer = new ArrayBuffer(3, { maxByteLength: 4 });
let mid = new Uint8Array(midBuffer);
mid[0] = 10;
mid[1] = 11;
mid[2] = 12;
let seen = [];
let result = Array.prototype.map.call(mid, function (value, index) {
  if (index === 0) {
    midBuffer.resize(2);
  }
  seen.push(value);
  return index;
});

sameArray(seen, [10, 11], "mid seen");
if (result.length !== 3) throw "mid result length";
if (result[0] !== 0) throw "mid result 0";
if (result[1] !== 1) throw "mid result 1";
if (result[2] !== undefined) throw "mid result 2";

true;
