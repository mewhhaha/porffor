function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

var conversionCalls = 0;
var value = {
  valueOf: function() {
    ++conversionCalls;
    return 2.3;
  },
};

var invalidTarget = new Uint8Array([0]);
var invalidReceiver = Object.setPrototypeOf([], invalidTarget);
invalidReceiver[1] = value;
assertSame(invalidTarget.hasOwnProperty(1), false, "invalid target index");
assertSame(invalidReceiver.hasOwnProperty(1), false, "invalid receiver index");
assertSame(invalidReceiver.length, 0, "invalid receiver length");

var validTarget = new Uint8Array([0]);
var validReceiver = Object.setPrototypeOf([], validTarget);
validReceiver[0] = value;
assertSame(validTarget[0], 0, "valid target value");
assertSame(validReceiver[0], value, "valid receiver value");
assertSame(validReceiver.length, 1, "valid receiver length");
assertSame(conversionCalls, 0, "value conversion calls");

true;
