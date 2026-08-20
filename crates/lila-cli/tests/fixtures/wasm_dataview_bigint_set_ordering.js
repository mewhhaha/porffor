function assertBytes(actual, expected, label) {
  if (actual.length !== expected.length) throw label + " length";
  for (let i = 0; i < expected.length; i += 1) {
    if (actual[i] !== expected[i]) throw label + " byte " + i;
  }
}

let moduloBuffer = new ArrayBuffer(32);
let moduloView = new DataView(moduloBuffer);
let positive = 0x123456789abcdef00102030405060708n;
let negative = -0x123456789abcdef00102030405060708n;

if (moduloView.setBigUint64(0, positive, false) !== undefined) {
  throw "positive result";
}
if (moduloView.setBigInt64(8, negative, true) !== undefined) {
  throw "negative result";
}
moduloView.setBigUint64(16, {
  valueOf: function () {
    return 0xabcdef01234567898877665544332211n;
  }
}, true);
moduloView.setBigInt64(24, -0x100000000000000000000000000000002n, false);

assertBytes(new Uint8Array(moduloBuffer), [
  0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
  0xf8, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe,
  0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
  0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe
], "modulo");

let receiverOffsetCoercions = 0;
let receiverValueCoercions = 0;
__lilaAssertThrows(TypeError, function () {
  DataView.prototype.setBigUint64.call({}, {
    valueOf: function () {
      receiverOffsetCoercions += 1;
      return 0;
    }
  }, {
    valueOf: function () {
      receiverValueCoercions += 1;
      return 1n;
    }
  });
});
if (receiverOffsetCoercions !== 0) throw "receiver validated after offset";
if (receiverValueCoercions !== 0) throw "receiver validated after value";

let immutableBuffer = new ArrayBuffer(8).transferToImmutable();
let immutableView = new DataView(immutableBuffer);
let immutableOffsetCoercions = 0;
let immutableValueCoercions = 0;
__lilaAssertThrows(TypeError, function () {
  immutableView.setBigInt64({
    valueOf: function () {
      immutableOffsetCoercions += 1;
      return 0;
    }
  }, {
    valueOf: function () {
      immutableValueCoercions += 1;
      return 1n;
    }
  });
});
if (immutableOffsetCoercions !== 0) throw "immutable checked after offset";
if (immutableValueCoercions !== 0) throw "immutable checked after value";

let orderingBuffer = new ArrayBuffer(8);
let orderingView = new DataView(orderingBuffer);
let coercionOrder = 0;
orderingView.setBigUint64({
  valueOf: function () {
    if (coercionOrder !== 0) throw "offset coercion order";
    coercionOrder = 1;
    return 0;
  }
}, {
  valueOf: function () {
    if (coercionOrder !== 1) throw "value coercion order";
    coercionOrder = 2;
    return 0x123456789abcdef00102030405060708n;
  }
}, false);
if (coercionOrder !== 2) throw "missing coercion";
assertBytes(new Uint8Array(orderingBuffer), [
  0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08
], "coercion");

let negativeIndexValueCoercions = 0;
__lilaAssertThrows(RangeError, function () {
  orderingView.setBigInt64(-1.5, {
    valueOf: function () {
      negativeIndexValueCoercions += 1;
      return 1n;
    }
  });
});
if (negativeIndexValueCoercions !== 0) throw "value converted before ToIndex";

let rangeValueCoercions = 0;
__lilaAssertThrows(RangeError, function () {
  orderingView.setBigUint64(1, {
    valueOf: function () {
      rangeValueCoercions += 1;
      return positive;
    }
  });
});
if (rangeValueCoercions !== 1) throw "range checked before value";

let unsafeIndexValueCoercions = 0;
__lilaAssertThrows(RangeError, function () {
  orderingView.setBigUint64(9007199254740992, {
    valueOf: function () {
      unsafeIndexValueCoercions += 1;
      return 1n;
    }
  });
});
if (unsafeIndexValueCoercions !== 0) throw "unsafe index converted value";

let detachedBuffer = new ArrayBuffer(8);
let detachedView = new DataView(detachedBuffer);
__lilaDetachArrayBuffer(detachedBuffer);
let detachedValueCoercions = 0;
__lilaAssertThrows(TypeError, function () {
  detachedView.setBigInt64(0, {
    valueOf: function () {
      detachedValueCoercions += 1;
      return negative;
    }
  });
});
if (detachedValueCoercions !== 1) throw "detached checked before value";

let detachedDuringValueBuffer = new ArrayBuffer(8);
let detachedDuringValueView = new DataView(detachedDuringValueBuffer);
__lilaAssertThrows(TypeError, function () {
  detachedDuringValueView.setBigUint64(0, {
    valueOf: function () {
      __lilaDetachArrayBuffer(detachedDuringValueBuffer);
      return positive;
    }
  });
});

6402;
