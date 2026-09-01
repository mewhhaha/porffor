function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

function assertErrorPrototype(callback, expectedPrototype, label) {
  try {
    callback();
  } catch (error) {
    assertSame(Object.getPrototypeOf(error), expectedPrototype, label + " prototype");
    return;
  }
  throw label + " did not throw";
}

var other = __lilaCreateRealm().global;
var otherSlice = other.Uint8Array.prototype.slice;

var detachedSource = new Uint8Array(1);
var detachedTarget = new Uint8Array(1);
detachedSource.constructor = {
  [Symbol.species]: function() {
    return detachedTarget;
  }
};
__lilaDetachArrayBuffer(detachedTarget.buffer);
assertErrorPrototype(function() {
  otherSlice.call(detachedSource);
}, other.TypeError.prototype, "borrowed slice detached species target");

var outOfBoundsBuffer = new ArrayBuffer(4, { maxByteLength: 4 });
var outOfBoundsTarget = new Uint8Array(outOfBoundsBuffer, 2, 2);
var outOfBoundsSource = new Uint8Array(1);
outOfBoundsSource.constructor = {
  [Symbol.species]: function() {
    return outOfBoundsTarget;
  }
};
outOfBoundsBuffer.resize(1);
assertErrorPrototype(function() {
  otherSlice.call(outOfBoundsSource);
}, other.TypeError.prototype, "borrowed slice out-of-bounds species target");

var nonTypedArraySource = new Uint8Array(1);
nonTypedArraySource.constructor = {
  [Symbol.species]: function() {
    return {};
  }
};
assertErrorPrototype(function() {
  otherSlice.call(nonTypedArraySource);
}, other.TypeError.prototype, "borrowed slice non-TypedArray species target");

var undersizedSource = new Uint8Array(2);
undersizedSource.constructor = {
  [Symbol.species]: function() {
    return new Uint8Array(1);
  }
};
assertErrorPrototype(function() {
  otherSlice.call(undersizedSource);
}, other.TypeError.prototype, "borrowed slice undersized species target");

true;
