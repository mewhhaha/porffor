function assert(condition, message) {
  if (!condition) throw message;
}

function capture(operation) {
  try {
    operation();
  } catch (error) {
    return error;
  }
  return undefined;
}

var symbolKeyConversions = 0;
function symbolCoercible(symbol) {
  var key = {};
  key[Symbol.toPrimitive] = function (hint) {
    assert(hint === "string", "ToPropertyKey hint");
    symbolKeyConversions += 1;
    return symbol;
  };
  return key;
}

var getSymbol = Symbol("get");
var getTarget = {};
getTarget[getSymbol] = 41;
assert(
  Reflect.get(getTarget, Object(getSymbol)) === 41,
  "Reflect.get boxed Symbol"
);
assert(
  Reflect.get(getTarget, symbolCoercible(getSymbol)) === 41,
  "Reflect.get object-to-Symbol"
);
assert(symbolKeyConversions === 1, "Reflect.get conversion count");

var hasSymbol = Symbol("has");
var hasTarget = {};
hasTarget[hasSymbol] = 42;
assert(
  Reflect.has(hasTarget, symbolCoercible(hasSymbol)),
  "Reflect.has object-to-Symbol"
);
assert(symbolKeyConversions === 2, "Reflect.has conversion count");

var defineSymbol = Symbol("define");
var defineTarget = {};
assert(
  Reflect.defineProperty(defineTarget, symbolCoercible(defineSymbol), {
    value: 43,
    configurable: true,
  }),
  "Reflect.defineProperty object-to-Symbol"
);
assert(defineTarget[defineSymbol] === 43, "Reflect.defineProperty exact Symbol");
assert(symbolKeyConversions === 3, "Reflect.defineProperty conversion count");

var deleteSymbol = Symbol("delete");
var deleteTarget = {};
deleteTarget[deleteSymbol] = 44;
assert(
  Reflect.deleteProperty(deleteTarget, symbolCoercible(deleteSymbol)),
  "Reflect.deleteProperty object-to-Symbol"
);
assert(!(deleteSymbol in deleteTarget), "Reflect.deleteProperty exact Symbol");
assert(symbolKeyConversions === 4, "Reflect.deleteProperty conversion count");

var setSymbol = Symbol("set");
function setTarget() {}
var setReceiver = [];
var setTrapCalls = 0;
var setProxy = new Proxy(setTarget, {
  set: function (target, key, value, receiver) {
    setTrapCalls += 1;
    assert(target === setTarget, "Reflect.set exact Function target");
    assert(typeof target === "function", "Reflect.set Function target tag");
    assert(key === setSymbol, "Reflect.set exact converted Symbol");
    assert(value === 45, "Reflect.set exact value");
    assert(receiver === setReceiver, "Reflect.set exact Array receiver");
    assert(Array.isArray(receiver), "Reflect.set Array receiver tag");
    return true;
  },
});
assert(
  Reflect.set(setProxy, symbolCoercible(setSymbol), 45, setReceiver),
  "Reflect.set object-to-Symbol"
);
assert(symbolKeyConversions === 5, "Reflect.set conversion count");
assert(setTrapCalls === 1, "Reflect.set trap count");

var abruptSentinel = {};
var abruptKeyConversions = 0;
var abruptKey = {};
abruptKey[Symbol.toPrimitive] = function () {
  abruptKeyConversions += 1;
  throw abruptSentinel;
};
var abruptTrapCalls = 0;
var abruptTarget = new Proxy({}, {
  get: function () {
    abruptTrapCalls += 1;
  },
  set: function () {
    abruptTrapCalls += 1;
    return true;
  },
  has: function () {
    abruptTrapCalls += 1;
    return true;
  },
  defineProperty: function () {
    abruptTrapCalls += 1;
    return true;
  },
  deleteProperty: function () {
    abruptTrapCalls += 1;
    return true;
  },
});

assert(
  capture(function () {
    Reflect.get(abruptTarget, abruptKey);
  }) === abruptSentinel,
  "Reflect.get abrupt key identity"
);
assert(
  capture(function () {
    Reflect.set(abruptTarget, abruptKey, 1);
  }) === abruptSentinel,
  "Reflect.set abrupt key identity"
);
assert(
  capture(function () {
    Reflect.has(abruptTarget, abruptKey);
  }) === abruptSentinel,
  "Reflect.has abrupt key identity"
);
assert(
  capture(function () {
    Reflect.defineProperty(abruptTarget, abruptKey, { value: 1 });
  }) === abruptSentinel,
  "Reflect.defineProperty abrupt key identity"
);
assert(
  capture(function () {
    Reflect.deleteProperty(abruptTarget, abruptKey);
  }) === abruptSentinel,
  "Reflect.deleteProperty abrupt key identity"
);
assert(abruptKeyConversions === 5, "abrupt key conversion count");
assert(abruptTrapCalls === 0, "abrupt key reached target internal method");

true;
