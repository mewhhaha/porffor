let buffer = new ArrayBuffer(3);
let view = new DataView(buffer);
view.setUint8(0, 7);

let object = { marker: 35 };

if (buffer.byteLength !== 3) throw "byteLength";
if (view.getUint8(0) !== 7) throw "byte";

object.marker + buffer.byteLength;
