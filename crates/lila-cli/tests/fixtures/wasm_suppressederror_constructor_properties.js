function checkDesc(desc, value, writable, enumerable, configurable, label) {
  if (desc === undefined) throw label + ":missing";
  if (desc.value !== value) throw label + ":value";
  if (desc.writable !== writable) throw label + ":writable";
  if (desc.enumerable !== enumerable) throw label + ":enumerable";
  if (desc.configurable !== configurable) throw label + ":configurable";
}

var error = { name: "error" };
var suppressed = { name: "suppressed" };
var sample = new SuppressedError(error, suppressed, "message");

if (!(sample instanceof SuppressedError)) throw "instanceof";
if (Object.getPrototypeOf(sample) !== SuppressedError.prototype) throw "prototype";

checkDesc(
  Object.getOwnPropertyDescriptor(sample, "message"),
  "message",
  true,
  false,
  true,
  "message"
);
checkDesc(
  Object.getOwnPropertyDescriptor(sample, "error"),
  error,
  true,
  false,
  true,
  "error"
);
checkDesc(
  Object.getOwnPropertyDescriptor(sample, "suppressed"),
  suppressed,
  true,
  false,
  true,
  "suppressed"
);

if (Object.getOwnPropertyDescriptor(new SuppressedError(error, suppressed), "message") !== undefined) {
  throw "implicit-message";
}
if (Object.getOwnPropertyDescriptor(new SuppressedError(error, suppressed, undefined), "message") !== undefined) {
  throw "explicit-message";
}

function findKey(keys, name) {
  for (var i = 0; i < keys.length; i++) {
    if (keys[i] === name) return i;
  }
  return -1;
}

var keys = Object.getOwnPropertyNames(new SuppressedError(error, suppressed, "ordered"));
var messageIndex = findKey(keys, "message");
var errorIndex = findKey(keys, "error");
var suppressedIndex = findKey(keys, "suppressed");
if (messageIndex === -1) throw "message-order";
if (errorIndex !== messageIndex + 1) throw "error-order";
if (suppressedIndex !== errorIndex + 1) throw "suppressed-order";

checkDesc(
  Object.getOwnPropertyDescriptor(SuppressedError, "length"),
  3,
  false,
  false,
  true,
  "length"
);
checkDesc(
  Object.getOwnPropertyDescriptor(SuppressedError, "name"),
  "SuppressedError",
  false,
  false,
  true,
  "name"
);
checkDesc(
  Object.getOwnPropertyDescriptor(this, "SuppressedError"),
  SuppressedError,
  true,
  false,
  true,
  "global"
);
checkDesc(
  Object.getOwnPropertyDescriptor(SuppressedError, "prototype"),
  SuppressedError.prototype,
  false,
  false,
  false,
  "prototype-desc"
);
checkDesc(
  Object.getOwnPropertyDescriptor(SuppressedError.prototype, "constructor"),
  SuppressedError,
  true,
  false,
  true,
  "prototype-constructor"
);
checkDesc(
  Object.getOwnPropertyDescriptor(SuppressedError.prototype, "message"),
  "",
  true,
  false,
  true,
  "prototype-message"
);
checkDesc(
  Object.getOwnPropertyDescriptor(SuppressedError.prototype, "name"),
  "SuppressedError",
  true,
  false,
  true,
  "prototype-name"
);

function checkMessage(value, expected, label) {
  checkDesc(
    Object.getOwnPropertyDescriptor(new SuppressedError(error, suppressed, value), "message"),
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

var custom = { x: 42 };
var newTarget = new Proxy(function() {}, {
  get: function(target, key) {
    if (key === "prototype") return custom;
    return target[key];
  }
});
var reflected = Reflect.construct(SuppressedError, [error, suppressed, "reflect"], newTarget);
if (Object.getPrototypeOf(reflected) !== custom) throw "reflect-prototype";
if (reflected.x !== 42) throw "reflect-prototype-property";

true;
