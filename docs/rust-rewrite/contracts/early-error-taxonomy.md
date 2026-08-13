# Contract: early-error taxonomy — pointer

## 2026-08-13 duplicate-formal-parameter amendment

T07 now classifies duplicate formal-parameter rejections through one new closed
code and exactly the two case-sensitive messages emitted by the pinned Boa
parser. The normative extension is:

`docs/rust-rewrite/contracts/duplicate-formal-parameter-early-errors.md`

The current domain has **19** variants and the one parse-failure table has
**17** rows. The 18-variant and 15-row counts below describe the original
taxonomy checkpoint; they are retained as measured history, not current counts.
The extension deliberately preserves sloppy Script ordinary functions with a
simple duplicate parameter list.

## 2026-08-12 parse-once amendment

T07 supersedes the contract's old two-parser boundary. `lila-front` now owns
the only product parse and returns goal-typed parsed syntax; dependency graph
entries retain that result for both request discovery and module-record
construction. The former module-reparse prefix and its P6 assertion have been
deleted because there is no wrapper message or second classifier to constrain.
References to either below are retained only as the measured history that led
to the single-table design, not as current implementation requirements.

The contract for the area *Early-error taxonomy: one closed `EarlyErrorCode`
domain and one fragment table, not two that have already drifted* lives at:

`docs/rust-rewrite/contracts/Early-error taxonomy: one closed EarlyErrorCode domain and one fragment table, not two that have already drifted.md`

This file exists because the area brief names `early-error-taxonomy.md` in its
`files_owned` list while the campaign's file-naming convention for contracts uses
the full area title. There is one contract; that file is it.

**Read §0 of that document first.** It records seven measured corrections to the
area brief, three of which change the encoding:

- §0.1 — `EarlyErrorCode` cannot live in `lila-ir`; `lila-front` does not
  depend on it and cannot name its types. The enum, the table and the classifier
  go in a new `crates/lila-front/src/early_error_code.rs`.
- §0.2 — the original domain had **18** inhabitants, not 20.
  `E_IR_DIAGNOSTIC` names the absence of a code and `E_TEST_EARLY` is a test
  fixture. The 2026-08-13 extension adds the nineteenth real condition.
- §0.4 — the `E_DUPLICATE_LEXICAL_DECLARATION` drift is bidirectional and one
  direction is conformance-visible, not merely taxonomy-visible.

---

## DISCREPANCY-FIXER stage (dry-run findings applied)

Seven findings, all applied. The contract body carries the detail; this is the
index.

| # | Severity | What was wrong | Where it is now recorded |
|---|---|---|---|
| 1 | **blocker** | §6.14's one routed cross-lane edit was never applied, so `lila-ir` still called the deleted `IrDiagnostic::early_error` and `cargo check -p lila-ir` failed. Assertions P7-P9 had therefore never been evaluated by rustc. | §6.14 (status: APPLIED), §6.16 (token accounting re-counted) |
| 2 | bug | The derived `(LinkError, Resolution, SyntaxError)` triple reached nothing: the engine short-circuited only on `kind == EarlyError`, and `resolution` negatives never entered the compile-only path. All 34 `resolution/SyntaxError` cases failed on wasm-aot. | ledger **L10** |
| 3 | bug | MC4 was discharged for `lila-ir` only. A link-only code could be named at a parse-stage producer in *both* crates. | §2.4.1 `ParseClassified`, MC-table status note |
| 4 | bug | `ParseCode::error_type` claimed `"SyntaxError"` for the caught-parser-abort case, scoring false test262 passes for `parse/SyntaxError` negatives whose source merely crashed boa. | ledger **L11** |
| 5 | polish | §0.5's reachability claim for row 6 (W5) is false; the row is dead through both entry points. | §0.5 (rewritten), ledger **L5** (first confirmed instance) |
| 6 | polish | The string oracle is injectable by user source text; L1 covered only boa *rewording*. | ledger **L1** (extended), `INTERPOLATING_MESSAGE_SHAPES` + assertion **P10** |
| 7 | polish | Two holes in the const-assertion set: P1 did not reject an empty *fragment string*, and nothing kept a `wire_name()` from colliding with the absence placeholder. | §2.5 (P1 restated, **P5'** added) |

Also closed in that historical implementation: **L6** (the old dependency-path
prefix existed in three copies) and half of **L8**. T07's parse-once amendment
subsequently removed the prefix and its assertion altogether.

Two behaviour changes need a pass-count report at §8 rung 6, reported
separately from B1/B2: the `resolution` family (L10) and `parse/SyntaxError`
negatives that were passing on a caught parser abort (L11 — an *expected*
pass-count decrease, since those passes were false).

---

## INTEGRATOR stage (compile gate)

Nothing was left outstanding by the lane note. The one routed cross-lane edit
(§6.14, `lowering.rs`'s call to the deleted `IrDiagnostic::early_error`) had
already been applied by the discrepancy-fixer, and the tree confirms it:
`grep -rn "IrDiagnostic::early_error" crates/` returns **0**, and the duplicate
`__proto__` producer now goes through `IrDiagnostic::rejected(EarlyErrorCode::ObjectDuplicateProto, …)`.

The value of this stage is therefore that the const-assertion set was **actually
evaluated by rustc for the first time**, which is what the whole encoding is for:

- `cargo check -p lila-front` — clean at that checkpoint. **P1–P6, P5′ and P10
  passed**: no empty
  fragment or fragment string, no `wire_name()` colliding with
  `NO_EARLY_ERROR_CODE`, the then-15-row `PARSE_FAILURE_RULE_TABLE` matched
  `PARSE_FAILURE_RULE_COUNT`, every row's witnesses are matched by that row's own
  fragments, and `INTERPOLATING_MESSAGE_SHAPES` eats no witness.
- `cargo check -p lila-ir` — clean at that checkpoint. **P7–P9 passed**: every
  code `rejection_kind` can name is `is_parse_classified`. The old prefix half
  of this statement was retired by the parse-once amendment.
- `cargo xc` — 0 errors, no new warnings.

Two of the note's §4.3 uncertainties are now resolved by the compiler rather
than by argument: `const PARSE_FAILURE_RULES: &[ParseFailureRule] = &PARSE_FAILURE_RULE_TABLE;`
is accepted, and slice indexing inside a `const fn` is accepted. Neither
fallback in §4.3 is needed.

Unchanged and still open: the two behaviour changes (L10, the `resolution`
family; L11, the `parse/SyntaxError` negatives that were passing on a caught
parser abort) need a rung-6 pass-count report, which is a conformance run and so
belongs elsewhere. L11 predicts a pass-count **decrease** and that decrease is
correct — those passes were false.
