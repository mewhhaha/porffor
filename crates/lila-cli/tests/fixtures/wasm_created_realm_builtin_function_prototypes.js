let $262 = { createRealm: __lilaCreateRealm };
let first = $262.createRealm().global;
let second = $262.createRealm().global;

function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
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
assertSame(
  Object.getPrototypeOf(first.Function.prototype),
  first.Object.prototype,
  "Function.prototype object prototype",
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
