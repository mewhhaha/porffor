function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

async function rejectionOf(promise, label) {
  try {
    await promise;
  } catch (error) {
    return error;
  }
  throw label + " fulfilled";
}

let arrayDefaultCalls = 0;
let objectDefaultCalls = 0;
let varTrace = "";

function arrayDefault() {
  arrayDefaultCalls++;
  varTrace += "array-default>";
  return 3;
}

function objectDefault() {
  objectDefaultCalls++;
  varTrace += "object-default>";
  return 9;
}

async function consumeVarPatterns() {
  for (var [selected = arrayDefault(), ...remaining] of [
    [undefined, 4, 5],
    [7, 8],
  ]) {
    varTrace += "array-body:" + selected + ":" + remaining.join(",") + ">";
    await 0;
    varTrace += "array-resume:" + selected + ":" + remaining.join(",") + ">";
  }

  for (var { value: objectValue = objectDefault(), ...objectRest } of [
    { extra: 10 },
    { value: 11, tail: 12 },
  ]) {
    varTrace +=
      "object-body:" +
      objectValue +
      ":" +
      (objectRest.extra === undefined ? objectRest.tail : objectRest.extra) +
      ">";
    await 0;
    varTrace +=
      "object-resume:" +
      objectValue +
      ":" +
      (objectRest.extra === undefined ? objectRest.tail : objectRest.extra) +
      ">";
  }

  return (
    selected +
    ":" +
    remaining.join(",") +
    ":" +
    objectValue +
    ":" +
    objectRest.tail
  );
}

let assignmentIteration = 0;
let assignmentFallbackCalls = 0;
let assignmentTrace = "";
const assignmentTarget = { slot: 0 };
const assignmentRestTarget = {};

function assignmentSourceKey() {
  assignmentTrace += "source-key:" + assignmentIteration + ">";
  return assignmentIteration === 0 ? "value" : "other";
}

function assignmentTargetBase() {
  assignmentTrace += "target-base>";
  return assignmentTarget;
}

function assignmentTargetKey() {
  assignmentTrace += "target-key>";
  return "slot";
}

function assignmentFallback() {
  assignmentFallbackCalls++;
  assignmentTrace += "fallback>";
  return 6;
}

function assignmentRestBase() {
  assignmentTrace += "rest-base>";
  return assignmentRestTarget;
}

const firstAssignmentSource = { extra: 1 };
Object.defineProperty(firstAssignmentSource, "value", {
  enumerable: true,
  get: function () {
    assignmentTrace += "get:0>";
    return undefined;
  },
});
const secondAssignmentSource = { extra: 2 };
Object.defineProperty(secondAssignmentSource, "other", {
  enumerable: true,
  get: function () {
    assignmentTrace += "get:1>";
    return 8;
  },
});

async function consumeAssignmentPattern() {
  for (
    {
      [assignmentSourceKey()]: assignmentTargetBase()[assignmentTargetKey()] =
        assignmentFallback(),
      ...assignmentRestBase().rest
    } of [firstAssignmentSource, secondAssignmentSource]
  ) {
    assignmentTrace +=
      "body:" + assignmentTarget.slot + ":" + assignmentRestTarget.rest.extra + ">";
    await 0;
    assignmentTrace +=
      "resume:" +
      assignmentTarget.slot +
      ":" +
      assignmentRestTarget.rest.extra +
      ">";
    assignmentIteration++;
  }
}

const patternError = { label: "pattern default" };
const innerCloseError = { label: "inner close" };
const outerCloseError = { label: "outer close" };
let innerCloseCalls = 0;
let outerCloseCalls = 0;
let abruptBodyCalls = 0;
let abruptAwaitCalls = 0;
const abruptTarget = {};

function throwPatternError() {
  throw patternError;
}

const failingInnerIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return { value: undefined, done: false };
      },
      return: function () {
        innerCloseCalls++;
        throw innerCloseError;
      },
    };
  },
};

const failingOuterIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return { value: failingInnerIterable, done: false };
      },
      return: function () {
        outerCloseCalls++;
        throw outerCloseError;
      },
    };
  },
};

async function rejectAbruptPattern() {
  for ([abruptTarget.value = throwPatternError()] of failingOuterIterable) {
    abruptBodyCalls++;
    await (abruptAwaitCalls++, 0);
  }
}

async function main() {
  same(
    await consumeVarPatterns(),
    "7:8:11:12",
    "var pattern bindings after await"
  );
  same(arrayDefaultCalls, 1, "array default count");
  same(objectDefaultCalls, 1, "object default count");
  same(
    varTrace,
    "array-default>array-body:3:4,5>array-resume:3:4,5>" +
      "array-body:7:8>array-resume:7:8>" +
      "object-default>object-body:9:10>object-resume:9:10>" +
      "object-body:11:12>object-resume:11:12>",
    "var pattern lifecycle"
  );

  await consumeAssignmentPattern();
  same(assignmentFallbackCalls, 1, "assignment fallback count");
  same(assignmentTarget.slot, 8, "assignment target value");
  same(assignmentRestTarget.rest.extra, 2, "assignment rest value");
  same(
    assignmentTrace,
    "source-key:0>target-base>target-key>get:0>fallback>rest-base>body:6:1>resume:6:1>" +
      "source-key:1>target-base>target-key>get:1>rest-base>body:8:2>resume:8:2>",
    "assignment pattern lifecycle"
  );

  same(
    await rejectionOf(rejectAbruptPattern(), "abrupt pattern loop"),
    patternError,
    "pattern rejection identity"
  );
  same(innerCloseCalls, 1, "inner IteratorClose count");
  same(outerCloseCalls, 1, "outer IteratorClose count");
  same(abruptBodyCalls, 0, "abrupt pattern body calls");
  same(abruptAwaitCalls, 0, "abrupt pattern await calls");

  print("plain-async-sync-for-of:nonlexical-pattern-heads=ok");
}

main().then(undefined, function (error) {
  print("plain-async-sync-for-of:nonlexical-pattern-heads=FAILED:" + error);
});

0;
