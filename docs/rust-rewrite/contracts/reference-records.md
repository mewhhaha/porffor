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
