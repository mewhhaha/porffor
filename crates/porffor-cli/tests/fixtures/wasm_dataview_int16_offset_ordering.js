function OffsetError() {}

let receiverOffsetCoercions = 0;
let receiverOffset = {
  valueOf: function () {
    receiverOffsetCoercions += 1;
    return 0;
  }
};
__porfAssertThrows(TypeError, function () {
  DataView.prototype.getUint16.call({}, receiverOffset, true);
});
if (receiverOffsetCoercions !== 0) throw "receiver validated after offset";

let detachedBuffer = new ArrayBuffer(2);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);
__porfAssertThrows(OffsetError, function () {
  detachedView.getInt16({
    valueOf: function () {
      throw new OffsetError();
    }
  });
});
__porfAssertThrows(RangeError, function () {
  detachedView.getInt16(9007199254740992);
});
__porfAssertThrows(TypeError, function () {
  detachedView.getInt16(9007199254740991);
});

let shrinkingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let fixedAfterShrink = new DataView(shrinkingBuffer, 0, 4);
__porfAssertThrows(TypeError, function () {
  fixedAfterShrink.getUint16({
    valueOf: function () {
      shrinkingBuffer.resize(2);
      return 0;
    }
  });
});

let growingBuffer = new ArrayBuffer(2, { maxByteLength: 4 });
let trackingAfterGrow = new DataView(growingBuffer);
if (trackingAfterGrow.getUint16({
  valueOf: function () {
    growingBuffer.resize(4);
    return 2;
  }
}, true) !== 0) throw "RAB growth not observed after offset coercion";

let trackingShrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let trackingAfterShrink = new DataView(trackingShrinkBuffer, 1);
__porfAssertThrows(RangeError, function () {
  trackingAfterShrink.getUint16({
    valueOf: function () {
      trackingShrinkBuffer.resize(2);
      return 0;
    }
  });
});

let growingSharedBuffer = new SharedArrayBuffer(2, { maxByteLength: 4 });
let sharedTrackingAfterGrow = new DataView(growingSharedBuffer);
if (sharedTrackingAfterGrow.getUint16({
  valueOf: function () {
    growingSharedBuffer.grow(4);
    return 2;
  }
}, true) !== 0) throw "GSAB growth not observed after offset coercion";

let fixedSharedBuffer = new SharedArrayBuffer(2, { maxByteLength: 4 });
let sharedFixedAfterGrow = new DataView(fixedSharedBuffer, 0, 2);
__porfAssertThrows(RangeError, function () {
  sharedFixedAfterGrow.getUint16({
    valueOf: function () {
      fixedSharedBuffer.grow(4);
      return 2;
    }
  });
});

let endianBuffer = new SharedArrayBuffer(2);
let endianView = new DataView(endianBuffer);
endianView.setUint8(0, 0x80);
endianView.setUint8(1, 0x01);
if (endianView.getInt16(0, false) !== -32767) throw "signed big endian";
if (endianView.getInt16(0, true) !== 384) throw "signed little endian";
if (endianView.getUint16(0, false) !== 32769) throw "unsigned big endian";
if (endianView.getUint16(0, true) !== 384) throw "unsigned little endian";
if (endianView.getUint16(0, Symbol("truthy")) !== 384) throw "symbol endian";

16;
