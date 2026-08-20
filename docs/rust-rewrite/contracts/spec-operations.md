# Spec-operation catalog contract

This area's contract lives in one document, because the catalog's
`StatementEmission` rows are witnessed by the iterator contract's `EmissionSite`
values and the two would drift if split:

**[Spec-operation catalog evidence and the iterator-protocol obligation witness](./Spec-operation%20catalog%20evidence%20and%20the%20iterator-protocol%20obligation%20witness.md)**

Start at §3 (type mapping, Part A) for the catalog half.

As built: `crates/lila-ir/src/operations.rs`. §12 (encoder addendum) records
the four deviations, the three added mistake classes, and ledger entries L6–L8.

**§13 is the dry-run discrepancy pass and supersedes §§1–12 where they
disagree.** For this half, read 13.2 (the catalog entry was forgeable), 13.3
(`ALL` is now macro-generated; the L1 test and `catalog_index` are deleted),
13.9 (`sites` is a slice), 13.10 (the single-source `TaskId` enum closes owner
membership over T00–T29) and 13.11 (what `EmitterEvidence` actually proves, and
how L2 must be scoped).

---

## INTEGRATOR stage — I7 applied

The lane note's **I7** (optional, sequenced last) is now **applied**: the dead
`ForOfArray` async path is deleted.

The evidence was re-measured in the tree before deleting, not taken from the
note. `StatementIr::ForOfArray` has exactly **one** construction site
(`lowering.rs`, the array index-walk head lowering) and it set
`async_plan: None`; `AsyncForOfPlanIr` had **zero** construction sites
workspace-wide — only a definition, a field type, an import and a parameter
type. So `compile_async_for_of_array` (448 lines) was unreachable from the
product path, which AGENTS.md says should fail to build and did not, because it
was `pub(crate)` and reached from arms fed by a field that is always `None`.

Deleted: `AsyncForOfPlanIr` and `ForOfArray.async_plan` (`ir.rs`),
`async_plan: None` at the construction site (`lowering.rs`),
`compile_async_for_of_array` plus its import and the two
`async_plan: Some(plan)` entry/exit-state arms (`control_flow.rs`), and the two
`ForOfArray { async_plan: Some(_), .. }` arms in `emit.rs`. The two dispatch
arms now bind `..` and call `compile_for_of_array` unconditionally.

`for await` over an array is unaffected: it does not reach `ForOfArray` at all.
`lower_for_of` routes any `for_of.r#await()` to `ForOfIterator`, whose
`AsyncForOfIteratorPlanIr` is a different type and is genuinely constructed —
which is *why* the `ForOfArray` path was dead.

`cargo check -p lila-ir`, `cargo check -p lila-aot-wasm` and `cargo xc`
are all clean after the deletion, and no warning appeared or disappeared, so
nothing else depended on it. Rung G is expected to diff empty and has not been
run here.
