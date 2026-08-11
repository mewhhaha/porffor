# b8 integrator notes

Serial integrator, batch 8. Written incrementally. Permitted commands only:
`cargo check`, `cargo xc`, `cargo fmt --all`, plus read-only inspection. No test,
no build, no test262, no commit.

---

## 0. Machine state found, and the one scheduling action taken

`free` reported **15 GiB of 15 used, 610 MB `MemAvailable`**. Two *identical*
sweep supervisors were running against the same snapshot name and the same
snapshot dir:

```
7255 -> 7259   report-all --snapshot-name baseline-wasm-aot-b2 \
                 --snapshot-dir target/test262-scratch/baseline --threads 2 --jobs 2 --resume   7.0 GiB
9921 -> 20482  (byte-identical command line)                                                    7.1 GiB
```

This is the duplicate IR-TRUTH's note flagged in its handover (its pids
7255/7259 and 9921/9924; 9924 has since been replaced by 20482 through the
supervisor's retry loop). Two writers on one resume journal is a correctness
hazard for the sweep independently of RAM, and at 610 MB available `cargo check`
could not run at all.

Both supervisors were killed first (so they could not respawn a worker), then
both workers. `MemAvailable` went 0.6 GiB -> 14 GiB. Sanctioned by the twice
confirmed scheduling law ("pause the sweep for heavy chunks, restart after");
the journal makes it safe, and the last log line was an ordinary
`test262 checkpoint: 130/382 cases`.

**Owed at the end of this session: restart exactly ONE supervisor.**

## 1. State found in the tree

Three of the four lanes were already committed as WIP checkpoints; the fourth is
in the working tree.

| lane | where it is | evidence |
|---|---|---|
| IR-TRUTH | committed `c8ed6a980` | `lowering.rs` +281/-83, `lib.rs` +68 |
| ERM-STACK + ASYNC-FROM-SYNC-CLOSE | committed `c839f620f` | 20 files, +3376; `async_disposable_stack.rs` 1647 new lines, `promise.rs` +211 |
| RE-VERDICT | committed `a66c2f67f` (`regexp.rs` +225) **plus 5 uncommitted files** | `git status`: `known_failures.rs`, `cli/regexp.rs`, the valid-pattern fixture, `known-failures.tsv`, `regexp.rs`; one untracked fixture `wasm_regexp_identity_escape_solidus.js` |

So this session is application-by-verification plus fixes, not a first apply.

## 2. Lane 1 — IR-TRUTH: `cargo check -p porffor-ir --all-targets`

### Two E0004s, both from ERM-STACK's cross-lane row, one of them undeclared

```
error[E0004] lowering.rs:7694  standard_builtin_signature   9 variants not covered
error[E0004] lowering.rs:25914 standard_builtin_call_info   9 variants not covered
```

The first is exactly ERM-STACK §0's declared build-blocker. **The second was
not declared** — its note asserts `standard_builtin_call_info` at `:25901` "has
a `_ =>`" and is therefore optional. It does not; read verbatim, the match ends
`StandardBuiltinId::ThrowTypeError => Some(ValueInfo::undefined()),` and closes.
This is the `AGENTS.md` "prefer exhaustive `match` to a catch-all" invariant
paying for itself: the omission was a compile error rather than a silent wrong
answer.

### Fix 1 (correctness, not transcription): the constructor's `constructor_instance`

ERM-STACK's proposed signature row spells the fourth tuple member
`ValueInfo::undefined()`. Applied verbatim that would have re-landed the defect
batch 7 just closed.

`AsyncDisposableStackConstructor` is in `builtins.rs::constructable()`
(verified, line 7437), and the fourth member is the static type of the
constructed instance. Both of its consumers are therefore reachable:
`lower_class`'s `inherited_instance` for
`class D extends AsyncDisposableStack {}`, and — the consumer the b6 lane note
records as missed by two prior reviews — `lower_new_expression`'s
`null_heritage_return_path` else-arm, for a **direct** `new AsyncDisposableStack()`.
With the instance typed nullish, `emit_method_call`'s statically-nullish
shortcut emits no call at all. That is the measured batch-5 `IteratorConstructor`
failure set and the batch-7 `IntlDateTimeFormatConstructor` failure verbatim,
both documented in comments 20 lines from the edit.

