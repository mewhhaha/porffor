# b7 integrator notes

Serial integrator, batch 7. Written incrementally. Permitted commands only:
`cargo check`, `cargo xc`, `cargo fmt --all -- --check`, plus read-only
inspection (`awk`, `grep`, `sh -n`, `git status/log/diff`). No test, no build,
no test262, no commit.

Machine at session start: sweep alive (pid 2209, `report-all --snapshot-name
baseline-wasm-aot-b2`, 4.5 GiB RSS, 188 % CPU), 10 GiB `MemAvailable`.
FRONTIER's driver was **not** running, so no pause was owed before `cargo xc`
(frontier note §6). Nothing was killed or started.

---

## 0. State found: all four lanes were already applied and committed

`git status --porcelain` printed **zero** lines at session start. HEAD is
`35b7203b9` "WIP checkpoint: batch 7 runner resumed after restart", on top of
`c8ca03832` "WIP checkpoint: batch 7 integrate/fix". A prior integrate/fix
session in this batch applied all four `-b7-integration.md` notes and committed
them; `target/lane-notes/b7-runner-findings.md` records that session's 25 fixes
and the runner's rung-0 confirmation.

So this session's job was **verification of what landed**, one strengthening,
and the gates — not a first application. Each lane was checked to be present at
the code level rather than assumed from the note:

| lane | applied? | evidence checked this session |
|---|---|---|
| RE-RT | yes | `data.rs`: `runtime_regexp_argument_literals`, `CandidateOutcome`, `RuntimeRegExpEntry`, `RuntimeRegExpEntryKind` + `ALL`/`word()`/`throws_syntax_error()`, the nine `RUNTIME_REGEXP_RECORD_*_WORD` indices with `RECORD_SIZE` derived; `expressions.rs`: `entry_kind_local`, the discriminant load at word 8, the derived throw chain; all **7** call sites carry `?` (`standard.rs:47536`, `string.rs:{1671,2177,2903,5549,6677,8125}`); `regexp.rs` **35** `#[test]`; both fixtures present |
| IR-SHAPES | yes | `lowering.rs`: `IntlDateTimeFormatConstructor` arm split (`:6515`), both RegExp catch-alls exhaustive on `RegExpCompileErrorKind` (`:24107`, `:25284`); `lowering_helpers.rs`: `AsyncForOfBindingForm` (4 variants, `classify` sole constructor, `rejection()`), `GeneratorPlanRejection` (13 variants) + `linear_generator_plan_with_reason` with `linear_generator_plan` a `.ok()` wrapper; `date.rs` **18** `#[test]`; both fixtures present |
| RUNG1C | yes | `language.rs` 45 / `language_errors.rs` 29 / `language_numerics.rs` 31 = 105; `main.rs` 20 `mod` lines; `rung1c-chunks.sh` 20 `run_chunk` invocations; `CURRENT_BATCH = 7`; tsv header `unfilled-allowed-until: batch-8`; T03 row rewritten |
| FRONTIER | n/a | owns no file under `crates/`; its corrections block (§0.0) is already in the note |

## 1. Static re-verification of the two hygiene invariants I cannot run

`known_failures::rung_1c_chunks_cover_every_cli_area_module` and the overlap
rule are runtime tests. Both were re-implemented over the actual files and pass:

```
chunks 20   stems 20   mods 20   (three-way diff: identical sets)
overlap violations: []        # (other + "::").ends_with(chunk + "::") => must --skip
sh -n scripts/rung1c-chunks.sh : OK
```

The `run_chunk() {` definition line at `rung1c-chunks.sh:265` is correctly not
counted: `RUN_CHUNK_OPENER` is `"run_chunk "` **with the trailing space**.

Counts recounted with the exact-line `awk` form, not a substring grep:

```
620  #[test] attributes across crates/porffor-cli/tests/cli/*.rs
```

which agrees with the `612 compiled / 611 executing` already written into
`docs/rust-rewrite/batch-workflow.md` (620 − 8 `spec-exec-oracle` gates − 1
`heap` `#[ignore]`).

**The T03 row's `422 of 611` was checked arithmetically and is right**, which is
worth recording because it is the number most likely to be assumed. It is the
sum of the *banked* counts in `target/watched/rung1c-done-counts` for the 15
rows in `rung1c-done` (431), minus `frontend`'s 8 gated tests, minus the one
`heap` ignore. It is deliberately **not** the sum of the modules' current
attribute counts (434), because `regexp` (33 → 35) and `date` (17 → 18) have
moved since they banked and will re-run.

## 2. Correctness review of the emitted control flow (the thing `cargo check` cannot see)

`emit_runtime_regexp_program_slots` is the one place in batch 7 where a mistake
is a malformed module rather than a compile error, so the label depths were
re-derived from the emitted sequence rather than trusted:

```
Block                       -> B
  Loop                      -> L
    index >= count ; BrIf(1)          # 0=L, 1=B    exit
    load SOURCE_WORD ; eq ; If        -> I_src
      load FLAGS_WORD ; eq ; If       -> I_flags
        load ENTRY_KIND_WORD -> entry_kind_local
        entry_kind == PROGRAM ; If    -> I_kind
          six program stores
        End                           # I_kind closed BEFORE the branch
        Br(3)                         # 0=I_flags 1=I_src 2=L 3=B
      End
    End
    index += 1 ; Br(0)                # back edge to L
  End
End
```

