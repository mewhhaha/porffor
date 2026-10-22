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


// Direct insertion retains canonical keys through multiple table expansions.
var direct = new Map();
var directKeys = [];
for (var i = 0; i < 517; ++i) {
  var longKey = "key\ud800\udc00\udfff:" + "x".repeat(257) + ":" + i;
  directKeys.push(longKey);
  assert(direct.getOrInsert(longKey, i) === i, "direct growth result");
  if ((i === 31 || i === 513) && typeof gc === "function") gc();
}
for (var i = 0; i < directKeys.length; ++i) {
  var sameKey = ("!" + directKeys[i]).slice(1);
  assert(direct.getOrInsert(sameKey, -1) === i, "equal string hit");
}
assert(direct.size === 517, "direct growth size");
var keyObject = { toString() { throw new Error("key coercion"); } };
var symbol = Symbol("key");
var mixedKeys = [NaN, -0, 1, "1", 1n, symbol, keyObject, null, undefined];
for (var i = 0; i < mixedKeys.length; ++i) {
  assert(direct.getOrInsert(mixedKeys[i], i + 1000) === i + 1000, "mixed insert");
  assert(direct.getOrInsert(mixedKeys[i], -1) === i + 1000, "mixed hit");
}
assert(direct.getOrInsert(0, -1) === 1001, "positive zero hit");
assert(direct.getOrInsert(0 / 0, -1) === 1000, "NaN hit");
assert([...direct.keys()].some(key => key === 0 && 1 / key === Infinity), "stored positive zero");
var evaluated = 0;
assert(direct.getOrInsert(NaN, ++evaluated) === 1000 && evaluated === 1, "hit still evaluates value argument");
expectTypeError(() => direct.getOrInsertComputed(NaN, null), "hit validates callback");

// Callback insertion must consult the mutated owner after arbitrary user code.
var changing = new Map([["before", 1]]);
var changingKey = "long\ud800:" + "y".repeat(257);
var callbackCalls = 0;
assert(changing.getOrInsertComputed(changingKey, function (key) {
  ++callbackCalls;
  assert(key === changingKey, "callback receives key");
  changing.clear();
  for (var j = 0; j < 517; ++j) changing.set("other:" + j, j);
  changing.getOrInsert(key, "nested");
  changing.delete(key);
  changing.set(("!" + key).slice(1), "intermediate");
  if (typeof gc === "function") gc();
  return "final";
}) === "final", "callback after clear/growth/reinsert");
assert(callbackCalls === 1 && changing.size === 518, "callback resulting size");
assert(changing.get(changingKey) === "final" && !changing.has("before"), "callback resulting owner");
assert([...changing.keys()][517] === changingKey, "callback order");
assert(changing.getOrInsertComputed(-0, function (key) {
  assert(1 / key === Infinity, "callback receives normalized zero");
  return 7;
}) === 7 && changing.get(0) === 7, "callback zero");
var marker = {};
try {
  changing.getOrInsertComputed("abrupt", function (key) {
    changing.set(key, "retained side effect");
    throw marker;
  });
  throw new Error("callback should throw");
} catch (error) {
  assert(error === marker, "callback preserves thrown identity");
}
assert(changing.get("abrupt") === "retained side effect", "abrupt callback mutation remains");

true;
