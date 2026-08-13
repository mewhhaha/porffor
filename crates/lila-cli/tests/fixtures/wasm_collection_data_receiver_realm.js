function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var other = __lilaCreateRealm().global;
var mapGet = other.Map.prototype.get;
var weakMapHas = other.WeakMap.prototype.has;
var setHas = other.Set.prototype.has;
var weakSetHas = other.WeakSet.prototype.has;

var messages = {
  mapNonObject: "Map method receiver is not an object",
  mapMissing: "Map method receiver does not have [[MapData]]",
  weakMapNonObject: "WeakMap method receiver is not an object",
  weakMapMissing: "WeakMap method receiver does not have [[WeakMapData]]",
  setNonObject: "Set method receiver is not an object",
  setMissing: "Set method receiver does not have [[SetData]]",
  weakSetNonObject: "WeakSet method receiver is not an object",
  weakSetMissing: "WeakSet method receiver does not have [[WeakSetData]]"
};

function expectOtherRealmTypeError(method, receiver, args, expected, label) {
  var threw = false;
  try {
    method.apply(receiver, args);
  } catch (error) {
    threw = true;
    assert(error instanceof other.TypeError, label + " defining realm");
    assert(!(error instanceof TypeError), label + " not entry realm");
    assert(error.message === expected, label + " error category");
  }
  assert(threw, label + " did not throw");
}

function makeArguments() {
  return arguments;
}

expectOtherRealmTypeError(
  mapGet,
  [],
  [0],
  messages.mapMissing,
  "Map Array"
);
expectOtherRealmTypeError(
  weakMapHas,
  function () {},
  [{}],
  messages.weakMapMissing,
  "WeakMap Function"
);
expectOtherRealmTypeError(
  setHas,
  makeArguments(1),
  [1],
  messages.setMissing,
  "Set Arguments"
);
expectOtherRealmTypeError(
  weakSetHas,
  {},
  [{}],
  messages.weakSetMissing,
  "WeakSet ordinary Object"
);

expectOtherRealmTypeError(mapGet, 0, [0], messages.mapNonObject, "Map number");
expectOtherRealmTypeError(
  weakMapHas,
  Symbol("receiver"),
  [{}],
  messages.weakMapNonObject,
  "WeakMap Symbol"
);
expectOtherRealmTypeError(setHas, 1n, [1], messages.setNonObject, "Set BigInt");
expectOtherRealmTypeError(
  weakSetHas,
  1844674407370955161600000n,
  [{}],
  messages.weakSetNonObject,
  "WeakSet heap BigInt"
);

var proxyTrapCount = 0;
function observeProxyTrap() {
  proxyTrapCount += 1;
  throw new Error("collection data brand check observed a Proxy trap");
}
var trappingHandler = {
  get: observeProxyTrap,
  getOwnPropertyDescriptor: observeProxyTrap,
  getPrototypeOf: observeProxyTrap,
  has: observeProxyTrap
};
expectOtherRealmTypeError(
  mapGet,
  new Proxy(new other.Map(), trappingHandler),
  [0],
  messages.mapMissing,
  "Map live Proxy"
);
assert(proxyTrapCount === 0, "live Proxy traps not observed");

var revoked = Proxy.revocable(new other.WeakSet(), trappingHandler);
revoked.revoke();
expectOtherRealmTypeError(
  weakSetHas,
  revoked.proxy,
  [{}],
  messages.weakSetMissing,
  "WeakSet revoked Proxy"
);
assert(proxyTrapCount === 0, "revoked Proxy traps not observed");

var map = new other.Map([[1, 2]]);
var weakMapKey = {};
var weakMap = new other.WeakMap([[weakMapKey, 3]]);
var set = new other.Set([4]);
var weakSetKey = {};
var weakSet = new other.WeakSet([weakSetKey]);
assert(mapGet.call(map, 1) === 2, "Map valid receiver");
assert(weakMapHas.call(weakMap, weakMapKey), "WeakMap valid receiver");
assert(setHas.call(set, 4), "Set valid receiver");
assert(weakSetHas.call(weakSet, weakSetKey), "WeakSet valid receiver");

true;
