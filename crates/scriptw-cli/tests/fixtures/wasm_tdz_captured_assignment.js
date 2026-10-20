// A plain assignment to a captured binding that is still in TDZ must throw
// a ReferenceError (9.1.1.1.5 step 3). Captured bindings are seeded
// `Initialized` in the lowering scope, so the lowerer cannot reject the
// write statically; the slot tag is the runtime witness. Assignment used to
// write the slot unchecked while reads threw.
function assertReferenceError(fn, label) {
  try {
    fn();
  } catch (e) {
    if (e instanceof ReferenceError) return;
    throw label + ": threw " + Object.prototype.toString.call(e) + " " + e;
  }
  throw label + ": did not throw";
}

assertReferenceError(function () {
  (function () {
    function f() { x = 1; }
    f();
    let x;
  })();
}, "closure plain assignment");

assertReferenceError(function () {
  (function () {
    function f() { [x] = [1]; }
    f();
    let x;
  })();
}, "closure destructuring assignment");

assertReferenceError(function () {
  (function () {
    function f() { for (x in { a: 1 }) {} }
    f();
    let x;
  })();
}, "closure for-in head assignment");

assertReferenceError(function () {
  (function () {
    function f() { for (x of [1]) {} }
    f();
    let x;
  })();
}, "closure for-of head assignment");

assertReferenceError(function () {
  (function () {
    function* g() { x = yield 1; }
    var it = g();
    it.next();
    it.next(7);
    let x;
  })();
}, "generator resumed assignment");

// No async-resume TDZ case: resumption always post-dates the synchronous
// `let`, so the slot is initialized by resume time. The generator case above
// covers the shared resumed-assignment path.

// Positive controls: the same writes past initialization land normally,
// including through every shape above.
(function () {
  let x = 0;
  function f() { x = 1; }
  f();
  if (x !== 1) throw "control plain: " + x;
  function d() { [x] = [2]; }
  d();
  if (x !== 2) throw "control destructure: " + x;
  function l() { for (x in { a: 0 }) {} }
  l();
  if (x !== "a") throw "control for-in: " + x;
  function o() { for (x of [3]) {} }
  o();
  if (x !== 3) throw "control for-of: " + x;
  function* g() { x = yield 1; }
  var it = g();
  it.next();
  it.next(4);
  if (x !== 4) throw "control generator: " + x;
})();

350;
