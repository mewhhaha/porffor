function Test262Error() {
}

let assert = {};
assert.throws = function (expectedErrorConstructor, func, message) {
  return __lilaAssertThrows(expectedErrorConstructor, func, message);
};

let from = Uint8Array.from;
assert.throws(TypeError, function () {
  from([]);
});

let arrayLike = {};
Object.defineProperty(arrayLike, "length", {
  get: function () {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function () {
  Uint8Array.from(arrayLike);
});

262;
