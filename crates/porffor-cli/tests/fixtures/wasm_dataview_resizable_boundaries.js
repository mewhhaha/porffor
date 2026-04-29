let buffer = new ArrayBuffer(8, { maxByteLength: 16 });
let auto = new DataView(buffer, 2);
let fixed = new DataView(buffer, 2, 4);

if (auto.byteOffset !== 2) throw "auto initial byteOffset";
if (auto.byteLength !== 6) throw "auto initial byteLength";
if (fixed.byteOffset !== 2) throw "fixed initial byteOffset";
if (fixed.byteLength !== 4) throw "fixed initial byteLength";

auto.setUint8(0, 0x11);
auto.setUint16(1, 0x2233);
fixed.setUint8(3, 0x44);

buffer.resize(12);

if (buffer.detached !== false) throw "grow detached";
if (buffer.byteLength !== 12) throw "grow byteLength";
if (buffer.maxByteLength !== 16) throw "grow maxByteLength";
if (auto.byteOffset !== 2) throw "auto grow byteOffset";
if (auto.byteLength !== 10) throw "auto grow byteLength";
if (fixed.byteOffset !== 2) throw "fixed grow byteOffset";
if (fixed.byteLength !== 4) throw "fixed grow byteLength";
if (auto.getUint8(0) !== 0x11) throw "auto grow getUint8";
if (auto.getUint16(1) !== 0x2233) throw "auto grow getUint16";
if (fixed.getUint8(3) !== 0x44) throw "fixed grow getUint8";

auto.setUint8(5, 0x55);
fixed.setUint16(0, 0x6677);
if (fixed.getUint16(0) !== 0x6677) throw "fixed grow getUint16";

buffer.resize(4);

if (buffer.detached !== false) throw "shrink detached";
if (buffer.byteLength !== 4) throw "shrink byteLength";
if (auto.byteOffset !== 2) throw "auto shrink byteOffset";
if (auto.byteLength !== 2) throw "auto shrink byteLength";

try {
  fixed.byteOffset;
  throw "fixed shrink byteOffset did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "fixed shrink byteOffset error";
}

try {
  fixed.byteLength;
  throw "fixed shrink byteLength did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "fixed shrink byteLength error";
}

try {
  fixed.getUint8(0);
  throw "fixed shrink getUint8 did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "fixed shrink getUint8 error";
}

try {
  fixed.setUint8(0, 1);
  throw "fixed shrink setUint8 did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "fixed shrink setUint8 error";
}

try {
  fixed.getUint16(0);
  throw "fixed shrink getUint16 did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "fixed shrink getUint16 error";
}

try {
  fixed.setUint16(0, 1);
  throw "fixed shrink setUint16 did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "fixed shrink setUint16 error";
}

try {
  auto.getUint8(2);
  throw "auto shrink getUint8 did not throw";
} catch (error) {
  if (!(error instanceof RangeError)) throw "auto shrink getUint8 error";
}

try {
  auto.setUint8(2, 1);
  throw "auto shrink setUint8 did not throw";
} catch (error) {
  if (!(error instanceof RangeError)) throw "auto shrink setUint8 error";
}

try {
  auto.getUint16(1);
  throw "auto shrink getUint16 did not throw";
} catch (error) {
  if (!(error instanceof RangeError)) throw "auto shrink getUint16 error";
}

try {
  auto.setUint16(1, 1);
  throw "auto shrink setUint16 did not throw";
} catch (error) {
  if (!(error instanceof RangeError)) throw "auto shrink setUint16 error";
}

let boundary = new DataView(buffer, 4);
if (boundary.byteOffset !== 4) throw "boundary byteOffset";
if (boundary.byteLength !== 0) throw "boundary byteLength";
try {
  boundary.getUint8(0);
  throw "boundary getUint8 did not throw";
} catch (error) {
  if (!(error instanceof RangeError)) throw "boundary getUint8 error";
}

