let $262 = { createRealm: __lilaCreateRealm };
let first = $262.createRealm().global;
let second = $262.createRealm().global;

function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

function assertDataDescriptor(object, key, value, writable, configurable, label) {
  let descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (descriptor === undefined) throw label + " missing";
  assertSame(descriptor.value, value, label + " value");
  assertSame(descriptor.writable, writable, label + " writable");
  assertSame(descriptor.enumerable, false, label + " enumerable");
  assertSame(descriptor.configurable, configurable, label + " configurable");
}

function assertCallableFunctionPrototype(value, objectPrototype, functionConstructor, label) {
  assertSame(typeof value, "function", label + " typeof");
  assertSame(Object.prototype.toString.call(value), "[object Function]", label + " tag");
  assertSame(Function.prototype.toString.call(value), "function () { [native code] }", label + " source");
  assertSame(Object.getPrototypeOf(value), objectPrototype, label + " object prototype");
  assertSame(Object.getPrototypeOf(functionConstructor), value, label + " constructor prototype");
  assertSame(value.constructor, functionConstructor, label + " constructor identity");
  assertSame(value(), undefined, label + " empty call");
  assertSame(value(null, undefined, 3), undefined, label + " argument call");

  assertDataDescriptor(value, "length", 0, false, true, label + " length");
  assertDataDescriptor(value, "name", "", false, true, label + " name");
  assertDataDescriptor(functionConstructor, "prototype", value, false, false, label + " publication");

  if (Object.prototype.hasOwnProperty.call(value, "prototype")) {
    throw label + " own prototype";
  }
  let constructed = true;
  try {
    new value();
  } catch (error) {
    assertSame(error.name, "TypeError", label + " construct error");
    constructed = false;
  }
  if (constructed) throw label + " constructable";
}

function assertRealmFunction(realm, value, label) {
  assertSame(Object.getPrototypeOf(value), realm.Function.prototype, label);
  if (Object.getPrototypeOf(value) === Function.prototype) {
    throw label + " entry realm";
  }
}

if (first.Function.prototype === Function.prototype ||
    second.Function.prototype === Function.prototype ||
    first.Function.prototype === second.Function.prototype) {
  throw "Function.prototype realm identity";
}
assertCallableFunctionPrototype(
  Function.prototype,
  Object.prototype,
  Function,
  "entry Function.prototype",
);
assertCallableFunctionPrototype(
  first.Function.prototype,
  first.Object.prototype,
  first.Function,
  "first Function.prototype",
);
assertCallableFunctionPrototype(
  second.Function.prototype,
  second.Object.prototype,
  second.Function,
  "second Function.prototype",
);

let mapSize = Object.getOwnPropertyDescriptor(first.Map.prototype, "size").get;
let firstBuiltins = [
  [first.Object, "constructor"],
  [first.String.prototype.charAt, "prototype method"],
  [mapSize, "accessor"],
  [first.Math.abs, "namespace function"],
  [first.isFinite, "global function"],
  [first.parseInt, "canonical host function"],
  [first.Function.prototype.call, "Function prototype method"],
];

for (let row of firstBuiltins) {
  assertRealmFunction(first, row[0], row[1]);
}
assertRealmFunction(second, second.String.prototype.charAt, "second realm method");
assertRealmFunction(second, second.parseFloat, "second realm canonical host function");
assertSame(first.parseInt, first.Number.parseInt, "first realm canonical identity");
assertSame(second.parseFloat, second.Number.parseFloat, "second realm canonical identity");

123;
