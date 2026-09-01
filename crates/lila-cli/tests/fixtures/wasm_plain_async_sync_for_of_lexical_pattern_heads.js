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

const beforeClosures = [];
const afterClosures = [];
let completeEnvironmentTrace = "";

async function collectCompleteEnvironments() {
  for (
    const [
      captured,
      { pair: [carried = captured + 1] },
      ...remaining
    ] of [
      [1, { pair: [] }, 3],
      [5, { pair: [8] }, 9],
    ]
  ) {
    beforeClosures.push(function () {
      return captured;
    });
    completeEnvironmentTrace +=
      "body:" + captured + ":" + carried + ":" + remaining.join(",") + ">";
    await 0;
    afterClosures.push(function () {
      return captured;
    });
    completeEnvironmentTrace +=
      "resume:" + captured + ":" + carried + ":" + remaining.join(",") + ">";
  }
}

let mutableTrace = "";

async function mutateLetPatternAfterAwait() {
  for (let { value, delta = 1 } of [
    { value: 2 },
    { value: 4, delta: 3 },
  ]) {
    await 0;
    value += delta;
    mutableTrace += value + ">";
  }
}

let computedObjectKeyCalls = 0;
let objectRestTrace = "";

function selectObjectPatternKey() {
  computedObjectKeyCalls++;
  return "selected";
}

async function collectComputedObjectRest() {
  for (
    const { [selectObjectPatternKey()]: selected, ...remaining } of [
      { selected: 2, kept: 3 },
      { selected: 5, kept: 7 },
    ]
  ) {
    objectRestTrace +=
      "body:" + selected + ":" + remaining.kept + ":" + remaining.selected + ">";
    await 0;
    objectRestTrace +=
      "resume:" + selected + ":" + remaining.kept + ":" + remaining.selected + ">";
  }
}

const objectRestCopyError = { label: "lexical object rest copy" };
const objectRestCloseError = { label: "lexical object rest close" };
let objectRestCloseCalls = 0;
let objectRestBodyCalls = 0;
const abruptObjectRestIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return {
          value: {
            get broken() {
              throw objectRestCopyError;
            },
          },
          done: false,
        };
      },
      return: function () {
        objectRestCloseCalls++;
        throw objectRestCloseError;
      },
    };
  },
};

async function rejectAbruptObjectRest() {
  for (const { ...remaining } of abruptObjectRestIterable) {
    objectRestBodyCalls++;
    await 0;
  }
}

const outerLater = 99;
let tdzOuterCloseCalls = 0;
let tdzBodyCalls = 0;
let tdzAwaitCalls = 0;
const tdzOuterIterable = {
  [Symbol.iterator]: function () {
    let yielded = false;
    return {
      next: function () {
        if (yielded) return { value: undefined, done: true };
        yielded = true;
        return { value: [], done: false };
      },
      return: function () {
        tdzOuterCloseCalls++;
        return { value: undefined, done: true };
      },
    };
  },
};

async function rejectForwardDefaultReference() {
  for (let [first = later, later = outerLater] of tdzOuterIterable) {
    tdzBodyCalls++;
    await (tdzAwaitCalls++, 0);
  }
}

let capturedHeadReader;

async function preserveCapturedHeadTdz() {
  for (
    const [head] of ((capturedHeadReader = function () {
      return head;
    }),
    [[1]])
  ) {
    await 0;
  }
}

let constOuterCloseCalls = 0;
const constOuterIterable = {
  [Symbol.iterator]: function () {
    let yielded = false;
    return {
      next: function () {
        if (yielded) return { value: undefined, done: true };
        yielded = true;
        return { value: { locked: 7 }, done: false };
      },
      return: function () {
        constOuterCloseCalls++;
        return { value: undefined, done: true };
      },
    };
  },
};

async function rejectConstWriteAfterAwait() {
  for (const { locked } of constOuterIterable) {
    await 0;
    locked = 8;
  }
}

const patternError = { label: "lexical pattern default" };
const innerCloseError = { label: "lexical inner close" };
const outerCloseError = { label: "lexical outer close" };
let abruptInnerCloseCalls = 0;
let abruptOuterCloseCalls = 0;
let abruptBodyCalls = 0;
let abruptAwaitCalls = 0;

function throwPatternError() {
  throw patternError;
}

const abruptInnerIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return { value: undefined, done: false };
      },
      return: function () {
        abruptInnerCloseCalls++;
        throw innerCloseError;
      },
    };
  },
};

const abruptOuterIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return { value: abruptInnerIterable, done: false };
      },
      return: function () {
        abruptOuterCloseCalls++;
        throw outerCloseError;
      },
    };
  },
};

