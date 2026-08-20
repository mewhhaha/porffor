let buffer = new ArrayBuffer(16);
let view = new DataView(buffer);
view.setUint8(0, 11);

function read(offset, extra) {
  return view.getUint8(offset) + extra + this.bias;
}

let holder = {
  call: read.bind({ bias: 5 }, 0),
};

let grown = new ArrayBuffer(70000);
let grownView = new DataView(grown);
grownView.setUint8(69999, 19);

holder.call(7) + grownView.getUint8(69999) + buffer.byteLength;
