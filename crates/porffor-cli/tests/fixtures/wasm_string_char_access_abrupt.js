function check(value, label) {
  if (!value) {
    throw "String char access abrupt fixture failed: " + label;
  }
}

function expectExactThrow(expected, callback, label) {
  var threw = false;
  try {
    callback();
  } catch (error) {
    threw = true;
    check(error === expected, label + " identity");
  }
  check(threw, label + " did not throw");
}

var sameFunctionCharAtCaught = false;
var sameFunctionCharAtReceiver = {
  toString: function() {
    throw "same-function charAt receiver";
  },
  charAt: String.prototype.charAt
};
try {
  sameFunctionCharAtReceiver.charAt(0);
} catch (error) {
  sameFunctionCharAtCaught = error === "same-function charAt receiver";
}
check(sameFunctionCharAtCaught, "same-function charAt receiver");

var sameFunctionCharCodeAtCaught = false;
var sameFunctionCharCodeAtReceiver = {
  toString: function() {
    throw "same-function charCodeAt receiver";
  },
  charCodeAt: String.prototype.charCodeAt
};
try {
  sameFunctionCharCodeAtReceiver.charCodeAt(0);
} catch (error) {
  sameFunctionCharCodeAtCaught = error === "same-function charCodeAt receiver";
}
check(sameFunctionCharCodeAtCaught, "same-function charCodeAt receiver");

var directCharAtReceiverSentinel = {};
var directCharAtReceiver = {
  toString: function() {
    throw directCharAtReceiverSentinel;
  },
  charAt: String.prototype.charAt
};
expectExactThrow(directCharAtReceiverSentinel, function() {
  directCharAtReceiver.charAt(0);
}, "direct charAt receiver");

var directCharAtPositionSentinel = {};
var directCharAtPosition = {
  valueOf: function() {
    throw directCharAtPositionSentinel;
  }
};
expectExactThrow(directCharAtPositionSentinel, function() {
  "abc".charAt(directCharAtPosition);
}, "direct charAt position");

var borrowedCharAtReceiverSentinel = {};
var borrowedCharAtReceiver = {
  toString: function() {
    throw borrowedCharAtReceiverSentinel;
  }
};
expectExactThrow(borrowedCharAtReceiverSentinel, function() {
  String.prototype.charAt.call(borrowedCharAtReceiver, 0);
}, "borrowed charAt receiver");

var borrowedCharAtPositionSentinel = {};
var borrowedCharAtPosition = {
  valueOf: function() {
    throw borrowedCharAtPositionSentinel;
  }
};
expectExactThrow(borrowedCharAtPositionSentinel, function() {
  String.prototype.charAt.call("abc", borrowedCharAtPosition);
}, "borrowed charAt position");

var directAtIndexSentinel = {};
var directAtIndex = {
  valueOf: function() {
    throw directAtIndexSentinel;
  }
};
expectExactThrow(directAtIndexSentinel, function() {
  "abc".at(directAtIndex);
}, "direct at index");

var borrowedAtReceiverSentinel = {};
var borrowedAtReceiver = {
  toString: function() {
    throw borrowedAtReceiverSentinel;
  }
};
expectExactThrow(borrowedAtReceiverSentinel, function() {
  String.prototype.at.call(borrowedAtReceiver, 0);
}, "borrowed at receiver");

var directCharCodeAtReceiverSentinel = {};
var directCharCodeAtReceiver = {
  toString: function() {
    throw directCharCodeAtReceiverSentinel;
  },
  charCodeAt: String.prototype.charCodeAt
};
expectExactThrow(directCharCodeAtReceiverSentinel, function() {
  directCharCodeAtReceiver.charCodeAt(0);
}, "direct charCodeAt receiver");

var borrowedCharCodeAtIndexSentinel = {};
var borrowedCharCodeAtIndex = {
  valueOf: function() {
    throw borrowedCharCodeAtIndexSentinel;
  }
};
expectExactThrow(borrowedCharCodeAtIndexSentinel, function() {
  String.prototype.charCodeAt.call("abc", borrowedCharCodeAtIndex);
}, "borrowed charCodeAt index");

var directCodePointAtReceiverSentinel = {};
var directCodePointAtReceiver = {
  toString: function() {
    throw directCodePointAtReceiverSentinel;
  },
  codePointAt: String.prototype.codePointAt
};
expectExactThrow(directCodePointAtReceiverSentinel, function() {
  directCodePointAtReceiver.codePointAt(0);
}, "direct codePointAt receiver");

var borrowedCodePointAtIndexSentinel = {};
var borrowedCodePointAtIndex = {
  valueOf: function() {
    throw borrowedCodePointAtIndexSentinel;
  }
};
expectExactThrow(borrowedCodePointAtIndexSentinel, function() {
  String.prototype.codePointAt.call("abc", borrowedCodePointAtIndex);
}, "borrowed codePointAt index");

true;