async function rejectAbruptLexicalPattern() {
  for (const [value = throwPatternError()] of abruptOuterIterable) {
    abruptBodyCalls++;
    await (abruptAwaitCalls++, 0);
  }
}

let emptyArrayInnerCloseCalls = 0;
const emptyArrayInnerIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return { value: 1, done: false };
      },
      return: function () {
        emptyArrayInnerCloseCalls++;
        return { value: undefined, done: true };
      },
    };
  },
};

async function consumeEmptyArrayPattern() {
  for (const [] of [emptyArrayInnerIterable]) {
    await 0;
  }
}

let emptyObjectOuterCloseCalls = 0;
let emptyObjectBodyCalls = 0;
const emptyObjectOuterIterable = {
  [Symbol.iterator]: function () {
    return {
      next: function () {
        return { value: null, done: false };
      },
      return: function () {
        emptyObjectOuterCloseCalls++;
        return { value: undefined, done: true };
      },
    };
  },
};

async function rejectEmptyObjectPattern() {
  for (const {} of emptyObjectOuterIterable) {
    emptyObjectBodyCalls++;
    await 0;
  }
}

async function main() {
  await collectCompleteEnvironments();
  same(
    beforeClosures.map(function (read) {
      return read();
    }).join(","),
    "1,5",
    "fresh closures before await"
  );
  same(
    afterClosures.map(function (read) {
      return read();
    }).join(","),
    "1,5",
    "fresh closures after await"
  );
  same(
    completeEnvironmentTrace,
    "body:1:2:3>resume:1:2:3>body:5:8:9>resume:5:8:9>",
    "complete fresh environment"
  );

  await mutateLetPatternAfterAwait();
  same(mutableTrace, "3>7>", "mutable let pattern after await");

  await collectComputedObjectRest();
  same(computedObjectKeyCalls, 2, "computed object key calls");
  same(
    objectRestTrace,
    "body:2:3:undefined>resume:2:3:undefined>" +
      "body:5:7:undefined>resume:5:7:undefined>",
    "computed object rest"
  );
  same(
    await rejectionOf(rejectAbruptObjectRest(), "abrupt object rest"),
    objectRestCopyError,
    "abrupt object rest identity"
  );
  same(objectRestCloseCalls, 1, "abrupt object rest outer close count");
  same(objectRestBodyCalls, 0, "abrupt object rest body calls");

  const tdzError = await rejectionOf(
    rejectForwardDefaultReference(),
    "forward default reference"
  );
  same(tdzError instanceof ReferenceError, true, "forward default TDZ error");
  same(tdzOuterCloseCalls, 1, "forward default outer close count");
  same(tdzBodyCalls, 0, "forward default body calls");
  same(tdzAwaitCalls, 0, "forward default await calls");

  await preserveCapturedHeadTdz();
  let capturedHeadError;
  try {
    capturedHeadReader();
  } catch (error) {
    capturedHeadError = error;
  }
  same(
    capturedHeadError instanceof ReferenceError,
    true,
    "captured iterable TDZ"
  );

  const constWriteError = await rejectionOf(
    rejectConstWriteAfterAwait(),
    "const pattern write"
  );
  same(constWriteError instanceof TypeError, true, "const pattern write error");
  same(constOuterCloseCalls, 1, "const pattern outer close count");

  same(
    await rejectionOf(rejectAbruptLexicalPattern(), "abrupt lexical pattern"),
    patternError,
    "abrupt lexical pattern identity"
  );
  same(abruptInnerCloseCalls, 1, "abrupt lexical inner close count");
  same(abruptOuterCloseCalls, 1, "abrupt lexical outer close count");
  same(abruptBodyCalls, 0, "abrupt lexical body calls");
  same(abruptAwaitCalls, 0, "abrupt lexical await calls");

  await consumeEmptyArrayPattern();
  same(emptyArrayInnerCloseCalls, 1, "empty array pattern inner close count");

  const emptyObjectError = await rejectionOf(
    rejectEmptyObjectPattern(),
    "empty object pattern"
  );
  same(emptyObjectError instanceof TypeError, true, "empty object pattern error");
  same(emptyObjectOuterCloseCalls, 1, "empty object pattern outer close count");
  same(emptyObjectBodyCalls, 0, "empty object pattern body calls");

  print("plain-async-sync-for-of:lexical-pattern-heads=ok");
}

main().then(undefined, function (error) {
  print("plain-async-sync-for-of:lexical-pattern-heads=FAILED:" + error);
});

0;
