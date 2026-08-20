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

var originalNumberToString = Object.getOwnPropertyDescriptor(
  Number.prototype,
  "toString"
);
var getterReceiver;
var proxyReceiver;
var proxyArgumentCount = -1;
var callableProxy = new Proxy(
  function () {
    throw new Error("callable Proxy target should not run directly");
  },
  {
    apply: function (target, receiver, args) {
      proxyReceiver = receiver;
      proxyArgumentCount = args.length;
      return "proxy";
    }
  }
);

Object.defineProperty(Number.prototype, "toString", {
  configurable: true,
  get: function () {
    "use strict";
    getterReceiver = this;
    return callableProxy;
  }
});

var proxyResult = Object.prototype.toLocaleString.call(1);
Object.defineProperty(Number.prototype, "toString", originalNumberToString);
assert(proxyResult === "proxy", "callable Proxy result");
assert(getterReceiver === 1, "strict getter primitive receiver");
assert(proxyReceiver === 1, "callable Proxy primitive thisArgument");
assert(proxyArgumentCount === 0, "callable Proxy argument count");

var originalBooleanToString = Object.getOwnPropertyDescriptor(
  Boolean.prototype,
  "toString"
);
Boolean.prototype.toString = function () {
  "use strict";
  return typeof this;
};
var directResult = Object.prototype.toLocaleString.call(true);
var listSeparator = ["", ""].toLocaleString();
var arrayResult = [true, false].toLocaleString();
Object.defineProperty(Boolean.prototype, "toString", originalBooleanToString);
assert(directResult === "boolean", "strict method primitive receiver");
assert(
  arrayResult === "boolean" + listSeparator + "boolean",
  "Array inherited Object.toLocaleString primitive receivers"
);

var other = __lilaCreateRealm().global;
var otherObjectToLocaleString = other.Object.prototype.toLocaleString;
assert(
  typeof otherObjectToLocaleString === "function" &&
    otherObjectToLocaleString !== Object.prototype.toLocaleString,
  "created realm Object.toLocaleString identity"
);

function assertOtherTypeError(error, message) {
  assert(
    Object.getPrototypeOf(error) === other.TypeError.prototype &&
      error instanceof other.TypeError &&
      !(error instanceof TypeError),
    message
  );
}

var nullishError = captureThrow(function () {
  otherObjectToLocaleString.call(null);
}, "created realm nullish receiver did not throw");
assertOtherTypeError(nullishError, "created realm nullish TypeError");

var originalOtherNumberToString = Object.getOwnPropertyDescriptor(
  other.Number.prototype,
  "toString"
);
other.Number.prototype.toString = 0;
var nonCallableError = captureThrow(function () {
  otherObjectToLocaleString.call(2);
}, "created realm non-callable method did not throw");
Object.defineProperty(
  other.Number.prototype,
  "toString",
  originalOtherNumberToString
);
assertOtherTypeError(nonCallableError, "created realm non-callable TypeError");

true;
