function check(value, message) { if (!value) throw message; }
var pending = [];
function checkAwaitTrace(stack, expected) {
  var order = ["before"];
  var disposed = stack.disposeAsync();
  order.push("after");
  var ticks = Promise.resolve().then(function () { order.push("tick1"); }).then(function () { order.push("tick2"); });
  var finished = disposed.then(function () { order.push("done"); });
  return Promise.all([ticks, finished]).then(function () { check(order.join(",") === expected, "one required Await: " + order); });
}
var empty = new AsyncDisposableStack();
empty.use(null);
empty.use(undefined);
pending.push(checkAwaitTrace(empty, "before,after,tick1,done,tick2"));
var successful = new AsyncDisposableStack();
var successfulCalls = 0;
successful.defer(function () { successfulCalls++; });
successful.use(null);
successful.use(undefined);
pending.push(checkAwaitTrace(successful, "before,after,tick1,done,tick2").then(function () {
  check(successfulCalls === 1, "no extra final Await after method Await");
}));
var first = Symbol("first");
var second = function () {};
var third = {};
var rejected = new AsyncDisposableStack();
var rejectedCalls = [];
rejected.defer(function () { rejectedCalls.push(1); return Promise.reject(first); });
rejected.defer(function () { rejectedCalls.push(2); throw second; });
rejected.defer(function () { rejectedCalls.push(3); return Promise.reject(third); });
pending.push(rejected.disposeAsync().then(function () { throw "unexpected fulfillment"; }, function (error) {
  check(rejectedCalls.join(",") === "3,2,1", "continue after all abrupt kinds");
  check(error instanceof SuppressedError && error.error === first, "latest error wraps prior error");
  check(error.suppressed instanceof SuppressedError && error.suppressed.error === second && error.suppressed.suppressed === third, "whole nested suppression values");
}));
var syncFirst = {};
var syncSecond = {};
var throwing = new AsyncDisposableStack();
var throwingOrder = [];
throwing.defer(function () { throwingOrder.push("lower"); throw syncFirst; });
throwing.defer(function () { throwingOrder.push("upper"); throw syncSecond; });
throwing.use(null);
throwing.use(undefined);
var throwingResult = throwing.disposeAsync();
throwingOrder.push("after");
check(throwingOrder.join(",") === "upper,lower,after", "synchronous Call throws do not suspend");
var throwingTicks = Promise.resolve().then(function () { throwingOrder.push("tick1"); }).then(function () { throwingOrder.push("tick2"); });
var throwingFinished = throwingResult.then(function () { throw "unexpected fulfillment"; }, function (error) {
  check(error.error === syncFirst && error.suppressed === syncSecond, "synchronous suppression order");
  throwingOrder.push("done");
});
pending.push(Promise.all([throwingTicks, throwingFinished]).then(function () {
  check(throwingOrder.join(",") === "upper,lower,after,tick1,done,tick2", "Empty plus synchronous throws still awaits once");
}));
var setupMarker = {};
var setupOrder = [];
var setup = new AsyncDisposableStack();
setup.defer(function () { setupOrder.push("lower"); });
var poisoned = Promise.resolve(1);
Object.defineProperty(poisoned, "constructor", { get: function () { setupOrder.push("constructor"); throw setupMarker; } });
setup.defer(function () { setupOrder.push("upper"); return poisoned; });
var setupResult = setup.disposeAsync();
setupOrder.push("after");
check(setupOrder.join(",") === "upper,constructor,lower,after", "Await setup Throw resumes synchronously");
pending.push(setupResult.then(function () { throw "setup failure fulfilled"; }, function (error) {
  check(error === setupMarker, "Await setup original abrupt identity");
}));
var thenReads = 0;
var fallback = new AsyncDisposableStack();
fallback.use({ [Symbol.dispose]: new Proxy(function () {}, { apply: function () {
  return { get then() { thenReads++; throw "sync return observed"; } };
} }) });
pending.push(fallback.disposeAsync().then(function () { check(thenReads === 0, "sync fallback discards result"); }));
Promise.all(pending).then(function () { print("async-stack-await:ok"); });
262;
