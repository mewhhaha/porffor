// A property read that throws, inside a `for`, inside a `try`.
//
// The read misses on `a` and finds a getter on `Object.prototype`, so it takes
// the prototype-walk read path in `objects.rs`. That path opens raw Wasm
// `If`/`Block` frames of its own, and the branch that propagates the getter's
// throw to the enclosing `try` used to be computed from the *tracked* frame
// depth plus a hand-counted correction. Inside a loop the correction was short,
// so the `br` landed on the loop's back edge instead of the handler: the body
// re-ran, threw again, branched to the back edge again, and the program spun
// until it trapped (~560,812 iterations of the body when this was measured).
//
// Two iterations is deliberate. Correct behaviour prints `iteration` exactly
// once and then `caught TypeError`; a regression prints `iteration` without
// bound. The bound in the loop header keeps a *correct* run cheap; it cannot
// bound a broken one, because the back edge never re-evaluates `j++`.

Object.defineProperty(Object.prototype, "zzz", {
  get: function () {
    throw new TypeError("thrown from a prototype accessor");
  },
  configurable: true
});

var a = {};
var caught = "none";

try {
  for (var j = 0; j < 2; j++) {
    print("iteration");
    var v = a.zzz;
    print("after read");
  }
} catch (e) {
  caught = e && e.name;
  print("caught " + caught);
}

print("end");
