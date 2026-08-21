// Consumer oracle for non-resumable synchronous `using`. Each section owns a
// fresh DisposeCapability so normal exit, acquisition failure and disposal
// error folding cannot accidentally satisfy one another.

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

function expectSuppressedDescriptor(object, key, value, label) {
  let descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (descriptor === undefined) throw label + " missing";
  same(descriptor.value, value, label + " value");
  same(descriptor.writable, true, label + " writable");
  same(descriptor.enumerable, false, label + " enumerable");
  same(descriptor.configurable, true, label + " configurable");
}

// Acquisition happens once, nullish resources register nothing, the acquired
// method is retained, `this` is the resource and normal exit is LIFO.
let normalTrace = [];
let methodReads = 0;
let first = { label: "first" };
let acquiredFirst = function () {
  if (this !== first) throw "first receiver";
  normalTrace.push("first");
};
Object.defineProperty(first, Symbol.dispose, {
  get: function () {
    methodReads++;
    return acquiredFirst;
  },
  configurable: true,
});
let second = resource("second", normalTrace);
{
  using skippedNull = null;
  using skippedUndefined = undefined;
  using firstResource = first;
  using secondResource = second;
  same(skippedNull, null, "null binding");
  same(skippedUndefined, undefined, "undefined binding");
  Object.defineProperty(first, Symbol.dispose, {
    value: function () {
      throw "late dispose method";
    },
    configurable: true,
  });
  same(methodReads, 1, "dispose method acquired once");
}
same(methodReads, 1, "dispose method read once total");
same(normalTrace.length, 2, "normal disposal count");
same(normalTrace[0], "second", "normal LIFO first");
same(normalTrace[1], "first", "normal LIFO second");

// The resource entry is the sole binding initializer. Its observable method
// acquisition happens while the binding is still in TDZ; after registration,
// the disposer observes the initialized binding before the scope is left.
let tdzResource = {};
let getterSawTdz = false;
let tdzDisposed = false;
{
  using tdzBinding = Object.defineProperty(tdzResource, Symbol.dispose, {
    get: function () {
      try {
        tdzBinding;
      } catch (error) {
        getterSawTdz = error instanceof ReferenceError;
      }
      return function () {
        if (this !== tdzResource) throw "TDZ disposer receiver";
        if (tdzBinding !== tdzResource) throw "TDZ binding not initialized";
        tdzDisposed = true;
      };
    },
  });
}
same(getterSawTdz, true, "dispose getter observes TDZ");
same(tdzDisposed, true, "TDZ resource disposed after initialization");

// A later initializer can fail only after the earlier resource was registered;
// unwinding that abrupt completion must still dispose the earlier entry.
let initializerError = { id: "initializer" };
let initializerTrace = [];
let initializerCalls = 0;
function failInitializer() {
  initializerCalls++;
  throw initializerError;
}
let initializerCaught;
try {
  {
    using acquiredBeforeFailure = resource("before initializer", initializerTrace);
    using neverInitialized = failInitializer();
  }
} catch (error) {
  initializerCaught = error;
}
same(initializerCalls, 1, "subsequent initializer once");
same(initializerCaught, initializerError, "subsequent initializer identity");
same(initializerTrace.length, 1, "dispose after initializer throw count");
same(initializerTrace[0], "before initializer", "dispose after initializer throw");

// With no incoming abrupt completion, one disposal failure is rethrown as the
// exact value rather than wrapped in SuppressedError.
let singleError = { id: "single" };
let singleTrace = [];
let singleCaught;
try {
  {
    using single = resource("single", singleTrace, singleError);
  }
} catch (error) {
  singleCaught = error;
}
same(singleCaught, singleError, "single error identity");
same(singleTrace.length, 1, "single disposer ran");

// FunctionBody owns the same pending-completion protocol as Block. A clean
// finalizer preserves Return, while one disposal failure replaces that
// non-Throw completion before the caller can observe a returned value.
let returnTrace = [];
function disposeBeforeReturn() {
  using returnedResource = resource("return", returnTrace);
  return 41;
}
same(disposeBeforeReturn(), 41, "return completion preserved");
same(returnTrace.length, 1, "dispose before return observable");
same(returnTrace[0], "return", "return disposer order");

let returnError = { id: "return disposal" };
let returnErrorTrace = [];
function disposalReplacesReturn() {
  using returnedResource = resource("return throw", returnErrorTrace, returnError);
  return 42;
}
let replacedReturn;
try {
  replacedReturn = disposalReplacesReturn();
} catch (error) {
  replacedReturn = error;
}
same(replacedReturn, returnError, "disposal replaces return");
same(returnErrorTrace.length, 1, "return throwing disposer ran");

// Every disposer runs despite throws. Reverse registration order determines
// call order; each later disposal failure wraps the accumulated completion, so
// the earliest registered failure is outermost over the original body error.
let bodyError = { id: "body" };
let firstError = { id: "first error" };
let secondError = { id: "second error" };
let thirdError = { id: "third error" };
let errorTrace = [];
let combined;
try {
  {
    using firstThrowing = resource("first throwing", errorTrace, firstError);
    using secondThrowing = resource("second throwing", errorTrace, secondError);
    using thirdThrowing = resource("third throwing", errorTrace, thirdError);
    throw bodyError;
  }
} catch (error) {
  combined = error;
}
same(errorTrace.length, 3, "all disposers continue");
same(errorTrace[0], "third throwing", "throwing LIFO first");
same(errorTrace[1], "second throwing", "throwing LIFO second");
same(errorTrace[2], "first throwing", "throwing LIFO third");
if (!(combined instanceof SuppressedError)) throw "outer SuppressedError";
expectSuppressedDescriptor(combined, "error", firstError, "outer error");
if (!(combined.suppressed instanceof SuppressedError)) throw "middle SuppressedError";
expectSuppressedDescriptor(combined, "suppressed", combined.suppressed, "outer suppressed");
expectSuppressedDescriptor(combined.suppressed, "error", secondError, "middle error");
if (!(combined.suppressed.suppressed instanceof SuppressedError)) {
  throw "inner SuppressedError";
}
expectSuppressedDescriptor(
  combined.suppressed,
  "suppressed",
  combined.suppressed.suppressed,
  "middle suppressed",
);
expectSuppressedDescriptor(combined.suppressed.suppressed, "error", thirdError, "inner error");
expectSuppressedDescriptor(
  combined.suppressed.suppressed,
  "suppressed",
  bodyError,
  "inner suppressed",
);
if (Object.getOwnPropertyDescriptor(combined, "message") !== undefined) {
  throw "outer suppression message";
}
if (Object.getOwnPropertyDescriptor(combined.suppressed, "message") !== undefined) {
  throw "middle suppression message";
}
if (Object.getOwnPropertyDescriptor(combined.suppressed.suppressed, "message") !== undefined) {
  throw "inner suppression message";
}

true;
