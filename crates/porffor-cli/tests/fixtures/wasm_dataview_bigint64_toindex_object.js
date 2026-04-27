let buffer = new ArrayBuffer(12);
let view = new DataView(buffer);

view.setUint8(0, 0x27);
view.setUint8(1, 0x02);
view.setUint8(2, 0x06);
view.setUint8(3, 0x02);
view.setUint8(4, 0x80);
view.setUint8(5, 0x00);
view.setUint8(6, 0x80);
view.setUint8(7, 0x01);
view.setUint8(8, 0x7f);
view.setUint8(9, 0x00);
view.setUint8(10, 0x01);
view.setUint8(11, 0x02);

(view.getBigInt64({}) === 0x2702060280008001n) +
  (view.getBigUint64({}) === 0x2702060280008001n) +
  (view.getBigInt64([]) === 0x2702060280008001n) +
  (view.getBigUint64([]) === 0x2702060280008001n);
