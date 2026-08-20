function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

var typedArrayPrototype = Object.getPrototypeOf(Uint8Array).prototype;
var invalidCanonicalKeys = ["1.1", "-0", "-1", "2", "3"];
for (var i = 0; i < invalidCanonicalKeys.length; i = i + 1) {
  Object.defineProperty(typedArrayPrototype, invalidCanonicalKeys[i], {
    configurable: true,
    get: function() {
      return 999;
    }
  });
}

var numeric = new Uint8Array([42, 43]);
var bigint = new BigInt64Array([42n, 43n]);
for (var j = 0; j < invalidCanonicalKeys.length; j = j + 1) {
  var key = invalidCanonicalKeys[j];
  assertSame(numeric[key], undefined, "numeric invalid key " + key);
  assertSame(bigint[key], undefined, "bigint invalid key " + key);
}

Object.defineProperty(typedArrayPrototype, "1.0", {
  configurable: true,
  get: function() {
    return 262;
  }
});
assertSame(numeric["1.0"], 262, "noncanonical key uses ordinary get");

true;
