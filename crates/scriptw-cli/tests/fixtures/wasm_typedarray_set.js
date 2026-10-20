function assertSame(actual, expected, label) {
  if (actual !== expected) throw label;
}

var direct = new Uint8Array([7]);
var directConversions = 0;
var directValue = {
  valueOf: function() {
    directConversions = directConversions + 1;
    return 9;
  }
};
var invalidCanonicalKeys = ["-0", "1.1", "-1", "1"];
for (var i = 0; i < invalidCanonicalKeys.length; i = i + 1) {
  var key = invalidCanonicalKeys[i];
  direct[key] = directValue;
  assertSame(direct.hasOwnProperty(key), false, "direct invalid index " + key);
}
assertSame(directConversions, 4, "direct writes convert values");
assertSame(direct[0], 7, "direct invalid writes preserve elements");

var target = new Uint8Array([0]);
var receiver = {};
var receiverConversions = 0;
var receiverValue = {
  valueOf: function() {
    receiverConversions = receiverConversions + 1;
    return 11;
  }
};
assertSame(
  Reflect.set(target, 0, receiverValue, receiver),
  true,
  "valid altered receiver"
);
assertSame(receiver[0], receiverValue, "valid altered receiver keeps value");
assertSame(receiverConversions, 0, "valid altered receiver skips conversion");

assertSame(
  Reflect.set(target, "1.1", receiverValue, receiver),
  true,
  "invalid altered receiver"
);
assertSame(receiver.hasOwnProperty("1.1"), false, "invalid receiver property absent");
assertSame(receiverConversions, 0, "invalid altered receiver skips conversion");

var typedReceiver = new Uint8Array([0]);
assertSame(
  Reflect.set(target, 0, 257, typedReceiver),
  true,
  "valid typed array receiver"
);
assertSame(typedReceiver[0], 1, "typed array receiver converts value");

assertSame(Reflect.set(target, "1.1", receiverValue), true, "same receiver invalid index");
assertSame(receiverConversions, 1, "same receiver invalid index converts value");

var inheritedTarget = new Uint8Array([0]);
var inheritedReceiver = Object.create(inheritedTarget);
inheritedReceiver[0] = receiverValue;
assertSame(inheritedTarget[0], 0, "inherited target remains unchanged");
assertSame(inheritedReceiver[0], receiverValue, "valid inherited index defines receiver");
assertSame(receiverConversions, 1, "valid inherited index skips conversion");

inheritedReceiver["1.1"] = receiverValue;
assertSame(
  inheritedReceiver.hasOwnProperty("1.1"),
  false,
  "invalid inherited index remains absent"
);
assertSame(receiverConversions, 1, "invalid inherited index skips conversion");

var lockedReceiver = Object.preventExtensions(Object.create(inheritedTarget));
var lockedReceiverThrew = false;
try {
  (function() {
    "use strict";
    lockedReceiver[0] = receiverValue;
  })();
} catch (error) {
  lockedReceiverThrew = error instanceof TypeError;
}
assertSame(lockedReceiverThrew, true, "strict locked inherited receiver");

true;
