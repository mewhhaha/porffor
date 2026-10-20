function assertTrace(trace, expected, label) {
  if (trace.length !== expected.length) throw label + " length";
  for (let i = 0; i < expected.length; i++) {
    if (trace[i] !== expected[i]) throw label + " entry " + i;
  }
}

// OrdinaryCreateFromConstructor observes NewTarget.prototype before the tag.
let trace = [];
let customPrototype = {};
let successfulTarget = new Proxy(function () {}, {
  get(target, key) {
    if (key !== "prototype") throw "unexpected successful NewTarget key";
    trace.push("prototype");
    return customPrototype;
  },
});
let locale = Reflect.construct(Intl.Locale, [{
  toString() {
    trace.push("tag");
    return "en";
  },
}, undefined], successfulTarget);
assertTrace(trace, ["prototype", "tag"], "successful construction order");
if (Object.getPrototypeOf(locale) !== customPrototype) throw "custom Locale prototype";
if (Intl.Locale.prototype.toString.call(locale) !== "en") throw "initialized Locale record";

// The prototype's complete tagged identity must survive allocation. Function,
// Array and Arguments prototypes expose a dropped/hard-coded Object tag under
// strict identity even though all four representations carry heap pointers.
function assertTaggedPrototype(prototype) {
  let taggedTarget = new Proxy(function () {}, {
    get(target, key) {
      if (key !== "prototype") throw "unexpected tagged NewTarget key";
      return prototype;
    },
  });
  let taggedLocale = Reflect.construct(Intl.Locale, ["en"], taggedTarget);
  if (Object.getPrototypeOf(taggedLocale) !== prototype) throw "tagged Locale prototype";
  if (Intl.Locale.prototype.toString.call(taggedLocale) !== "en") {
    throw "tagged prototype Locale record";
  }
}
assertTaggedPrototype(function localeFunctionPrototype() {});
assertTaggedPrototype([]);
assertTaggedPrototype((function () { return arguments; })());

// An abrupt prototype lookup precedes the primitive-tag TypeError and
// suppresses every options read.
let prototypeSentinel = {};
let abruptPrototypeTrace = [];
let abruptPrototypeTarget = new Proxy(function () {}, {
  get(target, key) {
    if (key !== "prototype") throw "unexpected abrupt NewTarget key";
    abruptPrototypeTrace.push("prototype");
    throw prototypeSentinel;
  },
});
let abruptPrototypeOptionsObserved = false;
let abruptPrototypeCaught = false;
try {
  Reflect.construct(Intl.Locale, [1, new Proxy({}, {
    get() {
      abruptPrototypeOptionsObserved = true;
      return undefined;
    },
  })], abruptPrototypeTarget);
} catch (error) {
  abruptPrototypeCaught = error === prototypeSentinel;
}
if (!abruptPrototypeCaught) throw "prototype abrupt completion identity";
assertTrace(abruptPrototypeTrace, ["prototype"], "primitive tag prototype precedence");
if (abruptPrototypeOptionsObserved) throw "prototype abrupt completion observed options";

// Once prototype reservation succeeds, the same primitive tag is rejected by
// the called constructor's Realm TypeError.
let primitiveTypeErrorTrace = [];
let primitiveTypeErrorTarget = new Proxy(function () {}, {
  get(target, key) {
    if (key !== "prototype") throw "unexpected primitive NewTarget key";
    primitiveTypeErrorTrace.push("prototype");
    return {};
  },
});
let primitiveTypeError;
try {
  Reflect.construct(Intl.Locale, [1], primitiveTypeErrorTarget);
} catch (error) {
  primitiveTypeError = error;
}
assertTrace(primitiveTypeErrorTrace, ["prototype"], "primitive tag TypeError order");
if (!(primitiveTypeError instanceof TypeError)) throw "primitive tag TypeError realm";

// A tag abrupt completion sees the prototype read first, but no options read.
trace = [];
let tagSentinel = {};
let abruptTagOptionsObserved = false;
let abruptTagTarget = new Proxy(function () {}, {
  get(target, key) {
    if (key !== "prototype") throw "unexpected tag-abrupt NewTarget key";
    trace.push("prototype");
    return {};
  },
});
let abruptTagCaught = false;
try {
  Reflect.construct(Intl.Locale, [{
    toString() {
      trace.push("tag");
      throw tagSentinel;
    },
  }, new Proxy({}, {
    get() {
      abruptTagOptionsObserved = true;
      return undefined;
    },
  })], abruptTagTarget);
} catch (error) {
  abruptTagCaught = error === tagSentinel;
}
if (!abruptTagCaught) throw "tag abrupt completion identity";
assertTrace(trace, ["prototype", "tag"], "tag abrupt construction order");
if (abruptTagOptionsObserved) throw "tag abrupt completion observed options";

262;
