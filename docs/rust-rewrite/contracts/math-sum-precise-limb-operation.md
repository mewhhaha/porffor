# Math.sumPrecise limb operation

Status: implemented and focused-verified as a source-equivalent compiler
invariant.

## Closed authority

`MathSumPreciseLimbOperation` is the private two-state authority for folding a
finite binary64 coefficient into the fixed-width exact accumulator. `Add`
belongs only to a non-negative term and `Subtract` belongs only to a negative
term. The domain has no derived capabilities: repeated projections borrow it
rather than depending on `Copy`, and it cannot be compared, formatted or
defaulted into a policy outside its exhaustive consumers.

Four exhaustive projections preserve the arithmetic and carry laws. `Add`
selects unsigned addition and carry-on-less-than for both the low addend and
incoming carry. `Subtract` selects unsigned subtraction and borrow-on-greater-
than at those same two stages. There is no wildcard, equality, Boolean or raw
instruction policy boundary.

The sign-bit branch in `emit_math_sum_precise_add_finite` is the complete
producer set: its true arm constructs `Subtract`, then its false arm constructs
`Add`. The fold retains the existing operand, result and carry instruction
order.

## Guard and verification

`math_sum_precise_limb_operation_structure.rs` recursively pins the complete
source census and bounds the exact producer and four instruction projections.
The neighboring `math_sum_precise_runtime_structure.rs` continues to own the
fixed-width accumulator, iterator and rounding invariants.

Focused verification passes the new structure target `3/3`, the neighboring
runtime structure target `6/6` and the existing Wasm-AOT runtime CLI fixture
`1/1`. Independent review confirmed the exact capability/mention closure,
adjacent-item declaration guard, globally ordered arithmetic/carry projections,
producer polarity and source equivalence. The package format and lane diff
checks are clean, and coordinated `cargo xc`, full formatter, diff,
module-boundary and task-plan checks are green. The Test262 Math tree and broad
workspace suites remain deferred; this capability closure makes no new
conformance claim.
