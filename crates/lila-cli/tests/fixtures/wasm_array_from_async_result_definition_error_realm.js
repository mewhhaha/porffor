function assertSame(actual, expected, label) {
  if (!Object.is(actual, expected)) throw label;
}

var other = __lilaCreateRealm().global;
var entryTypeErrorPrototype = TypeError.prototype;
var otherTypeErrorPrototype = other.TypeError.prototype;
var completed = 0;

function complete() {
  completed = completed + 1;
  if (completed === 7) {
    print("array-from-async-result-definition-error-realm:ok");
  }
}

function expectTypeError(promise, expectedPrototype, label) {
  promise.then(
    function () {
      throw label + " fulfilled";
    },
    function (error) {
      assertSame(Object.getPrototypeOf(error), expectedPrototype, label);
      complete();
    },
  );
}

var entryLockedIndex = {};
Object.defineProperty(entryLockedIndex, "0", {
  configurable: false,
  value: 0,
  writable: false,
});
var foreignConstructorReturningEntryIndex = new Proxy(other.Object, {
  construct: function () {
    return entryLockedIndex;
  },
});
expectTypeError(
  Array.fromAsync.call(foreignConstructorReturningEntryIndex, {
    0: 1,
    length: 1,
  }),
  entryTypeErrorPrototype,
  "entry method ignores foreign constructor Realm for index failure",
);

var otherLockedIndex = other.Object.create(other.Object.prototype);
Object.defineProperty(otherLockedIndex, "0", {
  configurable: false,
  value: 0,
  writable: false,
});
var entryConstructorReturningOtherIndex = new Proxy(Object, {
  construct: function () {
    return otherLockedIndex;
  },
});
expectTypeError(
  other.Array.fromAsync.call(entryConstructorReturningOtherIndex, {
    0: 1,
    length: 1,
  }),
  otherTypeErrorPrototype,
  "created method ignores entry constructor Realm for index failure",
);

var entryNonExtensible = Object.preventExtensions({});
var foreignConstructorReturningEntryNonExtensible = new Proxy(other.Object, {
  construct: function () {
    return entryNonExtensible;
  },
});
Array.fromAsync.call(foreignConstructorReturningEntryNonExtensible, {
  0: 1,
  length: 1,
}).then(
  function () {
    throw "non-extensible result fulfilled";
  },
  function (error) {
    assertSame(
      Object.getPrototypeOf(error),
      entryTypeErrorPrototype,
      "non-extensible result TypeError Realm",
    );
    assertSame(
      Object.hasOwn(error, "message"),
      false,
      "non-extensible result TypeError has no own message",
    );
    complete();
  },
);

var entryLockedLength = {};
Object.defineProperty(entryLockedLength, "length", {
  configurable: false,
  value: 99,
  writable: false,
});
var foreignConstructorReturningEntryLength = new Proxy(other.Object, {
  construct: function () {
    return entryLockedLength;
  },
});
expectTypeError(
  Array.fromAsync.call(foreignConstructorReturningEntryLength, {
    0: 1,
    length: 1,
  }),
  entryTypeErrorPrototype,
  "entry method ignores foreign constructor Realm for length failure",
);

var otherLockedLength = other.Object.create(other.Object.prototype);
Object.defineProperty(otherLockedLength, "length", {
  configurable: false,
  value: 99,
  writable: false,
});
var entryConstructorReturningOtherLength = new Proxy(Object, {
  construct: function () {
    return otherLockedLength;
  },
});
expectTypeError(
  other.Array.fromAsync.call(entryConstructorReturningOtherLength, {
    0: 1,
    length: 1,
  }),
  otherTypeErrorPrototype,
  "created method ignores entry constructor Realm for length failure",
);

var otherZeroLength = other.Object.create(other.Object.prototype);
Object.defineProperty(otherZeroLength, "length", {
  configurable: false,
  value: 99,
  writable: false,
});
var entryConstructorReturningOtherZeroLength = new Proxy(Object, {
  construct: function () {
    return otherZeroLength;
  },
});
expectTypeError(
  other.Array.fromAsync.call(entryConstructorReturningOtherZeroLength, {
    length: 0,
  }),
  otherTypeErrorPrototype,
  "zero-length fast path uses created method Realm",
);

var setterError = {};
var throwingLengthResult = {};
Object.defineProperty(throwingLengthResult, "length", {
  configurable: true,
  set: function () {
    throw setterError;
  },
});
var foreignConstructorReturningThrowingLength = new Proxy(other.Object, {
  construct: function () {
    return throwingLengthResult;
  },
});
other.Array.fromAsync.call(foreignConstructorReturningThrowingLength, {
  length: 0,
}).then(
  function () {
    throw "throwing length setter fulfilled";
  },
  function (error) {
    assertSame(error, setterError, "length setter error identity");
    complete();
  },
);

0;
