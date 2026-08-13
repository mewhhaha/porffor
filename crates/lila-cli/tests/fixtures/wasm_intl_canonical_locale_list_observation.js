function sameArray(actual, expected) {
  if (actual.length !== expected.length) return false;
  for (var i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) return false;
  }
  return true;
}

// Length is read once. Each later index is tested and then read against the
// original Proxy, so coercing element zero can shrink `length`, create element
// one, and still leave element two in the fixed walk.
var order = [];
var source = {
  length: 3,
  0: {
    toString: function () {
      order.push("toString:0");
      source.length = 1;
      source[1] = "pt-BR";
      return "en-US";
    },
  },
  2: "de-DE",
};
var observed = new Proxy(source, {
  get: function (target, key) {
    order.push("get:" + key);
    return target[key];
  },
  has: function (target, key) {
    order.push("has:" + key);
    return key in target;
  },
});
var observedResult = Intl.getCanonicalLocales(observed);
if (!sameArray(observedResult, ["en-US", "pt-BR", "de-DE"])) {
  throw "canonical locale observable result";
}
if (
  !sameArray(order, [
    "get:length",
    "has:0",
    "get:0",
    "toString:0",
    "has:1",
    "get:1",
    "has:2",
    "get:2",
  ])
) {
  throw "canonical locale observable order";
}

// A hole is skipped without Get or coercion, while an inherited index is
// present and contributes normally.
var sparse = Object.create({ 2: "fr-FR" });
sparse.length = 3;
sparse[0] = "es-ES";
if (!sameArray(Intl.getCanonicalLocales(sparse), ["es-ES", "fr-FR"])) {
  throw "canonical locale sparse inherited indices";
}

// Non-String primitives are boxed, so indexed properties inherited from the
// corresponding wrapper prototype remain observable.
var oldNumberZero = Object.getOwnPropertyDescriptor(Number.prototype, "0");
var oldNumberLength = Object.getOwnPropertyDescriptor(Number.prototype, "length");
Object.defineProperty(Number.prototype, "0", {
  configurable: true,
  value: "fr-CA",
});
Object.defineProperty(Number.prototype, "length", {
  configurable: true,
  value: 1,
});
var primitiveResult = Intl.getCanonicalLocales(7);
if (oldNumberZero === undefined) delete Number.prototype[0];
else Object.defineProperty(Number.prototype, "0", oldNumberZero);
if (oldNumberLength === undefined) delete Number.prototype.length;
else Object.defineProperty(Number.prototype, "length", oldNumberLength);
if (!sameArray(primitiveResult, ["fr-CA"])) {
  throw "canonical locale primitive wrapper";
}

// HasProperty owns the abrupt completion at an index. Its sentinel must escape
// unchanged and the corresponding Get must never run.
var sentinel = {};
var indexedGetObserved = false;
var abrupt = new Proxy(
  { length: 1, 0: "en-US" },
  {
    get: function (target, key) {
      if (key === "0") indexedGetObserved = true;
      return target[key];
    },
    has: function (_target, key) {
      if (key === "0") throw sentinel;
      return false;
    },
  },
);
var caughtSentinel = false;
try {
  Intl.getCanonicalLocales(abrupt);
} catch (error) {
  caughtSentinel = error === sentinel;
}
if (!caughtSentinel || indexedGetObserved) {
  throw "canonical locale HasProperty abrupt completion";
}

// Provider canonicalization precedes deduplication: both inputs are the same
// canonical locale after the pinned alias pass.
if (!sameArray(Intl.getCanonicalLocales(["iw-IL", "he-IL"]), ["he-IL"])) {
  throw "canonical locale alias deduplication";
}

262;
