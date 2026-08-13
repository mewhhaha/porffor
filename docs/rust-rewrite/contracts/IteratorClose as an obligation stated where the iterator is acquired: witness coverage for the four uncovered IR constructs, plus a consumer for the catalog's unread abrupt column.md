# Contract: IteratorClose as an obligation stated where the iterator is acquired

Witness coverage for the uncovered IR constructs, plus a consumer for the
catalog's unread `abrupt` column.

Status: **normative for the encoder**. Group A, generator-delegation slice B2,
call-spread slice B1 and the direct ArrayAccumulation replacement for Group C
are encoded. The
original counts below were obtained by reading the tree at
`claude/test-driven-rust-opus-pp6giw`; §13 records the 2026-08-12 B2 integration
and supersedes the earlier statements that no part of Group B had landed.

Predecessor: `docs/rust-rewrite/contracts/Spec-operation catalog evidence and
the iterator-protocol obligation witness.md` (round 1). This contract extends
its Part B and consumes its Part A. Its §13 amendments and §5/§12.4 ledger are
in force; where this document adds a ledger row it continues that numbering with
an `IC` prefix so the two ledgers cannot be confused.

Owned files (all edits this contract authorises):

- `crates/lila-ir/src/iterator_obligations.rs`
- `crates/lila-ir/src/operations.rs`
- `crates/lila-ir/src/ir.rs`
- `crates/lila-ir/src/lowering.rs` — **two lines only**, enumerated in §8 R3.
- `crates/lila-ir/src/lib.rs` — additions inside the two pre-existing
  `pub use` blocks only. No new `mod` line.
- `docs/rust-rewrite/contracts/…` (this file, and the short redirect
  `iterator-close-obligation.md`)
- `target/lane-notes/iterator-close-obligation-theory-integration.md` (new)

Files this contract **does not** touch, stated as a prohibition rather than an
omission: everything under `crates/lila-aot-wasm/`. That includes
`emission_sites.rs`, `control_flow.rs`, `expressions.rs`, `functions.rs`,
`planning.rs`, `data.rs`, `emit.rs`, `objects.rs`, `builtins/standard.rs`, and
of course the four batch-2 files (`intl_datetimeformat.rs`, `temporal*.rs`,
`emitted_function.rs`, `runtime_helpers.rs`). §4 and §5 specify work whose
landing requires such edits; that work is **specified here and applied by nobody
this round** — the patch text lives in the lane note.

---

## 0. How to read this, and six corrections to the area brief

Clause numbers in ECMA-262 move between editions; **abstract-operation names are
normative in this document and clause numbers are navigational**. If a number
does not resolve, follow the name and fix the number.

The area brief was written from a survey that this formalization re-measured.
Six of its statements are wrong, and two of them change what should be built.
They are listed first because §§1–9 depend on the corrected facts.

| # | Brief says | Measured | Consequence |
|---|---|---|---|
| **C1** | `ExprIr::ArrayLiteral` acquires an iterator "for its spread elements". | **`ExprIr::ArrayLiteral` never contains a spread element.** Both array-literal lowerers desugar spread *away* before the node exists: `lower_array_literal` (`lowering.rs:28119`) and `lower_staged_generator_array_literal` (`:15745`) emit `[].concat(chunk, …)` — `ExprIr::CallMethod` with key `"concat"` — and route a non-array spread operand through `Array.from`. The residual non-spread path bails with `unsupported_expr("array literal spread")` (`:28244`) and is unreachable behind the guard at `:28120`. | A `protocol` field on `ArrayLiteral` would be **decoration and a false claim** — the exact species this area exists to delete. Rejected. The real obligation is the *desugaring choice*, and it is designed in §5 as Group C, note-routed, not written. |
| **C2** | Four uncovered constructs. | **Three** IR constructs acquire an iterator with no stated discharge (`ArrayDestructure`, `SpreadArgument`, `GeneratorYield{delegate:true}`), plus one obligation that has **no IR node at all** (array-literal spread, C1). | The witness-coverage work is 3 constructs + 1 ledger row, not 4 fields. |
| **C3** | "6 `ExprIr::ArrayDestructure` construction sites in `lowering.rs`, 35 references workspace-wide." | **36** references; **5** constructions (`lowering.rs:14590`, `:31589`, `:31721`, `:31824`, `:31936`); `:12642` is a pattern. | Moot: §3 A3 attaches the witness to `ArrayDestructuringPatternIr`, which has exactly **2** construction sites, so none of the five is touched. |
| **C4** | "3 acquisition sites through `emit_get_iterator_from_value_locals` (`control_flow.rs:7687`, `:7779`, `:7891`)". | **One** call site (`:7687`). `:7779` is the function's own definition; `:7891` is `finish_get_iterator_from_method`, a different function. | The lane note's acquisition/close census is restated from measurement. |
| **C5** | "~11 close sites across `control_flow.rs`, `objects.rs` and `builtins/standard.rs`". | **62** close call sites across **seven** files: `builtins/standard.rs` 38, `builtins/collections.rs` 12, `control_flow.rs` 7, `objects.rs` 2, `builtins/promise.rs` 1, `builtins/array_from_async.rs` 1, `generator_delegation.rs` 1. | The emitter-side token retrofit is ~6× the size the brief priced. Recorded in the lane note so the batch that takes it is not surprised. |
| **C6** | `EmissionSite::ArrayDestructuring` is a "documentation-shaped lie", the catalog "ATTRIBUTES obligations to a site the witness type does not reach". | The **attribution is true** — `compile_array_destructure_from_value_locals` (`control_flow.rs:7656`, not `:8220`) really does emit all four obligations, verified step by step in §1.6. What is missing is the *other* direction: no IR construct has accepted responsibility for that emitter. | M3 is repaired by **adding** `ARRAY_DESTRUCTURING_PROTOCOL` and a witness field, not by deleting the row. Calling a true row a lie would have produced the wrong fix. |

One further correction that is not the brief's: every `control_flow.rs` line
number in `iterator_obligations.rs` and `operations.rs` is stale by 400–800
lines. §8 R6 repairs them; §2.4 tabulates cited-vs-actual.

Terminology, continued from round 1:

- **Obligation**: a spec step that must happen. **Discharge**: how the compiler
  accounts for it — `ByEmission(site)`, or `ByAssumption(premise)`.
- **Acquisition site**: a point in the IR where a construct causes a
  `GetIterator` to run. This is where the close obligation is *incurred*, and
  therefore where this contract makes it a required field.
- **Emission site**: a `lila-aot-wasm` function that performs a 7.4
  operation. This is where the close obligation is *discharged*, and it is
  outside this contract's file set — §10 says so as a prohibition.

---

## 1. Spec basis

### 1.1 Completion Records, `?`, and why an abrupt exit is a distinguished event (6.2.4, 5.2.3.4)

A Completion Record is `{ [[Type]], [[Value]], [[Target]] }` with
`[[Type]] ∈ {normal, break, continue, return, throw}` (6.2.4). `ReturnIfAbrupt`
(5.2.3.4), spelled `?`, is defined only over that closed domain: if `[[Type]]`
is not `normal`, the enclosing algorithm returns the record unchanged.

The repository models the domain as `CompletionKindIr` (six inhabitants: the
five spec types plus `Empty` for the empty `[[Value]]`/`[[Target]]` sentinel)
and `CompletionAbruptKind` (four inhabitants: `Throw`, `Return`, `Break`,
`Continue`) — `operations.rs:251-267`. Both are closed enums with exhaustive
`const fn` renderers and no catch-all. That part is already right.

What is missing is a *reader*. `CompletionAbruptKind` has exactly one mention
outside `operations.rs`, and it is the `pub use` line at `lib.rs:107`:

```sh
grep -rn "CompletionAbruptKind" crates/ --include=*.rs | grep -v lila-ir/src/operations.rs
# → crates/lila-ir/src/lib.rs:107  (the re-export)
```

`SpecOperationIr::abrupt()` (`operations.rs:798`) and
`SpecOperationCatalogEntry::abrupt()` (`:918`) have **zero** callers outside
`operations.rs`; the accessor's only reason for existing is that the field is
private. Round 1 shipped a machine-readable statement of which abstract
operations may throw and nothing reads it. §3 A4 gives it two readers, both
`const`-evaluated.

### 1.2 IteratorClose (7.4.11), and the asymmetry in step 4

`IteratorClose(iteratorRecord, completion)`:

1. Assert `iteratorRecord.[[Iterator]]` is an Object.
2. Let `iterator` be `iteratorRecord.[[Iterator]]`.
3. Let `innerResult` be `Completion(GetMethod(iterator, "return"))`.
4. If `innerResult.[[Type]]` is `normal`:
   a. Let `return` be `innerResult.[[Value]]`.
   b. If `return` is `undefined`, return `? completion`.
   c. Set `innerResult` to `Completion(Call(return, iterator))`.
5. **If `completion.[[Type]]` is `throw`, return `? completion`.**
6. **If `innerResult.[[Type]]` is `throw`, return `? innerResult`.**
7. If `innerResult.[[Value]]` is not an Object, throw a **TypeError**.
8. Return `? completion`.

Steps 5 and 6, in that order, are the whole content of the asymmetry:

- **The original completion is a `throw`** → the close's own error is
  *swallowed*. The program observes the original throw. (Step 5 fires before
  step 6 ever looks at `innerResult`.)
- **The original completion is `break` / `continue` / `return`** → an error
  raised by `GetMethod(iterator,"return")` or by `Call(return, iterator)`
  *replaces* it (step 6), and a non-Object result from `return()` becomes a
  fresh TypeError (step 7).

This is not a stylistic detail. It is the difference between two program
outcomes, and in this repository it is decided entirely by which of two
similarly-named plain functions a call site picks —
`emit_iterator_close` (`control_flow.rs:8479`) versus
`emit_iterator_close_preserving_current_throw` (`:8622`) — across 62 call sites.
That is mistake class **M5**, and §10 explains why typing it is not this
contract's to do.

Note also step 3's `GetMethod`: a `return` property that is neither `undefined`,
`null` nor callable makes `GetMethod` itself throw *before* step 5 is reached —
which is why `iterator-close-throw-get-method-abrupt.js` (§9.2) is the sharpest
trace in the corpus.

### 1.3 AsyncIteratorClose (7.4.12) and IfAbruptCloseIterator (7.4.13)

`AsyncIteratorClose` is 7.4.11 with `Await(innerResult)` inserted after the
`Call`, and with the same step-4/5/6 precedence. Its emission ordering is
explicitly out of scope (§10) — `compile_async_for_of_iterator`
(`control_flow.rs:5577`) open-codes it.

`IfAbruptCloseIterator(value, iteratorRecord)` is:

1. Assert `value` is a Completion Record.
2. If `value` is an abrupt completion, return
   `? IteratorClose(iteratorRecord, value)`.
3. Else set `value` to `value.[[Value]]`.

**This is the specification's own name for the obligation this contract is
about**: *this completion is abrupt, therefore close before propagating*. It is
a macro, not an operation with a signature, which is why it has no catalog row —
and why the obligation has to be carried by the *acquisition* rather than by a
row. Every one of the ~15 builtin consumers listed in §1.7 writes it out inline.

### 1.4 Where the obligation is incurred — the acquisition sites

| Spec site | What acquires | Close obligation |
|---|---|---|
| 8.6.2 ForIn/OfBodyEvaluation 7.a.ii, 7.b | `GetIterator` in ForIn/OfHeadEvaluation | Close on any abrupt body completion whose `LoopContinues` is false (14.7.1.1), i.e. `break`, `return`, `throw`, and a `continue` targeting an *outer* label. A `continue` targeting this loop does **not** close. |
| 8.6.3 IteratorBindingInitialization | one `GetIterator` per ArrayBindingPattern, **including each nested one** | Step 5 of ArrayBindingPattern evaluation: `if iteratorRecord.[[Done]] is false, return ? IteratorClose(iteratorRecord, result)` — on both normal and abrupt results. |
| 13.15.5.5 IteratorDestructuringAssignmentEvaluation | one `GetIterator` per ArrayAssignmentPattern, including nested | identical shape to 8.6.3; a different abstract operation with the same close discipline. |
| 14.4.14 `yield*` | `GetIterator(value, generatorKind)` once | three resume modes; see §1.5. |
| 13.2.4.1 ArrayAccumulation (SpreadElement) | `GetIterator(spreadObj)` per spread element of an array literal | none owed: every abrupt exit of the loop is a `IteratorStep`/`IteratorValue` failure, after which `[[Done]]` is already true. |
| 13.3.8.1 ArgumentListEvaluation (`...expr`) | `GetIterator(spreadObj)` per spread argument | same as 13.2.4.1: none owed. |
| 23.1.2.1, 24.1.1.2, 27.2.4.1, 27.1.4.x, `Object.groupBy`/`Map.groupBy` | one `GetIterator` each | `IfAbruptCloseIterator` inline. |

The two "none owed" rows are load-bearing and are the reason §4 B1's witness is
`ByAssumption`, not `ByEmission`. In ES2025 `IteratorStepValue` sets
`iteratorRecord.[[Done]]` to `true` on every abrupt path it has, so the caller's
`?` propagates with nothing left to close. Getting this backwards — emitting a
close after a `next()` that already threw — is an *observable extra call to
`return`*, i.e. a conformance failure in the opposite direction.

**One clause of these two rows is not yet established**, and B1's premise says so
(§4): whether GetIteratorFromMethod owes a close when `Get(iterator, "next")`
throws. If it does, both rows are incomplete and the destructuring `GetIterator`
path is missing a close as well. The vendored corpus does not settle it; the
`yield-star-next-get-abrupt` family asserts only that the reason propagates, with
no `returnCount`. **OPEN**, and it gates acceptance of B1's premise, not of
anything landed.

### 1.5 14.4.14 `yield*`, stated as three resume modes

`yield* AssignmentExpression`:

