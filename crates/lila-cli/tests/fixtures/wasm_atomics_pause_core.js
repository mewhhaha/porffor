function assertSame(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual;
}

function assertTypeError(fn, label) {
  let threw = false;
  try {
    fn();
  } catch (error) {
    threw = true;
    if (!(error instanceof TypeError)) throw label + " wrong error";
  }
  if (!threw) throw label + " missing throw";
}

if (typeof Atomics.pause !== "function") throw "pause function";
if (Atomics.pause.length !== 0) throw "pause length";
if (Atomics.pause.name !== "pause") throw "pause name";

var desc = Object.getOwnPropertyDescriptor(Atomics, "pause");
if (desc === undefined) throw "pause descriptor missing";
if (desc.value !== Atomics.pause) throw "pause descriptor value";
if (desc.writable !== true) throw "pause descriptor writable";
if (desc.enumerable !== false) throw "pause descriptor enumerable";
if (desc.configurable !== true) throw "pause descriptor configurable";

assertSame(Atomics.pause(), undefined, "no argument");
assertSame(Atomics.pause(undefined), undefined, "undefined");
assertSame(Atomics.pause(42), undefined, "integer");
assertSame(Atomics.pause(0), undefined, "zero");
assertSame(Atomics.pause(-0), undefined, "negative zero");
assertSame(Atomics.pause(9007199254740991), undefined, "max safe integer");

assertTypeError(function () { Atomics.pause(true); }, "true");
assertTypeError(function () { Atomics.pause(false); }, "false");
assertTypeError(function () { Atomics.pause(null); }, "null");
assertTypeError(function () { Atomics.pause(42.42); }, "fraction");
assertTypeError(function () { Atomics.pause(-42.42); }, "negative fraction");
assertTypeError(function () { Atomics.pause(NaN); }, "NaN");
assertTypeError(function () { Atomics.pause(Infinity); }, "Infinity");
assertTypeError(function () { Atomics.pause("42"); }, "string");
assertTypeError(function () { Atomics.pause(42n); }, "BigInt");
assertTypeError(function () { Atomics.pause({}); }, "object");
assertTypeError(function () { Atomics.pause([]); }, "array");
assertTypeError(function () { Atomics.pause(function () {}); }, "function");
assertTypeError(function () {
  Atomics.pause({
    valueOf() {
      return 42;
    }
  });
}, "valueOf object");
assertTypeError(function () { new Atomics.pause(); }, "constructor");

912;
