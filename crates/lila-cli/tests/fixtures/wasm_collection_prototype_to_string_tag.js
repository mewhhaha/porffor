function checkCollectionTag(prototype, expected, instance) {
  var descriptor = Object.getOwnPropertyDescriptor(
    prototype,
    Symbol.toStringTag
  );
  return (
    descriptor.value === expected &&
    descriptor.writable === false &&
    descriptor.enumerable === false &&
    descriptor.configurable === true &&
    Object.prototype.toString.call(instance) === "[object " + expected + "]"
  );
}

checkCollectionTag(Map.prototype, "Map", new Map()) &&
  checkCollectionTag(Set.prototype, "Set", new Set()) &&
  checkCollectionTag(WeakMap.prototype, "WeakMap", new WeakMap()) &&
  checkCollectionTag(WeakSet.prototype, "WeakSet", new WeakSet());