Applied `Self::fresh_constructed_instance_info()`, copying the
`IntlDateTimeFormatConstructor` arm exactly (`Object`, `{Object}`, no return
shape, fresh constructed instance) — the right precedent because that builtin,
like this one, has no instance shape helper. Reasoning recorded in a comment at
the site.

### Fix 2: every `standard_builtin_call_info` arm is `Some`

`None` at that site is **not** "no static information available" — it is a
refusal. All three consumers read it as "this call does not happen":

```rust
let Some(result) = self.standard_builtin_call_info(builtin, &args, "construct") else {
    return TypedExpr::undefined();
};
let Some(info) = self.standard_builtin_call_info(builtin, &lowered_args, "call") else {
    return (effective_function_id, Vec::new(), ValueInfo::undefined());
};
```

The `call` arm also replaces the argument vector with `Vec::new()`. A `None` arm
for `use`/`adopt` would have compiled, passed `cargo check`, and silently
dropped the call. Nine `Some` arms added, kinds matching the signature table,
with the trap written down at the site.

### Gate

```
cargo check -p porffor-ir --all-targets   EXIT 0
```

Warning set is **identical to `target/lane-notes/b4-baseline-xc.log`**, same six
`porffor-ir` sites, line numbers shifted only:

| site | b4 line | now |
|---|---|---|
| `analysis.rs` field `id` never read | 46 | 46 |
| `lowering.rs` variable need not be mutable | 9618 | 10005 |
| `lowering.rs` `GeneratedFunctionOutput` fields never read | 121 | 124 |
| `lowering.rs` `next_generated_function_index` never read | 546 | 548 |
| `lowering.rs` multiple associated items never used | 7582 | 7815 |
| `lowering_helpers.rs` `StaticStringGeneratorLoopBody` never used | 3 | 3 |

Zero new warnings.

## 3. Lane 2 — ERM-STACK: `cargo check -p porffor-aot-wasm`

**Zero errors on first check**, for ~1,650 blind lines. Because that is exactly
the result a lane which merely *compiles* would also produce, the wiring was
verified rather than inferred — an emitter with no call site compiles clean and
raises no dead-code warning when it is `pub(crate)`, which `AGENTS.md` names as a
shape this repository has shipped before:

| link | evidence |
|---|---|
| module | `builtins/mod.rs:3` `mod async_disposable_stack;` |
| compile dispatch | `builtins/standard.rs:9724-9750`, 9 arms, each calling a distinct `emit_async_disposable_stack_*` |
| intrinsic install | `bootstrap.rs:288` -> `install_async_disposable_stack_constructor_intrinsics` (`intrinsics/collections.rs:477`) |
| bootstrap gate | `bootstrap.rs:4507` `should_initialize_standard_builtin(AsyncDisposableStackConstructor)` |
| emitters | 9 `pub(crate) fn` in the new file, all named by the dispatch |

Warnings: `porffor-aot-wasm` lib **25**, lib-test **20** — identical to the
batch-7 head. Nothing the lane added is unreachable.

### The strengthening: two closed domains that were bare `u64`

This is the one place the lane left a genuine type weakness, and it is the
`AGENTS.md` newtype/exhaustive-match rule twice over.

**(a) Two domains sharing a type *and a value*.** `heap.rs` defined
`ASYNC_DISPOSABLE_STACK_STATE_{PENDING,DISPOSED}` and
`ASYNC_DISPOSABLE_STACK_ENTRY_KIND_{USE,ADOPT,DEFER,EMPTY}` as six `u64`
constants. `..._STATE_DISPOSED` and `..._ENTRY_KIND_ADOPT` are **both `1`**, and
they index different words of different records, so every emitter site accepted
either one silently. Replaced with `AsyncDisposableStackState` and
`AsyncDisposableStackEntryKind`, each with a `const fn word()`; all 14 use sites
converted, and the `as i64` casts now hang off `.word()`.

