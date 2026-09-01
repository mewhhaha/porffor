function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

var other = __lilaCreateRealm().global;
var otherPromisePrototype = other.Promise.prototype;
var otherArrayPrototype = other.Array.prototype;
var otherTypeErrorPrototype = other.TypeError.prototype;
var completed = 0;

function complete() {
  completed = completed + 1;
  if (completed === 3) print("array-from-async-promise-realm:ok");
}

var createdMethodPromise = other.Array.fromAsync.call(Array, [1]);
assertSame(
  Object.getPrototypeOf(createdMethodPromise),
  otherPromisePrototype,
  "created method Promise Realm",
);
createdMethodPromise.then(function (result) {
  assertSame(Object.getPrototypeOf(result), Array.prototype, "entry result Array Realm");
  complete();
});

var entryMethodPromise = Array.fromAsync.call(other.Array, [2]);
assertSame(
  Object.getPrototypeOf(entryMethodPromise),
  Promise.prototype,
  "entry method Promise Realm",
);
entryMethodPromise.then(function (result) {
  assertSame(Object.getPrototypeOf(result), otherArrayPrototype, "created result Array Realm");
  complete();
});

var rejectedCreatedMethodPromise = other.Array.fromAsync.call(Array, [], 0);
assertSame(
  Object.getPrototypeOf(rejectedCreatedMethodPromise),
  otherPromisePrototype,
  "created method rejected Promise Realm",
);
rejectedCreatedMethodPromise.then(
  function () {
    throw "invalid mapper fulfilled";
  },
  function (error) {
    assertSame(
      Object.getPrototypeOf(error),
      otherTypeErrorPrototype,
      "created method rejection error Realm",
    );
    complete();
  },
);

0;
