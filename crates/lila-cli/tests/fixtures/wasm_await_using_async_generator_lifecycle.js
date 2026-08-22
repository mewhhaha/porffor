// Consumer oracle for `await using` owned by an async generator. A reached
// declaration retains its async DisposeCapability across Yield and Await, then
// awaits every resource before completing the active request or draining later
// requests.

function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

function sameTrace(actual, expected, label) {
  same(actual.length, expected.length, label + " length");
  for (let i = 0; i < expected.length; i++) {
    same(actual[i], expected[i], label + " " + i);
  }
}

function asyncResource(label, trace, error, gate, hook) {
  let count = 0;
  let value = { label: label };
  Object.defineProperty(value, Symbol.asyncDispose, {
    get: function () {
      trace.push(label + ":get-async");
      return async function () {
        if (this !== value) throw label + " receiver";
        count++;
        trace.push(label + ":dispose");
        if (hook !== undefined) hook();
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

let normalTrace = [];
let normalDirect = asyncResource("normal:direct", normalTrace);
let normalFallback = syncFallbackResource("normal:fallback", normalTrace);
let releaseNormal;
let normalGate = new Promise(function (resolve) {
  releaseNormal = resolve;
});
async function* normalLifecycle() {
  await using direct = normalDirect.value;
  await using fallback = normalFallback.value;
  normalTrace.push("normal:before-yield");
  yield "normal:yield";
  normalTrace.push("normal:before-await");
  await normalGate;
  normalTrace.push("normal:resume");
  return 41;
}
let normal = normalLifecycle();
sameTrace(normalTrace, [], "normal before start");

async function* returnLifecycle(trace, held) {
  await using resource = held.value;
  yield "return:yield";
  throw "return resumed normally";
}

async function* throwLifecycle(trace, held) {
  await using resource = held.value;
  yield "throw:yield";
}

async function* rejectedAwait(trace, held, error) {
  await using resource = held.value;
  yield "rejection:yield";
  await Promise.reject(error);
  trace.push("rejection:unreachable");
}

async function* acquisitionFailure(trace, registered, failing) {
  await using first = registered.value;
  await using neverRegistered = failing;
  trace.push("acquisition:unreachable");
  yield;
}

async function* nestedLifecycle(trace, outer, innerFirst, innerSecond) {
  await using outerBinding = outer.value;
  yield "nested:outer-yield";
  {
    await using innerFirstBinding = innerFirst.value;
    await using innerSecondBinding = innerSecond.value;
    trace.push("nested:inner-body");
  }
  trace.push("nested:after-inner");
  yield "nested:after-inner-yield";
}

async function* suppressedLifecycle(trace, first, second, bodyError) {
  await using firstBinding = first.value;
  await using secondBinding = second.value;
  yield "suppressed:yield";
  throw bodyError;
}

async function* queuedLifecycle(trace, held, gate) {
  await using resource = held.value;
  yield "queued:yield";
  await gate;
  trace.push("queued:resume");
}

let reentrant;
let reentrantRequest;
async function* reentrantLifecycle(trace, held) {
  await using resource = held.value;
  yield "reentrant:yield";
}

async function main() {
  // Creation evaluates nothing. The first request acquires in source order;
  // neither Yield nor the later body Await starts disposal.
  let normalYield = await normal.next();
  same(normalYield.value, "normal:yield", "normal yielded value");
  same(normalYield.done, false, "normal yielded done");
  sameTrace(
    normalTrace,
    [
      "normal:direct:get-async",
      "normal:fallback:get-async",
      "normal:fallback:get-sync",
      "normal:before-yield",
    ],
    "normal while yielded"
  );
  let normalDonePromise = normal.next();
  sameTrace(
    normalTrace,
    [
      "normal:direct:get-async",
      "normal:fallback:get-async",
      "normal:fallback:get-sync",
      "normal:before-yield",
      "normal:before-await",
    ],
    "normal while awaiting"
  );
  releaseNormal();
  let normalDone = await normalDonePromise;
  same(normalDone.value, 41, "normal return value");
  same(normalDone.done, true, "normal completed");
  sameTrace(
    normalTrace,
    [
      "normal:direct:get-async",
      "normal:fallback:get-async",
      "normal:fallback:get-sync",
      "normal:before-yield",
      "normal:before-await",
      "normal:resume",
      "normal:fallback:dispose",
      "normal:direct:dispose",
    ],
    "normal completion LIFO"
  );
  same(normalDirect.count(), 1, "normal direct once");
  same(normalFallback.count(), 1, "normal fallback once");
  same(normalFallback.thenReads(), 0, "fallback thenable ignored");
  same((await normal.next()).done, true, "normal remains closed");

  // External Return and Throw complete only after the live resource is awaited.
  let returnTrace = [];
  let returnedResource = asyncResource("return", returnTrace);
  let returned = returnLifecycle(returnTrace, returnedResource);
  same((await returned.next()).value, "return:yield", "return yielded value");
  let returnedDone = await returned.return(42);
  same(returnedDone.value, 42, "external return value");
  same(returnedDone.done, true, "external return done");
  sameTrace(
    returnTrace,
    ["return:get-async", "return:dispose"],
    "external return disposal"
  );
  same(returnedResource.count(), 1, "external return once");

  let throwTrace = [];
  let thrownResource = asyncResource("throw", throwTrace);
  let thrown = throwLifecycle(throwTrace, thrownResource);
  same((await thrown.next()).value, "throw:yield", "throw yielded value");
  let throwError = { id: "external throw" };
  let throwCaught;
  try {
    await thrown.throw(throwError);
  } catch (error) {
    throwCaught = error;
  }
  same(throwCaught, throwError, "external throw identity");
  sameTrace(
    throwTrace,
    ["throw:get-async", "throw:dispose"],
    "external throw disposal"
  );
  same(thrownResource.count(), 1, "external throw once");

  // A rejected body Await resumes as Throw and reaches the same finalizer.
  let rejectionTrace = [];
  let rejectionResource = asyncResource("rejection", rejectionTrace);
  let rejectionError = { id: "await rejection" };
  let rejected = rejectedAwait(
    rejectionTrace,
    rejectionResource,
    rejectionError
  );
  same(
    (await rejected.next()).value,
    "rejection:yield",
    "rejection yielded value"
  );
  let rejectionCaught;
  try {
    await rejected.next();
  } catch (error) {
    rejectionCaught = error;
  }
  same(rejectionCaught, rejectionError, "await rejection identity");
  sameTrace(
    rejectionTrace,
    ["rejection:get-async", "rejection:dispose"],
    "rejected await disposal"
  );
  same(rejectionResource.count(), 1, "rejected await once");

  // A later acquisition failure never registers its entry, but an earlier
  // resource is awaited before the first request rejects.
  let acquisitionTrace = [];
  let registered = asyncResource("acquisition:registered", acquisitionTrace);
  let acquisitionError = { id: "acquisition" };
  let failing = {};
  Object.defineProperty(failing, Symbol.asyncDispose, {
    get: function () {
      acquisitionTrace.push("acquisition:failing:get-async");
      throw acquisitionError;
    },
  });
  let failedAcquisition = acquisitionFailure(
    acquisitionTrace,
    registered,
    failing
  );
  let acquisitionCaught;
  try {
    await failedAcquisition.next();
  } catch (error) {
    acquisitionCaught = error;
  }
  same(acquisitionCaught, acquisitionError, "acquisition error identity");
  sameTrace(
    acquisitionTrace,
    [
      "acquisition:registered:get-async",
      "acquisition:failing:get-async",
      "acquisition:registered:dispose",
    ],
    "acquisition failure disposal"
  );
  same(registered.count(), 1, "acquisition registered once");

  // A nested scope awaits its own LIFO stack before the next yield; the outer
  // scope remains live until the following request completes the generator.
  let nestedTrace = [];
  let outer = asyncResource("nested:outer", nestedTrace);
  let innerFirst = asyncResource("nested:inner:first", nestedTrace);
  let innerSecond = syncFallbackResource("nested:inner:second", nestedTrace);
  let nested = nestedLifecycle(nestedTrace, outer, innerFirst, innerSecond);
  same(
    (await nested.next()).value,
    "nested:outer-yield",
    "nested outer yield"
  );
  let nestedAfterInner = await nested.next();
  same(
    nestedAfterInner.value,
    "nested:after-inner-yield",
    "nested after-inner yield"
  );
  sameTrace(
    nestedTrace,
    [
      "nested:outer:get-async",
      "nested:inner:first:get-async",
      "nested:inner:second:get-async",
      "nested:inner:second:get-sync",
      "nested:inner-body",
      "nested:inner:second:dispose",
      "nested:inner:first:dispose",
      "nested:after-inner",
    ],
    "nested inner disposal"
  );
  same((await nested.next()).done, true, "nested completed");
  sameTrace(
    nestedTrace,
    [
      "nested:outer:get-async",
      "nested:inner:first:get-async",
      "nested:inner:second:get-async",
      "nested:inner:second:get-sync",
      "nested:inner-body",
      "nested:inner:second:dispose",
      "nested:inner:first:dispose",
      "nested:after-inner",
      "nested:outer:dispose",
    ],
    "nested outer disposal"
  );
  same(outer.count(), 1, "nested outer once");
  same(innerFirst.count(), 1, "nested inner first once");
  same(innerSecond.count(), 1, "nested inner second once");

  // Every disposal rejection becomes the new SuppressedError.error while the
  // previous pending error remains in .suppressed.
  let suppressedTrace = [];
  let bodyError = { id: "body" };
  let firstError = { id: "first disposer" };
  let secondError = { id: "second disposer" };
  let suppressFirst = asyncResource(
    "suppressed:first",
    suppressedTrace,
    firstError
  );
  let suppressSecond = syncFallbackResource(
    "suppressed:second",
    suppressedTrace,
    secondError
  );
  let suppressed = suppressedLifecycle(
    suppressedTrace,
    suppressFirst,
    suppressSecond,
    bodyError
  );
  same(
    (await suppressed.next()).value,
    "suppressed:yield",
    "suppressed yielded value"
  );
  let combined;
  try {
    await suppressed.next();
  } catch (error) {
    combined = error;
  }
  sameTrace(
    suppressedTrace,
    [
      "suppressed:first:get-async",
      "suppressed:second:get-async",
      "suppressed:second:get-sync",
      "suppressed:second:dispose",
      "suppressed:first:dispose",
    ],
    "suppressed disposal LIFO"
  );
  same(combined instanceof SuppressedError, true, "outer SuppressedError");
  same(combined.error, firstError, "outer disposer error");
  same(
    combined.suppressed instanceof SuppressedError,
    true,
    "inner SuppressedError"
  );
  same(combined.suppressed.error, secondError, "inner disposer error");
  same(combined.suppressed.suppressed, bodyError, "suppressed body error");
  same(suppressFirst.count(), 1, "suppressed first once");
  same(suppressSecond.count(), 1, "suppressed second once");

  // Later requests queue while a body Await is pending. Disposal finishes
  // before the active request settles and before the queue drains.
  let queuedTrace = [];
  let queuedResource = asyncResource("queued", queuedTrace);
  let releaseQueued;
  let queuedGate = new Promise(function (resolve) {
    releaseQueued = resolve;
  });
  let queued = queuedLifecycle(queuedTrace, queuedResource, queuedGate);
  same((await queued.next()).value, "queued:yield", "queued yielded value");
  let queuedCurrent = queued.next().then(function (result) {
    queuedTrace.push("queued:current-settled");
    return result;
  });
  let queuedLater = queued.next().then(function (result) {
    queuedTrace.push("queued:later-settled");
    return result;
  });
  sameTrace(queuedTrace, ["queued:get-async"], "queued while awaiting");
  releaseQueued();
  same((await queuedCurrent).done, true, "queued current done");
  same((await queuedLater).done, true, "queued later done");
  sameTrace(
    queuedTrace,
    [
      "queued:get-async",
      "queued:resume",
      "queued:dispose",
      "queued:current-settled",
      "queued:later-settled",
    ],
    "disposal before settlement and drain"
  );
  same(queuedResource.count(), 1, "queued once");

  // The async disposer begins synchronously while the generator is Executing.
  // Its reentrant next() queues, then settles only after disposal completes.
  // Unlike synchronous disposal, the active request's reaction is already
  // attached while Await is pending, so it runs before the queued reaction.
  let reentrantTrace = [];
  let reentrantResource = asyncResource(
    "reentrant",
    reentrantTrace,
    undefined,
    undefined,
    function () {
      reentrantTrace.push("reentrant:enqueue");
      reentrantRequest = reentrant.next().then(function (result) {
        reentrantTrace.push("reentrant:queued-settled");
        return result;
      });
    }
  );
  reentrant = reentrantLifecycle(reentrantTrace, reentrantResource);
  same(
    (await reentrant.next()).value,
    "reentrant:yield",
    "reentrant yielded value"
  );
  let reentrantCurrent = reentrant.next().then(function (result) {
    reentrantTrace.push("reentrant:current-settled");
    return result;
  });
  same((await reentrantCurrent).done, true, "reentrant current done");
  same((await reentrantRequest).done, true, "reentrant queued done");
  sameTrace(
    reentrantTrace,
    [
      "reentrant:get-async",
      "reentrant:dispose",
      "reentrant:enqueue",
      "reentrant:current-settled",
      "reentrant:queued-settled",
    ],
    "reentrant disposal and drain"
  );
  same(reentrantResource.count(), 1, "reentrant once");

  print("await-using-async-generator:true");
}

main().then(undefined, function (error) {
  print("await-using-async-generator:FAILED:" + error);
});

0;
