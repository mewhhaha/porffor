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

The work is split into three groups, and only one of them lands:

- **Group A (§3)** — everything that leaves `cargo xc` green with no edit
  outside `crates/porffor-ir`: the `emission_sites!` row list,
  `ARRAY_DESTRUCTURING_PROTOCOL` and the site↔witness↔catalog triangle
  (K1/J10/J11), the `protocol` field on `ArrayDestructuringPatternIr` (two
  `E0063`s, both in `lowering.rs`), and two `const`-evaluated readers for the
  `abrupt` column (`AbruptDiscipline` + callee containment, J12/J13).
- **Group B (§4)** — `SpreadArgument`'s witness and `YieldForm` replacing
  `delegate: bool`. Fully specified, including every one of the 13 out-of-crate
  pattern lines. **Applied by nobody this round**: §10 P1 forbids editing
  `crates/porffor-aot-wasm/`.
- **Group C (§5)** — the array-literal spread strategy. Designed, note-routed,
  not written.

§10's prohibitions are the load-bearing part for anyone extending this: P1 (no
`porffor-aot-wasm` edit), P2 (round 1's `pub(crate)` narrowing at
`iterator_obligations.rs:45-51` is not re-opened — the emitter-side close token
is a sibling type in the backend crate, never a witness reader), P5 (`lowering.rs`
gets exactly two lines).

§9.15 carries the paper trace that killed the brief's proposed
`#[must_use]`-consumed-by-value close token — it fails at
`objects.rs:14374`, where one emitter has two correct exits sharing one
iterator — and the scope-shaped design that replaces it.

---

## As built (encoder stage, Group A)

Five files changed, all under `crates/porffor-ir/src/`, plus this document and
the lane note. `git status` shows no `crates/porffor-aot-wasm/` path, which is
acceptance item §11.7.

| File | What landed |
|---|---|
| `iterator_obligations.rs` | `emission_sites!` (enum + `ALL` + `name` from one row list), `ARRAY_DESTRUCTURING_PROTOCOL`, `ALL_OBLIGATIONS` promoted out of `#[cfg(test)]`, `ALL_WITNESSES`, `site_is_witnessed`, const asserts **K1/K2/K3/K4**, eleven repaired citations |
| `operations.rs` | `AbruptDiscipline`, the `discipline` and `calls` columns on `StatementEmissionRow` and their five row values, const asserts **J10/J11/J12/J13**, the `SYNC_PROTOCOL_SITES` citation repair |
| `ir.rs` | `ArrayDestructuringPatternIr::protocol` — required, no `Default` |
| `lowering.rs` | the two construction sites `lower_array_binding_pattern` and `lower_array_assignment_pattern` name `ARRAY_DESTRUCTURING_PROTOCOL`. Nothing else. |
| `lib.rs` | `AbruptDiscipline` added inside the pre-existing `pub use operations::{…}` block. No new `mod` line. |

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

**Not verified here.** This stage ran no `cargo` command — the build lock
belongs to another batch, and the integrator runs the compile gate. Acceptance
items §11.1 (`cargo xc` clean) and §11.2 (the K1 counterfactual in a scratch
copy) are therefore open, and §11.12 (emitted bytes unchanged) is argued from
the shape of the change, not measured.
