function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function assertSame(actual, expected, message) {
  if (actual !== expected) throw new Error(message);
}

function assertDataDescriptor(
  object,
  key,
  expectedValue,
  expectedWritable,
  expectedConfigurable,
  message
) {
  var descriptor = Object.getOwnPropertyDescriptor(object, key);
  assert(descriptor !== undefined, message + " exists");
  assertSame(descriptor.value, expectedValue, message + " value");
  assertSame(descriptor.writable, expectedWritable, message + " writable");
  assertSame(descriptor.enumerable, false, message + " enumerable");
  assertSame(
    descriptor.configurable,
    expectedConfigurable,
    message + " configurable"
  );
}

function assertBuiltinFunction(value, realm, name, length, message) {
  assertSame(typeof value, "function", message + " typeof");
  assertSame(Object.getPrototypeOf(value), realm.Function.prototype, message + " Realm");
  assertDataDescriptor(value, "name", name, false, true, message + " name descriptor");
  assertDataDescriptor(value, "length", length, false, true, message + " length descriptor");
}

function expectOtherTypeError(action, message) {
  var thrown;
  try {
    action();
  } catch (error) {
    thrown = error;
  }
  assert(thrown !== undefined, message + " did not throw");
  assertSame(
    Object.getPrototypeOf(thrown),
    other.TypeError.prototype,
    message + " defining Realm"
  );
  assert(thrown instanceof other.TypeError, message + " other TypeError");
  assert(!(thrown instanceof TypeError), message + " not entry TypeError");
}

var other = __lilaCreateRealm().global;
var otherConstructor = other.FinalizationRegistry;
var otherPrototype = otherConstructor.prototype;
var otherRegister = otherPrototype.register;
var otherUnregister = otherPrototype.unregister;

assert(
  otherConstructor !== FinalizationRegistry,
  "created FinalizationRegistry constructor identity"
);
assert(
  otherPrototype !== FinalizationRegistry.prototype,
  "created FinalizationRegistry prototype identity"
);
assert(
  otherRegister !== FinalizationRegistry.prototype.register,
  "created FinalizationRegistry register identity"
);
assert(
  otherUnregister !== FinalizationRegistry.prototype.unregister,
  "created FinalizationRegistry unregister identity"
);
assertSame(
  Object.getPrototypeOf(otherPrototype),
  other.Object.prototype,
  "created FinalizationRegistry prototype parent"
);

assertDataDescriptor(
  other,
  "FinalizationRegistry",
  otherConstructor,
  true,
  true,
  "created FinalizationRegistry global descriptor"
);
assertBuiltinFunction(
  otherConstructor,
  other,
  "FinalizationRegistry",
  1,
  "created FinalizationRegistry constructor"
);
assertBuiltinFunction(
  otherRegister,
  other,
  "register",
  2,
  "created FinalizationRegistry register"
);
assertBuiltinFunction(
  otherUnregister,
  other,
  "unregister",
  1,
  "created FinalizationRegistry unregister"
);
assertDataDescriptor(
  otherConstructor,
  "prototype",
  otherPrototype,
  false,
  false,
  "created FinalizationRegistry prototype descriptor"
);
assertDataDescriptor(
  otherPrototype,
  "constructor",
  otherConstructor,
  true,
  true,
  "created FinalizationRegistry prototype constructor descriptor"
);
assertDataDescriptor(
  otherPrototype,
  "register",
  otherRegister,
  true,
  true,
  "created FinalizationRegistry register descriptor"
);
assertDataDescriptor(
  otherPrototype,
  "unregister",
  otherUnregister,
  true,
  true,
  "created FinalizationRegistry unregister descriptor"
);
assertDataDescriptor(
  otherPrototype,
  Symbol.toStringTag,
  "FinalizationRegistry",
  false,
  true,
  "created FinalizationRegistry toStringTag descriptor"
);
var finalizationRegistryPrototypeKeys = Reflect.ownKeys(otherPrototype);
assertSame(
  finalizationRegistryPrototypeKeys.length,
  4,
  "created FinalizationRegistry prototype own keys length"
);
assertSame(
  finalizationRegistryPrototypeKeys[0],
  "constructor",
  "created FinalizationRegistry prototype own keys constructor"
);
assertSame(
  finalizationRegistryPrototypeKeys[1],
  "register",
  "created FinalizationRegistry prototype own keys register"
);
assertSame(
  finalizationRegistryPrototypeKeys[2],
  "unregister",
  "created FinalizationRegistry prototype own keys unregister"
);
assertSame(
  finalizationRegistryPrototypeKeys[3],
  Symbol.toStringTag,
  "created FinalizationRegistry prototype own keys toStringTag"
);

var registry = new otherConstructor(function () {});
assertSame(
  Object.getPrototypeOf(registry),
  otherPrototype,
  "created FinalizationRegistry instance prototype"
);
var target = {};
var unregisterToken = {};
assertSame(
  otherRegister.call(registry, target, "held", unregisterToken),
  undefined,
  "created FinalizationRegistry register result"
);
assertSame(
  otherUnregister.call(registry, unregisterToken),
  true,
  "created FinalizationRegistry unregister match"
);
assertSame(
  otherUnregister.call(registry, unregisterToken),
  false,
  "created FinalizationRegistry unregister miss"
);

var entryRegistry = new FinalizationRegistry(function () {});
var entryTarget = {};
var entryToken = {};
assertSame(
  otherRegister.call(entryRegistry, entryTarget, "entry held", entryToken),
  undefined,
  "created register accepts entry FinalizationRegistry"
);
assertSame(
  otherUnregister.call(entryRegistry, entryToken),
  true,
  "created unregister accepts entry FinalizationRegistry"
);
var createdTarget = {};
var createdToken = {};
assertSame(
  FinalizationRegistry.prototype.register.call(
    registry,
    createdTarget,
    "created held",
    createdToken
  ),
  undefined,
  "entry register accepts created FinalizationRegistry"
);
assertSame(
  FinalizationRegistry.prototype.unregister.call(registry, createdToken),
  true,
  "entry unregister accepts created FinalizationRegistry"
);

expectOtherTypeError(
  function () {
    otherConstructor(function () {});
  },
  "created FinalizationRegistry requires-new TypeError"
);
expectOtherTypeError(
  function () {
    new otherConstructor(1);
  },
  "created FinalizationRegistry cleanup-callback TypeError"
);
expectOtherTypeError(
  function () {
    otherRegister.call({}, target, "held", unregisterToken);
  },
  "borrowed created FinalizationRegistry register TypeError"
);
expectOtherTypeError(
  function () {
    otherUnregister.call({}, unregisterToken);
  },
  "borrowed created FinalizationRegistry unregister TypeError"
);

var foreignNewTarget = other.Object.bind(null);
Object.defineProperty(foreignNewTarget, "prototype", {
  value: undefined,
  writable: true,
  configurable: true,
});
other.FinalizationRegistry = null;
var primitivePrototypes = [
  undefined,
  null,
  true,
  "",
  Symbol("prototype"),
  -1,
  0n,
];
for (var i = 0; i < primitivePrototypes.length; i = i + 1) {
  foreignNewTarget.prototype = primitivePrototypes[i];
  var reflected = Reflect.construct(
    FinalizationRegistry,
    [function () {}],
    foreignNewTarget
  );
  assertSame(
    Object.getPrototypeOf(reflected),
    otherPrototype,
    "foreign NewTarget private-slot FinalizationRegistry fallback"
  );
}

true;
