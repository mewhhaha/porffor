let ok = true;

let keyCalls = 0;
const missing = null;
ok = ok && missing?.[keyCalls++] === undefined;
ok = ok && keyCalls === 0;

let baseCalls = 0;
function base() {
  baseCalls += 1;
  return { first: { second: 7 } };
}
ok = ok && base()?.first.second === 7;
ok = ok && baseCalls === 1;

ok = ok && null?.first.second === undefined;

let groupedThrew = false;
try {
  (null?.first).second;
} catch (error) {
  groupedThrew = error instanceof TypeError;
}
ok = ok && groupedThrew;

let ordinaryTailThrew = false;
try {
  ({ first: null })?.first.second;
} catch (error) {
  ordinaryTailThrew = error instanceof TypeError;
}
ok = ok && ordinaryTailThrew;

let getterThrowCaught = false;
try {
  ({ get value() { throw 7; } })?.value;
} catch (error) {
  getterThrowCaught = error === 7;
}
ok = ok && getterThrowCaught;

function throwingKey() {
  throw 8;
}
let keyThrowCaught = false;
try {
  ({})?.[throwingKey()];
} catch (error) {
  keyThrowCaught = error === 8;
}
ok = ok && keyThrowCaught;

ok = ok && typeof (42)?.toString === "function";
ok = ok && typeof true?.valueOf === "function";
ok = ok && typeof 1n?.toString === "function";
ok = ok && typeof Symbol("s")?.valueOf === "function";
ok = ok && "hello"?.length === 5;
ok = ok && "hello"?.[1] === "e";

const originalStringToString = String.prototype.toString;
String.prototype.optionalMarker = 7;
String.prototype.toString = 8;
ok = ok && "value"?.optionalMarker === 7;
ok = ok && "value"?.toString === 8;
String.prototype.toString = originalStringToString;

const originalBigIntToString = BigInt.prototype.toString;
BigInt.prototype.optionalMarker = 9;
BigInt.prototype.toString = 10;
ok = ok && 1n?.optionalMarker === 9;
ok = ok && 1n?.toString === 10;
BigInt.prototype.toString = originalBigIntToString;

const values = [39, 42];
values.true = "named";
values[1.1] = "fractional";
ok = ok && values?.[0] === 39;
ok = ok && values?.[0, 1] === 42;
ok = ok && values?.[true] === "named";
ok = ok && values?.[1.1] === "fractional";

function runtimeNamedArrayKey(key, name) {
  const target = [];
  target[key] = 11;
  return target.length === 0 && target[name] === 11;
}
ok = ok && runtimeNamedArrayKey(1.1, "1.1");
ok = ok && runtimeNamedArrayKey(-1, "-1");
ok = ok && runtimeNamedArrayKey(NaN, "NaN");
ok = ok && runtimeNamedArrayKey(Infinity, "Infinity");

const indexed = [1, 2];
let runtimeIndex = 1;
indexed[runtimeIndex] = 12;
ok = ok && indexed.length === 2;
ok = ok && indexed[1] === 12;

let assignmentOrder = "";
function orderedKey() {
  assignmentOrder += "k";
  return 0;
}
function orderedValue() {
  assignmentOrder += "v";
  return 13;
}
const ordered = [];
ordered[orderedKey()] = orderedValue();
ok = ok && assignmentOrder === "kv";
ok = ok && ordered[0] === 13;

function strictPlainCall() {
  "use strict";
  return this === undefined;
}
ok = ok && strictPlainCall?.() === true;

const callReceiver = {
  marker: 14,
  method: function () {
    "use strict";
    return this.marker;
  }
};
ok = ok && callReceiver?.method() === 14;
ok = ok && callReceiver.method?.() === 14;
ok = ok && (callReceiver?.method)() === 14;
ok = ok && (callReceiver?.method)?.() === 14;

let groupedOrdinaryArgCount = 0;
let groupedOrdinaryThrew = false;
try {
  (null?.method)(groupedOrdinaryArgCount++);
} catch (error) {
  groupedOrdinaryThrew = error instanceof TypeError;
}
ok = ok && groupedOrdinaryThrew;
ok = ok && groupedOrdinaryArgCount === 1;

