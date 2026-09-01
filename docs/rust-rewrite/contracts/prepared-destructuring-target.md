# Prepared destructuring target

`PreparedDestructuringTarget` is the private, must-use, capability-free
six-variant boundary between target evaluation and the later destructuring
write. Its `Binding`, `AssignmentIdentifier`, `Property`, `Private`,
`NestedArray` and `NestedObject` variants mirror the closed
`DestructuringTargetIr` domain and carry exactly the values each write needs.

Preparation matches the IR target exhaustively. Property and private targets
evaluate and retain their receiver locals before the source value or default
initializer is observed; computed property targets also retain the evaluated
key locals. The remaining variants borrow their already-lowered IR facts.
There is no clone, wildcard or fallback variant.

The write consumes only `PreparedDestructuringTarget`. It no longer receives a
parallel IR discriminant and then assumes with `unreachable!` that the two
independently supplied variants agree. One exhaustive match now owns binding,
identifier Reference, property, private, nested-array and nested-object writes.
Adding a target kind therefore fails to compile at both preparation and write,
while constructing a mismatched prepared/IR pair is no longer expressible.

The property sub-variant retains the separate must-use
`PreparedDestructuringPropertyKey::{Static, Computed}` domain. Its exhaustive
write projection releases computed-key locals exactly once after PutValue;
static keys own no temporary locals.

Batch AD changes no evaluation, abrupt-completion, IteratorClose, Reference
strictness or temporary-local order. The recursive four-test guard pins the
six-variant mirror, sole exhaustive producer and consumer, absence of the
parallel discriminant and focused direct, property, nested and private runtime
witnesses. `cargo xc` passes. The prepared-target and neighboring iterator-step
structure guards pass `8/8`; the array-iterator, rest-setter-after-completion
and private-reference-order CLI witnesses pass `3/3`. No Test262 cohort or
semantic golden was run for this source-equivalent invariant.
