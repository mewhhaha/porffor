let ctors = [
  Float64Array,
  Float32Array,
  Int32Array,
  Int16Array,
  Int8Array,
  Uint32Array,
  Uint16Array,
  Uint8Array,
  Uint8ClampedArray
];

for (let i = 0; i < ctors.length; i = i + 1) {
  let C = ctors[i];
  if (!ArrayBuffer.isView(new C())) throw "base typed array";

  class TA extends C {}
  if (!ArrayBuffer.isView(new TA())) throw "typed array subclass";
}

if (!ArrayBuffer.isView(new DataView(new ArrayBuffer(8)))) throw "dataview";

if (ArrayBuffer.isView(new ArrayBuffer(8))) throw "arraybuffer";
if (ArrayBuffer.isView({})) throw "plain object";
if (ArrayBuffer.isView(0)) throw "number primitive";
if (ArrayBuffer.isView(undefined)) throw "undefined primitive";
if (ArrayBuffer.isView("x")) throw "string primitive";

class O {}
if (ArrayBuffer.isView(new O())) throw "unrelated subclass";

123;
