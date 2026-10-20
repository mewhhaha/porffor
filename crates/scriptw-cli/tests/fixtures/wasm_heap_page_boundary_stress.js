let total = 0;

for (let i = 0; i < 6; i = i + 1) {
  let size = 65536 + i * 4096 + 3;
  let buffer = new ArrayBuffer(size);
  let view = new DataView(buffer);

  view.setUint8(0, i + 1);
  view.setUint8(65535, i + 2);
  view.setUint8(size - 1, i + 3);

  if (buffer.byteLength !== size) throw "byteLength";
  total = total + view.getUint8(0);
  total = total + view.getUint8(65535);
  total = total + view.getUint8(size - 1);
}

total;
