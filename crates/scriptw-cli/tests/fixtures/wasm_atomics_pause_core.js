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

// ECMA-262 2027 Atomics.pause has no argument validation or coercion.
var ignored = [true, false, null, 42.42, -42.42, NaN, Infinity, "42", 42n,
  {}, [], function () {}, Symbol("ignored")];
for (var i = 0; i < ignored.length; ++i) {
  assertSame(Atomics.pause(ignored[i]), undefined, "ignored operand " + i);
}
var hooks = 0;
assertSame(Atomics.pause({
  get [Symbol.toPrimitive]() { ++hooks; throw "unexpected coercion"; },
  valueOf() { ++hooks; throw "unexpected valueOf"; }
}), undefined, "object not coerced");
assertSame(hooks, 0, "no hooks");
assertTypeError(function () { new Atomics.pause(); }, "constructor");

912;
