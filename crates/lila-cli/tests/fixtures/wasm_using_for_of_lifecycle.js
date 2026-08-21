// Consumer oracle for synchronous `using` in a `for-of` head. Lifecycle and
// IteratorClose cases use custom iterables so the generic iterator protocol,
// per-iteration disposal capability and close ordering remain observable.

function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

function sameTrace(actual, expected, label) {
  same(actual.length, expected.length, label + " length");
  for (let i = 0; i < expected.length; i++) {
    same(actual[i], expected[i], label + " " + i);
  }
}

function resource(label, trace, error) {
  let value = { label: label };
  value[Symbol.dispose] = function () {
    if (this !== value) throw label + " receiver";
    trace.push(label + ":dispose");
    if (error !== undefined) throw error;
  };
  return value;
}

function genericIterable(values, trace, label) {
  let index = 0;
  let iterator = {};
  let iterable = {};
  iterator.next = function () {
    if (this !== iterator) throw label + " next receiver";
    trace.push(label + ":next:" + index);
    if (index === values.length) return { value: undefined, done: true };
    return { value: values[index++], done: false };
  };
  let close = function () {
    if (this !== iterator) throw label + " return receiver";
    trace.push(label + ":close");
    return { value: undefined, done: true };
  };
  Object.defineProperty(iterator, "return", {
    get: function () {
      if (this !== iterator) throw label + " return getter receiver";
      trace.push(label + ":get-return");
      return close;
    },
  });
  iterable[Symbol.iterator] = function () {
    if (this !== iterable) throw label + " iterator receiver";
    trace.push(label + ":iterator");
    return iterator;
  };
  return iterable;
}

// The iterable expression is evaluated inside the head binding's TDZ. The
// comma keeps abrupt propagation load-bearing before its right operand, while
// the same-named outer binding becomes visible again after the loop fails.
let shadowed = "outer";
let tdzCaught;
try {
  for (using shadowed of (shadowed, {})) {}
} catch (error) {
  tdzCaught = error;
}
same(tdzCaught instanceof ReferenceError, true, "head binding TDZ");
same(shadowed, "outer", "outer binding restored after TDZ");

// A labelled continue targeting this loop disposes the current iteration,
// does not close the iterator and creates a fresh captured binding before the
// next call to `next`.
let continueTrace = [];
let captured = [];
let first = resource("continue:first", continueTrace);
let second = resource("continue:second", continueTrace);
let third = resource("continue:third", continueTrace);
continueLoop: for (
  using current of genericIterable(
    [first, second, third],
    continueTrace,
    "continue"
  )
) {
  captured.push(function () {
    return current;
  });
  if (captured.length < 3) continue continueLoop;
}
same(captured[0](), first, "first fresh captured binding");
same(captured[1](), second, "second fresh captured binding");
same(captured[2](), third, "third fresh captured binding");
sameTrace(
  continueTrace,
  [
    "continue:iterator",
    "continue:next:0",
    "continue:first:dispose",
    "continue:next:1",
    "continue:second:dispose",
    "continue:next:2",
    "continue:third:dispose",
    "continue:next:3",
  ],
  "continue disposal before next without close"
);

// A continue targeting an outer loop is not local LoopContinues. It disposes
// this iteration, closes this iterator and only then resumes the outer loop.
let outerContinueTrace = [];
let outerContinueCount = 0;
outerLoop: for (let outerIndex = 0; outerIndex < 2; outerIndex++) {
  for (
    using current of genericIterable(
      [resource("outer:" + outerIndex, outerContinueTrace)],
      outerContinueTrace,
      "outer:" + outerIndex
    )
  ) {
    outerContinueTrace.push("outer:body:" + outerIndex);
    outerContinueCount++;
    continue outerLoop;
  }
}
same(outerContinueCount, 2, "outer continue count");
sameTrace(
  outerContinueTrace,
  [
    "outer:0:iterator",
    "outer:0:next:0",
    "outer:body:0",
    "outer:0:dispose",
    "outer:0:get-return",
    "outer:0:close",
    "outer:1:iterator",
    "outer:1:next:0",
    "outer:body:1",
    "outer:1:dispose",
    "outer:1:get-return",
    "outer:1:close",
  ],
  "outer continue disposal before close"
);