Br(3) reaches `B`; the discriminant `If` contributes no depth because it is
opened and closed inside `I_flags`. The note's claim holds. Two further
properties that make the post-loop test sound, both checked:

* `entry_kind_local` is written **only** inside `I_flags`, so a row that matches
  the source but not the flags cannot leave a stale kind behind; and a full match
  always leaves the loop immediately, so no later iteration can overwrite it.
* the no-match exit is `BrIf(1)`, which leaves `entry_kind_local` at its
  initialised `Program` word — hence a total miss does not throw, which is the
  policy §2.2 of the RE-RT note argues for.

The derived throw chain (`ALL.filter(throws_syntax_error).map(word)`) emits
`LocalGet/I64Const/I64Eq` per word and `I32Or` for every position after the
first; that is a correct fold for any arity, and today it is arity 1.

`emit_throw_runtime_error_to_active_handler` at that site resolves to
`emit_return_current_completion` (all seven call sites are builtin bodies, so
`is_main()` is false), which pushes the ABI results and emits `Return` — valid at
any block depth, including inside the callers at `string.rs:{2177,2903,5549}`
that still have an `If` open around the call.

## 3. The one change this session made

`crates/porffor-aot-wasm/src/data.rs` — a doc comment on
`collect_finite_string_choices`, which had none.

It is the only new item in the RegExp path with a `_ => {}` over an open domain,
and it is now **load-bearing for a throw**: a literal this function collects
reaches `RegExpProgram::compile`, and an `InvalidSyntax` verdict becomes a
`RUNTIME_REGEXP_ENTRY_KIND_REJECTED` row that throws `SyntaxError` at all seven
call sites for any runtime pattern with the same bytes. The doc states the
asymmetry (an unrecognised shape costs coverage only; a recognised one widens
what the compiler refuses at run time), points at
`RUNTIME_REGEXP_ENTRY_KIND_REJECTED`'s own warning that `porffor-ir`'s ~20
`invalid_syntax(` sites are unaudited, and records that the missing
concatenation arm (`new RegExp(a + b)`, RE-RT probe 6) is **deliberate**, not an
oversight — closing it is exactly the widening the doc warns about and needs its
own measured gate.

Nothing else was changed. Two candidate strengthenings were considered and
rejected on the "does a plausible mistake become a compile error?" test:

* `runtime_regexp_programs: Vec<(String, String, RuntimeRegExpEntry)>` — a
  source/flags swap is silent, but a named struct with two `String` fields does
  not make it a compile error either, and only newtypes would. Two call sites;
  not worth the churn without a measurement that the risk is live.
* `RuntimeRegExpEntryKind::ALL` is hand-written. Stable Rust cannot enumerate a
  enum's variants, so any "fix" is decoration; the real trigger is the
  `error[E0004]` a new variant produces at the two exhaustive matches, and the
  type's own doc already says so.

## 4. Gate status

| gate | result |
|---|---|
| `cargo check -p porffor-aot-wasm` | **EXIT 0** |
| `cargo xc` (`check --workspace --all-targets`) | **EXIT 0**, 0 errors |
| new warnings | **none**. Workspace warning set is identical to `target/lane-notes/b4-baseline-xc.log` **minus one**: `porffor-aot-wasm` lib 26 → 25 and lib-test 21 → 20, the dropped one being `functions.rs: unused variable: receiver_is_array`. `porffor-ir` 6 lib / 5 lib-test unchanged, same six sites. |
| `cargo fmt --all -- --check` | **clean** (exit 0), before and after the edit |

The `porffor-ir` lib-test line reads "5 warnings (4 duplicates)" where b4 read
"(5 duplicates)". That is not a new warning: the same six sites are reported, and
`lowering.rs:124` (`GeneratedFunctionOutput`'s dead fields) is merely emitted
first under the lib-test unit this time.

## 5. What remains unverified, and by whose rule

Everything below rung 0. This role is `cargo check`/`xc` only, so:

* **no `--list`** — 612 compiled / 611 executing is carried from the prior
  session's measurement (recorded in the T03 row and in `batch-workflow.md`) and
  independently agrees with 620 − 8 by the `awk` recount above, but it was not
  re-measured here.
* **no rung 1c.** Seven chunks are due to execute on the next run — the three
  `language_*` (never banked), `string` and `functions` (targeted-invalidated by
  the prior session for the RE-RT compiler change), and `regexp`/`date` (counts
  sidecar firing on 33 → 35 and 17 → 18). Pause the sweep for the three
  `language_*` chunks and unconditionally for `date::` (11.48 GiB peak).
* **no test262.** RE-RT's three staged gates are unrun: `annexB/built-ins/RegExp/prototype/compile`
  (23 cases, the measured Bug), `built-ins/RegExp/named-groups` (36), and
  `built-ins/RegExp/prototype` (487, a **delta** — it cannot exit 0). The third is
  the one that matters most and is the easiest to skip: it is the only check on
  the false-`InvalidSyntax` risk that the new `Rejected` row creates.
* **RE-RT's six-line discriminator probe** (§1 of its note) is still owed. It is
  the first thing to run after a build: pre-fix it prints
  `THREW TypeError: RegExp.prototype.exec unsupported pattern`, post-fix `true`.
  If it still throws, the `CallIndirect` arm's narrowing is not matching.
* **IR-SHAPES's size measurement** (§1.5) and the `date::` fixture verdict.
* `built-ins/Array/fromAsync` is expected to stay **93/95** — the two cases'
  `detail_hash` changes with the new message, the count does not. A claim of
  95/95 would be wrong.
