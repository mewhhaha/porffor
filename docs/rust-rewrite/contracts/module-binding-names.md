# Contract: module binding-name domains — pointer

The contract for the area *Module binding-name domains: `[[LocalName]]` vs
`[[ExportName]]` vs merged storage name* lives at:

`docs/rust-rewrite/contracts/Module binding-name domains: [[LocalName]] vs [[ExportName]] vs merged storage name.md`

This file exists because the area brief names `module-binding-names.md` in its
`files_owned` list while the campaign's file-naming convention for contracts
uses the full area title. There is one contract; that file is it.

**§10 of that document is the dry-run discrepancy pass and supersedes §§1–9
where they disagree** — notably V6/K4, the minted-vs-source disjointness claim,
ledger R1's stated reason, and the K1/K2 overclaims.

---

## INTEGRATOR stage

The lane note's §2 ("edits needed in files this lane does not own") is **None**,
and the containment claim was re-checked here rather than assumed: no type or
minting function named in the note appears outside `crates/porffor-ir/`, and the
only hit inside `porffor-ir` but outside `modules/` is
`lowering.rs`'s `.map(ModuleLinkErrorIr::to_diagnostic)`, which is unaffected by
adding a variant.

The §3 shared-file coordination in `lib.rs` resolved cleanly: this lane's
`mod binding_names;` and the iterator lane's `mod iterator_obligations;` are both
present in the alphabetical run, with both `pub use` blocks intact.

`cargo check -p porffor-ir` and `cargo xc` are clean.
