function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual;
  }
}

function formatPropertyName(propertyKey, objectName) {
  switch (typeof propertyKey) {
    case "symbol":
      if (Symbol.keyFor(propertyKey) !== undefined) {
        return objectName + "[Symbol.for('" + Symbol.keyFor(propertyKey) + "')]";
      } else if (propertyKey.description.startsWith("Symbol.")) {
        return objectName + "[" + propertyKey.description + "]";
      } else {
        return objectName + "[Symbol('" + propertyKey.description + "')]";
      }
    case "string":
      return objectName ? objectName + "." + propertyKey : propertyKey;
    default:
      return objectName + "[" + propertyKey + "]";
  }
}

var calls = [];
var expected = { value: 5, done: true };
var original = {
  return: function () {
    return expected;
  },
};

var method = original.return;
original.return = function () {
  calls.push("call " + formatPropertyName("return", "originalIter"));
  return method.apply(original, arguments);
};

var observed = new Proxy(original, {
  get: function (target, key, receiver) {
    calls.push("get " + formatPropertyName(key, "originalIter"));
    return Reflect.get(target, key, receiver);
  },
});

var wrapper = Iterator.from(observed);
check(calls[0], "get originalIter[Symbol.iterator]", "initial get iterator");
check(calls[1], "get originalIter.next", "initial get next");

var result = wrapper.return();
check(result, expected, "return result identity");
check(calls.length, 4, "final call length");
check(calls[2], "get originalIter.return", "return get");
check(calls[3], "call originalIter.return", "return call");
true;
