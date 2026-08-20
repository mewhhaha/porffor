// Constructor-only oracle for %DisposableStack%. Synchronous disposal
// methods intentionally remain absent until their algorithms exist.
"use strict";

// Strict global PutValue performs an observable HasProperty before Set. Keep
// both the stored value and expression result live across that helper path.
var strictGlobalCompound = 0;
var strictGlobalPrefix = 0;
function exerciseStrictGlobalWrites() {
  let compoundResult = (strictGlobalCompound += 1);
  let prefixResult = ++strictGlobalPrefix;
  if (compoundResult !== 1 || strictGlobalCompound !== 1) {
    throw "strict global compound assignment";
  }
  if (prefixResult !== 1 || strictGlobalPrefix !== 1) {
    throw "strict global prefix update";
  }
}
exerciseStrictGlobalWrites();

function expectThrowsTypeError(thunk, label) {
  let threw = false;
  try {
    thunk();
  } catch (error) {
    threw = error instanceof TypeError;
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

if (typeof DisposableStack !== "function") throw "constructor missing";
if (DisposableStack.name !== "DisposableStack") throw "constructor name";
if (DisposableStack.length !== 0) throw "constructor length";
if (Object.getPrototypeOf(DisposableStack) !== Function.prototype) {
  throw "constructor prototype";
}
expectDescriptor(this, "DisposableStack", true, false, true, "global");
expectDescriptor(DisposableStack, "prototype", false, false, false, "prototype slot");
expectDescriptor(DisposableStack.prototype, "constructor", true, false, true, "constructor link");
expectDescriptor(DisposableStack.prototype, Symbol.toStringTag, false, false, true, "toStringTag");
if (DisposableStack.prototype[Symbol.toStringTag] !== "DisposableStack") {
  throw "toStringTag value";
}
if (Object.getPrototypeOf(DisposableStack.prototype) !== Object.prototype) {
  throw "prototype parent";
}

expectThrowsTypeError(function () {
  DisposableStack();
}, "call without new");

let stack = new DisposableStack();
if (Object.getPrototypeOf(stack) !== DisposableStack.prototype) throw "instance prototype";
if (!Object.isExtensible(stack)) throw "instance extensible";

function CustomTarget() {}
let customPrototype = [];
CustomTarget.prototype = customPrototype;
let custom = Reflect.construct(DisposableStack, [], CustomTarget);
if (Object.getPrototypeOf(custom) !== customPrototype) throw "custom prototype";

for (let primitive of [undefined, null, true, "x", 1, Symbol("x")]) {
  function PrimitiveTarget() {}
  PrimitiveTarget.prototype = primitive;
  let fallback = Reflect.construct(DisposableStack, [], PrimitiveTarget);
  if (Object.getPrototypeOf(fallback) !== DisposableStack.prototype) {
    throw "primitive prototype fallback";
  }
}

let prototypeGets = 0;
let sentinel = {};
let observingTarget = new Proxy(function () {}, {
  get: function (_target, key) {
    if (key === "prototype") {
      prototypeGets++;
      throw sentinel;
    }
  },
});
let sawSentinel = false;
try {
  Reflect.construct(DisposableStack, [], observingTarget);
} catch (error) {
  sawSentinel = error === sentinel;
}
if (!sawSentinel || prototypeGets !== 1) throw "prototype Get";

// No placeholders: these properties stay absent until the real synchronous
// disposal algorithms land.
for (let name of ["use", "adopt", "defer", "move", "dispose", "disposed"]) {
  if (Object.prototype.hasOwnProperty.call(DisposableStack.prototype, name)) {
    throw "placeholder " + name;
  }
}
if (Object.prototype.hasOwnProperty.call(DisposableStack.prototype, Symbol.dispose)) {
  throw "placeholder Symbol.dispose";
}

for (let name of ["use", "adopt", "defer", "move"]) {
  expectThrowsTypeError(function () {
    AsyncDisposableStack.prototype[name].call(stack);
  }, "async brand " + name);
}

AsyncDisposableStack.prototype.disposeAsync.call(stack).then(
  function () {
    throw "async dispose brand fulfilled";
  },
  function (error) {
    if (!(error instanceof TypeError)) throw "async dispose brand reason";
    print("disposable-stack-async-brand:true");
  }
);

true;
