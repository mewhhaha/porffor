let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

if (other.Iterator.prototype === Iterator.prototype) {
  throw "Iterator prototype realm identity";
}

function assertOtherRealmIterator(prototype, label) {
  let C = new other.Function();
  C.prototype = prototype;
  let value = Reflect.construct(Iterator, [], C);
  if (Object.getPrototypeOf(value) !== other.Iterator.prototype) {
    throw label + " realm prototype";
  }
}

assertOtherRealmIterator(undefined, "undefined");
assertOtherRealmIterator(null, "null");
assertOtherRealmIterator(true, "boolean");
assertOtherRealmIterator("", "string");
assertOtherRealmIterator(Symbol(), "symbol");
assertOtherRealmIterator(0, "number");

let boundNewTarget = new other.Function().bind(null);
let boundFallback = Reflect.construct(Iterator, [], boundNewTarget);
if (Object.getPrototypeOf(boundFallback) !== other.Iterator.prototype) {
  throw "bound new target realm prototype";
}

let innerPrototypeReads = 0;
let outerPrototypeReads = 0;
let nestedTarget = new other.Function();
nestedTarget.prototype = null;
let innerProxy = new Proxy(nestedTarget, {
  get: function (target, key, receiver) {
    if (key === "prototype") innerPrototypeReads = innerPrototypeReads + 1;
    return Reflect.get(target, key, receiver);
  },
});
let outerProxy = new Proxy(innerProxy, {
  get: function (target, key, receiver) {
    if (key === "prototype") outerPrototypeReads = outerPrototypeReads + 1;
    return Reflect.get(target, key, receiver);
  },
});
let nestedProxyFallback = Reflect.construct(Iterator, [], outerProxy);
if (!(Object.getPrototypeOf(nestedProxyFallback) === other.Iterator.prototype &&
      innerPrototypeReads === 1 && outerPrototypeReads === 1)) {
  throw "nested proxy new target realm prototype";
}

function assertCustomPrototype(prototype, label) {
  let C = new other.Function();
  C.prototype = prototype;
  let value = Reflect.construct(Iterator, [], C);
  let actual = Object.getPrototypeOf(value);
  if (actual !== prototype) throw label + " custom prototype identity";
  return actual;
}

let objectPrototype = assertCustomPrototype({ custom: true }, "Object");
if (typeof objectPrototype !== "object" || Array.isArray(objectPrototype)) {
  throw "Object custom prototype tag";
}
let functionPrototype = assertCustomPrototype(function () {}, "Function");
if (typeof functionPrototype !== "function") {
  throw "Function custom prototype tag";
}
let arrayPrototype = assertCustomPrototype([], "Array");
if (!Array.isArray(arrayPrototype)) throw "Array custom prototype tag";

let reads = 0;
let observedNewTarget = new Proxy(new other.Function(), {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      reads = reads + 1;
      return null;
    }
    return Reflect.get(target, key, receiver);
  },
});
let observed = Reflect.construct(Iterator, [], observedNewTarget);
if (!(reads === 1 && Object.getPrototypeOf(observed) === other.Iterator.prototype)) {
  throw "prototype Get count/fallback";
}

let abruptMarker = {};
let abruptReads = 0;
let abruptNewTarget = new Proxy(new other.Function(), {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      abruptReads = abruptReads + 1;
      throw abruptMarker;
    }
    return Reflect.get(target, key, receiver);
  },
});
let abruptThrew = false;
try {
  Reflect.construct(Iterator, [], abruptNewTarget);
} catch (error) {
  abruptThrew = error === abruptMarker;
}
if (!(abruptThrew && abruptReads === 1)) throw "prototype Get abrupt completion";

let objectFallbackPrototype = [];
let objectRevocable;
objectRevocable = Proxy.revocable(new other.Function(), {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      objectRevocable.revoke();
      return objectFallbackPrototype;
    }
    return Reflect.get(target, key, receiver);
  },
});
let objectFallback = Reflect.construct(Iterator, [], objectRevocable.proxy);
let objectFallbackActual = Object.getPrototypeOf(objectFallback);
if (!(objectFallbackActual === objectFallbackPrototype &&
      Array.isArray(objectFallbackActual))) {
  throw "object prototype must not resolve revoked function realm";
}

let primitiveRevocable;
primitiveRevocable = Proxy.revocable(new other.Function(), {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      primitiveRevocable.revoke();
      return null;
    }
    return Reflect.get(target, key, receiver);
  },
});
let revocationThrew = false;
try {
  Reflect.construct(Iterator, [], primitiveRevocable.proxy);
} catch (error) {
  revocationThrew = error instanceof TypeError;
}
if (!revocationThrew) throw "revoked function realm fallback";

262;
