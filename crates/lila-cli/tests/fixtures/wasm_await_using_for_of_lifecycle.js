// Consumer oracle for plain-async `await using` in a synchronous `for-of`
// head. Custom iterables keep per-iteration disposal, IteratorClose and the
// generic iterator protocol observable.

function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

function sameTrace(actual, expected, label) {
  same(actual.length, expected.length, label + " length");
  for (let i = 0; i < expected.length; i++) {
    same(actual[i], expected[i], label + " " + i);
  }
}

function asyncResource(label, trace, error) {
  let count = 0;
  let value = { label: label };
  Object.defineProperty(value, Symbol.asyncDispose, {
    get: function () {
      trace.push(label + ":get-async");
      return function () {
        if (this !== value) throw label + " receiver";
        count++;
        trace.push(label + ":dispose-start");
        return Promise.resolve().then(function () {
          trace.push(label + ":dispose-end");
          if (error !== undefined) throw error;
        });
      };
    },
  });
  Object.defineProperty(value, Symbol.dispose, {
    get: function () {
      trace.push(label + ":bad-get-sync");
      throw label + " sync fallback read";
    },
  });
  return {
    value: value,
    count: function () {
      return count;
    },
  };
}

function syncFallbackResource(label, trace) {
  let count = 0;
  let thenReads = 0;
  let value = { label: label };
  Object.defineProperty(value, Symbol.asyncDispose, {
    get: function () {
      trace.push(label + ":get-async");
      return undefined;
    },
  });
  Object.defineProperty(value, Symbol.dispose, {
    get: function () {
      trace.push(label + ":get-sync");
      return function () {
        if (this !== value) throw label + " receiver";
        count++;
        trace.push(label + ":dispose");
        return {
          get then() {
            thenReads++;
            trace.push(label + ":bad-then");
            return function () {};
          },
        };
      };
    },
  });
  return {
    value: value,
    count: function () {
      return count;
    },
    thenReads: function () {
      return thenReads;
    },
  };
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

async function headTdz() {
  let shadowed = "outer";
  let caught;
  try {
    for (await using shadowed of [shadowed]) {}
  } catch (error) {
    caught = error;
  }
  same(caught instanceof ReferenceError, true, "head binding TDZ");
  same(shadowed, "outer", "outer binding after TDZ");
}

async function validOfGrammar() {
  for (await using of of []) {}
  return true;
}

async function acquisitionTdz(trace) {
  let value = {
    get [Symbol.asyncDispose]() {
      let state = "visible";
      try {
        acquired;
      } catch (error) {
        state = error.name;
      }
      trace.push("acquire-tdz:" + state);
      return function () {
        trace.push("acquire-tdz:dispose");
      };
    },
  };
  for (await using acquired of [value]) {
    same(acquired, value, "binding initialized after acquisition");
    trace.push("acquire-tdz:body");
  }
}

async function normalIterations(trace, direct, fallback) {
  let captures = [];
  for (
    await using current of genericIterable(
      [direct.value, fallback.value],
      trace,
      "normal"
    )
  ) {
    captures.push(function () {
      return current;
    });
    trace.push("normal:body:" + current.label);
    if (current === direct.value) {
      same(direct.count(), 0, "direct undisposed in body");
    } else {
      same(fallback.count(), 0, "fallback undisposed in body");
    }
  }
  same(captures[0](), direct.value, "first fresh captured binding");
  same(captures[1](), fallback.value, "second fresh captured binding");
}

async function localContinue(trace, first, second) {
  continueLoop: for (
    await using current of genericIterable(
      [first.value, second.value],
      trace,
      "continue"
    )
  ) {
    trace.push("continue:body:" + current.label);
    continue continueLoop;
  }
}

async function breakLoop(trace, held) {
  breakLoop: for (
    await using current of genericIterable([held.value], trace, "break")
  ) {
    trace.push("break:body");
    break breakLoop;
  }
  trace.push("break:after");
}

async function returnLoop(trace, held) {
  for (
    await using current of genericIterable([held.value], trace, "return")
  ) {
    trace.push("return:body");
    return current.label;
  }
  return "unreachable";
}

async function throwLoop(trace, held, error) {
  for (
    await using current of genericIterable([held.value], trace, "throw")
  ) {
    trace.push("throw:body");
    throw error;
  }
}

async function immutableBinding(trace, held) {
  for (
    await using current of genericIterable([held.value], trace, "immutable")
  ) {
    trace.push("immutable:body");
    current = null;
  }
}

async function laterAcquisitionFailure(trace, first, invalid) {
  for (
    await using current of genericIterable(
      [first.value, invalid],
      trace,
      "acquisition"
    )
  ) {
    trace.push("acquisition:body:" + current.label);
  }
}

async function nestedResumeRead(trace, outer, inner) {
  for (
    await using current of genericIterable([outer.value], trace, "resume-read")
  ) {
    {
      await using nested = inner.value;
      same(nested, inner.value, "nested resume binding");
      trace.push("resume-read:inner-body");
    }
    same(current, outer.value, "outer head binding after nested finalizer");
    trace.push("resume-read:outer:" + current.label);
  }
}

async function nestedSuppression(trace, outer, inner, bodyError) {
  for (
    await using current of genericIterable([outer.value], trace, "suppressed")
  ) {
    await using nested = inner.value;
    same(current, outer.value, "outer binding in nested scope");
    same(nested, inner.value, "inner binding");
    trace.push("suppressed:body");
    throw bodyError;
  }
}

async function main() {
  await headTdz();
  same(await validOfGrammar(), true, "valid of grammar");
  let acquisitionTdzTrace = [];
  await acquisitionTdz(acquisitionTdzTrace);
  sameTrace(
    acquisitionTdzTrace,
    [
      "acquire-tdz:ReferenceError",
      "acquire-tdz:body",
      "acquire-tdz:dispose",
    ],
    "acquisition before binding initialization"
  );

  let normalTrace = [];
  let direct = asyncResource("normal:direct", normalTrace);
  let fallback = syncFallbackResource("normal:fallback", normalTrace);
  await normalIterations(normalTrace, direct, fallback);
  sameTrace(
    normalTrace,
    [
      "normal:iterator",
      "normal:next:0",
      "normal:direct:get-async",
      "normal:body:normal:direct",
      "normal:direct:dispose-start",
      "normal:direct:dispose-end",
      "normal:next:1",
      "normal:fallback:get-async",
      "normal:fallback:get-sync",
      "normal:body:normal:fallback",
      "normal:fallback:dispose",
      "normal:next:2",
    ],
    "normal disposal before next"
  );
  same(direct.count(), 1, "direct exactly once");
  same(fallback.count(), 1, "fallback exactly once");
  same(fallback.thenReads(), 0, "sync fallback return ignored");

  let continueTrace = [];
  let continueFirst = asyncResource("continue:first", continueTrace);
  let continueSecond = asyncResource("continue:second", continueTrace);
  await localContinue(continueTrace, continueFirst, continueSecond);
  sameTrace(
    continueTrace,
    [
      "continue:iterator",
      "continue:next:0",
      "continue:first:get-async",
      "continue:body:continue:first",
      "continue:first:dispose-start",
      "continue:first:dispose-end",
      "continue:next:1",
      "continue:second:get-async",
      "continue:body:continue:second",
      "continue:second:dispose-start",
      "continue:second:dispose-end",
      "continue:next:2",
    ],
    "continue disposes before next without close"
  );

  let breakTrace = [];
  let broken = asyncResource("break", breakTrace);
  await breakLoop(breakTrace, broken);
  sameTrace(
    breakTrace,
    [
      "break:iterator",
      "break:next:0",
      "break:get-async",
      "break:body",
      "break:dispose-start",
      "break:dispose-end",
      "break:get-return",
      "break:close",
      "break:after",
    ],
    "break disposal before close"
  );

  let returnTrace = [];
  let returned = asyncResource("return", returnTrace);
  same(await returnLoop(returnTrace, returned), "return", "return value");
  sameTrace(
    returnTrace,
    [
      "return:iterator",
      "return:next:0",
      "return:get-async",
      "return:body",
      "return:dispose-start",
      "return:dispose-end",
      "return:get-return",
      "return:close",
    ],
    "return disposal before close"
  );

  let bodyError = { id: "body" };
  let throwTrace = [];
  let thrown = asyncResource("throw", throwTrace);
  let throwCaught;
  try {
    await throwLoop(throwTrace, thrown, bodyError);
  } catch (error) {
    throwCaught = error;
  }
  same(throwCaught, bodyError, "body error identity");
  sameTrace(
    throwTrace,
    [
      "throw:iterator",
      "throw:next:0",
      "throw:get-async",
      "throw:body",
      "throw:dispose-start",
      "throw:dispose-end",
      "throw:get-return",
      "throw:close",
    ],
    "throw disposal before close"
  );

  let immutableTrace = [];
  let immutable = asyncResource("immutable", immutableTrace);
  let immutableCaught;
  try {
    await immutableBinding(immutableTrace, immutable);
  } catch (error) {
    immutableCaught = error;
  }
  same(immutableCaught instanceof TypeError, true, "head binding immutable");
  sameTrace(
    immutableTrace,
    [
      "immutable:iterator",
      "immutable:next:0",
      "immutable:get-async",
      "immutable:body",
      "immutable:dispose-start",
      "immutable:dispose-end",
      "immutable:get-return",
      "immutable:close",
    ],
    "immutable assignment disposal before close"
  );

  let acquisitionTrace = [];
  let first = asyncResource("acquisition:first", acquisitionTrace);
  let acquisitionError = { id: "acquisition" };
  let invalid = { label: "invalid" };
  Object.defineProperty(invalid, Symbol.asyncDispose, {
    get: function () {
      acquisitionTrace.push("acquisition:get-async");
      throw acquisitionError;
    },
  });
  let acquisitionCaught;
  try {
    await laterAcquisitionFailure(acquisitionTrace, first, invalid);
  } catch (error) {
    acquisitionCaught = error;
  }
  same(acquisitionCaught, acquisitionError, "acquisition error identity");
  sameTrace(
    acquisitionTrace,
    [
      "acquisition:iterator",
      "acquisition:next:0",
      "acquisition:first:get-async",
      "acquisition:body:acquisition:first",
      "acquisition:first:dispose-start",
      "acquisition:first:dispose-end",
      "acquisition:next:1",
      "acquisition:get-async",
      "acquisition:get-return",
      "acquisition:close",
    ],
    "later acquisition failure closes after prior disposal"
  );

  let resumeReadTrace = [];
  let resumeReadOuter = asyncResource("resume-read:head", resumeReadTrace);
  let resumeReadInner = asyncResource("resume-read:inner", resumeReadTrace);
  await nestedResumeRead(resumeReadTrace, resumeReadOuter, resumeReadInner);
  sameTrace(
    resumeReadTrace,
    [
      "resume-read:iterator",
      "resume-read:next:0",
      "resume-read:head:get-async",
      "resume-read:inner:get-async",
      "resume-read:inner-body",
      "resume-read:inner:dispose-start",
      "resume-read:inner:dispose-end",
      "resume-read:outer:resume-read:head",
      "resume-read:head:dispose-start",
      "resume-read:head:dispose-end",
      "resume-read:next:1",
    ],
    "outer head binding survives nested finalizer"
  );
  same(resumeReadOuter.count(), 1, "resume-read head exactly once");
  same(resumeReadInner.count(), 1, "resume-read inner exactly once");

  let suppressionTrace = [];
  let outerError = { id: "outer disposal" };
  let innerError = { id: "inner disposal" };
  let suppressedBody = { id: "suppressed body" };
  let outer = asyncResource("suppressed:outer", suppressionTrace, outerError);
  let inner = asyncResource("suppressed:inner", suppressionTrace, innerError);
  let folded;
  try {
    await nestedSuppression(suppressionTrace, outer, inner, suppressedBody);
  } catch (error) {
    folded = error;
  }
  same(folded instanceof SuppressedError, true, "outer suppression brand");
  same(folded.error, outerError, "outer suppression error");
  same(folded.suppressed instanceof SuppressedError, true, "inner suppression brand");
  same(folded.suppressed.error, innerError, "inner suppression error");
  same(folded.suppressed.suppressed, suppressedBody, "body suppression identity");
  sameTrace(
    suppressionTrace,
    [
      "suppressed:iterator",
      "suppressed:next:0",
      "suppressed:outer:get-async",
      "suppressed:inner:get-async",
      "suppressed:body",
      "suppressed:inner:dispose-start",
      "suppressed:inner:dispose-end",
      "suppressed:outer:dispose-start",
      "suppressed:outer:dispose-end",
      "suppressed:get-return",
      "suppressed:close",
    ],
    "nested LIFO suppression before close"
  );
  same(outer.count(), 1, "outer suppression exactly once");
  same(inner.count(), 1, "inner suppression exactly once");

  print("await-using-for-of:true");
}

main().catch(function (error) {
  print("await-using-for-of:FAILED:" + error);
  throw error;
});
