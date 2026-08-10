# Batch 4 INTEGRATOR log

Window: 2026-08-10 01:20 – 02:25 UTC. Branch `claude/test-driven-rust-opus-pp6giw`.
Started at `aa9afa1a0`; the orchestrator checkpointed my in-flight edits into
`a3b7d8cd0` mid-run (I committed nothing myself).

**All three lanes' owned files were already committed at `aa9afa1a0`.** The
integrator's work was therefore (1) the cross-file edits each lane filed as "not
my file, apply this", (2) compile repair, (3) type strengthening, (4) the rung-0
gates. Scope was `cargo check`/`xc`/`fmt` only — no test, no test262 run.

---

## 1. GATE STATUS (measured in the live tree at 02:20 UTC)

| Gate | Result |
|---|---|
| `cargo xc` (= `check --workspace --all-targets`) | **0 errors** |
| warnings in `crates/**` | **33**, and the set is **identical, line for line, to the pre-batch baseline** (`HEAD~1` = `091487732`, captured in `target/lane-notes/b4-baseline-xc.log`) |
| `cargo fmt --all -- --check` | **clean** (exit 0, no diff) |

The baseline was produced by an independent `cargo xc` over `091487732` and
decomposes as `porffor-aot-wasm` 26 (lib) + `porffor-ir` 6 + `porffor-test262` 1
= 33, which reproduces batch 3's recorded count exactly. **No new warning was
introduced by any of the three lanes, by the cross-file edits, or by me.**

Not run, and out of this stage's scope: every test target, rung 1b/1c, rung G,
and all test262. `cargo xc` type-checks the CLI test targets but executes none of
them.

---

## 2. Cross-file edits applied, in the lanes' stated integration order

| # | Filed by | File | Edit | State |
|---|---|---|---|---|
| 1 | handle-cluster Half A | `porffor-test262/src/lib.rs` | (already committed) | landed; `cargo check -p porffor-test262` clean |
| 2 | zdt-era §0.1 | `porffor-ir/src/names.rs` | 3 function-id consts (`…ZONED_DATE_TIME_PROTOTYPE_{ERA,ERA_YEAR}_GETTER…`, `…TO_PLAIN_DATE_TIME…`) | applied |
| 3 | zdt-era §0.2 | `porffor-aot-wasm/src/module.rs` | 3 variants into the `=> None` or-pattern | applied |
| 4 | handle-cluster §4.1 | `porffor-aot-wasm/src/emit.rs` | export `THROW_ERROR_MESSAGE_EXPORT` | applied |
| 5 | handle-cluster §4.2 | `porffor-aot-wasm/src/emit.rs` | the `debug_dump` line for it | applied |
| 6 | handle-cluster §4b | `porffor-aot-wasm/src/objects.rs` | clear the message global at the one direct name-set site | applied, **then strengthened** — see §4.1 |
| 7 | throw-prop §2.1 | `builtins/errors.rs` | drop `extra_depth` from `emit_throw_runtime_error_to_active_handler` | applied |
| 8 | throw-prop §2.2a | `builtins/standard.rs` | 1 `If`+`push_control` → `open_frame` | applied |
| 9 | throw-prop §2.2b | `builtins/standard.rs` | drop the `0,` argument | applied |
| 10 | throw-prop §2.2c | `builtins/standard.rs` | 8 `…_with_extra_depth` → `…_if_needed` | applied, 0 left |
| 11 | throw-prop §2.3 | `generator_delegation.rs` | 28 `If`+`push_control` → `open_frame` | applied 28/28 (−28 lines; `pop_control` still 28) |
| 12 | throw-prop §2.4 | `modules.rs` | the 1 hand edit (a comment sits between the two lines) | applied |

Two lane invariants re-counted after integration, not taken on trust:

- `grep -rn extra_depth crates/porffor-aot-wasm/src` returns **only doc-comment
  prose** — zero occurrences in code. The `extra_depth` parameter is gone from
  the crate.
- `self.push_control(...)` has **0 call sites** left crate-wide; the name
  survives only as `FunctionBuilder::push_control`'s definition and in
  `code_sink`'s tests.

Every lane note's stated failure mode for a *missed* application was verified to
be real, in the order the notes predicted: zdt §0.1 failed as E0432 naming
exactly the three constants; §0.2 would have been E0004; throw-prop §2.1–2.4
failed as arity/name errors. Nothing failed silently.

---

## 3. BLOCKER encountered, and how it was worked around (recorded, not fixed)

`porffor-aot-wasm` depends on `porffor-ir`, so *no* rung-0 check of this batch's
backend work is possible while `porffor-ir` is red — and it was red for the first
hour, in files no batch-4 lane touched (the concurrent theory round 3):

