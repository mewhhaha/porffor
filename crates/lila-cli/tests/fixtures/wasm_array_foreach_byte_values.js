let buffer = new ArrayBuffer(2);
let view = new DataView(buffer);
let values = [255, 256, -1];
let expected = [255, 0, 255];

values.forEach(function (value, i) {
  view.setUint8(0, value);
  view.setUint8(1, expected[i]);
});

view.getUint8(0) + view.getUint8(1);
