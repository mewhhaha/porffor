let $262 = { createRealm: __lilaCreateRealm };
let firstRealm = $262.createRealm().global;
let secondRealm = $262.createRealm().global;

function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

function dataDescriptor(object, key, value, writable, configurable, label) {
  let descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (descriptor === undefined) throw label + " missing";
  same(descriptor.value, value, label + " value");
  same(descriptor.writable, writable, label + " writable");
  same(descriptor.enumerable, false, label + " enumerable");
  same(descriptor.configurable, configurable, label + " configurable");
}

function intrinsicDescriptor(realm, label) {
  let intrinsic = realm.Function.prototype[realm.Symbol.hasInstance];
  same(typeof intrinsic, "function", label + " callable");
  dataDescriptor(
    realm.Function.prototype,
    realm.Symbol.hasInstance,
    intrinsic,
    false,
    false,
    label + " property",
  );
  dataDescriptor(intrinsic, "length", 1, false, true, label + " length");
  dataDescriptor(
    intrinsic,
    "name",
    "[Symbol.hasInstance]",
    false,
    true,
    label + " name",
  );
  return intrinsic;
}

let hasInstance = intrinsicDescriptor(globalThis, "entry realm");
let firstHasInstance = intrinsicDescriptor(firstRealm, "first realm");
let secondHasInstance = intrinsicDescriptor(secondRealm, "second realm");

if (hasInstance === firstHasInstance ||
    hasInstance === secondHasInstance ||
    firstHasInstance === secondHasInstance) {
  throw "realm-local intrinsic identity";
}
same(Object.getPrototypeOf(firstHasInstance), firstRealm.Function.prototype, "first realm prototype");
same(Object.getPrototypeOf(secondHasInstance), secondRealm.Function.prototype, "second realm prototype");

same(hasInstance.call(undefined, {}), false, "undefined receiver");
same(hasInstance.call({}, {}), false, "non-callable receiver");

function Candidate() {}
same(hasInstance.call(Candidate, undefined), false, "undefined candidate");
same(hasInstance.call(Candidate, null), false, "null candidate");
same(hasInstance.call(Candidate, 1), false, "number candidate");
same(hasInstance.call(Candidate, "value"), false, "string candidate");

let candidate = new Candidate();
same(hasInstance.call(Candidate, candidate), true, "positive chain");
same(hasInstance.call(Candidate, Object.create(candidate)), true, "deep positive chain");
same(hasInstance.call(Candidate, Object.create(null)), false, "negative chain");

let boundPrototypeGets = 0;
let bound = Candidate.bind(null);
Object.defineProperty(bound, "prototype", {
  get: function () {
    boundPrototypeGets++;
    throw "bound prototype read";
  },
  configurable: true,
});
same(hasInstance.call(bound, candidate), true, "bound target recursion");
same(boundPrototypeGets, 0, "bound target before prototype Get");

let token = {};
let customCalls = 0;
let customThis;
function CustomTarget() {}
Object.defineProperty(CustomTarget, Symbol.hasInstance, {
  value: function (value) {
    customCalls++;
    customThis = this;
    return value === token;
  },
  configurable: true,
});
let customBound = CustomTarget.bind(null);
same(hasInstance.call(customBound, token), true, "bound custom handler result");
same(customCalls, 1, "bound custom handler count");
same(customThis, CustomTarget, "bound custom handler receiver");

let prototypeMarker = {};
let PoisonedPrototype = Object.getOwnPropertyDescriptor({
  get value() {},
}, "value").get;
same(
  Object.prototype.hasOwnProperty.call(PoisonedPrototype, "prototype"),
  false,
  "call-only function starts without prototype",
);
Object.defineProperty(PoisonedPrototype, "prototype", {
  get: function () {
    throw prototypeMarker;
  },
});
let poisonedCaught = false;
try {
  hasInstance.call(PoisonedPrototype, {});
} catch (error) {
  poisonedCaught = error === prototypeMarker;
}
same(poisonedCaught, true, "poisoned prototype abrupt");

function HasDefaultPrototype() {}
let defaultPrototypeRejected = false;
try {
  Object.defineProperty(HasDefaultPrototype, "prototype", {
    get: function () {},
  });
} catch (error) {
  defaultPrototypeRejected = error.name === "TypeError";
}
same(defaultPrototypeRejected, true, "default prototype stays non-configurable");

let FlexiblePrototype = Object.getOwnPropertyDescriptor({
  get value() {},
}, "value").get;
FlexiblePrototype.prototype = {};
Object.defineProperty(FlexiblePrototype, "prototype", {
  get: function () {
    return 1;
  },
  configurable: true,
});
same(FlexiblePrototype.prototype, 1, "configurable prototype changes kind");

function PrimitivePrototype() {}
PrimitivePrototype.prototype = 1;
let primitivePrototypeCaught = false;
try {
  hasInstance.call(PrimitivePrototype, {});
} catch (error) {
  primitivePrototypeCaught = error.name === "TypeError";
}
same(primitivePrototypeCaught, true, "non-object prototype TypeError");

let proxyMarker = {};
let proxyCandidate = new Proxy({}, {
  getPrototypeOf: function () {
    throw proxyMarker;
  },
});
let proxyCaught = false;
try {
  hasInstance.call(Candidate, proxyCandidate);
} catch (error) {
  proxyCaught = error === proxyMarker;
}
same(proxyCaught, true, "Proxy GetPrototypeOf abrupt");

let proxyChainCaught = false;
try {
  hasInstance.call(Candidate, Object.create(proxyCandidate));
} catch (error) {
  proxyChainCaught = error === proxyMarker;
}
same(proxyChainCaught, true, "Proxy chain abrupt");

true;
