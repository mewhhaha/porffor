let resizable = new ArrayBuffer(2, { maxByteLength: 6 });
let bytes = new Uint8Array(resizable);
bytes[0] = 44;
bytes[1] = 55;
let fixed = resizable.transferToFixedLength(4);
if (fixed.byteLength !== 4) throw "fixed length";
if (fixed.maxByteLength !== 4) throw "fixed max";
if (fixed.resizable !== false) throw "fixed flag";
if (resizable.detached !== true) throw "source detached";
let fixedView = new DataView(fixed);
if (fixedView.getUint8(0) !== 44) throw "fixed byte 0";
if (fixedView.getUint8(1) !== 55) throw "fixed byte 1";
if (fixedView.getUint8(2) !== 0) throw "fixed growth zero";

let omitted = new ArrayBuffer(3, { maxByteLength: 8 });
let omittedMoved = omitted.transferToFixedLength();
if (omittedMoved.byteLength !== 3) throw "omitted length";
if (omittedMoved.maxByteLength !== 3) throw "omitted max";
if (omittedMoved.resizable !== false) throw "omitted fixed";
if (omitted.detached !== true) throw "omitted source detached";

let shrink = new ArrayBuffer(3);
let shrinkBytes = new Uint8Array(shrink);
shrinkBytes[0] = 1;
shrinkBytes[1] = 2;
shrinkBytes[2] = 3;
let shrunk = shrink.transferToFixedLength(2);
let shrunkView = new DataView(shrunk);
if (shrunk.byteLength !== 2) throw "shrunk length";
if (shrunk.maxByteLength !== 2) throw "shrunk max";
if (shrunkView.getUint8(0) !== 1) throw "shrunk byte 0";
if (shrunkView.getUint8(1) !== 2) throw "shrunk byte 1";

let invalidReceiverThrew = false;
try {
  ArrayBuffer.prototype.transferToFixedLength.call({});
} catch (error) {
  invalidReceiverThrew = true;
}
if (!invalidReceiverThrew) throw "invalid receiver";

let detached = new ArrayBuffer(1);
__porfDetachArrayBuffer(detached);
let detachedThrew = false;
try {
  detached.transferToFixedLength();
} catch (error) {
  detachedThrew = true;
}
if (!detachedThrew) throw "detached receiver";

let negativeThrew = false;
try {
  new ArrayBuffer(1).transferToFixedLength(-1);
} catch (error) {
  negativeThrew = true;
}
if (!negativeThrew) throw "negative length";

let detachedDuringCoercion = new ArrayBuffer(1);
let coercions = 0;
function fixedDetachDuringCoercionValueOf() {
  coercions += 1;
  __porfDetachArrayBuffer(detachedDuringCoercion);
  return 1;
}

let detachedDuringCoercionThrew = false;
try {
  detachedDuringCoercion.transferToFixedLength({
    valueOf: fixedDetachDuringCoercionValueOf
  });
} catch (error) {
  detachedDuringCoercionThrew = true;
}
if (!detachedDuringCoercionThrew) throw "detached during coercion";
if (coercions !== 1) throw "fixed coercion before detach";

123;
