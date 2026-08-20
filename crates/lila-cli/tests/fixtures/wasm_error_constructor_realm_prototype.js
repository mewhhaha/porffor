let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

function assertOtherRealmError(prototype, label) {
  let C = new other.Function();
  C.prototype = prototype;
  let value = Reflect.construct(Error, [label], C);
  if (Object.getPrototypeOf(value) !== other.Error.prototype) {
    throw label + " realm prototype";
  }
  if (!(Error.isError(value) && value.message === label)) {
    throw label + " Error brand/message";
  }
}

assertOtherRealmError(undefined, "undefined");
assertOtherRealmError(null, "null");
assertOtherRealmError(true, "boolean");
assertOtherRealmError("", "string");
assertOtherRealmError(Symbol(), "symbol");
assertOtherRealmError(0, "number");

let called = other.Error("called");
if (!(Object.getPrototypeOf(called) === other.Error.prototype &&
      Error.isError(called) &&
      called.message === "called")) {
  throw "called active Error realm";
}

function assertCustomPrototype(prototype, label) {
  let C = new other.Function();
  C.prototype = prototype;
  let value = Reflect.construct(Error, [label], C);
  let actual = Object.getPrototypeOf(value);
  if (actual !== prototype) throw label + " custom prototype tag/identity";
  if (!(Error.isError(value) && value.message === label)) {
    throw label + " custom prototype Error brand/message";
  }
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
let argumentsPrototype = assertCustomPrototype(
  (function () { return arguments; })(),
  "Arguments",
);
if (Object.prototype.toString.call(argumentsPrototype) !== "[object Arguments]") {
  throw "Arguments custom prototype tag";
}

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
let observed = Reflect.construct(Error, ["observed"], observedNewTarget);
if (!(reads === 1 &&
      Object.getPrototypeOf(observed) === other.Error.prototype &&
      observed.message === "observed")) {
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
  Reflect.construct(Error, [], abruptNewTarget);
} catch (error) {
  abruptThrew = error === abruptMarker;
}
if (!(abruptThrew && abruptReads === 1)) {
  throw "prototype Get abrupt completion";
}

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
let objectFallback = Reflect.construct(Error, [], objectRevocable.proxy);
let objectFallbackActual = Object.getPrototypeOf(objectFallback);
if (!(objectFallbackActual === objectFallbackPrototype &&
      Array.isArray(objectFallbackActual) &&
      Error.isError(objectFallback))) {
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
  Reflect.construct(Error, [], primitiveRevocable.proxy);
} catch (error) {
  revocationThrew = error instanceof TypeError;
}
if (!revocationThrew) throw "revoked function realm fallback";

262;
