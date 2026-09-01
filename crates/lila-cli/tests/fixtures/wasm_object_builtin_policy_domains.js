"use strict";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var source = { first: 11, second: 22 };
var values = Object.values(source);
var entries = Object.entries(source);
assert(values.length === 2, "values length");
assert(values[0] === 11 && values[1] === 22, "values projection");
assert(entries.length === 2, "entries length");
assert(entries[0][0] === "first" && entries[0][1] === 11, "first entry");
assert(entries[1][0] === "second" && entries[1][1] === 22, "second entry");

var entriesThrew = false;
try {
  Object.entries(null);
} catch (error) {
  entriesThrew = error instanceof TypeError;
}
assert(entriesThrew, "entries nullish rejection");

var valuesThrew = false;
try {
  Object.values(undefined);
} catch (error) {
  valuesThrew = error instanceof TypeError;
}
assert(valuesThrew, "values nullish rejection");

var sealedOnly = {};
Object.defineProperty(sealedOnly, "value", {
  value: 1,
  writable: true,
  configurable: false,
});
Object.preventExtensions(sealedOnly);
assert(Object.isSealed(sealedOnly), "sealed writable object");
assert(!Object.isFrozen(sealedOnly), "writable object is not frozen");

var frozen = Object.freeze({ value: 1 });
assert(Object.isSealed(frozen), "frozen object is sealed");
assert(Object.isFrozen(frozen), "frozen object");

function getter() {
  throw new Error("getter invoked");
}
function setter() {}

var prototype = {};
Object.defineProperty(prototype, "accessor", {
  get: getter,
  set: setter,
});
var receiver = Object.create(prototype);
assert(
  Object.prototype.__lookupGetter__.call(receiver, "accessor") === getter,
  "getter lookup",
);
assert(
  Object.prototype.__lookupSetter__.call(receiver, "accessor") === setter,
  "setter lookup",
);

true;
