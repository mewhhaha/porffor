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


true;
