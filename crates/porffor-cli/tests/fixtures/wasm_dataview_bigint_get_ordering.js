function OffsetError() {}

let receiverOffsetCoercions = 0;
let receiverOffset = {
  valueOf: function () {
    receiverOffsetCoercions += 1;
    return 0;
  }
};
__porfAssertThrows(TypeError, function () {
  DataView.prototype.getBigUint64.call({}, receiverOffset, true);
});
if (receiverOffsetCoercions !== 0) throw "receiver validated after offset";

let detachedBuffer = new ArrayBuffer(8);
let detachedView = new DataView(detachedBuffer);
__porfDetachArrayBuffer(detachedBuffer);
__porfAssertThrows(OffsetError, function () {
  detachedView.getBigInt64({
    valueOf: function () {
      throw new OffsetError();
    }
  });
});
__porfAssertThrows(RangeError, function () {
  detachedView.getBigUint64(9007199254740992);
});
__porfAssertThrows(TypeError, function () {
  detachedView.getBigUint64(9007199254740991);
});
__porfAssertThrows(RangeError, function () {
  new DataView(new ArrayBuffer(8)).getBigInt64(1e100);
});

let shrinkingBuffer = new ArrayBuffer(16, { maxByteLength: 24 });
let fixedAfterShrink = new DataView(shrinkingBuffer, 0, 16);
__porfAssertThrows(TypeError, function () {
  fixedAfterShrink.getBigInt64({
    valueOf: function () {
      shrinkingBuffer.resize(8);
      return 0;
    }
  });
});

let growingBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
let trackingAfterGrow = new DataView(growingBuffer);
if (trackingAfterGrow.getBigUint64({
  valueOf: function () {
    growingBuffer.resize(16);
    return 8;
  }
}, true) !== 0n) throw "RAB growth not observed";

let trackingShrinkBuffer = new ArrayBuffer(16, { maxByteLength: 16 });
let trackingAfterShrink = new DataView(trackingShrinkBuffer, 1);
__porfAssertThrows(RangeError, function () {
  trackingAfterShrink.getBigInt64({
    valueOf: function () {
      trackingShrinkBuffer.resize(8);
      return 0;
    }
  });
});

let growingSharedBuffer = new SharedArrayBuffer(8, { maxByteLength: 16 });
let sharedTrackingAfterGrow = new DataView(growingSharedBuffer);
if (sharedTrackingAfterGrow.getBigInt64({
  valueOf: function () {
    growingSharedBuffer.grow(16);
    return 8;
  }
}, true) !== 0n) throw "GSAB growth not observed";

let fixedSharedBuffer = new SharedArrayBuffer(8, { maxByteLength: 16 });
let sharedFixedAfterGrow = new DataView(fixedSharedBuffer, 0, 8);
__porfAssertThrows(RangeError, function () {
  sharedFixedAfterGrow.getBigUint64({
    valueOf: function () {
      fixedSharedBuffer.grow(16);
      return 8;
    }
  });
});

let endianBuffer = new ArrayBuffer(16);
let endianBytes = new Uint8Array(endianBuffer);
endianBytes[0] = 0x80;
endianBytes[1] = 0x01;
endianBytes[2] = 0x02;
endianBytes[3] = 0x03;
endianBytes[4] = 0x04;
endianBytes[5] = 0x05;
endianBytes[6] = 0x06;
endianBytes[7] = 0x07;
let endianView = new DataView(endianBuffer);
if (endianView.getBigInt64(0, false) !== -0x7ffefdfcfbfaf9f9n) {
  throw "signed high bit";
}
if (endianView.getBigUint64(0, false) !== 0x8001020304050607n) {
  throw "unsigned high bit";
}
if (endianView.getBigInt64(0, true) !== 0x0706050403020180n) {
  throw "signed little endian";
}
if (endianView.getBigUint64(0, true) !== 0x0706050403020180n) {
  throw "unsigned little endian";
}
if (endianView.getBigUint64(0, Symbol("truthy")) !== 0x0706050403020180n) {
  throw "truthy endian";
}
for (let i = 8; i < 16; i += 1) endianBytes[i] = 0xff;
if (endianView.getBigInt64(8, true) !== -1n) throw "signed all bits";
if (endianView.getBigUint64(8, true) !== 0xffffffffffffffffn) {
  throw "unsigned all bits";
}

4288;
