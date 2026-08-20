function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

var constructors = [Int8Array, Int16Array, Int32Array, Uint8Array, Uint16Array, Uint32Array];

for (var i = 0; i < constructors.length; i++) {
  var TA = constructors[i];
  var view = new TA(new ArrayBuffer(TA.BYTES_PER_ELEMENT * 4));
  assertSame(Atomics.store(view, 0, 1), 1, TA.name + " first store result");
  assertSame(Atomics.load(view, 0), 1, TA.name + " first load");
  assertSame(Atomics.store(view, 0, 3), 3, TA.name + " second store result");
  assertSame(Atomics.load(view, 0), 3, TA.name + " second load");
}

567;
