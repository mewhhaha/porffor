let buffer = new ArrayBuffer(8);
let view = new DataView(buffer);
let bytes = new Uint8Array(buffer);

let valueOfIndex = {
  valueOf: function () {
    return 2;
  }
};

let toStringIndex = {
  toString: function () {
    return 3;
  }
};

view.setUint8(0, 10);
view.setUint8(1, 20);
view.setUint8(2, 30);
view.setUint8(3, 40);

if (view.getUint8(-0.999) !== 10) {
  throw new Error("fractional negative index should truncate to zero");
}

if (view.getUint8(valueOfIndex) !== 30) {
  throw new Error("valueOf index should be honored");
}

if (view.getUint8(toStringIndex) !== 40) {
  throw new Error("toString index should be honored");
}

view.setUint8(-0.1, 42);
if (bytes[0] !== 42) {
  throw new Error("setUint8 fractional negative index should write index zero");
}

view.setInt8(valueOfIndex, -1);
if (view.getInt8(valueOfIndex) !== -1) {
  throw new Error("setInt8 valueOf index should write signed byte");
}

let detachedBuffer = new ArrayBuffer(1);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);

__porfAssertThrows(RangeError, function () {
  detachedView.setUint8(Infinity, 0);
});

__porfAssertThrows(RangeError, function () {
  detachedView.setInt8(-1, 0);
});

__porfAssertThrows(TypeError, function () {
  detachedView.setUint8(0, 0);
});

2;
