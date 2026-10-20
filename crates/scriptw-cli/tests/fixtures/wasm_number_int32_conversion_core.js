function fail(label, actual, expected) {
  throw label + ": " + actual + " !== " + expected;
}

function check(actual, expected, label) {
  if (actual !== expected) {
    fail(label, actual, expected);
  }
}

let values = [
  2 ** 63,
  2 ** 63 + 2048,
  -(2 ** 63 + 2048),
  2 ** 52 + 1,
  4294967297,
  -1,
  Infinity,
  NaN,
];

function runtimeValue(index) {
  return values[index];
}

check(Math.clz32(runtimeValue(5)), 0, "clz32 negative residue");
check(Math.clz32(runtimeValue(4)), 31, "clz32 wrapped residue");
check(Math.clz32(runtimeValue(3)), 31, "clz32 exact 2^52 residue");
check(Math.clz32(runtimeValue(1)), 20, "clz32 out-of-i64 residue");
check(Math.clz32(runtimeValue(6)), 32, "clz32 infinity");
check(Math.clz32(runtimeValue(7)), 32, "clz32 NaN");

let trace = "";

function mark(label, value) {
  trace += label;
  return value;
}

function coercible(label, value) {
  return {
    valueOf: function () {
      trace += label;
      return value;
    },
  };
}

let imulResult = Math.imul(
  mark("1", coercible("L", runtimeValue(1))),
  mark("2", coercible("R", 1))
);
check(imulResult, 2048, "imul out-of-i64 residue");
check(trace, "12LR", "imul evaluation and coercion order");
check(Math.imul(runtimeValue(2), 1), -2048, "imul signed interpretation");
check(Math.imul(runtimeValue(6), 1), 0, "imul infinity");
check(Math.imul(runtimeValue(7), 1), 0, "imul NaN");

let buffer = new ArrayBuffer(16);
let view = new DataView(buffer);

view.setUint32(0, runtimeValue(1));
check(view.getUint32(0), 2048, "DataView uint32 residue");
view.setInt32(4, runtimeValue(2));
check(view.getInt32(4), -2048, "DataView int32 signed residue");
view.setUint16(8, runtimeValue(1));
check(view.getUint16(8), 2048, "DataView uint16 residue");
view.setInt16(10, runtimeValue(2));
check(view.getInt16(10), -2048, "DataView int16 signed residue");
view.setUint8(12, runtimeValue(0));
check(view.getUint8(12), 0, "DataView uint8 2^63 conversion");
view.setInt8(13, runtimeValue(0));
check(view.getInt8(13), 0, "DataView int8 2^63 conversion");

trace = "";
view.setUint32(
  mark("1", coercible("I", 0)),
  mark("2", coercible("V", runtimeValue(1))),
  mark("3", false)
);
check(trace, "123IV", "DataView evaluation and coercion order");
check(view.getUint32(0), 2048, "DataView ordered conversion residue");

let uint32 = new Uint32Array([runtimeValue(1), runtimeValue(2)]);
check(uint32[0], 2048, "Uint32Array positive residue");
check(uint32[1], 4294965248, "Uint32Array negative residue");

let int32 = new Int32Array([runtimeValue(1), runtimeValue(2)]);
check(int32[0], 2048, "Int32Array positive residue");
check(int32[1], -2048, "Int32Array signed residue");

let uint16 = new Uint16Array([runtimeValue(1), runtimeValue(2)]);
check(uint16[0], 2048, "Uint16Array positive residue");
check(uint16[1], 63488, "Uint16Array negative residue");

let int16 = new Int16Array([runtimeValue(1), runtimeValue(2)]);
check(int16[0], 2048, "Int16Array positive residue");
check(int16[1], -2048, "Int16Array signed residue");

let uint8 = new Uint8Array([runtimeValue(0)]);
check(uint8[0], 0, "Uint8Array 2^63 conversion");
let int8 = new Int8Array([runtimeValue(0)]);
check(int8[0], 0, "Int8Array 2^63 conversion");

let shared = new SharedArrayBuffer(8);
let atomicUint32 = new Uint32Array(shared, 0, 1);
check(Atomics.store(atomicUint32, 0, runtimeValue(1)), runtimeValue(1), "Atomics.store return");
check(Atomics.load(atomicUint32, 0), 2048, "Atomics.store uint32 residue");
let atomicInt32 = new Int32Array(shared, 4, 1);
Atomics.store(atomicInt32, 0, runtimeValue(2));
check(Atomics.load(atomicInt32, 0), -2048, "Atomics.store signed residue");

true;
