function OffsetError() {}

let receiverOffsetCoercions = 0;
let receiverOffset = {
  valueOf: function () {
    receiverOffsetCoercions += 1;
    return 0;
  }
};
__porfAssertThrows(TypeError, function () {
  DataView.prototype.getFloat32.call({}, receiverOffset, true);
});
if (receiverOffsetCoercions !== 0) throw "receiver validated after offset";

let detachedBuffer = new ArrayBuffer(8);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);
__porfAssertThrows(OffsetError, function () {
  detachedView.getFloat64({
    valueOf: function () {
      throw new OffsetError();
    }
  });
});
__porfAssertThrows(RangeError, function () {
  detachedView.getFloat32(9007199254740992);
});
__porfAssertThrows(TypeError, function () {
  detachedView.getFloat32(9007199254740991);
});

let shrinkingBuffer = new ArrayBuffer(16, { maxByteLength: 24 });
let fixedAfterShrink = new DataView(shrinkingBuffer, 0, 16);
__porfAssertThrows(TypeError, function () {
  fixedAfterShrink.getFloat64({
    valueOf: function () {
      shrinkingBuffer.resize(8);
      return 0;
    }
  });
});

let growingBuffer = new ArrayBuffer(4, { maxByteLength: 8 });
let trackingAfterGrow = new DataView(growingBuffer);
if (trackingAfterGrow.getFloat32({
  valueOf: function () {
    growingBuffer.resize(8);
    return 4;
  }
}, true) !== 0) throw "RAB growth not observed after offset coercion";

let trackingShrinkBuffer = new ArrayBuffer(16, { maxByteLength: 16 });
let trackingAfterShrink = new DataView(trackingShrinkBuffer, 1);
__porfAssertThrows(RangeError, function () {
  trackingAfterShrink.getFloat64({
    valueOf: function () {
      trackingShrinkBuffer.resize(8);
      return 0;
    }
  });
});

let growingSharedBuffer = new SharedArrayBuffer(8, { maxByteLength: 16 });
let sharedTrackingAfterGrow = new DataView(growingSharedBuffer);
if (sharedTrackingAfterGrow.getFloat64({
  valueOf: function () {
    growingSharedBuffer.grow(16);
    return 8;
  }
}, true) !== 0) throw "GSAB growth not observed after offset coercion";

let fixedSharedBuffer = new SharedArrayBuffer(8, { maxByteLength: 16 });
let sharedFixedAfterGrow = new DataView(fixedSharedBuffer, 0, 8);
__porfAssertThrows(RangeError, function () {
  sharedFixedAfterGrow.getFloat64({
    valueOf: function () {
      fixedSharedBuffer.grow(16);
      return 8;
    }
  });
});

let endianBuffer = new SharedArrayBuffer(8);
let endianView = new DataView(endianBuffer);
endianView.setFloat32(0, 1.5, false);
if (endianView.getFloat32(0, false) !== 1.5) throw "Float32 big endian";
if (endianView.getFloat32(0, true) === 1.5) throw "Float32 little endian distinction";
endianView.setFloat32(0, -2.25, true);
if (endianView.getFloat32(0, Symbol("truthy")) !== -2.25) throw "Float32 truthy endian";

endianView.setFloat64(0, 1.25, false);
if (endianView.getFloat64(0, false) !== 1.25) throw "Float64 big endian";
if (endianView.getFloat64(0, true) === 1.25) throw "Float64 little endian distinction";
endianView.setFloat64(0, -0, true);
if (1 / endianView.getFloat64(0, true) !== -Infinity) throw "Float64 negative zero";

3264;
