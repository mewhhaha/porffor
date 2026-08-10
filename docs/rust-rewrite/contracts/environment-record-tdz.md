# Contract: Environment Record binding lifecycle — pointer

The contract for the area *Environment Record binding lifecycle: TDZ as a
per-binding typestate on `BindingInfo`, replacing the parallel string-set stack
and the storage-name prefix* lives at:

`docs/rust-rewrite/contracts/Environment Record binding lifecycle: TDZ as a per-binding typestate on BindingInfo, replacing the parallel string-set stack and the storage-name prefix.md`

This file exists because the area brief names `environment-record-tdz.md` in its
`files_owned` list while the campaign's file-naming convention for contracts uses
the full area title. There is one contract; that file is it.

**Read §0 (Measurement ledger, and where the area brief is wrong) before
implementing anything.** Seven of the brief's numbers and structural claims do
not survive contact with the code, and four of the corrections change what gets
built:

- **§0.1/§0.2** — 41 `BindingInfo` struct literals, not 44 (three of the brief's
  lines are the struct definition and two return types). 4 `mark_tdz_binding` and
  10 `clear_tdz_binding` call sites, not 5 and 11.
- **§0.4** — M5 is not confined to the script/module top level.
  `lower_root_statement_items_with_function_bindings` is also the entry for
  **every function body**, via two call sites in the two function-lowering paths.
- **§0.5** — brief item (4) cannot be implemented as written: `analysis.rs` has
  no `BindingInfo` and two of the four prefix tests hold only an analysis-minted
  `&str`. The `$tdz.` prefix is a name domain and stays one, closed by a newtype.
- **§0.6/§0.7** — brief item (6) understates what is reachable. There are
  **seven** Reference-shaped sites that resolve a binding and act on it; one
  checks TDZ today. The write side (9.1.1.1.5 step 3) is **in scope**, and the
  read side has four further holes (three compound-assign arms and `lower_update`)
  the brief does not mention.
- **§2.2 / M2b** — the brief's `Initialization` enum needs a payload on its
  `Uninitialized` variant. A bare two-state field makes `lexical_storage_name`
  wrong and the resulting mis-shadowing is silent; and a naive M5 fix that just
  calls the predeclare wrapper at the root *introduces* a storage-name split at
  every non-`direct_lexical` scope. This is the strongest argument for the
  `PendingInitialization` token carrying the storage name.

§5 carries the ledger (L1-L4), the two named premises (P1 `namespace.rs`
bail-out, P2 `BindingStorage`) and one open proof obligation (O1) with its
measured price.

---

## Encoder status (this round)

**Landed.** §5b of the full contract is the encoder record: which mistake classes
produce the promised compile error, which moved to the ledger, and the six
deviations from §2/§4 a reviewer should check.

Discharged as compile errors: **M1** (E0063 x 41), **M2**/**M2b** (E0382 +
data dependency + the created-name reuse rule), **M3** (E0609), **M4**
(E0599/E0308), **M5** (E0061 at three statement-list entries), **M6**/**M6b**
(E0004 at seven Reference sites), **M8** (E0004 over `boa_ast::Declaration`).
**M7** stays premise **P2** by design.

Ledger grew from L1-L4 to **L1-L7**: L5 (`DestructuringTargetIr` has no throwing
target, so one array-assignment site reports a lowering gap instead of throwing),
L6 (`LoweredInitializer::evaluated` is a named loophole with six enumerated call
sites), L7 (the root sweep runs before `prepare_root_function_binding_ids`, not
after; observationally equivalent because the overlap is an early error).

Written blind — no `cargo` or `rustc` was run. The integrator owns the compile
gate; `target/lane-notes/environment-record-tdz-theory-integration.md` carries the
hub edits, the `lowering.rs` region list and the aot-wasm premise.
