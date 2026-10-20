function readIterator(propertyKey) {
  return arguments[propertyKey];
}

readIterator(Symbol.iterator) === Array.prototype.values;
