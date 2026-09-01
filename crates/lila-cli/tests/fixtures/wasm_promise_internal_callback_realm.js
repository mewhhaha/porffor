function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertOtherTypeError(callback, expectedPrototype, label) {
  try {
    callback();
  } catch (error) {
    assertSame(Object.getPrototypeOf(error), expectedPrototype, label);
    return;
  }
  throw label + " did not throw";
}

function assertFunctionPrototype(callback, expectedPrototype, label) {
  assertSame(Object.getPrototypeOf(callback), expectedPrototype, label);
}

function noop() {}

var other = __lilaCreateRealm().global;
var otherFunctionPrototype = other.Function.prototype;
var otherTypeErrorPrototype = other.TypeError.prototype;

assertOtherTypeError(function() {
  other.Promise.prototype.then.call({});
}, otherTypeErrorPrototype, "borrowed Promise.then receiver TypeError realm");

assertOtherTypeError(function() {
  other.Promise.prototype.finally.call(null, noop);
}, otherTypeErrorPrototype, "borrowed Promise.finally receiver TypeError realm");

assertOtherTypeError(function() {
  other.Promise.prototype.catch.call(null, noop);
}, otherTypeErrorPrototype, "borrowed Promise.catch ToObject TypeError realm");

assertOtherTypeError(function() {
  other.Promise.prototype.catch.call({ then: 0 }, noop);
}, otherTypeErrorPrototype, "borrowed Promise.catch then TypeError realm");

var nonCallableFinallyReceiver = new other.Promise(function() {});
nonCallableFinallyReceiver.then = 0;
assertOtherTypeError(function() {
  other.Promise.prototype.finally.call(nonCallableFinallyReceiver, noop);
}, otherTypeErrorPrototype, "borrowed Promise.finally then TypeError realm");

var primitiveCatchThis;
var primitiveCatchRejected;
other.Number.prototype.then = function(onFulfilled, onRejected) {
  "use strict";
  primitiveCatchThis = this;
  assertSame(onFulfilled, undefined, "borrowed Promise.catch primitive first argument");
  primitiveCatchRejected = onRejected;
  return "primitive catch";
};
assertSame(
  other.Promise.prototype.catch.call(7, noop),
  "primitive catch",
  "borrowed Promise.catch created-Realm primitive wrapper"
);
assertSame(primitiveCatchThis, 7, "borrowed Promise.catch primitive receiver");
assertSame(primitiveCatchRejected, noop, "borrowed Promise.catch primitive rejected callback");

var catchThenGetterSentinel = {};
var catchThenGetterError;
var poisonedCatchReceiver = {};
Object.defineProperty(poisonedCatchReceiver, "then", {
  get: function() {
    throw catchThenGetterSentinel;
  }
});
try {
  other.Promise.prototype.catch.call(poisonedCatchReceiver, noop);
} catch (error) {
  catchThenGetterError = error;
}
assertSame(
  catchThenGetterError,
  catchThenGetterSentinel,
  "borrowed Promise.catch then getter abrupt completion"
);

var delegatedThenReceiver;
var delegatedThenArguments;
var delegatedThenProxy = new Proxy(function() {}, {
  apply: function(target, receiver, argumentsList) {
    delegatedThenReceiver = receiver;
    delegatedThenArguments = argumentsList;
    return "proxy delegated then";
  }
});
var proxyCatchReceiver = { then: delegatedThenProxy };
assertSame(
  other.Promise.prototype.catch.call(proxyCatchReceiver, noop),
  "proxy delegated then",
  "borrowed Promise.catch callable Proxy result"
);
assertSame(
  delegatedThenReceiver,
  proxyCatchReceiver,
  "borrowed Promise.catch callable Proxy receiver"
);
assertSame(delegatedThenArguments.length, 2, "borrowed Promise.catch callable Proxy argument count");
assertSame(delegatedThenArguments[0], undefined, "borrowed Promise.catch callable Proxy first argument");
assertSame(delegatedThenArguments[1], noop, "borrowed Promise.catch callable Proxy rejected callback");

var observedResolve;
var observedReject;
var observedPromise = new other.Promise(function(resolve, reject) {
  observedResolve = resolve;
  observedReject = reject;
});
assertFunctionPrototype(observedResolve, otherFunctionPrototype, "resolving function prototypes resolve");
assertFunctionPrototype(observedReject, otherFunctionPrototype, "resolving function prototypes reject");

var capabilityExecutor;
function CapabilityConstructor(executor) {
  capabilityExecutor = executor;
  executor(noop, noop);
}
other.Promise.resolve.call(CapabilityConstructor, 1);
assertFunctionPrototype(capabilityExecutor, otherFunctionPrototype, "capability executor prototype");
assertOtherTypeError(function() {
  capabilityExecutor(noop, noop);
}, otherTypeErrorPrototype, "capability executor TypeError realm");

function NotPromise(executor) {
  executor(noop, noop);
}
NotPromise.resolve = function(value) {
  return value;
};

function captureThenable(target) {
  return {
    then: function(resolve, reject) {
      target.resolve = resolve;
      target.reject = reject;
    }
  };
}

var allCallbacks = {};
other.Promise.all.call(NotPromise, [captureThenable(allCallbacks)]);
assertFunctionPrototype(allCallbacks.resolve, otherFunctionPrototype, "standard combinator function prototypes all resolve");

