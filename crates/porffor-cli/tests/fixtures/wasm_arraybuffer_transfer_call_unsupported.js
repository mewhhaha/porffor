let omitted = new ArrayBuffer(4);
let omittedView = new DataView(omitted);
let omittedBytes = new Uint8Array(omitted);
omittedBytes[0] = 17;
omittedBytes[1] = 34;
let omittedMoved = omitted.transfer();
if (omittedMoved.byteLength !== 4) throw "omitted length";
if (omittedMoved.maxByteLength !== 4) throw "omitted max";
if (omittedMoved.resizable !== false) throw "omitted fixed";
if (omitted.detached !== true) throw "omitted source detached";
if (omitted.byteLength !== 0) throw "omitted source byteLength";
let omittedSliceThrew = false;
try {
  omitted.slice(0);
} catch (error) {
  omittedSliceThrew = true;
}
if (!omittedSliceThrew) throw "omitted slice detached";
let omittedViewThrew = false;
try {
  omittedView.byteLength;
} catch (error) {
  omittedViewThrew = true;
}
if (!omittedViewThrew) throw "omitted view detached";
let omittedMovedView = new DataView(omittedMoved);
if (omittedMovedView.getUint8(0) !== 17) throw "omitted byte 0";
if (omittedMovedView.getUint8(1) !== 34) throw "omitted byte 1";

let same = new ArrayBuffer(4);
let sameBytes = new Uint8Array(same);
sameBytes[0] = 17;
sameBytes[1] = 34;
let sameMoved = same.transfer(4);
if (sameMoved.byteLength !== 4) throw "same length";
if (sameMoved.maxByteLength !== 4) throw "same max";
let sameMovedView = new DataView(sameMoved);
if (sameMovedView.getUint8(0) !== 17) throw "same byte 0";
if (sameMovedView.getUint8(1) !== 34) throw "same byte 1";

let smaller = new ArrayBuffer(4);
let smallerBytes = new Uint8Array(smaller);
smallerBytes[0] = 17;
smallerBytes[1] = 34;
let smallerMoved = smaller.transfer(1);
if (smallerMoved.byteLength !== 1) throw "smaller length";
if (smallerMoved.maxByteLength !== 1) throw "smaller max";
let smallerMovedView = new DataView(smallerMoved);
if (smallerMovedView.getUint8(0) !== 17) throw "smaller byte 0";

let larger = new ArrayBuffer(2);
let largerBytes = new Uint8Array(larger);
largerBytes[0] = 17;
largerBytes[1] = 34;
let largerMoved = larger.transfer(4);
if (largerMoved.byteLength !== 4) throw "larger length";
if (largerMoved.maxByteLength !== 4) throw "larger max";
let largerMovedView = new DataView(largerMoved);
if (largerMovedView.getUint8(0) !== 17) throw "larger byte 0";
if (largerMovedView.getUint8(1) !== 34) throw "larger byte 1";
if (largerMovedView.getUint8(2) !== 0) throw "larger growth zero";

let zero = new ArrayBuffer(4);
let zeroMoved = zero.transfer(0);
if (zeroMoved.byteLength !== 0) throw "zero length";
if (zeroMoved.maxByteLength !== 0) throw "zero max";
if (zero.detached !== true) throw "zero source detached";

let invalidReceiverThrew = false;
try {
  ArrayBuffer.prototype.transfer.call({});
} catch (error) {
  invalidReceiverThrew = true;
}
if (!invalidReceiverThrew) throw "invalid receiver";

let detached = new ArrayBuffer(1);
__porfDetachArrayBuffer(detached);
let detachedThrew = false;
try {
  detached.transfer();
} catch (error) {
  detachedThrew = true;
}
if (!detachedThrew) throw "detached receiver";

function badLengthValueOf() {
  throw new TypeError("bad length");
}

let badLengthThrew = false;
try {
  new ArrayBuffer(1).transfer({ valueOf: badLengthValueOf });
} catch (error) {
  badLengthThrew = true;
}
if (!badLengthThrew) throw "bad length";

let infinityThrew = false;
try {
  new ArrayBuffer(1).transfer(Infinity);
} catch (error) {
  infinityThrew = true;
}
if (!infinityThrew) throw "infinity length";

let detachedDuringCoercion = new ArrayBuffer(1);
let coercions = 0;
function detachDuringCoercionValueOf() {
  coercions += 1;
  __porfDetachArrayBuffer(detachedDuringCoercion);
  return 1;
}

let detachedDuringCoercionThrew = false;
try {
  detachedDuringCoercion.transfer({ valueOf: detachDuringCoercionValueOf });
} catch (error) {
  detachedDuringCoercionThrew = true;
}
if (!detachedDuringCoercionThrew) throw "detached during coercion";
if (coercions !== 1) throw "transfer coercion before detach";

let resizable = new ArrayBuffer(2, { maxByteLength: 5 });
let resizableView = new DataView(resizable);
resizableView.setUint8(0, 9);
resizableView.setUint8(1, 8);
let movedResizable = resizable.transfer(4);
if (movedResizable.byteLength !== 4) throw "resizable transfer length";
if (movedResizable.maxByteLength !== 5) throw "resizable transfer max";
if (movedResizable.resizable !== true) throw "resizable transfer flag";
let movedResizableView = new DataView(movedResizable);
if (movedResizableView.getUint8(0) !== 9) throw "resizable transfer byte 0";
if (movedResizableView.getUint8(1) !== 8) throw "resizable transfer byte 1";
if (movedResizableView.getUint8(2) !== 0) throw "resizable transfer growth";
if (resizable.detached !== true) throw "resizable transfer source detached";

let overMaxThrew = false;
try {
  new ArrayBuffer(1, { maxByteLength: 2 }).transfer(3);
} catch (error) {
  overMaxThrew = true;
}
if (!overMaxThrew) throw "resizable over max";

123;
