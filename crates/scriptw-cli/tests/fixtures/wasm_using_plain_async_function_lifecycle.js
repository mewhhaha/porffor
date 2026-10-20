// Consumer oracle for synchronous `using` owned by a plain async function.
// Each reached scope owns activation-backed disposal state: suspension retains
// it, while every terminal completion disposes before settling the Promise.

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
  Object.defineProperty(value, Symbol.dispose, {
    get: function () {
      trace.push(label + ":acquire");
      return function () {
        if (this !== value) throw label + " receiver";
        trace.push(label + ":dispose");
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
async function normalLifecycle() {
  using first = resource("normal:first", normalTrace);
  using second = resource("normal:second", normalTrace);
  normalTrace.push("normal:suspend");
  await normalGate;
  normalTrace.push("normal:resume");
}
sameTrace(normalTrace, [], "normal before call");

async function explicitReturn(trace) {
  using held = resource("return", trace);
  await 0;
  trace.push("return:body");
  return 42;
}

async function sourceThrow(trace, error) {
  using held = resource("throw", trace);
  await 0;
  trace.push("throw:body");
  throw error;
}

async function rejectedAwait(trace, error) {
  using held = resource("rejection", trace);
  await Promise.reject(error);
  trace.push("rejection:unreachable");
}

async function acquisitionFailure(trace, failing) {
  using registered = resource("acquisition:registered", trace);
  using neverRegistered = failing;
  trace.push("acquisition:unreachable");
  await 0;
}

async function nestedLifecycle(trace) {
  using outer = resource("nested:outer", trace);
  await 0;
  {
    using innerFirst = resource("nested:inner:first", trace);
    using innerSecond = resource("nested:inner:second", trace);
    trace.push("nested:inner:body");
  }
  trace.push("nested:outer:body");
}

async function suppressedLifecycle(trace, bodyError, firstError, secondError) {
  using first = resource("suppressed:first", trace, firstError);
  using second = resource("suppressed:second", trace, secondError);
  await 0;
  throw bodyError;
}

async function main() {
  // Calling reaches acquisition in source order, but the first await retains
  // both records without disposing them.
  let normalPromise = normalLifecycle();
  sameTrace(
    normalTrace,
    ["normal:first:acquire", "normal:second:acquire", "normal:suspend"],
    "normal while suspended"
  );
  releaseNormal();
  let normalResult = await normalPromise;
  same(normalResult, undefined, "normal result");
  sameTrace(
    normalTrace,
    [
      "normal:first:acquire",
      "normal:second:acquire",
      "normal:suspend",
      "normal:resume",
      "normal:second:dispose",
      "normal:first:dispose",
    ],
    "normal completion LIFO"
  );

  // Explicit Return is settled only after disposal.
  let returnTrace = [];
  let returnResult = await explicitReturn(returnTrace);
  same(returnResult, 42, "return result");
  sameTrace(
    returnTrace,
    ["return:acquire", "return:body", "return:dispose"],
    "return disposal before resolution"
  );

  // Source Throw retains exact error identity, after disposal.
  let throwError = { id: "source throw" };
  let throwTrace = [];
  let throwCaught;
  try {
    await sourceThrow(throwTrace, throwError);
  } catch (error) {
    throwCaught = error;
  }
  same(throwCaught, throwError, "source throw identity");
  sameTrace(
    throwTrace,
    ["throw:acquire", "throw:body", "throw:dispose"],
    "source throw disposal before rejection"
  );

  // A rejected await resumes as Throw and reaches the same scope exit.
  let rejectionError = { id: "await rejection" };
  let rejectionTrace = [];
  let rejectionCaught;
  try {
    await rejectedAwait(rejectionTrace, rejectionError);
  } catch (error) {
    rejectionCaught = error;
  }
  same(rejectionCaught, rejectionError, "await rejection identity");
  sameTrace(
    rejectionTrace,
    ["rejection:acquire", "rejection:dispose"],
    "rejected await disposal"
  );

  // If a later GetMethod throws, it is never registered. Every prior record
  // is still disposed before the async function rejects.
  let acquisitionError = { id: "acquisition" };
  let acquisitionTrace = [];
  let failing = {};
  Object.defineProperty(failing, Symbol.dispose, {
    get: function () {
      acquisitionTrace.push("acquisition:failing:acquire");
      throw acquisitionError;
    },
  });
  let acquisitionCaught;
  try {
    await acquisitionFailure(acquisitionTrace, failing);
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

  // A nested non-await scope owns a distinct capability and disposes before
  // execution returns to the outer async-function-body scope.
  let nestedTrace = [];
  await nestedLifecycle(nestedTrace);
  sameTrace(
    nestedTrace,
    [
      "nested:outer:acquire",
      "nested:inner:first:acquire",
      "nested:inner:second:acquire",
      "nested:inner:body",
      "nested:inner:second:dispose",
      "nested:inner:first:dispose",
      "nested:outer:body",
      "nested:outer:dispose",
    ],
    "nested scopes"
  );

  // Disposal continues after every throw. Each later error is folded over the
  // pending Throw completion in reverse registration order.
  let bodyError = { id: "body" };
  let firstError = { id: "first disposer" };
  let secondError = { id: "second disposer" };
  let suppressedTrace = [];
  let combined;
  try {
    await suppressedLifecycle(
      suppressedTrace,
      bodyError,
      firstError,
      secondError
    );
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

  // Settled async functions cannot consume their capability a second time.
  await 0;
  same(normalTrace.length, 6, "normal exactly once");
  same(returnTrace.length, 3, "return exactly once");
  same(throwTrace.length, 3, "throw exactly once");
  same(rejectionTrace.length, 2, "rejection exactly once");
  same(acquisitionTrace.length, 3, "acquisition exactly once");
  same(nestedTrace.length, 8, "nested exactly once");
  same(suppressedTrace.length, 4, "suppressed exactly once");

  print("using-plain-async-function:true");
}

main().then(undefined, function (error) {
  print("using-plain-async-function:FAILED:" + error);
});

0;
