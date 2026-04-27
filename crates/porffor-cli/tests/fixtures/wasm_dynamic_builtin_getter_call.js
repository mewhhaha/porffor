let buffer = new ArrayBuffer(4);
let view = new DataView(buffer);
let desc = Object.getOwnPropertyDescriptor(DataView.prototype, "byteLength");

desc.get.call(view);
