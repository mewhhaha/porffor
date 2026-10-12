# Labelled-break iterator closing

The workspace pins Boa 0.21.1 as the spec-exec oracle. The local repair in
`src/bytecompiler/statement/break.rs` closes each skipped inner iterator loop
when a labelled break transfers to an outer loop or block. The target loop
retains its own close operation. This follows the pinned ECMA-262
ForIn/OfBodyEvaluation IteratorClose rule and mirrors the existing continue
action handling without changing the dependency version.

Strict/sloppy spec-exec controls cover local and outer breaks, outer continue,
finally, throwing close identity and asynchronous for-await closing. The
[exact repair evidence](../../docs/rust-rewrite/generator-oracle-repair-20261012.json)
retains the original differential requests and all subsequent verdicts. These
selected checks do not establish full upstream Boa or Test262 acceptance.
