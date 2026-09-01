function assert(condition, label) {
  if (!condition) throw new Error(label);
}

var otherRealm = __lilaCreateRealm();
var other = otherRealm.global;
var OtherProxy = other.Proxy;
var otherRevocable = OtherProxy.revocable;

function assertOtherRealmTypeError(action, label) {
  try {
    action();
  } catch (error) {
    assert(
      Object.getPrototypeOf(error) === other.TypeError.prototype,
      label + " defining Realm"
    );
    assert(Object.getPrototypeOf(error) !== TypeError.prototype, label + " entry Realm");
    return;
  }
  throw new Error(label + " did not throw");
}

assertOtherRealmTypeError(function () {
  new OtherProxy(0, {});
}, "Proxy target");
assertOtherRealmTypeError(function () {
  new OtherProxy({}, 0);
}, "Proxy handler");
assertOtherRealmTypeError(function () {
  otherRevocable(0, {});
}, "Proxy.revocable target");
assertOtherRealmTypeError(function () {
  otherRevocable({}, 0);
}, "Proxy.revocable handler");

var revocable = otherRevocable({}, {});
assert(
  Object.getPrototypeOf(revocable) === other.Object.prototype,
  "revocable result Object prototype"
);
assert(
  Object.getPrototypeOf(revocable) !== Object.prototype,
  "revocable result entry Object prototype"
);
assert(
  Object.getPrototypeOf(revocable.revoke) === other.Function.prototype,
  "revoke Function prototype"
);
assert(
  Object.getPrototypeOf(revocable.revoke) !== Function.prototype,
  "revoke entry Function prototype"
);
assert(revocable.revoke.length === 0, "revoke length");
assert(revocable.revoke.name === "", "revoke name");
revocable.revoke();
revocable.revoke();

true;
