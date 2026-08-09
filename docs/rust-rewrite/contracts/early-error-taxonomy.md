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

---

## DISCREPANCY-FIXER stage (dry-run findings applied)

Seven findings, all applied. The contract body carries the detail; this is the
index.

| # | Severity | What was wrong | Where it is now recorded |
|---|---|---|---|
| 1 | **blocker** | §6.14's one routed cross-lane edit was never applied, so `porffor-ir` still called the deleted `IrDiagnostic::early_error` and `cargo check -p porffor-ir` failed. Assertions P7-P9 had therefore never been evaluated by rustc. | §6.14 (status: APPLIED), §6.16 (token accounting re-counted) |
| 2 | bug | The derived `(LinkError, Resolution, SyntaxError)` triple reached nothing: the engine short-circuited only on `kind == EarlyError`, and `resolution` negatives never entered the compile-only path. All 34 `resolution/SyntaxError` cases failed on wasm-aot. | ledger **L10** |
| 3 | bug | MC4 was discharged for `porffor-ir` only. A link-only code could be named at a parse-stage producer in *both* crates. | §2.4.1 `ParseClassified`, MC-table status note |
| 4 | bug | `ParseCode::error_type` claimed `"SyntaxError"` for the caught-parser-abort case, scoring false test262 passes for `parse/SyntaxError` negatives whose source merely crashed boa. | ledger **L11** |
| 5 | polish | §0.5's reachability claim for row 6 (W5) is false; the row is dead through both entry points. | §0.5 (rewritten), ledger **L5** (first confirmed instance) |
| 6 | polish | The string oracle is injectable by user source text; L1 covered only boa *rewording*. | ledger **L1** (extended), `INTERPOLATING_MESSAGE_SHAPES` + assertion **P10** |
| 7 | polish | Two holes in the const-assertion set: P1 did not reject an empty *fragment string*, and nothing kept a `wire_name()` from colliding with the absence placeholder. | §2.5 (P1 restated, **P5'** added) |

Also closed in passing: **L6** (the reparse prefix existed in three copies) and
half of **L8** (`MODULE_REPARSE_PREFIX` is now `pub`, having acquired the second
consumer L8 named as the condition for promotion).

Two behaviour changes need a pass-count report at §8 rung 6, reported
separately from B1/B2: the `resolution` family (L10) and `parse/SyntaxError`
negatives that were passing on a caught parser abort (L11 — an *expected*
pass-count decrease, since those passes were false).
