function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function captureThrow(run, message) {
  try {
    run();
  } catch (error) {
    return error;
  }
  throw new Error(message);
}

let mainElement = { toLocaleString: 0 };
let mainError = captureThrow(function () {
  Array.prototype.toLocaleString.call([mainElement]);
}, "main Array non-callable element did not throw");
assert(mainError instanceof TypeError, "main Array TypeError realm");

let other = __lilaCreateRealm().global;
let otherArrayToLocaleString = other.Array.prototype.toLocaleString;
assert(
  typeof otherArrayToLocaleString === "function" &&
    otherArrayToLocaleString !== Array.prototype.toLocaleString,
  "created realm Array toLocaleString identity"
);

let otherArrayError = captureThrow(function () {
  otherArrayToLocaleString.call([{ toLocaleString: 0 }]);
}, "other Array non-callable element did not throw");
assert(
  Object.getPrototypeOf(otherArrayError) === other.TypeError.prototype &&
    otherArrayError instanceof other.TypeError &&
    !(otherArrayError instanceof TypeError),
  "other Array TypeError realm"
);

let otherTypedArrayToLocaleString = other.Uint8Array.prototype.toLocaleString;
assert(
  typeof otherTypedArrayToLocaleString === "function",
  "created realm TypedArray toLocaleString"
);
let originalNumberToLocaleString = other.Number.prototype.toLocaleString;
other.Number.prototype.toLocaleString = 0;
let otherTypedArrayError;
try {
  otherTypedArrayToLocaleString.call(new other.Uint8Array([1]));
} catch (error) {
  otherTypedArrayError = error;
}
other.Number.prototype.toLocaleString = originalNumberToLocaleString;
assert(otherTypedArrayError !== undefined, "other TypedArray did not throw");
assert(
  Object.getPrototypeOf(otherTypedArrayError) === other.TypeError.prototype &&
    otherTypedArrayError instanceof other.TypeError &&
    !(otherTypedArrayError instanceof TypeError),
  "other TypedArray TypeError realm"
);

let proxyReceiver;
let proxyArgumentCount = -1;
let callableProxy = new Proxy(function () {
  throw new Error("callable Proxy target should not run directly");
}, {
  apply: function (target, receiver, args) {
    proxyReceiver = receiver;
    proxyArgumentCount = args.length;
    return "proxy";
  }
});
let proxyElement = { toLocaleString: callableProxy };
assert(
  Array.prototype.toLocaleString.call([proxyElement]) === "proxy",
  "callable Proxy result"
);
assert(proxyReceiver === proxyElement, "callable Proxy receiver");
assert(proxyArgumentCount === 0, "callable Proxy argument count");

let revoked = Proxy.revocable(function () {
  return "unreachable";
}, {});
let revokedElement = { toLocaleString: revoked.proxy };
revoked.revoke();
assert(
  typeof revoked.proxy === "function",
  "revoked Proxy retains callable shape"
);
let revokedError = captureThrow(function () {
  Array.prototype.toLocaleString.call([revokedElement]);
}, "revoked callable Proxy did not throw");
assert(revokedError instanceof TypeError, "revoked callable Proxy Call");

true;
