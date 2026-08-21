// Consumer oracle for synchronous `using` owned by an async generator. Every
// reached scope retains one activation-backed capability across both Yield and
// Await, then disposes before completing the current request or draining later
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

function resource(label, trace, error, hook) {
  let value = { label: label };
  Object.defineProperty(value, Symbol.dispose, {
    get: function () {
      trace.push(label + ":acquire");
      return function () {
        if (this !== value) throw label + " receiver";
        trace.push(label + ":dispose");
        if (hook !== undefined) hook();
        if (error !== undefined) throw error;
      };
    },
  });
  return value;
}

let normalTrace = [];
let releaseNormal;
let normalGate = new Promise(function (resolve) {
  releaseNormal = resolve;
});
async function* normalLifecycle() {
  using first = resource("normal:first", normalTrace);
  using second = resource("normal:second", normalTrace);
  normalTrace.push("normal:before-yield");
  yield "normal:yield";
  normalTrace.push("normal:before-await");
  await normalGate;
  normalTrace.push("normal:resume");
  return 41;
}
let normal = normalLifecycle();
sameTrace(normalTrace, [], "normal before start");

async function* returnLifecycle(trace) {
  using held = resource("return", trace);
  yield "return:yield";
  throw "return resumed normally";
}

async function* throwLifecycle(trace) {
  using held = resource("throw", trace);
  yield "throw:yield";
}

async function* rejectedAwait(trace, error) {
  using held = resource("rejection", trace);
  yield "rejection:yield";
  await Promise.reject(error);
  trace.push("rejection:unreachable");
}

async function* acquisitionFailure(trace, failing) {
  using registered = resource("acquisition:registered", trace);
  using neverRegistered = failing;
  trace.push("acquisition:unreachable");
  yield;
}

async function* nestedLifecycle(trace) {
  using outer = resource("nested:outer", trace);
  yield "nested:outer-yield";
  {
    using innerFirst = resource("nested:inner:first", trace);
    using innerSecond = resource("nested:inner:second", trace);
    trace.push("nested:inner:body");
  }
  trace.push("nested:after-inner");
  yield "nested:after-inner-yield";
}

async function* suppressedLifecycle(trace, bodyError, firstError, secondError) {
  using first = resource("suppressed:first", trace, firstError);
  using second = resource("suppressed:second", trace, secondError);
  yield "suppressed:yield";
  throw bodyError;
}

async function* queuedLifecycle(trace, gate) {
  using held = resource("queued", trace);
  yield "queued:yield";
  await gate;
  trace.push("queued:resume");
}

let reentrant;
let reentrantRequest;
async function* reentrantLifecycle(trace) {
  using held = resource("reentrant", trace, undefined, function () {
    trace.push("reentrant:enqueue");
    reentrantRequest = reentrant.next().then(function (result) {
      trace.push("reentrant:queued-settled");
      return result;
    });
  });
  yield "reentrant:yield";
}

