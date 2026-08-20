// PutValue (6.2.5.6) step 3.d: a `[[Set]]` that answered false throws a
// TypeError only when the *Reference's* [[Strict]] is true.
//
// The point of this fixture is where the write sits: top-level script code,
// inside a `try`. The Reference carries its own [[Strict]], so the failure
// guard is a runtime `If` block rather than a compile-time decision, and the
// throw reaches the active handler by Wasm label depth. A guard that opens a
// block without compensating that branch immediate lands on the wrong label or
// fails module validation. A strict write inside a *function* body cannot show
// it: an emitted function returns a completion instead of branching.
"use strict";

var frozen = Object.freeze({ x: 1 });
var frozenThrew = false;
try {
  frozen.x = 2;
} catch (e) {
  frozenThrew = e instanceof TypeError;
}

var closed = Object.preventExtensions({});
var closedThrew = false;
try {
  closed.added = 1;
} catch (e) {
  closedThrew = e instanceof TypeError;
}

frozenThrew && frozen.x === 1 && closedThrew && !("added" in closed);
