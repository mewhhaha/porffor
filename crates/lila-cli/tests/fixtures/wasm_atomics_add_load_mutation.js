function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

var constructors = [Int8Array, Int16Array, Int32Array, Uint8Array, Uint16Array, Uint32Array];

for (var i = 0; i < constructors.length; i++) {
  var TA = constructors[i];
  var view = new TA(new ArrayBuffer(TA.BYTES_PER_ELEMENT * 4));
  assertSame(Atomics.load(view, 0), 0, TA.name + " initial load");
  assertSame(Atomics.add(view, 0, 1), 0, TA.name + " first add result");
  assertSame(Atomics.load(view, 0), 1, TA.name + " first load");
  assertSame(Atomics.add(view, 0, 2), 1, TA.name + " second add result");
  assertSame(Atomics.load(view, 0), 3, TA.name + " second load");
}

456;