**(b) The disposal walk's dispatch had a silent fallthrough.** The emitted chain
is `kind != Empty ? (kind == Use ? … : (kind == Adopt ? … : <defer>))`. Its last
arm is an emitted `Else`, so a fifth entry kind would have been **disposed as a
`defer`** — called with an undefined receiver and no arguments — with nothing to
notice. That is precisely the class `data.rs`'s `RuntimeRegExpEntryKind` was
introduced for in batch 7, one record over, and its doc says so in as many words.

So the dispatch is now *derived*, following that precedent exactly: the policy is
stated once as `AsyncDisposableStackEntryKind::dispose_call() ->
Option<AsyncDisposableStackDisposeCall>`, and the emitter builds its comparison
chain by iterating `ALL`. "No call at all" is the `Option`'s `None`, not a
variant, so the emitter's match over call shapes has no arm it must prove
unreachable and needs no `unreachable!`.

**The emitted bytes are unchanged for today's domain**, derived instruction by
instruction rather than assumed — this matters because rung G is unavailable to
this role and the lane is otherwise unverified. With
`ALL = [Use, Adopt, Defer, Empty]`, `no_call_kinds = [Empty]` and
`calling_kinds = [Use, Adopt, Defer]` (`last = 2`):

| emitted | derived | hand-written original |
|---|---|---|
| guard | `LocalGet, I64Const(3), I64Ne, If` | identical |
| i=0 `Use` | `LocalGet, I64Const(0), I64Eq, If`, call(method, resource receiver, `[]`), `Else` | identical |
| i=1 `Adopt` | `LocalGet, I64Const(1), I64Eq, If`, call(method, undefined, `[V]`), `Else` | identical |
| i=2 `Defer` | call(method, undefined, `[]`), no `Else` | identical |
| close | 2 `End` (chain) + 1 `End` (guard) = **3** | 3 |

Temp-local discipline is untouched: each `emit_function_or_proxy_call_leave_throw_completion`
reserves and releases internally, and the loop calls it the same three times.

## 4. Lane 3 — ASYNC-FROM-SYNC-CLOSE

Landed in the same commit as ERM-STACK and covered by the same
`cargo check -p porffor-aot-wasm`: **zero errors, zero new warnings**. The lane
edited `promise.rs` only, and — correctly — left `control_flow.rs` and `ir.rs`
alone after showing the lane spec's location premise was wrong (all six failing
cases are `yield*`, not `for await`).

### Its one cross-file request, applied

`crates/porffor-cli/tests/fixtures/wasm_async_from_sync_iterator_close_on_rejection.js`
was **unreferenced** — verified by grep across `crates/porffor-cli/tests/`, zero
hits. An orphan fixture is the "no call site" shape again: it costs nothing to
compile and proves nothing.

Wired as `iterator::run_wasm_backend_closes_the_sync_iterator_when_an_async_from_sync_value_rejects`,
pasted from §5.1 of the lane note, with the per-term meaning of the marker
recorded at the test so a future reader does not delete a term to make it green.
The fixture's own `print` was checked to emit exactly the asserted marker, and
its `record()` order (`a,b,c,d,e,f,h`) to match the asserted `|` order.

Every `N` in `a=A/1|b=B/1|c=C|d=D|e=E/1|f=F/0|h=H/1` is a **count**, so a double
close fails as loudly as a missing one. `h` is the only oracle in the tree for
the guard's pending-kind term.

Distinguish this from carried item (D): `wasm_async_for_of_closure_capture.js` is
**deliberately** unreferenced until it lands green and was left alone.

## 5. Lane 4 — RE-VERDICT

`cargo xc` covers it; zero errors, zero new warnings. The lane found a third
over-eager site the batch task did not predict (`\u{…}` is not an atom escape at
all, so **every astral pattern written the ordinary way was a `SyntaxError`**),
which is likely the largest single contributor in the RegExp delta run and is the
thing to look at first there.

### The ledger hygiene invariants, re-verified statically

The lane deleted the `UNFILLED` row. `known_failures::*` are runtime tests this
role cannot run, so they were re-implemented over the actual files:

