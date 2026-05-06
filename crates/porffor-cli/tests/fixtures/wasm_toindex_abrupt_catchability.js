function catchesX(fn) {
  try {
    fn();
  } catch (e) {
    return e === "x";
  }
  return false;
}

function throwingIndex() {
  return {
    valueOf: function () {
      throw "x";
    }
  };
}

let score = 0;

if (catchesX(function () {
  new Uint8Array(new ArrayBuffer(8), throwingIndex());
})) score += 1;

if (catchesX(function () {
  new Uint8Array(new ArrayBuffer(8), 0, throwingIndex());
})) score += 1;

let view = new DataView(new ArrayBuffer(8));

if (catchesX(function () {
  view.getUint8(throwingIndex());
})) score += 1;

if (catchesX(function () {
  view.setUint16(throwingIndex(), 1);
})) score += 1;

if (catchesX(function () {
  view.getBigInt64(throwingIndex());
})) score += 1;

if (catchesX(function () {
  new ArrayBuffer(1, { maxByteLength: 4 }).resize(throwingIndex());
})) score += 1;

if (catchesX(function () {
  new ArrayBuffer(1).transfer(throwingIndex());
})) score += 1;

score === 7 ? 123 : score;
