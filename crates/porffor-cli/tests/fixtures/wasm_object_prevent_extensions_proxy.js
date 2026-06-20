var failures = 0;

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

var abruptThrew = throwsTypeError(function() {
  Object.preventExtensions(new Proxy({}, {
    preventExtensions: function() {
      throw new TypeError("boom");
    }
  }));
});
if (!abruptThrew) failures |= 1;

var falseProxy = new Proxy({}, {
  preventExtensions: function() {
    return 0;
  }
});
if (Reflect.preventExtensions(falseProxy) !== false) failures |= 2;
if (!throwsTypeError(function() { Object.preventExtensions(falseProxy); })) failures |= 4;

var extensibleTarget = {};
var invariantProxy = new Proxy(extensibleTarget, {
  preventExtensions: function() {
    return true;
  }
});
if (!throwsTypeError(function() { Reflect.preventExtensions(invariantProxy); })) failures |= 8;

var fixedTarget = {};
Object.preventExtensions(fixedTarget);
var trueCalls = 0;
var trueProxy = new Proxy(fixedTarget, {
  preventExtensions: function(target) {
    trueCalls = trueCalls + 1;
    return !Object.isExtensible(target);
  }
});
if (Reflect.preventExtensions(trueProxy) !== true) failures |= 16;
if (trueCalls !== 1) failures |= 32;

var fallbackTarget = {};
var fallbackProxy = new Proxy(fallbackTarget, {});
if (Reflect.preventExtensions(fallbackProxy) !== true) failures |= 64;
fallbackTarget.x = 1;
if (fallbackTarget.x !== undefined) failures |= 128;

var nestedTarget = {};
var nestedInner = new Proxy(nestedTarget, {});
var nestedOuterNull = new Proxy(nestedInner, {
  preventExtensions: null,
});
Object.preventExtensions(nestedOuterNull);
nestedTarget.y = 1;
if (nestedTarget.y !== undefined) failures |= 256;

var nestedFalseTarget = new Proxy({}, {
  preventExtensions: function() {
    return false;
  },
});
var nestedOuterMissing = new Proxy(nestedFalseTarget, {});
if (!throwsTypeError(function() { Object.preventExtensions(nestedOuterMissing); })) failures |= 512;

var nonCallableProxy = new Proxy({}, {
  preventExtensions: 1,
});
if (!throwsTypeError(function() { Reflect.preventExtensions(nonCallableProxy); })) failures |= 1024;

var revoked = Proxy.revocable({}, {});
revoked.revoke();
if (!throwsTypeError(function() { Object.preventExtensions(revoked.proxy); })) failures |= 2048;

failures === 0;