let groupedOptionalArgCount = 0;
ok = ok && (null?.method)?.(groupedOptionalArgCount++) === undefined;
ok = ok && groupedOptionalArgCount === 0;

String.prototype.optionalReceiver = function () {
  "use strict";
  return this === "z";
};
Number.prototype.optionalReceiver = function () {
  "use strict";
  return this === 3;
};
Boolean.prototype.optionalReceiver = function () {
  "use strict";
  return this === true;
};
BigInt.prototype.optionalReceiver = function () {
  "use strict";
  return this === 3n;
};
const strictReceiverSymbol = Symbol("strict receiver");
Symbol.prototype.optionalReceiver = function () {
  "use strict";
  return this === strictReceiverSymbol;
};
ok = ok && "z"?.optionalReceiver() === true;
ok = ok && (3)?.optionalReceiver() === true;
ok = ok && true?.optionalReceiver() === true;
ok = ok && 3n?.optionalReceiver() === true;
ok = ok && strictReceiverSymbol?.optionalReceiver() === true;

function optionalStrictStringFactory() {
  return "z";
}
function optionalStrictNumberFactory() {
  return 3;
}
function optionalStrictBooleanFactory() {
  return true;
}
function optionalStrictBigIntFactory() {
  return 3n;
}
function optionalStrictSymbolFactory() {
  return strictReceiverSymbol;
}
String.prototype.optionalFactoryReceiver = function () {
  "use strict";
  return this === "z";
};
Number.prototype.optionalFactoryReceiver = function () {
  "use strict";
  return this === 3;
};
Boolean.prototype.optionalFactoryReceiver = function () {
  "use strict";
  return this === true;
};
BigInt.prototype.optionalFactoryReceiver = function () {
  "use strict";
  return this === 3n;
};
Symbol.prototype.optionalFactoryReceiver = function () {
  "use strict";
  return this === strictReceiverSymbol;
};
ok = ok && optionalStrictStringFactory?.().optionalFactoryReceiver() === true;
ok = ok && optionalStrictNumberFactory?.().optionalFactoryReceiver() === true;
ok = ok && optionalStrictBooleanFactory?.().optionalFactoryReceiver() === true;
ok = ok && optionalStrictBigIntFactory?.().optionalFactoryReceiver() === true;
ok = ok && optionalStrictSymbolFactory?.().optionalFactoryReceiver() === true;

function sloppyPrimitiveReceiver() {
  return typeof this === "object";
}
String.prototype.optionalSloppyReceiver = sloppyPrimitiveReceiver;
Number.prototype.optionalSloppyReceiver = sloppyPrimitiveReceiver;
Boolean.prototype.optionalSloppyReceiver = sloppyPrimitiveReceiver;
BigInt.prototype.optionalSloppyReceiver = sloppyPrimitiveReceiver;
Symbol.prototype.optionalSloppyReceiver = sloppyPrimitiveReceiver;
ok = ok && "z"?.optionalSloppyReceiver() === true;
ok = ok && (3)?.optionalSloppyReceiver() === true;
ok = ok && true?.optionalSloppyReceiver() === true;
ok = ok && 3n?.optionalSloppyReceiver() === true;
ok = ok && Symbol("sloppy receiver")?.optionalSloppyReceiver() === true;

const defaultMethodSymbol = Symbol("default methods");
ok = ok && defaultMethodSymbol?.valueOf() === defaultMethodSymbol;
ok = ok && defaultMethodSymbol?.toString() === "Symbol(default methods)";

ok = ok && "ab"?.toUpperCase() === "AB";
ok = ok && "ab"?.slice(1) === "b";
ok = ok && "ab"?.includes("a") === true;
ok = ok && "ab"?.charAt(0) === "a";
function optionalStringFactory() {
  return "ab";
}
ok = ok && optionalStringFactory?.().toUpperCase() === "AB";

