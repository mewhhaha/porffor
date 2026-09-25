function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

function assertError(errorType, fn, label) {
  var threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof errorType)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

function ProtoSentinel() {}

// TypedArray ( ...args ) step 6.c: a non-Object first argument goes through
// ToIndex before AllocateTypedArray reads NewTarget.prototype.
var prototypeReads = 0;
var throwingTarget = function () {}.bind(null);
Object.defineProperty(throwingTarget, "prototype", {
  get() {
    prototypeReads = prototypeReads + 1;
    throw new ProtoSentinel();
  }
});

var constructors = [
  Int8Array, Uint8Array, Uint8ClampedArray, Int16Array, Uint16Array,
  Int32Array, Uint32Array, Float32Array, Float64Array, BigInt64Array,
  BigUint64Array
];
for (var i = 0; i < constructors.length; i++) {
  var TA = constructors[i];
  assertError(TypeError, function () { Reflect.construct(TA, [Symbol()], throwingTarget); }, TA.name + " Symbol length");
  assertError(TypeError, function () { Reflect.construct(TA, [1n], throwingTarget); }, TA.name + " BigInt length");
  assertError(RangeError, function () { Reflect.construct(TA, [-1], throwingTarget); }, TA.name + " negative length");
  assertError(RangeError, function () { Reflect.construct(TA, [2 ** 53], throwingTarget); }, TA.name + " oversized length");
  assertSame(prototypeReads, 0, TA.name + " ToIndex precedes the prototype read");

  // A valid length, no argument, and every Object argument read the
  // prototype first.
  assertError(ProtoSentinel, function () { Reflect.construct(TA, [1], throwingTarget); }, TA.name + " valid length");
  assertError(ProtoSentinel, function () { Reflect.construct(TA, [], throwingTarget); }, TA.name + " no argument");
  assertError(ProtoSentinel, function () {
    Reflect.construct(TA, [{ get length() { throw new TypeError(); } }], throwingTarget);
  }, TA.name + " array-like argument");
  assertError(ProtoSentinel, function () {
    Reflect.construct(TA, [new ArrayBuffer(8), { valueOf() { throw new TypeError(); } }], throwingTarget);
  }, TA.name + " buffer argument");
  assertSame(prototypeReads, 4, TA.name + " prototype reads");
  prototypeReads = 0;

  assertSame(new TA("3").length, 3, TA.name + " string length");
  assertSame(new TA(true).length, 1, TA.name + " boolean length");
  assertSame(new TA(null).length, 0, TA.name + " null length");
}

41;
