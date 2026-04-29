let buffer = new ArrayBuffer(8);
let sab = new SharedArrayBuffer(8);

let view = new DataView(buffer, 1, undefined);
if (view.byteLength !== 7) throw "undefined byteLength";
if (view.byteOffset !== 1) throw "byteOffset";
if (view.buffer !== buffer) throw "buffer";

let shared = new DataView(sab, 1, undefined);
if (shared.byteLength !== 7) throw "sab undefined byteLength";
if (shared.byteOffset !== 1) throw "sab byteOffset";
if (shared.buffer !== sab) throw "sab buffer";

if (new DataView(buffer, NaN).byteOffset !== 0) throw "NaN offset";
if (new DataView(buffer, "2").byteOffset !== 2) throw "string offset";
if (new DataView(buffer, true).byteOffset !== 1) throw "boolean offset";
if (new DataView(buffer, null).byteOffset !== 0) throw "null offset";
if (new DataView(buffer, 1.9).byteOffset !== 1) throw "fraction offset";
if (new DataView(buffer, -0.9).byteOffset !== 0) throw "negative fraction offset";
if (new DataView(buffer, 1, NaN).byteLength !== 0) throw "NaN length";
if (new DataView(buffer, 1, "2").byteLength !== 2) throw "string length";
if (new DataView(buffer, 1, true).byteLength !== 1) throw "boolean length";
if (new DataView(buffer, 1, null).byteLength !== 0) throw "null length";
if (new DataView(buffer, 1, 2.9).byteLength !== 2) throw "fraction length";
if (new DataView(buffer, 1, -0.9).byteLength !== 0) throw "negative fraction length";

let offsetCalls = 0;
let objectOffset = new DataView(buffer, { valueOf: function () { offsetCalls = offsetCalls + 1; return 3.9; } });
if (objectOffset.byteOffset !== 3 || offsetCalls !== 1) throw "object offset";

let lengthCalls = 0;
let objectLength = new DataView(buffer, 1, { valueOf: function () { lengthCalls = lengthCalls + 1; return 4.9; } });
if (objectLength.byteLength !== 4 || lengthCalls !== 1) throw "object length";

let threwOffset = false;
try {
  new DataView(buffer, -1);
} catch (error) {
  threwOffset = true;
}
if (!threwOffset) throw "negative integer offset";

let threwLength = false;
try {
  new DataView(buffer, 0, -1);
} catch (error) {
  threwLength = true;
}
if (!threwLength) throw "negative integer length";

123;
