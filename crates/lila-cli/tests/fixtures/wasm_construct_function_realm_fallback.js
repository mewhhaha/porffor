let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

function fallbackUsesNewTargetRealm(C, otherC, newTarget) {
  let result = Reflect.construct(C, [], newTarget);
  return Object.getPrototypeOf(result) === otherC.prototype;
}

let fallbacks = [null, 1];
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
