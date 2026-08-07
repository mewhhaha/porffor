// A hoisted function declaration is lowered before the top-level `const` that
// it captures, so at capture time the binding can still hold its hoist-time
// TDZ placeholder (kind `Undefined`). Publishing that placeholder as the
// capture's proven kind makes `signature.return_kind` claim `fb()` is
// `undefined`, which then constant-folds `typeof fb()` to the literal string
// "undefined" without ever calling `fb`.
//
// Both values below are plain observable JavaScript, so a regression shows up
// as a changed string rather than as a crash.
//
// NOTE: `fb() + 1`, `fb() | 0`, `"q" in fb()` and `var { q } = fb()` are all
// still wrong for a const-captured object (they read the heap handle as a
// Number, throw, or bind the wrong value). That is a separate, older defect in
// how a precisely-typed Object return kind selects operators; it reproduces
// without any hoisting, e.g. `const B = {}; const fb = function () { return B; };`.
// It is deliberately not asserted here so this test keeps its narrow subject.
const B = { q: 1 };

function fb() {
  return B;
}

print("const-capture-return-kind:" + typeof fb() + ":" + fb().q);
