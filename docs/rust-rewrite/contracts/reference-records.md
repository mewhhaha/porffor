# Contract: Reference Records — pointer

The contract for the area *Reference Records: one record, a carried
`[[Strict]]`, and a write that consumes it* lives at:

`docs/rust-rewrite/contracts/Reference Records: one record, a carried [[Strict]], and a write that consumes it.md`

This file exists because the area brief names `reference-records.md` in its
`files_owned` list while the campaign's file-naming convention for contracts
uses the full area title. There is one contract; that file is it.

Read **§5 (Deviations from the area brief)** before implementing anything. Five
points in the brief would produce a defect or a decoration if followed verbatim:

- **§5.1** — the brief lists nine `ExprIr` variants that need a `strictness`
  field. Three of them (`AssignIdentifier`, `CompoundAssignIdentifier`,
  `UpdateIdentifier`) must **not** get one; PutValue's Environment Record
  consumer is discharged at lowering time and the field would be read by zero
  backend arms. Two the brief omits (`DeleteProperty`, `DeleteGlobalProperty`)
  are included.
- **§5.2** — there are **two** reference-reconstruction catch-alls, not one.
  `lower_update` (`lowering.rs:32871`) is the worse of the pair.
- **§5.3** — `SuperPropertyWrite` does not merely lack `[[ThisValue]]`; the
  backend writes to the super base instead of the receiver.
- **§5.4** — two corpus entries do not test what the brief says. Two of the
  named MC1 oracles are MC3 oracles, and the third is `eval`-wrapped and inert
  on the AOT path.
- **§5.5** — `lowering_helpers.rs` is owned but needs no edit; `lib.rs` is not
  owned and has 262 `ExprIr::` references (all in `#[cfg(test)]`, all using
  `..`).

**§4.5 states a scope extension the campaign must accept or refuse explicitly.**
MC3 — the live conformance gap where no IR node can express
`"use strict"; Object.freeze(o); o.x = 2` throwing — cannot be made
load-bearing inside the brief's `files_owned` list. The measured cost outside it
is 3 files, 5 signatures/fields, 4 call sites (`objects.rs`, `emit.rs`,
`control_flow.rs`; none on batch 2's hold list).

---

## Encoder's report

Implemented; see
`target/lane-notes/Reference Records: one record, a carried [[Strict]], and a write that consumes it-theory-integration.md`
for the full account, the instructions for files this lane does not own, and the
self-review. Three findings that change the contract itself:

**§4.5's scope extension is not needed.** `object_write_strict_flag_local` is a
`pub(crate)` field of `FunctionBuilder` (`emit.rs:302`) and
`emit_object_write_strict` (`objects.rs:14867`) is already the scoped-override
idiom. Generalising it as `with_reference_strictness` inside the *owned*
`expressions.rs` makes the carried `[[Strict]]` reach every ordinary write,
super write, update and compound assign without touching `objects.rs`,
`emit.rs`, or any signature. MC3 is discharged, backend half included. §4.5's
"3 files, 5 signatures/fields, 4 call sites" was measured correctly for the
*parameter-threading* design; a scoped override needs none of it.

**§2.5's AST-derived `lower_reference` was not implemented.** Both catch-alls
are deleted and the reconstruction is now a single total function over all 77
`ExprIr` variants with no `_` arm, so a new read specialisation is `E0004`
rather than a silent downgrade. But the record is still built from the lowered
read, not from `AssignTarget`/`UpdateTarget`/`PropertyAccess`. Building from the
AST requires re-deriving the key through ~150 lines of `ValueKind`-dependent
logic with lowerer side effects, or lowering the base twice — trading MC4a for
MC5. Named as a follow-up lane for an agent with build access; §2.5 stands as
written, unimplemented.

**Two named types were refused as decoration.** `ReferenceBase::Binding` (§2.2)
is constructed nowhere once §1.5's proof is taken seriously, and `SuperThisValue`
(§2.2, I4) would be constructed at two sites and read by none until S5/MC4b
wires the receiver through `emit.rs`. Both are recorded in the ledger — MC4b as
new entry **L6** — rather than shipped as types that hold no invariant.