- **Committed at `aa9afa1a0`:** `E0308` at `lowering.rs:8591` —
  `LexicalScopeInstantiation::instantiate(self, …)` passes `ScriptLowerer<'_>`
  where `&mut ScriptLowerer<'_>` is wanted (the enclosing `fn lower(mut self, …)`
  takes `self` by value). **The one-character fix is `&mut self`.** That is what
  I applied *in a throwaway worktree only*; the live tree was left alone.
- Additionally red in the working tree at 01:31–01:44, transiently:
  `numeric_conversions.rs:510/515/520` (`residue_pow2_i64(truncated, 32)` passing
  an integer for `ResidueWidth`), `lowering.rs:20959/21002/21032` (E0532 on
  `NumberFormatFold::RangeError`), `lowering.rs:35218…35300` (E0061/E0277/E0593).

Workaround: `git worktree add --detach /home/user/porffor-b4check aa9afa1a0`, my
batch-4 files copied in, theory r3's committed E0308 patched locally, checks run
there. **Both worktrees are now removed.** By 02:16 the live tree's `porffor-ir`
compiled on its own and every number in §1 is from the **live tree**, not the
scaffold.

> **Sharp edge worth writing down.** Two git worktrees of this repo pointed at one
> `CARGO_TARGET_DIR` **corrupt each other's artifacts**: the `porffor-*` path
> dependencies collide on their metadata hash, so the second tree's `.rmeta`
> overwrites the first's while the first's fingerprint still reads "fresh". The
> symptom was 21 phantom `E0599: no variant named TemporalZonedDateTimePrototypeEraGetter`
> errors in a tree that plainly had the variant. Give each worktree its own
> `CARGO_TARGET_DIR`, or run them serially and `touch` the sources first.

---

## 4. Type strengthening done (all three verified by `cargo xc` + `fmt`)

### 4.1 The two throw-diagnostic globals are now set through one function

`builtins/errors.rs` gains `emit_set_thrown_error_text(name, message: Option<&str>, function)`.
The pairing *is* the invariant — a site that sets the name and forgets the
message reports a previous, unrelated throw's message — and it was previously
spelled out at three independent sites, one of which (the array-length path in
`objects.rs`) had already forgotten, which is why handle-cluster filed §4b.
`None` is now the explicit "this throw carries no message" answer rather than an
omission. Emitted instruction order is unchanged at all three sites.

After this, the complete inventory of sites touching either global is:
`errors.rs` (the helper, plus `emit_capture_throw_error_name`, which reads both
off a user-thrown value and zeroes the message on entry), and `control_flow.rs:2883`
/ `promise.rs:755` — both of which zero the *name* and then call
`emit_capture_throw_error_name`. Verified by reading all four, not by grep count.

### 4.2 `code_sink::Function::{new, byte_len}` are marked `#[cfg(test)]`

They were the batch's **one new warning** (`never used` in the lib build). They
are not dead: `emitted_function.rs`'s decoder tests need the run-length
constructor for a mixed `i64`/`i32` declaration. Marking them test-only keeps
them and removes the warning, and leaves `new_with_locals_types` as the only
constructor a product path can reach — which is the shape AGENTS.md asks for
("code with no call site should fail to build"). `code_sink`'s own tests were
moved off `new`.

### 4.3 Considered and deliberately NOT done

- **A generation/identity tag on `LabelDepth`.** `branch_depth_to` catches a
  target whose frame is *closed* (`checked_sub` underflows), but not one whose
  frame closed and whose depth was then re-reached by a *sibling* frame — that
  still yields an in-range immediate naming the wrong block. A monotonic label id
  would turn it into a panic. Rejected for this batch: it converts a currently
  silent case into a new hard failure, and I have no way to run the CLI suite or
  the sweep to bound the blast radius. Recorded as a follow-up with a gate
  (rung 1c green first, then add it, then rung 1c again).
- **The stale `era` getter declarations** (`zdt-era` note §5.1): all four `era`
  getters declare `ValueKind::Undefined` with a one-kind `KindSet` while a
  `gregory` receiver returns the string `"ce"`. `porffor-ir/src/lowering.rs` was
  being rewritten by theory round 3 in the same working tree throughout my
  window; editing 4 tables in a 1.75 MB file mid-rewrite risked clobbering their
  in-flight work for a change that is measured non-fatal today
  (`intl402/Temporal/PlainDate` 488/488). **Owner: the next batch's temporal
  lane**, as one edit across all four getters plus their `ValueInfo` twins
  (`lowering.rs` ~:6901/:27507 and siblings), to `Dynamic` + `String|Undefined`.

---

## 5. handle-cluster's own top risk is REFUTED by measurement

The lane's §7.1 calls the completeness of the 125-string
`RUNTIME_ERROR_MESSAGE_LITERALS` table "the riskiest thing in this patch and the
one thing a unit test cannot check": a missing string is compile-clean and
run-time fatal (`string `<msg>` must exist in pool`).

