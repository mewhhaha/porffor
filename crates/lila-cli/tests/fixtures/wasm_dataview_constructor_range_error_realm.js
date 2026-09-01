function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

var other = __lilaCreateRealm().global;

function assertOtherRangeError(callback, label) {
  try {
    callback();
  } catch (error) {
    assertSame(
      Object.getPrototypeOf(error),
      other.RangeError.prototype,
      label + " prototype"
    );
    return;
  }
  throw label + " did not throw";
}

assertOtherRangeError(function() {
  new other.DataView(new ArrayBuffer(1), -1);
}, "borrowed DataView byteOffset ToIndex realm");

assertOtherRangeError(function() {
  new other.DataView(new ArrayBuffer(0), 1);
}, "borrowed DataView byteOffset capacity realm");

assertOtherRangeError(function() {
  new other.DataView(new ArrayBuffer(1), 0, -1);
}, "borrowed DataView byteLength ToIndex realm");

assertOtherRangeError(function() {
  new other.DataView(new ArrayBuffer(1), 0, 2);
}, "borrowed DataView byteLength capacity realm");

var offsetBuffer = new ArrayBuffer(2, { maxByteLength: 2 });
var offsetNewTarget = function() {}.bind(null);
Object.defineProperty(offsetNewTarget, "prototype", {
  get: function() {
    offsetBuffer.resize(0);
    return {};
  }
});
assertOtherRangeError(function() {
  Reflect.construct(other.DataView, [offsetBuffer, 1], offsetNewTarget);
}, "borrowed DataView post-prototype byteOffset realm");

var lengthBuffer = new ArrayBuffer(2, { maxByteLength: 2 });
var lengthNewTarget = function() {}.bind(null);
Object.defineProperty(lengthNewTarget, "prototype", {
  get: function() {
    lengthBuffer.resize(1);
    return {};
  }
});
assertOtherRangeError(function() {
  Reflect.construct(other.DataView, [lengthBuffer, 0, 2], lengthNewTarget);
}, "borrowed DataView post-prototype byteLength realm");

true;
