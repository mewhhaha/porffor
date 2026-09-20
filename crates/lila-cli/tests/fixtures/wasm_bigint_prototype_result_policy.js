const heap = 1208925819614629174706176n;

if ((255n).toString(16) !== "ff") throw "immediate radix result";
if (heap.toString(16) !== "100000000000000000000") {
  throw "heap radix result";
}

if ((255n).toLocaleString("en-US") !== "255") {
  throw "immediate locale result";
}
if (heap.toLocaleString("en-US", { useGrouping: false }) !==
    "1208925819614629174706176") {
  throw "heap locale result";
}
if (Object(255n).toLocaleString("en-US") !== "255") {
  throw "boxed locale result";
}

let localeTouches = 0;
let optionsTouches = 0;
const localeSentinel = {};
const optionsSentinel = {};
const throwingLocales = new Proxy({}, {
  get: function() {
    localeTouches += 1;
    throw localeSentinel;
  }
});
const throwingOptions = new Proxy({}, {
  get: function() {
    optionsTouches += 1;
    throw optionsSentinel;
  }
});
let localeAbruptIdentity = false;
try {
  (255n).toLocaleString(throwingLocales, throwingOptions);
} catch (error) {
  localeAbruptIdentity = error === localeSentinel;
}
if (!localeAbruptIdentity || localeTouches !== 1 || optionsTouches !== 0) {
  throw "locale abrupt identity and observation order";
}
let optionsAbruptIdentity = false;
try {
  (255n).toLocaleString("en-US", throwingOptions);
} catch (error) {
  optionsAbruptIdentity = error === optionsSentinel;
}
if (!optionsAbruptIdentity || localeTouches !== 1 || optionsTouches !== 1) {
  throw "options abrupt identity and one observation";
}

if ((255n).valueOf() !== 255n) throw "primitive exact value";
if (Object(heap).valueOf() !== heap) throw "boxed exact value";

let receiverTypeError = false;
try {
  BigInt.prototype.toLocaleString.call(1, "en-US");
} catch (error) {
  receiverTypeError = error instanceof TypeError;
}
if (!receiverTypeError) throw "locale receiver TypeError";

const sentinel = {};
let radixAbruptIdentity = false;
try {
  (1n).toString({
    valueOf: function() {
      throw sentinel;
    }
  });
} catch (error) {
  radixAbruptIdentity = error === sentinel;
}
if (!radixAbruptIdentity) throw "radix abrupt identity";

let radixRangeError = false;
try {
  (1n).toString(1);
} catch (error) {
  radixRangeError = error instanceof RangeError;
}
if (!radixRangeError) throw "radix RangeError";

const capturedMainLexical = Symbol("main captured lexical");
function readCapturedMainLexical() {
  return capturedMainLexical;
}
let mainSymbolToNumericTypeError = false;
try {
  capturedMainLexical++;
} catch (error) {
  mainSymbolToNumericTypeError = error instanceof TypeError;
}
if (
  !mainSymbolToNumericTypeError ||
  readCapturedMainLexical() !== capturedMainLexical
) {
  throw "main lexical Symbol ToNumeric realm fallback";
}

const other = __lilaCreateRealm().global;

function expectForeignRangeError(run, label) {
  try {
    run();
  } catch (error) {
    if (error instanceof other.RangeError && !(error instanceof RangeError)) {
      return;
    }
  }
  throw label;
}

function expectForeignTypeError(run, label) {
  try {
    run();
  } catch (error) {
    if (error instanceof other.TypeError && !(error instanceof TypeError)) {
      return;
    }
  }
  throw label;
}

expectForeignRangeError(function() {
  other.BigInt.prototype.toString.call(1n, 1);
}, "foreign immediate radix RangeError realm");
expectForeignRangeError(function() {
  other.BigInt.prototype.toString.call(heap, 37);
}, "foreign heap radix RangeError realm");

expectForeignTypeError(function() {
  other.BigInt.prototype.toString.call(1n, 2n);
}, "foreign immediate BigInt radix TypeError realm");
expectForeignTypeError(function() {
  other.BigInt.prototype.toString.call(heap, Symbol("radix"));
}, "foreign heap Symbol radix TypeError realm");
expectForeignTypeError(function() {
  other.BigInt.prototype.toString.call(1n, {
    valueOf: function() {
      return 2n;
    }
  });
}, "foreign implicit radix TypeError realm");

123;
