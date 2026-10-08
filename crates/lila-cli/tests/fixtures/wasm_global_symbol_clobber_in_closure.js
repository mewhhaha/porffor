function check(condition, message) {
  if (!condition) throw message;
}

var TE = TypeError;
function throwsTE(f, message) {
  try {
    f();
  } catch (e) {
    check(e instanceof TE, message + " throws TypeError");
    return;
  }
  throw message + " did not throw";
}

Symbol = undefined;

var directThrew = false;
try {
  Symbol.iterator;
} catch (e) {
  directThrew = e instanceof TE;
}
check(directThrew, "direct read throws TypeError");

throwsTE(() => Symbol.iterator, "arrow Symbol.iterator read");
throwsTE(function() { return Symbol.iterator; }, "function Symbol.iterator read");

// The map helper itself must keep working after the clobber.
var iterator = [0].values();
check(iterator.map(x => x + 1).next().value === 1, "map works after clobber");

// Static member-call dispatch must observe the clobber too.
Object = undefined;
throwsTE(function() { return Object.keys({}); }, "Object.keys");
throwsTE(function() { return Object.is(1, 1); }, "Object.is");
throwsTE(function() { return Object.values({}); }, "Object.values");

Array = undefined;
throwsTE(function() { return Array.of(1); }, "Array.of");

Math = undefined;
throwsTE(function() { return Math.pow(2, 3); }, "Math.pow");

String = undefined;
throwsTE(function() { return String.fromCharCode(65); }, "String.fromCharCode");

Number = undefined;
throwsTE(function() { return Number(1); }, "Number call");

// Primitive method dispatch does not go through the global.
check((1).toFixed(2) === "1.00", "number method still works");

true;