1. `iteratorRecord` ← `? GetIterator(value, generatorKind)`.
2. `received` ← `NormalCompletion(undefined)`. Repeat:
   - **normal**: `innerResult` ← `? Call(next, iterator, «received.[[Value]]»)`
     (async: `Await`); require Object; if `done` → return `IteratorValue`; else
     `received` ← the completion of yielding it.
   - **throw**: `throw` ← `? GetMethod(iterator, "throw")`.
     - If `throw` is not `undefined`: `innerResult` ← `? Call(throw, iterator, …)`;
       require Object; …
     - **Else** — the branch this contract cares about — *the iterator gets a
       chance to clean up first*: let `closeCompletion` be
       `NormalCompletion(empty)`; perform
       `? AsyncIteratorClose(iteratorRecord, closeCompletion)` (async) or
       `? IteratorClose(iteratorRecord, closeCompletion)` (sync); **then throw a
       TypeError**.
   - **return**: `return` ← `? GetMethod(iterator, "return")`; if `undefined`,
     return `Completion(received)` (async: after `Await`); else
     `innerResult` ← `? Call(return, iterator, «received.[[Value]]»)`; require
     Object; if `done` → return; else yield.

So `yield*` owes four protocol operations *plus* a close-then-TypeError branch
that exists in no other construct. Today the whole of that is requested by
`delegate: bool` (`ir.rs:1974`). Verified present in the emitter:
`generator_delegation.rs:1088` closes and `:1098` throws
`"yield* iterator has no throw method"`, in that order, on the sync path. The
async path (`:610-632`) reaches the TypeError only after finding that neither
`throw` nor `return` exists, which is consistent with 7.4.12 (a missing `return`
makes the close a no-op).

### 1.6 What `compile_array_destructure_from_value_locals` actually emits

Read at `control_flow.rs:7656-7770`, because C6 turns on it:

| Obligation | Evidence |
|---|---|
| `GetIterator` (7.4.2) | `emit_get_iterator_from_value_locals(…)` at `:7687`, which reads `@@iterator`, checks callability, calls it, checks the result is an Object and caches `next` once (`finish_get_iterator_from_method`, `:7891`). |
| `IteratorStep` (7.4.8) | `emit_destructuring_iterator_step` (`:8099`), called per element from `compile_array_destructuring_element` (`:7980`). |
| `IteratorValue` (7.4.9) | same function; `value` is read after `done`. |
| `IteratorClose` (7.4.11) | **both** halves. Normal completion: `locals.done == 0` guard at `:7707`, then `emit_iterator_close` (`:7710`). Abrupt completion: the same `done == 0` guard at `:7726`, then `emit_iterator_close_preserving_current_throw` (`:7729`), then `emit_propagate_current_completion_if_throw`. |

The `done` guard is 8.6.3 step 5 / 13.15.5.5's
`if iteratorRecord.[[Done]] is false` — which is exactly what
`array-elem-iter-nrml-close-skip.js` (§9.5) pins. So the catalog's row is
**true**, and `ARRAY_DESTRUCTURING_PROTOCOL` may honestly be
`emitted_by(EmissionSite::ArrayDestructuring)`.

### 1.7 The builtin consumers, and why they are not typed here

`IfAbruptCloseIterator` is written out inline at 62 close call sites (§2.3).
They are all in `lila-aot-wasm`, they are batch 5's lane this round, and none
of them is reachable from a `lila-ir` type. §10 states the prohibition; the
lane note carries the design.

### 1.8 Where the spec leaves latitude, and the choice this contract makes

1. **Nothing in 7.4 requires that a compiler represent the close obligation at
   all.** A compiler may re-derive it at each emission site. This contract
   chooses to state it *at acquisition*, in `lila-ir`, because the emission
   sites outnumber the acquisition sites 62 to 6 and because an acquisition with
   no stated discharge is invisible today.
2. **7.4 does not say a specialization is forbidden.** `for (x of arr)` may be
   an index walk if the realm is intact. This contract keeps round 1's choice:
   a specialization must *name* the premise it relies on, and the premise's
   truth remains ledger L3.
3. **Whether one `EmissionSite` variant may denote a family of emitter arms.**
   §4 B2 needs `yield*` to name both `compile_generator_delegation` and
   `compile_async_generator_delegation`, because the lowerer cannot know which
   fires (the emitter selects on `FunctionExecutionKind::AsyncGenerator` at
   `control_flow.rs:1938`). The choice made here is: **yes, provided the
   `emission_sites_are_backed` arm names every member**, so the E0599 guarantee
   covers all of them. The alternative — widening
   `ObligationDischarge::ByEmission` to a slice, as §13.9 did for the catalog —
   is rejected: the catalog needed a slice because *different operations* are
   credited to *different arms*; here one obligation is discharged by one of two
   arms selected at emit time, which a single named family denotes exactly.

---

## 2. Measured baseline

Every number below was produced by the command beside it, run at the repository
root on this branch. Re-derive rather than trust.

### 2.1 The constructs

| Construct | refs | constructions | where |
|---|---|---|---|
| `ExprIr::ArrayDestructure` | 36 | 5 | `lowering.rs:14590, 31589, 31721, 31824, 31936` |
| `ArrayDestructuringPatternIr` | 14 | **2** | `lowering.rs:32307, 32361` |
| `ExprIr::SpreadArgument` | 23 | **1** | `lowering.rs:25199` |
| `ExprIr::ArrayLiteral` | 48 (excluding 33 `TypedArrayLiteral*` false hits in `lila-test262`) | 9 | `lowering.rs:15776, 15825, 15835, 28149, 28212, 28222, 28257, 28271`; `lila-aot-wasm/src/builtins/json.rs:97` |
| `StatementIr::GeneratorYield` | 76 | **1** | `lowering.rs:15502` |

```sh
grep -rn "ArrayDestructure" crates/ --include=*.rs | wc -l                 # 36
grep -rn "ArrayDestructuringPatternIr" crates/ --include=*.rs | wc -l      # 14
grep -rn "SpreadArgument" crates/ --include=*.rs | wc -l                   # 23
grep -rn "GeneratorYield" crates/ --include=*.rs | wc -l                   # 76
grep -rn "ArrayDestructuringPatternIr *{" crates/ --include=*.rs           # 2 hits, both lowering.rs
```

`ArrayDestructuringPatternIr` is the pivotal measurement. It is
`pub struct ArrayDestructuringPatternIr { pub elements: Vec<ArrayDestructuringElementIr> }`
(`ir.rs:826-828`). Outside its two construction sites, **every** use in the
workspace — including all six `lila-aot-wasm` uses — is a `&`-borrow that
reads `.elements`. There is no struct-literal pattern, no `..Default`, and no
exhaustive destructuring anywhere. Adding a field is therefore `E0063` at
exactly two lines, both inside `crates/lila-ir`, and byte-neutral everywhere
else.

### 2.2 The witness and the sites

`EmissionSite::ArrayDestructuring` has exactly four mentions, as the brief says:

```sh
grep -rn "EmissionSite::ArrayDestructuring" crates/ --include=*.rs
# iterator_obligations.rs:112 (variant), :120 (name arm)
# operations.rs:1012 (SYNC_PROTOCOL_SITES)
# lila-aot-wasm/src/emission_sites.rs:31 (the name-resolution join)
```

Six witness constant *names* (`ARRAY_INDEX_WALK`, `ARRAY_INDEX_WALK_RESUMABLE`,
`STRING_CODE_POINT_WALK`, `SYNC_ITERATOR_PROTOCOL`, `ASYNC_ITERATOR_PROTOCOL`,
`NO_ITERATION`) but **five distinct values**: `ARRAY_INDEX_WALK_RESUMABLE` is
defined as `Self::ARRAY_INDEX_WALK` (`iterator_obligations.rs:397`). Any tie
written over the constants must be written over the *name* list, not over
value-distinctness.

None of the six names `ArrayDestructuring`. That asymmetry — a site the catalog
credits and no acquisition has accepted — is what §3 A2 closes.

### 2.3 The emitter side (context only; not edited this round)

```sh
grep -rn "self\.emit_iterator_close" crates/lila-aot-wasm/src/ | wc -l   # 64
grep -rno "self\.emit_iterator_close[a-z_]*" crates/lila-aot-wasm/src/ \
  | awk -F: '{print $3}' | sort | uniq -c
#  15 self.emit_iterator_close
#   2 self.emit_iterator_close_condition_i32
#  43 self.emit_iterator_close_preserving_current_throw
#   4 self.emit_iterator_close_preserving_saved_throw
```

62 close call sites (64 minus the two `_condition_i32` predicate calls), across
seven files: `builtins/standard.rs` 38, `builtins/collections.rs` 12,
`control_flow.rs` 7, `objects.rs` 2, `builtins/promise.rs` 1,
`builtins/array_from_async.rs` 1, `generator_delegation.rs` 1.

Definitions: `emit_iterator_close_condition_i32` `control_flow.rs:8451`;
`emit_iterator_close` `:8479`; `emit_iterator_close_preserving_current_throw`
`:8622`; `emit_iterator_close_preserving_saved_throw` `:8637`.
`IteratorCloseOnThrowLocals` is `emit.rs:99`, `#[derive(Debug, Clone, Copy)]`,
eleven `pub(crate)` `u32` fields. Two functions take it as
`Option<…>`: `objects.rs:14383` and `builtins/standard.rs:4418`.

### 2.4 Stale citations in the two owned files

| Cited in an owned file | Actual |
|---|---|
| `compile_for_of_array` `control_flow.rs:5732` | **5238** |
| `compile_for_of_string` `:5850` | **5353** |
| `compile_async_for_of_iterator` `:6078` | **5577** |
| `compile_for_of_iterator` `:7422` | **6874** |
| `compile_array_destructure_from_value_locals` `:8220` | **7656** |
| `emit_iterator_close_condition_i32` `:9018` | **8451** |

```sh
grep -n "fn compile_for_of_array\|fn compile_for_of_string\|fn compile_async_for_of_iterator\|fn compile_for_of_iterator\|fn compile_array_destructure_from_value_locals\|fn emit_iterator_close_condition_i32" \
  crates/lila-aot-wasm/src/control_flow.rs
```

`emit_iterator_close` at `:8479` and `_preserving_current_throw` at `:8622` are
correct as cited. §8 R6 repairs the six wrong ones in the two owned files.

### 2.5 The unread column

```sh
grep -rn "CompletionAbruptKind" crates/ --include=*.rs | grep -v operations.rs   # 1 hit: lib.rs:107
grep -rn "\.abrupt()" crates/ --include=*.rs | grep -v operations.rs             # 0 hits
```

Zero consumers. `SpecOperationCatalogEntry::abrupt()` is a `pub` accessor with
no caller, kept alive by `pub` — the "survival by `pub`" shape round 1's I7
named.

### 2.6 The behavioural pins that already exist

`crates/lila-cli/tests/cli/iterator.rs` holds 30 `#[test]` functions, of
which **5** name iterator closing (the brief said 6):

```
:26  run_wasm_backend_closes_iterators_for_return_and_outer_continue
:55  run_wasm_backend_routes_iterator_close_errors_through_outer_cleanup
:75  run_wasm_backend_preserves_throw_when_iterator_close_throws
:95  run_wasm_backend_preserves_throw_when_iterator_close_returns_primitive
:188 run_wasm_backend_succeeds_for_iterator_close_callable_proxy_preserves_throw_fixture
```

These matter for §7: mistake class **M6** has *no shipped-defect ledger entry*
and *no git-log instance*. It is a real gap, and it is not a wound. The
contract does not inflate it.

---

## 3. Type mapping — Group A: what lands this round

Group A is exactly the work that leaves `cargo xc` green with **no edit outside
`crates/lila-ir`**. Every item is stated as the Rust the encoder writes.

### A1. `emission_sites!` — enum, `ALL`, and `name` from one row list

`EmissionSite` today declares its variants (`iterator_obligations.rs:100-113`)
and its `name` arm (`:116-122`) separately, and has no `ALL`. §3 A2 needs an
`ALL` to quantify over, and a hand-written `ALL` reintroduces round 1's ledger
L1 exactly (a variant absent from every list is invisible to every const
expression). Round 1 solved this once, with `spec_operations!`
(`operations.rs:655-677`). Copy that shape:

```rust
/// Declares [`EmissionSite`], its `ALL` enumeration and its `name` renderer
/// from **one** row list, so "added a variant, forgot the list" is not
/// expressible. This is `spec_operations!`'s shape (operations.rs), applied to
/// the site domain for the same reason: §3 A2's ties quantify over `ALL`, and a
/// hand-written `ALL` would make them silently partial.
macro_rules! emission_sites {
    ($( $variant:ident => $name:literal ),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum EmissionSite {
            $( $variant , )+
        }

        impl EmissionSite {
            pub const ALL: &'static [EmissionSite] = &[ $( EmissionSite::$variant , )+ ];

            pub const fn name(self) -> &'static str {
                match self { $( EmissionSite::$variant => $name , )+ }
            }
        }
    };
}

emission_sites! {
    SyncForOfIterator  => "compile_for_of_iterator",
    AsyncForOfIterator => "compile_async_for_of_iterator",
    ArrayDestructuring => "compile_array_destructure_from_value_locals",
}
```

The per-variant doc comments currently on the enum move above the macro
invocation as a module-level comment block, or are dropped; a macro-generated
variant cannot carry one from the row. Keep the `ArrayDestructuring` prose —
with the line number repaired per §2.4 — as a comment beside its row.

**The variant set is unchanged.** `lila-aot-wasm/src/emission_sites.rs` is
therefore untouched, which is why A1 is in Group A.

### A2. `ARRAY_DESTRUCTURING_PROTOCOL`, and the site↔witness tie

The witness constant, justified line by line in §1.6:

```rust
/// `ExprIr::ArrayDestructure`, both spec operations (8.6.3
/// IteratorBindingInitialization and 13.15.5.5
/// IteratorDestructuringAssignmentEvaluation) and every nested pattern inside
/// them.
///
/// Every obligation is really emitted, verified at `control_flow.rs:7656-7770`:
/// `emit_get_iterator_from_value_locals` (`:7687`),
/// `emit_destructuring_iterator_step` (`:8099`) per element, and **both** halves
/// of 7.4.11 step 4 — `emit_iterator_close` under the `[[Done]]` guard on the
/// normal path (`:7710`) and `emit_iterator_close_preserving_current_throw`
/// under the same guard on the abrupt path (`:7729`). There is no array fast
/// path here, so every array destructuring pays the real protocol.
pub const ARRAY_DESTRUCTURING_PROTOCOL: Self =
    Self::emitted_by(EmissionSite::ArrayDestructuring);
```

The witness census, `pub(crate)` so `lila-aot-wasm` cannot read it (§10 P2):

