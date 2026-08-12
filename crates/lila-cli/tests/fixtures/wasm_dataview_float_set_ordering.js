function ValueError() {}

let receiverOffsetCoercions = 0;
let receiverValueCoercions = 0;
let receiverOffset = {
  valueOf: function () {
    receiverOffsetCoercions += 1;
    return 0;
  }
};
let receiverValue = {
  valueOf: function () {
    receiverValueCoercions += 1;
    return 1;
  }
};
__lilaAssertThrows(TypeError, function () {
  DataView.prototype.setFloat32.call({}, receiverOffset, receiverValue, true);
});
if (receiverOffsetCoercions !== 0) throw "receiver validated after offset";
if (receiverValueCoercions !== 0) throw "receiver validated after value";

let immutableBuffer = new ArrayBuffer(8).transferToImmutable();
let immutableView = new DataView(immutableBuffer);
let immutableOffsetCoercions = 0;
let immutableValueCoercions = 0;
__lilaAssertThrows(TypeError, function () {
  immutableView.setFloat64({
    valueOf: function () {
      immutableOffsetCoercions += 1;
      return 0;
    }
  }, {
    valueOf: function () {
      immutableValueCoercions += 1;
      return 1;
    }
  });
});
if (immutableOffsetCoercions !== 0) throw "immutable checked after offset";
if (immutableValueCoercions !== 0) throw "immutable checked after value";

let orderingView = new DataView(new ArrayBuffer(8));
let negativeIndexValueCoercions = 0;
__lilaAssertThrows(RangeError, function () {
  orderingView.setFloat16(-1, {
    valueOf: function () {
      negativeIndexValueCoercions += 1;
      return 1;
    }
  });
});
if (negativeIndexValueCoercions !== 0) throw "value converted before ToIndex";
__lilaAssertThrows(ValueError, function () {
  orderingView.setFloat32(100, {
    valueOf: function () {
      throw new ValueError();
    }
  });
});

let detachedBuffer = new ArrayBuffer(8);
let detachedView = new DataView(detachedBuffer);
__lilaDetachArrayBuffer(detachedBuffer);
__lilaAssertThrows(ValueError, function () {
  detachedView.setFloat64(0, {
    valueOf: function () {
      throw new ValueError();
    }
  });
});
let unsafeIndexValueCoercions = 0;
__lilaAssertThrows(RangeError, function () {
  detachedView.setFloat32(9007199254740992, {
    valueOf: function () {
      unsafeIndexValueCoercions += 1;
      return 1;
    }
  });
});
if (unsafeIndexValueCoercions !== 0) throw "unsafe index converted value";
__lilaAssertThrows(TypeError, function () {
  detachedView.setFloat32(9007199254740991, 0);
});
__lilaAssertThrows(RangeError, function () {
  orderingView.setFloat64(1e100, 0);
});

let detachedDuringValueBuffer = new ArrayBuffer(8);
let detachedDuringValueView = new DataView(detachedDuringValueBuffer);
__lilaAssertThrows(TypeError, function () {
  detachedDuringValueView.setFloat64(0, {
    valueOf: function () {
      __lilaDetachArrayBuffer(detachedDuringValueBuffer);
      return 1;
    }
  });
});

let shrinkingBuffer = new ArrayBuffer(16, { maxByteLength: 24 });
let fixedAfterShrink = new DataView(shrinkingBuffer, 0, 16);
__lilaAssertThrows(TypeError, function () {
  fixedAfterShrink.setFloat64(0, {
    valueOf: function () {
      shrinkingBuffer.resize(8);
      return 1;
    }
  });
});

let growingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let trackingAfterGrow = new DataView(growingBuffer);
if (trackingAfterGrow.setFloat32(4, {
  valueOf: function () {
    growingBuffer.resize(8);
    return 1.5;
  }
}, true) !== undefined) throw "RAB grow result";
if (trackingAfterGrow.getFloat32(4, true) !== 1.5) throw "RAB growth not observed";

let trackingShrinkBuffer = new ArrayBuffer(16, { maxByteLength: 16 });
let trackingAfterShrink = new DataView(trackingShrinkBuffer, 1);
__lilaAssertThrows(RangeError, function () {
  trackingAfterShrink.setFloat64(0, {
    valueOf: function () {
      trackingShrinkBuffer.resize(8);
      return 1;
    }
  });
});

