// Consumer-level oracle for the complete synchronous %DisposableStack%
// lifecycle. The fixture deliberately mixes all three resource-record kinds:
// the contract is their shared ordering, ownership and error-folding behavior,
// not three independent happy-path examples.

function capture(thunk) {
  try {
    thunk();
  } catch (error) {
    return error;
  }
  throw "expected throw";
}

function expectThrows(kind, thunk, label) {
  let error = capture(thunk);
  if (!(error instanceof kind)) throw label;
  return error;
}

function expectDataDescriptor(object, key, writable, enumerable, configurable, label) {
  let descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (descriptor === undefined) throw label + " missing";
  if (descriptor.writable !== writable) throw label + " writable";
  if (descriptor.enumerable !== enumerable) throw label + " enumerable";
  if (descriptor.configurable !== configurable) throw label + " configurable";
}

// --- intrinsic surface and exact alias identity ----------------------------

let prototype = DisposableStack.prototype;
let methodNames = ["use", "adopt", "defer", "move", "dispose"];
let methodLengths = [1, 2, 1, 0, 0];

for (let i = 0; i < methodNames.length; i++) {
  let name = methodNames[i];
  let method = prototype[name];
  if (typeof method !== "function") throw "method missing " + name;
  if (method.name !== name) throw "method name " + name;
  if (method.length !== methodLengths[i]) throw "method length " + name;
  expectDataDescriptor(prototype, name, true, false, true, "method " + name);
  expectThrows(TypeError, function () {
    new method();
  }, "constructible method " + name);
}

expectDataDescriptor(prototype, Symbol.dispose, true, false, true, "Symbol.dispose");
if (prototype[Symbol.dispose] !== prototype.dispose) throw "Symbol.dispose identity";

let disposedDescriptor = Object.getOwnPropertyDescriptor(prototype, "disposed");
if (disposedDescriptor === undefined) throw "disposed descriptor";
if (typeof disposedDescriptor.get !== "function") throw "disposed getter";
if (disposedDescriptor.get.name !== "get disposed") throw "disposed getter name";
if (disposedDescriptor.get.length !== 0) throw "disposed getter length";
if (disposedDescriptor.set !== undefined) throw "disposed setter";
if (disposedDescriptor.enumerable !== false) throw "disposed enumerable";
if (disposedDescriptor.configurable !== true) throw "disposed configurable";
expectThrows(TypeError, function () {
  new disposedDescriptor.get();
}, "constructible disposed getter");

// Every member consumes the same distinct synchronous brand. A lookalike,
// either intrinsic object and the real async brand must all be rejected.
let wrongReceivers = [
  {},
  { "[[DisposableState]]": "pending" },
  prototype,
  DisposableStack,
  1,
  new AsyncDisposableStack(),
];
for (let i = 0; i < wrongReceivers.length; i++) {
  let receiver = wrongReceivers[i];
  for (let j = 0; j < methodNames.length; j++) {
    expectThrows(TypeError, function () {
      prototype[methodNames[j]].call(receiver);
    }, "method brand " + i + ":" + methodNames[j]);
  }
  expectThrows(TypeError, function () {
    disposedDescriptor.get.call(receiver);
  }, "getter brand " + i);
}

// --- registration, acquired method and the three call conventions ---------

let stack = new DisposableStack();
if (stack.disposed !== false) throw "fresh pending";
if (stack.use(null) !== null) throw "use null return";
if (stack.use(undefined) !== undefined) throw "use undefined return";

expectThrows(TypeError, function () {
  stack.use(1);
}, "use primitive");
expectThrows(TypeError, function () {
  stack.use({});
}, "use missing method");
let nonCallable = {};
nonCallable[Symbol.dispose] = 1;
expectThrows(TypeError, function () {
  stack.use(nonCallable);
}, "use non-callable method");
expectThrows(TypeError, function () {
  stack.adopt("not appended", {});
}, "adopt non-callable callback");
expectThrows(TypeError, function () {
  stack.defer({});
}, "defer non-callable callback");

let calls = [];
let methodReads = 0;
let resource = { label: "resource" };
let acquired = function () {
  if (this !== resource) throw "use receiver";
  if (arguments.length !== 0) throw "use arguments";
  calls.push("use");
};
Object.defineProperty(resource, Symbol.dispose, {
  get: function () {
    methodReads++;
    return acquired;
  },
  configurable: true,
});
if (stack.use(resource) !== resource) throw "use return";
Object.defineProperty(resource, Symbol.dispose, {
  value: function () {
    throw "late disposer replacement";
  },
  writable: true,
  configurable: true,
});

let adopted = { label: "adopted" };
if (stack.adopt(adopted, function (value) {
  "use strict";
  if (this !== undefined) throw "adopt receiver";
  if (arguments.length !== 1 || value !== adopted) throw "adopt arguments";
  calls.push("adopt");
}) !== adopted) {
  throw "adopt return";
}
if (stack.defer(function () {
  "use strict";
  if (this !== undefined) throw "defer receiver";
  if (arguments.length !== 0) throw "defer arguments";
  calls.push("defer");
}) !== undefined) {
  throw "defer return";
}