I checked it statically instead, to a fixpoint
(`/tmp/…/scratchpad/pool_audit2.py`, method described here so it can be re-run):

1. Seed with the `message` argument position of the 8 throw entry points
   (`emit_throw_runtime_error`, `…_to_active_handler`, `…_with_prototype_local`,
   `emit_runtime_error_object`, and the four
   `emit_throw_current_function_realm_*`).
2. Whenever the argument at a tracked `(fn, index)` is a bare identifier that is
   a `&str` **parameter of the enclosing function**, that enclosing `(fn, index)`
   becomes tracked too. Repeat to a fixpoint.
3. Collect every string literal that ever lands at a tracked position, including
   literals embedded in `match`/`if` message expressions.

Result: **5 rounds to a fixpoint, 616 tracked `(fn, argument)` pairs, 779
distinct message literals reaching a throw, 0 of them absent** from `data.rs` /
`intl_datetimeformat.rs`. The 10 named `*_MESSAGE` consts were resolved by hand
and are all present (including `BIGINT_DIVISION_BY_ZERO_MESSAGE`, the one batch 3
missed).

Stated limits, so this is not read as more than it is: membership is tested
against *every literal appearing in* `data.rs` plus the Intl DTF pool file, which
over-approximates the interned set slightly (a literal could appear in `data.rs`
in a non-interning context — though `StringPool::collect`'s first act is one
unconditional `for value in [ … ]` block, so the approximation is close); and a
message assembled at run time through `format!` is outside the analysis (one
site, `emit.rs:3493`, which formats a builtin debug name). **This does not
replace rung 1b/1c** — it does mean the integrator found no reason to expect the
panic the lane warned about, and the runner should not budget for an iterate-on-
panics loop.

---

## 6. HAND-OFF — the batch's one hard deadline

`crates/porffor-cli/tests/cli/known_failures.rs:82` still reads
`const CURRENT_BATCH: u32 = 3;`, and `known-failures.tsv` still carries the
`cli / UNFILLED / unfilled / T03` row under `# unfilled-allowed-until: batch-4`.

`ledger_is_well_formed` is a **libtest** assertion, so `cargo xc` cannot see it:
the batch is green at rung 0 with the deadline unmet. Bumping `CURRENT_BATCH` to
4 while the `unfilled` row survives turns rung 1c red for a reason unrelated to
any lane; filling the row requires a completed rung 1c. **The bump and the fill
are one edit and belong to whoever completes rung 1c.** If this batch ends with
`CURRENT_BATCH` still at 3, the expiry silently slid a batch — which is the exact
failure mode the header exists to prevent, so say so out loud rather than
letting it pass.

handle-cluster's other ledger edit is landed and consistent: the T24 row, its
`const _` assertion and the `#[should_panic]` were deleted together, and `cargo
xc` proves the `const _` hygiene still resolves (it is a compile-time check).

## 7. For the runner, in priority order

1. **Rung 1c** (`cargo test -p porffor-cli --test cli -- --test-threads=2` under
   `run-watched.sh --label b4-cli --stall 900`). It is the gate for
   throw-propagation-label-depth, it is the only thing that can adjudicate a
   branch immediate, and it is what fills T03. Expect two *new* panic messages to
   be meaningful rather than noise, both from `code_sink`:
   `function body for <FunctionIdentity::wasm_name> has an unclosed control
   frame: N label(s) still open` and `` wasm `end` with no open label ``. Both
   were previously anonymous wasmtime module-validation failures. (The findings
   fixer added the `for <name>` clause via `Function::into_body_named`; this line
   quoted the nameless earlier wording, which no longer occurs on the product
   path — `into_body` is now `#[cfg(test)]`.)
2. The three lane verify prefixes, **with one correction**: engine
   `render_wasmtime wasm_backend_characterization` — the second filter is
   required and was missing. `render_wasmtime` alone selects only the new pure
   unit test, which passes even if `emit.rs` never exports
   `throw_error_message`; the assertions that can see an inert Half B live in
   `wasm_backend_characterization_matrix_locks_public_surface_and_outcomes`.
   See handle-cluster note §3.6b. Then test262 `detail_hash`, aot-wasm
   `runtime_error_message_pool_tests global_index_registry`, cli `language::` /
   `known_failures::` / `throw_propagation::` / `date::`, then the zdt lane's five
   `porf test262 run` lines and the throw-prop lane's
   `language/statements/{try,switch,for}`.
3. Rung G is **inverted** for throw-propagation (an empty diff means the repair
   did not land) and **inapplicable** to the other two lanes (both change bytes by
   construction: 125 new pool strings shift every string offset; new
   `StandardBuiltinId` variants shift `all_functions()` order). Do not use it as
   this batch's gate.
