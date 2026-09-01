function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertDataDescriptor(object, key, value, label) {
  var descriptor = Object.getOwnPropertyDescriptor(object, key);
  assertSame(descriptor.value, value, label + " value");
  assertSame(descriptor.writable, true, label + " writable");
  assertSame(descriptor.enumerable, false, label + " enumerable");
  assertSame(descriptor.configurable, true, label + " configurable");
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

var other = __lilaCreateRealm().global;
var otherPromise = other.Promise;
var otherPrototype = otherPromise.prototype;

assertSame(otherPromise === Promise, false, "created realm Promise identity");
assertSame(Object.getPrototypeOf(otherPromise), other.Function.prototype, "Promise function realm");
assertSame(Object.getPrototypeOf(otherPrototype), other.Object.prototype, "Promise prototype realm");
assertSame(otherPromise.name, "Promise", "Promise name");
assertSame(otherPromise.length, 1, "Promise length");
assertDataDescriptor(other, "Promise", otherPromise, "created realm Promise global descriptor");

var constructorPrototypeDescriptor = Object.getOwnPropertyDescriptor(otherPromise, "prototype");
assertSame(constructorPrototypeDescriptor.value, otherPrototype, "Promise prototype descriptor value");
assertSame(constructorPrototypeDescriptor.writable, false, "Promise prototype descriptor writable");
assertSame(constructorPrototypeDescriptor.enumerable, false, "Promise prototype descriptor enumerable");
assertSame(constructorPrototypeDescriptor.configurable, false, "Promise prototype descriptor configurable");
assertDataDescriptor(otherPrototype, "constructor", otherPromise, "Promise prototype constructor descriptor");

var prototypeMethodNames = ["then", "catch", "finally"];
var prototypeMethodLengths = [2, 1, 1];
for (var i = 0; i < prototypeMethodNames.length; i += 1) {
  var prototypeMethodName = prototypeMethodNames[i];
  var prototypeMethod = otherPrototype[prototypeMethodName];
  assertSame(prototypeMethod === Promise.prototype[prototypeMethodName], false, prototypeMethodName + " identity");
  assertSame(Object.getPrototypeOf(prototypeMethod), other.Function.prototype, prototypeMethodName + " function realm");
  assertSame(prototypeMethod.name, prototypeMethodName, prototypeMethodName + " name");
  assertSame(prototypeMethod.length, prototypeMethodLengths[i], prototypeMethodName + " length");
  assertDataDescriptor(otherPrototype, prototypeMethodName, prototypeMethod, prototypeMethodName + " descriptor");
}

var staticMethodNames = [
  "resolve",
  "reject",
  "all",
  "allSettled",
  "allKeyed",
  "allSettledKeyed",
  "any",
  "race",
  "withResolvers",
  "try"
];
var staticMethodLengths = [1, 1, 1, 1, 1, 1, 1, 1, 0, 1];
for (var j = 0; j < staticMethodNames.length; j += 1) {
  var staticMethodName = staticMethodNames[j];
  var staticMethod = otherPromise[staticMethodName];
  assertSame(staticMethod === Promise[staticMethodName], false, staticMethodName + " identity");
  assertSame(Object.getPrototypeOf(staticMethod), other.Function.prototype, staticMethodName + " function realm");
  assertSame(staticMethod.name, staticMethodName, staticMethodName + " name");
  assertSame(staticMethod.length, staticMethodLengths[j], staticMethodName + " length");
  assertDataDescriptor(otherPromise, staticMethodName, staticMethod, staticMethodName + " descriptor");
}

var tagDescriptor = Object.getOwnPropertyDescriptor(otherPrototype, Symbol.toStringTag);
assertSame(tagDescriptor.value, "Promise", "created realm Promise toStringTag descriptor");
assertSame(tagDescriptor.writable, false, "Promise tag writable");
assertSame(tagDescriptor.enumerable, false, "Promise tag enumerable");
assertSame(tagDescriptor.configurable, true, "Promise tag configurable");

var speciesDescriptor = Object.getOwnPropertyDescriptor(otherPromise, Symbol.species);
assertSame(typeof speciesDescriptor.get, "function", "Promise species getter");
assertSame(speciesDescriptor.set, undefined, "Promise species setter");
assertSame(speciesDescriptor.enumerable, false, "Promise species enumerable");
assertSame(speciesDescriptor.configurable, true, "Promise species configurable");
assertSame(Object.getPrototypeOf(speciesDescriptor.get), other.Function.prototype, "Promise species function realm");
assertSame(speciesDescriptor.get.name, "get [Symbol.species]", "Promise species name");
assertSame(speciesDescriptor.get.length, 0, "Promise species length");
assertSame(speciesDescriptor.get.call(otherPromise), otherPromise, "Promise species receiver");

var observedResolve;
var observedReject;
var promise = new otherPromise(function(resolve, reject) {
  observedResolve = resolve;
  observedReject = reject;
  resolve(7);
});
assertSame(Object.getPrototypeOf(promise), otherPrototype, "created Promise allocation prototype");
assertSame(promise.constructor, otherPromise, "created Promise constructor identity");
assertSame(Object.getPrototypeOf(observedResolve), other.Function.prototype, "created Promise resolve function realm");
assertSame(Object.getPrototypeOf(observedReject), other.Function.prototype, "created Promise reject function realm");
assertSame(observedResolve === observedReject, false, "created Promise resolving function identities");

var resolved = otherPromise.resolve(9);
assertSame(Object.getPrototypeOf(resolved), otherPrototype, "created Promise.resolve allocation prototype");

var borrowedWithResolvers = otherPromise.withResolvers.call(Promise);
assertSame(Object.getPrototypeOf(borrowedWithResolvers), other.Object.prototype, "borrowed Promise.withResolvers result object realm");
assertSame(Object.getPrototypeOf(borrowedWithResolvers.promise), Promise.prototype, "borrowed Promise.withResolvers constructor promise realm");
assertSame(Object.getPrototypeOf(borrowedWithResolvers.resolve), Function.prototype, "borrowed Promise.withResolvers resolve function realm");
assertSame(Object.getPrototypeOf(borrowedWithResolvers.reject), Function.prototype, "borrowed Promise.withResolvers reject function realm");

var entryWithResolvers = Promise.withResolvers.call(otherPromise);
assertSame(Object.getPrototypeOf(entryWithResolvers), Object.prototype, "entry Promise.withResolvers result object realm");
assertSame(Object.getPrototypeOf(entryWithResolvers.promise), otherPrototype, "entry Promise.withResolvers constructor promise realm");
assertSame(Object.getPrototypeOf(entryWithResolvers.resolve), other.Function.prototype, "entry Promise.withResolvers resolve function realm");
assertSame(Object.getPrototypeOf(entryWithResolvers.reject), other.Function.prototype, "entry Promise.withResolvers reject function realm");

assertOtherTypeError(function() {
  otherPromise(function() {});
}, other.TypeError.prototype, "created Promise constructor TypeError realm");

true;
