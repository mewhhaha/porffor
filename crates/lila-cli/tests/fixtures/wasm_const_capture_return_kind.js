// A hoisted function declaration is lowered before the top-level `const` that
// it captures, so at capture time the binding can still hold its hoist-time
// TDZ placeholder (kind `Undefined`). Publishing that placeholder as the
// capture's proven kind makes `signature.return_kind` claim `fb()` is
// `undefined`, which then constant-folds `typeof fb()` to the literal string
// "undefined" without ever calling `fb`.
//
// Initializers outside the static inferencer and destructured bindings must
// likewise retain their runtime kinds when captured by hoisted functions.
// The unknown initializer can return a callable, and hoist-time metadata must
// not turn a later const's TDZ into an initialized value.
//
// Object coercion, `in`, and destructuring exercise separate lowering paths;
// this regression checks the capture's kind, callable target, and TDZ lifecycle.
const B = { q: 1 };

function fb() {
  return B;
}

function makeObject() {
  return { r: 2 };
}
const C = makeObject();
function fc() {
  return C;
}

const { d } = { d: 3 };
function fd() {
  return d;
}

function makeCallable() {
  return function () { return 7; };
}
const F = makeCallable();
function ff() {
  return F;
}

let tdzCaught = false;
try {
  ft();
} catch (error) {
  tdzCaught = error instanceof ReferenceError;
}
const T = 11;
function ft() {
  return T;
}

let E;
function fe() {
  return E;
}

print("const-capture-return-kind:" + typeof fb() + ":" + fb().q + ":" +
      typeof fc() + ":" + fc().r + ":" + fd() + ":" +
      typeof ff() + ":" + ff()() + ":" + tdzCaught + ":" + ft() +
      ":" + typeof fe());
