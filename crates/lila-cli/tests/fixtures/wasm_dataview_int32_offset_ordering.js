function OffsetError() {}

let receiverOffsetCoercions = 0;
let receiverOffset = {
  valueOf: function () {
    receiverOffsetCoercions += 1;
    return 0;
  }
};
__lilaAssertThrows(TypeError, function () {
  DataView.prototype.getUint32.call({}, receiverOffset, true);
});
if (receiverOffsetCoercions !== 0) throw "receiver validated after offset";

let detachedBuffer = new ArrayBuffer(4);
let detachedView = new DataView(detachedBuffer);
__lilaDetachArrayBuffer(detachedBuffer);
__lilaAssertThrows(OffsetError, function () {
  detachedView.getInt32({
    valueOf: function () {
      throw new OffsetError();
    }
  });
});
__lilaAssertThrows(RangeError, function () {
  detachedView.getInt32(9007199254740992);
});
__lilaAssertThrows(TypeError, function () {
  detachedView.getInt32(9007199254740991);
});

let shrinkingBuffer = new ArrayBuffer(8, { maxByteLength: 12 });
let fixedAfterShrink = new DataView(shrinkingBuffer, 0, 8);
__lilaAssertThrows(TypeError, function () {
  fixedAfterShrink.getUint32({
    valueOf: function () {
      shrinkingBuffer.resize(4);
      return 0;
    }
  });
});

let growingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let trackingAfterGrow = new DataView(growingBuffer);
if (trackingAfterGrow.getUint32({
  valueOf: function () {
    growingBuffer.resize(8);
    return 4;
  }
}, true) !== 0) throw "RAB growth not observed after offset coercion";

let trackingShrinkBuffer = new ArrayBuffer(8, { maxByteLength: 8 });
let trackingAfterShrink = new DataView(trackingShrinkBuffer, 1);
__lilaAssertThrows(RangeError, function () {
  trackingAfterShrink.getUint32({
    valueOf: function () {
      trackingShrinkBuffer.resize(4);
      return 0;
    }
  });
});

let growingSharedBuffer = new SharedArrayBuffer(4, { maxByteLength: 8 });
let sharedTrackingAfterGrow = new DataView(growingSharedBuffer);
if (sharedTrackingAfterGrow.getUint32({
  valueOf: function () {
    growingSharedBuffer.grow(8);
    return 4;
  }
}, true) !== 0) throw "GSAB growth not observed after offset coercion";

let fixedSharedBuffer = new SharedArrayBuffer(4, { maxByteLength: 8 });
let sharedFixedAfterGrow = new DataView(fixedSharedBuffer, 0, 4);
__lilaAssertThrows(RangeError, function () {
  sharedFixedAfterGrow.getUint32({
    valueOf: function () {
      fixedSharedBuffer.grow(8);
      return 4;
    }
  });
});

let endianBuffer = new SharedArrayBuffer(4);
let endianView = new DataView(endianBuffer);
endianView.setUint8(0, 0x80);
endianView.setUint8(1, 0x00);
endianView.setUint8(2, 0x00);
endianView.setUint8(3, 0x01);
if (endianView.getInt32(0, false) !== -2147483647) throw "signed big endian";
if (endianView.getInt32(0, true) !== 16777344) throw "signed little endian";
if (endianView.getUint32(0, false) !== 2147483649) throw "unsigned big endian";
if (endianView.getUint32(0, true) !== 16777344) throw "unsigned little endian";
if (endianView.getUint32(0, Symbol("truthy")) !== 16777344) throw "symbol endian";

32;
