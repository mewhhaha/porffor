"use strict";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var mapGroups = Map.groupBy([-0, 0, 1, 2, 3], function (value) {
  return value === 0 ? value : value % 2;
});
assert(mapGroups instanceof Map, "Map result brand");
assert(
  Object.getPrototypeOf(mapGroups) === Map.prototype,
  "Map result prototype",
);
assert(mapGroups.size === 2, "Map result size");
assert(mapGroups.has(0), "Map zero key");
assert(mapGroups.get(0).length === 3, "Map zero group");
assert(mapGroups.get(1)[0] === 1 && mapGroups.get(1)[1] === 3, "Map odd group");

var symbolKey = Symbol("group");
var objectGroups = Object.groupBy(
  ["proto", "symbol", "plain"],
  function (value) {
    if (value === "proto") return "__proto__";
    if (value === "symbol") return symbolKey;
    return 7;
  },
);
assert(Object.getPrototypeOf(objectGroups) === null, "Object result prototype");
assert(objectGroups.__proto__[0] === "proto", "Object __proto__ group");
assert(objectGroups[symbolKey][0] === "symbol", "Object symbol group");
assert(objectGroups["7"][0] === "plain", "Object string-converted group");

true;
