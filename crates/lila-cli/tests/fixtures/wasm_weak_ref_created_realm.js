function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function assertDescriptor(object, key, value, writable, enumerable, configurable, message) {
  var descriptor = Object.getOwnPropertyDescriptor(object, key);
  assert(descriptor !== undefined, message + " exists");
  assert(descriptor.value === value, message + " value");
  assert(descriptor.writable === writable, message + " writable");
  assert(descriptor.enumerable === enumerable, message + " enumerable");
  assert(descriptor.configurable === configurable, message + " configurable");
}

function expectOtherTypeError(action, message) {
  var threw = false;
  try {
    action();
  } catch (error) {
    threw = true;
    assert(error instanceof other.TypeError, message + " defining Realm");
    assert(!(error instanceof TypeError), message + " not entry Realm");
  }
  assert(threw, message + " did not throw");
}

var other = __lilaCreateRealm().global;
var otherConstructor = other.WeakRef;
var otherPrototype = otherConstructor.prototype;
assert(otherConstructor !== WeakRef, "created WeakRef constructor identity");
assert(otherPrototype !== WeakRef.prototype, "created WeakRef prototype identity");
assert(
  Object.getPrototypeOf(otherConstructor) === other.Function.prototype,
  "created WeakRef constructor parent"
);
assert(
  Object.getPrototypeOf(otherPrototype) === other.Object.prototype,
  "created WeakRef prototype parent"
);
assertDescriptor(
  other,
  "WeakRef",
  otherConstructor,
  true,
  false,
  true,
  "created WeakRef global descriptor"
);
assertDescriptor(
  otherConstructor,
  "prototype",
  otherPrototype,
  false,
  false,
  false,
  "created WeakRef prototype descriptor"
);
assertDescriptor(
  otherPrototype,
  "constructor",
  otherConstructor,
  true,
  false,
  true,
  "created WeakRef prototype constructor descriptor"
);
assertDescriptor(
  otherPrototype,
  "deref",
  otherPrototype.deref,
  true,
  false,
  true,
  "created WeakRef deref descriptor"
);
assertDescriptor(
  otherPrototype,
  Symbol.toStringTag,
  "WeakRef",
  false,
  false,
  true,
  "created WeakRef toStringTag descriptor"
);
var weakRefPrototypeKeys = Reflect.ownKeys(otherPrototype);
assert(weakRefPrototypeKeys.length === 3, "created WeakRef prototype own keys length");
assert(
  weakRefPrototypeKeys[0] === "constructor" &&
    weakRefPrototypeKeys[1] === "deref" &&
    weakRefPrototypeKeys[2] === Symbol.toStringTag,
  "created WeakRef prototype own keys order"
);

var target = {};
var created = new otherConstructor(target);
assert(
  Object.getPrototypeOf(created) === otherPrototype,
  "created WeakRef instance prototype"
);
assert(created.deref() === target, "created WeakRef deref result");
assert(
  WeakRef.prototype.deref.call(created) === target,
  "entry WeakRef deref accepts created instance"
);

var newTarget = new other.Function();
other.WeakRef = null;
var primitivePrototypes = [
  undefined,
  null,
  true,
  "",
  Symbol("prototype"),
  -1,
  0n,
];
for (var i = 0; i < primitivePrototypes.length; i += 1) {
  newTarget.prototype = primitivePrototypes[i];
  var reflected = Reflect.construct(WeakRef, [target], newTarget);
  assert(
    Object.getPrototypeOf(reflected) === otherPrototype,
    "foreign NewTarget primitive prototype fallback"
  );
}

expectOtherTypeError(
  function () { otherConstructor(target); },
  "created WeakRef requires-new TypeError"
);
expectOtherTypeError(
  function () { new otherConstructor(1); },
  "created WeakRef invalid-target TypeError"
);
expectOtherTypeError(
  function () { otherPrototype.deref.call({}); },
  "borrowed created WeakRef deref TypeError"
);

true;
