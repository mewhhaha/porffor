function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

function objectDefineThrows(target, key, descriptor, label) {
  var threw = false;
  try {
    Object.defineProperty(target, key, descriptor);
  } catch (error) {
    threw = error instanceof TypeError;
  }
  assertSame(threw, true, label);
}

var numeric = new Uint8Array([1, 2]);
assertSame(
  Object.defineProperty(numeric, 0, { value: 42 }),
  numeric,
  "Object valid numeric index"
);
assertSame(numeric[0], 42, "Object numeric value");
assertSame(
  Reflect.defineProperty(numeric, 1, { value: 43, configurable: true }),
  true,
  "Reflect valid numeric index"
);
assertSame(numeric[1], 43, "Reflect numeric value");

var bigint = new BigInt64Array([1n]);
assertSame(
  Reflect.defineProperty(bigint, 0, { value: 42n }),
  true,
  "Reflect valid bigint index"
);
assertSame(bigint[0], 42n, "Reflect bigint value");

var invalidCanonicalKeys = ["-0", "-1", "0.1", "1.1", "2", "Infinity"];
for (var i = 0; i < invalidCanonicalKeys.length; i = i + 1) {
  var invalidKey = invalidCanonicalKeys[i];
  assertSame(
    Reflect.defineProperty(numeric, invalidKey, { value: 9 }),
    false,
    "Reflect invalid canonical key " + invalidKey
  );
  assertSame(
    Object.getOwnPropertyDescriptor(numeric, invalidKey),
    undefined,
    "invalid canonical key remains absent " + invalidKey
  );
  objectDefineThrows(
    numeric,
    invalidKey,
    { value: 9 },
    "Object invalid canonical key " + invalidKey
  );
}

assertSame(
  Reflect.defineProperty(numeric, 0, { get: function() { return 1; } }),
  false,
  "numeric accessor descriptor"
);
assertSame(
  Reflect.defineProperty(numeric, 0, { configurable: false }),
  false,
  "numeric non-configurable descriptor"
);
assertSame(
  Reflect.defineProperty(numeric, 0, { enumerable: false }),
  false,
  "numeric non-enumerable descriptor"
);
assertSame(
  Reflect.defineProperty(numeric, 0, { writable: false }),
  false,
  "numeric non-writable descriptor"
);
objectDefineThrows(
  numeric,
  0,
  { writable: false },
  "Object rejects incompatible numeric descriptor"
);

function ValueConversionSentinel() {}
var valueConversionSentinel = new ValueConversionSentinel();
var abruptValue = {
  valueOf: function() {
    throw valueConversionSentinel;
  }
};
var abruptValueObserved = false;
try {
  Object.defineProperty(numeric, 0, { value: abruptValue });
} catch (error) {
  abruptValueObserved =
    error === valueConversionSentinel && error instanceof ValueConversionSentinel;
}
assertSame(abruptValueObserved, true, "numeric value conversion abrupt completion");

var detaching = new Uint8Array([17]);
var detachingValue = {
  valueOf: function() {
    __porfDetachArrayBuffer(detaching.buffer);
    return 42;
  }
};
assertSame(
  Reflect.defineProperty(detaching, 0, { value: detachingValue }),
  true,
  "value conversion may detach buffer"
);
assertSame(detaching[0], undefined, "detached conversion does not write");

var detached = new Uint8Array([1]);
__porfDetachArrayBuffer(detached.buffer);
assertSame(
  Reflect.defineProperty(detached, 0, { value: 2 }),
  false,
  "Reflect detached index"
);
objectDefineThrows(detached, "-0", { value: 2 }, "Object detached invalid index");

var other = __porfCreateRealm().global;
var otherDetached = new other.Uint8Array(1);
__porfDetachArrayBuffer(otherDetached.buffer);
objectDefineThrows(
  otherDetached,
  "1.5",
  { value: 2 },
  "Object cross-realm detached index"
);

assertSame(
  Reflect.defineProperty(numeric, "1.0", { value: "ordinary", configurable: true }),
  true,
  "noncanonical numeric string is ordinary"
);
assertSame(numeric["1.0"], "ordinary", "noncanonical ordinary value");
var symbol = Symbol("typed array define own property");
assertSame(
  Reflect.defineProperty(numeric, symbol, { value: "symbol", configurable: true }),
  true,
  "symbol key is ordinary"
);
assertSame(numeric[symbol], "symbol", "symbol ordinary value");

Object.preventExtensions(numeric);
assertSame(
  Reflect.defineProperty(numeric, "newOrdinary", { value: 1 }),
  false,
  "non-extensible ordinary key"
);

var trapCount = 0;
var proxy = new Proxy(numeric, {
  defineProperty: function(target, key, descriptor) {
    trapCount = trapCount + 1;
    assertSame(key, "0", "proxy key");
    assertSame(descriptor.value, 99, "proxy descriptor value");
    return true;
  }
});
assertSame(
  Reflect.defineProperty(proxy, 0, {
    value: 99,
    writable: true,
    enumerable: true,
    configurable: true
  }),
  true,
  "Proxy trap result"
);
assertSame(trapCount, 1, "Proxy trap runs before typed array semantics");
assertSame(numeric[0], 42, "Proxy trap does not fall through");

var rejectingProxy = new Proxy(numeric, {
  defineProperty: function() {
    return false;
  }
});
assertSame(
  Reflect.defineProperty(rejectingProxy, 0, { value: 100 }),
  false,
  "Reflect Proxy false result"
);
objectDefineThrows(
  rejectingProxy,
  0,
  { value: 100 },
  "Object Proxy false result"
);

true;
