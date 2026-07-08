function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

if (typeof Atomics.compareExchange !== "function") throw "compareExchange function";
if (Atomics.compareExchange.length !== 4) throw "compareExchange length";
if (Atomics.compareExchange.name !== "compareExchange") throw "compareExchange name";

var constructors = [Int8Array, Int16Array, Int32Array, Uint8Array, Uint16Array, Uint32Array];

for (var i = 0; i < constructors.length; i++) {
  var TA = constructors[i];
  var view = new TA(new ArrayBuffer(TA.BYTES_PER_ELEMENT * 4));
  var control = new TA(new ArrayBuffer(TA.BYTES_PER_ELEMENT));

  assertSame(Atomics.store(view, 0, 7), 7, TA.name + " initial store");
  assertSame(Atomics.compareExchange(view, 0, 8, 12), 7, TA.name + " failed result");
  assertSame(Atomics.load(view, 0), 7, TA.name + " failed load");
  assertSame(Atomics.compareExchange(view, 0, 7, 12), 7, TA.name + " success result");
  assertSame(Atomics.load(view, 0), 12, TA.name + " success load");

  control[0] = 12345;
  view[0] = 12345;
  assertSame(
    Atomics.compareExchange(view, 0, 12345, 0),
    control[0],
    TA.name + " converted expected result"
  );
  assertSame(Atomics.load(view, 0), 0, TA.name + " converted expected load");
}

789;
