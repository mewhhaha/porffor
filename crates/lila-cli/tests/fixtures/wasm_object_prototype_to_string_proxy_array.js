var target = [];
target.join = undefined;

var direct = new Proxy(target, {});
var nested = new Proxy(direct, {});

var directObjectTag = Object.prototype.toString.call(direct);
var nestedObjectTag = Object.prototype.toString.call(nested);
var directArrayFallback = Array.prototype.toString.call(direct);
var nestedArrayFallback = Array.prototype.toString.call(nested);

var revokedObjectThrows = false;
var revokedArrayFallbackThrows = false;
var otherObjectError = null;
var otherArrayIsArrayError = null;
var otherArrayToStringError = null;
var mainTypedArrayToString = new Uint8Array(0).toString;
var revocable = Proxy.revocable([], {});
revocable.revoke();

try {
  Object.prototype.toString.call(revocable.proxy);
} catch (error) {
  revokedObjectThrows = error instanceof TypeError;
}

try {
  Array.prototype.toString.call(revocable.proxy);
} catch (error) {
  revokedArrayFallbackThrows = error instanceof TypeError;
}

var other = __lilaCreateRealm().global;
try {
  other.Object.prototype.toString.call(revocable.proxy);
} catch (error) {
  otherObjectError = error;
}

try {
  other.Array.isArray(revocable.proxy);
} catch (error) {
  otherArrayIsArrayError = error;
}

try {
  other.Array.prototype.toString.call(revocable.proxy);
} catch (error) {
  otherArrayToStringError = error;
}

var otherArrayToString = other.Array.prototype.toString;
var otherTypedArrayToString = other.Uint8Array.prototype.toString;
var otherArrayToStringDescriptor = Object.getOwnPropertyDescriptor(
  other.Array.prototype,
  "toString",
);

directObjectTag === "[object Array]"
  && nestedObjectTag === "[object Array]"
  && directArrayFallback === "[object Array]"
  && nestedArrayFallback === "[object Array]"
  && revokedObjectThrows
  && revokedArrayFallbackThrows
  && mainTypedArrayToString === Array.prototype.toString
  && otherArrayToString === otherTypedArrayToString
  && otherArrayToString.name === "toString"
  && otherArrayToString.length === 0
  && otherArrayToStringDescriptor.value === otherArrayToString
  && otherArrayToStringDescriptor.writable
  && !otherArrayToStringDescriptor.enumerable
  && otherArrayToStringDescriptor.configurable
  && otherArrayToString.call(direct) === "[object Array]"
  && Object.getPrototypeOf(otherObjectError) === other.TypeError.prototype
  && Object.getPrototypeOf(otherArrayIsArrayError) === other.TypeError.prototype
  && Object.getPrototypeOf(otherArrayToStringError) === other.TypeError.prototype;