```
ledger data rows            : 4   (heap ignore + 3 perf ignores)
sorted (OutOfOrder check)   : True
`const _: fn()` assertions  : 1, resolving to a real test that carries #[ignore]
every #[ignore] declared    : 4/4  (1 in tests/cli/heap.rs, 3 in tests/perf.rs)
unfilled rows remaining     : 0
attribute-on-one-line       : no violations across tests/cli/*.rs
should_panic rows           : none remain in the tree
```

Worth stating because it changes what rung 1c can detect: with **no
`should_panic` row left**, the drift table's "declared failure starts passing"
and "fails for a different reason" rows currently have no travellers. The ledger
is now purely an `#[ignore]` register. That is the correct state, not a gap — but
it means rung 1c's gate value now rests entirely on ordinary red tests and the
hygiene checks.

`cargo fmt --all` rewrapped four files; the one-physical-line rule for `#[…]`
attributes, which `known_failures::scan_source` enforces and which a rewrap could
plausibly have broken, was re-checked afterwards and holds.

### Rung 1c chunk partition — still valid

```
chunks 20   stems 20   mods 20   three-way identical: True
overlap: `array` must --skip `typed_array` -> present   OK
sh -n scripts/rung1c-chunks.sh : OK
```

## 6. Counts, recounted (do not cite, recount)

`#[test]` attributes in `crates/porffor-cli/tests/cli/*.rs`, exact-line `awk`
form (never the substring grep):

```
623
```

620 at the b7 head, +1 ERM-STACK (`object.rs`), +1 RE-VERDICT (`regexp.rs`),
+1 this session (`iterator.rs`). RE-VERDICT's note counted 622 mid-batch and
correctly said it would move again.

Per-module, for the chunks whose count sidecar will re-trigger:

| module | b7 banked | now |
|---|---|---|
| `regexp` | 35 | **36** |
| `object` | 35 | **36** |
| `iterator` | 30 | **31** |
| `date` | 18 | 18 |

The compiled/executing split (`612`/`611` at b7) was **not** re-measured — that
needs `--list`, which is a build. 623 − 8 `spec-exec-oracle` gates − 1 `heap`
`#[ignore]` predicts 614/613, but treat that as arithmetic, not a measurement.

## 7. Gate status

| gate | result |
|---|---|
| `cargo check -p porffor-ir --all-targets` | **EXIT 0** |
| `cargo check -p porffor-aot-wasm --all-targets` | **EXIT 0** |
| `cargo xc` (`check --workspace --all-targets`) | **EXIT 0**, 0 errors |
| new warnings | **none.** 31 unique `crates/porffor*` warning sites, every one present in `b4-baseline-xc.log`. Per-crate totals differ from b4 only by the single warning batch 7 removed (`porffor-aot-wasm` 26→25 lib, 21→20 lib-test). |
| `cargo fmt --all -- --check` | **clean** (exit 0) after one `cargo fmt --all` |

`porffor-ir` lib-test reads "5 warnings (4 duplicates)" on one run and
"(5 duplicates)" on another. Same six sites both times; it is only which unit
reports a shared site first. b7 recorded the same flip.

## 8. What remains unverified, and by whose rule

Everything below rung 0 — this role is `cargo check`/`xc` only.

* **No test, no build, no test262.** Not one behavioural claim in any of the four
  lane notes has been measured. In particular ERM-STACK's own honest framing
  stands: the claim is "the intrinsic is implemented", never a pass count.
* **Rung G does not apply** to lanes 2-4 (feature work changes bytes by design).
  It *would* have applied to my dispatch rewrite, which is why byte-identity was
  derived by hand in §3 instead of asserted.
* **Item B, the RegExp delta**, is still owed and is the highest-value run:
  `built-ins/RegExp/prototype` (487) as a delta against the baseline snapshot.
  RE-VERDICT's DEFECT 3 (`\u{…}`) makes `built-ins/RegExp` (488) and
  `unicodeSets` (114) at least as interesting.
* **The three new/changed CLI tests** are the cheapest real signal:
  `object::…async_disposable_stack_surface…`, `iterator::…async_from_sync…`,
  `regexp::…identity_escape_solidus…`. None has ever run.
* Sweep-paused chunks: `language_*` and unconditionally `date::` (11.48 GiB).

## 9. Filed forward (analysed here, deliberately not changed)

