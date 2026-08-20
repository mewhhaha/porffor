let literal = 1n;
let negativeLiteral = -1n;
let fromNumber = BigInt(42);
let fromNegativeNumber = BigInt(-7);

__lilaAssertThrows(RangeError, function () {
  BigInt(1.5);
});

__lilaAssertThrows(RangeError, function () {
  BigInt(NaN);
});

__lilaAssertThrows(RangeError, function () {
  BigInt(Infinity);
});

__lilaAssertThrows(TypeError, function () {
  BigInt(undefined);
});

__lilaAssertThrows(TypeError, function () {
  new BigInt(1);
});

(literal === 1n) +
  (negativeLiteral === -1n) +
  (fromNumber === 42n) +
  (fromNegativeNumber === -7n) +
  (literal !== 2n);
