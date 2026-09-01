function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

var other = __lilaCreateRealm().global;

function assertOtherTypeError(callback, label) {
  try {
    callback();
  } catch (error) {
    assertSame(
      Object.getPrototypeOf(error),
      other.TypeError.prototype,
      label + " prototype"
    );
    return;
  }
  throw label + " did not throw";
}

assertOtherTypeError(function() {
  other.DataView(new ArrayBuffer(1));
}, "borrowed DataView requires new");

var invalidBufferOffsetCoercions = 0;
assertOtherTypeError(function() {
  new other.DataView({}, {
    valueOf: function() {
      invalidBufferOffsetCoercions += 1;
      return 0;
    }
  });
}, "borrowed DataView invalid buffer");
assertSame(invalidBufferOffsetCoercions, 0, "invalid buffer precedes offset coercion");

var detachedConstructorBuffer = new ArrayBuffer(1);
__lilaDetachArrayBuffer(detachedConstructorBuffer);
var detachedConstructorOffsetCoercions = 0;
assertOtherTypeError(function() {
  new other.DataView(detachedConstructorBuffer, {
    valueOf: function() {
      detachedConstructorOffsetCoercions += 1;
      return 0;
    }
  });
}, "borrowed DataView detached constructor buffer");
assertSame(
  detachedConstructorOffsetCoercions,
  1,
  "offset coercion precedes detached constructor check"
);

var postPrototypeBuffer = new ArrayBuffer(1);
var detachingNewTarget = function() {}.bind(null);
Object.defineProperty(detachingNewTarget, "prototype", {
  get: function() {
    __lilaDetachArrayBuffer(postPrototypeBuffer);
    return {};
  }
});
assertOtherTypeError(function() {
  Reflect.construct(other.DataView, [postPrototypeBuffer], detachingNewTarget);
}, "borrowed DataView post-prototype detachment");

var otherPrototype = other.DataView.prototype;
var borrowedGetter = otherPrototype.getUint8;
var borrowedSetter = otherPrototype.setUint8;
var borrowedBufferGetter = Object.getOwnPropertyDescriptor(otherPrototype, "buffer").get;

var invalidGetterIndexCoercions = 0;
assertOtherTypeError(function() {
  borrowedGetter.call({}, {
    valueOf: function() {
      invalidGetterIndexCoercions += 1;
      return 0;
    }
  });
}, "borrowed DataView getter invalid receiver");
assertSame(invalidGetterIndexCoercions, 0, "getter receiver check precedes index coercion");

var invalidSetterIndexCoercions = 0;
var invalidSetterValueCoercions = 0;
assertOtherTypeError(function() {
  borrowedSetter.call({}, {
    valueOf: function() {
      invalidSetterIndexCoercions += 1;
      return 0;
    }
  }, {
    valueOf: function() {
      invalidSetterValueCoercions += 1;
      return 1;
    }
  });
}, "borrowed DataView setter invalid receiver");
assertSame(invalidSetterValueCoercions, 0, "setter receiver check precedes value coercion");
assertSame(invalidSetterIndexCoercions, 0, "setter receiver check precedes index coercion");

assertOtherTypeError(function() {
  borrowedBufferGetter.call({});
}, "borrowed DataView private-slot getter invalid receiver");

var detachedMethodBuffer = new ArrayBuffer(1);
var detachedMethodView = new DataView(detachedMethodBuffer);
__lilaDetachArrayBuffer(detachedMethodBuffer);
var detachedMethodIndexCoercions = 0;
assertOtherTypeError(function() {
  borrowedGetter.call(detachedMethodView, {
    valueOf: function() {
      detachedMethodIndexCoercions += 1;
      return 0;
    }
  });
}, "borrowed DataView getter detached buffer");
assertSame(detachedMethodIndexCoercions, 1, "index coercion precedes detached method check");

var resizedBuffer = new ArrayBuffer(2, { maxByteLength: 2 });
var resizedView = new DataView(resizedBuffer, 1, 1);
resizedBuffer.resize(0);
assertOtherTypeError(function() {
  borrowedGetter.call(resizedView, 0);
}, "borrowed DataView getter out-of-bounds view");

true;
