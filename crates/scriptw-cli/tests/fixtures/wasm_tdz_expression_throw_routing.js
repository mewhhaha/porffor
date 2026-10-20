// A `RuntimeThrow` inside an expression must route its Throw completion to
// the active handler immediately. It used to fall through: the enclosing
// expression then overwrote the thrown value (`let x = !x` threw `false`,
// `typeof x` threw `"object"`, `x ? 1 : 2` threw `1`) or cleared the
// completion and continued silently (`let x = x + 1` never threw).
function assertReferenceError(fn, label) {
  try {
    fn();
  } catch (e) {
    if (e instanceof ReferenceError) return;
    throw label + ": threw " + Object.prototype.toString.call(e) + " " + e;
  }
  throw label + ": did not throw";
}

assertReferenceError(function () { (function () { let x = x + 1; })(); }, "self-init add");
assertReferenceError(function () { (function () { let x = x - 1; })(); }, "self-init sub");
assertReferenceError(function () { (function () { let x = x * 2; })(); }, "self-init mul");
assertReferenceError(function () { (function () { let x = x == 1; })(); }, "self-init eq");
assertReferenceError(function () { (function () { let x = !x; })(); }, "self-init not");
assertReferenceError(function () { (function () { let x = -x; })(); }, "self-init neg");
assertReferenceError(function () { (function () { let x = typeof x; })(); }, "self-init typeof");
assertReferenceError(function () { (function () { let x = x ? 1 : 2; })(); }, "self-init cond");
assertReferenceError(function () { (function () { let y = x + 1; let x; })(); }, "later add");
assertReferenceError(function () { (function () { let y = !x; let x; })(); }, "later not");

// Positive control: the same shapes past initialization evaluate normally.
(function () {
  let x = 40;
  let y = x + 1;
  if (y !== 41) throw "control add: " + y;
  if (!x !== false) throw "control not";
  if ((x ? 1 : 2) !== 1) throw "control cond";
})();

349;