let proto = { marker: 1 };
let customNewTarget = function () {};
let prototypeGetters = 0;
Object.defineProperty(customNewTarget, "prototype", {
  get: function () {
    prototypeGetters += 1;
    buffer.resize(10);
    return proto;
  }
});
let custom = Reflect.construct(DataView, [buffer, 0], customNewTarget);
if (prototypeGetters !== 1) throw "prototype getter count";
if (Object.getPrototypeOf(custom) !== proto) throw "custom prototype";
if (buffer.byteLength !== 10) throw "custom resized byteLength";
let afterCustom = new DataView(buffer, 0, 8);
if (afterCustom.byteLength !== 8) throw "custom constructor length";
afterCustom.setUint8(7, 0x88);
if (afterCustom.getUint8(7) !== 0x88) throw "custom get after resize";

let short = new ArrayBuffer(3, { maxByteLength: 3 });
let boundNewTarget = function () {}.bind(null);
Object.defineProperty(boundNewTarget, "prototype", {
  get: function () {
    try {
      short.resize(2);
    } catch (error) {}
  }
});
let boundResult = Reflect.construct(DataView, [short, 1, 1], boundNewTarget);
if (boundResult.constructor !== DataView) throw "bound constructor";
if (boundResult.byteLength !== 1) throw "bound byteLength";

let floatBuffer = new ArrayBuffer(16, { maxByteLength: 32 });
let floatFixed = new DataView(floatBuffer, 4, 8);
floatFixed.setFloat64(0, 13.5);
floatBuffer.resize(20);
if (floatFixed.getFloat64(0) !== 13.5) throw "float fixed grow getFloat64";
floatBuffer.resize(12);
floatFixed.setFloat64(0, 27.25);
if (floatFixed.getFloat64(0) !== 27.25) throw "float fixed in-bounds shrink";
let floatAuto = new DataView(floatBuffer, 0);
floatBuffer.resize(11);
try {
  floatFixed.getFloat64(0);
  throw "float fixed shrink getFloat64 did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "float fixed shrink getFloat64 error";
}
try {
  floatFixed.setFloat64(0, 1);
  throw "float fixed shrink setFloat64 did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "float fixed shrink setFloat64 error";
}
try {
  floatAuto.getFloat64(4);
  throw "float auto getFloat64 did not throw";
} catch (error) {
  if (!(error instanceof RangeError)) throw "float auto getFloat64 error";
}
try {
  floatAuto.setFloat64(4, 1);
  throw "float auto setFloat64 did not throw";
} catch (error) {
  if (!(error instanceof RangeError)) throw "float auto setFloat64 error";
}

let bigBuffer = new ArrayBuffer(16, { maxByteLength: 24 });
let bigFixed = new DataView(bigBuffer, 8, 8);
bigFixed.setBigUint64(0, 0x0102030405060708n);
bigBuffer.resize(24);
if (bigFixed.getBigUint64(0) !== 0x0102030405060708n) throw "big fixed grow getBigUint64";
bigBuffer.resize(16);
bigFixed.setBigUint64(0, 0x1112131415161718n);
if (bigFixed.getBigUint64(0) !== 0x1112131415161718n) throw "big fixed in-bounds shrink";
let bigAuto = new DataView(bigBuffer, 0);
bigBuffer.resize(15);
try {
  bigFixed.getBigUint64(0);
  throw "big fixed shrink getBigUint64 did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "big fixed shrink getBigUint64 error";
}
try {
  bigFixed.setBigUint64(0, 1n);
  throw "big fixed shrink setBigUint64 did not throw";
} catch (error) {
  if (!(error instanceof TypeError)) throw "big fixed shrink setBigUint64 error";
}
try {
  bigAuto.getBigUint64(8);
  throw "big auto getBigUint64 did not throw";
} catch (error) {
  if (!(error instanceof RangeError)) throw "big auto getBigUint64 error";
}

123;
