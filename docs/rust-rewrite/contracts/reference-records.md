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

---

## DISCREPANCY-FIXER stage (dry-run findings applied)

Twelve findings against the encoder's landing, all applied except one that is
recorded rather than fixed and one that is corrected in the contract only.

| # | Severity | What was wrong | Where it now lives |
|---|---|---|---|
| 1 | **blocker** | The scoped override (see the encoder's report above) makes the *runtime* strictness guard live in `main` for every property write. That guard opens one Wasm block the compile-time arm does not, and two sites forwarded `Br` immediates without compensating for it — one of them compensating its sibling branch and not its throw. `"use strict"; try { a.length = 0 } catch (e) {}` at top level branched one label too shallow. | contract §4.5.1, ledger **L8**; `RUNTIME_STRICT_GUARD_BLOCK_DEPTH` in `objects.rs`; new fixture pair + CLI tests |
| 2 | bug | `carried_strictness` attributed a **TypeError** shape to every strict reference write, including the three global-write variants whose PutValue **2.a** is a **ReferenceError**. That narrowed the `catch` binding of `"use strict"; try { undeclaredXyz = 1 } catch (e) {}` to the wrong error shape. | §2.6; `PutValueFailure`, `carried_put_value_failure`, MC7 |
| 3 | bug | `GlobalPropertyUpdate` / `GlobalPropertyCompoundAssign` bound `strictness: _` and wrote back through the *unchecked* emitter — a field constructed at 3 sites and read at 0, which is what I9 prohibits. | ledger **L5** (closed); `emit_reference_global_property_write`; `planning.rs` budgets |
| 4 | bug | Contract choice **C5** asserted the tree already defers `ToPropertyKey` past the RHS. False for the plain-assignment path. | §1.7 C5 (corrected), ledger **L7**. **Recorded, not fixed** — the safe fix splits `PropertyKeyIr`, a shared backend enum; named as a follow-up lane with build access |
| 5 | bug | Corpus entry 9 was labelled "the canonical single-record trace for I5"; it is a `with`-scope case, and §4.4 excludes Object Environment Records from this lane. | §6 entry 9 (relabelled out of scope) |
| 6 | bug | Corpus entry 13, "the only proof for the 32187 literal", does not reach `lower_identifier_arithmetic_general` at all. | §6 entry 13 (replaced with a source that satisfies the measured reachability condition) |
| 7 | polish | `ReferencePins` derived `Default` and exposed `none()`, so `ReferencePins::none().materialize(record.write(..))` compiled and silently discarded the real chain. | §2.4; `ReferenceRecord::pin_operands` is the sole producer; ledger **L3** shrunk |
| 8 | polish | `base_mut() -> &mut ReferenceBase` permitted whole-value assignment, so a property Reference could be swapped for a global one after its `[[Strict]]` was chosen — a doc comment asserting an invariant the type did not carry. | §2.2; `base_mut` deleted |
| 9 | polish | MC2's compile error was never built: three emitters still took `strict: bool`, and `i64::from(self.is_current_function_strict())` still sat at the `I64Const` site. | §3 MC2 note; three signatures now take `Strictness`; `ambient_object_write_strict_flag_word` renames the fallback |
| 10 | polish | `const _: () = assert!(REFERENCE_STRICTNESS_FLAG_LOCALS == 1)` compared a constant to a literal, not to what the emitter reserves — decoration by AGENTS.md's test. | `with_reference_strictness` reserves into `[u32; REFERENCE_STRICTNESS_FLAG_LOCALS]` and destructures it; the const-assert is deleted |
| 11 | polish | Invariant I7's **AST** half did not exist: `AssignTarget` and `UpdateTarget` were still matched with a catch-all and a `let ... else`. | both are now exhaustive matches over the closed boa enums; `lower_property_access_update` extracted so `lower_update` can match |
| 12 | polish | `ExprIr::GlobalPropertyWrite.implicit` was reported as read by nobody. | Refuted: its consumer is the `implicit_globals` counter in `ir.rs`'s AST-stat visitor. Field kept, with a doc comment saying so and saying it is not a backend input |

**Open proof obligation I7, restated.** The `ExprIr`-side half was already
closed (77 variants, no `_`). The AST-side half named in the obligation is now
closed too (finding 11). What remains open is §2.5's `lower_reference` itself:
the record is still *recovered from the lowered read* rather than built from
`AssignTarget`/`UpdateTarget`/`PropertyAccess`. Both catch-alls are gone and both
domains are exhaustive, so drift on either side is a compile error; the
AST-derived constructor stays a follow-up lane for the reason the encoder gave.

**Not verified by this stage.** No cargo or rustc command was run: the integrator
owns the compile gate. Every claim above is from reading the tree.

---

## INTEGRATOR stage (compile gate)

Applied from
`target/lane-notes/Reference Records: …-theory-integration.md`, on top of the
encoder and discrepancy-fixer stages.

| Note item | Status |
|---|---|
| §2.1 — `emit.rs`'s `object_write_strict_flag_local` doc comment still asserted the belief F1 falsifies (`None` ⇒ "the compile-time strictness of the enclosing function is authoritative") | **APPLIED.** Rewritten to say `None` means *no Reference is in play* — property installation, class field definition, internal helper writes — where `ambient_object_write_strict_flag_word` is authoritative. Doc-only. |
| §2.2 — three helpers still taking `strict: bool` | already applied by the discrepancy-fixer (finding 9). Verified: `environments.rs:1265`, `environments.rs:1329`, `objects.rs:7674` take `Strictness`; the only surviving `strict: bool` in those two files is `environments.rs:668`, which is not a Reference write. **Ledger L4 closed.** |
| §2.3 — `DestructuringTargetIr::AssignmentProperty` carried no `[[Strict]]`, so 13.15.5.4's PutValue used the emitting function's ambient mode | **APPLIED** (see below). **Ledger L6′ closed.** |
| §2.4 / S5 / MC4b — `super.x = v` writes to the super base, not `GetThisValue(V)` | **not attempted**, for the encoder's reason: the fix needs a new `this_value` operand on the IR node wired into `emit_ordinary_set_result_with_receiver_fallback`, and a receiver change written without a runtime oracle is more likely to produce a second wrong answer than a right one. Stays ledger **L6**. |
| §1 — optional move of `mod reference` from `ir.rs`'s `#[path]` to `lib.rs` | **deliberately skipped.** Module location holds no invariant: no mistake becomes a compile error either way, so by the contract's own §5.1 standard it is not worth a redeploy of a shared file while another workflow is committing the tree. The `#[path]` form is correct as it stands. |

### §2.3, as landed

Three files, one new field, and the field is read.

1. `porffor-ir/src/ir.rs` — `DestructuringTargetIr::AssignmentProperty` gains
   `strictness: Strictness`. Its one construction site is
   `lowering.rs`'s `lower_array_assignment_property_target`, so omitting it is
   `E0063`; a `bool` there is `E0308` for the reasons in `Strictness`'s doc.
   `AssignmentIdentifier` and `AssignmentPrivate` still carry none, and the
   field comment says why (branch 4.c and PrivateSet respectively).
2. `porffor-ir/src/lowering.rs` — filled from `self.reference_strictness()`,
   the crate's single producer. The `grep -c 'self\.reference_strictness()'`
   gate in ledger L1 moves from **21** to **22**.
3. `porffor-aot-wasm/src/control_flow.rs` — `PreparedDestructuringTarget::Property`
   carries it from `prepare_destructuring_target` to `put_destructuring_target`,
   which wraps `compile_property_write_to_locals` in
   `with_reference_strictness` (promoted to `pub(crate)` for this call).
   Analysis arms in `data.rs` and `planning.rs` took `..` / `strictness: _`.
4. `porffor-aot-wasm/src/planning.rs` — the array-pattern
   `AssignmentProperty` temp-local budget gains
   `REFERENCE_STRICTNESS_FLAG_LOCALS`, because `with_reference_strictness`
   reserves one and `reserve_temp_local` **asserts** rather than diagnosing.
   The object-pattern path is covered by its flat `128 + …` budget.

**Why this is not a new instance of discrepancy-fixer finding 1.**
`with_reference_strictness` opens no Wasm block of its own; the extra block is
opened by the runtime guard inside `emit_object_write_set_failure_else` /
`emit_object_write_non_extensible_failure`, and both already compensate their
`Br` immediates with `RUNTIME_STRICT_GUARD_BLOCK_DEPTH`. This site inherits that
compensation rather than adding a second one.

**Behavioural surface.** `({ x: o.p } = src)` and `[o.p] = src` now select
PutValue 3.d's TypeError from the Reference's `[[Strict]]`. The two answers
differ exactly when lowering hoists the pattern into a generated function whose
mode differs from the owner plan's — the same divergence class item 2 of the
encoder's §4 describes, now closed for destructuring as well.

### Compile-gate outcome

`cargo check -p porffor-ir` clean; `cargo check -p porffor-aot-wasm` clean;
`cargo xc` (`check --workspace --all-targets`) exit 0, **0 errors**, and the
warning set is identical to the pre-integration baseline after normalising line
numbers. `cargo fmt --all -- --check` exit 0.
