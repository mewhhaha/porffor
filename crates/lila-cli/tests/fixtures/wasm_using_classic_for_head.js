// Consumer oracle for synchronous `using` in a classic `for` head. The loop
// owns one DisposeCapability across initialization, test, body and update.

function same(actual, expected, label) {
  if (actual !== expected) throw label;
}

function resource(label, trace, error) {
  let value = { label: label };
  value[Symbol.dispose] = function () {
    if (this !== value) throw label + " receiver";
    trace.push(label);
    if (error !== undefined) throw error;
  };
  return value;
}

// A labelled continue stays inside the capability. A labelled break consumes
// it once, after both iterations and their update boundary.
let labelledTrace = [];
let iterations = 0;
outer: for (
  using held = resource("labelled", labelledTrace);
  iterations < 3;
  iterations++
) {
  same(labelledTrace.length, 0, "disposed during body");
  if (iterations === 0) continue outer;
  break outer;
}
same(iterations, 1, "labelled continue reached update");
same(labelledTrace.length, 1, "labelled exit disposal count");
same(labelledTrace[0], "labelled", "labelled exit disposal");

// In a classic head, `using of` is an ordinary binding rather than the
// restricted `for-of` lookahead. Nullish acquisition registers no entry.
for (using of = null; ; ) break;

// The head owns a fresh binding environment without changing the surrounding
// using binding. Its resource is disposed before the outer binding is exposed
// again, and the outer resource remains live until its own scope exits.
let shadowTrace = [];
{
  using shadowed = resource("outer", shadowTrace);
  for (using shadowed = resource("inner", shadowTrace); false; ) {}
  same(shadowed.label, "outer", "outer binding restored");
  same(shadowTrace.length, 1, "inner disposed before outer");
  same(shadowTrace[0], "inner", "inner shadow disposal");
}
same(shadowTrace.length, 2, "shadow disposal count");
same(shadowTrace[1], "outer", "outer shadow disposal");

// Every head binding is created uninitialized before the first acquisition.
// Defining this getter inside the initializer makes it close over the for-head
// environment: reading the later binding must hit TDZ, never the outer name.
let later = "outer";
let laterGetterSawTdz = false;
let laterBindingDisposed = false;
let tdzResource = {};
for (
  using first = Object.defineProperty(tdzResource, Symbol.dispose, {
      get: function () {
        try {
          later;
        } catch (error) {
          laterGetterSawTdz = error instanceof ReferenceError;
        }
        return function () {
          if (this !== tdzResource) throw "later TDZ receiver";
          same(later, null, "later initialized before disposal");
          laterBindingDisposed = true;
        };
      },
    }),
    later = null;
  false;
) {}
same(laterGetterSawTdz, true, "later binding getter observes TDZ");
same(laterBindingDisposed, true, "later binding disposed");
same(later, "outer", "later outer binding unchanged");

// A false first test still exits through the capability. Entries registered in
// one head dispose in reverse source order.
let normalTrace = [];
for (
  using first = resource("first", normalTrace),
    second = resource("second", normalTrace);
  false;
) {}
same(normalTrace.length, 2, "normal disposal count");
same(normalTrace[0], "second", "normal LIFO first");
same(normalTrace[1], "first", "normal LIFO second");

// Failure in a later initializer disposes every earlier registered entry and
// otherwise preserves the initializer's exact thrown value.
let initializerError = { id: "initializer" };
let initializerTrace = [];
let initializerCalls = 0;
function failInitializer() {
  initializerCalls++;
  throw initializerError;
}
let initializerCaught;
try {
  for (
    using acquired = resource("before initializer", initializerTrace),
      neverInitialized = failInitializer();
    false;
  ) {}
} catch (error) {
  initializerCaught = error;
}
same(initializerCalls, 1, "subsequent initializer once");
same(initializerCaught, initializerError, "initializer error identity");
same(initializerTrace.length, 1, "dispose after initializer failure count");
same(initializerTrace[0], "before initializer", "dispose after initializer failure");

// A disposer failure folds over the abrupt initializer completion in the same
// order as an ordinary synchronous using scope.
let disposerError = { id: "disposer" };
let suppressedInitializerError = { id: "suppressed initializer" };
let suppressionTrace = [];
function failSuppressedInitializer() {
  throw suppressedInitializerError;
}
let combined;
try {
  for (
    using throwing = resource("throwing", suppressionTrace, disposerError),
      neverInitialized = failSuppressedInitializer();
    false;
  ) {}
} catch (error) {
  combined = error;
}
same(suppressionTrace.length, 1, "suppression disposer count");
if (!(combined instanceof SuppressedError)) throw "initializer SuppressedError";
same(combined.error, disposerError, "suppression error order");
same(combined.suppressed, suppressedInitializerError, "suppression pending order");

// The using binding is immutable. Assignment in the update expression throws
// while the capability is active, then the registered resource is disposed.
let updateTrace = [];
let updateCaught;
try {
  for (
    using held = resource("update", updateTrace);
    held.label === "update";
    held = null
  ) {
    same(held.label, "update", "using binding in body");
    same(updateTrace.length, 0, "disposed before update");
  }
} catch (error) {
  updateCaught = error;
}
same(updateCaught instanceof TypeError, true, "immutable update TypeError");
same(updateTrace.length, 1, "dispose after update failure count");
same(updateTrace[0], "update", "dispose after update failure");

true;
