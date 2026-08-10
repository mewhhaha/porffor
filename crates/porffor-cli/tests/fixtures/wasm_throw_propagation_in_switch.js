// The same throwing property read as `wasm_throw_propagation_in_loop.js`, but
// inside a `switch` rather than a `for`.
//
// The failure mode is the mirror image of the loop's. A `switch` lowers to a
// `block` per case plus a surrounding breakable `block`, so a branch that is
// one label too shallow lands on a `block` that simply ends — the throw is
// discarded and execution continues past the `switch` as if nothing had
// happened. Nothing traps, nothing spins, and the program prints `end` with a
// completed `TypeError` silently dropped, which is why this shape survived
// every probe that only checked for a crash.
//
// Correct behaviour: `caught TypeError` is printed and `after read` is not.

Object.defineProperty(Object.prototype, "zzz", {
  get: function () {
    throw new TypeError("thrown from a prototype accessor");
  },
  configurable: true
});

var a = {};
var caught = "none";

try {
  switch (1) {
    case 1:
      var v = a.zzz;
      print("after read");
      break;
    default:
      print("default");
  }
} catch (e) {
  caught = e && e.name;
  print("caught " + caught);
}

print("end");
