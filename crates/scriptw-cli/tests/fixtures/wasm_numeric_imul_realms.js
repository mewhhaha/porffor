function assert(condition, message) { if (!condition) throw new Error(message); }
var other = __scriptwCreateRealm().global;
var foreign = { imul: other.Math.imul };
assert(foreign.imul(3, 7) === 21, "foreign numeric result");
var secondCoerced = false;
try { foreign.imul(Symbol("bad"), { valueOf() { secondCoerced = true; return 7; } }); throw 1; }
catch (error) {
  assert(error instanceof other.TypeError && !(error instanceof TypeError), "foreign builtin error realm");
}
assert(!secondCoerced, "foreign abrupt suppresses rhs coercion");
var original = Math.imul;
var crossRealmReceiver = other.Math;
crossRealmReceiver.imul = original;
assert(crossRealmReceiver.imul(3, 7) === 21, "current-realm builtin on foreign receiver");
// A native builtin remains an opaque caller for ordinary coercion callbacks.
function opaqueValueOf() { assert(opaqueValueOf.caller === null, "native caller remains opaque"); return 3; }
function caller() { return Math.imul({valueOf: opaqueValueOf}, 7); }
assert(caller() === 21, "ordinary caller boundary fallback");
true;
