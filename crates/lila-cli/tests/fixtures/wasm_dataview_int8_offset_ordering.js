function OffsetError() {}

let receiverOffsetCoercions = 0;
let receiverOffset = {
  valueOf: function () {
    receiverOffsetCoercions += 1;
    return 0;
  }
};
__lilaAssertThrows(TypeError, function () {
  DataView.prototype.getUint8.call({}, receiverOffset);
});
if (receiverOffsetCoercions !== 0) throw "receiver validated after offset";

let detachedBuffer = new ArrayBuffer(1);
let detachedView = new DataView(detachedBuffer);
__lilaDetachArrayBuffer(detachedBuffer);
__lilaAssertThrows(OffsetError, function () {
  detachedView.getUint8({
    valueOf: function () {
      throw new OffsetError();
    }
  });
});
__lilaAssertThrows(RangeError, function () {
  detachedView.getUint8(9007199254740992);
});
__lilaAssertThrows(TypeError, function () {
  detachedView.getUint8(9007199254740991);
});
__lilaAssertThrows(TypeError, function () {
  detachedView.getUint8(2);
});

let shrinkingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let fixedAfterShrink = new DataView(shrinkingBuffer, 0, 4);
__lilaAssertThrows(TypeError, function () {
  fixedAfterShrink.getInt8({
    valueOf: function () {
      shrinkingBuffer.resize(2);
      return 0;
    }
  });
});

let growingBuffer = new ArrayBuffer(2, { maxByteLength: 4 });
let trackingAfterGrow = new DataView(growingBuffer);
if (trackingAfterGrow.getUint8({
  valueOf: function () {
    growingBuffer.resize(4);
    return 3;
  }
}) !== 0) throw "RAB growth not observed after offset coercion";

let trackingShrinkBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
let trackingAfterShrink = new DataView(trackingShrinkBuffer, 1);
__lilaAssertThrows(RangeError, function () {
  trackingAfterShrink.getUint8({
    valueOf: function () {
      trackingShrinkBuffer.resize(2);
      return 1;
    }
  });
});

let growingSharedBuffer = new SharedArrayBuffer(2, { maxByteLength: 4 });
let sharedTrackingAfterGrow = new DataView(growingSharedBuffer);
if (sharedTrackingAfterGrow.getUint8({
  valueOf: function () {
    growingSharedBuffer.grow(4);
    return 3;
  }
}) !== 0) throw "GSAB growth not observed after offset coercion";

let fixedSharedBuffer = new SharedArrayBuffer(2, { maxByteLength: 4 });
let sharedFixedAfterGrow = new DataView(fixedSharedBuffer, 0, 2);
__lilaAssertThrows(RangeError, function () {
  sharedFixedAfterGrow.getUint8({
    valueOf: function () {
      fixedSharedBuffer.grow(4);
      return 3;
    }
  });
});

let signedBuffer = new SharedArrayBuffer(1);
let signedView = new DataView(signedBuffer);
signedView.setUint8(0, 255);
if (signedView.getInt8(0) !== -1) throw "signed shared byte";

34;
