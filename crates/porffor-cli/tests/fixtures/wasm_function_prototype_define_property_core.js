function F() {
  throw new TypeError();
}

function Test262Error(message) {
}

function $DONOTEVALUATE() {
  throw "Test262: This statement should not be evaluated.";
}

function __porfUnsupportedHost(name) {
  throw name + " unsupported in wasm-aot host harness";
}

function __porfIsHTMLDDA() {
  return null;
}

function AbstractModuleSource() {
  throw new TypeError();
}

function __porfAbstractModuleSourceToStringTag() {
  return undefined;
}

function tagGetter() {
  return undefined;
}

Object.defineProperty(F, "prototype", {
  value: F.prototype,
  writable: false,
  enumerable: false,
  configurable: false,
});

var desc = Object.getOwnPropertyDescriptor(F, "prototype");
if (desc.value !== F.prototype) {
  throw "prototype value";
}
if (desc.writable !== false) {
  throw "prototype writable";
}
if (desc.enumerable !== false) {
  throw "prototype enumerable";
}
if (desc.configurable !== false) {
  throw "prototype configurable";
}

Object.defineProperty(F.prototype, Symbol.toStringTag, {
  get: tagGetter,
  set: undefined,
  enumerable: false,
  configurable: true,
});

Object.defineProperty(AbstractModuleSource, "prototype", {
  value: AbstractModuleSource.prototype,
  writable: false,
  enumerable: false,
  configurable: false,
});

Object.defineProperty(AbstractModuleSource.prototype, Symbol.toStringTag, {
  get: __porfAbstractModuleSourceToStringTag,
  set: undefined,
  enumerable: false,
  configurable: true,
});

var tagDesc = Object.getOwnPropertyDescriptor(F.prototype, Symbol.toStringTag);
if (tagDesc.get !== tagGetter) {
  throw "tag getter";
}
if (tagDesc.set !== undefined) {
  throw "tag setter";
}
if (tagDesc.enumerable !== false) {
  throw "tag enumerable";
}
if (tagDesc.configurable !== true) {
  throw "tag configurable";
}
if (tagDesc.get.call(F.prototype) !== undefined) {
  throw "tag call";
}

var threw = false;
try {
  new F();
} catch (error) {
  threw = true;
}
if (!threw) {
  throw "constructor did not throw";
}

var holder = {
  F: F,
  AbstractModuleSource: F,
};

var $262 = {
  AbstractModuleSource: F,
};
var $263 = {
  AbstractModuleSource: AbstractModuleSource,
};

if (holder.F !== F) {
  throw "holder value";
}
if (holder.AbstractModuleSource !== F) {
  throw "holder abstract value";
}
if ($262.AbstractModuleSource !== F) {
  throw "$262 abstract value";
}
if ($263.AbstractModuleSource !== AbstractModuleSource) {
  throw "$263 abstract value";
}

function assert(mustBeTrue, message) {
  if (mustBeTrue) {
    return;
  }
  throw message;
}

assert.sameValue = function (actual, expected, message) {
  if (actual === expected) {
    return;
  }
  throw message;
};

var boxed = new Number(42);
if (boxed.valueOf() !== 42) {
  throw "boxed number valueOf";
}

true;
