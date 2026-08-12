function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

var numeric = new Uint8Array([42, 43]);
assertSame(Reflect.has(numeric, 0), true, "numeric zero");
assertSame(Reflect.has(numeric, "1"), true, "numeric one");

var bigint = new BigInt64Array([42n]);
assertSame(Reflect.has(bigint, 0), true, "bigint zero");

var typedArrayBaseConstructor = Object.getPrototypeOf(Int8Array);
var typedArrayPrototype = typedArrayBaseConstructor.prototype;
typedArrayPrototype["-0"] = "inherited minus zero";
typedArrayPrototype["42"] = "inherited out of bounds";
var invalidCanonicalKeys = ["-0", "1.1", "-1", "2", "42", "NaN", "Infinity"];
for (var i = 0; i < invalidCanonicalKeys.length; i = i + 1) {
  assertSame(
    Reflect.has(numeric, invalidCanonicalKeys[i]),
    false,
    "invalid canonical key " + invalidCanonicalKeys[i]
  );
}

typedArrayPrototype.inheritedOrdinary = 1;
numeric.ownOrdinary = 2;
assertSame(Reflect.has(numeric, "inheritedOrdinary"), true, "inherited ordinary");
assertSame(Reflect.has(numeric, "ownOrdinary"), true, "own ordinary");
assertSame(Reflect.has(numeric, "missingOrdinary"), false, "missing ordinary");

var nonCanonicalKey = "1.0";
typedArrayPrototype[nonCanonicalKey] = 3;
assertSame(Reflect.has(numeric, nonCanonicalKey), true, "inherited noncanonical");

var symbol = Symbol("typed array has property");
numeric[symbol] = 4;
assertSame(Reflect.has(numeric, symbol), true, "own symbol");

var detached = new Uint8Array([9]);
detached.ordinary = "survives";
var detachedSymbol = Symbol("detached");
detached[detachedSymbol] = true;
__lilaDetachArrayBuffer(detached.buffer);
assertSame(Reflect.has(detached, 0), false, "detached index");
assertSame(Reflect.has(detached, "ordinary"), true, "detached ordinary");
assertSame(Reflect.has(detached, detachedSymbol), true, "detached symbol");

var withFallbackObject = { value: 17 };
with (detached) {
  Infinity;
  assertSame(withFallbackObject.value, 17, "detached numeric with fallback");
}

var detachedBigInt = new BigInt64Array([9n]);
__lilaDetachArrayBuffer(detachedBigInt.buffer);
with (detachedBigInt) {
  Infinity;
  assertSame(withFallbackObject.value, 17, "detached bigint with fallback");
}

function ParentHasSentinel() {}

function catchesParentHasSentinel(callback, sentinel) {
  try {
    callback();
  } catch (error) {
    return error === sentinel && error instanceof ParentHasSentinel;
  }
  return false;
}

var parentTrapCount = 0;
var parentHasSentinel = new ParentHasSentinel();
var parent = new Proxy(typedArrayPrototype, {
  has: function() {
    parentTrapCount = parentTrapCount + 1;
    throw parentHasSentinel;
  }
});
var proxyPrototypeView = new Uint8Array(1);
Object.setPrototypeOf(proxyPrototypeView, parent);
assertSame(Reflect.has(proxyPrototypeView, 0), true, "valid index bypasses parent");
assertSame(Reflect.has(proxyPrototypeView, 1), false, "invalid index bypasses parent");
assertSame(parentTrapCount, 0, "canonical keys do not reach parent");
assertSame(
  catchesParentHasSentinel(function() {
    Reflect.has(proxyPrototypeView, "missing");
  }, parentHasSentinel),
  true,
  "ordinary key preserves the parent trap exception"
);
Object.defineProperty(proxyPrototypeView, "missing", { value: true });
assertSame(Reflect.has(proxyPrototypeView, "missing"), true, "own ordinary bypasses parent");

var trackingBuffer = new ArrayBuffer(4, { maxByteLength: 6 });
var tracking = new Uint8Array(trackingBuffer, 1);
assertSame(Reflect.has(tracking, 2), true, "tracking initial");
assertSame(Reflect.has(tracking, 3), false, "tracking initial bound");
trackingBuffer.resize(6);
assertSame(Reflect.has(tracking, 4), true, "tracking grow");
trackingBuffer.resize(2);
assertSame(Reflect.has(tracking, 0), true, "tracking shrink");
assertSame(Reflect.has(tracking, 1), false, "tracking shrink bound");
trackingBuffer.resize(0);
assertSame(Reflect.has(tracking, 0), false, "tracking out of bounds");

var fixedBuffer = new ArrayBuffer(4, { maxByteLength: 6 });
var fixed = new Uint8Array(fixedBuffer, 1, 2);
assertSame(Reflect.has(fixed, 1), true, "fixed initial");
fixedBuffer.resize(6);
assertSame(Reflect.has(fixed, 1), true, "fixed grow");
assertSame(Reflect.has(fixed, 2), false, "fixed grow bound");
fixedBuffer.resize(2);
assertSame(Reflect.has(fixed, 0), false, "fixed out of bounds");

var other = __lilaCreateRealm().global;
var otherDetached = new other.Uint8Array(1);
__lilaDetachArrayBuffer(otherDetached.buffer);
assertSame(Reflect.has(otherDetached, 0), false, "cross-realm detached index");

true;
