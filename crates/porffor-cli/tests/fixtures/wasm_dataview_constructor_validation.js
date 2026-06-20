__porfAssertThrows(TypeError, function () {
  new DataView(1);
});

__porfAssertThrows(TypeError, function () {
  new DataView({});
});

let buffer = new ArrayBuffer(8);

__porfAssertThrows(RangeError, function () {
  new DataView(buffer, -1);
});

__porfAssertThrows(RangeError, function () {
  new DataView(buffer, 9);
});

__porfAssertThrows(RangeError, function () {
  new DataView(buffer, 4, 5);
});

let view = new DataView(buffer, 2, 3);
if (view.byteOffset !== 2) throw "byteOffset";
if (view.byteLength !== 3) throw "explicit byteLength";

let sample1 = new DataView(buffer, 0);
let sample2 = new DataView(buffer, 4);
let sample3 = new DataView(buffer, 6, 2);
let sample4 = new DataView(buffer, 8);

if (sample1.byteLength !== 8) throw "sample1 byteLength";
if (sample2.byteLength !== 4) throw "sample2 byteLength";
if (sample3.byteLength !== 2) throw "sample3 byteLength";
if (sample4.byteLength !== 0) throw "sample4 byteLength";

view.byteOffset + view.byteLength;
