function check(value, message) { if (!value) throw new Error(message); }
function same(actual, expected, message) { check(Object.is(actual, expected), message); }
function throws(constructor, action, message) { let caught = false; try { action(); } catch (error) { caught = error instanceof constructor; } check(caught, message); }

const f = Intl.supportedValuesOf; const sentinel = {};
let caught;
try { f({ get [Symbol.toPrimitive]() { throw sentinel; } }); } catch (error) { caught = error; }
same(caught, sentinel, "unchanged getter throw");
try { f({ [Symbol.toPrimitive]() { throw sentinel; } }); } catch (error) { caught = error; }
same(caught, sentinel, "unchanged conversion throw");
throws(TypeError, () => f({ [Symbol.toPrimitive]() { return Symbol(); } }), "symbol conversion result");
throws(TypeError, () => f({ toString() { return {}; }, valueOf() { return {}; } }), "no primitive result");
let inner;
const outer = f({ [Symbol.toPrimitive](hint) {
  same(hint, "string", "nested hint"); inner = f("unit"); inner.push("mutated"); return "timeZone";
} });
check(outer.includes("UTC") && !outer.includes("mutated"), "nested call cannot corrupt outer list");
check(!f("unit").includes("mutated"), "nested result cannot corrupt catalogue");

print("ok abrupt_and_reentrant_coercion");
262;
