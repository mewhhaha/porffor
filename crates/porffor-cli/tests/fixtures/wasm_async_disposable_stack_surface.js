// End-to-end oracle for the %AsyncDisposableStack% intrinsic, independent of
// test262. Everything asserted here is synchronously observable: the disposal
// walk's microtask sequencing is covered by
// built-ins/AsyncDisposableStack/prototype/disposeAsync/**, not by this file,
// so a green run here means "the object exists and its synchronous surface is
// right", never "disposal works".

function expectThrows(kind, thunk, label) {
  let threw = false;
  try {
    thunk();
  } catch (error) {
    threw = error instanceof kind;
  }
  if (!threw) throw label;
}

function expectDescriptor(object, key, writable, enumerable, configurable, label) {
  let descriptor = Object.getOwnPropertyDescriptor(object, key);
  if (descriptor === undefined) throw label + " missing";
  if (descriptor.writable !== writable) throw label + " writable";
  if (descriptor.enumerable !== enumerable) throw label + " enumerable";
  if (descriptor.configurable !== configurable) throw label + " configurable";
}

// --- constructor object -----------------------------------------------------

if (typeof AsyncDisposableStack !== "function") throw "constructor missing";
if (AsyncDisposableStack.length !== 0) throw "constructor length";
if (AsyncDisposableStack.name !== "AsyncDisposableStack") throw "constructor name";
if (Object.getPrototypeOf(AsyncDisposableStack) !== Function.prototype) {
  throw "constructor proto";
}
expectDescriptor(this, "AsyncDisposableStack", true, false, true, "global binding");
expectDescriptor(AsyncDisposableStack, "prototype", false, false, false, "prototype slot");
expectDescriptor(AsyncDisposableStack, "length", false, false, true, "constructor length desc");
expectDescriptor(AsyncDisposableStack, "name", false, false, true, "constructor name desc");
expectThrows(TypeError, function () {
  AsyncDisposableStack();
}, "call without new");

// --- prototype object -------------------------------------------------------

let prototype = AsyncDisposableStack.prototype;
if (Object.getPrototypeOf(prototype) !== Object.prototype) throw "prototype proto";
expectDescriptor(prototype, "constructor", true, false, true, "prototype constructor");
if (prototype.constructor !== AsyncDisposableStack) throw "prototype constructor value";
expectDescriptor(prototype, Symbol.toStringTag, false, false, true, "toStringTag");
if (prototype[Symbol.toStringTag] !== "AsyncDisposableStack") throw "toStringTag value";

let methodNames = ["use", "adopt", "defer", "move", "disposeAsync"];
let methodLengths = [1, 2, 1, 0, 0];
for (let index = 0; index < methodNames.length; index++) {
  let name = methodNames[index];
  let arity = methodLengths[index];
  let method = prototype[name];
  if (typeof method !== "function") throw "method missing " + name;
  if (method.length !== arity) throw "method length " + name;
  if (method.name !== name) throw "method name " + name;
  expectDescriptor(prototype, name, true, false, true, "method desc " + name);
  expectDescriptor(method, "length", false, false, true, "method length desc " + name);
  expectDescriptor(method, "name", false, false, true, "method name desc " + name);
}

// @@asyncDispose must be the *same function object* as disposeAsync.
expectDescriptor(prototype, Symbol.asyncDispose, true, false, true, "asyncDispose desc");
if (prototype[Symbol.asyncDispose] !== prototype.disposeAsync) throw "asyncDispose identity";

let disposedDescriptor = Object.getOwnPropertyDescriptor(prototype, "disposed");
if (typeof disposedDescriptor.get !== "function") throw "disposed getter";
if (disposedDescriptor.set !== undefined) throw "disposed setter";
if (disposedDescriptor.enumerable !== false) throw "disposed enumerable";
if (disposedDescriptor.configurable !== true) throw "disposed configurable";
if (disposedDescriptor.get.name !== "get disposed") throw "disposed getter name";
if (disposedDescriptor.get.length !== 0) throw "disposed getter length";

// The constructor constructs; none of the prototype members do. Both halves
// have to hold at once, which is what a "make it constructible" stub breaks.
let probe = new AsyncDisposableStack();
if (Object.getPrototypeOf(probe) !== prototype) throw "instance proto";
if (Object.isExtensible(probe) !== true) throw "instance extensible";
let members = ["use", "adopt", "defer", "move", "disposeAsync"];
for (let index = 0; index < members.length; index++) {
  let method = prototype[members[index]];
  expectThrows(TypeError, function () {
    new method();
  }, "constructible member " + members[index]);
}

// --- brand checks -----------------------------------------------------------

let foreign = { "[[AsyncDisposableState]]": {} };
for (let index = 0; index < members.length - 1; index++) {
  let method = prototype[members[index]];
  expectThrows(TypeError, function () {
    method.call(foreign);
  }, "brand object " + members[index]);
  expectThrows(TypeError, function () {
    method.call(prototype);
  }, "brand prototype " + members[index]);
  expectThrows(TypeError, function () {
    method.call(1);
  }, "brand primitive " + members[index]);
}
expectThrows(TypeError, function () {
  disposedDescriptor.get.call([]);
}, "brand getter");

// --- use / adopt / defer ----------------------------------------------------

let stack = new AsyncDisposableStack();
if (stack.disposed !== false) throw "fresh disposed";

