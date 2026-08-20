function check(condition, message) {
  if (!condition) throw message;
}

var initializerClosures = [];
var testClosures = [];
var bodyClosures = [];
var updateClosures = [];
for (
  let index = (initializerClosures.push(function() { return index; }), 0);
  (testClosures.push(function() { return index; }), index < 2);
  (index += 1, updateClosures.push(function() { return index; }))
) {
  bodyClosures.push(function() { return index; });
}
check(initializerClosures[0]() === 0, "classic initializer environment");
check(bodyClosures[0]() === 0 && bodyClosures[1]() === 1, "classic body environments");
check(updateClosures[0]() === 1 && updateClosures[1]() === 2, "classic update environments");
check(
  testClosures[0]() === 0 && testClosures[1]() === 1 && testClosures[2]() === 2,
  "classic test environments",
);

var constClosure;
for (const constant = 7; ; ) {
  constClosure = function() { return constant; };
  break;
}
check(constClosure() === 7, "classic const environment");

var arrayClosures = [];
for (let value of [10, 20]) {
  arrayClosures.push(function() { return value; });
}
check(arrayClosures[0]() === 10 && arrayClosures[1]() === 20, "array for-of environments");

var stringClosures = [];
for (let value of "ab") {
  stringClosures.push(function() { return value; });
}
check(stringClosures[0]() === "a" && stringClosures[1]() === "b", "string for-of environments");

var arrayKeyClosures = [];
for (let key in [10, 20]) {
  arrayKeyClosures.push(function() { return key; });
}
check(arrayKeyClosures[0]() === "0" && arrayKeyClosures[1]() === "1", "array for-in environments");

var stringKeyClosures = [];
for (let key in "ab") {
  stringKeyClosures.push(function() { return key; });
}
check(stringKeyClosures[0]() === "0" && stringKeyClosures[1]() === "1", "string for-in environments");

var objectKeyClosures = [];
for (let key in { first: 1, second: 2 }) {
  objectKeyClosures.push(function() { return key; });
}
check(
  objectKeyClosures[0]() === "first" && objectKeyClosures[1]() === "second",
  "object for-in environments",
);

var readOuter;
var mutateOuter;
var abrupt = {};
try {
  {
    let outer = 0;
    readOuter = function() { return outer; };
    mutateOuter = function() { outer += 1; };
    for (let value of [0]) {
      mutateOuter();
      check(outer === 1 && value === 0, "iteration parent cell identity");
      try {
        throw abrupt;
      } finally {
        outer += 1;
      }
    }
  }
} catch (error) {
  check(error === abrupt, "iteration abrupt value");
}
check(readOuter() === 2, "iteration abrupt parent restoration");

var delayedHead;
for (let shadow of (delayedHead = function() { return shadow; }, [])) {}
var sawHeadTdz = false;
try {
  delayedHead();
} catch (error) {
  sawHeadTdz = error instanceof ReferenceError;
}
check(sawHeadTdz, "retained for-of head TDZ");

var labelledTrace = "";
outerLoop: for (let outerValue of [0, 1]) {
  for (let innerValue of [0]) {
    try {
      if (outerValue === 0) continue outerLoop;
      break outerLoop;
    } finally {
      labelledTrace += String(outerValue) + String(innerValue);
    }
  }
}
check(labelledTrace === "0010", "labelled lexical loop cleanup");

function makeIterator(first, second, closeBehavior) {
  var position = 0;
  var iterator = {
    next: function() {
      position += 1;
      if (position === 1) return { done: false, value: first };
      if (position === 2) return { done: false, value: second };
      return { done: true };
    },
    return: function() {
      closeBehavior.calls = closeBehavior.calls + 1;
      if (closeBehavior.throwValue !== undefined) throw closeBehavior.throwValue;
      if (closeBehavior.primitiveReturn) return 1;
      return {};
    },
  };
  iterator[Symbol.iterator] = function() { return iterator; };
  return iterator;
}

var retainedClose = { calls: 0 };
var iteratorClosures = [];
for (let value of makeIterator(30, 40, retainedClose)) {
  iteratorClosures.push(function() { return value; });
}
check(iteratorClosures[0]() === 30 && iteratorClosures[1]() === 40, "iterator for-of environments");
check(retainedClose.calls === 0, "normal iterator completion must not close");

var continueClose = { calls: 0 };
for (let value of makeIterator(1, 2, continueClose)) {
  if (value === 1) continue;
  break;
}
check(continueClose.calls === 1, "current continue and break IteratorClose");

true;
