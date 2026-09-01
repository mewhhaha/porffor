function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

var other = __lilaCreateRealm().global;
var otherTypeErrorPrototype = other.TypeError.prototype;
var completed = 0;

function complete() {
  completed = completed + 1;
  if (completed === 4) print("array-from-async-internal-callback-realm:ok");
}

other.Array.fromAsync.call(Array, { 0: 1, length: 1 }).then(
  function (arrayLikeFulfilledValue) {
    assertSame(
      arrayLikeFulfilledValue[0],
      1,
      "array-like fulfilled callback state",
    );
    complete();
  },
  function () {
    throw "array-like fulfillment rejected";
  },
);

var invalidAsyncIterable = {};
invalidAsyncIterable[Symbol.asyncIterator] = function () {
  return {
    next: function () {
      return Promise.resolve(1);
    },
  };
};

other.Array.fromAsync.call(Array, invalidAsyncIterable).then(
  function () {
    throw "iterableFulfilledTypeError fulfilled";
  },
  function (iterableFulfilledTypeError) {
    assertSame(
      Object.getPrototypeOf(iterableFulfilledTypeError),
      otherTypeErrorPrototype,
      "iterable fulfillment callback TypeError Realm",
    );
    complete();
  },
);

var arrayLikeRejectedReason = {};
other.Array.fromAsync.call(Array, {
  0: Promise.reject(arrayLikeRejectedReason),
  length: 1,
}).then(
  function () {
    throw "array-like rejection fulfilled";
  },
  function (reason) {
    assertSame(
      reason,
      arrayLikeRejectedReason,
      "array-like rejected callback state",
    );
    complete();
  },
);

var iterableRejectedReason = {};
other.Array.fromAsync.call(Array, [Promise.reject(iterableRejectedReason)])
  .then(
    function () {
      throw "iterable rejection fulfilled";
    },
    function (reason) {
      assertSame(
        reason,
        iterableRejectedReason,
        "iterable rejected callback state",
      );
      complete();
    },
  );

0;
