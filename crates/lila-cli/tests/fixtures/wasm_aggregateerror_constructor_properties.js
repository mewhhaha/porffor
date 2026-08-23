function checkDesc(desc, value, writable, enumerable, configurable, label) {
  if (desc === undefined) throw label + ":missing";
  if (desc.value !== value) throw label + ":value";
  if (desc.writable !== writable) throw label + ":writable";
  if (desc.enumerable !== enumerable) throw label + ":enumerable";
  if (desc.configurable !== configurable) throw label + ":configurable";
}

var errors = [];
var message = "my-message";
var cause = { message: "my-cause" };

var caused = new AggregateError(errors, message, { cause: cause });
checkDesc(
  Object.getOwnPropertyDescriptor(caused, "cause"),
  cause,
  true,
  false,
  true,
  "cause"
);

if (Object.getOwnPropertyDescriptor(new AggregateError(errors, message), "cause") !== undefined) {
  throw "missing-cause";
}

var undefinedCause = Object.getOwnPropertyDescriptor(
  new AggregateError(errors, message, { cause: undefined }),
  "cause"
);
checkDesc(undefinedCause, undefined, true, false, true, "undefined-cause");

var sequence = "";
var orderedMessage = {
  toString: function() {
    sequence += "m";
    return "ordered-message";
  }
};
var orderedOptions = {};
Object.defineProperty(orderedOptions, "cause", {
  get: function() {
    sequence += "c";
    return "ordered-cause";
  }
});
var orderedErrors = {};
orderedErrors[Symbol.iterator] = function() {
  sequence += "i";
  return {
    next: function() {
      return { done: true };
    }
  };
};
var ordered = new AggregateError(orderedErrors, orderedMessage, orderedOptions);
if (sequence !== "mci") throw "construction-order:" + sequence;
var orderedKeys = Object.getOwnPropertyNames(ordered);
if (orderedKeys.length !== 3) throw "own-key-count:" + orderedKeys.length;
if (orderedKeys[0] !== "message") throw "own-key-message:" + orderedKeys[0];
if (orderedKeys[1] !== "cause") throw "own-key-cause:" + orderedKeys[1];
if (orderedKeys[2] !== "errors") throw "own-key-errors:" + orderedKeys[2];

var omittedSequence = "";
var omittedOptions = {};
Object.defineProperty(omittedOptions, "cause", {
  get: function() {
    omittedSequence += "c";
    return "omitted-cause";
  }
});
var omittedErrors = {};
omittedErrors[Symbol.iterator] = function() {
  omittedSequence += "i";
  return {
    next: function() {
      return { done: true };
    }
  };
};
var omitted = new AggregateError(omittedErrors, undefined, omittedOptions);
if (omittedSequence !== "ci") throw "omitted-message-order:" + omittedSequence;
if (Object.getOwnPropertyDescriptor(omitted, "message") !== undefined) {
  throw "omitted-message-present";
}
var omittedKeys = Object.getOwnPropertyNames(omitted);
if (omittedKeys.length !== 2) throw "omitted-own-key-count:" + omittedKeys.length;
if (omittedKeys[0] !== "cause") throw "omitted-own-key-cause:" + omittedKeys[0];
if (omittedKeys[1] !== "errors") throw "omitted-own-key-errors:" + omittedKeys[1];

var messageMarker = {};
var messageCauseRead = false;
var messageIteratorStarted = false;
var messageBlockedOptions = {};
Object.defineProperty(messageBlockedOptions, "cause", {
  get: function() {
    messageCauseRead = true;
    return "unreachable-cause";
  }
});
var messageBlockedErrors = {};
messageBlockedErrors[Symbol.iterator] = function() {
  messageIteratorStarted = true;
  return {
    next: function() {
      return { done: true };
    }
  };
};
var throwingMessage = {
  toString: function() {
    throw messageMarker;
  }
};
try {
  new AggregateError(messageBlockedErrors, throwingMessage, messageBlockedOptions);
  throw "message-abrupt-missing";
} catch (error) {
  if (error !== messageMarker) throw "message-abrupt-value";
}
if (messageCauseRead) throw "message-abrupt-read-cause";
if (messageIteratorStarted) throw "message-abrupt-started-iterator";

var causeMarker = {};
var iteratorStarted = false;
var blockedErrors = {};
blockedErrors[Symbol.iterator] = function() {
  iteratorStarted = true;
  return {
    next: function() {
      return { done: true };
    }
  };
};
var abruptOptions = {};
Object.defineProperty(abruptOptions, "cause", {
  get: function() {
    throw causeMarker;
  }
});
try {
  new AggregateError(blockedErrors, "message", abruptOptions);
  throw "cause-abrupt-missing";
} catch (error) {
  if (error !== causeMarker) throw "cause-abrupt-value";
}
if (iteratorStarted) throw "cause-abrupt-started-iterator";

checkDesc(
  Object.getOwnPropertyDescriptor(AggregateError, "length"),
  2,
  false,
  false,
  true,
  "length"
);
checkDesc(
  Object.getOwnPropertyDescriptor(AggregateError, "name"),
  "AggregateError",
  false,
  false,
  true,
  "name"
);
checkDesc(
  Object.getOwnPropertyDescriptor(this, "AggregateError"),
  AggregateError,
  true,
  false,
  true,
  "global"
);
checkDesc(
  Object.getOwnPropertyDescriptor(AggregateError, "prototype"),
  AggregateError.prototype,
  false,
  false,
  false,
  "prototype"
);
checkDesc(
  Object.getOwnPropertyDescriptor(AggregateError.prototype, "constructor"),
  AggregateError,
  true,
  false,
  true,
  "prototype-constructor"
);
checkDesc(
  Object.getOwnPropertyDescriptor(AggregateError.prototype, "message"),
  "",
  true,
  false,
  true,
  "prototype-message"
);
checkDesc(
  Object.getOwnPropertyDescriptor(AggregateError.prototype, "name"),
  "AggregateError",
  true,
  false,
  true,
  "prototype-name"
);

function checkMessage(value, expected, label) {
  checkDesc(
    Object.getOwnPropertyDescriptor(new AggregateError([], value), "message"),
    expected,
    true,
    false,
    true,
    label
  );
}

checkMessage("42", "42", "message-string");
checkMessage(42, "42", "message-number");
checkMessage(false, "false", "message-false");
checkMessage(true, "true", "message-true");
checkMessage({ toString: function() { return "string"; } }, "string", "message-object");
checkMessage(null, "null", "message-null");

true;
