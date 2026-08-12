var failures = 0;

function throwsTypeError(fn) {
  try {
    fn();
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}

var target = {};
var seenTarget = null;
var seenHandler = null;
var handler = {
  isExtensible: function(t) {
    seenTarget = t;
    seenHandler = this;
    return Object.isExtensible(t);
  },
};
var proxy = new Proxy(target, handler);

if (Object.isExtensible(proxy) !== true) failures |= 1;
if (seenTarget !== target) failures |= 2;
if (seenHandler !== handler) failures |= 4;
Object.preventExtensions(target);
if (Object.isExtensible(proxy) !== false) failures |= 8;

var mismatch = new Proxy({}, {
  isExtensible: function() {
    return false;
  },
});
if (!throwsTypeError(function() { Object.isExtensible(mismatch); })) failures |= 16;

var array = [];
var arrayExtensible = true;
var arrayTarget = new Proxy(array, {
  isExtensible: function() {
    return arrayExtensible;
  },
});
var arrayProxy = new Proxy(arrayTarget, {
  isExtensible: undefined,
});
if (Object.isExtensible(arrayProxy) !== true) failures |= 32;
Object.preventExtensions(array);
arrayExtensible = false;
if (Object.isExtensible(arrayProxy) !== false) failures |= 64;

var revoked = Proxy.revocable({}, {});
revoked.revoke();
if (!throwsTypeError(function() { Object.isExtensible(revoked.proxy); })) failures |= 128;

failures === 0;
