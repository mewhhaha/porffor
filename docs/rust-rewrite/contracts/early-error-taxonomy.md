# Contract: early-error taxonomy — pointer

The contract for the area *Early-error taxonomy: one closed `EarlyErrorCode`
domain and one fragment table, not two that have already drifted* lives at:

`docs/rust-rewrite/contracts/Early-error taxonomy: one closed EarlyErrorCode domain and one fragment table, not two that have already drifted.md`

This file exists because the area brief names `early-error-taxonomy.md` in its
`files_owned` list while the campaign's file-naming convention for contracts uses
the full area title. There is one contract; that file is it.

**Read §0 of that document first.** It records seven measured corrections to the
area brief, three of which change the encoding:

- §0.1 — `EarlyErrorCode` cannot live in `porffor-ir`; `porffor-front` does not
  depend on it and cannot name its types. The enum, the table and the classifier
  go in a new `crates/porffor-front/src/early_error_code.rs`.
- §0.2 — the domain has **18** inhabitants, not 20. `E_IR_DIAGNOSTIC` names the
  absence of a code and `E_TEST_EARLY` is a test fixture.
- §0.4 — the `E_DUPLICATE_LEXICAL_DECLARATION` drift is bidirectional and one
  direction is conformance-visible, not merely taxonomy-visible.
