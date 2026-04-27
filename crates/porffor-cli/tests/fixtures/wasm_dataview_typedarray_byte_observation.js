let buffer = new ArrayBuffer(4);
let view = new DataView(buffer);
let bytes = new Uint8Array(buffer, 0);

view.setUint8(0, 255);
view.setUint8(1, 256);
view.setInt8(2, -1);

bytes[0] + bytes[1] + bytes[2];
