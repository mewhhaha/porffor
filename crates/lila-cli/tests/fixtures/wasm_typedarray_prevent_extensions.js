function throwsTypeError(callback) {
  try {
    callback();
    return false;
  } catch (error) {
    return error instanceof TypeError;
  }
}

let resizableBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
let fixedResizableView = new Uint8Array(resizableBuffer, 0, 4);
let emptyFixedResizableView = new Uint8Array(resizableBuffer, 0, 0);
let lengthTrackingView = new Uint8Array(resizableBuffer);

for (let view of [
  fixedResizableView,
  emptyFixedResizableView,
  lengthTrackingView,
]) {
  if (Reflect.preventExtensions(view) !== false) throw "resizable Reflect result";
  if (!Object.isExtensible(view)) throw "resizable view became non-extensible";
  if (Object.isSealed(view)) throw "resizable view reported sealed";
  if (Object.isFrozen(view)) throw "resizable view reported frozen";
  if (!throwsTypeError(function () {
    Object.preventExtensions(view);
  })) throw "resizable Object.preventExtensions";
  if (!throwsTypeError(function () {
    Object.freeze(view);
  })) throw "resizable Object.freeze";
}

let fixedBufferView = new Uint8Array(new ArrayBuffer(0));
if (Reflect.preventExtensions(fixedBufferView) !== true) throw "fixed buffer Reflect result";
if (Object.isExtensible(fixedBufferView)) throw "fixed buffer view stayed extensible";
if (Object.freeze(fixedBufferView) !== fixedBufferView) throw "fixed buffer freeze identity";
if (!Object.isSealed(fixedBufferView)) throw "fixed buffer view not sealed";
if (!Object.isFrozen(fixedBufferView)) throw "fixed buffer view not frozen";

let growableSharedBuffer = new SharedArrayBuffer(8, { maxByteLength: 16 });
let fixedSharedView = new Uint8Array(growableSharedBuffer, 0, 0);
if (Object.freeze(fixedSharedView) !== fixedSharedView) throw "fixed shared freeze";
if (Object.isExtensible(fixedSharedView)) throw "fixed shared view stayed extensible";
if (!Object.isSealed(fixedSharedView)) throw "fixed shared view not sealed";
if (!Object.isFrozen(fixedSharedView)) throw "fixed shared view not frozen";

let trackingSharedView = new Uint8Array(growableSharedBuffer);
if (Reflect.preventExtensions(trackingSharedView) !== false) throw "tracking shared Reflect result";
if (!Object.isExtensible(trackingSharedView)) throw "tracking shared view became non-extensible";
if (Object.isSealed(trackingSharedView)) throw "tracking shared view reported sealed";
if (Object.isFrozen(trackingSharedView)) throw "tracking shared view reported frozen";

true;
