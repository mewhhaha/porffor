function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

function assertElementDescriptor(view, key, expected, label) {
  var descriptor = Object.getOwnPropertyDescriptor(view, key);
  assertSame(descriptor.value, expected, label + " value");
  assertSame(descriptor.writable, true, label + " writable");
  assertSame(descriptor.enumerable, true, label + " enumerable");
  assertSame(descriptor.configurable, true, label + " configurable");
}

var numeric = new Uint8Array([42, 43]);
assertElementDescriptor(numeric, "0", 42, "numeric zero");
assertElementDescriptor(numeric, 1, 43, "numeric one");

var bigint = new BigInt64Array([42n]);
assertElementDescriptor(bigint, "0", 42n, "bigint zero");

var invalidCanonicalKeys = ["-0", "1.1", "0.1", "-1", "2", "NaN", "Infinity"];
for (var i = 0; i < invalidCanonicalKeys.length; i = i + 1) {
  assertSame(
    Object.getOwnPropertyDescriptor(numeric, invalidCanonicalKeys[i]),
    undefined,
    "invalid canonical key " + invalidCanonicalKeys[i]
  );
}

var ordinaryKeys = ["undef", "1.0", "+1", "1000000000000000000000", "0.0000001"];
for (var j = 0; j < ordinaryKeys.length; j = j + 1) {
  var ordinaryKey = ordinaryKeys[j];
  Object.defineProperty(numeric, ordinaryKey, { value: ordinaryKey });
  assertSame(
    Object.getOwnPropertyDescriptor(numeric, ordinaryKey).value,
    ordinaryKey,
    "ordinary key " + ordinaryKey
  );
}

var symbol = Symbol("typed array own property");
Object.defineProperty(numeric, symbol, { value: "symbol value" });
assertSame(
  Object.getOwnPropertyDescriptor(numeric, symbol).value,
  "symbol value",
  "symbol key"
);

var detached = new Uint8Array([9]);
detached.ordinary = "survives";
__lilaDetachArrayBuffer(detached.buffer);
assertSame(Object.getOwnPropertyDescriptor(detached, 0), undefined, "detached index");
assertSame(
  Object.getOwnPropertyDescriptor(detached, "ordinary").value,
  "survives",
  "detached ordinary key"
);

var other = __lilaCreateRealm().global;
var otherDetached = new other.Uint8Array(1);
__lilaDetachArrayBuffer(otherDetached.buffer);
assertSame(
  Object.getOwnPropertyDescriptor(otherDetached, 0),
  undefined,
  "cross-realm detached index"
);

true;
