const heap = 1208925819614629174706176n;

if ((255n).toString(16) !== "ff") throw "immediate radix result";
if (heap.toString(16) !== "100000000000000000000") {
  throw "heap radix result";
}

if ((255n).toLocaleString("en-US") !== "255") {
  throw "immediate locale fallback";
}
if (heap.toLocaleString("en-US", { useGrouping: false }) !==
    "1208925819614629174706176") {
  throw "heap locale fallback";
}
if (Object(255n).toLocaleString("en-US") !== "255") {
  throw "boxed locale fallback";
}

let localeTouches = 0;
let optionsTouches = 0;
const localeSentinel = {};
const optionsSentinel = {};
const untouchedLocales = new Proxy({}, {
  get: function() {
    localeTouches += 1;
    throw localeSentinel;
  }
});
const untouchedOptions = new Proxy({}, {
  get: function() {
    optionsTouches += 1;
    throw optionsSentinel;
  }
});
if ((255n).toLocaleString(untouchedLocales, untouchedOptions) !== "255") {
  throw "locale reserved arguments result";
}
if (localeTouches !== 0 || optionsTouches !== 0) {
  throw "locale reserved arguments observed";
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
