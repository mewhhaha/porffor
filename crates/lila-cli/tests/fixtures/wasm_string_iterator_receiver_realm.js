function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var other = __lilaCreateRealm().global;
var iterator = other.String.prototype[Symbol.iterator].call("xy");
var next = Object.getPrototypeOf(iterator).next;
var result = next.call(iterator);

assert(result.value === "x", "String iterator result value");
assert(result.done === false, "String iterator result completion");
assert(
  Object.getPrototypeOf(result) === other.Object.prototype,
  "String iterator result defining realm"
);

var incompatibleReceiverError = null;
try {
  next.call({});
} catch (error) {
  incompatibleReceiverError = error;
}

assert(incompatibleReceiverError !== null, "incompatible receiver did not throw");
assert(
  incompatibleReceiverError.message ===
    "String Iterator next called on incompatible receiver",
  "incompatible receiver message"
);
assert(
  incompatibleReceiverError instanceof other.TypeError,
  "incompatible receiver defining realm"
);
assert(
  !(incompatibleReceiverError instanceof TypeError),
  "incompatible receiver not entry realm"
);

true;
