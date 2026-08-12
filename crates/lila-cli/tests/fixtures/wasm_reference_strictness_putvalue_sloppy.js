// The sloppy control for `wasm_reference_strictness_putvalue_strict.js`, byte
// for byte the same writes without the directive prologue.
//
// PutValue 6.2.5.6 step 3.d throws only when the Reference's [[Strict]] is
// true, so both writes here must be silent no-ops. This is the MC3' guard: it
// fails if the strict case was "fixed" by hardcoding a strict inhabitant on the
// IR node or by leaving the runtime guard's flag word constant.

var frozen = Object.freeze({ x: 1 });
var frozenThrew = false;
try {
  frozen.x = 2;
} catch (e) {
  frozenThrew = true;
}

var closed = Object.preventExtensions({});
var closedThrew = false;
try {
  closed.added = 1;
} catch (e) {
  closedThrew = true;
}

!frozenThrew && frozen.x === 1 && !closedThrew && !("added" in closed);
