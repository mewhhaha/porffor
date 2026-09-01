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
  assertSame(
    Object.getPrototypeOf(value),
    realm.Function.prototype,
    message + " Realm"
  );
  assertDataDescriptor(
    value,
    "name",
    name,
    false,
    true,
    message + " name descriptor"
  );
  assertDataDescriptor(
    value,
    "length",
    length,
    false,
    true,
    message + " length descriptor"
  );
}

function isConstructor(value) {
  try {
    Reflect.construct(Object, [], value);
    return true;
  } catch (error) {
    return false;
  }
}

function assertOwnKeys(object, expected, message) {
  var actual = Reflect.ownKeys(object);
  assertSame(actual.length, expected.length, message + " length");
  for (var i = 0; i < expected.length; i = i + 1) {
    assertSame(actual[i], expected[i], message + " key " + i);
  }
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
var otherWeakMap = other.WeakMap;
var otherWeakMapPrototype = otherWeakMap.prototype;
var otherWeakSet = other.WeakSet;
var otherWeakSetPrototype = otherWeakSet.prototype;

var weakCollectionGlobalNames = [
  "Map",
  "WeakMap",
  "WeakSet",
  "WeakRef",
  "FinalizationRegistry",
  "Set",
];
var otherGlobalKeys = Reflect.ownKeys(other);
var observedWeakCollectionGlobalNames = [];
for (var globalKeyIndex = 0; globalKeyIndex < otherGlobalKeys.length; globalKeyIndex += 1) {
  var globalKey = otherGlobalKeys[globalKeyIndex];
  for (var globalNameIndex = 0; globalNameIndex < weakCollectionGlobalNames.length; globalNameIndex += 1) {
    if (globalKey === weakCollectionGlobalNames[globalNameIndex]) {
      observedWeakCollectionGlobalNames.push(globalKey);
    }
  }
}
assertSame(
  observedWeakCollectionGlobalNames.join("|"),
  weakCollectionGlobalNames.join("|"),
  "created weak collection global catalog order"
);

assert(otherWeakMap !== WeakMap, "created WeakMap constructor identity");
assert(
  otherWeakMapPrototype !== WeakMap.prototype,
  "created WeakMap prototype identity"
);
assert(otherWeakSet !== WeakSet, "created WeakSet constructor identity");
assert(
  otherWeakSetPrototype !== WeakSet.prototype,
  "created WeakSet prototype identity"
);
assertSame(
  Object.getPrototypeOf(otherWeakMapPrototype),
  other.Object.prototype,
  "created WeakMap prototype parent"
);
assertSame(
  Object.getPrototypeOf(otherWeakSetPrototype),
  other.Object.prototype,
  "created WeakSet prototype parent"
);

assertDataDescriptor(
  other,
  "WeakMap",
  otherWeakMap,
  true,
  true,
  "created WeakMap global descriptor"
);
assertDataDescriptor(
  other,
  "WeakSet",
  otherWeakSet,
  true,
  true,
  "created WeakSet global descriptor"
);
assertBuiltinFunction(otherWeakMap, other, "WeakMap", 0, "created WeakMap");
assertBuiltinFunction(otherWeakSet, other, "WeakSet", 0, "created WeakSet");
assert(isConstructor(otherWeakMap), "created WeakMap IsConstructor");
assert(isConstructor(otherWeakSet), "created WeakSet IsConstructor");
assertDataDescriptor(
  otherWeakMap,
  "prototype",
  otherWeakMapPrototype,
  false,
  false,
  "created WeakMap prototype descriptor"
);
assertDataDescriptor(
  otherWeakSet,
  "prototype",
  otherWeakSetPrototype,
  false,
  false,
  "created WeakSet prototype descriptor"
);
assertDataDescriptor(
  otherWeakMapPrototype,
  "constructor",
  otherWeakMap,
  true,
  true,
  "created WeakMap prototype constructor descriptor"
);
assertDataDescriptor(
  otherWeakSetPrototype,
  "constructor",
  otherWeakSet,
  true,
  true,
  "created WeakSet prototype constructor descriptor"
);

var weakMapMethodNames = [
  "delete",
  "get",
  "getOrInsert",
  "getOrInsertComputed",
  "has",
  "set",
];
var weakMapMethodLengths = [1, 1, 2, 2, 1, 2];
for (var weakMapIndex = 0; weakMapIndex < weakMapMethodNames.length; weakMapIndex += 1) {
  var weakMapMethodName = weakMapMethodNames[weakMapIndex];
  var weakMapMethod = otherWeakMapPrototype[weakMapMethodName];
  assert(
    weakMapMethod !== WeakMap.prototype[weakMapMethodName],
    "created WeakMap " + weakMapMethodName + " identity"
  );
  assertBuiltinFunction(
    weakMapMethod,
    other,
    weakMapMethodName,
    weakMapMethodLengths[weakMapIndex],
    "created WeakMap " + weakMapMethodName
  );
  assertDataDescriptor(
    otherWeakMapPrototype,
    weakMapMethodName,
    weakMapMethod,
    true,
    true,
    "created WeakMap " + weakMapMethodName + " descriptor"
  );
  assert(
    !isConstructor(weakMapMethod),
    "created WeakMap " + weakMapMethodName + " non-constructable"
  );
}

var weakSetMethodNames = ["add", "delete", "has"];
var weakSetMethodLengths = [1, 1, 1];
for (var weakSetIndex = 0; weakSetIndex < weakSetMethodNames.length; weakSetIndex += 1) {
  var weakSetMethodName = weakSetMethodNames[weakSetIndex];
  var weakSetMethod = otherWeakSetPrototype[weakSetMethodName];
  assert(
    weakSetMethod !== WeakSet.prototype[weakSetMethodName],
    "created WeakSet " + weakSetMethodName + " identity"
  );
  assertBuiltinFunction(
    weakSetMethod,
    other,
    weakSetMethodName,
    weakSetMethodLengths[weakSetIndex],
    "created WeakSet " + weakSetMethodName
  );
  assertDataDescriptor(
    otherWeakSetPrototype,
    weakSetMethodName,
    weakSetMethod,
    true,
    true,
    "created WeakSet " + weakSetMethodName + " descriptor"
  );
  assert(
    !isConstructor(weakSetMethod),
    "created WeakSet " + weakSetMethodName + " non-constructable"
  );
}

assertDataDescriptor(
  otherWeakMapPrototype,
  Symbol.toStringTag,
  "WeakMap",
  false,
  true,
  "created WeakMap toStringTag descriptor"
);
assertDataDescriptor(
  otherWeakSetPrototype,
  Symbol.toStringTag,
  "WeakSet",
  false,
  true,
  "created WeakSet toStringTag descriptor"
);
assertOwnKeys(
  otherWeakMapPrototype,
  [
    "constructor",
    "delete",
    "get",
    "getOrInsert",
    "getOrInsertComputed",
    "has",
    "set",
    Symbol.toStringTag,
  ],
  "created WeakMap constructor-before-method own-key order"
);
assertOwnKeys(
  otherWeakSetPrototype,
  ["constructor", "add", "delete", "has", Symbol.toStringTag],
  "created WeakSet constructor-before-method own-key order"
);

var weakMapKey = {};
var weakMapValue = {};
var createdWeakMap = new otherWeakMap([[weakMapKey, weakMapValue]]);
assertSame(
  Object.getPrototypeOf(createdWeakMap),
  otherWeakMapPrototype,
  "created WeakMap instance prototype"
);
assertSame(
  otherWeakMapPrototype.get.call(createdWeakMap, weakMapKey),
  weakMapValue,
  "created WeakMap object-iterable construction"
);
var secondWeakMapKey = {};
assertSame(
  otherWeakMapPrototype.set.call(createdWeakMap, secondWeakMapKey, 2),
  createdWeakMap,
  "created WeakMap set returns receiver"
);
assertSame(
  otherWeakMapPrototype.has.call(createdWeakMap, secondWeakMapKey),
  true,
  "created WeakMap has inserted key"
);
assertSame(
  otherWeakMapPrototype.delete.call(createdWeakMap, secondWeakMapKey),
  true,
  "created WeakMap delete match"
);
assertSame(
  otherWeakMapPrototype.delete.call(createdWeakMap, secondWeakMapKey),
  false,
  "created WeakMap delete miss"
);

var weakSetValue = {};
var createdWeakSet = new otherWeakSet([weakSetValue]);
assertSame(
  Object.getPrototypeOf(createdWeakSet),
  otherWeakSetPrototype,
  "created WeakSet instance prototype"
);
assertSame(
  otherWeakSetPrototype.has.call(createdWeakSet, weakSetValue),
  true,
  "created WeakSet object-iterable construction"
);
var secondWeakSetValue = {};
assertSame(
  otherWeakSetPrototype.add.call(createdWeakSet, secondWeakSetValue),
  createdWeakSet,
  "created WeakSet add returns receiver"
);
assertSame(
  otherWeakSetPrototype.delete.call(createdWeakSet, secondWeakSetValue),
  true,
  "created WeakSet delete match"
);
assertSame(
  otherWeakSetPrototype.delete.call(createdWeakSet, secondWeakSetValue),
  false,
  "created WeakSet delete miss"
);

assertSame(
  WeakMap.prototype.get.call(createdWeakMap, weakMapKey),
  weakMapValue,
  "entry WeakMap method accepts created instance"
);
var entryWeakMapKey = {};
var entryWeakMap = new WeakMap([[entryWeakMapKey, 3]]);
assertSame(
  otherWeakMapPrototype.get.call(entryWeakMap, entryWeakMapKey),
  3,
  "created WeakMap method accepts entry instance"
);
assertSame(
  WeakSet.prototype.has.call(createdWeakSet, weakSetValue),
  true,
  "entry WeakSet method accepts created instance"
);
var entryWeakSetValue = {};
var entryWeakSet = new WeakSet([entryWeakSetValue]);
assertSame(
  otherWeakSetPrototype.has.call(entryWeakSet, entryWeakSetValue),
  true,
  "created WeakSet method accepts entry instance"
);

expectOtherTypeError(
  function () {
    otherWeakMap();
  },
  "created WeakMap requires-new TypeError"
);
expectOtherTypeError(
  function () {
    otherWeakSet();
  },
  "created WeakSet requires-new TypeError"
);
expectOtherTypeError(
  function () {
    otherWeakMapPrototype.set.call(createdWeakMap, 1, 1);
  },
  "created WeakMap invalid-key TypeError"
);
expectOtherTypeError(
  function () {
    otherWeakSetPrototype.add.call(createdWeakSet, 1);
  },
  "created WeakSet invalid-value TypeError"
);
expectOtherTypeError(
  function () {
    otherWeakMapPrototype.get.call({}, weakMapKey);
  },
  "borrowed created WeakMap method TypeError"
);
expectOtherTypeError(
  function () {
    otherWeakSetPrototype.has.call({}, weakSetValue);
  },
  "borrowed created WeakSet method TypeError"
);

var foreignNewTarget = other.Object.bind(null);
Object.defineProperty(foreignNewTarget, "prototype", {
  value: undefined,
  writable: true,
  configurable: true,
});
other.WeakMap = null;
other.WeakSet = null;
var primitivePrototypes = [
  undefined,
  null,
  true,
  "",
  Symbol("prototype"),
  -1,
  0n,
];
for (var prototypeIndex = 0; prototypeIndex < primitivePrototypes.length; prototypeIndex += 1) {
  foreignNewTarget.prototype = primitivePrototypes[prototypeIndex];
  var reflectedWeakMap = Reflect.construct(WeakMap, [], foreignNewTarget);
  assertSame(
    Object.getPrototypeOf(reflectedWeakMap),
    otherWeakMapPrototype,
    "private-slot foreign NewTarget primitive WeakMap fallback"
  );
  var reflectedWeakSet = Reflect.construct(WeakSet, [], foreignNewTarget);
  assertSame(
    Object.getPrototypeOf(reflectedWeakSet),
    otherWeakSetPrototype,
    "private-slot foreign NewTarget primitive WeakSet fallback"
  );
}

true;