var allSettledCallbacks = {};
other.Promise.allSettled.call(NotPromise, [captureThenable(allSettledCallbacks)]);
assertFunctionPrototype(allSettledCallbacks.resolve, otherFunctionPrototype, "standard combinator function prototypes allSettled resolve");
assertFunctionPrototype(allSettledCallbacks.reject, otherFunctionPrototype, "standard combinator function prototypes allSettled reject");

var anyCallbacks = {};
other.Promise.any.call(NotPromise, [captureThenable(anyCallbacks)]);
assertFunctionPrototype(anyCallbacks.reject, otherFunctionPrototype, "standard combinator function prototypes any reject");

var allKeyedCallbacks = {};
other.Promise.allKeyed.call(NotPromise, { key: captureThenable(allKeyedCallbacks) });
assertFunctionPrototype(allKeyedCallbacks.resolve, otherFunctionPrototype, "keyed combinator function prototypes all resolve");

var allSettledKeyedCallbacks = {};
other.Promise.allSettledKeyed.call(NotPromise, {
  key: captureThenable(allSettledKeyedCallbacks)
});
assertFunctionPrototype(allSettledKeyedCallbacks.resolve, otherFunctionPrototype, "keyed combinator function prototypes allSettled resolve");
assertFunctionPrototype(allSettledKeyedCallbacks.reject, otherFunctionPrototype, "keyed combinator function prototypes allSettled reject");

var finallyContinuations = [];
var cleanupPromise = {
  then: function(continuation) {
    finallyContinuations.push(continuation);
    return {};
  }
};
function FinallySpecies(executor) {
  executor(noop, noop);
  return cleanupPromise;
}
FinallySpecies[Symbol.species] = FinallySpecies;

var thenFinally;
var catchFinally;
var finallyReceiver = {
  constructor: FinallySpecies,
  then: function(onFulfilled, onRejected) {
    thenFinally = onFulfilled;
    catchFinally = onRejected;
    return {};
  }
};
other.Promise.prototype.finally.call(finallyReceiver, function() {
  return {};
});
assertFunctionPrototype(thenFinally, otherFunctionPrototype, "finally outer function prototypes then");
assertFunctionPrototype(catchFinally, otherFunctionPrototype, "finally outer function prototypes catch");

thenFinally("kept value");
catchFinally("kept reason");
assertSame(finallyContinuations.length, 2, "finally continuation count");
assertFunctionPrototype(finallyContinuations[0], otherFunctionPrototype, "finally continuation prototypes value thunk");
assertFunctionPrototype(finallyContinuations[1], otherFunctionPrototype, "finally continuation prototypes thrower");
assertSame(finallyContinuations[0](), "kept value", "finally value thunk result");
try {
  finallyContinuations[1]();
  throw "finally thrower did not throw";
} catch (reason) {
  assertSame(reason, "kept reason", "finally thrower result");
}

function MissingCapabilitySpecies(executor) {}
MissingCapabilitySpecies[Symbol.species] = MissingCapabilitySpecies;

var missingCapabilityThenFinally;
var missingCapabilityReceiver = {
  constructor: MissingCapabilitySpecies,
  then: function(onFulfilled) {
    missingCapabilityThenFinally = onFulfilled;
    return {};
  }
};
other.Promise.prototype.finally.call(missingCapabilityReceiver, function() {
  return {};
});
assertOtherTypeError(function() {
  missingCapabilityThenFinally("unreachable value");
}, otherTypeErrorPrototype, "borrowed Promise.finally PromiseResolve TypeError realm");

function entryPromiseWithConstructor(constructor) {
  var promise = Promise.resolve(1);
  Object.defineProperty(promise, "constructor", {
    value: constructor,
    configurable: true
  });
  return promise;
}

var defaultSpeciesPromise = other.Promise.prototype.then.call(
  entryPromiseWithConstructor(undefined),
  noop
);
assertSame(
  Object.getPrototypeOf(defaultSpeciesPromise),
  other.Promise.prototype,
  "borrowed Promise.then default species constructor realm"
);

assertOtherTypeError(function() {
  other.Promise.prototype.then.call(entryPromiseWithConstructor(0), noop);
}, otherTypeErrorPrototype, "borrowed Promise.then constructor TypeError realm");

var invalidSpeciesConstructor = {};
invalidSpeciesConstructor[Symbol.species] = 0;
assertOtherTypeError(function() {
  other.Promise.prototype.finally.call(
    entryPromiseWithConstructor(invalidSpeciesConstructor),
    noop
  );
}, otherTypeErrorPrototype, "borrowed Promise.finally species TypeError realm");

var promiseTryCallbackChecked = false;
other.Promise.try(0).then(undefined, function(error) {
  assertSame(Object.getPrototypeOf(error), otherTypeErrorPrototype, "Promise.try callback TypeError realm");
  promiseTryCallbackChecked = true;
});

observedResolve(observedPromise);
observedPromise.then(undefined, function(error) {
  assertSame(Object.getPrototypeOf(error), otherTypeErrorPrototype, "Promise self-resolution TypeError realm");
  assertSame(promiseTryCallbackChecked, true, "Promise.try callback rejection checkpoint");
  print("promise-internal-callback-realm:ok");
});

true;