// Spelled with plain assignment rather than computed (async) method syntax, so
// this fixture measures the intrinsic and not object-literal lowering.
let asyncDisposable = {};
asyncDisposable[Symbol.asyncDispose] = function () {};
let syncDisposable = {};
syncDisposable[Symbol.dispose] = function () {};
if (stack.use(asyncDisposable) !== asyncDisposable) throw "use returns async value";
if (stack.use(syncDisposable) !== syncDisposable) throw "use returns sync value";
if (stack.use(null) !== null) throw "use returns null";
if (stack.use(undefined) !== undefined) throw "use returns undefined";

expectThrows(TypeError, function () {
  stack.use({});
}, "use missing dispose");
let notCallableAsync = {};
notCallableAsync[Symbol.asyncDispose] = 1;
expectThrows(TypeError, function () {
  stack.use(notCallableAsync);
}, "use asyncDispose not callable");
let notCallableSync = {};
notCallableSync[Symbol.dispose] = 1;
expectThrows(TypeError, function () {
  stack.use(notCallableSync);
}, "use dispose not callable");
// A null @@asyncDispose is `undefined` to GetMethod, so the lookup falls
// through to an absent @@dispose and fails there rather than at the null.
let nullAsync = {};
nullAsync[Symbol.asyncDispose] = null;
expectThrows(TypeError, function () {
  stack.use(nullAsync);
}, "use asyncDispose null falls back and fails");
expectThrows(TypeError, function () {
  stack.use(1);
}, "use primitive");

// @@asyncDispose is consulted first, and @@dispose only when it is absent.
let reads = [];
let ordered = {};
Object.defineProperty(ordered, Symbol.asyncDispose, {
  get: function () {
    reads.push("asyncDispose");
    return undefined;
  },
});
Object.defineProperty(ordered, Symbol.dispose, {
  get: function () {
    reads.push("dispose");
    return function () {};
  },
});
stack.use(ordered);
if (reads.length !== 2 || reads[0] !== "asyncDispose" || reads[1] !== "dispose") {
  throw "dispose method lookup order";
}

let adopted = {};
if (stack.adopt(adopted, function () {}) !== adopted) throw "adopt returns value";
if (stack.adopt(null, function () {}) !== null) throw "adopt returns null";
expectThrows(TypeError, function () {
  stack.adopt(null, {});
}, "adopt callback not callable");
if (stack.defer(function () {}) !== undefined) throw "defer returns undefined";
expectThrows(TypeError, function () {
  stack.defer({});
}, "defer callback not callable");
if (stack.disposed !== false) throw "still pending";

// --- move -------------------------------------------------------------------

let ranDuringMove = false;
let source = new AsyncDisposableStack();
source.defer(function () {
  ranDuringMove = true;
});
let moved = source.move();
if (ranDuringMove) throw "move disposed resources";
if (moved === source) throw "move returned receiver";
if (!(moved instanceof AsyncDisposableStack)) throw "move instance";
if (Object.getPrototypeOf(moved) !== prototype) throw "move proto";
if (source.disposed !== true) throw "move source disposed";
if (moved.disposed !== false) throw "move target pending";
expectThrows(ReferenceError, function () {
  source.use(null);
}, "use after move");
expectThrows(ReferenceError, function () {
  source.adopt(null, function () {});
}, "adopt after move");
expectThrows(ReferenceError, function () {
  source.defer(function () {});
}, "defer after move");
expectThrows(ReferenceError, function () {
  source.move();
}, "move after move");

// move() always mints a base %AsyncDisposableStack.prototype% instance, never
// one derived from the receiver. Spelled with Reflect.construct rather than
// `class ... extends` so this fixture stays an oracle for THIS lane's code and
// not for class-heritage lowering; the `extends` spelling is covered by
// built-ins/AsyncDisposableStack/prototype/move/still-returns-new-asyncdisposablestack-when-subclassed.js.
function CustomTarget() {}
CustomTarget.prototype = Object.create(prototype);
let derived = Reflect.construct(AsyncDisposableStack, [], CustomTarget);
if (Object.getPrototypeOf(derived) !== CustomTarget.prototype) throw "newtarget proto";
if (derived.disposed !== false) throw "newtarget instance pending";
let fromDerived = derived.move();
if (Object.getPrototypeOf(fromDerived) !== prototype) throw "move ignores receiver proto";
if (derived.disposed !== true) throw "move disposes derived receiver";

// A NewTarget whose `prototype` is not an object falls back to the intrinsic.
function PlainTarget() {}
PlainTarget.prototype = 1;
let fallback = Reflect.construct(AsyncDisposableStack, [], PlainTarget);
if (Object.getPrototypeOf(fallback) !== prototype) throw "newtarget primitive fallback";

// --- disposeAsync, synchronous half ----------------------------------------

let closing = new AsyncDisposableStack();
let pending = closing.disposeAsync();
if (Object.getPrototypeOf(pending) !== Promise.prototype) throw "disposeAsync promise";
// The state flips before the first await, which is what stops a second call
// from re-entering the disposal walk.
if (closing.disposed !== true) throw "disposeAsync sets disposed synchronously";
expectThrows(ReferenceError, function () {
  closing.use(null);
}, "use after disposeAsync");
if (Object.getPrototypeOf(closing.disposeAsync()) !== Promise.prototype) {
  throw "second disposeAsync promise";
}

true;
