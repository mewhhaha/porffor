"use strict";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function expectTypeError(action, message) {
  var threw = false;
  try {
    action();
  } catch (error) {
    threw = true;
    assert(error instanceof TypeError, message + " type");
  }
  assert(threw, message + " did not throw");
}

var map = new Map([["present", 1]]);
assert(map.getOrInsert("present", 9) === 1, "Map direct existing");
assert(map.getOrInsert("direct", 2) === 2, "Map direct result");
assert(map.get("direct") === 2, "Map direct insertion");

var mapCalls = 0;
assert(
  map.getOrInsertComputed("present", function () {
    mapCalls += 1;
    return 9;
  }) === 1,
  "Map computed existing"
);
assert(mapCalls === 0, "Map callback called for existing key");
assert(
  map.getOrInsertComputed("computed", function (key) {
    mapCalls += 1;
    assert(this === undefined, "Map callback this");
    assert(key === "computed", "Map callback key");
    map.set(key, "mutation");
    return "callback";
  }) === "callback",
  "Map computed result"
);
assert(mapCalls === 1, "Map callback count");
assert(map.get("computed") === "callback", "Map callback mutation overwrite");

var weakMap = new WeakMap();
var presentKey = {};
var directKey = {};
var computedKey = {};
weakMap.set(presentKey, 3);
assert(weakMap.getOrInsert(presentKey, 9) === 3, "WeakMap direct existing");
assert(weakMap.getOrInsert(directKey, 4) === 4, "WeakMap direct result");
assert(weakMap.get(directKey) === 4, "WeakMap direct insertion");

var weakMapCalls = 0;
assert(
  weakMap.getOrInsertComputed(presentKey, function () {
    weakMapCalls += 1;
    return 9;
  }) === 3,
  "WeakMap computed existing"
);
assert(weakMapCalls === 0, "WeakMap callback called for existing key");
assert(
  weakMap.getOrInsertComputed(computedKey, function (key) {
    weakMapCalls += 1;
    assert(this === undefined, "WeakMap callback this");
    assert(key === computedKey, "WeakMap callback key");
    weakMap.set(key, "mutation");
    return "callback";
  }) === "callback",
  "WeakMap computed result"
);
assert(weakMapCalls === 1, "WeakMap callback count");
assert(
  weakMap.get(computedKey) === "callback",
  "WeakMap callback mutation overwrite"
);

expectTypeError(
  function () {
    weakMap.getOrInsertComputed({}, 0);
  },
  "WeakMap non-callable callback"
);
expectTypeError(
  function () {
    weakMap.getOrInsertComputed(1, function () {
      weakMapCalls += 1;
      return 1;
    });
  },
  "WeakMap invalid key"
);
assert(weakMapCalls === 1, "WeakMap callback called for invalid key");

true;
