function assertEntryTypeError(callback, label) {
  try {
    callback();
  } catch (error) {
    if (Object.getPrototypeOf(error) !== TypeError.prototype) throw label + " prototype";
    return;
  }
  throw label + " did not throw";
}

assertEntryTypeError(function() {
  Atomics.pause(1.5);
}, "Atomics.pause invalid iteration");

assertEntryTypeError(function() {
  Atomics.notify({}, 0);
}, "Atomics.notify invalid receiver");

assertEntryTypeError(function() {
  Atomics.notify(new Uint8Array(new SharedArrayBuffer(1)), 0);
}, "Atomics.notify invalid element kind");

assertEntryTypeError(function() {
  Atomics.waitAsync(new Int32Array(new ArrayBuffer(4)), 0, 0);
}, "Atomics.waitAsync non-shared buffer");

assertEntryTypeError(function() {
  Atomics.wait(new Int32Array(new ArrayBuffer(4)), 0, 0, 0);
}, "Atomics.wait non-shared buffer");

assertEntryTypeError(function() {
  Atomics.add(new Float32Array(new SharedArrayBuffer(4)), 0, 1);
}, "Atomics integer operation invalid element kind");

var detachedBuffer = new ArrayBuffer(4);
var detachedArray = new Int32Array(detachedBuffer);
__lilaDetachArrayBuffer(detachedBuffer);
assertEntryTypeError(function() {
  Atomics.add(detachedArray, 0, 1);
}, "Atomics integer operation detached buffer");

true;
