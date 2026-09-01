# Obsolete static-generator cache removal

Status: implemented as a source-equivalent T02 state-invariant closure.

The IR lowerer carried `static_generator_sum_values` and
`static_generator_element_values` through its root state, conditional-flow
snapshots, heap-shape visits and invalidation path. Both maps were constructed
empty and had no producer: the complete source census contained reads,
clones, intersections and clears, but no insertion or replacement with a
non-empty map. They were therefore empty for every reachable lowerer state.

The fields and their projections are deleted rather than documenting an
always-empty convention. Direct-call, assignment, declaration, initializer
and for-of branches that consulted the maps are deleted with them. Ordinary
expression lowering now appears directly in those callers, matching the only
branch they could previously enter.

The live generator and iterator authorities remain separate and explicit:
generator-expression call overrides, numeric generator-declaration parsing,
object-iterator literal and IIFE folding, iterator-binding values and the
array-literal result fold are unchanged. The IIFE fold still recognizes a
zero-argument generator-declaration call; its sole call-shape parser is now
inline at that use instead of surviving as a one-caller abstraction.

The complete original 29-line cache-identifier census has SHA-256
`8043d5ff10f4b61f90d5caea850ee1f648d81a7c5bfd413715fd1776194bd27c`.
The deleted no-op declaration preparation and always-false declaration probe
have SHA-256
`51ca4e5119307e3df723701e54632dc8f37cfe0f231ea0bd6401c10e7d1bd0d2`.
The deleted cache-backed call projection island has SHA-256
`1f6bb5a929cb2250a07ba4d1deb96379788633d9da460d1e95e43c9d61360c1e`.
The deleted array-iterator construction island has SHA-256
`455ea8b701e57fe6497169d2cfcf94bae1f804a6f21e37c7f47e044ea3eba1bb`.
The deleted assignment specialization has SHA-256
`8291e01437badcde47d9d2d412b4b68fb4fb6025daa5d26459ab55b70b6dae79`.

This state-invariant closure has no new JavaScript behavior and changes no
reachable lowered IR: every removed decision was guarded by a map that could
only be empty. It adds no Test262 materialization, capability claim or
published count.

At the Batch BY checkpoint, `cargo check -p lila-ir` is green without new
project warnings, the focused absence target passes `3/3`, and the retained
iterator-fallback and generator-suspension units pass `3/3`.

## Backend closure

The cache deletion also removed the only ordinary IR producer for the private
`$LilaStaticGenerator.values` call shape. The remaining Wasm backend protocol
is therefore deleted in full: its two IR names, unconditional string-table
entries, special array-iterator method variant, marker writer, marker reader
and all three iterator-close consumers.

This follow-on is a correctness closure rather than a behavior-neutral state
cleanup. The marker reader treated `$LilaStaticGeneratorIterator` as an
ordinary own data property on every iterator-close path. Source JavaScript
could forge that property on an Array iterator and make an empty destructuring
close set the private `$ArrayIterator.done` slot. Both retired spellings now
remain ordinary source properties and have no compiler meaning.

The backend absence structure target inventories every former producer and
consumer file. The Wasm CLI regression forges the old marker, performs an empty
destructuring close, and proves that the iterator is not exhausted; it also
calls an ordinary source method whose key is the old synthetic method spelling.

At the Batch BZ checkpoint, `cargo check -p lila-aot-wasm` is green with only
the pre-existing vendored parser warning. The iterator-kind and backend-absence
structure targets pass `6/6`; the new source-property regression plus the
ordinary Array iterator-policy and Iterator.zip controls pass `3/3`. Formatting,
module-boundary, task-plan, exact-shortcut and diff checks are green, with the
shortcut inventory unchanged at 240 entries.
