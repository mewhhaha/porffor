// Consumer oracle for `await using` owned by a plain async function. A reached
// declaration acquires exactly one async-dispose method, registers before its
// binding becomes visible, and awaits every resource in reverse order before
// settling the function's Promise.

function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

function sameTrace(actual, expected, label) {
  same(actual.length, expected.length, label + " length");
  for (let i = 0; i < expected.length; i++) {
    same(actual[i], expected[i], label + " " + i);
  }
}

function asyncResource(label, trace, error, gate) {
  let count = 0;
  let value = { label: label };
  Object.defineProperty(value, Symbol.asyncDispose, {
    get: function () {
      trace.push(label + ":get-async");
      return async function () {
        if (this !== value) throw label + " receiver";
        count++;
        trace.push(label + ":dispose");
        await gate;
        if (error !== undefined) throw error;
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

function syncFallbackResource(label, trace, error) {
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
        if (error !== undefined) throw error;
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

async function protocolSelection(trace, direct, fallback) {
  await using selectedAsync = direct.value;
  await using selectedFallback = fallback.value;
  trace.push("protocol:body");
}

async function acquisitionAndRegistration(trace, first) {
  try {
    await using registered = {
      get [Symbol.asyncDispose]() {
        let state = "visible";
        try {
          registered;
        } catch (error) {
          state = error.name;
        }
        trace.push("acquisition:tdz:" + state);
        return async function () {
          trace.push("acquisition:registered-dispose");
        };
      },
    };
    await using rejected = first;
    trace.push("acquisition:unreachable");
  } catch (error) {
    trace.push("acquisition:caught:" + error.name);
  }
}

async function evaluatedEmpty(trace, sameJob) {
  trace.push("empty:start:" + sameJob());
  {
    await using empty = null;
    trace.push("empty:body:" + sameJob());
  }
  trace.push("empty:after:" + sameJob());
}

async function unreachableEmpty(trace, sameJob) {
  outer: {
    trace.push("unreachable:before:" + sameJob());
    break outer;
    await using empty = null;
  }
  trace.push("unreachable:after:" + sameJob());
}

async function sequentialDisposal(trace, first, second) {
  await using firstBinding = first.value;
  await using secondBinding = second.value;
  trace.push("sequential:body");
}

async function explicitReturn(trace, held) {
  await using resource = held.value;
  trace.push("return:body");
  return 42;
}

async function sourceThrow(trace, held, error) {
  await using resource = held.value;
  trace.push("throw:body");
  throw error;
}

async function disposerRejects(trace, held) {
  await using resource = held.value;
  trace.push("reject:body");
}

async function nestedScopes(trace, outer, innerFirst, innerSecond) {
  await using outerBinding = outer.value;
  {
    await using innerFirstBinding = innerFirst.value;
    await using innerSecondBinding = innerSecond.value;
    trace.push("nested:inner-body");
  }
  trace.push("nested:outer-body");
}

async function suppressedErrors(trace, first, second, bodyError) {
  await using firstBinding = first.value;
  await using secondBinding = second.value;
  trace.push("suppressed:body");
  throw bodyError;
}

async function main() {
  // @@asyncDispose wins. The sync fallback is read only after an undefined
  // async method, and its successful return value is discarded before Await.
  let protocolTrace = [];
  let direct = asyncResource("protocol:direct", protocolTrace);
  let fallback = syncFallbackResource("protocol:fallback", protocolTrace);
  let protocolResult = await protocolSelection(protocolTrace, direct, fallback);
  same(protocolResult, undefined, "normal completion result");
  sameTrace(
    protocolTrace,
    [
      "protocol:direct:get-async",
      "protocol:fallback:get-async",
      "protocol:fallback:get-sync",
      "protocol:body",
      "protocol:fallback:dispose",
      "protocol:direct:dispose",
    ],
    "protocol selection"
  );
  same(direct.count(), 1, "direct exactly once");
  same(fallback.count(), 1, "fallback exactly once");
  same(fallback.thenReads(), 0, "fallback thenable ignored");

  // Method validation precedes registration and binding initialization. A
  // later acquisition failure disposes the earlier registered resource.
  let acquisitionTrace = [];
  let invalid = {};
  invalid[Symbol.asyncDispose] = 1;
  await acquisitionAndRegistration(acquisitionTrace, invalid);
  sameTrace(
    acquisitionTrace,
    [
      "acquisition:tdz:ReferenceError",
      "acquisition:registered-dispose",
      "acquisition:caught:TypeError",
    ],
    "acquisition and registration"
  );

  // A reached empty entry still performs one scope-exit Await. An unreachable
  // declaration registers no entry and creates no suspension.
  let schedulingTrace = [];
  let inSameJob = true;
  let evaluated = evaluatedEmpty(schedulingTrace, function () {
    return inSameJob;
  });
  inSameJob = false;
  await evaluated;
  sameTrace(
    schedulingTrace,
    ["empty:start:true", "empty:body:true", "empty:after:false"],
    "evaluated empty scheduling"
  );

  schedulingTrace = [];
  inSameJob = true;
  let unreachable = unreachableEmpty(schedulingTrace, function () {
    return inSameJob;
  });
  inSameJob = false;
  await unreachable;
  sameTrace(
    schedulingTrace,
    ["unreachable:before:true", "unreachable:after:true"],
    "unreachable empty scheduling"
  );

  // Disposal is strictly sequential: the next LIFO callback cannot start
  // until the prior callback's Promise settles.
  let sequentialTrace = [];
  let releaseFirst;
  let releaseSecond;
  let firstGate = new Promise(function (resolve) {
    releaseFirst = resolve;
  });
  let secondGate = new Promise(function (resolve) {
    releaseSecond = resolve;
  });
  let firstStartedResolve;
  let secondStartedResolve;
  let firstStarted = new Promise(function (resolve) {
    firstStartedResolve = resolve;
  });
  let secondStarted = new Promise(function (resolve) {
    secondStartedResolve = resolve;
  });
  let first = asyncResource("sequential:first", sequentialTrace, undefined, {
    then: function (resolve) {
      firstStartedResolve();
      firstGate.then(resolve);
    },
  });
  let second = asyncResource("sequential:second", sequentialTrace, undefined, {
    then: function (resolve) {
      secondStartedResolve();
      secondGate.then(resolve);
    },
  });
  let sequential = sequentialDisposal(sequentialTrace, first, second);
  await secondStarted;
  same(
    sequentialTrace.indexOf("sequential:first:dispose"),
    -1,
    "first waits for second"
  );
  releaseSecond();
  await firstStarted;
  releaseFirst();
  await sequential;
  sameTrace(
    sequentialTrace,
    [
      "sequential:first:get-async",
      "sequential:second:get-async",
      "sequential:body",
      "sequential:second:dispose",
      "sequential:first:dispose",
    ],
    "sequential reverse awaits"
  );

  // Return and source Throw settle only after disposal; a rejected disposer
  // replaces Normal with the exact rejection reason.
  let returnTrace = [];
  let returned = asyncResource("return", returnTrace);
  same(await explicitReturn(returnTrace, returned), 42, "return value");
  sameTrace(
    returnTrace,
    ["return:get-async", "return:body", "return:dispose"],
    "return disposal"
  );

  let throwTrace = [];
  let bodyError = { name: "body" };
  let thrown = asyncResource("throw", throwTrace);
  let caughtBody;
  try {
    await sourceThrow(throwTrace, thrown, bodyError);
  } catch (error) {
    caughtBody = error;
  }
  same(caughtBody, bodyError, "body error identity");
  sameTrace(
    throwTrace,
    ["throw:get-async", "throw:body", "throw:dispose"],
    "throw disposal"
  );

  let rejectionTrace = [];
  let rejection = { name: "rejection" };
  let rejecting = syncFallbackResource("reject", rejectionTrace, rejection);
  let caughtRejection;
  try {
    await disposerRejects(rejectionTrace, rejecting);
  } catch (error) {
    caughtRejection = error;
  }
  same(caughtRejection, rejection, "disposer rejection identity");
  sameTrace(
    rejectionTrace,
    [
      "reject:get-async",
      "reject:get-sync",
      "reject:body",
      "reject:dispose",
    ],
    "rejected disposer"
  );

  // Nested scopes dispose inner resources before the outer scope, and every
  // registered callback is invoked exactly once.
  let nestedTrace = [];
  let outer = asyncResource("nested:outer", nestedTrace);
  let innerFirst = asyncResource("nested:inner:first", nestedTrace);
  let innerSecond = asyncResource("nested:inner:second", nestedTrace);
  await nestedScopes(nestedTrace, outer, innerFirst, innerSecond);
  sameTrace(
    nestedTrace,
    [
      "nested:outer:get-async",
      "nested:inner:first:get-async",
      "nested:inner:second:get-async",
      "nested:inner-body",
      "nested:inner:second:dispose",
      "nested:inner:first:dispose",
      "nested:outer-body",
      "nested:outer:dispose",
    ],
    "nested disposal"
  );
  same(outer.count(), 1, "outer once");
  same(innerFirst.count(), 1, "inner first once");
  same(innerSecond.count(), 1, "inner second once");

  // Each later disposal rejection becomes the new SuppressedError.error.
  let suppressedTrace = [];
  let suppressedBody = { name: "suppressed body" };
  let firstError = { name: "first disposal" };
  let secondError = { name: "second disposal" };
  let suppressFirst = asyncResource(
    "suppressed:first",
    suppressedTrace,
    firstError
  );
  let suppressSecond = asyncResource(
    "suppressed:second",
    suppressedTrace,
    secondError
  );
  let folded;
  try {
    await suppressedErrors(
      suppressedTrace,
      suppressFirst,
      suppressSecond,
      suppressedBody
    );
  } catch (error) {
    folded = error;
  }
  same(folded instanceof SuppressedError, true, "outer suppressed error");
  same(folded.error, firstError, "outer error");
  same(
    folded.suppressed instanceof SuppressedError,
    true,
    "inner suppressed error"
  );
  same(folded.suppressed.error, secondError, "inner error");
  same(folded.suppressed.suppressed, suppressedBody, "suppressed body");
  sameTrace(
    suppressedTrace,
    [
      "suppressed:first:get-async",
      "suppressed:second:get-async",
      "suppressed:body",
      "suppressed:second:dispose",
      "suppressed:first:dispose",
    ],
    "suppressed disposal order"
  );

  print("await-using-plain-async:true");
}

main().catch(function (error) {
  print("await-using-plain-async:FAILED:" + error);
  throw error;
});
