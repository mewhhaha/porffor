var observations = [];
var messageValue = "before";
var receiver = {
  get name() {
    observations.push("get-name");
    return {
      toString: function () {
        observations.push("string-name");
        messageValue = "after";
        return "N";
      },
    };
  },
  get message() {
    observations.push("get-message");
    var captured = messageValue;
    return {
      toString: function () {
        observations.push("string-message");
        return captured;
      },
    };
  },
};

var orderedResult = Error.prototype.toString.call(receiver);
var ordered =
  orderedResult === "N: after" &&
  observations.join(",") ===
    "get-name,string-name,get-message,string-message";

var token = {};
var messageObservedAfterThrow = false;
var preservedThrow = false;
try {
  Error.prototype.toString.call({
    name: {
      toString: function () {
        throw token;
      },
    },
    get message() {
      messageObservedAfterThrow = true;
      return "unreachable";
    },
  });
} catch (error) {
  preservedThrow = error === token;
}

var arrayReceiver = [];
arrayReceiver.name = "Array";
arrayReceiver.message = "array";
var arrayResult = Error.prototype.toString.call(arrayReceiver);

function argumentsResult() {
  arguments.name = "Arguments";
  arguments.message = "arguments";
  return Error.prototype.toString.call(arguments);
}

function callableReceiver() {}
callableReceiver.message = "function";
var functionResult = Error.prototype.toString.call(callableReceiver);

var proxyReads = [];
var proxyReceiver = new Proxy(
  { name: "Proxy", message: "proxy" },
  {
    get: function (target, key) {
      proxyReads.push(key);
      return target[key];
    },
  },
);
var proxyResult = Error.prototype.toString.call(proxyReceiver);

var other = __lilaCreateRealm().global;
var otherToString = other.Error.prototype.toString;

function throwsOtherTypeError(value) {
  try {
    otherToString.call(value);
  } catch (error) {
    return error instanceof other.TypeError && !(error instanceof TypeError);
  }
  return false;
}

var nameSymbolRealm = throwsOtherTypeError({ name: Symbol() });
var messageSymbolRealm = throwsOtherTypeError({ message: Symbol() });
var nonprimitiveRealm = throwsOtherTypeError({
  name: {
    toString: function () {
      return {};
    },
    valueOf: function () {
      return {};
    },
  },
});
var noncallableHookRealm = throwsOtherTypeError({
  name: {
    [Symbol.toPrimitive]: 1,
  },
});

ordered &&
  preservedThrow &&
  messageObservedAfterThrow === false &&
  arrayResult === "Array: array" &&
  argumentsResult() === "Arguments: arguments" &&
  functionResult === "callableReceiver: function" &&
  proxyResult === "Proxy: proxy" &&
  proxyReads.join(",") === "name,message" &&
  nameSymbolRealm &&
  messageSymbolRealm &&
  nonprimitiveRealm &&
  noncallableHookRealm;