**The two immutable-binding messages.** Both IR-TRUTH (§ "Filed for batch 9",
item 3) and ERM-STACK reach for this and neither owned both files.
`const x = 1; x = 2;` throws `assignment to immutable binding`
(`lowering.rs:31898`) while `const x = 1; [x] = [2];` throws
`assignment to immutable destructuring target` (`data.rs:161`,
`control_flow.rs:8289`) — one spec error, two messages.

The safety analysis a batch-9 lane would otherwise redo, done here:

* it is a **two-site** edit, not one — the pool literal and the throw site must
  move together or the `must exist in pool` panic class fires at run time;
* `RUNTIME_ERROR_MESSAGE_LITERALS` must stay sorted and unique
  (`the_runtime_error_message_table_is_sorted_and_unique`, which also asserts
  `len() >= 125`). A rename keeps the length, and
  `"assignment to immutable binding" < "assignment to unresolvable reference"`,
  so sortedness survives;
* a duplicate against the lowering path's own message costs nothing —
  `intern_string` returns early on a hit, as the table's doc states;
* **nothing asserts the destructuring text** (grepped `crates/` over `.rs`,
  `.js`, `.tsv`: four hits, all the definition/throw sites plus `lib.rs:9433`
  asserting the *identifier* message).

Not done here because it is a user-visible behavioural change that `cargo check`
cannot gate, in a session with no test access, and both lanes deliberately
deferred it. The deeper fix the table's own doc names — one
`RUNTIME_ERROR_MESSAGES` domain the emitters index into, making "forgot to
intern" a compile error — is ~1,120 call sites and belongs to a lane of its own.

## 10. Sweep restarted — and the duplicate nearly recurred

**A concurrent agent restarted the sweep while I was in the gates**, so by the
time I went to honour the restart I owed, supervisor `9752` was already up. My
own `setsid` restart therefore made it two again — the exact hazard I had just
cleaned up. Killed mine, kept the incumbent:

```
supervisor count : 1   (pid 9752)
worker count     : 1   (pid 9755, --threads 2 --jobs 2 --resume, same snapshot)
MemAvailable     : 7 GiB
```

The lesson is worth more than the incident: **`report-all --resume` is not
self-interlocking.** Nothing in the supervisor or in `report-all` detects a second
process on the same `--snapshot-name`/`--snapshot-dir`; two writers share one
resume journal and one aggregate silently. This has now happened twice in two
batches (IR-TRUTH's handover, and again here), and both times a human had to
notice it in `ps`. Always `ps` for `[s]weep-supervisor.sh` *before* starting one,
and count after. A pidfile or an `O_EXCL` lock beside the snapshot is the real
fix and belongs to whichever lane next owns the sweep tooling.

### What the pause cost the journal: nothing

Killing the workers charged **29** `was in flight when a previous process died`
strikes. That is the journal working as designed, and it was checked rather than
assumed that they did not turn into quarantines — a case at 2 strikes is recorded
as an outcome-`Crash` failure **without being run**, which would have silently
corrupted the node:

```
strike-1 lines charged : 29
quarantines            : 0
live strikes map       : {}      # …built-ins_Atomics-….attempts — empty, i.e. all retired
```

So every charged case was re-run and completed. The standing risk is real though:
**a second kill while the same cases are in flight takes them to strike 2**, so do
not pause and resume repeatedly inside one node.

### Sweep health at handover — do not misread the quiet log

The log had not grown for 45 s at handover, which `batch-workflow.md` says to
treat as suspicious. It is not a stall here, and the distinction was measured
rather than waited out:

```
worker CPU time : 00:31:58 -> 00:33:11 over 30 s wall   (~2.4 cores, both threads busy)
state           : Sl
node in flight  : built-ins/Atomics/wait/{negative-timeout-agent,nan-for-timeout}.js
```

`report-all` prints a checkpoint only every 10 cases, so a slow node is
indistinguishable from a hung one by log growth alone over short windows. **CPU
time delta on the worker pid is the cheap discriminator** and belongs next to the
"judge by whether the log is growing" rule. Note the node: `Atomics/wait` is T17's
old hang, closed in batch 6 — it is now merely slow, and it is the node the
`--stall 900` headroom exists for.