let growingSharedBuffer = new SharedArrayBuffer(8, { maxByteLength: 16 });
let sharedTrackingAfterGrow = new DataView(growingSharedBuffer);
if (sharedTrackingAfterGrow.setFloat64(8, {
  valueOf: function () {
    growingSharedBuffer.grow(16);
    return 2.5;
  }
}, true) !== undefined) throw "GSAB grow result";
if (sharedTrackingAfterGrow.getFloat64(8, true) !== 2.5) {
  throw "GSAB growth not observed";
}

let fixedSharedBuffer = new SharedArrayBuffer(8, { maxByteLength: 16 });
let sharedFixedAfterGrow = new DataView(fixedSharedBuffer, 0, 8);
__lilaAssertThrows(RangeError, function () {
  sharedFixedAfterGrow.setFloat64(8, {
    valueOf: function () {
      fixedSharedBuffer.grow(16);
      return 1;
    }
  });
});

let endianBuffer = new ArrayBuffer(8);
let endianView = new DataView(endianBuffer);
let endianBytes = new Uint8Array(endianBuffer);
if (endianView.setFloat16(0, 1.00048828125, false) !== undefined) {
  throw "Float16 result";
}
if (endianView.getFloat16(0, false) !== 1) throw "Float16 even tie";
endianView.setFloat16(0, 1.0004882812500002, false);
if (endianView.getFloat16(0, false) !== 1.0009765625) {
  throw "Float16 above midpoint";
}
endianView.setFloat16(0, 1.0004882812499998, false);
if (endianView.getFloat16(0, false) !== 1) {
  throw "Float16 below midpoint";
}
endianView.setFloat16(0, 1.00146484375, false);
if (endianView.getFloat16(0, false) !== 1.001953125) throw "Float16 odd tie";
endianView.setFloat16(0, NaN, false);
if (endianView.getFloat16(0, false) === endianView.getFloat16(0, false)) {
  throw "Float16 NaN";
}
endianView.setFloat16(0, Infinity, false);
if (endianView.getFloat16(0, false) !== Infinity) throw "Float16 infinity";
endianView.setFloat16(0, 1.5, false);
if (endianBytes[0] !== 0x3e || endianBytes[1] !== 0x00) {
  throw "Float16 big endian bytes";
}
endianView.setFloat16(0, 1.5, true);
if (endianBytes[0] !== 0x00 || endianBytes[1] !== 0x3e) {
  throw "Float16 little endian bytes";
}
endianView.setFloat32(0, -0, true);
if (1 / endianView.getFloat32(0, true) !== -Infinity) throw "Float32 negative zero";
endianView.setFloat32(0, 1.5, false);
if (endianBytes[0] !== 0x3f || endianBytes[1] !== 0xc0 ||
    endianBytes[2] !== 0x00 || endianBytes[3] !== 0x00) {
  throw "Float32 big endian bytes";
}
endianView.setFloat32(0, 1.5, true);
if (endianBytes[0] !== 0x00 || endianBytes[1] !== 0x00 ||
    endianBytes[2] !== 0xc0 || endianBytes[3] !== 0x3f) {
  throw "Float32 little endian bytes";
}
endianView.setFloat64(0, 1.25, false);
if (endianView.getFloat64(0, false) !== 1.25) throw "Float64 big endian";
if (endianBytes[0] !== 0x3f || endianBytes[1] !== 0xf4 ||
    endianBytes[2] !== 0x00 || endianBytes[3] !== 0x00 ||
    endianBytes[4] !== 0x00 || endianBytes[5] !== 0x00 ||
    endianBytes[6] !== 0x00 || endianBytes[7] !== 0x00) {
  throw "Float64 big endian bytes";
}
endianView.setFloat64(0, -2.5, true);
if (endianView.getFloat64(0, true) !== -2.5) throw "Float64 little endian";
endianView.setFloat64(0, 1.25, true);
if (endianBytes[0] !== 0x00 || endianBytes[1] !== 0x00 ||
    endianBytes[2] !== 0x00 || endianBytes[3] !== 0x00 ||
    endianBytes[4] !== 0x00 || endianBytes[5] !== 0x00 ||
    endianBytes[6] !== 0xf4 || endianBytes[7] !== 0x3f) {
  throw "Float64 little endian bytes";
}

3272;
