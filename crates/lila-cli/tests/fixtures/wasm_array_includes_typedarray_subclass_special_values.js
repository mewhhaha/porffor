function exercise(TA) {
  var bytes = TA.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(bytes * 4, { maxByteLength: bytes * 8 });
  var view = new TA(buffer);
  view[0] = -Infinity;
  view[1] = Infinity;
  view[2] = NaN;
  view[3] = 0;
  return view.length === 4 &&
    view[0] === -Infinity &&
    view[1] === Infinity &&
    view[2] !== view[2] &&
    Array.prototype.includes.call(view, NaN);
}

class MyFloat32Array extends Float32Array {}

exercise(Float32Array) &&
  exercise(Float64Array) &&
  exercise(MyFloat32Array);
