let literal = 1n;
let negativeLiteral = -1n;
let fromNumber = BigInt(42);
let fromNegativeNumber = BigInt(-7);

__porfAssertThrows(RangeError, function () {
  BigInt(1.5);
});

__porfAssertThrows(RangeError, function () {
  BigInt(NaN);
});

__porfAssertThrows(RangeError, function () {
  BigInt(Infinity);
});

__porfAssertThrows(TypeError, function () {
  BigInt(undefined);
});

__porfAssertThrows(TypeError, function () {
  new BigInt(1);
});

(literal === 1n) +
  (negativeLiteral === -1n) +
  (fromNumber === 42n) +
  (fromNegativeNumber === -7n) +
  (literal !== 2n);
