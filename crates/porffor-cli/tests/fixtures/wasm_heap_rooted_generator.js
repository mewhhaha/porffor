let buffer = new ArrayBuffer(8);
let view = new DataView(buffer);
view.setUint8(0, 13);

let iterator = (function* () {
  yield view.getUint8(0);
  yield buffer.byteLength;
})();
let grown = new ArrayBuffer(70000);
let grownView = new DataView(grown);
grownView.setUint8(69999, 29);

let first = iterator.next();
let second = iterator.next();

first.value + second.value + grownView.getUint8(69999);
