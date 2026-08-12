let buffer = new ArrayBuffer(6);
let view = new DataView(buffer, 1, 4);

view.setUint16(0, 0x1234);
let big = view.getUint16(0);
let little = view.getUint16(0, true);

view.setUint16(2, 0xabcd, true);
let littleWrite = view.getUint16(2, true);
let bigRead = view.getUint16(2);

big + little + littleWrite + bigRead;
