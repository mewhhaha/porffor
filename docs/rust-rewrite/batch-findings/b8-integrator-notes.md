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
