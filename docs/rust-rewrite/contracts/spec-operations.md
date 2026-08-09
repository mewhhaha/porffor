# Spec-operation catalog contract

This area's contract lives in one document, because the catalog's
`StatementEmission` rows are witnessed by the iterator contract's `EmissionSite`
values and the two would drift if split:

**[Spec-operation catalog evidence and the iterator-protocol obligation witness](./Spec-operation%20catalog%20evidence%20and%20the%20iterator-protocol%20obligation%20witness.md)**

Start at §3 (type mapping, Part A) for the catalog half.

As built: `crates/porffor-ir/src/operations.rs`. §12 (encoder addendum) records
the four deviations, the three added mistake classes, and ledger entries L6–L8.