if (stack.dispose() !== undefined) throw "dispose return";
if (stack.disposed !== true) throw "disposed after walk";
if (methodReads !== 1) throw "use method acquired once";
if (calls.length !== 3) throw "three callbacks";
if (calls[0] !== "defer" || calls[1] !== "adopt" || calls[2] !== "use") {
  throw "mixed-kind LIFO";
}
if (stack.dispose() !== undefined || calls.length !== 3) throw "dispose idempotence";

// --- disposed-before-callback and re-entry ---------------------------------

let reentry = new DisposableStack();
let reentryRan = false;
reentry.defer(function () {
  reentryRan = true;
  if (reentry.disposed !== true) throw "state changed after callback";
  if (reentry.dispose() !== undefined) throw "reentrant dispose result";
  expectThrows(ReferenceError, function () {
    reentry.use(null);
  }, "reentrant use");
  expectThrows(ReferenceError, function () {
    reentry.adopt(1, function () {});
  }, "reentrant adopt");
  expectThrows(ReferenceError, function () {
    reentry.defer(function () {});
  }, "reentrant defer");
  expectThrows(ReferenceError, function () {
    reentry.move();
  }, "reentrant move");
});
reentry.dispose();
if (!reentryRan) throw "reentry callback";

// Pending-state rejection precedes argument validation and GetMethod.
let getterReadAfterDispose = false;
let lateResource = {};
Object.defineProperty(lateResource, Symbol.dispose, {
  get: function () {
    getterReadAfterDispose = true;
    return function () {};
  },
});
expectThrows(ReferenceError, function () {
  reentry.use(lateResource);
}, "disposed use precedence");
if (getterReadAfterDispose) throw "disposed use observed getter";
expectThrows(ReferenceError, function () {
  reentry.adopt(1, {});
}, "disposed adopt precedence");
expectThrows(ReferenceError, function () {
  reentry.defer({});
}, "disposed defer precedence");

// --- move transfers ownership without observing the receiver prototype ----

let moveCalls = [];
let source = new DisposableStack();
source.adopt("first", function (value) {
  moveCalls.push(value);
});
source.defer(function () {
  moveCalls.push("second");
});
let moved = source.move();
if (moveCalls.length !== 0) throw "move invoked callbacks";
if (moved === source) throw "move returned source";
if (Object.getPrototypeOf(moved) !== prototype) throw "move base prototype";
if (source.disposed !== true || moved.disposed !== false) throw "move states";
if (source.dispose() !== undefined || moveCalls.length !== 0) throw "source retained entries";
expectThrows(ReferenceError, function () {
  source.move();
}, "move source twice");
moved.dispose();
if (moveCalls.length !== 2 || moveCalls[0] !== "second" || moveCalls[1] !== "first") {
  throw "moved LIFO";
}

function CustomTarget() {}
CustomTarget.prototype = Object.create(prototype);
let derived = Reflect.construct(DisposableStack, [], CustomTarget);
let movedDerived = derived.move();
if (Object.getPrototypeOf(movedDerived) !== prototype) throw "move ignored receiver prototype";
if (derived.disposed !== true || movedDerived.disposed !== false) throw "derived move states";

// --- throw folding: exact first identity and nested suppression order -------

let singleError = { id: "single" };
let single = new DisposableStack();
single.defer(function () {
  throw singleError;
});
let singleCaught = capture(function () {
  single.dispose();
});
if (singleCaught !== singleError) throw "single error identity";
if (single.disposed !== true) throw "single throw state";

let e1 = { id: "e1" };
let e2 = { id: "e2" };
let e3 = { id: "e3" };
let errorOrder = [];
let multiple = new DisposableStack();
multiple.defer(function () {
  errorOrder.push("e1");
  throw e1;
});
multiple.defer(function () {
  errorOrder.push("e2");
  throw e2;
});
multiple.defer(function () {
  errorOrder.push("e3");
  throw e3;
});
let combined = capture(function () {
  multiple.dispose();
});
if (errorOrder.length !== 3) throw "error walk stopped";
if (errorOrder[0] !== "e3" || errorOrder[1] !== "e2" || errorOrder[2] !== "e1") {
  throw "error LIFO";
}
if (!(combined instanceof SuppressedError)) throw "outer SuppressedError";
if (combined.error !== e1) throw "outer new error";
if (!(combined.suppressed instanceof SuppressedError)) throw "inner SuppressedError";
if (combined.suppressed.error !== e2) throw "inner new error";
if (combined.suppressed.suppressed !== e3) throw "oldest suppressed error";
if (Object.getOwnPropertyDescriptor(combined, "message") !== undefined) {
  throw "outer suppression message";
}
if (Object.getOwnPropertyDescriptor(combined.suppressed, "message") !== undefined) {
  throw "inner suppression message";
}
if (multiple.disposed !== true) throw "multiple throw state";

true;
