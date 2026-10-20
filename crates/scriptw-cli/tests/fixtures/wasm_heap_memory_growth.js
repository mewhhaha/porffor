let buffer = new ArrayBuffer(70000);
let view = new DataView(buffer);

view.setUint8(0, 11);
view.setUint8(65535, 22);
view.setUint8(69999, 33);

if (buffer.byteLength !== 70000) throw "byteLength";
if (view.getUint8(0) !== 11) throw "first byte";
if (view.getUint8(65535) !== 22) throw "page boundary byte";
if (view.getUint8(69999) !== 33) throw "last byte";

view.getUint8(0) + view.getUint8(65535) + view.getUint8(69999);
