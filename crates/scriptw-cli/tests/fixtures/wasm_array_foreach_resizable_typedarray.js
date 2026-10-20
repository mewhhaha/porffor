function collect(array) {
  let values = [];
  Array.prototype.forEach.call(array, function (value) {
    values.push(value);
  });
  return values;
}

function same(values, expected) {
  if (values.length !== expected.length) return false;
  for (let i = 0; i < values.length; i++) {
    if (values[i] !== expected[i]) return false;
  }
  return true;
}

let rab = new ArrayBuffer(4, { maxByteLength: 8 });
let tracking = new Uint8Array(rab);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}

let fixed = new Uint8Array(rab, 0, 4);
let fixedOffset = new Uint8Array(rab, 2, 2);
let trackingOffset = new Uint8Array(rab, 2);

if (!same(collect(fixed), [0, 2, 4, 6])) throw "fixed initial";
if (!same(collect(fixedOffset), [4, 6])) throw "fixed offset initial";
if (!same(collect(tracking), [0, 2, 4, 6])) throw "tracking initial";
if (!same(collect(trackingOffset), [4, 6])) throw "tracking offset initial";

rab.resize(3);
if (!same(collect(fixed), [])) throw "fixed shrink out";
if (!same(collect(fixedOffset), [])) throw "fixed offset shrink out";
if (!same(collect(tracking), [0, 2, 4])) throw "tracking shrink";
if (!same(collect(trackingOffset), [4])) throw "tracking offset shrink";

rab.resize(1);
if (!same(collect(fixed), [])) throw "fixed shrink one";
if (!same(collect(fixedOffset), [])) throw "fixed offset shrink one";
if (!same(collect(tracking), [0])) throw "tracking shrink one";
if (!same(collect(trackingOffset), [])) throw "tracking offset shrink one";

rab.resize(6);
for (let i = 0; i < tracking.length; i++) {
  tracking[i] = i * 2;
}

if (!same(collect(fixed), [0, 2, 4, 6])) throw "fixed grow";
if (!same(collect(fixedOffset), [4, 6])) throw "fixed offset grow";
if (!same(collect(tracking), [0, 2, 4, 6, 8, 10])) throw "tracking grow";
if (!same(collect(trackingOffset), [4, 6, 8, 10])) throw "tracking offset grow";

let midBuffer = new ArrayBuffer(3, { maxByteLength: 4 });
let mid = new Uint8Array(midBuffer);
mid[0] = 10;
mid[1] = 11;
mid[2] = 12;
let seen = [];
Array.prototype.forEach.call(mid, function (value, index) {
  if (index === 0) {
    midBuffer.resize(2);
  }
  seen.push(value);
});

same(seen, [10, 11]);
