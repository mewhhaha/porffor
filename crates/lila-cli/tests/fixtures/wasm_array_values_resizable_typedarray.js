function sameArray(actual, expected, label) {
  if (actual.length !== expected.length) throw label + " length " + actual.length;
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) throw label + " value " + i;
  }
}

function collectAndResize(iterator, expected, rab, resizeAfter, resizeTo, label) {
  let values = [];
  let resized = false;
  while (true) {
    let next = iterator.next();
    if (next.done) break;
    values.push(next.value);
    if (!resized && values.length === resizeAfter) {
      rab.resize(resizeTo);
      resized = true;
    }
  }
  if (!resized) throw label + " did not resize";
  sameArray(values, expected, label);
}

function throwsTypeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (e) {
    threw = e && e.name === "TypeError";
  }
  if (!threw) throw label + " did not throw TypeError";
}

let rab = new ArrayBuffer(4, { maxByteLength: 6 });
let write = new Uint8Array(rab);
for (let i = 0; i < 4; i++) write[i] = i * 2;

let fixed = new Uint8Array(rab, 0, 4);
throwsTypeError(function () {
  collectAndResize(Array.prototype.values.call(fixed), [], rab, 2, 3, "fixed shrink");
}, "fixed shrink");

rab = new ArrayBuffer(4, { maxByteLength: 6 });
write = new Uint8Array(rab);
for (let i = 0; i < 4; i++) write[i] = i * 2;

let fixedOffset = new Uint8Array(rab, 2, 2);
throwsTypeError(function () {
  collectAndResize(Array.prototype.values.call(fixedOffset), [], rab, 2, 3, "fixed offset shrink");
}, "fixed offset shrink");

rab = new ArrayBuffer(4, { maxByteLength: 6 });
write = new Uint8Array(rab);
for (let i = 0; i < 4; i++) write[i] = i * 2;

let tracking = new Uint8Array(rab, 0);
collectAndResize(Array.prototype.values.call(tracking), [0, 2, 4], rab, 2, 3, "tracking shrink");

rab = new ArrayBuffer(4, { maxByteLength: 6 });
write = new Uint8Array(rab);
for (let i = 0; i < 4; i++) write[i] = i * 2;

let trackingOffset = new Uint8Array(rab, 2);
collectAndResize(Array.prototype.values.call(trackingOffset), [4, 6], rab, 2, 3, "tracking offset shrink");

true;
