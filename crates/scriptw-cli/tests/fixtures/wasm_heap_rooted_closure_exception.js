let buffer = new ArrayBuffer(8);
let view = new DataView(buffer);
view.setUint8(0, 17);

function readCaptured(offset) {
  return view.getUint8(offset);
}

let caughtRead;

try {
  throw { read: readCaptured };
} catch (e) {
  caughtRead = e.read;
}

let grown = new ArrayBuffer(70000);
let grownView = new DataView(grown);
grownView.setUint8(69999, 23);

caughtRead(0) + grownView.getUint8(69999) + buffer.byteLength;
