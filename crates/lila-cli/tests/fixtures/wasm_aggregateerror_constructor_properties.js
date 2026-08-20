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
