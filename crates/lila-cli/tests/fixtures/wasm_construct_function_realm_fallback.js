let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

if (!Array.isArray(other.Array.prototype)) throw "created Array.prototype is not an Array";
if (other.Array.prototype.length !== 0) throw "created Array.prototype length";
if (Object.getPrototypeOf(other.Array.prototype) !== other.Object.prototype) {
  throw "created Array.prototype prototype";
}

let arrayPrototypeLengthDescriptor = Object.getOwnPropertyDescriptor(
  other.Array.prototype,
  "length",
);
if (!(arrayPrototypeLengthDescriptor.value === 0 &&
      arrayPrototypeLengthDescriptor.writable === true &&
      arrayPrototypeLengthDescriptor.enumerable === false &&
      arrayPrototypeLengthDescriptor.configurable === false)) {
  throw "created Array.prototype length descriptor";
}

let arrayPrototypeDescriptor = Object.getOwnPropertyDescriptor(other.Array, "prototype");
if (!(arrayPrototypeDescriptor.value === other.Array.prototype &&
      arrayPrototypeDescriptor.writable === false &&
      arrayPrototypeDescriptor.enumerable === false &&
      arrayPrototypeDescriptor.configurable === false)) {
  throw "created Array prototype descriptor";
}

let arrayConstructorDescriptor = Object.getOwnPropertyDescriptor(
  other.Array.prototype,
  "constructor",
);
if (!(arrayConstructorDescriptor.value === other.Array &&
      arrayConstructorDescriptor.writable === true &&
      arrayConstructorDescriptor.enumerable === false &&
      arrayConstructorDescriptor.configurable === true)) {
  throw "created Array constructor descriptor";
}

other.Array.prototype[2] = 42;
if (!(other.Array.prototype.length === 3 && other.Array.prototype[2] === 42)) {
  throw "created Array.prototype indexed semantics";
}
delete other.Array.prototype[2];
other.Array.prototype.length = 0;

function fallbackUsesNewTargetRealm(C, otherC, newTarget) {
  let result = Reflect.construct(C, [], newTarget);
  return Object.getPrototypeOf(result) === otherC.prototype;
}

let fallbacks = [undefined, null, true, "prototype", Symbol("prototype"), 1];
for (let value of fallbacks) {
  // Proxy is a constructable function from the other realm with no own
  // `prototype` initially, so these primitive assignments are ordinary data
  // properties and do not violate Proxy [[Get]] invariants.
  other.Proxy.prototype = value;
  if (!fallbackUsesNewTargetRealm(Object, other.Object, other.Proxy)) throw "Object realm fallback";
  if (!fallbackUsesNewTargetRealm(String, other.String, other.Proxy)) throw "String realm fallback";
  if (!fallbackUsesNewTargetRealm(Number, other.Number, other.Proxy)) throw "Number realm fallback";
  if (!fallbackUsesNewTargetRealm(Boolean, other.Boolean, other.Proxy)) throw "Boolean realm fallback";
  if (!fallbackUsesNewTargetRealm(Array, other.Array, other.Proxy)) throw "Array realm fallback";
  if (!fallbackUsesNewTargetRealm(Uint8Array, other.Uint8Array, other.Proxy)) {
    throw "Uint8Array realm fallback";
  }
}

let otherRealmBoundNewTarget = other.Proxy.bind(null);
if (otherRealmBoundNewTarget.prototype !== undefined) {
  throw "bound function unexpectedly has prototype";
}
let boundRealmValue = Reflect.construct(Object, [], otherRealmBoundNewTarget);
if (Object.getPrototypeOf(boundRealmValue) !== other.Object.prototype) {
  throw "bound function realm fallback";
}

let proxyPrototypeGets = 0;
let wrappedOtherRealmNewTarget = new Proxy(other.Proxy, {
  get: function (target, key, receiver) {
    if (key === "prototype") proxyPrototypeGets++;
    return Reflect.get(target, key, receiver);
  },
});
let wrappedRealmValue = Reflect.construct(Object, [], wrappedOtherRealmNewTarget);
if (!(proxyPrototypeGets === 1 &&
      Object.getPrototypeOf(wrappedRealmValue) === other.Object.prototype)) {
  throw "proxy function realm fallback";
}

let revocable;
revocable = Proxy.revocable(function () {}, {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      revocable.revoke();
      return 1;
    }
    return Reflect.get(target, key, receiver);
  },
});

let revocationThrew = false;
try {
  Reflect.construct(Object, [], revocable.proxy);
} catch (error) {
  revocationThrew = error instanceof TypeError;
}

if (!revocationThrew) throw "revoked function realm fallback";

true;
