function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

var growing = [1];
var growingTrace = "";
for (var growingValue of growing) {
  growingTrace += growingValue;
  if (growingValue === 1) {
    growing.push(2);
  }
}
assert(growingTrace === "12", "Array iterator did not observe length growth");

var inheritedGetterCalls = 0;
var inheritedValueCount = 0;
var inheritedSecondValue;
Object.defineProperty(Array.prototype, "1", {
  configurable: true,
  get: function () {
    inheritedGetterCalls += 1;
    return 20;
  }
});
try {
  for (var inheritedValue of [10, , 30]) {
    if (inheritedValueCount === 1) {
      inheritedSecondValue = inheritedValue;
    }
    inheritedValueCount += 1;
  }
} finally {
  delete Array.prototype[1];
}
assert(inheritedValueCount === 3, "Array iterator skipped a hole");
assert(inheritedSecondValue === 20, "Array iterator bypassed inherited indexed Get");
assert(inheritedGetterCalls === 1, "Array iterator repeated inherited indexed Get");

var originalArrayIterator = Array.prototype[Symbol.iterator];
var iteratorCalls = 0;
var nextCalls = 0;
var returnCalls = 0;
var customIterationResult;
Array.prototype[Symbol.iterator] = function () {
  iteratorCalls += 1;
  return {
    next: function () {
      nextCalls += 1;
      return { done: false, value: "4" };
    },
    return: function () {
      returnCalls += 1;
      return {};
    }
  };
};
try {
  for (var customValue of [1, 2]) {
    customIterationResult = customValue + 1;
    break;
  }
} finally {
  Array.prototype[Symbol.iterator] = originalArrayIterator;
}
assert(iteratorCalls === 1, "for-of skipped Array @@iterator");
assert(nextCalls === 1, "for-of stepped the custom iterator incorrectly");
assert(customIterationResult === "41", "for-of retained Array element typing");
assert(returnCalls === 1, "for-of break skipped IteratorClose");

true;
