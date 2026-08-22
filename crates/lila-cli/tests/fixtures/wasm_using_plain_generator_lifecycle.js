// Consumer oracle for synchronous `using` owned by a plain generator. Each
// generator owns a fresh disposal capability so suspension and terminal
// completion paths cannot accidentally satisfy one another.

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

// Calling the generator allocates no resource. The first resume acquires both
// methods in source order, and suspension retains the capability without
// disposing it. Normal completion consumes it once in LIFO order.
let normalTrace = [];
function* normalLifecycle() {
  using first = resource("normal:first", normalTrace);
  using second = resource("normal:second", normalTrace);
  yield "normal:yield";
  normalTrace.push("normal:body-end");
  return 41;
}
let normal = normalLifecycle();
sameTrace(normalTrace, [], "normal before start");
let normalYield = normal.next();
same(normalYield.value, "normal:yield", "normal yielded value");
same(normalYield.done, false, "normal yielded done");
sameTrace(
  normalTrace,
  ["normal:first:acquire", "normal:second:acquire"],
  "normal while suspended"
);
let normalDone = normal.next();
same(normalDone.value, 41, "normal return value");
same(normalDone.done, true, "normal completed");
sameTrace(
  normalTrace,
  [
    "normal:first:acquire",
    "normal:second:acquire",
    "normal:body-end",
    "normal:second:dispose",
    "normal:first:dispose",
  ],
  "normal completion LIFO"
);
same(normal.next().done, true, "normal remains closed");
same(normal.return(99).done, true, "normal return after close");
same(normalTrace.length, 5, "normal exactly once");

// An injected Return completion exits the suspended using scope, disposes
// before publishing the iterator result and remains terminal.
let returnTrace = [];
function* returnLifecycle() {
  using held = resource("return", returnTrace);
  yield "return:yield";
  throw "return resumed normally";
}
let returned = returnLifecycle();
sameTrace(returnTrace, [], "return before start");
same(returned.next().value, "return:yield", "return yielded value");
sameTrace(returnTrace, ["return:acquire"], "return while suspended");
let returnedDone = returned.return(42);
same(returnedDone.value, 42, "injected return value");
same(returnedDone.done, true, "injected return done");
sameTrace(
  returnTrace,
  ["return:acquire", "return:dispose"],
  "return disposal"
);
same(returned.next().done, true, "returned generator closed");
same(returnTrace.length, 2, "return exactly once");

// An injected Throw completion likewise disposes before the exact thrown value
// escapes. A later resume cannot repeat disposal.
let injectedError = { id: "injected" };
let throwTrace = [];
function* throwLifecycle() {
  using held = resource("throw", throwTrace);
  yield "throw:yield";
}
let thrown = throwLifecycle();
same(thrown.next().value, "throw:yield", "throw yielded value");
let injectedCaught;
try {
  thrown.throw(injectedError);
} catch (error) {
  injectedCaught = error;
}
same(injectedCaught, injectedError, "injected throw identity");
sameTrace(throwTrace, ["throw:acquire", "throw:dispose"], "throw disposal");
same(thrown.next().done, true, "thrown generator closed");
same(throwTrace.length, 2, "throw exactly once");

// If a later resource's GetMethod fails, the earlier registered resource is
// disposed while the generator transitions directly from suspended-start to
// completed. The body is never reached.
let acquisitionError = { id: "acquisition" };
let acquisitionTrace = [];
let failingResource = {};
Object.defineProperty(failingResource, Symbol.dispose, {
  get: function () {
    acquisitionTrace.push("failing:acquire");
    throw acquisitionError;
  },
});
function* acquisitionFailure() {
  using registered = resource("registered", acquisitionTrace);
  using neverRegistered = failingResource;
  acquisitionTrace.push("acquisition:body");
  yield;
}
let failedAcquisition = acquisitionFailure();
sameTrace(acquisitionTrace, [], "acquisition before start");
let acquisitionCaught;
try {
  failedAcquisition.next();
} catch (error) {
  acquisitionCaught = error;
}
same(acquisitionCaught, acquisitionError, "acquisition error identity");
sameTrace(
  acquisitionTrace,
  ["registered:acquire", "failing:acquire", "registered:dispose"],
  "acquisition failure disposal"
);
same(failedAcquisition.next().done, true, "acquisition generator closed");
same(acquisitionTrace.length, 3, "acquisition exactly once");

// A nested scope has its own capability. Leaving it after resumption disposes
// only its resources before the next linear suspension; the generator-body
// resource remains live until the generator itself completes.
let nestedTrace = [];
function* nestedLifecycle() {
  using outer = resource("nested:outer", nestedTrace);
  yield "nested:outer";
  {
    using innerFirst = resource("nested:inner:first", nestedTrace);
    using innerSecond = resource("nested:inner:second", nestedTrace);
  }
  yield "nested:after-inner";
}
let nested = nestedLifecycle();
same(nested.next().value, "nested:outer", "nested outer yield");
sameTrace(nestedTrace, ["nested:outer:acquire"], "nested outer suspended");
same(nested.next().value, "nested:after-inner", "nested after inner yield");
sameTrace(
  nestedTrace,
  [
    "nested:outer:acquire",
    "nested:inner:first:acquire",
    "nested:inner:second:acquire",
    "nested:inner:second:dispose",
    "nested:inner:first:dispose",
  ],
  "nested scope LIFO before outer completion"
);
same(nested.next().done, true, "nested completed");
sameTrace(
  nestedTrace,
  [
    "nested:outer:acquire",
    "nested:inner:first:acquire",
    "nested:inner:second:acquire",
    "nested:inner:second:dispose",
    "nested:inner:first:dispose",
    "nested:outer:dispose",
  ],
  "nested outer disposal"
);
same(nested.next().done, true, "nested remains closed");
same(nestedTrace.length, 6, "nested exactly once");

// Disposal continues after every throw. Reverse registration order folds each
// new disposer error over the pending body Throw completion.
let bodyError = { id: "body" };
let firstError = { id: "first disposer" };
let secondError = { id: "second disposer" };
let suppressedTrace = [];
function* suppressedLifecycle() {
  using first = resource("suppressed:first", suppressedTrace, firstError);
  using second = resource("suppressed:second", suppressedTrace, secondError);
  yield "suppressed:yield";
  throw bodyError;
}
let suppressedGenerator = suppressedLifecycle();
same(
  suppressedGenerator.next().value,
  "suppressed:yield",
  "suppressed yielded value"
);
let combined;
try {
  suppressedGenerator.next();
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
same(suppressedGenerator.next().done, true, "suppressed generator closed");
same(suppressedTrace.length, 4, "suppressed exactly once");

true;
