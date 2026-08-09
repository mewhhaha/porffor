# Iterator-protocol obligation contract

This area's contract lives in one document, because the iterator obligations'
`EmissionSite` values are what witness the spec-operation catalog's
`StatementEmission` rows and the two would drift if split:

**[Spec-operation catalog evidence and the iterator-protocol obligation witness](./Spec-operation%20catalog%20evidence%20and%20the%20iterator-protocol%20obligation%20witness.md)**

Start at §1.2–§1.4 (spec basis) and §4 (type mapping, Part B) for the iterator
half. §9 holds the dry-run corpus and the three corrections to the area brief.

As built: `crates/porffor-ir/src/iterator_obligations.rs`, with the
`EmissionSite` → real-function join in
`crates/porffor-aot-wasm/src/emission_sites.rs`. §12 (encoder addendum) records
the deviations; D3 and D4 are the two that touch this half.