async function main() {
  // Creation evaluates nothing. The first request acquires in source order,
  // Yield retains both records, and the later Await also retains them.
  let normalYield = await normal.next();
  same(normalYield.value, "normal:yield", "normal yielded value");
  same(normalYield.done, false, "normal yielded done");
  sameTrace(
    normalTrace,
    ["normal:first:acquire", "normal:second:acquire", "normal:before-yield"],
    "normal while yielded"
  );
  let normalDonePromise = normal.next();
  sameTrace(
    normalTrace,
    [
      "normal:first:acquire",
      "normal:second:acquire",
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
      "normal:first:acquire",
      "normal:second:acquire",
      "normal:before-yield",
      "normal:before-await",
      "normal:resume",
      "normal:second:dispose",
      "normal:first:dispose",
    ],
    "normal completion LIFO"
  );
  same((await normal.next()).done, true, "normal remains closed");

  // An external Return completion disposes before its iterator result settles.
  let returnTrace = [];
  let returned = returnLifecycle(returnTrace);
  same((await returned.next()).value, "return:yield", "return yielded value");
  let returnedDone = await returned.return(42);
  same(returnedDone.value, 42, "external return value");
  same(returnedDone.done, true, "external return done");
  sameTrace(
    returnTrace,
    ["return:acquire", "return:dispose"],
    "external return disposal"
  );
  same((await returned.next()).done, true, "returned remains closed");

  // An external Throw preserves exact identity after disposing the live scope.
  let throwError = { id: "external throw" };
  let throwTrace = [];
  let thrown = throwLifecycle(throwTrace);
  same((await thrown.next()).value, "throw:yield", "throw yielded value");
  let throwCaught;
  try {
    await thrown.throw(throwError);
  } catch (error) {
    throwCaught = error;
  }
  same(throwCaught, throwError, "external throw identity");
  sameTrace(
    throwTrace,
    ["throw:acquire", "throw:dispose"],
    "external throw disposal"
  );
  same((await thrown.next()).done, true, "thrown remains closed");

  // A rejected Await resumes as Throw and reaches the same disposal frame.
  let rejectionError = { id: "await rejection" };
  let rejectionTrace = [];
  let rejected = rejectedAwait(rejectionTrace, rejectionError);
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
    ["rejection:acquire", "rejection:dispose"],
    "rejected await disposal"
  );

  // If a later GetMethod throws, it was never registered; every earlier entry
  // is still disposed before the first request rejects.
  let acquisitionError = { id: "acquisition" };
  let acquisitionTrace = [];
  let failing = {};
  Object.defineProperty(failing, Symbol.dispose, {
    get: function () {
      acquisitionTrace.push("acquisition:failing:acquire");
      throw acquisitionError;
    },
  });
  let failedAcquisition = acquisitionFailure(acquisitionTrace, failing);
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
      "acquisition:registered:acquire",
      "acquisition:failing:acquire",
      "acquisition:registered:dispose",
    ],
    "acquisition failure disposal"
  );
  same(
    (await failedAcquisition.next()).done,
    true,
    "acquisition remains closed"
  );

  // A nested non-suspending scope owns a distinct capability and disposes
  // before the next yielded result; the outer scope remains live.
  let nestedTrace = [];
  let nested = nestedLifecycle(nestedTrace);
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
      "nested:outer:acquire",
      "nested:inner:first:acquire",
      "nested:inner:second:acquire",
      "nested:inner:body",
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
      "nested:outer:acquire",
      "nested:inner:first:acquire",
      "nested:inner:second:acquire",
      "nested:inner:body",
      "nested:inner:second:dispose",
      "nested:inner:first:dispose",
      "nested:after-inner",
      "nested:outer:dispose",
    ],
    "nested outer disposal"
  );

  // LIFO disposal continues after every throw and folds each later error over
  // the pending body Throw completion.
  let bodyError = { id: "body" };
  let firstError = { id: "first disposer" };
  let secondError = { id: "second disposer" };
  let suppressedTrace = [];
  let suppressed = suppressedLifecycle(
    suppressedTrace,
    bodyError,
    firstError,
    secondError
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
      "suppressed:first:acquire",
      "suppressed:second:acquire",
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
  same((await suppressed.next()).done, true, "suppressed remains closed");

  // A later request may queue while the current request is suspended at Await.
  // Disposal precedes both current-request settlement and queue drain.
  let queuedTrace = [];
  let releaseQueued;
  let queuedGate = new Promise(function (resolve) {
    releaseQueued = resolve;
  });
  let queued = queuedLifecycle(queuedTrace, queuedGate);
  same((await queued.next()).value, "queued:yield", "queued yielded value");
  let queuedCurrent = queued.next().then(function (result) {
    queuedTrace.push("queued:current-settled");
    return result;
  });
  let queuedLater = queued.next().then(function (result) {
    queuedTrace.push("queued:later-settled");
    return result;
  });
  sameTrace(queuedTrace, ["queued:acquire"], "queued while awaiting");
  releaseQueued();
  same((await queuedCurrent).done, true, "queued current done");
  same((await queuedLater).done, true, "queued later done");
  sameTrace(
    queuedTrace,
    [
      "queued:acquire",
      "queued:resume",
      "queued:dispose",
      "queued:current-settled",
      "queued:later-settled",
    ],
    "disposal before settlement and drain"
  );

  // A disposer may synchronously enqueue a request against the generator that
  // is still Executing. Disposal finishes before queue drain resolves the
  // reentrant request and before the current request's reaction runs.
  let reentrantTrace = [];
  reentrant = reentrantLifecycle(reentrantTrace);
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
      "reentrant:acquire",
      "reentrant:dispose",
      "reentrant:enqueue",
      "reentrant:queued-settled",
      "reentrant:current-settled",
    ],
    "reentrant disposal and drain"
  );

  await 0;
  same(normalTrace.length, 7, "normal exactly once");
  same(returnTrace.length, 2, "return exactly once");
  same(throwTrace.length, 2, "throw exactly once");
  same(rejectionTrace.length, 2, "rejection exactly once");
  same(acquisitionTrace.length, 3, "acquisition exactly once");
  same(nestedTrace.length, 8, "nested exactly once");
  same(suppressedTrace.length, 4, "suppressed exactly once");
  same(queuedTrace.length, 5, "queued exactly once");
  same(reentrantTrace.length, 5, "reentrant exactly once");

  print("using-async-generator:true");
}

main().then(undefined, function (error) {
  print("using-async-generator:FAILED:" + error);
});

0;