// Break disposes the entered iteration before closing its iterator.
let breakTrace = [];
breakLoop: for (
  using current of genericIterable(
    [resource("break", breakTrace)],
    breakTrace,
    "break"
  )
) {
  breakTrace.push("break:body");
  break breakLoop;
}
sameTrace(
  breakTrace,
  [
    "break:iterator",
    "break:next:0",
    "break:body",
    "break:dispose",
    "break:get-return",
    "break:close",
  ],
  "break disposal before close"
);

// Return keeps its value but still disposes before IteratorClose.
let returnTrace = [];
function returnFromUsingForOf() {
  for (
    using current of genericIterable(
      [resource("return", returnTrace)],
      returnTrace,
      "return"
    )
  ) {
    returnTrace.push("return:body");
    return current.label;
  }
  return "unreachable";
}
same(returnFromUsingForOf(), "return", "return completion preserved");
sameTrace(
  returnTrace,
  [
    "return:iterator",
    "return:next:0",
    "return:body",
    "return:dispose",
    "return:get-return",
    "return:close",
  ],
  "return disposal before close"
);

// A body throw is the pending completion while disposal runs, then the folded
// completion is supplied to IteratorClose.
let bodyError = { id: "body" };
let throwTrace = [];
let throwCaught;
try {
  for (
    using current of genericIterable(
      [resource("throw", throwTrace)],
      throwTrace,
      "throw"
    )
  ) {
    throwTrace.push("throw:body");
    throw bodyError;
  }
} catch (error) {
  throwCaught = error;
}
same(throwCaught, bodyError, "body error identity");
sameTrace(
  throwTrace,
  [
    "throw:iterator",
    "throw:next:0",
    "throw:body",
    "throw:dispose",
    "throw:get-return",
    "throw:close",
  ],
  "throw disposal before close"
);

// A disposer failure turns a normal body completion abrupt and therefore
// closes the iterator only after the disposer has run.
let disposerError = { id: "disposer" };
let disposerTrace = [];
let disposerCaught;
try {
  for (
    using current of genericIterable(
      [resource("disposer", disposerTrace, disposerError)],
      disposerTrace,
      "disposer"
    )
  ) {
    disposerTrace.push("disposer:body");
  }
} catch (error) {
  disposerCaught = error;
}
same(disposerCaught, disposerError, "disposer error identity");
sameTrace(
  disposerTrace,
  [
    "disposer:iterator",
    "disposer:next:0",
    "disposer:body",
    "disposer:dispose",
    "disposer:get-return",
    "disposer:close",
  ],
  "disposer throw before close"
);

// A later iteration's GetMethod failure occurs only after the previous
// iteration was disposed. The empty current capability is consumed before the
// acquisition error closes the iterator.
let acquisitionError = { id: "acquisition" };
let acquisitionTrace = [];
let invalid = {};
Object.defineProperty(invalid, Symbol.dispose, {
  get: function () {
    acquisitionTrace.push("acquisition:get-dispose");
    throw acquisitionError;
  },
});
let acquisitionCaught;
try {
  for (
    using current of genericIterable(
      [resource("acquisition:first", acquisitionTrace), invalid],
      acquisitionTrace,
      "acquisition"
    )
  ) {
    acquisitionTrace.push("acquisition:body:" + current.label);
  }
} catch (error) {
  acquisitionCaught = error;
}
same(acquisitionCaught, acquisitionError, "acquisition error identity");
sameTrace(
  acquisitionTrace,
  [
    "acquisition:iterator",
    "acquisition:next:0",
    "acquisition:body:acquisition:first",
    "acquisition:first:dispose",
    "acquisition:next:1",
    "acquisition:get-dispose",
    "acquisition:get-return",
    "acquisition:close",
  ],
  "acquisition failure disposes then closes"
);

// The iteration binding is immutable. Its assignment failure is a body throw,
// so disposal still precedes IteratorClose.
let immutableTrace = [];
let immutableCaught;
try {
  for (
    using current of genericIterable(
      [resource("immutable", immutableTrace)],
      immutableTrace,
      "immutable"
    )
  ) {
    immutableTrace.push("immutable:body");
    current = null;
  }
} catch (error) {
  immutableCaught = error;
}
same(immutableCaught instanceof TypeError, true, "using binding immutable");
sameTrace(
  immutableTrace,
  [
    "immutable:iterator",
    "immutable:next:0",
    "immutable:body",
    "immutable:dispose",
    "immutable:get-return",
    "immutable:close",
  ],
  "immutable throw disposal before close"
);

true;
