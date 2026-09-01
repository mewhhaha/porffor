# Array `copyWithin` traversal direction authority

Status: implemented and dry-reviewed for the Wasm-AOT
`Array.prototype.copyWithin` compiler on 2026-08-28. Focused structure
verification is recorded below; broader Array execution remains deferred.

## Closed direction domain

The compiler emits one of two legal traversal starts:

- `ArrayCopyWithinDirection::Forward` retains the normalized source and target
  cursors and publishes a `+1` step; and
- `ArrayCopyWithinDirection::Backward` rewinds both cursors by `count - 1`
  and publishes a `-1` step.

The enum is private to the Array builtin module and derives no capabilities.
Its sole consumer takes it by value and projects it through one exhaustive
match. The direction cannot be compared, copied, defaulted or converted to a
Boolean. Adding another direction therefore fails to compile until its cursor
start and step are stated together.

The emitted Wasm still carries the selected step in a local because overlap is
decided at runtime. That raw local has no direct writer in the `copyWithin`
compiler body: both legal values are published only by the typed projection.

## Producer boundary

`compile_array_prototype_copy_within_builtin` has exactly two producers:

1. `Forward` initializes the default traversal after the source, target and
   count have been computed; and
2. `Backward` appears only inside the nested overlap check where
   `from < to < from + count`.

This keeps the overlap predicate independent from the direction mechanics
while preventing the cursor rewind and negative step from drifting into
separate decisions. Argument coercion, clamping, property observation,
TypedArray borrowing, deletion, writes and result identity are unchanged.

## Durable witness

`crates/lila-aot-wasm/tests/array_copy_within_direction_structure.rs` bounds
the dedicated direction module and the complete `copyWithin` compiler body.
It pins:

- the exact two-case, capability-free domain;
- one exhaustive projection and no catch-all or equality escape hatch;
- the complete forward and backward instruction sequences;
- the sole two writes to the direction local inside that projection;
- one forward producer before the overlap guard and one backward producer
  inside it; and
- no direct direction-local write in the compiler body.

The focused structure target passes `3/3`. Direct `rustfmt --check` for the
changed Rust files and the scoped diff check are also green. The structure
target is the owner witness for this source-equivalent invariant; it does not
replace behavioral Array tests.

## Nonclaims

This slice does not alter `copyWithin` observable behavior, fix another Array
method, remove a Test262 materializer or change published conformance counts.
It does not claim complete sparse, Proxy, TypedArray-borrowed or abrupt
`copyWithin` coverage, and it does not close T16.