let callBaseCount = 0;
let callGetterCount = 0;
const callHolder = {
  marker: 15,
  get method() {
    callGetterCount += 1;
    return function () {
      "use strict";
      return this.marker;
    };
  }
};
function getCallHolder() {
  callBaseCount += 1;
  return callHolder;
}
ok = ok && getCallHolder()?.method?.() === 15;
ok = ok && callBaseCount === 1;
ok = ok && callGetterCount === 1;

let lazyCallArgCount = 0;
const absentCall = null;
ok = ok && absentCall?.(lazyCallArgCount++) === undefined;
ok = ok && lazyCallArgCount === 0;

let nonCallableArgCount = 0;
let nonCallableThrew = false;
try {
  (1)?.(nonCallableArgCount++);
} catch (error) {
  nonCallableThrew = error instanceof TypeError;
}
ok = ok && nonCallableThrew;
ok = ok && nonCallableArgCount === 1;

let ordinaryCallArgCount = 0;
let ordinaryCallThrew = false;
try {
  ({ method: null })?.method(ordinaryCallArgCount++);
} catch (error) {
  ordinaryCallThrew = error instanceof TypeError;
}
ok = ok && ordinaryCallThrew;
ok = ok && ordinaryCallArgCount === 1;

let skippedCallArgCount = 0;
const absentReceiver = null;
ok = ok && absentReceiver?.method?.(skippedCallArgCount++).value === undefined;
ok = ok && skippedCallArgCount === 0;

const nullCallResult = {
  method() {
    return null;
  }
};
let ordinaryCallTailThrew = false;
try {
  nullCallResult?.method?.().value;
} catch (error) {
  ordinaryCallTailThrew = error instanceof TypeError;
}
ok = ok && ordinaryCallTailThrew;

const objectCallResult = {
  method() {
    return { value: 16 };
  }
};
ok = ok && objectCallResult?.method?.().value === 16;

let optionalSuperCalled = false;
let optionalSuperContext;
let optionalSuperOrder = "";
let optionalSuperArgCount = 0;
function optionalSuperKey() {
  optionalSuperOrder += "k";
  return "ordered";
}
function optionalSuperArg() {
  optionalSuperArgCount += 1;
  optionalSuperOrder += "a";
  return 17;
}
class OptionalSuperBase {
  method() {
    optionalSuperCalled = true;
    optionalSuperContext = this;
  }

  ordered(value) {
    optionalSuperOrder += "m";
    optionalSuperContext = this;
    return value;
  }

  get absent() {
    optionalSuperOrder += "g";
    return null;
  }

  get exploding() {
    optionalSuperOrder += "t";
    throw 23;
  }
}
class OptionalSuperDerived extends OptionalSuperBase {
  method() {
    super.method?.();
  }

  computed() {
    return super[optionalSuperKey()]?.(optionalSuperArg());
  }

  grouped() {
    return (super.ordered)?.(optionalSuperArg());
  }

  skipped() {
    return super.absent?.(optionalSuperArg());
  }

  throwing() {
    return super.exploding?.(optionalSuperArg());
  }
}
const optionalSuperInstance = new OptionalSuperDerived();
optionalSuperInstance.method();
ok = ok && optionalSuperCalled;
ok = ok && optionalSuperContext === optionalSuperInstance;

optionalSuperOrder = "";
ok = ok && optionalSuperInstance.computed() === 17;
ok = ok && optionalSuperOrder === "kam";
ok = ok && optionalSuperArgCount === 1;
ok = ok && optionalSuperContext === optionalSuperInstance;

optionalSuperOrder = "";
ok = ok && optionalSuperInstance.grouped() === 17;
ok = ok && optionalSuperOrder === "am";
ok = ok && optionalSuperArgCount === 2;
ok = ok && optionalSuperContext === optionalSuperInstance;

optionalSuperOrder = "";
ok = ok && optionalSuperInstance.skipped() === undefined;
ok = ok && optionalSuperOrder === "g";
ok = ok && optionalSuperArgCount === 2;

optionalSuperOrder = "";
let optionalSuperThrow;
try {
  optionalSuperInstance.throwing();
} catch (error) {
  optionalSuperThrow = error;
}
ok = ok && optionalSuperThrow === 23;
ok = ok && optionalSuperOrder === "t";
ok = ok && optionalSuperArgCount === 2;

ok;
