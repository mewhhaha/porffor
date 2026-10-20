// A bare `for (x in …)` head PutValues the key into `x` every iteration, so
// the head is a capture of the outer binding. The free-reference scanner
// used to skip for-in bare-identifier and member heads (for-of scanned
// both), and the write lowered to an implicit global instead of the
// captured binding.
(function () {
  let x = 0;
  function f() { for (x in { a: 1 }) {} }
  f();
  if (x !== "a") throw "bare head missed the outer binding: " + x;
  if ("x" in globalThis) throw "bare head leaked to the global object";
})();

(function () {
  let o = {};
  function f() { for (o.k in { a: 1 }) {} }
  f();
  if (o.k !== "a") throw "member head missed the captured base: " + o.k;
})();

(function () {
  let y = 0;
  function f() { for (y of [5]) {} }
  f();
  if (y !== 5) throw "for-of control: " + y;
})();

351;
