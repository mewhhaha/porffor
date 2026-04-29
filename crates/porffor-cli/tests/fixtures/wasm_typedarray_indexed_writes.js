let buffer = new ArrayBuffer(8);
let view = new DataView(buffer);
let bytes = new Uint8Array(buffer, 1);

let assigned0 = bytes[0] = 0;
let assigned1 = bytes[1] = 1;
let assigned255 = bytes[2] = 255;
let assigned256 = bytes[3] = 256;
let assignedNegative = bytes[4] = -1;

let stored =
  view.getUint8(1) +
  view.getUint8(2) +
  view.getUint8(3) +
  view.getUint8(4) +
  view.getUint8(5);
let assigned = assigned0 + assigned1 + assigned255 + assigned256 + assignedNegative;

stored + assigned;
