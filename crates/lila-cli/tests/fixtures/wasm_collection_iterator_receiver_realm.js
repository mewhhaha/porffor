function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var other = __lilaCreateRealm().global;
var otherMapNext = Object.getPrototypeOf(new other.Map().keys()).next;
var otherSetNext = Object.getPrototypeOf(new other.Set().values()).next;
var mapNonObjectMessage =
  "Map Iterator.prototype.next receiver is not an object";
var mapMissingSlotsMessage =
  "Map Iterator.prototype.next receiver does not have [[Map]]";
var setNonObjectMessage =
  "Set Iterator.prototype.next receiver is not an object";
var setMissingSlotsMessage =
  "Set Iterator.prototype.next receiver does not have [[Set]]";

function expectOtherRealmTypeError(next, receiver, expectedMessage, label) {
  var threw = false;
  try {
    next.call(receiver);
  } catch (error) {
    threw = true;
    assert(error instanceof other.TypeError, label + " defining realm");
    assert(!(error instanceof TypeError), label + " not entry realm");
    assert(error.message === expectedMessage, label + " error category");
  }
  assert(threw, label + " did not throw");
}

expectOtherRealmTypeError(otherMapNext, 0, mapNonObjectMessage, "Map number");
expectOtherRealmTypeError(
  otherMapNext,
  {},
  mapMissingSlotsMessage,
  "Map ordinary object"
);
expectOtherRealmTypeError(otherSetNext, null, setNonObjectMessage, "Set null");
expectOtherRealmTypeError(
  otherSetNext,
  {},
  setMissingSlotsMessage,
  "Set ordinary object"
);

function makeArguments() {
  return arguments;
}

expectOtherRealmTypeError(
  otherMapNext,
  [],
  mapMissingSlotsMessage,
  "Map Array object"
);
expectOtherRealmTypeError(
  otherSetNext,
  function () {},
  setMissingSlotsMessage,
  "Set Function object"
);
expectOtherRealmTypeError(
  otherMapNext,
  makeArguments(1),
  mapMissingSlotsMessage,
  "Map Arguments object"
);

var proxyTrapCount = 0;
function observeProxyTrap() {
  proxyTrapCount += 1;
  throw new Error("collection iterator brand check observed a Proxy trap");
}
var trappingHandler = {
  get: observeProxyTrap,
  getOwnPropertyDescriptor: observeProxyTrap,
  getPrototypeOf: observeProxyTrap,
  has: observeProxyTrap
};
var proxiedMapIterator = new Proxy(new other.Map().keys(), trappingHandler);
expectOtherRealmTypeError(
  otherMapNext,
  proxiedMapIterator,
  mapMissingSlotsMessage,
  "Map Proxy over iterator"
);
assert(proxyTrapCount === 0, "live Proxy traps not observed");

var revocableSetIterator = Proxy.revocable(
  new other.Set().values(),
  trappingHandler
);
revocableSetIterator.revoke();
expectOtherRealmTypeError(
  otherSetNext,
  revocableSetIterator.proxy,
  setMissingSlotsMessage,
  "Set revoked Proxy over iterator"
);
assert(proxyTrapCount === 0, "revoked Proxy traps not observed");

var mapIterator = other.Map.prototype.keys.call(new other.Map([[1, 2]]));
var setIterator = other.Set.prototype.values.call(new other.Set([3]));
assert(otherMapNext.call(mapIterator).value === 1, "Map valid receiver");
assert(otherSetNext.call(setIterator).value === 3, "Set valid receiver");

true;
