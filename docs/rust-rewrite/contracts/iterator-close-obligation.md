# IteratorClose obligation contract (round 2)

The contract lives in one document:

**[IteratorClose as an obligation stated where the iterator is acquired: witness coverage for the four uncovered IR constructs, plus a consumer for the catalog's unread abrupt column](./IteratorClose%20as%20an%20obligation%20stated%20where%20the%20iterator%20is%20acquired%3A%20witness%20coverage%20for%20the%20four%20uncovered%20IR%20constructs%2C%20plus%20a%20consumer%20for%20the%20catalog%27s%20unread%20abrupt%20column.md)**

It extends round 1's
[Spec-operation catalog evidence and the iterator-protocol obligation witness](./Spec-operation%20catalog%20evidence%20and%20the%20iterator-protocol%20obligation%20witness.md)
(see also the short redirect [iterator-protocol.md](./iterator-protocol.md)),
whose §13 amendments and L1–L8 ledger remain in force. Round 2's ledger rows
carry an `IC` prefix so the two cannot be confused.

**Read §0 first.** It carries six measured corrections to the area brief, two of
which change what gets built:

- **C1** — `ExprIr::ArrayLiteral` never contains a spread element; array-literal
  spread is desugared to `[].concat(…)` / `Array.from(…)` before the node
  exists. A `protocol` field there would be decoration *and* a false claim. §5
  designs the replacement (`ArraySpreadStrategy`) and note-routes it.
- **C6** — the `EmissionSite::ArrayDestructuring` catalog row is **true**, not a
  lie: `compile_array_destructure_from_value_locals` really emits all four
  obligations, both halves of 7.4.11 step 4 included (§1.6, and the
  `array-elem-iter-nrml-close-skip.js` trace at §9.5). The repair is to *add*
  `ARRAY_DESTRUCTURING_PROTOCOL`, not to delete the row.

The original work was split into three groups; Group A and both Group B seams
are now encoded, while Group C remains open:

- **Group A (§3)** — everything that leaves `cargo xc` green with no edit
  outside `crates/lila-ir`: the `emission_sites!` row list,
  `ARRAY_DESTRUCTURING_PROTOCOL` and the site↔witness↔catalog triangle
  (K1/J10/J11), the `protocol` field on `ArrayDestructuringPatternIr` (two
  `E0063`s, both in `lowering.rs`), and two `const`-evaluated readers for the
  `abrupt` column (`AbruptDiscipline` + callee containment, J12/J13).
- **Group B (§4)** — two independent witness seams. `YieldForm` now replaces
  `StatementIr::GeneratorYield`'s `delegate: bool` and carries the
  one-inhabitant `GeneratorDelegationProtocol` (integrated 2026-08-12; see the
  main contract's §13). `SpreadArgumentIr` now likewise requires the
  one-inhabitant `SpreadArgumentProtocol`; its witness credits the real
  argument-vector emitter for acquisition, step and value only, and makes the
  absence of an `IteratorClose` claim explicit (integrated 2026-08-12; §14).
- **Group C (§5)** — the array-literal spread strategy remains unencoded. The
  honest closed target is `ArraySpreadStrategy::{ProvenDense,
  GeneralIterator}`, but the current lowerer has no realm/version proof that
  can construct `ProvenDense`: a known dense Array must still observe a patched
  `%Array.prototype%[@@iterator]`. §14 records why an uninhabited fast-path type
  would be decoration rather than an invariant.

§10's prohibitions are the load-bearing part for anyone extending this: P1 (no
`lila-aot-wasm` edit), P2 (round 1's `pub(crate)` narrowing at
`iterator_obligations.rs:45-51` is not re-opened — the emitter-side close token
is a sibling type in the backend crate, never a witness reader), P5 (`lowering.rs`
gets exactly two lines).

§9.15 carries the paper trace that killed the brief's proposed
`#[must_use]`-consumed-by-value close token — it fails at
`objects.rs:14374`, where one emitter has two correct exits sharing one
iterator — and the scope-shaped design that replaces it.

---

## As built (encoder stage, Group A)

Five files changed, all under `crates/lila-ir/src/`, plus this document and
the lane note. Acceptance item §11.7 is "no hunk in `crates/lila-aot-wasm/` is
attributable to this contract" — **not** "`git status` is empty there". The
checkout is shared with concurrent lanes, and a bare `git status` reads as a
false negative.

| File | What landed |
|---|---|
| `iterator_obligations.rs` | `emission_sites!` (enum + `ALL` + `name` from one row list), `ARRAY_DESTRUCTURING_PROTOCOL`, `ALL_OBLIGATIONS` promoted out of `#[cfg(test)]`, `ALL_WITNESSES`, `site_is_witnessed`, const asserts **K1/K2/K3/K4**, eleven repaired citations |
| `operations.rs` | `AbruptDiscipline`, the `discipline` and `calls` columns on `StatementEmissionRow` and their five row values, const asserts **J10/J11/J12/J13**, the `SYNC_PROTOCOL_SITES` citation repair |
| `ir.rs` | `ArrayDestructuringPatternIr::protocol` — required, no `Default` |
| `lowering.rs` | the two construction sites `lower_array_binding_pattern` and `lower_array_assignment_pattern` name `ARRAY_DESTRUCTURING_PROTOCOL`. Nothing else. |
| `lib.rs` | `AbruptDiscipline` added inside the pre-existing `pub use operations::{…}` block (round 4 adds `ArrayPatternProtocol` to the `iterator_obligations` block). No new `mod` line. |

Two deletions, because a runtime check that survives beside the compile-time
check it duplicates is evidence the compile-time one is decoration:

- the `#[cfg(test)]`-local `ALL_OBLIGATIONS` copy in `iterator_obligations.rs`,
  replaced by the module-level `pub(crate)` const that `site_is_witnessed`
  consumes on the product path;
- the test `iterator_protocol_witnesses_emit_every_obligation`, whose two
  hand-listed pairs asserted at test time exactly what the
  `emits_every_obligation` const assertions (now three, with K2) assert at
  `cargo check` time — and which under-covered, because the third witness was
  not in its list.

Three deviations from the contract's letter, all recorded in the lane note's
self-review: `emission_sites!` accepts `#[$meta]` per row so the per-variant
evidence prose stays on the variants (§3 A1 assumed it could not); the six
stale-citation repairs of §2.4 were extended to five further stale citations
found in the same doc comments while repairing them; and one const assert **K4**
was added beyond the contract's list, because deleting the redundant runtime
test left `EmissionSite::name` with no reader and the honest repair turned out
to be a real invariant — two sites may not name one emitter function
(mistake class **M3d** in the note).

## Round-4 discrepancy fixes (applied)

Eleven findings from the round-4 dry run were applied. The contract above is
amended in place; this is the index.

| # | Severity | What changed |
|---|---|---|
| J13's claim was empty for a weakened callee | **blocker** | J13 gains a justification clause: a row claiming an abrupt exit must name a callee that can produce one. Containment alone never fires when `Get` is set to `NO_ABRUPT`, because the callee slice goes empty and the loop body never runs. M4b's "after this contract" cell and checklist item 6 are now true. |
| `ALL_WITNESSES` was a hand-maintained census (IC-4) | bug | `iterator_witnesses!` generates the constants **and** the census from one row list. **K3 retired.** IC-4 is closed by a type, and its stated reason ("not rows a macro can expand twice") is corrected as wrong. |
| The `protocol` field's type was the whole witness domain (M1a) | bug | `ArrayPatternProtocol` — one inhabitant, private constructor — is the field's type. Any other witness is `E0308`. K2 now asks its question through `ArrayPatternProtocol::ARRAY_DESTRUCTURING.witness()`, which also gives that accessor a const consumer. The same hole on the three `ForOf*` fields is recorded as **IC-7**. |
| J10 asked "witnessed for *some* obligation" | bug | `StatementEmissionRow` gains `pub obligation: IteratorObligation`; `site_emits(site, obligation)` replaces `site_is_witnessed` at J10. B1's `SYNC_CLOSE_SITES` split becomes a build-time constraint instead of a convention. |
| `SpreadLoopExitsOnlyWhenDone` was false at two lines | bug | The §9.11 read is complete. Two abrupt exits (`Get(iterator,"next")` and the not-callable TypeError) leave a non-done iterator; the conclusion survives because both are *inside* GetIterator. Renamed `SpreadCloseOwedOnlyAfterAcquisition` with the reason that is true. |
| IC-5 called a falsified premise "documentation" | bug | `lower_array_literal`'s spread guard narrows from `possible_kinds.contains(Array)` to `is_subset_of({Array})`. `function f(x) { return [...x]; }` was lowering to `[].concat(x)` and appending a non-array iterable instead of iterating it — wrong under a pristine realm. **This is the one emitted-byte change in the batch.** |
| `into_entry` was `pub` on an all-`pub`-fields struct | polish | Both `into_entry`s are `pub(crate)`; the FORGED-row hole the doc comment claimed to close is now actually closed. Narrowing the structs themselves is **IC-8**. |
| `AbruptDiscipline::name` had zero callers | polish | A `const` distinctness assertion over `AbruptDiscipline::ALL`, K4's treatment applied to this area's own type. |
| `ForOfLoweringIr::protocol()` had zero callers | polish | Deleted; `into_statement_and_kind` reads the witness and `debug_assert`s two real conditions instead. |
| The `CloseOnAbruptExitWithStep4Precedence` doc over-claimed | polish | Reworded to name the two helpers and the two completion classes without asserting every site exercises the break/return branch — `compile_array_destructure_from_value_locals` does not. |
| J12(b) was `abrupt.len() == 4` | polish | A membership scan over `CompletionAbruptKind::ALL` (new, with a bitmask `const _`), so `&[Throw, Return, Break, Break]` no longer passes. `CONTROL_COMPLETIONS` is defined *from* `ALL`. |
| Stale citations; §4 B2 "none"; checklist item 7 | polish | `objects.rs` and `lib.rs` citations re-derived or replaced by function names; the nine `(_)`/`(..)` sites are called nine; every `functions.rs` line is marked `+150` and grep-first; B2's "none" becomes the five `lib.rs` `delegate:` lines; item 7 is restated as "no hunk attributable to this contract" with the owned file set named. |

**Not verified here.** This stage ran no `cargo` command — the build lock
belongs to another batch, and the integrator runs the compile gate. Acceptance
items §11.1 (`cargo xc` clean) and §11.2 (the K1 counterfactual in a scratch
copy) are therefore open, and §11.12 (emitted bytes unchanged) is argued from
the shape of the change, not measured.

## Generator-delegation addendum (2026-08-12)

Group B2 is now encoded. `YieldForm::{Plain, Delegate}` replaces the raw IR
boolean; `Delegate` requires `GeneratorDelegationProtocol::YIELD_STAR`, whose
private constructor prevents an unrelated witness from occupying that field.
`EmissionSite::GeneratorDelegation` is backed by both sync and async delegation
functions, and the catalog/witness const joins credit all four iterator
obligations. Backend consumers match the form exhaustively while preserving the
existing emitter branches and instruction order. `generator_delegation.rs` was
untouched.

This addendum was dry-written only. No Cargo command or execution test was run;
the batch integrator owns those gates. Group B1 (`SpreadArgument`) is now
integrated as described in §14 of the main contract; Group C remains open.
