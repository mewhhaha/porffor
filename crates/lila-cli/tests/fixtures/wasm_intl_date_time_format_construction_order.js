function assertTrace(actual, expected, label) {
  if (actual.length !== expected.length) throw label + " length";
  for (let i = 0; i < expected.length; i++) {
    if (actual[i] !== expected[i]) throw label + " entry " + i;
  }
}

let trace = [];
let customPrototype = {};
let successfulTarget = new Proxy(function () {}, {
  get(target, key) {
    if (key !== "prototype") throw "unexpected successful NewTarget key";
    trace.push("prototype");
    return customPrototype;
  },
});
let locales = {
  get length() {
    trace.push("locales length");
    return 1;
  },
  get 0() {
    trace.push("locales 0");
    return {
      toString() {
        trace.push("locale toString");
        return "en";
      },
    };
  },
};
let options = {
  get localeMatcher() {
    trace.push("options localeMatcher");
    return "lookup";
  },
};
let formatter = Reflect.construct(
  Intl.DateTimeFormat,
  [locales, options],
  successfulTarget,
);
assertTrace(
  trace,
  [
    "prototype",
    "locales length",
    "locales 0",
    "locale toString",
    "options localeMatcher",
  ],
  "successful construction order",
);
if (Object.getPrototypeOf(formatter) !== customPrototype) throw "custom prototype";
let resolved = Intl.DateTimeFormat.prototype.resolvedOptions.call(formatter);
if (typeof resolved.locale !== "string") throw "initialized DateTimeFormat record";

function assertTaggedPrototype(prototype) {
  let taggedTarget = new Proxy(function () {}, {
    get(target, key) {
      if (key !== "prototype") throw "unexpected tagged NewTarget key";
      return prototype;
    },
  });
  let taggedFormatter = Reflect.construct(Intl.DateTimeFormat, [["en"]], taggedTarget);
  if (Object.getPrototypeOf(taggedFormatter) !== prototype) throw "tagged prototype";
  let taggedOptions = Intl.DateTimeFormat.prototype.resolvedOptions.call(taggedFormatter);
  if (typeof taggedOptions.locale !== "string") throw "tagged prototype record";
}
assertTaggedPrototype(function dateTimeFormatFunctionPrototype() {});
assertTaggedPrototype([]);
assertTaggedPrototype((function () { return arguments; })());

let prototypeSentinel = {};
let abruptPrototypeTrace = [];
let abruptPrototypeTarget = new Proxy(function () {}, {
  get(target, key) {
    if (key !== "prototype") throw "unexpected abrupt NewTarget key";
    abruptPrototypeTrace.push("prototype");
    throw prototypeSentinel;
  },
});
let abruptLocaleObserved = false;
let abruptOptionsObserved = false;
let abruptPrototypeCaught = false;
try {
  Reflect.construct(
    Intl.DateTimeFormat,
    [
      {
        get length() {
          abruptLocaleObserved = true;
          throw "locale must remain unobserved";
        },
      },
      new Proxy({}, {
        get() {
          abruptOptionsObserved = true;
          throw "options must remain unobserved";
        },
      }),
    ],
    abruptPrototypeTarget,
  );
} catch (error) {
  abruptPrototypeCaught = error === prototypeSentinel;
}
if (!abruptPrototypeCaught) throw "prototype abrupt completion identity";
assertTrace(abruptPrototypeTrace, ["prototype"], "prototype abrupt order");
if (abruptLocaleObserved) throw "prototype abrupt completion observed locales";
if (abruptOptionsObserved) throw "prototype abrupt completion observed options";

trace = [];
let localeSentinel = {};
let abruptLocaleTarget = new Proxy(function () {}, {
  get(target, key) {
    if (key !== "prototype") throw "unexpected locale-abrupt NewTarget key";
    trace.push("prototype");
    return {};
  },
});
let localeAbruptOptionsObserved = false;
let abruptLocaleCaught = false;
try {
  Reflect.construct(
    Intl.DateTimeFormat,
    [
      [{
        toString() {
          trace.push("locale toString");
          throw localeSentinel;
        },
      }],
      new Proxy({}, {
        get() {
          localeAbruptOptionsObserved = true;
          throw "options must remain unobserved";
        },
      }),
    ],
    abruptLocaleTarget,
  );
} catch (error) {
  abruptLocaleCaught = error === localeSentinel;
}
if (!abruptLocaleCaught) throw "locale abrupt completion identity";
assertTrace(trace, ["prototype", "locale toString"], "locale abrupt order");
if (localeAbruptOptionsObserved) throw "locale abrupt completion observed options";

262;
