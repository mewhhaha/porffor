function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var originalStringIteratorDescriptor = Object.getOwnPropertyDescriptor(
  String.prototype,
  Symbol.iterator
);
var accessorReceiver;
var iteratorMethodReceiver;
var iteratorMethodCalls = 0;
var nextCalls = 0;
var returnCalls = 0;
var customIterationResult;
var customIterator = {
  next: function () {
    nextCalls += 1;
    return { done: false, value: 4 };
  },
  return: function () {
    returnCalls += 1;
    return {};
  }
};
function customStringIterator() {
  "use strict";
  iteratorMethodCalls += 1;
  iteratorMethodReceiver = this;
  return customIterator;
}
Object.defineProperty(String.prototype, Symbol.iterator, {
  configurable: true,
  get: function () {
    "use strict";
    accessorReceiver = this;
    return customStringIterator;
  }
});
try {
  for (var customValue of "ab") {
    customIterationResult = customValue + 1;
    break;
  }
} finally {
  Object.defineProperty(
    String.prototype,
    Symbol.iterator,
    originalStringIteratorDescriptor
  );
}
assert(accessorReceiver === "ab", "String @@iterator getter receiver was boxed");
assert(iteratorMethodReceiver === "ab", "String @@iterator method receiver was boxed");
assert(iteratorMethodCalls === 1, "for-of called String @@iterator incorrectly");
assert(nextCalls === 1, "for-of stepped the custom String iterator incorrectly");
assert(customIterationResult === 5, "for-of retained String element typing");
assert(returnCalls === 1, "for-of break skipped IteratorClose");
assert(
  String.prototype[Symbol.iterator] === originalStringIteratorDescriptor.value,
  "String @@iterator descriptor was not restored"
);

var builtInIterator = originalStringIteratorDescriptor.value.call("xy");
var stringIteratorPrototype = Object.getPrototypeOf(builtInIterator);
var originalNextDescriptor = Object.getOwnPropertyDescriptor(
  stringIteratorPrototype,
  "next"
);
var replacementNextCalls = 0;
var replacementNextReceiverIsStringIterator = false;
var replacementNextValue;
Object.defineProperty(stringIteratorPrototype, "next", {
  configurable: true,
  writable: true,
  value: function () {
    "use strict";
    replacementNextCalls += 1;
    replacementNextReceiverIsStringIterator =
      Object.getPrototypeOf(this) === stringIteratorPrototype;
    return { done: false, value: "replacement" };
  }
});
try {
  for (var observedValue of "xy") {
    replacementNextValue = observedValue;
    break;
  }
} finally {
  Object.defineProperty(stringIteratorPrototype, "next", originalNextDescriptor);
}
assert(replacementNextCalls === 1, "for-of skipped String iterator next");
assert(
  replacementNextReceiverIsStringIterator,
  "String iterator next received the wrong receiver"
);
assert(replacementNextValue === "replacement", "for-of ignored replacement next");
assert(
  stringIteratorPrototype.next === originalNextDescriptor.value,
  "String iterator next descriptor was not restored"
);

true;