```rust
**The census is generated, not hand-maintained.** An `iterator_witnesses!`
macro — `emission_sites!`'s shape applied to the witness domain — declares every
`pub const` **and** `ALL_WITNESSES` from one row list, so "added a constant,
forgot the census" is not expressible. An alias row is written
`ARRAY_INDEX_WALK_RESUMABLE => Self::ARRAY_INDEX_WALK`, which is why the census
is over names rather than values (§2.2): seven names, six distinct values.

```rust
macro_rules! iterator_witnesses {
    ($( $( #[$meta:meta] )* $name:ident => $body:expr ),+ $(,)?) => {
        impl IteratorProtocolWitness { $( $( #[$meta] )* pub const $name: Self = $body; )+ }
        pub(crate) const ALL_WITNESSES: &[IteratorProtocolWitness] =
            &[ $( IteratorProtocolWitness::$name , )+ ];
    };
}
```

```rust
/// True when some witness constant discharges **`obligation`** by emission at
/// `site`. Per-obligation, not per-site: a site may run 7.4.2/7.4.8/7.4.9 and
/// owe no 7.4.11 (B1's `CallArgumentSpread` is the worked example), and J10 has
/// to be able to say so.
pub(crate) const fn site_emits(site: EmissionSite, obligation: IteratorObligation) -> bool {
    let mut i = 0;
    while i < ALL_WITNESSES.len() {
        // `match`, not `if let`, to stay in the const-fn shape the existing
        // `assumes` / `emits` helpers in this file already prove compiles.
        // Discriminant comparison via `as u8` is likewise their existing idiom:
        // `EmissionSite` has no `const` `PartialEq`.
        match ALL_WITNESSES[i].discharge(obligation) {
            ObligationDischarge::ByEmission(actual) => {
                if actual as u8 == site as u8 { return true; }
            }
            ObligationDischarge::ByAssumption(_) => {}
        }
        i += 1;
    }
    false
}

/// K1's question, derived from `site_emits` rather than re-deriving the scan.
pub(crate) const fn site_is_witnessed(site: EmissionSite) -> bool {
    let mut j = 0;
    while j < ALL_OBLIGATIONS.len() {
        if site_emits(site, ALL_OBLIGATIONS[j]) { return true; }
        j += 1;
    }
    false
}
```

`ALL_OBLIGATIONS` is currently a `#[cfg(test)]` const (`:658`). Promote it to a
module-level `pub(crate) const ALL_OBLIGATIONS: [IteratorObligation; 4]` and let
the test module use the promoted one. It now has a product-path caller, so this
is the opposite of the §14.2 dead-code trap.

Three const assertions, in `iterator_obligations.rs`:

```rust
// (K1) Every `EmissionSite` is witnessed. FAILS ON TODAY'S TREE: no witness
//      names `ArrayDestructuring`. That is the point — a tie that passes before
//      and after the fix is decoration.
const _: () = {
    let mut i = 0;
    while i < EmissionSite::ALL.len() {
        assert!(
            site_is_witnessed(EmissionSite::ALL[i]),
            "an EmissionSite names an emitter arm that no IR construct's witness has accepted"
        );
        i += 1;
    }
};

// (K2) The new constant says what its doc comment says — the same shape as the
//      four existing witness assertions — and it is asked of the value
//      reachable *through the IR field's type*, so this is also
//      `ArrayPatternProtocol::witness`'s const consumer.
const _: () = assert!(
    emits_every_obligation(
        ArrayPatternProtocol::ARRAY_DESTRUCTURING.witness(),
        EmissionSite::ArrayDestructuring,
    ),
    "ArrayPatternProtocol::ARRAY_DESTRUCTURING must emit all four 7.4 obligations at \
     compile_array_destructure_from_value_locals"
);
```

**K3 is retired.** It asserted `ALL_WITNESSES.len() == 7`, which is the
`ALL.len() == 29` shape round 1's §13.3 showed cannot detect its own omission —
the count is exactly what forgetting a row preserves. Ledger **IC-4** used to
justify keeping it on the grounds that "the strong form is not available: each
constant's body is a different four-argument expression, not a row". That reason
was wrong. A `macro_rules!` row can carry an expression fragment, so the
`iterator_witnesses!` expansion above generates the constants and the census from
one row list exactly as `emission_sites!` does for the sites — and an alias row
removes the `ARRAY_INDEX_WALK_RESUMABLE` wart as a bonus. Completeness is now
definitional and K1 is total rather than conditional on a hand-maintained
census. IC-4 is closed by the macro, not by a length check.

And one in `operations.rs`, which is the assertion the brief asked for. Extend
the file's existing import line (`operations.rs:15`) rather than path-qualifying
at the use site:

```rust
use crate::iterator_obligations::{site_emits, EmissionSite, IteratorObligation};
```

```rust
// (J10) The catalog may not credit an emitter arm that no acquisition has
//       accepted **for this row's own operation**. It reads
//       `iterator_obligations::site_emits`, so a future divergence between the
//       two tables fails to build.
//
//       Per-obligation, and that is what makes B1's split enforceable rather
//       than conventional. Asking only "is this site witnessed for *some*
//       obligation" accepts `EmissionSite::CallArgumentSpread` on the
//       `IteratorClose` row — the exact mistake `SYNC_CLOSE_SITES` exists to
//       prevent — because the same site legitimately emits `GetIterator`.
//       `StatementEmissionRow` therefore gains `pub obligation:
//       IteratorObligation`, with `AsyncIteratorClose` mapping to
//       `IteratorObligation::IteratorClose`.
const _: () = {
    let mut i = 0;
    while i < STATEMENT_EMISSION_ROWS.len() {
        let row = STATEMENT_EMISSION_ROWS[i];
        let mut j = 0;
        while j < row.sites.len() {
            assert!(
                site_emits(row.sites[j], row.obligation),
                "a statement-emission row credits a site that no witness constant discharges by \
                 emission of that row's own operation"
            );
            j += 1;
        }
        i += 1;
    }
};

// (J11) …and the converse: every site must be credited by some catalog row, so
//       a variant cannot exist purely to satisfy K1.
const _: () = {
    let mut i = 0;
    while i < EmissionSite::ALL.len() {
        let mut found = false;
        let mut j = 0;
        while j < STATEMENT_EMISSION_ROWS.len() {
            let sites = STATEMENT_EMISSION_ROWS[j].sites;
            let mut k = 0;
            while k < sites.len() {
                if sites[k] as u8 == EmissionSite::ALL[i] as u8 {
                    found = true;
                }
                k += 1;
            }
            j += 1;
        }
        assert!(found, "an EmissionSite is named by no catalog row");
        i += 1;
    }
};
```

K1 ∧ J10 ∧ J11 is the triangle: **site ⇒ witness**, **catalog ⇒ site (for the
row's own obligation)**, **site ⇒ catalog**. Adding a site without a witness
fails K1; adding a site without a row fails J11; naming a site in a row that no
witness reaches *for that operation* fails J10; and adding a site at all fails
`emission_sites_are_backed` unless it names a real function.

**What the triangle does not close, stated exactly.** All four ties quantify over
witness *constants*. No const expression ties a witness constant to an IR field
that holds it, so a Group B lane can add `EmissionSite::CallArgumentSpread`, add
`CALL_ARGUMENT_SPREAD_PROTOCOL` to the `iterator_witnesses!` rows, add the catalog
row, and simply **not** add `protocol` to `SpreadOperandIr`: K1, K2, J10, J11 and
`emission_sites_are_backed` all pass, and the acquisition still states nothing.
The earlier claim that "a new site cannot land without its witness and its row in
the same patch" and that "Group B is atomic per item — there is no half-landing"
was therefore too strong; what is checked is that the *constant* exists, not that
any construct holds it. The constant is even shielded from `dead_code` by its own
census entry, which is round 1's "survival by `pub`" shape one visibility level
down.

The closure is per-construct newtypes at the IR field, as A3 now does for array
patterns: a field whose type has one inhabitant makes the constant reachable from
exactly one struct field, and a missing field is `E0063`. Group B must do the
same for `SpreadOperandIr` and `YieldForm`, and the atomicity claim holds only
for items that do.

### A3. The witness field on `ArrayDestructuringPatternIr`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayDestructuringPatternIr {
    pub elements: Vec<ArrayDestructuringElementIr>,
    /// How this pattern's own `GetIterator` discharged the four 7.4
    /// obligations.
    ///
    /// **Per pattern, not per statement**, because 8.6.3 and 13.15.5.5 acquire
    /// a fresh iterator for every ArrayBindingPattern / ArrayAssignmentPattern
    /// — including each one reached through
    /// `DestructuringTargetIr::NestedArray`, which is why the field lives here
    /// rather than on `ExprIr::ArrayDestructure`.
    ///
    /// The type is `ArrayPatternProtocol`, **not** `IteratorProtocolWitness`.
    ///
    /// **The emitter must not read this**, and cannot: every reader of a
    /// witness's contents — including `ArrayPatternProtocol::witness` — is
    /// `pub(crate)` to `lila-ir` (round 1 §13.12), so a `lila-aot-wasm`
    /// arm that binds it and branches on it is `E0624`.
    /// Non-optional, no `Default`.
    pub protocol: ArrayPatternProtocol,
}

/// The witness slot on `ArrayDestructuringPatternIr`. One inhabitant, private
/// constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayPatternProtocol(IteratorProtocolWitness);

impl ArrayPatternProtocol {
    pub const ARRAY_DESTRUCTURING: Self =
        Self(IteratorProtocolWitness::ARRAY_DESTRUCTURING_PROTOCOL);
    pub(crate) const fn witness(self) -> IteratorProtocolWitness { self.0 }
}
```

**Why a newtype, and this is the correction to M1a.** M1a claimed the guarantee
bought by the required field is that "the author must name a constant that lives
beside its premises". With the bare witness type, the field's type is the *whole
witness domain*: `protocol: IteratorProtocolWitness::NO_ITERATION` — or
`::ARRAY_INDEX_WALK`, which assumes away all of 23.1.3.x — compiles at both
`lowering.rs` construction sites and every const assertion still passes, because
K2 pins the constant's **contents**, not which field may hold it. The guarantee
was "a constant", not "the right constant". A newtype with one inhabitant and a
private constructor makes any other witness `E0308`, and K2 asks its question of
`ArrayPatternProtocol::ARRAY_DESTRUCTURING.witness()` so the accessor has a const
consumer rather than surviving on `pub(crate)`.

The same hole is open on the three `ForOf*` `protocol` fields, where
`lowering.rs` already *selects* the constant with an `if`/`else` chain — precisely
the shape a copy-paste gets wrong. It is recorded as ledger **IC-6** rather than
fixed here: unlike the array pattern, those three fields legitimately admit
different witnesses, so the newtype would have to be `ForOfProtocol` with three
inhabitants and would not by itself stop the wrong one being chosen. Closing it
means moving the choice into a function that takes the `KindSet` and returns the
newtype, which is a `lowering.rs` change this area does not own.

Why this placement and not `ExprIr::ArrayDestructure`, stated because it is the
one design decision in Group A that a reviewer might want to overturn:

1. **It is where the spec puts it.** One `GetIterator` per array pattern,
   nested patterns included. A field on the `ExprIr` variant would witness the
   outermost acquisition and silently cover none of the nested ones.
2. **It costs two lines instead of four files.**
   `ExprIr::ArrayDestructure { value, pattern, assignment }` is matched
   exhaustively without `..` at `lila-aot-wasm/src/expressions.rs:1326` and
   `:3086` and at `lila-ir/src/lib.rs:2159` and `:2214`; a fourth field is
   `E0027` at all four. `ArrayDestructuringPatternIr` is matched exhaustively
   **nowhere** (§2.1), so a second field is `E0063` at exactly its two
   constructions.
3. **The compile error is the same error in the same crate.** `E0063 missing
   field 'protocol'` at `lowering.rs:32307` and `:32361`.

The field is `pub`, matching the three `ForOf*` `protocol` fields. Forgery is
already prevented one level down:
`IteratorProtocolWitness::new` and the four discharge constructors are private
to `iterator_obligations` (round 1 §12.2 D4), so the only inhabitants available
at a construction site are the named constants.

### A4. Two readers for the `abrupt` column

**A4a — the propagation-discipline column.** A closed enum whose three variants
are each backed by an emitter pair this formalization has read; nothing
speculative:

```rust
/// How an emitter arm gets an abrupt completion out of itself.
///
/// Closed, and each variant names real emitted code. This is the column that
/// makes `abrupt` mean something: a row that says "this operation may throw"
/// must also say what happens to the throw.
///
/// What this proves is a **table** claim: that the row states a discipline
/// consistent with its `abrupt` set. It does **not** prove that the named
/// emitter body implements that discipline — `lila-ir` cannot see
/// `lila-aot-wasm`. That boundary is ledger **L2**, restated as **IC-1**.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbruptDiscipline {
    /// The operation has no abrupt completion. `abrupt` must be empty.
    NoAbruptExit,
    /// Abrupt completions leave through the shared current-completion channel
    /// and close nothing: any iterator in scope belongs to the caller, and
    /// 7.4.8/7.4.9 have already set `[[Done]]`.
    /// (`emit_propagate_throw_from_locals_if_needed`,
    /// `emit_propagate_current_completion_if_throw`.)
    PropagateWithoutClose,
    /// The arm closes the acquired iterator and routes the two completion
    /// classes 7.4.11 step 4 distinguishes to the two helpers that implement
    /// them: `emit_iterator_close_preserving_current_throw` when a throw is
    /// already in flight (step 5 — the original wins), and
    /// `emit_iterator_close` otherwise (steps 6-8 — the close's error is not
    /// swallowed).
    ///
    /// It does **not** claim that every site exercises the break/return branch,
    /// and the first wording did. `compile_for_of_iterator` does: it branches on
    /// `saved_completion == THROW` and its else-arm really is the
    /// break/return/continue close. `compile_array_destructure_from_value_locals`
    /// — the site this contract adds — does not: there the `emit_iterator_close`
    /// call is the **normal**-completion close 8.6.3 step 5 requires, and the
    /// abrupt arm is unconditionally the step-5 helper for every abrupt kind.
    /// Both are step 4's two halves; only one site has a third case. A reader
    /// checking the old wording against `control_flow.rs` found a claim the site
    /// does not support — the species this area exists to delete.
    CloseOnAbruptExitWithStep4Precedence,
}

impl AbruptDiscipline {
    pub const fn name(self) -> &'static str {
        match self {
            Self::NoAbruptExit => "no abrupt exit",
            Self::PropagateWithoutClose => "propagate without close",
            Self::CloseOnAbruptExitWithStep4Precedence => "close, 7.4.11 step 4 precedence",
        }
    }

    const ALL: [Self; 3] = [
        Self::NoAbruptExit,
        Self::PropagateWithoutClose,
        Self::CloseOnAbruptExitWithStep4Precedence,
    ];
}

// The three names are pairwise distinct — K4's treatment, applied to this
// area's own new type. `name()` was defended as an exhaustiveness anchor and
// then shipped with zero callers workspace-wide, which is the same shape §1.1
// diagnoses for `CompletionAbruptKind`; K4, twenty lines away in the sibling
// file, sets a stricter standard. Six lines make it a `const` reader that
// catches a real mistake: a copy-pasted arm whose string was not changed makes
// two disciplines indistinguishable in every table the column feeds.
const _: () = {
    let mut i = 0;
    while i < AbruptDiscipline::ALL.len() {
        let mut j = i + 1;
        while j < AbruptDiscipline::ALL.len() {
            assert!(
                !str_eq(AbruptDiscipline::ALL[i].name(), AbruptDiscipline::ALL[j].name()),
                "two AbruptDiscipline variants render the same name"
            );
            j += 1;
        }
        i += 1;
    }
};
```

The `lib.rs` re-export of `AbruptDiscipline` is held until something outside the
crate needs it; today nothing does.

`StatementEmissionRow` gains `pub discipline: AbruptDiscipline` — a required
field with no `Default`, so a new row cannot omit it (`E0063`). The five
existing rows take:

| row | `abrupt` | discipline |
|---|---|---|
| `GetIterator` | `MAY_THROW` | `PropagateWithoutClose` |
| `IteratorStep` | `MAY_THROW` | `PropagateWithoutClose` |
| `IteratorValue` | `MAY_THROW` | `PropagateWithoutClose` |
| `IteratorClose` | `CONTROL_COMPLETIONS` | `CloseOnAbruptExitWithStep4Precedence` |
| `AsyncIteratorClose` | `CONTROL_COMPLETIONS` | `CloseOnAbruptExitWithStep4Precedence` |

```rust
// (J12) `abrupt` and `discipline` agree, in both directions.
//   (a) empty `abrupt` ⟺ `NoAbruptExit`. This is the M4 sentence: a MAY_THROW
//       row with no declared discipline is unrepresentable, because the only
//       discipline compatible with an empty `abrupt` is the one that says there
//       is no abrupt exit.
//   (b) a row claiming 7.4.11 step 4 must carry the completions step 4
//       distinguishes — the asymmetry has no content unless break/continue/
//       return are possible alongside throw.
const _: () = {
    let mut i = 0;
    while i < STATEMENT_EMISSION_ROWS.len() {
        let row = STATEMENT_EMISSION_ROWS[i];
        let is_none = matches!(row.discipline, AbruptDiscipline::NoAbruptExit);
        assert!(
            row.abrupt.is_empty() == is_none,
            "a statement-emission row's abrupt set and propagation discipline disagree"
        );
        if matches!(
            row.discipline,
            AbruptDiscipline::CloseOnAbruptExitWithStep4Precedence
        ) {
            // A **membership scan**, not `row.abrupt.len() == 4`. The length
            // test passes on `&[Throw, Return, Break, Break]`, in which
            // `Continue` — the kind whose outer-label carve-out is the subtlest
            // part of 14.7.1.1 — is silently absent, and writing the slice out
            // by hand instead of reusing the `CONTROL_COMPLETIONS` alias is
            // exactly the plausible mistake. J13 already contains this idiom.
            let mut k = 0;
            while k < CompletionAbruptKind::ALL.len() {
                let mut found = false;
                let mut m = 0;
                while m < row.abrupt.len() {
                    if row.abrupt[m] as u8 == CompletionAbruptKind::ALL[k] as u8 {
                        found = true;
                    }
                    m += 1;
                }
                assert!(
                    found,
                    "a row claiming 7.4.11 step 4 precedence must admit all four abrupt kinds"
                );
                k += 1;
            }
        }
        i += 1;
    }
};
```

`CompletionAbruptKind::ALL` is added for this, with a bitmask `const _` proving
it lists each kind exactly once — a duplicate plus an omission is what preserves
a length. `CONTROL_COMPLETIONS` is then defined as `&CompletionAbruptKind::ALL`
rather than as a second hand-written list of the same four kinds.

**A4b — callee containment.** This is the reader that actually consumes
`SpecOperationIr::abrupt()`, and it is spec-derived rather than invented: an
operation's abrupt set must include the abrupt sets of the operations its own
definition invokes.

`StatementEmissionRow` gains `pub calls: &'static [SpecOperationIr]`, filled
from the spec text quoted in §1.2/§1.3:

| row | `calls` | from |
|---|---|---|
| `GetIterator` | `GetMethod`, `Call`, `Get` | 7.4.2 steps 3–5 |
| `IteratorStep` | `Call`, `Get`, `ToBoolean` | 7.4.8 → IteratorNext + IteratorComplete |
| `IteratorValue` | `Get` | 7.4.9 |
| `IteratorClose` | `GetMethod`, `Call` | 7.4.11 steps 3, 4.c |
| `AsyncIteratorClose` | `GetMethod`, `Call` | 7.4.12 steps 3, 4.c (`Await` has no variant) |

```rust
// (J13) Callee containment, **plus** the justification check without which the
//       containment claim is empty.
//
//       Containment alone cannot catch the mistake M4b names. Setting `Get` to
//       `NO_ABRUPT` makes `row.calls[j].abrupt()` an *empty* slice, so the
//       `while k < callee.len()` body never runs, nothing is asserted, and the
//       build stays green: containment is monotone in the wrong direction for a
//       weakened callee. The first version of this assertion shipped with that
//       hole and the claim "marking `Get` as `NO_ABRUPT` now fails the build"
//       was therefore false.
//
//       The repair is one extra scan per row: a row that claims an abrupt exit
//       must name at least one callee that can produce one. It passes on today's
//       five rows — GetIterator/IteratorStep/IteratorClose/AsyncIteratorClose
//       all list `GetMethod` or `Call` — and fails the instant `Get` becomes
//       `NO_ABRUPT`, because `IteratorValue`'s only callee is `Get`.
const _: () = {
    let mut i = 0;
    while i < STATEMENT_EMISSION_ROWS.len() {
        let row = STATEMENT_EMISSION_ROWS[i];
        assert!(
            !row.calls.is_empty(),
            "a statement-emission row names no callee; 7.4's operations all invoke something"
        );
        let mut justified = false;
        let mut c = 0;
        while c < row.calls.len() {
            if !row.calls[c].abrupt().is_empty() {
                justified = true;
            }
            c += 1;
        }
        assert!(
            row.abrupt.is_empty() || justified,
            "a statement-emission row claims an abrupt exit no callee it names can produce"
        );
        let mut j = 0;
        while j < row.calls.len() {
            let callee = row.calls[j].abrupt();
            let mut k = 0;
            while k < callee.len() {
                let mut found = false;
                let mut m = 0;
                while m < row.abrupt.len() {
                    if row.abrupt[m] as u8 == callee[k] as u8 {
                        found = true;
                    }
                    m += 1;
                }
                assert!(
                    found,
                    "a statement-emission row omits an abrupt completion its callee may return"
                );
                k += 1;
            }
            j += 1;
        }
        i += 1;
    }
};
```

J13 passes on today's values and is a *guard*, not a repair — unlike K1, which
must fail before the fix. Both kinds are legitimate; the difference is stated so
the dry-runner checks the right thing for each (§9.13, §9.14).

Honest residual, recorded as ledger **IC-2**: `calls` is hand-written, so a row
can weaken J13 by declaring fewer callees than the spec text gives. It cannot
*forge* anything — every entry is a real `SpecOperationIr` variant, and
declaring more callees only makes the check stricter.

### A5. `lib.rs` re-export additions

Inside the existing `pub use iterator_obligations::{…}` block: add
`ArrayPatternProtocol` only. `ALL_WITNESSES`, `site_is_witnessed`, `site_emits`
and `ALL_OBLIGATIONS` are `pub(crate)` by design (§10 P2) and must not appear
there; `ArrayPatternProtocol` must, because it is the type of a `pub` struct
field (see A3), while its `witness()` accessor stays `pub(crate)` so P2 is
unaffected.

Inside the existing `pub use operations::{…}` block, the original round added
`AbruptDiscipline`. The bounded IC-8 follow-up removes it together with
`StatementEmissionRow`, `TrackedGapRow`, `STATEMENT_EMISSION_ROWS` and
`TRACKED_GAP_ROWS`. All five are crate-private implementation details of the
catalog assembly; the public `SPEC_OPERATION_CATALOG`, entry type and accessors
remain. A downstream import of a raw evidence row is now `E0603`, so the
hand-written input tables cannot become a second catalog API.

`StatementEmissionRow::into_entry` and `TrackedGapRow::into_entry` are
`pub(crate)`, **not** `pub`; the row types and their two input tables are now
crate-private as well. Every field of `SpecOperationCatalogEntry` is private so
that the only producers are this module's three constructors; a `pub`
`into_entry` on a struct whose own fields are all `pub` would give that back,
since any consumer crate could write

```rust
StatementEmissionRow { name: "ArraySpeciesCreate", abrupt: &[], discipline: NoAbruptExit,
                       calls: &[], sites: &[], .. }.into_entry()
```

and mint a catalog entry claiming `StatementEmission` for an unimplemented
operation, bypassing J7, J10, J12 and J13 — which quantify only over
`STATEMENT_EMISSION_ROWS`. Nothing outside `operations.rs` reads
`StatementEmissionRow`, `TrackedGapRow`, `STATEMENT_EMISSION_ROWS`,
`TRACKED_GAP_ROWS` or `into_entry`, so narrowing costs nothing.

---

## 4. Type mapping — Group B: specified in full, applied by nobody this round

Group B is the remainder of M1 and all of M2. Each item is **complete** — the
type, the construction site, and every pattern site that must be repaired — and
each requires edits in `crates/lila-aot-wasm`, which §10 P1 forbids this
round. The encoder does **not** write Group B. The lane note carries the patch
text and the line list so the batch that owns `lila-aot-wasm` can apply it as
one mechanical change.

Group B is atomic per item **for everything the const ties can see**: a new
`EmissionSite` variant without its witness fails K1, without its catalog row
fails J11, and without an `emission_sites_are_backed` arm fails `E0004`. What
they cannot see is the IR field — a lane can add the variant, the constant and
the row and never add `protocol` to `SpreadOperandIr`, and everything passes (see
A2's "what the triangle does not close"). So each Group B item must give its IR
field a **per-construct newtype**, as A3 does for array patterns:
`CallArgumentSpreadProtocol` with the single inhabitant
`CALL_ARGUMENT_SPREAD_PROTOCOL`, and `YieldForm::Delegate` carrying
`GeneratorDelegationProtocol`. With the newtype the item really is atomic; with a
bare `IteratorProtocolWitness` field it is not, and the field can also hold the
wrong constant.

### B1. `SpreadArgument` — call-argument spread (13.3.8.1)

```rust
/// The operand of `...expr` in an argument list, and the 7.4 discharge for the
/// iterator that spreading it acquires.
///
/// A one-field payload rather than a second tuple field so that the eight
/// `ExprIr::SpreadArgument(_)` / `(..)` patterns in the workspace keep
/// compiling; only the six that bind the operand need `.value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpreadOperandIr {
    pub value: Box<TypedExpr>,
    pub protocol: IteratorProtocolWitness,
}

// ir.rs:1654
SpreadArgument(SpreadOperandIr),
```

The witness. This is the one place where reading the emitter changes the answer
from what the brief assumed:

```rust
/// `ExprIr::SpreadArgument` — 13.3.8.1 ArgumentListEvaluation for `f(...it)`.
///
/// `GetIterator`, `IteratorStep` and `IteratorValue` are really emitted, by
/// `FunctionBuilder::emit_call_args_vector`, which open-codes the `@@iterator`
/// read, the callability and object checks, the once-only `next` cache and the
/// per-step `done`/`value` reads. (Cited by *name*, not by line: a concurrent
/// lane added 243 lines above it and every `:76xx` citation this section
/// originally carried is now wrong by about +150.)
///
/// `IteratorClose` is **not** emitted, and must not be: no abrupt exit of the
/// spread lowering leaves an iterator that 13.2.4.1/13.3.8.1 owes a close for.
/// Emitting a close there would be an *observable extra call to `return`*.
pub const CALL_ARGUMENT_SPREAD_PROTOCOL: Self = Self::new(
    GetIteratorDischarge::emitted(EmissionSite::CallArgumentSpread),
    IteratorStepDischarge::emitted(EmissionSite::CallArgumentSpread),
    IteratorValueDischarge::emitted(EmissionSite::CallArgumentSpread),
    IteratorCloseDischarge::assumed(IntactnessPremise::SpreadCloseOwedOnlyAfterAcquisition),
);
```

New premise. **The wording is the round-4 correction**, and the correction
matters: the premise was `SpreadLoopExitsOnlyWhenDone`, "every abrupt exit of a
spread loop happens after the iterator has been marked done", and the completed
read (§9.11) shows that is **false at exactly two lines**. Inside
`emit_call_args_vector` there are two abrupt exits that occur *after the iterator
object exists* and *before any step*: the `Get(iterator, "next")` throw, and the
invented `"Spread iterator next must be callable"` TypeError. Both leave an
iterator that is not done. The conclusion — no close owed — still holds, but for
a different reason: those steps are inside **GetIterator** itself, whose
abruption 13.3.8.1 step 3 propagates with `?` before the caller holds an
iteratorRecord at all.

Ship the premise with the reason that is true, because leaving the false wording
invites the next reader either to weaken it or to "fix" the emitter by adding a
close — which this contract itself says is an observable extra `return()` call:

```rust
/// No abrupt exit of the spread lowering leaves an iterator that 13.2.4.1 /
/// 13.3.8.1 owes a close for: loop-internal exits are step/value failures with
/// `[[Done]]` already set, and every pre-loop exit — including the
/// `Get(iterator, "next")` throw and the not-callable TypeError beside it — is
/// *inside* GetIterator, whose abruption propagates with `?` before the caller
/// holds an iteratorRecord.
///
/// [`PremiseKind::ImplementationFact`]: a claim about *our emitter's exit
/// structure*, established by reading it, not a condition on the user's
/// program. Verified by the completed read §9.11 required, over the whole of
/// `emit_call_args_vector`.
SpreadCloseOwedOnlyAfterAcquisition,
```

One question this premise does **not** settle, and it is open: whether ES2025's
GetIteratorFromMethod owes an `IteratorClose` when `Get(iterator, "next")`
throws. If it does, the premise needs a fifth clause, the two "none owed" rows in
§1.4 are incomplete, and the destructuring `GetIterator` path
(`finish_get_iterator_from_method`, which propagates the next-read throw before
the abrupt-target frame is opened) is missing a close. The vendored suite does
not settle it — the `yield-star-next-get-abrupt` family asserts only that the
reason propagates, with no `returnCount`. Resolve against the current spec text
before B1's premise is accepted.

with `kind()` returning `PremiseKind::ImplementationFact` — a new arm in the
exhaustive match at `iterator_obligations.rs:215`, plus a `name()` arm.

New site: `CallArgumentSpread => "emit_call_args_vector"`, backed by
`FunctionBuilder::emit_call_args_vector` (`pub(crate)`, in `functions.rs`;
locate it with `rg -n 'fn emit_call_args_vector' crates/lila-aot-wasm/src/functions.rs`
rather than by the stale `:7632`).

Catalog: `EmissionSite::CallArgumentSpread` joins `SYNC_PROTOCOL_SITES` for the
`GetIterator`, `IteratorStep` and `IteratorValue` rows **only** — not the
`IteratorClose` row, which would be false. `SYNC_PROTOCOL_SITES` therefore
splits into two constants:

```rust
const SYNC_PROTOCOL_SITES: &[EmissionSite] = &[
    EmissionSite::SyncForOfIterator,
    EmissionSite::AsyncForOfIterator,
    EmissionSite::ArrayDestructuring,
    EmissionSite::CallArgumentSpread,
    EmissionSite::GeneratorDelegation,
];
/// The subset that emits 7.4.11. `CallArgumentSpread` is deliberately absent:
/// its loop never owes a close (see `SpreadCloseOwedOnlyAfterAcquisition`).
/// Since J10 is per-obligation, putting it back is a **build failure**, not a
/// review catch: no witness discharges `IteratorClose` by emission at that site.
const SYNC_CLOSE_SITES: &[EmissionSite] = &[
    EmissionSite::SyncForOfIterator,
    EmissionSite::AsyncForOfIterator,
    EmissionSite::ArrayDestructuring,
    EmissionSite::GeneratorDelegation,
];
```

**Construction (1 site, `crates/lila-ir`):** `lowering.rs:25199`

```rust
ExprIr::SpreadArgument(SpreadOperandIr {
    value: Box::new(spread_value),
    protocol: IteratorProtocolWitness::CALL_ARGUMENT_SPREAD_PROTOCOL,
}),
```

**Pattern repairs, in `crates/lila-ir` (4 product + 2 `#[cfg(test)]`):**

| line | today | becomes |
|---|---|---|
| `lowering.rs:12509` | or-pattern arm `\| ExprIr::SpreadArgument(value)` | split into its own arm binding `operand`, using `&operand.value` |
| `ir.rs` (`visit_expr`'s `SpreadArgument` arm — was cited `:3000`, now `:3021` after this round's +21 lines, so grep rather than trust) | `ExprIr::SpreadArgument(value) => …visit_expr(value)` | `…(operand) => …visit_expr(&operand.value)` |
| `early_errors.rs:32` | or-pattern arm `\| ExprIr::SpreadArgument(operand)` | split into its own arm |
| `lib.rs` (was cited `:1298`/`:1303`, re-derived as `:1312`/`:1317`) | `ExprIr::SpreadArgument(ref value)` (both inside `#[cfg(test)] mod tests`) | `…(ref operand)` + `&operand.value` |

**Pattern repairs outside `crates/lila-ir` (6) — the lane note's patch:**
`lila-aot-wasm/src/data.rs:3363`, one site in `functions.rs` (cited `:7667`;
now about +150 after a concurrent lane), `planning.rs:2935` (or-pattern), `:3385`
(or-pattern), `:4768`, `:8106`. Plus one new arm in `emission_sites.rs`. The
**nine** `(_)`/`(..)` patterns — the list below was called "the eight" and has
nine entries — are untouched: `expressions.rs:1361`, `:3148`, one in
`functions.rs` (cited `:7642`, same +150 drift), `planning.rs:6182`, `:6843`,
`:6870`, `:7525`, `reference.rs:160`, `:411`. Re-derive every `functions.rs` line
with `rg` before applying; do not trust the numbers in this table.

### B2. `YieldForm` — `yield*` (14.4.14)

```rust
/// Which of 14.4.14's two productions a `StatementIr::GeneratorYield` is.
///
/// A closed two-element domain that was a `bool`. `yield*` owes four protocol
/// operations plus the close-then-TypeError branch of the throw resume mode
/// (§1.5); `yield` owes none of it. Under this type, requesting the delegation
/// without stating the discharge has no spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YieldForm {
    /// `yield expr`.
    Plain,
    /// `yield* expr`, and the 7.4 discharge for the iterator it acquires.
    Delegate(IteratorProtocolWitness),
}
```

`StatementIr::GeneratorYield`'s `delegate: bool` becomes `form: YieldForm`. The
field is **renamed**, deliberately: `delegate: YieldForm::Plain` reads as a
contradiction, and the rename makes every stale pattern an `E0026` naming the
old field rather than a type error further from the cause.

Witness:

```rust
/// `StatementIr::GeneratorYield { form: YieldForm::Delegate(..) }` — 14.4.14.
///
/// All four obligations are emitted, by the two delegation arms named together
/// as `EmissionSite::GeneratorDelegation` (§1.8 choice 3): the lowerer cannot
/// know which fires, because the emitter selects on
/// `FunctionExecutionKind::AsyncGenerator` at `control_flow.rs:1938`.
///
/// `IteratorClose` is `ByEmission` on the strength of the sync throw path,
/// which closes and *then* throws the TypeError in 14.4.14's prescribed order:
/// `emit_iterator_close` at `generator_delegation.rs:1088`, then
/// `"yield* iterator has no throw method"` at `:1098`.
pub const YIELD_STAR_DELEGATION_PROTOCOL: Self =
    Self::emitted_by(EmissionSite::GeneratorDelegation);
```

New site: `GeneratorDelegation => "compile_generator_delegation"`, whose
`emission_sites_are_backed` arm names **both** members so the E0599 guarantee
covers each:

```rust
EmissionSite::GeneratorDelegation => {
    let _ = FunctionBuilder::compile_generator_delegation;
    let _ = FunctionBuilder::compile_async_generator_delegation;
}
```

**Construction (1 site, `crates/lila-ir`):** `lowering.rs:15502-15508`, the
*only* place in the workspace that builds a `GeneratorYield`:

```rust
StatementIr::GeneratorYield {
    value,
    form: if delegate {
        YieldForm::Delegate(IteratorProtocolWitness::YIELD_STAR_DELEGATION_PROTOCOL)
    } else {
        YieldForm::Plain
    },
    suspend_state,
    resume_state,
    resume_mode,
},
```

The `delegate: bool` parameters of `lower_linear_generator_yield` (`:15468`) and
`lower_linear_generator_yield_value` (`:15480`) are **unchanged**, and so are
the twelve call sites that pass `yield_expression.delegate()` into them
(`lowering.rs:10290, 10373, 10384, 10426, 14347, 15352, 15566, 15620, 15912,
15989, 16122, 16620`). The bool→`YieldForm` conversion happens exactly once, at
the single point where the IR is built.

Honest residual, ledger **IC-3**: the bool survives as a parameter of two
private helpers. What M2 claims is precisely that the *IR variant* cannot be
built delegating-without-a-witness; it does not claim the word `bool` has been
eliminated from the lowering path. Closing that would mean rewriting twelve call
sites to duplicate the same `if`, which is churn without a new compile error.

**Pattern repairs, in `crates/lila-ir`: five**, all in `lib.rs`, all
`#[cfg(test)]` — `lib.rs:6027, 6056, 6097, 6150, 6323` spell `delegate:` and
break under the field rename (`grep -c delegate crates/lila-ir/src/lib.rs`
returns 5). This section used to say "none" and then hedge in the next sentence;
the count costs one command and belongs in the applier's estimate.

Every *other* `lila-ir` pattern on `GeneratorYield` either uses `..` or binds
only `value`/`resume_mode` (`lowering.rs:12108, 12851, 12857, 12914, 12988,
13182, 13267, 13273, 13281, 15998, 16002, 16015, 16019`; `ir.rs:2153, 2664`;
`lowering_helpers.rs:49`; `early_errors.rs:298`).

**Pattern repairs outside `crates/lila-ir` (7) — the lane note's patch:**
`control_flow.rs:1943` and `:2090` (full-field patterns, no `..`; also the
`*delegate` reads at `:1952`, `:2097`), `data.rs:2774`/`:2778`,
`emit.rs:621`/`:632`/`:806`, `planning.rs:4179`/`:4196`. Plus one new arm in
`emission_sites.rs`.

Note for the applier: `control_flow.rs:1943` and `:2090` are in the
generator-yield dispatch, ~6,500 lines away from the `emit_iterator_close*`
region (`:8451-8700`) that batch 5's iterator lane owns. The two do not overlap
textually, but they are the same file, so the patch is sequenced after batch 5
lands rather than beside it.

---

## 5. Group C — original array-literal spread design (superseded by §15)

C1 established that `ExprIr::ArrayLiteral` never carries a spread. The
obligation of 13.2.4.1 is nevertheless real, and it is discharged by a
*desugaring* with three branches (`lowering.rs:28120-28233`, mirrored at
`:15752-15845`):

| operand | lowered to | what is assumed |
|---|---|---|
| `possible_kinds` contains `Array` (`:28153`) — the staged-generator twin instead tests `kind == ValueKind::Array` (`:15784`) | pushed straight into `[].concat(…)` | that `Array.prototype.concat` + IsConcatSpreadable is observationally equal to 13.2.4.1's `GetIterator` loop, i.e. `%Array.prototype%[@@iterator]` and `%ArrayIteratorPrototype%.next` are intact **and** `@@isConcatSpreadable` is unpatched |
| `kind == ValueKind::Arguments` (`:28157`) | `Array.prototype.slice.call(args)` | the same, for the arguments exotic object |
| otherwise (`:28176`) | `Array.from(Array, operand)` | nothing: `Array.from` runs the real protocol (23.1.2.1) |

Two facts worth recording independently: the two lowerers use **different
guards** for the same premise (`possible_kinds.contains(Array)` versus
`kind == Array`), and the first branch's premise is strictly stronger than
`ArrayIteratorIntact` because `concat` additionally consults
`@@isConcatSpreadable` (23.1.3.1), which the iterator protocol does not.

The place that could carry a witness is the branch selection, and the type that
would carry it is:

```rust
/// Which desugaring `lower_array_literal` chose for one SpreadElement, and the
/// 7.4 discharge that choice implies. Exhaustive, no catch-all: a fourth fast
/// path (Set, Map, TypedArray) cannot be added without stating a witness.
enum ArraySpreadStrategy {
    ConcatArrayLike(IteratorProtocolWitness),
    SliceArguments(IteratorProtocolWitness),
    ArrayFromIterator(IteratorProtocolWitness),
}
```

selected by one private `fn array_spread_strategy(&self, operand: &TypedExpr) ->
ArraySpreadStrategy` and matched exhaustively at both lowerers.

**It is not written this round, and no field is added to `ExprIr::ArrayLiteral`.**
Reasons, in order:

1. A `protocol` field on `ArrayLiteral` would be attached to a node that never
   spreads. It would be decoration by AGENTS.md's test *and* a false claim by
   this area's own standard — the second failure being the worse one, since this
   contract exists to delete false claims.
2. Writing `ArraySpreadStrategy` restructures ~40 lines inside
   `lower_array_literal` and its staged-generator twin. `lowering.rs` is the
   shared hub; this area's allowance is a narrow enumerated *construction-site*
   allowance (§8 R3, two lines), and a walk restructure collides with batch 5's
   recursion-depth lane.

Ledger **IC-5** records the historical obligation. Section 15 supersedes this
shortcut design with the integrated direct ArrayAccumulation seam.

---

## 6. The runtime-checked ledger

Round 1's L1–L8 stand unchanged. These are this contract's additions. Each says
what cannot be a type, and why.

| id | Invariant | Why no type can carry it | The check that replaces it |
|---|---|---|---|
| **IC-1** | An emitter body actually implements the `AbruptDiscipline` its row declares. | `lila-ir` cannot see `lila-aot-wasm`; the dependency runs the other way, and an emitter arm's type is `(&mut Function) -> Result<(), EmitError>`, which has no channel for "this arm closes before propagating". This is round 1's **L2** in a new suit, and it is stated here so the A4a claim is not over-read: J12 proves the *table* is coherent, not that the emitter is. | Nothing, this round. The lane note's scope type (§7 of the note) is the design that would close it; it lives in `lila-aot-wasm`. |
| **IC-2** | A `StatementEmissionRow`'s `calls` column lists every operation the row's spec definition invokes. | The column is transcribed from spec text by hand. Nothing in the crate can read ECMA-262. | Bounded rather than checked: every entry is a real `SpecOperationIr` variant (type-checked), and under-listing only *weakens* J13 — it can never forge a containment. Over-listing makes the check stricter. |
| **IC-3** | The lowering path carries no `bool` standing for "is this a delegation". | `lower_linear_generator_yield{,_value}` keep a `delegate: bool` parameter; the conversion to `YieldForm` happens once at the IR construction. Removing the parameter means twelve duplicated `if`s and buys no new compile error. | Nothing. The mistake M2 names — building the IR variant delegating without a witness — is `E0063`/`E0308` regardless. |
| **IC-4** | `ALL_WITNESSES` lists every witness constant. | ~~Stable Rust has no way to enumerate a type's associated constants; the constants are four-argument expressions, not rows a macro can expand twice.~~ **That reason was wrong.** A `macro_rules!` row carries expression fragments perfectly well. | **CLOSED by a type.** `iterator_witnesses!` expands one row list into both the `pub const`s and `ALL_WITNESSES`, exactly as `emission_sites!` does for the sites; an alias row (`ARRAY_INDEX_WALK_RESUMABLE => Self::ARRAY_INDEX_WALK`) even removes the aliasing wart. K3's length check is **retired** — it is the shape that cannot detect its own omission — and K1 is now total rather than conditional on a hand-maintained census. |
| **IC-5** | An array-literal SpreadElement states how it discharged 13.2.4.1. | Historically there was no IR node: lowering erased spread through `concat` / `Array.from` (C1). | **Closed in §15.** Spread-bearing literals now create `ArrayAccumulationIr`; every `ArraySpreadIr` requires the one-inhabitant `ArraySpreadProtocol`, and the backend emits the general iterator protocol with no dense shortcut. |
| **IC-7** | The `ForOf*` `protocol` fields hold the *right* witness, not merely *a* witness. | The three fields legitimately admit three different constants, and `lowering.rs` selects between them with an `if`/`else` chain — the shape a copy-paste gets wrong. A one-inhabitant newtype (A3's `ArrayPatternProtocol`) cannot apply. Closing it means moving the choice into a function that takes the `KindSet` and returns a `ForOfProtocol`, which is a `lowering.rs` restructure. | Nothing at the field. `ForOfLoweringIr::into_statement_and_kind` now *reads* the witness on the way out and `debug_assert`s the two conditions that are checkable — an `Empty` statement must carry `NO_ITERATION`, and a real `ForOf*` statement must not — which also replaces the unread `protocol()` accessor. |
| **IC-8** | Raw operation evidence is not part of the public API. | **CLOSED (2026-08-13).** `AbruptDiscipline`, `StatementEmissionRow`, `TrackedGapRow`, `STATEMENT_EMISSION_ROWS` and `TRACKED_GAP_ROWS` are `pub(crate)` and removed from `lib.rs`; no Rust consumer existed outside `operations.rs`. | The visibility boundary is the check: downstream imports are `E0603`. Consumers use the privately assembled `SPEC_OPERATION_CATALOG`, `spec_operation_catalog` and `find_spec_operation`; catalog contents and runtime behavior are unchanged. |
| **IC-6** | `lila-aot-wasm` acquisition sites that no `lila-ir` construct reaches are witnessed. | Some acquisitions have no IR construct at all — the ~15 builtin consumers of `IfAbruptCloseIterator` (§1.7) are emitted from `StandardBuiltinId` arms, not from user-program IR. A witness on an acquisition that the user's program does not spell has nothing to attach to. | Nothing, and deliberately not the same thing as a gap: the builtins' close discipline is pinned by the five CLI fixtures in §2.6 and by Test262. Named here so the next reader does not mistake `EmissionSite`'s small variant set for a claim that only that many arms run the protocol. |

---

## 7. Mistake-class table

| # | Plausible mistake | Today | After this contract |
|---|---|---|---|
| **M1a** | Add or edit an array destructuring pattern without saying how its `GetIterator` discharged 7.4 — including a *nested* pattern, which acquires its own iterator. | `ArrayDestructuringPatternIr { elements }` compiles. Nothing is stated; the site is invisible. | **`E0063` missing field `protocol`** at both `lowering.rs` construction sites. No `Default`, no `Option`, and `IteratorProtocolWitness::new` is module-private. **And the field's type is `ArrayPatternProtocol`, not `IteratorProtocolWitness`** — with the bare witness type the author had to name *a* constant, not *the right* constant, and `protocol: IteratorProtocolWitness::NO_ITERATION` compiled at both sites with every const assertion still passing. One inhabitant, private constructor, so anything else is `E0308`. **Group A.** |
| **M1b** | Add a call-argument spread path with no stated close discharge. | `ExprIr::SpreadArgument(Box::new(v))` compiles. | **`E0063`** on `SpreadOperandIr`. **Group B — specified, not applied.** |
| **M1c** | Add a fifth *iterator-consuming `ExprIr` variant* with no witness at all. | Nothing. | **Still nothing.** No type in Rust can force an arbitrary new enum variant to carry a field. The guarantee this contract buys is *per named construct*, and saying otherwise would be exactly the over-claim §0 C6 corrected. Stated in the acceptance checklist as a non-claim. |
| **M2** | Request 14.4.14's delegation — four protocol operations plus the close-then-TypeError branch — with a bare `true`. | `delegate: bool` (`ir.rs:1974`). One construction site, and nothing there says what is owed. | **Unconstructible**: `YieldForm::Delegate` has no nullary spelling; `form: true` is `E0308`; a stale `delegate:` pattern is `E0026`. **Group B — specified, not applied.** |
| **M3a** | The catalog credits an emitter arm that no acquisition has accepted. | Live: `SYNC_PROTOCOL_SITES` names `ArrayDestructuring` (`operations.rs:1012`) and no witness constant does. | **`E0080`** — const assert **K1**, which *fails on today's tree* and passes once `ARRAY_DESTRUCTURING_PROTOCOL` and A3's field exist. **Group A.** |
| **M3b** | Add an `EmissionSite` variant that no catalog row credits — a variant existing only to satisfy K1. | Nothing; round 1's L6 was exactly this, retired by hand. | **`E0080`** — const assert **J11**. **Group A.** |
| **M3c** | Add an `EmissionSite` variant and forget `EmissionSite::ALL`, silently making K1 and J11 partial. | `ALL` does not exist. | **Unrepresentable** — enum, `ALL` and `name()` are three expansions of one `emission_sites!` row list (A1), the `spec_operations!` shape. **Group A.** |
| **M4a** | A row whose `abrupt` says the operation may throw, with nothing said about how the throw leaves. | `abrupt` has zero readers (§2.5). | **`E0063`** for the missing `discipline` field, then **`E0080`** via const assert **J12** if the declared discipline and the `abrupt` set disagree. Scope: a *table* claim. Ledger **IC-1** states what it does not prove. **Group A.** |
| **M4b** | Mark an operation the iterator protocol depends on as non-throwing — commit `ca09433c1`'s shape. | `SpecOperationIr::abrupt()` is total on the variant and nothing reads it. | **`E0080`** — const assert **J13**, *including its justification clause*. Containment alone does **not** catch this: weakening `Get` to `NO_ABRUPT` empties the callee slice, so the containment loop body never runs and the build stays green. J13 therefore also asserts that a row claiming an abrupt exit names at least one callee that can produce one, which fails at `IteratorValue` — whose only callee is `Get` — the instant `Get` is weakened. **Group A.** |
| **M4c** | Claim 7.4.11 step-4 precedence on a row whose `abrupt` admits only `Throw`, i.e. claim an asymmetry with only one side. | Nothing. | **`E0080`** — J12 (b). **Group A.** |
| **M5** | Close with the wrong precedence: `emit_iterator_close` where step 5 requires the original throw to win, or `_preserving_current_throw` where step 6 requires the close's error to replace a `break`/`return`. | A choice between two similarly-named plain functions at 62 call sites. Silent wrong answer; `iterator-close-throw-get-method-abrupt.js` is the trace that separates them. | **Not typed.** The distinction lives entirely in `lila-aot-wasm`. `AbruptDiscipline` gives the fork a *name* in the IR crate's vocabulary so the emitter-side design has something to be checked against; the check itself is the lane note's scope type. Ledger **IC-1**. |
| **M6** | Add an abrupt exit out of an iteration region without closing — the failure `IfAbruptCloseIterator` exists to name. | `Option<IteratorCloseOnThrowLocals>::None` is a legal argument at every call site of the two functions that take it (`objects.rs:14383`, `builtins/standard.rs:4418`). One acquisition through `emit_get_iterator_from_value_locals` against 62 close sites, with nothing linking them. | **Not typed, and honestly bounded.** There is *no shipped-defect ledger entry and no git-log instance* of this class; five CLI fixtures already pin the behaviour (§2.6). It is a real structural gap, not a wound, and this contract does not inflate it. Designed in the lane note. |

---

## 8. Retrofit map

Ordered. Each step must leave `cargo check -p lila-ir` clean before the next
begins — that is the whole verification strategy, per the campaign's method.
Steps R1–R6 are Group A and are the encoder's work. R7 is the note.

**R1 — `iterator_obligations.rs`, the site domain.** Introduce `emission_sites!`
(A1) and re-declare the three existing variants through it. Promote
`ALL_OBLIGATIONS` out of `#[cfg(test)]` to a `pub(crate) const`. No behaviour
change, no new variant. `cargo check -p lila-ir --all-targets`.

**R2 — `iterator_obligations.rs`, the witness.** Add
`ARRAY_DESTRUCTURING_PROTOCOL`, `ALL_WITNESSES`, `site_is_witnessed`, and const
asserts K1, K2, K3. **After R2 the crate must fail K1 until R3 lands? No — K1
passes as soon as the constant exists.** K1's pre-state failure is a property of
the tree *before* R2, and the dry-runner verifies it by the counterfactual in
§9.13, not by a broken intermediate. Repair the `EmissionSite::ArrayDestructuring`
line-number citation here (§2.4).

**R3 — `ir.rs` and `lowering.rs`, the field.** Add `protocol` to
`ArrayDestructuringPatternIr` (A3). This makes `cargo check -p lila-ir` fail
with `E0063` at exactly two lines, which are then filled:

- `crates/lila-ir/src/lowering.rs:32307` —
  `Some(ArrayDestructuringPatternIr { elements })` becomes
  `Some(ArrayDestructuringPatternIr { elements, protocol: IteratorProtocolWitness::ARRAY_DESTRUCTURING_PROTOCOL })`
  (inside `lower_array_binding_pattern`, the 8.6.3 side).
- `crates/lila-ir/src/lowering.rs:32361` — the identical change inside
  `lower_array_assignment_pattern` (the 13.15.5.5 side).

**These two lines are the entirety of this contract's `lowering.rs` allowance.**
No other line of `lowering.rs` may be edited by Group A: not the walk structure,
not the recursion helpers, not the descriptor-shape helpers. The five
`ExprIr::ArrayDestructure` construction sites (`:14590, 31589, 31721, 31824,
31936`) are **not** touched — they pass `pattern` through unchanged, which is
the design's main practical dividend.

Both sides take the same constant: they are two different spec operations
running the same emitter, and the emitter distinguishes them by the `assignment`
flag, not by protocol.

**R4 — `operations.rs`, the ties.** Add const asserts J10 and J11. Repair the
five stale `control_flow.rs` citations in the `SYNC_PROTOCOL_SITES` doc comment
(§2.4).

**R5 — `operations.rs`, the `abrupt` readers.** Add `AbruptDiscipline`, the
`discipline` and `calls` columns on `StatementEmissionRow`, the five rows' new
values per A4, and const asserts J12 and J13. `TrackedGapRow` is **not** given a
discipline: a row that says "not implemented" has no propagation to declare, and
inventing one would be the false-claim shape.

**R6 — `lib.rs`.** Add `AbruptDiscipline` to the existing
`pub use operations::{…}` block. Nothing else; no new `mod` line — that line
belongs to the descriptor area.

**R7 — the lane note.** Write
`target/lane-notes/iterator-close-obligation-theory-integration.md` with: the
Group B patch (types, construction sites, and every one of the 13 out-of-crate
pattern lines), the §5 `ArraySpreadStrategy` design and its two insertion points,
and the emitter-side close-scope design with its paper trace (§9.15).

### Untouched, deliberately

- **`crates/lila-aot-wasm/**` in its entirety.** §10 P1.
- **`ExprIr::ArrayLiteral`.** §5 / C1.
- **`ExprIr::ArrayDestructure`'s `assignment: bool`.** It is also a closed
  two-element domain masquerading as a bool — 8.6.3 versus 13.15.5.5 are two
  different abstract operations — and it is *adjacent, not in scope*. Recording
  it here so the next lane finds it; typing it would touch four exhaustive
  patterns including two in `lila-aot-wasm`.
- **`ObligationDischarge::ByEmission`'s arity.** §1.8 choice 3 explains why it
  stays a single site.
- **Round 1's `pub(crate)` narrowing** (`iterator_obligations.rs:45-51`,
  §13.12). §10 P2.
- **The `#[cfg(test)]` patterns at `lib.rs:2159` and `:2214`**, which spell
  `ExprIr::ArrayDestructure { value, pattern, assignment }` exhaustively without
  `..`. A3 does not reach them — the new field is on
  `ArrayDestructuringPatternIr`, not on the variant. Recorded because it is
  exactly what a variant-level field *would* have broken, and it is the second
  half of A3's cost argument.

---

## 9. Dry-run corpus and expected traces

Symbolic execution against the code, on paper. No `cargo` command and no
Test262 run is part of this contract's stage. All twelve JavaScript files were
confirmed present:

```sh
ls test262/vendor/test262/test/language/statements/for-of/iterator-close-*.js
```

### 9.1 `for-of/iterator-close-via-break.js` — the baseline

`break` out of a `for-of` body. `LoopContinues` is false, so 8.6.2 step 7.b
closes with a `break` completion. Covered today by
`SYNC_ITERATOR_PROTOCOL` on `StatementIr::ForOfIterator`; the close predicate is
`emit_iterator_close_condition_i32` (`control_flow.rs:8451`), verified by round
1 §9.1 to be exactly `¬LoopContinues`. **Trace expectation: unchanged by this
contract.** Its role is to fix what a discharged obligation looks like before
the newly covered constructs are compared against it.

### 9.2 `for-of/iterator-close-throw-get-method-abrupt.js` — M5's fork

`GetMethod(iterator, "return")` throws while a throw is already in flight.
7.4.11 step 5 fires before step 6: **the original throw wins and the close's
error is swallowed**. The emitter must be on the
`emit_iterator_close_preserving_current_throw` side.
**Trace expectation: unchanged.** It is quoted here because it is the file that
demonstrates the two close helpers are a semantic fork rather than a naming
preference, which is the whole justification for `AbruptDiscipline`'s third
variant carrying the words "both sides".

### 9.3 `for-of/iterator-close-via-return.js` — the other side of step 4

`return` from inside the body: abrupt but not a throw, so step 5 does not fire
and an error from the close is **not** swallowed (step 6). Paired with 9.2 this
pins the asymmetry from both directions. **Trace expectation: unchanged.**

### 9.4 `assignment/dstr/array-elem-iter-thrw-close.js` — M3 and M1a together

The iterator's step throws during array destructuring; the iterator must be
closed. Path today: `ExprIr::ArrayDestructure` → `compile_array_destructure_to_locals`
→ `compile_array_destructure_from_value_locals` → the `done == 0` guard at
`:7726` → `emit_iterator_close_preserving_current_throw` (`:7729`).

**Trace expectation after Group A:** identical emitted bytes. The only change is
that the `ArrayDestructuringPatternIr` reaching that emitter now carries
`ARRAY_DESTRUCTURING_PROTOCOL`, which the emitter cannot read (`E0624` if it
tried). The dry-runner must confirm the *no-byte-change* claim by inspection —
the field is `Copy`, adds no allocation, and no `lila-aot-wasm` code path
branches on it.

### 9.5 `assignment/dstr/array-elem-iter-nrml-close-skip.js` — the question the row actually asks

`[ _ ] = vals` where `next()` returns `{done: true}` immediately. Required:
`nextCount === 1`, `returnCount === 0`. The close must be **skipped** because
`[[Done]]` is true.

This is the trace that settles C6. It shows the destructuring site runs
`GetIterator`, `IteratorStep`, `IteratorValue` *and* `IteratorClose`, with
`IteratorClose` under 8.6.3 step 5's `[[Done]]` guard — which is exactly what
`control_flow.rs:7707` emits. **So `ARRAY_DESTRUCTURING_PROTOCOL` may honestly
be `emitted_by`, and the catalog's row was never a lie.** Had this trace come
out the other way, K1 would have had to be satisfied by *deleting* the row.

### 9.6 `assignment/destructuring/target-assign-throws-iterator-return-get-throws.js`

The assignment target throws *and* the `return` getter throws. Step 5: the
original wins. This traces M5 at a site that is not a for-of loop, which is what
proves the lane note's scope design must handle more than loop bodies.
**Trace expectation: unchanged; a requirement on the note, not on this
contract's code.**

### 9.7 `yield/star-rhs-iter-thrw-violation-no-rtrn.js` — M2 exactly

`yield*` receives `throw()`, the inner iterator has neither `throw` nor
`return`. Required: a TypeError from 14.4.14's close-then-throw branch. Emitter:
`generator_delegation.rs:1088` closes (a no-op, since `return` is absent —
7.4.11 step 4.b returns the completion), then `:1098` throws.

**Trace expectation:** unchanged today; under Group B this entire branch is owed
on the strength of `YieldForm::Delegate(YIELD_STAR_DELEGATION_PROTOCOL)` rather
than on a `true`.

### 9.8 `yield/star-rhs-iter-thrw-violation-rtrn-call-err.js` — M2 × M5

Same violation path, but `return()` exists and throws. Two independent unstated
facts today: *that* delegation is owed (the bool) and *which precedence* the
close uses. Group B states the first; the second stays ledger IC-1.

### 9.9 `yield/star-rhs-iter-rtrn-res-done-err.js` — the third resume mode

`yield*`'s `return` path. Shows `YieldForm::Delegate` owes obligations under all
of next/throw/return, not only the throw path 9.7 and 9.8 cover — which is why
the witness is `emitted_by` on all four obligations rather than on a subset.

### 9.10 `array/spread-err-sngl-err-itr-step.js` — the trace that refuted C1

`[...iter]` where `next()` throws. The file's own `info:` block cites
`sec-runtime-semantics-arrayaccumulation` and shows `GetIterator` + a
`ReturnIfAbrupt` loop with **no** `IteratorClose`.

Dry run against our lowering: `lower_array_literal` (`:28120`) sees a spread, so
it never builds an `ExprIr::ArrayLiteral` containing one. `iter`'s
`possible_kinds` is `{Object}`, so branch 3 fires and the expression becomes
`Array.from(Array, iter)` — the real protocol, inside the builtin. The throw
propagates from `Array.from`.

**This is the trace that establishes there is nothing on `ExprIr::ArrayLiteral`
to witness**, and the reason §5 exists instead of a fourth field. The
dry-runner should re-derive it rather than accept it: read `:28120-28233` and
confirm no path constructs an `ArrayLiteral` whose elements contain a spread.

### 9.11 `call/spread-err-mult-err-itr-step.js` — M1b, and the premise to complete

`f(0, ...iter)` where `next()` throws. Path: `lower_call_args_expanding_spread`
(`:25188`) builds `ExprIr::SpreadArgument`; `emit_call_args_vector`
(`functions.rs:7632`) open-codes the protocol and propagates the throw with
**no** close — which §1.4 says is correct.

**This read is now complete, and it changed the premise's wording.** Two abrupt
exits inside `emit_call_args_vector` occur *after the iterator object exists* and
*before any step*: the `Get(iterator, "next")` throw, and the invented
`"Spread iterator next must be callable"` TypeError beside it. Both leave an
iterator that is **not** done, so `SpreadLoopExitsOnlyWhenDone` — "every abrupt
exit of a spread loop happens after the iterator has been marked done" — is false
at exactly those two lines. The *conclusion* survives: those two steps are inside
**GetIterator**, whose abruption 13.3.8.1 step 3 propagates with `?` before the
caller holds an iteratorRecord, so no close is owed. §4 B1 now ships the premise
as `SpreadCloseOwedOnlyAfterAcquisition` with that reason. Leaving the old
wording in place would invite the next reader either to weaken it or to "fix" the
emitter by adding a close, which this contract itself calls an observable extra
`return()` call.

One clause remains open and is called out in §1.4 and §4 B1: whether ES2025's
GetIteratorFromMethod owes a close when `Get(iterator, "next")` throws. That is a
spec-text question, not an emitter-reading question.

### 9.12 `built-ins/Array/from/iter-map-fn-err.js` — M4's rider

The `mapfn` throws inside `Array.from`'s loop, so 23.1.2.1's inline
`IfAbruptCloseIterator` must run. This is a builtin consumer whose catalog row
carries a non-empty `abrupt` column that nothing reads today. After A4 the
column has two `const`-evaluated readers — and the honest statement is that
neither of them sees this file's behaviour at all: J12 and J13 check the table,
`Array.from`'s emitter is checked by this fixture. Ledger IC-1 says exactly
that, and §9.12's role is to make the boundary concrete rather than abstract.

### 9.13 A2 (adversarial, Rust-level) — the tie must fail before and pass after

Two counterfactual builds, both traced on paper:

**Before R2.** `EmissionSite::ALL = [SyncForOfIterator, AsyncForOfIterator,
ArrayDestructuring]`. `ALL_WITNESSES` = the six existing names, five distinct
values. `site_is_witnessed(SyncForOfIterator)` → `SYNC_ITERATOR_PROTOCOL`
discharges `GetIterator` by `ByEmission(SyncForOfIterator)` → `true`.
`AsyncForOfIterator` → `true` via `ASYNC_ITERATOR_PROTOCOL`.
`ArrayDestructuring` → scan all seven witnesses × four obligations:
`ARRAY_INDEX_WALK` (×2 names) and `STRING_CODE_POINT_WALK` and `NO_ITERATION`
are `ByAssumption` throughout; the two protocol constants are
`ByEmission(SyncForOfIterator)` / `ByEmission(AsyncForOfIterator)`. **No match.**
K1 asserts `false` → **`E0080` at `cargo check -p lila-ir`**, message "an
EmissionSite names an emitter arm that no IR construct's witness has accepted".

**After R2.** `ARRAY_DESTRUCTURING_PROTOCOL` is
`emitted_by(EmissionSite::ArrayDestructuring)`, so all four of its obligations
are `ByEmission(ArrayDestructuring)` and the scan matches on the first
obligation. K1 passes. K2 passes by `emits_every_obligation`. J10 passes because
`SYNC_PROTOCOL_SITES`'s three entries are each witnessed. J11 passes because
each of the three variants appears in `SYNC_PROTOCOL_SITES`.

**A tie that passed in both states would be decoration.** This one does not, and
the dry-runner must confirm the failing state by reverting R2's constant in a
scratch copy rather than by reasoning about it.

### 9.14 A4 (adversarial, Rust-level) — what J12/J13 catch, and what they cannot

**Catches.** A new `StatementEmissionRow` for, say, `IteratorStepValue` with
`abrupt: MAY_THROW` and no `discipline` field → `E0063`. With
`discipline: NoAbruptExit` → J12(a) fails. With
`discipline: CloseOnAbruptExitWithStep4Precedence` and `abrupt: MAY_THROW` →
J12(b) fails. With `calls: &[SpecOperationIr::Call]` and `abrupt: NO_ABRUPT` →
J12(a) fails first, and J13 would too.

**Cannot catch, and this goes in the ledger rather than in the claim:** a row
that declares `CloseOnAbruptExitWithStep4Precedence` while
`compile_for_of_iterator` propagates without closing. `lila-ir` cannot see
the body. IC-1.

**Also cannot catch:** a row that declares `calls: &[]`. J13's first assertion
rejects the empty slice for statement rows, but a row listing one callee where
the spec lists three passes. IC-2 bounds it.

### 9.15 A5 (paper only, for the lane note) — and the finding that changes the design

The brief proposed making `IteratorCloseOnThrowLocals` non-`Copy`, non-`Clone`,
`#[must_use]`, consumed by value at each exit emitter, so that an exit which does
not discharge the close is `E0382`/`E0505`.

**Traced on paper against the real code, that design fails immediately, and on a
*correct* program.** `emit_create_data_property_or_throw` (cited `objects.rs:14374`
when this was written; re-derived as `:14734`, and it has moved again since — grep
for the function name rather than trusting any of these numbers) takes one
`Option<IteratorCloseOnThrowLocals>` and uses it at **two** exits —
cited `:14548`/`:14599`, re-derived `:14908`/`:14959`, the non-configurable and
non-extensible throws. Both are correct;
both need the same iterator closed. A token moved at `:14548` is `E0382 use of
moved value` at `:14599`. The by-value design would reject the code it exists to
protect, and the natural repair — `Clone` — deletes the guarantee.

The design the lane note carries instead inverts what is gated: **not the close,
but the exit**. A `#[must_use]`, non-`Copy` scope value is created at
acquisition; the exit emitters reachable *inside* that scope are its own methods,
each taking an `AbruptExitKind` (a closed enum: `Throw`, `Return`, `Break`,
`Continue`) and selecting the 7.4.11 step-4 side from it. The scope's `Drop`
obligation is discharged by an explicit `finish()` that emits the
normal-completion close. Two exits then cost two method calls on one scope, and
picking the wrong precedence stops being a choice between two similarly-named
free functions — it becomes a value in a closed domain that the scope maps.

Second, decisive check: **the design reads nothing from `lila-ir`'s witness.**
The scope is built from `IteratorCloseOnThrowLocals` (`emit.rs:99`, eleven `u32`
locals) and an `AbruptExitKind`; neither `IteratorProtocolWitness` nor any of its
`pub(crate)` readers appears in it. Round 1's §13.12 narrowing therefore survives
intact, and the sibling relationship the brief demanded holds: the emitter's
token is a *different type in a different crate*, not an extension of the
witness. Had the paper trace needed the witness, this section would say the
design is wrong instead of shipping it.

---

## 10. Prohibitions

Stated as prohibitions, not preferences.

**P1 — no edit under `crates/lila-aot-wasm/`.** This campaign works in the
spec/IR layer while batch 2 verifies in the backend builtins and batch 5 owns
the iterator-emission lane. Group B and the §5 design are specified here and
applied by whoever owns that crate next. A contract that quietly edits thirteen
backend pattern lines to make its own claims land has spent another lane's
concurrency budget without asking.

**P2 — round 1's `pub(crate)` narrowing is not re-opened.** Every reader of a
witness's contents (`get_iterator`, `iterator_step`, `iterator_value`,
`iterator_close`, `discharge`, `is_fully_emitted`,
`ObligationDischarge::is_emitted`) stays `pub(crate)` to `lila-ir`
(`iterator_obligations.rs:45-51`, round 1 §13.12), and the two new helpers
`ALL_WITNESSES` and `site_is_witnessed` are `pub(crate)` for the same reason.
The emitter-side close token is a **sibling type living in
`lila-aot-wasm`**, not an extension of the witness. A design that makes the
brief's sentence "an abrupt exit that does not discharge the close fails to
compile" literally true by letting the emitter read the witness has spent round
1's payment for nothing.

**P3 — 7.4.12's async close emission ordering is out of scope.**
`compile_async_for_of_iterator` (`control_flow.rs:5577`) open-codes it. Not
read, not changed, not claimed.

**P4 — no new `mod` line in `lib.rs`.** That line belongs to the descriptor
area. This contract adds no module.

**P5 — `lowering.rs` gets two lines.** `:32307` and `:32361`, both listed in
§8 R3. Not the walk structure, not the recursion helpers, not the
descriptor-shape helpers at `:25598-:26177`.

---

## 11. Acceptance checklist

The encoder is done when all of the following hold. Items marked **claim** are
what this contract asserts; items marked **non-claim** are stated so nobody
reads more into the result than is there.

1. `cargo check -p lila-ir --all-targets` is clean, and `cargo xc` is clean.
2. **claim** Reverting `ARRAY_DESTRUCTURING_PROTOCOL` in a scratch copy makes
   K1 fail with `E0080` (§9.13). A tie that passes in both states has not been
   built.
3. **claim** Deleting `protocol` from either `lowering.rs:32307` or `:32361` is
   `E0063 missing field 'protocol'`.
4. **claim** Adding an `EmissionSite` variant without a witness *constant*
   fails K1; without a catalog row fails J11; without an
   `emission_sites_are_backed` arm fails `E0004`; and it cannot be omitted from
   `EmissionSite::ALL` at all. **Nor from `ALL_WITNESSES`**, which
   `iterator_witnesses!` now generates from the same rows as the constants.
   *Not* claimed: that any IR construct actually holds the constant — see A2's
   "what the triangle does not close" and the per-construct newtype that closes
   it for array patterns.
4b. **claim** Crediting a site on the `IteratorClose` row when no witness
   discharges `IteratorClose` by emission there fails J10, which is
   per-obligation.
5. **claim** Adding a `StatementEmissionRow` without `discipline` or `calls` is
   `E0063`; with `abrupt: MAY_THROW, discipline: NoAbruptExit` is `E0080`.
6. **claim** Marking `SpecOperationIr::Get` as `NO_ABRUPT` is `E0080` at J13 —
   which required J13's *justification* clause, not containment alone.
   Containment is vacuous for a weakened callee: an empty callee slice makes the
   containment loop body never execute. The clause "a row that claims an abrupt
   exit names at least one callee that can produce one" is what fires, at
   `IteratorValue`, whose only callee is `Get`.
7. **claim** No hunk in `crates/lila-aot-wasm/` is attributable to this
   contract. **Not dischargeable by a bare `git status`**: the checkout is
   shared, and at the time this checklist was written `git status` already
   listed `crates/lila-aot-wasm/src/functions.rs` as modified by a concurrent
   lane (a `MethodCallDestination`/`DestinationWritten` typestate for
   `emit_method_call`, unrelated to iterator obligations, and not one of the four
   batch-2 files). An integrator running item 7 literally reads a false negative.
   The owned file set is: `crates/lila-ir/src/iterator_obligations.rs`,
   `operations.rs`, `ir.rs`, `lowering.rs`, `lib.rs`, plus `docs/` and
   `target/lane-notes/`.
8. **claim** The six stale `control_flow.rs` citations of §2.4 are repaired in
   the two owned files.
9. **non-claim** Nothing here makes an arbitrary *new* iterator-consuming
   `ExprIr` variant carry a witness (M1c). The guarantee is per named construct.
10. **non-claim** Nothing here proves an emitter body implements the discipline
    its row declares (IC-1), nor that a `calls` column is complete (IC-2).
11. **non-claim** `SpreadArgument`, `GeneratorYield`'s `YieldForm` and the
    array-literal spread strategy were **not** landed by Group A. Sections 13,
    14 and 15 supersede this historical non-claim with the three integrated
    typed seams.
12. Emitted bytes are unchanged **by Group A**. Every Group A change is a
    compile-time addition: a new field on a struct the emitter only borrows, new
    `const` items, and two macros that re-declare existing items.
    `ForOfLoweringIr::into_statement_and_kind`'s two `debug_assert`s add no
    emitted Wasm.

    **One exception, added in round 4 and deliberately not silent:** the IC-5
    fix narrows `lower_array_literal`'s spread guard from
    `possible_kinds.contains(Array)` to `possible_kinds.is_subset_of({Array})`,
    so a spread whose operand is not statically an array now lowers to
    `Array.from` instead of `[].concat`. That **changes emitted bytes**, and it is
    a bug fix rather than a refactor: `function f(x) { return [...x]; }` was
    appending a non-array iterable instead of iterating it, under a pristine
    realm. Rung G will be non-empty for that reason and no other; the diff should
    show changes only in functions containing an array-literal spread whose
    operand is not statically an `Array`.

---

## 12. Round-4 integration gate (integrator record)

Tree `1939975ad`, branch `claude/test-driven-rust-opus-pp6giw`. Batch 5 held the
build lock throughout; every command queued on it.

### 12.1 Compile gate

`cargo check -p lila-ir` **exit 0** (6 warnings, all baseline);
`cargo xc` **exit 0**, 0 errors; `cargo fmt --all` clean after formatting
`operations.rs`, `ir.rs` and `lib.rs`.

Group A compiled as written. Of §6.5's three ranked failure predictions, (a) the
rustfmt reflow of the `lib.rs` `pub use` block **did** occur and is fixed; (b)
the `emission_sites!` `#[$meta:meta]` passthrough compiled — `///` really does
desugar to `#[doc = "…"]` and deviation 1 of §6.3 is confirmed correct against
the compiler, not merely argued; (c) J13's four nested `const` `while` loops
evaluated without hitting a const-eval limit.

### 12.2 Counterfactuals — every const assertion was made to fail on purpose

The failure mode this area exists to remove is an assertion that passes in both
the pre-change and post-change tree. §11.2 asks for the K1 counterfactual by
name. All four were executed against the real tree — patch, compile, record,
restore, verify the restore by `md5sum`.

| # | Injected mistake | Diagnostic |
|---|---|---|
| **K1** (+K2, +J10) | `ARRAY_DESTRUCTURING_PROTOCOL` re-pointed at `EmissionSite::SyncForOfIterator`, so no witness names `ArrayDestructuring` | **three** `E0080`s: `an EmissionSite names an emitter arm that no IR construct's witness has accepted` (K1, `:746`); `ArrayPatternProtocol::ARRAY_DESTRUCTURING must emit all four 7.4 obligations…` (K2, `:759`); `a statement-emission row credits a site that no witness constant discharges by emission of that row's own operation` (J10, `operations.rs:1538`) |
| **M1a** | one of the two `ArrayDestructuringPatternIr` construction sites in `lowering.rs` drops `protocol` | **`E0063`** `missing field protocol in initializer of ir::ArrayDestructuringPatternIr` (`lowering.rs:32338`) |
| **M3d / K4** | `ArrayDestructuring`'s row copied `SyncForOfIterator`'s emitter name, so two variants denote one arm | **`E0080`** `two EmissionSite variants name the same emitter function` (`:791`) — and **only** K4 fired, so it is targeted rather than incidental |
| **M4b / J13** | `SpecOperationIr::Get` moved from `MAY_THROW` into `NO_ABRUPT` — the shape of shipped-defect commit `ca09433c1` | **`E0080`** `a statement-emission row claims an abrupt exit no callee it names can produce` (`operations.rs:1671`) |

Two results are worth more than the pass/fail:

- **K1 is not decoration.** It fails on the pre-`ARRAY_DESTRUCTURING_PROTOCOL`
  shape and passes after, which is the exact discrimination §11.2 asks for.
- **M3d was found during encoding rather than specified in the contract**, and
  K4 catches it alone — nothing else in the triangle (K1, J10, J11,
  `emission_sites_are_backed`) notices two variants sharing one emitter, because
  all four still resolve. That is the const assert earning `EmissionSite::name`
  its place as an input rather than as an unread renderer.

### 12.3 Historical integration record for Groups B and C

The round-3 blocker in §2.6 — the incomplete `SpreadLoopExitsOnlyWhenDone` read —
was discharged by the round-4 rewrite to `SpreadCloseOwedOnlyAfterAcquisition`.
What still blocks Group B is **only** concurrency and crate boundary:

- `SpreadArgument` needs six pattern repairs plus an `emission_sites.rs` arm in
  `lila-aot-wasm` (`data.rs`, `functions.rs`, `planning.rs` ×4).
- `YieldForm` needs seven out-of-crate pattern lines, two of them at
  `control_flow.rs:1943`/`:2090` — the same file batch 5's iterator lane is live
  in. §3.4's own sequencing note says to apply it *after* batch 5 lands.

Group C (`ArraySpreadStrategy`, ledger IC-5) restructures ~40 lines inside
`lower_array_literal` and its staged twin in `lowering.rs`, the crate's largest
contention surface, and changes emitted bytes — unverifiable without rung G.

None of the three were applied. Each is fully specified in the lane note; none
is blocked on analysis.

---

## 13. Generator-delegation integration (2026-08-12)

This section supersedes §4 B2, mistake-table row M2, checklist non-claim 11 and
§12.3 wherever they say `YieldForm` is not encoded. B1 (`SpreadArgument`) is
superseded by §14; Group C is superseded by §15.

The encoded seam is narrower and stronger than B2's first code sketch:

- `StatementIr::GeneratorYield { delegate: bool }` is now
  `StatementIr::GeneratorYield { form: YieldForm }`, where the closed domain is
  `Plain | Delegate(GeneratorDelegationProtocol)`.
- `GeneratorDelegationProtocol` is a one-inhabitant wrapper with a private
  constructor. Its only public value, `YIELD_STAR`, contains
  `IteratorProtocolWitness::YIELD_STAR_DELEGATION_PROTOCOL`. A const assertion
  asks through the wrapper and proves that all four obligations are discharged
  at `EmissionSite::GeneratorDelegation`; pointing the wrapper at any other
  otherwise-valid witness is therefore `E0080`.
- `EmissionSite::GeneratorDelegation` names the sync/async emitter family.
  `emission_sites_are_backed` resolves both
  `compile_generator_delegation` and `compile_async_generator_delegation`, as
  §1.8 choice 3 requires.
- The statement-emission catalog credits that site for `GetIterator`,
  `IteratorStep`, `IteratorValue`, `IteratorClose` and `AsyncIteratorClose`.
  K1/J10/J11 therefore cover the new site in both directions.
- The parser-facing `delegate: bool` remains on the two private lowering
  helpers, per ledger IC-3, and is converted exactly once at the sole
  `GeneratorYield` construction. Every backend consumer that observes the
  distinction matches `YieldForm` exhaustively.

No emitted instruction sequence was rewritten: the new exhaustive matches
enter the same delegation return before the plain-yield instructions, and the
same plain path otherwise. `generator_delegation.rs` is untouched.

This integration was dry-written. Static formatting, whitespace and module
boundary checks are recorded by the batch handoff; no Cargo command or runtime
test was run in this lane. The cheapest focused verification is
`cargo check -p lila-ir -p lila-aot-wasm`, followed by the existing focused
generator and iterator filters in T15.

---

## 14. Call-argument spread integration and the uninhabited dense strategy (2026-08-12)

This section supersedes §4 B1, M1b, §12.3 and every earlier sentence that says
the `SpreadArgument` witness is only specified.

`ExprIr::SpreadArgument(Box<TypedExpr>)` is now
`ExprIr::SpreadArgument(SpreadArgumentIr)`. The payload has two required
fields: the operand and a `SpreadArgumentProtocol`. The protocol wrapper has a
private constructor and one public inhabitant, `ARGUMENT_LIST`, so a new IR
construction that omits the discharge is `E0063` and one that substitutes a
for-of, destructuring or delegation witness is `E0308`.

The witness is deliberately partial:

| obligation | discharge | evidence |
|---|---|---|
| `GetIterator` | `ByEmission(CallArgumentSpread)` | `emit_call_args_vector` reads `@@iterator`, calls it, checks the returned object and caches `next` once |
| `IteratorStep` | `ByEmission(CallArgumentSpread)` | the same emitter calls cached `next`, checks the result object and reads/coerces `done` |
| `IteratorValue` | `ByEmission(CallArgumentSpread)` | the same loop reads `value` only after `done` is false |
| `IteratorClose` | `ByAssumption(SpreadCloseOwedOnlyAfterAcquisition)` | 13.3.8.1 propagates acquisition and step/value abrupt completions; adding a close would be observable extra `return()` behavior |

`EmissionSite::CallArgumentSpread` is name-resolved to
`FunctionBuilder::emit_call_args_vector`. The first three statement-emission
rows include it; the `IteratorClose` row uses a smaller `SYNC_CLOSE_SITES`
domain that excludes it. K1/J10/J11 therefore make either accidental catalog
claim a const-evaluation failure. The AOT pattern only unwraps `.value`; it
cannot inspect the protocol witness, so the existing evaluation order,
temporary-local plan and Wasm instruction sequence are preserved.

### Why `ArraySpreadStrategy::ProvenDense` does not land with it

Call-argument spread has no dense shortcut: every current spread operand enters
the general iterator loop. Array-literal spread is different and is desugared
earlier, in `lower_array_literal` and its staged-generator twin. The smallest
honest future domain is:

```rust
enum ArraySpreadStrategy {
    ProvenDense,
    GeneralIterator,
}
```

There is presently no valid constructor for `ProvenDense`. An inferred dense
array shape proves backing-layout facts, not that
`%Array.prototype%[@@iterator]` and `%ArrayIteratorPrototype%.next` retain their
initial values; even `[...[1, 2]]` must observe an iterator method patched
earlier in the script. The only related lowerer fact,
`array_prototype_mutated`, initializes to `true` and has no transition that
proves the realm intact. Selecting `ProvenDense` from `ValueKind::Array` or a
dense `HeapShape` would therefore certify a false premise. Declaring the enum
while making that variant unreachable would instead be speculative decoration.

At the time of the §14 integration, Group C remained open until either (a) a
realm/version witness made the intact premise constructible, or (b)
array-literal spread deleted the shortcut and used a direct/general iterator
accumulator. Section 15 records the subsequent choice of (b).

The change was dry-written only. No Cargo command or execution test ran in this
lane. The cheapest compile gate is
`cargo check -p lila-ir -p lila-aot-wasm`; the focused semantic gate is the
pinned `language/expressions/call` spread subtree followed by the existing T15
CLI iterator regression.

## 15. ArrayAccumulation integration (2026-08-13)

Group C chose option (b): the array-literal shortcut is deleted rather than
certified. Plain no-spread literals retain shaped `ExprIr::ArrayLiteral`;
spread-bearing literals lower to `ExprIr::ArrayAccumulation`. Every spread is
an `ArraySpreadIr` carrying the one-inhabitant
`ArraySpreadProtocol::ARRAY_ACCUMULATION`. Its witness credits
`EmissionSite::ArrayLiteralSpread` for GetIterator, IteratorStep and
IteratorValue, and its fourth slot names the implementation fact
`ArrayAccumulationDoesNotClose`. The backend cannot inspect the witness.

`ArrayAccumulationTargetIr::{Fresh, SuspensionOwned}` separates uninterrupted
evaluation from staged generator evaluation. `SuspensionOwned` requires
distinct `ArrayAccumulatorArraySlot` and
`ArrayAccumulatorU64NextIndexSlot` values, initialized before the first
element. Prefix elements are committed before a following suspension, and the
final expression returns the same fresh array.

The hidden next-index carrier stores an exact raw `u64`; it is never persisted
through an ECMAScript Number, and a contribution at `u64::MAX` throws before
addition can wrap. That is an explicit backend representation limit, not a
claim that an unbounded mathematical integer fits in the carrier. Property-key
creation still follows the specification's `ToString(𝔽(nextIndex))`:
the raw counter is converted to Number at that operation, including above
2^53. The array-index seam remains exact: indexes below `2^32 - 1` use direct
fresh-array storage, `2^32 - 1` becomes the ordinary named key
`"4294967295"` without changing `length`, and an elision there throws
RangeError.

There is no dense spread path and no inherited-setter lookup. There is also no
IteratorClose path: ArrayAccumulation propagates acquisition, step and value
abrupt completions directly. No Cargo or Test262 command ran in this dry-write;
the central verifier owns the compile, focused runtime and pinned
`language/expressions/array` gates.

## 16. IC-8 public-surface closure (2026-08-13)

The raw catalog inputs are no longer public API. `AbruptDiscipline`,
`StatementEmissionRow`, `TrackedGapRow`, `STATEMENT_EMISSION_ROWS` and
`TRACKED_GAP_ROWS` are crate-private, and `lila-ir` no longer re-exports them.
The public assembled catalog, its entry and status types, and its lookup
functions are unchanged.

This closes the "survival by `pub`" residue without changing a row, count or
emitter: downstream code can inspect the catalog but cannot couple itself to
the hand-written assembly tables. No Cargo or Test262 command ran in this
visibility-only follow-up; its compile and rustdoc gates remain central.
