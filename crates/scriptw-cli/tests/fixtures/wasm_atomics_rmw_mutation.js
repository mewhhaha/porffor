function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

var constructors = [Int8Array, Int16Array, Int32Array, Uint8Array, Uint16Array, Uint32Array];

for (var i = 0; i < constructors.length; i++) {
  var TA = constructors[i];
  var view = new TA(new ArrayBuffer(TA.BYTES_PER_ELEMENT * 4));
  assertSame(Atomics.store(view, 0, 7), 7, TA.name + " initial store");
  assertSame(Atomics.exchange(view, 0, 12), 7, TA.name + " exchange result");
  assertSame(Atomics.load(view, 0), 12, TA.name + " exchange load");
  assertSame(Atomics.sub(view, 0, 4), 12, TA.name + " sub result");
  assertSame(Atomics.load(view, 0), 8, TA.name + " sub load");
  assertSame(Atomics.and(view, 0, 3), 8, TA.name + " and result");
  assertSame(Atomics.load(view, 0), 0, TA.name + " and load");
  assertSame(Atomics.or(view, 0, 5), 0, TA.name + " or result");
  assertSame(Atomics.load(view, 0), 5, TA.name + " or load");
  assertSame(Atomics.xor(view, 0, 3), 5, TA.name + " xor result");
  assertSame(Atomics.load(view, 0), 6, TA.name + " xor load");
}

678;
