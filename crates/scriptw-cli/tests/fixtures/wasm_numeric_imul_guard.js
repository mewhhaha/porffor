// The fast candidate is a static method call with exactly two Number operands.
// Callers, getter results, wrappers and coercions must keep ordinary semantics.
function assert(condition, message) { if (!condition) throw new Error(message); }
function same(a, b) { return a === b ? a !== 0 || 1 / a === 1 / b : a !== a && b !== b; }
var original = Math.imul;
var numbers = [0, -0, 1, -1, 0.5, -1.5, 2147483647, 2147483648,
  4294967295, 4294967296, 4294967297, -4294967297, 2 ** 53, 2 ** 70,
  Number.MAX_VALUE, -Number.MAX_VALUE, Infinity, -Infinity, NaN];
for (var i = 0; i < numbers.length; ++i) {
  for (var j = 0; j < numbers.length; ++j) {
    // Captured direct calls are deliberately generic; compare exact bitwise result.
    var expected = original(numbers[i], numbers[j]);
    var actual = Math.imul(numbers[i], numbers[j]);
    assert(same(actual, expected), "numeric pair " + i + ":" + j);
  }
}
assert(Math.imul() === 0 && Math.imul(9) === 0, "missing arguments");
var trace = [];
assert(Math.imul(3, 4, trace.push("extra")) === 12 && trace.join() === "extra", "extra argument effects");
assert(Math.imul(...[3, 7]) === 21, "spread fallback");
function tail(a, b) { "use strict"; return Math.imul(a, b); }
assert(tail(3, 7) === 21, "tail continuation");
var receiver = { imul: original };
assert(receiver.imul(3, 7) === 21, "builtin ignores this");
receiver.imul = function (a, b) { assert(this === receiver, "patched this"); return a + b; };
assert(receiver.imul(3, 7) === 10, "patched method");
Math.imul = function (a, b) { return a - b; };
assert(Math.imul(9, 2) === 7, "patched Math");
Math.imul = original;
trace = [];
var getterOwner = { get imul() { trace.push("get"); return original; } };
assert(getterOwner.imul((trace.push("left"), 3), (trace.push("right"), 7)) === 21, "getter call");
assert(trace.join() === "get,left,right", "lookup before operands");
trace = [];
var owner = new Proxy({ imul: original }, { get(target, key) { trace.push("lookup:" + key); return target[key]; } });
assert(owner.imul((trace.push("left"), 4), (trace.push("right"), 8)) === 32, "proxy receiver");
assert(trace.join() === "lookup:imul,left,right", "proxy lookup order");
var capturedOwner = { imul: original };
assert(capturedOwner.imul((capturedOwner.imul = function () { return 99; }, 3), 7) === 21, "captured callee survives mutation");
assert(capturedOwner.imul(3, 7) === 99, "next call observes mutation");
trace = [];
var bound = { imul: original.bind(null, 3) };
assert(bound.imul((trace.push("a"), 7), (trace.push("b"), 99)) === 21, "bound target");
assert(trace.join() === "a,b", "bound arguments evaluated once");
trace = [];
var proxied = { imul: new Proxy(original, { apply(target, self, args) { trace.push("apply"); return Reflect.apply(target, self, args) + 1; } }) };
assert(proxied.imul((trace.push("left"), 3), (trace.push("right"), 7)) === 22, "proxy callable");
assert(trace.join() === "left,right,apply", "proxy arguments before apply");
var revoked = Proxy.revocable(original, {});var revokedOwner = { imul: revoked.proxy };revoked.revoke();
var marker = {};
trace = [];
try { revokedOwner.imul((trace.push("left"), 3), (trace.push("right"), 7)); throw marker; }
catch (error) { assert(error !== marker && error instanceof TypeError, "revoked proxy TypeError"); }
assert(trace.join() === "left,right", "revoked callee checks after operands");
trace = [];
var lhs = { valueOf() { trace.push("coerce-left"); return 3; } };
var rhs = { [Symbol.toPrimitive](hint) { trace.push("coerce-right:" + hint); return 7; } };
assert(Math.imul((trace.push("eval-left"), lhs), (trace.push("eval-right"), rhs)) === 21, "coercing operands");
assert(trace.join() === "eval-left,eval-right,coerce-left,coerce-right:number", "all eval before left-to-right coercions");
trace = [];
var abrupt = { valueOf() { trace.push("throw"); throw marker; } };
try { Math.imul(abrupt, (trace.push("eval-right"), rhs)); throw 1; }
catch (error) { assert(error === marker, "coercion original thrown value"); }
assert(trace.join() === "eval-right,throw", "throw suppresses rhs coercion");
for (var invalid of [1n, Symbol("invalid")]) {
  try { Math.imul(invalid, 7); throw marker; }
  catch (error) { assert(error !== marker && error instanceof TypeError, "nonnumeric throws"); }
}
var dynamicKey = "im" + "ul";
assert(Math[dynamicKey](3, 7) === 21, "computed key fallback");
true;
