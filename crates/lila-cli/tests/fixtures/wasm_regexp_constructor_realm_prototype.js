let $262 = { createRealm: __lilaCreateRealm };
let other = $262.createRealm().global;

if (other.RegExp.prototype === RegExp.prototype) {
  throw "RegExp prototype realm identity";
}

function assertPrototype(value, expected, label) {
  if (Object.getPrototypeOf(value) !== expected) {
    throw label + " prototype";
  }
}

assertPrototype(RegExp("a"), RegExp.prototype, "entry active call");
assertPrototype(new RegExp("a"), RegExp.prototype, "entry construct");
assertPrototype(other.RegExp("a"), other.RegExp.prototype, "created active call");
assertPrototype(
  new other.RegExp("a"),
  other.RegExp.prototype,
  "created construct",
);

let primitives = [undefined, null, true, "", Symbol(), 0];
for (let i = 0; i < primitives.length; i = i + 1) {
  other.Proxy.prototype = primitives[i];
  assertPrototype(
    Reflect.construct(RegExp, ["a"], other.Proxy),
    other.RegExp.prototype,
    "primitive fallback " + i,
  );
}

function assertCustomPrototype(prototype, label) {
  other.Proxy.prototype = prototype;
  let value = Reflect.construct(RegExp, ["a"], other.Proxy);
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
if (!Array.isArray(arrayPrototype)) {
  throw "Array custom prototype tag";
}

let reads = 0;
let observedNewTarget = new Proxy(other.Proxy, {
  get: function (target, key, receiver) {
    if (key === "prototype") {
      reads = reads + 1;
      return null;
    }
    return Reflect.get(target, key, receiver);
  },
});
let observed = Reflect.construct(RegExp, ["a"], observedNewTarget);
if (!(reads === 1 &&
      Object.getPrototypeOf(observed) === other.RegExp.prototype)) {
  throw "prototype Get count/fallback";
}

262;
