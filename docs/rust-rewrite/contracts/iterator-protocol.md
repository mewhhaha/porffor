# Iterator-protocol obligation contract

This area's contract lives in one document, because the iterator obligations'
`EmissionSite` values are what witness the spec-operation catalog's
`StatementEmission` rows and the two would drift if split:

**[Spec-operation catalog evidence and the iterator-protocol obligation witness](./Spec-operation%20catalog%20evidence%20and%20the%20iterator-protocol%20obligation%20witness.md)**

Start at §1.2–§1.4 (spec basis) and §4 (type mapping, Part B) for the iterator
half. §9 holds the dry-run corpus and the three corrections to the area brief.

As built: `crates/lila-ir/src/iterator_obligations.rs`, with the
`EmissionSite` → real-function join in
`crates/lila-aot-wasm/src/emission_sites.rs`. §12 (encoder addendum) records
the deviations; D3 and D4 are the two that touch this half.

**§13 is the dry-run discrepancy pass and supersedes §§1–12 where they
disagree.** For this half, read 13.1 (the integration was not applied and now
is), 13.4 (the slot transposition was not actually `E0308`; the allocators fix
it), 13.5 (there was already a fourth for-of specialization, so the witness is
attached to the head lowering), 13.6 (`IntactnessPremise` conflated three kinds
of claim), 13.7 (a partial intactness guard exists and is not consulted), 13.9
(`EmissionSite` is a set; L6 retired) and 13.12 (the "emitter must not read
this" rule is now `pub(crate)`).
