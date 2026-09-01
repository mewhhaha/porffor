# Contract: spec-operation catalog evidence and the iterator-protocol obligation witness

Status: **normative for the encoder**. Stage: formalization only — no source was
edited to produce this document. Every count in it was obtained by reading the
tree at `claude/test-driven-rust-opus-pp6giw`, not estimated; the command that
produced each count is given so the dry-runner can re-derive it.

> **Read §13 first.** A dry-run discrepancy pass amended this document after the
> encoding landed. §13 supersedes every claim it names — including parts of §2,
> §3 A1/A3/A5/A6, §5 L1/L3/L4/L6, §6 mistake classes 1, 4, 7 and 8, §7, §10 and
> §12. Do not cite §§1–12 without checking §13.
>
> **Read §18 for the current synchronous Array `for-of` state.** It supersedes
> every earlier present-tense claim that `StatementIr::ForOfArray`,
> `compile_for_of_array`, or the synchronous `ARRAY_INDEX_WALK` remains live.
> Older counts and traces remain as historical evidence for why the shortcut
> was removed.
>
> **Read §19 for the current synchronous String `for-of` state.** It supersedes
> every earlier present-tense claim that `StatementIr::ForOfString`,
> `compile_for_of_string`, `STRING_CODE_POINT_WALK`, `StringIteratorIntact`, or
> `StringWalkIsCodePoint` remains live. Older counts and traces remain as
> historical evidence for why the shortcut was removed.

Owned area files:

- `crates/lila-ir/src/operations.rs`
- `crates/lila-ir/src/iterator_obligations.rs` (new)
- `docs/rust-rewrite/contracts/spec-operations.md` (new; redirect to this file)
- `docs/rust-rewrite/contracts/iterator-protocol.md` (new; redirect to this file)

Files this contract also requires touching, and which are **not** batch-2 files
(`intl_datetimeformat.rs`, `temporal*.rs`, `emitted_function.rs`,
`runtime_helpers.rs` are untouched throughout):
`crates/lila-ir/src/ir.rs`, `crates/lila-ir/src/lib.rs`,
`crates/lila-ir/src/lowering.rs`, `crates/lila-aot-wasm/src/control_flow.rs`,
`crates/lila-aot-wasm/src/emit.rs`, `crates/lila-aot-wasm/src/emission_sites.rs` (new).

---

## 0. How to read this, and one warning about clause numbers

ECMA-262 renumbers clause 7.4 (Operations on Iterator Objects) in most editions,
and 14.7.5.6's inner step letters shifted when `IteratorStepValue` was introduced.
**Abstract-operation names are normative in this document; clause numbers are
navigational.** Where the area brief and the edition in `test262/vendor` disagree
on a number, both are given. If the encoder finds a number that does not resolve,
follow the name and fix the number — do not follow the number.

Terminology used throughout:

- **Signature** of an abstract operation: its normal codomain plus the set of
  abrupt completion types it may return. This is what the catalog records today.
- **Evidence**: a value that can only be constructed if the thing it asserts is
  true of this codebase. This is what the catalog does *not* have today, and is
  the whole subject of Part A.
- **Obligation**: a spec step that must happen. **Discharge**: how our compiler
  accounts for it — by emitting it, or by assuming it away on a stated premise.
  Part B is about making the discharge a value rather than a silence.

---

## 1. Spec basis

### 1.1 Completion Records and propagation by construction (6.2.4, 5.2.3.4)

A Completion Record (6.2.4) is `{ [[Type]], [[Value]], [[Target]] }` with
`[[Type]] ∈ {normal, break, continue, return, throw}`. The repo models this as
`CompletionKindIr` (six inhabitants — the five spec types plus `Empty`, which
models the *empty* `[[Value]]`/`[[Target]]` sentinel promoted to a kind) and
`CompletionRecordIr<T>` (`operations.rs:566-775`).

The load-bearing property of 5.2.3.4 is that `?` and `!` make abrupt propagation
**structural**: a spec step written `Let x be ? Op(v)` cannot forget to propagate,
because forgetting is not expressible in the notation. Our analogue at the IR
level is the catalog's `abrupt` field — and it is exactly the place where the
guarantee is lost today, because `abrupt` is a free per-row argument
(`operations.rs:1197-1211`). A row can say `NO_ABRUPT` for an operation the
emitter emits with a throw path, and nothing notices. This is the shape of
commit `ca09433c1` (ToPrimitive abrupt completions compiled as if they could not
escape).

**Postcondition this contract imposes:** for every operation the shared
`SpecOperationIr` emitter handles, `abrupt` is a *function of the variant*, not
an argument. Nobody can pass the wrong set because there is no parameter to pass.

`UpdateEmpty(completionRecord, value)` (6.2.4, Completion Record clause) replaces
an empty `[[Value]]` and is the operation 14.7.5.6 applies to the loop body's
result before handing it to `IteratorClose`. Modelled by
`CompletionRecordIr::update_empty` (`operations.rs:731-755`). Measured: zero call
sites anywhere in the workspace outside its own unit test.

### 1.2 The iterator protocol (7.4), stated as four obligations

For an `iterate`-kind for-of head over expression `E`, the spec requires, in this
order and with these observable effects:

**O1 — GetIterator(obj, sync)** (7.4.2, plus GetIteratorFromMethod in editions
that split it).
1. `method` ← `? GetMethod(obj, %Symbol.iterator%)` — an observable `[[Get]]` on
   `obj`, walking the prototype chain.
2. If `method` is undefined, throw a **TypeError**.
3. `iterator` ← `? Call(method, obj)` — the user's `@@iterator` function runs,
   with `obj` as receiver.
4. If `iterator` is not an Object, throw a **TypeError**.
5. `nextMethod` ← `? Get(iterator, "next")` — a second observable `[[Get]]`,
   performed **once**, not once per step.
6. Return Iterator Record `{ [[Iterator]]: iterator, [[NextMethod]]: nextMethod,
   [[Done]]: false }`.

Note step 5's *once*: an implementation that re-reads `next` each iteration is
observably wrong for an iterator whose `next` is an accessor or is reassigned
mid-loop.

**O2 — IteratorStep(iteratorRecord)** (7.4.8 in the brief's edition; fused with
O3 as `IteratorStepValue` in ES2025).
1. `result` ← `? IteratorNext(iteratorRecord)` = `? Call(record.[[NextMethod]],
   record.[[Iterator]])`; if the result is not an Object, throw TypeError.
2. `done` ← `? IteratorComplete(result)` = `ToBoolean(? Get(result, "done"))`.
3. If `done` is **true**, set `record.[[Done]]` to true and return **false**.
4. Return `result`.

**O3 — IteratorValue(result)** (7.4.9) = `? Get(result, "value")`. Ordering
matters: `"done"` is read before `"value"`, and `"value"` is not read at all on
the exhausting step.

**O4 — IteratorClose(iteratorRecord, completion)** (7.4.11).
1. Assert `record.[[Iterator]]` is an Object.
2. `innerResult` ← `Completion(GetMethod(iterator, "return"))`.
3. If `innerResult` is a normal completion:
   a. `return_` ← `innerResult.[[Value]]`;
   b. if `return_` is undefined, return `? completion` (nothing to call);
   c. `innerResult` ← `Completion(Call(return_, iterator))`.
4. If `completion` is a **throw** completion, return `? completion` — the
   original throw wins and any error raised by steps 2–3 is **swallowed**.
5. If `innerResult` is a throw completion, return `? innerResult`.
6. If `innerResult.[[Value]]` is not an Object, throw a **TypeError**.
7. Return `? completion`.

Steps 4 and 5 are the asymmetry that `iterator-close-non-throw-get-method-abrupt.js`
pins: with a `break` completion, a throwing `return` getter escapes; with a
`throw` completion, it does not.

`AsyncIteratorClose` is the same shape with the `return()` result awaited before
the "is it an Object" check.

### 1.3 When the close obligation fires (14.7.5.6 + LoopContinues 14.7.1.1)

`ForIn/OfBodyEvaluation` step 6 (`6.g.ii` in the brief's edition, `6.k.ii` in
ES2024) reads, for `iterationKind` = *iterate*:

> If `LoopContinues(result, labelSet)` is false, then …
> Return `? IteratorClose(iteratorRecord, UpdateEmpty(result, V))`.

`LoopContinues(completion, labelSet)` (14.7.1.1):

1. normal completion → **true** (keep looping)
2. not a continue completion → **false**
3. continue with `[[Target]]` empty → **true**
4. `labelSet` contains `[[Target]]` → **true**
5. otherwise → **false**

So the exact close predicate is:

```
close  ⟺  kind ∈ {throw, return, break}
        ∨  (kind = continue ∧ target ∉ labelSet(this loop))
```

Two consequences that a witness design must not get wrong:

- A plain `continue`, and a `continue L` where `L` labels *this* loop, do **not**
  close. Normal exhaustion (`nextResult.[[Done]]` true) does **not** close either
  — the loop returns before reaching the `LoopContinues` branch.
- A `continue L` where `L` labels an **enclosing** statement **does** close.

**Correction to the area brief.** The brief calls
`iterator-close-via-continue.js` "the negative control … IteratorClose must NOT
run". That is wrong for that file. Read at
`test262/vendor/test262/test/language/statements/for-of/iterator-close-via-continue.js`,
it wraps the loop in `L: do { … continue L; … } while (false)` and asserts
`returnCount === 1` — the close **must** run, because `L` is not in the for-of's
own label set. Its `returnCount === 0` assertion is *inside* the body, before the
continue. The file is a positive control for clause 5 of `LoopContinues`. The
true negative control (unlabelled `continue`, close must not run) has no dedicated
file at that path and is supplied as a paper trace in §9.

### 1.4 The well-known iterators are ordinary replaceable properties

`Array.prototype[%Symbol.iterator%]` (23.1.3.x — `23.1.3.36` in the brief's
edition) has initial value `%Array.prototype.values%` and attributes
`{ [[Writable]]: true, [[Enumerable]]: false, [[Configurable]]: true }`. It is
therefore assignable, deletable and shadowable — by an own property on the array,
by a subclass prototype, or by assignment to `Array.prototype`.

`CreateArrayIterator` (23.1.5.1) additionally specifies that the iterator's step
closure performs, **per step**:

- `len` ← `? LengthOfArrayLike(array)` (so `length` is re-read every iteration), and
- `elementValue` ← `? Get(array, ! ToString(𝔽(index)))` (a real `[[Get]]`, so a
  hole consults `Array.prototype`, and an index accessor runs).

`String.prototype[%Symbol.iterator%]` is likewise replaceable, and
`%StringIteratorPrototype%.next` steps by `CodePointAt(s, position)` (11.1.5),
which yields an **unpaired surrogate as a one-unit code point** rather than
skipping or replacing it.

Lowering `for (x of arr)` to an index walk therefore is not "an optimization"; it
is sound only relative to a conjunction of premises about the realm and the
value. This contract does not discharge those premises. It makes them **named
values in the IR** so that they cannot be relied on silently, and so that adding
a fourth specialization forces someone to write them down.

### 1.5 Where the spec leaves latitude, and the choice made here

| Latitude | Choice | Why |
|---|---|---|
| The spec does not say how an implementation records that it specialized a loop. | Record it as a **value on the `StatementIr` variant**, not a comment or a lowering-local flag. | A comment is not checked; a lowering-local flag dies at the function boundary. The `StatementIr` variant is the only thing that reaches the emitter, the planners, and the tests. |
| Nothing requires the catalog's `abrupt` set to be derived rather than stated. | **Derive it** from `SpecOperationIr`. | It is a total function of the operation. Anything a total function can produce should not be a parameter. |
| Nothing requires deleting a spec-shaped type that models an operation we have not implemented. | **Delete it**, and turn its row into `TrackedGap`. | AGENTS.md: "Survival by `pub` is not survival." A type with no call site is a claim, and this area exists because claims were being read as implementations. Reintroducing a 20-line record when a caller exists is cheap; a false catalog row is not. |
| ES2025 fuses `IteratorStep`+`IteratorValue` into `IteratorStepValue`. | Keep the **four-obligation** decomposition (GetIterator / IteratorStep / IteratorValue / IteratorClose). | It is what the existing catalog names, what the emitted code separates, and what the corpus tests key on. Record `IteratorStepValue` as a naming note only. |
| We could model `[[Done]]` in the IR and have the emitter consult it. | We do **not**. The emitter reads nothing new; `[[Done]]`'s only IR-level presence is as the name of the suspension slot on the `for await` path. | Rung G must diff empty. An IR field the emitter reads changes bytes; an IR field the emitter ignores changes nothing but still forces the author to fill it in. |

---

## 2. Measured baseline

Every number below was counted, not estimated.

| Fact | Value | How counted |
|---|---|---|
| Catalog rows | **46** | `grep -c 'lowered_op(' operations.rs` = 47, minus the `const fn lowered_op` definition at `operations.rs:1197` |
| `SpecOperationIr` variants | **29** | enumerated at `operations.rs:786-816` |
| Rows with `SharedWasmEmitter` | **29** | 46 − 17; names are in exact bijection with the 29 variants |
| Rows with `SharedRustModel` | **17** | Type, IntegerIndexedConversion, IsLessThan, CreateDataProperty, DefinePropertyOrThrow, ToPropertyDescriptor, FromPropertyDescriptor, OrdinaryCreateFromConstructor, SpeciesConstructor, ArraySpeciesCreate, GetIterator, IteratorStep, IteratorValue, IteratorClose, AsyncIteratorClose, Completion, UpdateEmpty |
| Rows with `CatalogOnly` | **0** — and the test at `operations.rs:1323` *panics* if one ever appears | `operations.rs:1323-1328` |
| Catalog/enum join today | one `&'static str` compare at runtime | `find_spec_operation`, `operations.rs:1217-1221` |
| Product call sites of `SPEC_OPERATION_CATALOG`, `spec_operation_catalog`, `find_spec_operation` | **0** outside `lila-ir/src/operations.rs`; the only mention is the `pub use` at `lib.rs:85,93` | workspace-wide grep |
| Spec-record types with **zero** call sites outside `operations.rs` (their sole mention being the `pub use` at `lib.rs:86-93`) | **9** | `PropertyDescriptorIr`, `PropertyDescriptorKind`, `IteratorRecordIr`, `CreateDataPropertyIr`, `DefinePropertyIr`, `OrdinaryCreateFromConstructorIr`, `SpeciesConstructorIr`, `ArraySpeciesCreateIr` (the brief's seven) **plus** `IntegerIndexedConversionIr`/`IntegerIndexedElementType` and `AbstractRelationalComparisonResult` |
| `ValueKind::known_ecmascript_type` (the "Type" model) product call sites | **0** — its 8 call sites are all inside `#[cfg(test)] mod tests`, which starts at `ir.rs:3280` | grep + module boundary |
| `StatementIr::abrupt_completion_record` product call sites | **0** — called only by `is_abrupt_completion_statement` (`ir.rs:2026`), which is itself called only from tests | grep + module boundary |
| `CompletionRecordIr::update_empty` call sites | **0** anywhere but its own unit test | grep |
| `COMPLETION_ABI_SLOTS` → backend join | **test-only**: the `use` is inside `#[cfg(test)] mod tests` at `abi.rs:45-46`. The real, compile-time join is `CompletionKindIr::abi_code()` used in the `const` initialisers at `abi.rs:3-8`. Out of scope. | `abi.rs:1-88` |
| `StatementIr` variants | **33** | enumerated between `ir.rs:1839` and `ir.rs:2012` |
| `StatementIr::abrupt_completion_record` catch-all | `_ => None` at `ir.rs:2021`, covering **29** of the 33 variants | `ir.rs:2015-2023` |
| for-of specializations | **3**: `ForOfArray` (`ir.rs:1924`), `ForOfString` (`ir.rs:1932`), `ForOfIterator` (`ir.rs:1939`) | |
| for-of specialization **construction** sites | **3**, all in one `if/else if/else` — `lowering.rs:13418`, `:13431`, `:13504` | |
| Total `ForOf*` mention sites workspace-wide | **83** (`control_flow.rs` 10, `data.rs` 3, `emit.rs` 11, `planning.rs` 24, `early_errors.rs` 3, `ir.rs` 6, `lib.rs` 19, `lowering.rs` 7) | grep, excluding `AsyncForOf*PlanIr` lines |
| Of those, patterns that list **every** field with no `..` (so adding a field breaks them) | **6**, all in `control_flow.rs`: `:3022`, `:3053`, `:3068`, `:3368`, `:3399`, `:3414` | inspection |
| `AsyncForOfPlanIr` construction sites | **0**. The type is defined (`ir.rs:1754`), used as `ForOfArray.async_plan: Option<AsyncForOfPlanIr>` (`ir.rs:1930`), imported (`control_flow.rs:5`) and consumed by `compile_async_for_of_array` (`control_flow.rs:5283-5731`, ~449 lines) — which is therefore **unreachable from the product path**. The only `ForOfArray` construction sets `async_plan: None` (`lowering.rs:13424`). | grep |
| `AsyncForOfIteratorPlanIr` binding-name field reads in the backend | **3** (`control_flow.rs:6268`, `:6273`, `:6283`), plus **1** for the dead array plan (`:5371`), plus **6** in `lila-ir/src/lib.rs` tests | grep |
| Tests in `operations.rs` | **21** | `grep -c '#\[test\]'` |
| `for-of` files in the pinned corpus | **183** | `ls test262/vendor/test262/test/language/statements/for-of/` |

Two derived facts that matter more than any single number:

1. **All 17 `SharedRustModel` rows are false, not just the five iterator ones.**
   The brief flags GetIterator/IteratorStep/IteratorValue/IteratorClose/
   AsyncIteratorClose. Measurement shows `Type`, `Completion` and `UpdateEmpty`
   are equally unbacked (their models exist but have zero product call sites),
   and the remaining nine model types have zero call sites at all. The row set
   that survives contact with the product path is **empty**.
2. **The catalog has no consumer.** Nothing outside `operations.rs` reads it. Its
   sole function today is to be read by humans, which is precisely why a false
   row is the worst kind of defect it can carry, and why the fix must be
   compile-time rather than a better test.

---

## 3. Type mapping, Part A — the catalog becomes evidence

The task domain lives in `crates/lila-ir/src/task.rs`; the catalog types and
rows live in `crates/lila-ir/src/operations.rs`.

### A1. `OperationLoweringStatus` — closed, evidence-carrying, no free variants

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationLoweringStatus {
    /// Emitted by the shared `SpecOperationIr` emitter arm in
    /// `lila-aot-wasm/src/operations.rs`. The evidence names the variant.
    SharedWasmEmitter(EmitterEvidence),
    /// Emitted, but by a statement-shaped emitter arm rather than a
    /// `SpecOperationIr` arm. The evidence names which arm.
    StatementEmission(EmissionSite),
    /// Not implemented. Both fields are closed/validated; neither is free text.
    TrackedGap { reason: TrackedGapReason, owner: TaskId },
}
```

Deleted variants and why:

- **`CatalogOnly` is deleted.** Today it exists only so that a test can `panic!`
  on it (`operations.rs:1323-1328`). A variant whose sole semantics is "must never
  occur" is a runtime check standing in for a type that simply lacks the variant.
- **`SharedRustModel` is deleted.** After §4 and §8 there are **zero** rows for
  which it would be true, and an enum variant with no constructor site does not
  fail to build. Reintroduce it — together with `RustModelEvidence` — in the same
  patch that gives some model type its first product call site, not before.
  *(Brief's alternative, recorded and rejected: keep `SharedRustModel(RustModelEvidence)`
  where the evidence is a witness value naming the Rust type. Rejected because
  a witness "naming a type" is a `&'static str` in a trench coat unless it is
  keyed to a real call site, and no call site exists to key it to.)*

```rust
/// Proof that a `SpecOperationIr` variant stands behind a catalog row.
/// The field is private and there is no public constructor: the only way to
/// obtain one is `SpecOperationIr::emitter_evidence`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitterEvidence {
    operation: SpecOperationIr,
}

impl EmitterEvidence {
    pub const fn operation(self) -> SpecOperationIr { self.operation }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrackedGapReason {
    /// No `SpecOperationIr` variant and no emitter arm implements it.
    NoImplementation,
    /// A Rust model type exists in `lila-ir` but nothing on the product path
    /// constructs it.
    ModelWithoutCallSite,
}

// `task.rs` generates the enum, `ALL`, `as_str`, `Display` and `FromStr` from
// this one registry. The generated `as_str` match is exhaustive.
define_task_ids! {
    T00 = 0 => "T00",
    // ...one row for every task in ledger order...
    T29 = 29 => "T29",
}
```

The generated `TaskId` enum is the only task-owner domain. A row uses
`TaskId::T04`; a nonexistent `TaskId::T30` is `E0599`, while an external
`"T30"` fails `FromStr` before a `TaskId` exists. Its stable text remains
`T00`–`T29`. `T26-unclassified` remains a Test262 classification fallback and
is deliberately rejected by the task-id parser.

### A2. `NormalResult` — the codomain stops being a string

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NormalResult {
    Unused,
    Boolean,
    BooleanOrUndefined,
    String,
    Number,
    NumberOrBigInt,
    BigInt,
    Integer,
    Object,
    ObjectOrUndefined,
    ObjectOrFalse,
    Array,
    Constructor,
    CallableOrUndefined,
    PropertyKey,
    PropertyDescriptor,
    LanguageValue,
    LanguageType,
    IteratorRecord,
    CompletionRecord,
}
```

Exactly the 20 distinct strings the current 46 rows use, plus nothing. This
deletes the runtime `assert!(!entry.normal_result.is_empty())` at
`operations.rs:1312`: emptiness is now unrepresentable. `NormalResult::name()` is
a `const fn` with an exhaustive match, used only for rendering.

### A3. The signature becomes a total function of the variant

Three `const fn`s on `SpecOperationIr`, each an **exhaustive match with no
catch-all**:

```rust
impl SpecOperationIr {
    pub const fn family(self) -> SpecOperationFamily { /* 29 arms */ }
    pub const fn normal_result(self) -> NormalResult   { /* 29 arms */ }
    pub const fn abrupt(self) -> &'static [CompletionAbruptKind] { /* 29 arms */ }

    pub const fn emitter_evidence(self) -> EmitterEvidence {
        EmitterEvidence { operation: self }
    }

    /// The row. Not a table entry that happens to match — the row *is* the
    /// variant, so a variant without a row is not expressible.
    pub const fn catalog_entry(self) -> SpecOperationCatalogEntry {
        SpecOperationCatalogEntry {
            name: self.name(),
            family: self.family(),
            normal_result: self.normal_result(),
            abrupt: self.abrupt(),
            lowering_status:
                OperationLoweringStatus::SharedWasmEmitter(self.emitter_evidence()),
        }
    }
}
```

`ToPrimitive(hint)` matches as `Self::ToPrimitive(_)` in all four, exactly as
`name()` already does at `operations.rs:824`.

Values for the 29 emitter rows are carried over verbatim from the current table
(`operations.rs:880-1124`), i.e. `abrupt()` returns `MAY_THROW` for exactly the
operations whose current row says `MAY_THROW`, and `NO_ABRUPT` for the seven that
say `NO_ABRUPT` (IsCallable, IsConstructor, IsPropertyKey, ToBoolean, SameValue,
SameValueZero, StrictEqualityComparison). **This is a pure re-encoding: no row's
signature changes in this area.** Any change to an `abrupt` set is a separate
lane with its own dry run.

### A4. The gap table cannot express an implementation claim

```rust
/// A row for an operation we have *not* implemented. By construction it cannot
/// carry `SharedWasmEmitter` or `StatementEmission`: there is no field for one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackedGapRow {
    pub name: &'static str,
    pub family: SpecOperationFamily,
    pub normal_result: NormalResult,
    pub abrupt: &'static [CompletionAbruptKind],
    pub reason: TrackedGapReason,
    pub owner: TaskId,
}

pub const TRACKED_GAP_ROWS: &[TrackedGapRow] = &[ /* the 12 rows of §3.6 */ ];
```

`TrackedGapRow::into_entry()` is a `const fn` producing a
`SpecOperationCatalogEntry` whose `lowering_status` is always
`TrackedGap { reason, owner }`. There is no other path from a `TrackedGapRow` to
a status.

### A5. `SpecOperationIr::ALL`, the assembled catalog, and the const joins

```rust
impl SpecOperationIr {
    pub const ALL: &'static [SpecOperationIr] = &[ /* 29 entries, one per variant */ ];
}

pub const SPEC_OPERATION_ROW_COUNT: usize =
    SpecOperationIr::ALL.len() + TRACKED_GAP_ROWS.len();

pub const SPEC_OPERATION_CATALOG: [SpecOperationCatalogEntry; SPEC_OPERATION_ROW_COUNT] =
    build_catalog();   // const fn: emitter rows first, then gap rows, both in order
```

Const assertions (all fire at `cargo check`, all replace tests):

```rust
/// Byte-wise `str` equality, usable in const. Stable since 1.55; the toolchain
/// is 1.94.1 stable, edition 2021.
const fn str_eq(a: &str, b: &str) -> bool { /* len check + byte loop */ }

// (J1) every catalog name is distinct — replaces the runtime test
//      `operations_catalog_names_are_unique` (operations.rs:1296).
const _: () = {
    let mut i = 0;
    while i < SPEC_OPERATION_ROW_COUNT {
        let mut j = i + 1;
        while j < SPEC_OPERATION_ROW_COUNT {
            assert!(!str_eq(SPEC_OPERATION_CATALOG[i].name,
                            SPEC_OPERATION_CATALOG[j].name),
                    "duplicate spec operation name");
            j += 1;
        }
        i += 1;
    }
};

// (J2) no gap row shadows an implemented operation: a name in TRACKED_GAP_ROWS
//      must not equal any `SpecOperationIr` name. Subsumed by J1 given the
//      assembly order, and stated separately because it is the invariant the
//      reader cares about; keep whichever the encoder finds clearer, not both.

// (J3) `ALL` is dense and duplicate-free.
const _: () = {
    let mut seen = [false; SPEC_OPERATION_ROW_COUNT];   // over-sized on purpose
    let mut i = 0;
    while i < SpecOperationIr::ALL.len() {
        let idx = SpecOperationIr::ALL[i].catalog_index();
        assert!(!seen[idx], "duplicate catalog_index");
        seen[idx] = true;
        i += 1;
    }
    let mut j = 0;
    while j < SpecOperationIr::ALL.len() { assert!(seen[j]); j += 1; }
};
```

`catalog_index(self) -> usize` is a fourth exhaustive `const fn` match. Its
purpose is J3, not lookup.

**What this buys, exactly, in each drift direction:**

- *Add a `SpecOperationIr` variant, forget the row.* **Impossible.** The row is
  `catalog_entry(self)`, whose four component matches are exhaustive: the new
  variant produces four `E0004 non-exhaustive patterns` errors before anything
  else is considered.
- *Add a catalog row for an operation with no variant, claiming it is
  implemented.* **Impossible.** Emitter rows come only from
  `SpecOperationIr::ALL`; hand-written rows can only be `TrackedGapRow`, whose
  type has no field capable of holding an implementation status.
- *Add a row that duplicates an existing name.* **Compile error** (J1).
- *Give a gap row a nonexistent owner.* **Compile error** (`TaskId::T30` has no
  such variant); untrusted text fails `TaskId::from_str` before construction.
- *Add a variant, add all four arms, and forget to list it in `ALL`.* **Not
  caught at compile time** — see ledger entry **L1**. Stable Rust has no
  `variant_count`, and no arrangement of const asserts can observe a variant that
  is absent from every list. The residual harm is bounded: the missing variant is
  absent from the *enumeration*, but it cannot produce a *false* row, because
  rows are derived. L1 covers it with the one test that survives.

### A6. The resulting 46 rows

| # | Row | Status after this area |
|---|---|---|
| 1–29 | the 29 `SpecOperationIr` names | `SharedWasmEmitter(evidence)` — derived |
| 30 | `GetIterator` | `StatementEmission(SyncForOfIterator)` |
| 31 | `IteratorStep` | `StatementEmission(SyncForOfIterator)` |
| 32 | `IteratorValue` | `StatementEmission(SyncForOfIterator)` |
| 33 | `IteratorClose` | `StatementEmission(SyncForOfIterator)` |
| 34 | `AsyncIteratorClose` | `StatementEmission(AsyncForOfIterator)` |
| 35 | `Type` | `TrackedGap { ModelWithoutCallSite, T04 }` |
| 36 | `Completion` | `TrackedGap { ModelWithoutCallSite, T04 }` |
| 37 | `UpdateEmpty` | `TrackedGap { ModelWithoutCallSite, T04 }` |
| 38 | `IntegerIndexedConversion` | `TrackedGap { NoImplementation, T04 }` |
| 39 | `IsLessThan` | `TrackedGap { NoImplementation, T04 }` |
| 40 | `CreateDataProperty` | `TrackedGap { NoImplementation, T04 }` |
| 41 | `DefinePropertyOrThrow` | `TrackedGap { NoImplementation, T04 }` |
| 42 | `ToPropertyDescriptor` | `TrackedGap { NoImplementation, T04 }` |
| 43 | `FromPropertyDescriptor` | `TrackedGap { NoImplementation, T04 }` |
| 44 | `OrdinaryCreateFromConstructor` | `TrackedGap { NoImplementation, T04 }` |
| 45 | `SpeciesConstructor` | `TrackedGap { NoImplementation, T04 }` |
| 46 | `ArraySpeciesCreate` | `TrackedGap { NoImplementation, T04 }` |

29 + 5 + 12 = 46. The row count is unchanged; **17 false implementation claims
become 5 honest emission claims and 12 honest gaps.**

Evidence for rows 30–34 (read, not assumed):
`compile_for_of_iterator` (`control_flow.rs:7422`) emits the `@@iterator` `Get`,
the `Call`, the `Get` of `"next"` (`control_flow.rs:8401`, `:8455`), the per-step
`next()` call and `"done"`/`"value"` reads, and routes exits through
`emit_iterator_close_condition_i32` (`:9018`) into `emit_iterator_close` (`:9046`)
or `emit_iterator_close_preserving_current_throw` (`:9193`).
`compile_async_for_of_iterator` (`control_flow.rs:6085`) open-codes the async
close at `control_flow.rs:6813-6900+`.

---

## 4. Type mapping, Part B — the iterator-protocol obligation witness

New file: `crates/lila-ir/src/iterator_obligations.rs`, re-exported from
`lib.rs`.

### B1. The obligation and its discharge

```rust
/// The four 7.4 obligations a for-of head incurs. Closed by the spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IteratorObligation { GetIterator, IteratorStep, IteratorValue, IteratorClose }

/// Which emitter arm performs the operation. Closed; each variant is joined to a
/// real function by R7 below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmissionSite {
    /// `FunctionBuilder::compile_for_of_iterator` (control_flow.rs:7422)
    SyncForOfIterator,
    /// `FunctionBuilder::compile_async_for_of_iterator` (control_flow.rs:6085)
    AsyncForOfIterator,
    /// `FunctionBuilder::compile_array_destructure_from_value_locals` (control_flow.rs:8220)
    ArrayDestructuring,
}

/// The premises a specialization may rely on. Closed: a lowering that needs a
/// premise not in this list must add a variant, which is a diff a reviewer sees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntactnessPremise {
    /// `%Array.prototype%[@@iterator]` is still `%Array.prototype.values%`, no own
    /// `@@iterator` shadows it on the value or on an intermediate prototype, and
    /// `%ArrayIteratorPrototype%.next` is unpatched. (23.1.3.x, 23.1.5.1)
    ArrayIteratorIntact,
    /// `length` is read once before the walk; `CreateArrayIterator` re-reads
    /// `LengthOfArrayLike` on every step. (23.1.5.1)
    ArrayLengthReadOnce,
    /// Elements are read from the backing storage rather than by `[[Get]]`, so a
    /// hole does not consult `Array.prototype` and an index accessor does not run.
    ArrayElementReadBypassesGet,
    /// `%String.prototype%[@@iterator]` is still `%String.prototype[@@iterator]%`
    /// and `%StringIteratorPrototype%.next` is unpatched. (22.1.3.x)
    StringIteratorIntact,
    /// The walk steps by code point over the internal encoding; `CodePointAt`
    /// (11.1.5) yields an unpaired surrogate as a one-unit code point.
    StringWalkIsCodePoint,
    /// There is no iterator object, so there is nothing `IteratorClose` could
    /// call: the close obligation is vacuous rather than skipped.
    NoIteratorObjectExists,
}

/// How one obligation was accounted for. Two cases, both carrying payload:
/// there is no "unknown", no `Default`, and no unit variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObligationDischarge {
    ByEmission(EmissionSite),
    ByAssumption(IntactnessPremise),
}
```

### B2. The witness — four distinct newtypes so a swap is a type error

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GetIteratorDischarge(ObligationDischarge);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IteratorStepDischarge(ObligationDischarge);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IteratorValueDischarge(ObligationDischarge);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IteratorCloseDischarge(ObligationDischarge);
// each: `pub const fn new(ObligationDischarge) -> Self` and `pub const fn get(self) -> ObligationDischarge`

/// Non-defaultable, non-optional. Every for-of specialization carries one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IteratorProtocolWitness {
    get_iterator: GetIteratorDischarge,
    iterator_step: IteratorStepDischarge,
    iterator_value: IteratorValueDischarge,
    iterator_close: IteratorCloseDischarge,
}

impl IteratorProtocolWitness {
    pub const fn new(
        get_iterator: GetIteratorDischarge,
        iterator_step: IteratorStepDischarge,
        iterator_value: IteratorValueDischarge,
        iterator_close: IteratorCloseDischarge,
    ) -> Self { /* field-by-field */ }

    pub const fn discharge(&self, obligation: IteratorObligation) -> ObligationDischarge {
        match obligation { /* 4 arms, exhaustive */ }
    }

    pub const fn is_fully_emitted(&self) -> bool { /* all four ByEmission */ }
}
```

Properties, and the mistakes each kills:

- **No `Default`, no `Option`, all fields private, one constructor.** A new
  specialization variant cannot compile without constructing one.
- **Four distinct argument types.** Transposing "what I assumed about `next`"
  with "what I assumed about `return`" is `E0308 mismatched types`, not a
  plausible-looking wrong witness.
- **Adding a fifth obligation** (say `IteratorNext` were split out) changes
  `IteratorProtocolWitness::new`'s arity and adds an arm to `discharge`: every
  construction site and the `IteratorObligation` match fail to build.
- `IteratorProtocolWitness` is `Copy` and free of `String`, so putting it on a
  `StatementIr` variant costs no allocation and preserves `PartialEq`/`Clone`.

### B3. The three canonical witnesses

```rust
impl IteratorProtocolWitness {
    /// `StatementIr::ForOfArray` — the index walk. All four obligations are
    /// discharged by assumption; the premises are the contract of §1.4.
    pub const ARRAY_INDEX_WALK: Self = Self::new(
        GetIteratorDischarge::new(ObligationDischarge::ByAssumption(
            IntactnessPremise::ArrayIteratorIntact)),
        IteratorStepDischarge::new(ObligationDischarge::ByAssumption(
            IntactnessPremise::ArrayLengthReadOnce)),
        IteratorValueDischarge::new(ObligationDischarge::ByAssumption(
            IntactnessPremise::ArrayElementReadBypassesGet)),
        IteratorCloseDischarge::new(ObligationDischarge::ByAssumption(
            IntactnessPremise::NoIteratorObjectExists)),
    );

    /// `StatementIr::ForOfString` — the code-point walk.
    pub const STRING_CODE_POINT_WALK: Self = Self::new(
        GetIteratorDischarge::new(ObligationDischarge::ByAssumption(
            IntactnessPremise::StringIteratorIntact)),
        IteratorStepDischarge::new(ObligationDischarge::ByAssumption(
            IntactnessPremise::StringWalkIsCodePoint)),
        IteratorValueDischarge::new(ObligationDischarge::ByAssumption(
            IntactnessPremise::StringWalkIsCodePoint)),
        IteratorCloseDischarge::new(ObligationDischarge::ByAssumption(
            IntactnessPremise::NoIteratorObjectExists)),
    );

    /// `StatementIr::ForOfIterator`, sync.
    pub const SYNC_ITERATOR_PROTOCOL: Self = /* all four ByEmission(SyncForOfIterator) */;
    /// `StatementIr::ForOfIterator`, `for await`.
    pub const ASYNC_ITERATOR_PROTOCOL: Self = /* all four ByEmission(AsyncForOfIterator) */;
}
```

These four constants are the **only** values `lowering.rs` may use. They are
`const`, so a reviewer diffing a premise change sees it in one place rather than
at a construction site 13,000 lines into `lowering.rs`.

### B4. `StatementIr` changes

```rust
ForOfArray {
    mode, name, iterable, body, lexical_environment,
    protocol: IteratorProtocolWitness,     // NEW, non-optional
    // async_plan: REMOVED — see R6
},
ForOfString {
    mode, name, iterable, body, lexical_environment,
    protocol: IteratorProtocolWitness,     // NEW, non-optional
},
ForOfIterator {
    mode, name, iterable, body, lexical_environment,
    protocol: IteratorProtocolWitness,     // NEW, non-optional
    async_plan: Option<AsyncForOfIteratorPlanIr>,   // unchanged shape, changed contents (R5)
},
```

The emitter must **not** read `protocol`. All six full-field patterns in
`control_flow.rs` (§2) gain `..`. This is what makes rung G diff empty.

### B5. `IteratorRecordIr` — earn a call site or die; it earns one

`IteratorRecordIr<T>` has zero call sites. The brief asks that `ForOfIterator`
become its owner "as the `[[Done]]` carrier `IteratorClose` keys off". Taken
literally that would be decoration: the sync path's `[[Iterator]]`/`[[NextMethod]]`
live in unnamed emitter temporaries, and the emitter is forbidden from reading a
new IR field, so a record on the sync path would carry nothing checkable.

There **is** a real call site, on the async path. `AsyncForOfIteratorPlanIr`
(`ir.rs:1765-1775`) already is an Iterator Record, spelled as loose `String`s:

```rust
pub iterator_binding: String,   // [[Iterator]]
pub next_binding: String,       // [[NextMethod]]
pub done_binding: String,       // [[Done]]
```

Three same-typed fields, freely transposable at the construction site
(`lowering.rs:13496-13499`) and at all three backend reads
(`control_flow.rs:6268`, `:6273`, `:6283`). Transposing `iterator_binding` and
`next_binding` type-checks today and miscompiles every `for await`.

So:

```rust
// operations.rs — the type parameter goes; three slot newtypes arrive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IteratorSlot(String);        // [[Iterator]]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextMethodSlot(String);      // [[NextMethod]]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoneSlot(String);            // [[Done]]
// each: `pub fn new(String) -> Self`, `pub fn as_str(&self) -> &str`

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IteratorRecordIr {
    iterator: IteratorSlot,
    next_method: NextMethodSlot,
    done: DoneSlot,
    kind: IteratorRecordKind,
}

impl IteratorRecordIr {
    pub fn sync(iterator: IteratorSlot, next_method: NextMethodSlot, done: DoneSlot) -> Self;
    pub fn async_(iterator: IteratorSlot, next_method: NextMethodSlot, done: DoneSlot) -> Self;
    pub fn iterator(&self) -> &IteratorSlot;
    pub fn next_method(&self) -> &NextMethodSlot;
    pub fn done(&self) -> &DoneSlot;
    pub const fn kind(&self) -> IteratorRecordKind;
}
```

and `AsyncForOfIteratorPlanIr` becomes:

```rust
pub struct AsyncForOfIteratorPlanIr {
    pub entry_state: u32,
    pub value_resume_state: u32,
    pub close_resume_state: u32,
    pub exit_state: u32,
    pub record: IteratorRecordIr,               // replaces the three String fields
    pub async_iterator_binding: String,         // not an Iterator Record field
    pub close_on_rejection_binding: String,     // not an Iterator Record field
}
```

The `kind` field stops being decoration too: it is set to `Async` by the only
constructor `lowering.rs` may call for this plan, so `kind` and the plan's
existence cannot disagree.

**Removed with the type parameter:** `map_value`, `mark_done`, `is_done`, and the
`done: bool` field — none has a call site, and `[[Done]]` at IR level is the name
of a suspension slot, not a compile-time boolean.

### B6. `StatementIr::abrupt_completion_record` becomes exhaustive

`ir.rs:2015-2023`. Replace `_ => None` (`ir.rs:2021`, currently absorbing 29 of 33
variants) with 29 explicit `Self::X => None` arms grouped by or-patterns. The
obligation this creates: **adding a 34th `StatementIr` variant forces its author
to state whether it is an abrupt completion.** That is worth having whether or not
the function has a caller today (measured: it has none on the product path).

Keep `CompletionRecordIr` as the return type; its rows stay `TrackedGap
{ ModelWithoutCallSite, T04 }` until something on the product path constructs one.

---

## 5. The runtime-checked ledger

These are the only places where a test remains load-bearing. Each entry states
what cannot be a type and why.

| id | Invariant | Why no type can carry it | The check that replaces it |
|---|---|---|---|
| **L1** | Every `SpecOperationIr` variant appears in `SpecOperationIr::ALL`. | Stable Rust has no `variant_count`. A variant absent from every list is invisible to every const expression. (Harm is bounded: rows are *derived* from `catalog_entry`, so a variant missing from `ALL` yields an incomplete enumeration, never a false claim.) | One test in `operations.rs`: `SPEC_OPERATION_CATALOG.len() == SPEC_OPERATION_ROW_COUNT` **and** each `ALL` entry round-trips `catalog_index`. Its failure message must name `ALL`. This is the only surviving catalog test, and it is not vacuous because it checks the assembly, not the table's own contents. |
| **L2** | A row's `abrupt` set matches what the emitter arm actually emits. | `lila-ir` cannot see `lila-aot-wasm`; the dependency runs the other way, and the emitter arm's type is `(&mut Function) -> Result<(), EmitError>`, which has no channel for "this arm emits a throw path". | **Not checked in this area.** Recorded as an open defect of the same class as `ca09433c1`. Proposed follow-up lane: change the emitter arm's success type to a `Emitted::{MayThrow, NoAbrupt}` that the caller must match, and const-assert it against `SpecOperationIr::abrupt()`. Out of scope here because it lives in `lila-aot-wasm/src/operations.rs` and would touch the emitted-code path. |
| **L3** | An `IntactnessPremise` is *true* of the program being compiled. | The premise is a statement about the user's source, not about our code. No type can prove it; only a lowering-time guard can, and building that guard is a separate lane by scope. | **Nothing.** Deliberately. The witness makes the premise *nameable and greppable*; it does not make it true. §9's adversarial traces record the exact programs for which `ArrayIteratorIntact` is false today. |
| **L4** | `static_object_iterator_iife_source_values` (`lowering.rs:35919-35944`) picks a specialization on a **substring of whitespace-stripped source text** (`"[Symbol.iterator]:null"`, `"[Symbol.iterator]:undefined"`, `"=>{name}.next()"`). Reformatting the program changes which specialization fires. | It is a source-text oracle, not a semantic one; typing it would mean rewriting it. Out of scope by the area definition. | **Named unsound guard.** Recorded here so it is not rediscovered as a mystery. Must be cited by any future lane that touches for-of specialization. |
| **L5** | `KindSet::EMPTY.is_subset_of(KindSet::from_kind(ValueKind::Array))` is `true` (`ir.rs:339`). A value with an *empty* `possible_kinds` therefore selects `ForOfArray` in `lowering/for_of.rs::lower_for_of_head`. | The subset test is correct set theory; the *use* of it as "is definitely an Array" is the bug. Fixing it changes which specialization fires, i.e. changes emitted bytes — out of scope. | **Recorded, not fixed.** The `ARRAY_INDEX_WALK` witness's premises are stated as holding "for a value whose inferred kind set is exactly `{Array}`"; whether the guard actually establishes that is L5's business. Flagged for the lane that closes L3. |

---

## 6. Mistake-class table

| # | Plausible mistake | Today | After this contract |
|---|---|---|---|
| 1 | Mark an operation as implemented when nothing implements it (17 rows do). | Row compiles; only a human reading the table notices. | **Impossible.** Implementation statuses are only reachable from `SpecOperationIr::catalog_entry` (`SharedWasmEmitter`) or an `EmissionSite` (`StatementEmission`). A hand-written row is a `TrackedGapRow`, which has **no field** that can hold either — `E0560 struct has no field named …` / `E0609`. |
| 2a | Add a `SpecOperationIr` variant, forget its catalog row. | Silently absent; `find_spec_operation` returns `None` at runtime. | **`E0004` × 4** — `SpecOperationIr::{family, normal_result, abrupt, catalog_index}` are exhaustive matches with no catch-all. |
| 2b | Add a catalog row with no variant (e.g. `"ToNumericString"`). | Compiles; joins to nothing. | Only expressible as a `TrackedGapRow`, i.e. **an honest gap**. To claim implementation you must add the variant, which lands you in 2a's four matches. |
| 3 | Record the wrong `abrupt` set on an implemented row (`MAY_THROW` vs `NO_ABRUPT`). | Free per-row argument at `operations.rs:1201`. | **No parameter exists.** `emitter_row`'s only input is the variant; `abrupt` comes from `SpecOperationIr::abrupt()`. (Whether that function itself is right is **ledger L2**.) |
| 4 | Give a row an owner that is not a backlog task id. | Runtime `assert_eq!(task, "T04")` in a test (`operations.rs:1321`). | **`E0599`** for a nonexistent `TaskId` variant. Text crosses the boundary only through `FromStr`, which returns an error without constructing a task id. |
| 5 | Leave a `CatalogOnly` row in the table. | Test `panic!`s (`operations.rs:1325`). | **`E0599`/`E0433`** — the variant does not exist. |
| 6 | A correctly-typed spec record survives with no call site because it is `pub`. Nine measured instances. | No warning; `pub` suppresses dead-code detection. | Eight are **deleted** (§8); `IteratorRecordIr` acquires a real call site inside `AsyncForOfIteratorPlanIr`. Deleting the last user of any of them now produces `E0432 unresolved import` at `lib.rs`, which is a build failure. |
| 7 | Transpose `[[Iterator]]` and `[[NextMethod]]` in the `for await` plan. | Both are `String`; compiles; miscompiles every `for await`. | **`E0308`** — `IteratorSlot` vs `NextMethodSlot`. |
| 8 | Add a fourth for-of specialization (TypedArray walk, Map/Set walk) and silently assume the protocol away. | New sibling variant of `ForOfArray`; passes every test whose body has no `break`/`throw`. | **`E0063 missing field `protocol`** at the construction site. There is no `Default` and no `Option`. The author must pick `ByEmission(site)` or `ByAssumption(premise)` for all four obligations, and if the premise they need is not in `IntactnessPremise` they must add a variant — a diff a reviewer sees. |
| 9 | Over-claim in the witness: mark a specialization as closing on *every* abrupt exit, when `continue` targeting this loop must not close. | Nothing represents the close predicate at all. | Partly. The witness records *whether* close is emitted, not the predicate. The predicate lives in `emit_iterator_close_condition_i32` (`control_flow.rs:9018`) and was **verified correct** by dry run (§9.1): it excludes exactly `continue` whose aux equals this loop's continue frame. The contract's job here is to have read it and recorded it, so a future edit to that function has a stated spec obligation to violate. |
| 10 | Add a 34th `StatementIr` variant that is an abrupt completion and forget to say so. | `_ => None` at `ir.rs:2021` silently answers "not abrupt". | **`E0004`** — the catch-all is gone. |
| 11 | Reformat a program and change which for-of specialization fires. | Real, live (`lowering.rs:35919-35944`). | **Not fixed.** Ledger **L4**. Named so the next reader does not have to rediscover it. |
| 12 | Rename or delete an emitter function that an `EmissionSite` claims to name. | `EmissionSite` does not exist. | **`E0599`** — R7 puts a `let _ = FunctionBuilder::compile_for_of_iterator;`-style path reference behind an exhaustive `match site` in `lila-aot-wasm`. (Guarantee: the *name* resolves. Not the signature.) |
| 13 | Vacuous green: a test asserting the table's own contents, or a status label nothing backs. | Two live instances (`operations.rs:1285`, `:1307`), one of which asserts `rust_modeled.contains("IteratorClose")` while no Rust model has a call site. | **Deleted** (§8). Their non-vacuous content moves into const asserts J1/J3 and the derived-row design. One test survives, L1, and it checks assembly rather than contents. |

---

## 7. Retrofit map

Ordered. Each step must leave the tree `cargo check`-clean before the next
begins; this is the whole verification strategy for the area.

**R0 — `crates/lila-ir/src/iterator_obligations.rs` (new, self-contained).**
All of §4 B1–B3. Depends on nothing. `cargo check -p lila-ir` after adding
`mod iterator_obligations;` and the `pub use`.

**R1 — `task.rs` plus `operations.rs`, Part A types.** The single-source
`TaskId` registry; `NormalResult`, `TrackedGapReason`, `EmitterEvidence`, the new
`OperationLoweringStatus`, `TrackedGapRow`, `str_eq`. Delete `CatalogOnly` and
`SharedRustModel`. Not yet wired to the catalog.

**R2 — `operations.rs`, the four exhaustive `const fn`s and the assembly.**
`family`, `normal_result`, `abrupt`, `catalog_index`, `emitter_evidence`,
`catalog_entry`, `ALL`, `TRACKED_GAP_ROWS` (12 rows per §3.6), `build_catalog`,
`SPEC_OPERATION_CATALOG`, const asserts J1/J3. Delete the old
`SPEC_OPERATION_CATALOG` literal (`operations.rs:872-1195`) and `lowered_op`
(`:1197-1211`). `find_spec_operation` keeps its signature.
`SPEC_OPERATION_CATALOG` changes from `&[T]` to `[T; N]`; the `pub use` at
`lib.rs:93` is unchanged, but any `.iter()` call site must be re-checked —
measured: there are **none** outside `operations.rs`.

**R3 — `operations.rs`, deletions.** §8. Removes eight types and thirteen tests.
Update the `pub use` block at `lib.rs:85-94` in the same step, or R3 will not
compile — that is the point (mistake class 6).

**R4 — `ir.rs`: the witness field and the exhaustive match.** Add
`protocol: IteratorProtocolWitness` to the three for-of variants (§4 B4). Make
`abrupt_completion_record` exhaustive (§4 B6). Then fix, in order:
1. `lowering.rs:13418`, `:13431`, `:13504` — the three construction sites; use
   `IteratorProtocolWitness::ARRAY_INDEX_WALK`, `::STRING_CODE_POINT_WALK`, and
   `::SYNC_ITERATOR_PROTOCOL` / `::ASYNC_ITERATOR_PROTOCOL` selected on
   `async_plan.is_some()`.
2. `control_flow.rs:3022`, `:3053`, `:3068`, `:3368`, `:3399`, `:3414` — the six
   full-field patterns. **Add `..`.** Do not bind `protocol`.
3. Everything else already uses `..`; `cargo check` will say otherwise if this
   measurement is wrong.

**R5 — `IteratorRecordIr` retrofit.** §4 B5. Touches `operations.rs`,
`ir.rs:1765-1775`, `lowering.rs:13471-13501` (construction),
`control_flow.rs:6268`, `:6273`, `:6283` (reads — mechanical
`plan.iterator_binding` → `plan.record.iterator().as_str()`), and six assertions
in `lila-ir/src/lib.rs` tests (`:7508`, `:7512`, `:7516`, `:7556`, and the
`async_plan: Some(...)` destructurings at `:6734`, `:6788`, `:7493`, `:7541`,
`:7597` — these keep working, only the field reads change).

**R6 — delete the dead `ForOfArray` async path. REQUIRED, sequenced last, and
droppable.** Evidence: `AsyncForOfPlanIr` has **zero** construction sites
workspace-wide, and the sole `ForOfArray` construction sets `async_plan: None`
(`lowering.rs:13424`); therefore `compile_async_for_of_array`
(`control_flow.rs:5283-5731`, ~449 lines) is unreachable from the product path,
which AGENTS.md says should fail to build. Delete: the `async_plan` field on
`ForOfArray` (`ir.rs:1930`), `AsyncForOfPlanIr` (`ir.rs:1754-1762`),
`compile_async_for_of_array`, its call at `control_flow.rs:3376-3387`, and the
`ForOfArray { async_plan: Some(..), .. }` arms at `control_flow.rs:1036`,
`:1075`, `emit.rs:435`, `:613`. **Nothing else in this contract depends on R6**;
if the batch is running long, drop it and record it as an open item rather than
half-doing it.

**R7 — `crates/lila-aot-wasm/src/emission_sites.rs` (new).** Close the
`EmissionSite` → real-function join:

```rust
use lila_ir::EmissionSite;
use crate::control_flow::FunctionBuilder;   // or wherever it is re-exported

/// Not called. Exists so that renaming or deleting an emitter arm that an
/// `EmissionSite` names is a compile error, and so that adding an
/// `EmissionSite` variant is a compile error until it names something real.
#[allow(dead_code, path_statements)]
fn emission_sites_are_backed(site: EmissionSite) {
    match site {
        EmissionSite::SyncForOfIterator  => { let _ = FunctionBuilder::compile_for_of_iterator; }
        EmissionSite::AsyncForOfIterator => { let _ = FunctionBuilder::compile_async_for_of_iterator; }
        EmissionSite::ArrayDestructuring => { let _ = FunctionBuilder::compile_array_destructure_from_value_locals; }
    }
}
```

`FunctionBuilder` is `pub(crate)` (`emit.rs:137`), so this file must live inside
`lila-aot-wasm`. Guarantee is name resolution, not signature — stated so
nobody over-reads it.

**R8 — the two redirect docs.** `docs/rust-rewrite/contracts/spec-operations.md`
and `.../iterator-protocol.md`, each four lines pointing here. The contract is
one document because Part A's `StatementEmission` rows are witnessed by Part B's
`EmissionSite`; splitting them would let the two drift.

### Untouched, deliberately

- `crates/lila-aot-wasm/src/abi.rs` — the completion ABI. Its real join is
  already a compile-time `const` (`CompletionKindIr::abi_code()`); out of scope.
- Every emitted byte. No emitter reads `protocol`; `EmissionSite` is only
  referenced by a function that is never called; `IteratorRecordIr` changes field
  *types*, not the strings those fields hold. **Rung G must diff empty. If it does
  not, the retrofit is wrong, not the gate.**
- `lowering.rs:13413-13437`, the specialization *decision*. The witness records
  what the decision assumed; it does not change the decision. The
  `Array.prototype[@@iterator]` hole is still open after this area lands.
- `lowering.rs:35919-35944`, the source-text oracle. Ledger L4.
- `crates/lila-aot-wasm/src/{intl_datetimeformat,temporal*,emitted_function,runtime_helpers}.rs`
  — batch 2's files. Not referenced by any step above.

---

## 8. Deletions

Types (all measured at zero call sites outside `operations.rs`; sole mention is
the `pub use` at `lib.rs:86-93`):

| Type | Lines | Row that claimed it | New row status |
|---|---|---|---|
| `PropertyDescriptorIr<T>` | `operations.rs:280-444` | ToPropertyDescriptor, FromPropertyDescriptor | `TrackedGap{NoImplementation}` |
| `PropertyDescriptorKind` | `:273-278` | — | — |
| `CreateDataPropertyIr<T>` | `:446-465` | CreateDataProperty | `TrackedGap{NoImplementation}` |
| `DefinePropertyIr<T>` | `:467-490` | DefinePropertyOrThrow | `TrackedGap{NoImplementation}` |
| `OrdinaryCreateFromConstructorIr<T>` | `:492-512` | OrdinaryCreateFromConstructor | `TrackedGap{NoImplementation}` |
| `SpeciesConstructorIr<T>` | `:514-534` | SpeciesConstructor | `TrackedGap{NoImplementation}` |
| `ArraySpeciesCreateIr<T>` | `:536-556` | ArraySpeciesCreate | `TrackedGap{NoImplementation}` |
| `IntegerIndexedConversionIr` + `IntegerIndexedElementType` | `:152-220` | IntegerIndexedConversion | `TrackedGap{NoImplementation}` |
| `AbstractRelationalComparisonResult` | `:127-150` | IsLessThan | `TrackedGap{NoImplementation}` |

The last two are **beyond the brief's seven**, found by the same measurement.
They are mandated on the same reasoning; if the encoder wants to stage them, they
may be deferred to a follow-up **provided their rows still become
`TrackedGap{NoImplementation}`** — the false row is the defect, the dead type is
the symptom.

Retained despite having no product call site, with reasons:

- `EcmaLanguageType` + `ValueKind::known_ecmascript_type` (`ir.rs:196-208`): a
  correct model, one call site away, and the natural home for the `Type`
  operation. Row → `TrackedGap{ModelWithoutCallSite, T04}`.
- `CompletionRecordIr<T>` including `update_empty`: its return-type role in
  `abrupt_completion_record` is what makes R4's exhaustive match a real
  obligation over 33 variants. Rows `Completion` / `UpdateEmpty` →
  `TrackedGap{ModelWithoutCallSite, T04}`.

Tests deleted (13 of 21 in `operations.rs`):

| Line | Test | Why |
|---|---|---|
| `:1285` | `operations_catalog_covers_t04_required_operations` | Vacuous: `REQUIRED_T04_OPERATIONS` (`:1236-1283`) is a transcription of the table's own `name` column. |
| `:1295` | `operations_catalog_names_are_unique` | Replaced by const assert J1. |
| `:1307` | `operations_catalog_tracks_every_gap_or_shared_lowering` | Vacuous *and* false: 46 lines asserting that labels the table declares are the labels the table declares, including `rust_modeled.contains("IteratorClose")` while no Rust model of `IteratorClose` has a call site. Its four real fragments are replaced by: `NormalResult` (non-emptiness), `TaskId` (closed owner membership), variant deletion (`CatalogOnly`), and derived rows (the 46 `contains` lines). |
| `:1617` | `operations_catalog_marks_abrupt_capable_operations` | Becomes an assertion about `abrupt()`'s own match once `abrupt` is derived. |
| `:1379`, `:1438` | the two `IteratorRecordIr` tests | Replaced by tests of the non-generic form (slot newtypes, `kind`). |
| `:1395`, `:1419` | `AbstractRelationalComparisonResult`, `IntegerIndexedConversionIr` tests | Types deleted. |
| `:1451`, `:1472`, `:1509` | `PropertyDescriptorIr` tests | Type deleted. |
| `:1530`, `:1546`, `:1566` | `CreateDataPropertyIr`, `DefinePropertyIr`, constructor/species tests | Types deleted. |

Tests retained: `:1598` (`EcmaLanguageType`), `:1630`, `:1644`, `:1664`
(completion ABI — these join to `abi.rs` and are not vacuous), `:1671`, `:1691`,
`:1711` (`CompletionRecordIr` behaviour). Plus **one new** test, ledger L1.

New tests added, and they must not be vacuous: for each of the four
`IteratorProtocolWitness` constants, assert the *shape* the constant claims —
`ARRAY_INDEX_WALK.is_fully_emitted() == false` and every discharge is
`ByAssumption`; `SYNC_ITERATOR_PROTOCOL.is_fully_emitted() == true`. These assert
a property of the constant that a careless edit could break (flipping one
obligation to `ByEmission` for the array walk would be a lie), which is the line
between this and the tests being deleted.

---

## 9. Dry-run corpus and expected traces

The dry-runner executes these on paper against the code, and records the result
in this file's companion dry-run report. Three entries in the area brief were
found to be mis-stated during formalization; the corrections are normative.

### 9.1 `iterator-close-via-break.js` / `-via-throw.js` / `-via-return.js`

All three reach `StatementIr::ForOfIterator` (the iterable is an object literal
with a computed `@@iterator`, so `possible_kinds` is not `⊆ {Array}` or
`⊆ {String}`). Baseline trace, read at `control_flow.rs:7690-7800`:

1. Body compiles inside a `Block` pushed on `finally_stack`.
2. `save_current_completion` captures kind + aux (the target frame).
3. A `continue` whose aux equals **this** loop's `continue_frame` is rewritten to
   `Normal` (`:7710-7719`) — it is not an exit.
4. `emit_iterator_close_condition_i32` (`:9018-9044`) computes
   `throw ∨ return ∨ break ∨ (continue ∧ aux ≠ this loop's continue frame)`.
   **This is exactly `¬LoopContinues`** from §1.3.
5. If the kind is `throw`, `emit_iterator_close_preserving_current_throw`
   (`:9193`) saves the completion, sets `Normal`, closes inside a guarded block,
   and restores — implementing IteratorClose step 4 (original throw wins,
   inner error swallowed). Otherwise `emit_iterator_close` (`:9046`) runs and a
   throw from it propagates — step 5.

**Verdict for the witness: `SYNC_ITERATOR_PROTOCOL`'s claim that all four
obligations are `ByEmission(SyncForOfIterator)` is honest.**

### 9.2 `iterator-close-via-continue.js` — **brief correction**

Not a negative control. `L: do { for (var x of iterable) { … continue L; } } while (false)`
— `L` labels the `do`, not the for-of, so clause 5 of `LoopContinues` returns
false and `IteratorClose` **must** run; the file asserts `returnCount === 1`. The
emitter's step 3 above compares aux against **this loop's** continue frame and
therefore closes. Correct.

The genuine negative control is the paper trace **N1**: `for (const x of it) { if (x) continue; }`
with an unlabelled `continue` — aux equals this loop's continue frame, step 3
rewrites the completion to `Normal`, the close condition is false, `return()` is
never called. A witness design that marked *every* abrupt body exit as closing
would be falsified here.

### 9.3 `iterator-close-non-throw-get-method-abrupt.js`

`break` completion + a throwing `get return()`. Expected: the `Test262Error`
escapes (steps 4/5 of §1.2 O4). Trace: `emit_iterator_close` reads `"return"` via
`emit_object_read` and then `emit_propagate_current_completion_if_throw`
(`control_flow.rs:9072-9073`), so the getter's throw propagates. This is the row
for `GetMethod` (`MAY_THROW`) doing real work, i.e. the mistake-class-3 probe.
Records: this case is served by the **non**-preserving path, and would be
*wrongly* swallowed if a future edit routed all closes through
`emit_iterator_close_preserving_current_throw`.

### 9.4 `Array.prototype.Symbol.iterator.js` — **brief correction**

The brief calls this "the exact test for the ForOfArray intactness premise". It
is not. Read in full, the file does **not** patch `Array.prototype[@@iterator]`;
it writes `for (var value of array[Symbol.iterator]())`. The iterable is the
result of a dynamic call, so `possible_kinds` is not `⊆ {Array}` and
**`ForOfIterator`** fires. The file is a useful control for the generic protocol
over a real Array Iterator (including a hole at index 5, which must yield
`undefined`), not an intactness probe.

The nearest existing corpus files that *do* patch `Array.prototype[@@iterator]`
are the three generated destructuring cases
(`for-of/dstr/{const,let,var}-ary-ptrn-elem-id-iter-val-array-prototype.js`).
Note carefully: in those, the **outer** loop's iterable is `[[1, 2, 3]]`, so
`ForOfArray` fires and the patched generator is skipped — but the patch happens
to yield the same single value, so the outer loop is *accidentally insensitive*.
The observable difference is in the inner destructuring (`z === 42`), which is a
different lane. **Conclusion the contract records: no file in the 183-case
`for-of` directory falsifies the `ArrayIteratorIntact` premise on the loop's own
iterator.** The hole is real and the corpus does not see it — which is the
argument for a code invariant rather than a test.

### 9.5 `string-astral.js`

`'a𐐨b𐐨'` must produce 4 iterations. Reaches `ForOfString`
(`possible_kinds ⊆ {String}`). `compile_for_of_string`
(`control_flow.rs:5850-5985`) steps via `emit_decode_utf8_scalar_at_index`, i.e.
a code-point walk — hence the `StringWalkIsCodePoint` premise. The premise the
witness *names but does not verify*: an **unpaired** surrogate must still be one
iteration yielding a one-unit string (`CodePointAt`, 11.1.5). The dry-runner must
record what the internal encoding does with `'\ud801'` alone; if it is not
representable as a UTF-8 scalar, `StringWalkIsCodePoint` is a false premise and
the finding belongs in the dry-run report, not in a silent fix.

### 9.6 `generic-iterable.js`

The `ForOfIterator` control case. Establishes the reference trace against which
9.4's and 9.5's witnesses are stated.

### 9.7 Adversarial paper traces (no execution)

| id | Program | Spec answer | Traced answer | Confirms |
|---|---|---|---|---|
| **A1** | `Array.prototype[Symbol.iterator] = function*(){ yield 99; }; for (const x of [1,2]) log(x);` | `99` | `1`, `2`. `[1,2]` has `possible_kinds = {Array}`; `lowering/for_of.rs::lower_for_of_head` selects `ForOfArray`; `compile_for_of_array` (`control_flow.rs:5732`) is a bare `emit_array_length` + `emit_array_read` index walk with **no** `@@iterator` `Get` anywhere in it. | Mistake class 6 / ledger L3. Fixes the exact premise `ArrayIteratorIntact` must state. |
| **A2** | `class A extends Array { *[Symbol.iterator]() { yield 7; } } for (const x of new A(1,2,3)) log(x);` | `7` | Depends on whether `new A(...)` narrows to `{Array}`. Dry-runner must resolve this against `lowering.rs`'s `new`-expression kind inference and record it. If it narrows, A2 is a second, independent route to the same premise violation — and the one a "did anyone assign to `Array.prototype`?" guard would miss. | Mistake class 6; scopes any future intactness guard. |
| **A3a** | Add `SpecOperationIr::IsLessThan` with no catalog row. | build failure | **`E0004` in four places** (`family`, `normal_result`, `abrupt`, `catalog_index`). | Mistake class 2a. |
| **A3b** | Add a catalog row `"ToNumericString"` with no variant. | build failure *if it claims implementation* | Only expressible as `TrackedGapRow`, i.e. an honest gap; claiming `SharedWasmEmitter` is `E0560`/`E0609` because the struct has no such field. A duplicate name is `E0080` from const assert J1. | Mistake class 2b. |
| **A4** | `{[Symbol.iterator]:null}` vs `{ [Symbol.iterator] : null }`. | identical | Identical *after* whitespace stripping at `lowering.rs:35924`, but `{[Symbol . iterator]:null}` (space inside the member expression) is not, and neither is a computed key written any other way. | Ledger L4. Record; do not fix. |
| **N1** | `for (const x of it) { if (x) continue; }` | `return()` **not** called | Not called — `control_flow.rs:7710-7719` rewrites this loop's own `continue` to `Normal` before the close condition is computed. | The real negative control for §9.2; falsifies an over-claiming witness. |

---

## 10. Acceptance checklist

The area is done when all of these hold. Each is checkable at rung 0 or rung G;
none needs the real suite.

1. `cargo check -p lila-ir` and `cargo check -p lila-aot-wasm` are clean.
2. `SPEC_OPERATION_CATALOG` has **46** rows: 29 `SharedWasmEmitter`, 5
   `StatementEmission`, 12 `TrackedGap`. **Zero** rows claim an implementation
   that does not exist.
3. `OperationLoweringStatus` has exactly three variants. `CatalogOnly` and
   `SharedRustModel` are gone.
4. `grep -rn 'SharedWasmEmitter(' crates/` shows the constructor is reachable
   only from `SpecOperationIr::emitter_evidence`.
5. Commenting out one arm of `SpecOperationIr::abrupt` fails to build. **Verify
   this by doing it**, per batch-workflow's rule about confirming a check fails
   rather than trusting that it would.
6. Deleting `IteratorProtocolWitness::ARRAY_INDEX_WALK` fails to build at
   `lowering.rs:13418`. Verify.
7. Swapping the second and third arguments of `IteratorRecordIr::sync` at its
   construction site fails to build. Verify.
8. Adding a fourth `StatementIr::ForOf*` variant fails to build until a
   `protocol` is supplied. Verify with a scratch variant, then revert.
9. `StatementIr::abrupt_completion_record` contains no `_` arm; adding a 34th
   `StatementIr` variant fails to build. Verify.
10. The nine deleted types produce `E0432` if re-added to `lib.rs`'s `pub use`
    without a call site — i.e. `lib.rs:86-93` no longer names them.
11. **Rung G diffs empty.** Capture before R0, compare after R8. A non-empty diff
    means an emitter arm read the witness, or the `IteratorRecordIr` retrofit
    changed a binding name. Both are bugs in the retrofit, not in the gate.
12. `operations.rs` has 9 tests (8 retained + L1) plus the four witness-shape
    tests in `iterator_obligations.rs`. No test asserts the contents of a table
    it is reading from.

## 11. What is still broken after this lands

Stated plainly, because the area's value depends on not overselling it.

- `for (const x of arr)` with a patched `Array.prototype[Symbol.iterator]` is
  **still observably wrong** (trace A1). This contract names and types the
  premise; it does not discharge it. Closing it is a separate lane and will
  change emitted bytes.
- The `abrupt` sets are still asserted, not proved against the emitter
  (ledger L2).
- The source-text specialization oracle is still there (ledger L4).
- `KindSet::EMPTY` still satisfies "is definitely an Array" (ledger L5).
- Zero abstract operations were newly emitted. The catalog is more honest by 17
  rows; the compiler is not more capable by one.

---

## 12. Encoder addendum (as built)

Written by the encoder after implementing §3–§7. The full integration patch for
files outside the encoder's ownership is
`target/lane-notes/Spec-operation catalog evidence and the iterator-protocol obligation witness-theory-integration.md`.
**No cargo command was run in this lane** (batch 2 holds the build lock); the
integrator runs the gate.

### 12.1 Landed in the owned files

- `crates/lila-ir/src/task.rs` and `operations.rs` — Part A in full:
  the closed `TaskId` registry, `NormalResult`, `TrackedGapReason`,
  `EmitterEvidence`, the three-variant
  `OperationLoweringStatus`, `StatementEmissionRow`, `TrackedGapRow`, `str_eq`,
  the five exhaustive `const fn`s on `SpecOperationIr` (`family`,
  `normal_result`, `abrupt`, `catalog_index`, plus `name`), `ALL`,
  `catalog_entry`, `build_catalog`, `SPEC_OPERATION_CATALOG`, const asserts
  J1/J3/J4/J5. Plus B5's `IteratorRecordIr` retrofit. Eight types and thirteen
  tests deleted; three tests remain plus L1 and the record test.
- `crates/lila-ir/src/iterator_obligations.rs` — Part B in full.

### 12.2 Deviations, each strictly stricter or a correction

- **D1.** `SPEC_OPERATION_CATALOG` is a `static` (with a private `const CATALOG`
  behind it for the const asserts). A `const` array is an rvalue at each use, so
  `.iter()` on it cannot produce the `&'static` that `find_spec_operation`
  returns.
- **D2.** The current table uses **21** distinct `normal_result` strings, not 20:
  `"Completion"` and `"Completion Record"` are both present. The contract's
  20-variant `NormalResult` is shipped as written, collapsing both to
  `"Completion Record"`. Rendering-only; nothing consumes the field.
- **D3.** `IteratorRecordIr` ships without `kind` and without the `sync`/`async_`
  pair. A field written by the single reachable constructor is a constant, and
  `sync()` would be a constructor with no call site — the defect §8 deletes eight
  types for. `IteratorRecordKind` is deleted. Reintroduce both when the sync
  for-of path gains a record.
- **D4.** `IteratorProtocolWitness::new` and the four discharge constructors are
  module-private. §4 B3's sentence "these four constants are the only values
  `lowering.rs` may use" thereby becomes a property of the type rather than a
  rule in a document.

### 12.3 Additions to the mistake-class table

| # | Plausible mistake | After |
|---|---|---|
| 14 | Invent a fifth witness inline at a `StatementIr` construction site instead of using a named constant. | **`E0603`** — `IteratorProtocolWitness::new` is private to `iterator_obligations`. The author must add a constant next to the premises it claims. (D4.) |
| 15 | Change one of the three catalog tables and leave the contract's "29 + 5 + 12 = 46" census standing. | **Compile error** — const assert J4 states the census. |
| 16 | Give `catalog_index` two variants the same slot, or a slot that addresses another operation's row. | **Compile error** — const asserts J3 and J5. |

### 12.4 Additions to the runtime-checked ledger (§5)

| id | Invariant | Why no type can carry it | The check |
|---|---|---|---|
| **L6** | `EmissionSite::ArrayDestructuring` has zero construction sites: no row and no witness names it. | Its value is the R7 match arm, which pins `compile_array_destructure_from_value_locals` to something real and forces a new variant to name something real. A variant that is only *matched* still breaks the build when its function disappears, which a merely-defined type does not. | None. Recorded so a later lane either finds the anchor persuasive or deletes variant and arm together. |
| **L7** | `NormalResult::name`, `TrackedGapReason::name`, `IntactnessPremise::name` have no caller (the other two renderers are used by test failure messages). | `pub` suppresses dead-code detection — the exact "survival by `pub`" this area names. Kept because the catalog's only consumer is a human reader, and a row that cannot be rendered cannot be reviewed. | None. Recorded rather than hidden. |
| **L8** | `STATEMENT_EMISSION_ROWS` and `TRACKED_GAP_ROWS` still state `family` / `normal_result` / `abrupt` by hand, and a `StatementEmissionRow` can name an arm that does not in fact emit it. | These operations have no `SpecOperationIr` variant to derive from — that is what makes them gaps. Inventing variants for unimplemented operations would recreate the false-claim defect one level down. R7 proves the named function exists, not what it emits (same boundary as L2). | Nothing. Bounded: a wrong signature on a row that also says "not implemented" cannot be read as an implementation claim. |

---

# 13. Amendments from the dry-run discrepancy pass (applied)

**This section supersedes every claim it names.** It was written after a
symbolic dry run of the encoded tree, and the code changes it describes are in
the tree. Where §§1–12 and this section disagree, this section is correct.

## 13.1 The integration steps were not applied; they now are

§12.1 and §7 said Part A and Part B had "landed", and §10 item 1 said
`cargo check -p lila-ir` was clean. Neither was true: `lib.rs` had no
`mod iterator_obligations;` (so `operations.rs`'s
`use crate::iterator_obligations::EmissionSite;` was `E0433`) and still exported
eleven types the rewrite had deleted (`E0432` ×11). Measured across
`crates/**/*.rs`, `IteratorProtocolWitness`, `EmissionSite`, `IteratorRecordIr`,
the three slot newtypes and the two row tables had **exactly one** mention
outside their own two files — the stale `pub use` line. Part B had zero
construction sites, i.e. it was the "survival by `pub`" defect this area exists
to delete.

Applied, in `lila-ir` unless stated: `mod iterator_obligations;`; the export
block replaced (eleven dead names dropped, `iterator_obligations`'s surface
added); `ir.rs` retrofitted (`AsyncForOfIteratorPlanIr::record`, `protocol` on
the three `ForOf*` variants, `abrupt_completion_record` exhaustive over all 33
variants with no catch-all); `lowering.rs` construction sites; the six
full-field `ForOf*` patterns in `lila-aot-wasm/src/control_flow.rs` gained
`..`; `compile_array_destructure_from_value_locals` widened to `pub(crate)`; and
`lila-aot-wasm/src/emission_sites.rs` created and declared. **I7 (deleting
the dead `ForOfArray` async path) was not applied** and remains an open item —
`AsyncForOfPlanIr` still has zero construction sites.

## 13.2 Mistake class 1 was false: the catalog entry was forgeable

§6 class 1 claimed an implementation status was reachable only from
`SpecOperationIr::catalog_entry` or an `EmissionSite`. It was not:
`SpecOperationCatalogEntry` had five `pub` fields, so the whole evidence design
was bypassed by writing the struct literal out, and `EmitterEvidence` was not
tied to the row's `name`, so even legitimately-obtained evidence could be
attached to any name.

Now: every field of `SpecOperationCatalogEntry` is private, a new private
`source: RowSource` field makes the struct unconstructible outside
`operations.rs`, accessors replace field reads, `catalog_entry` derives `name`
from `evidence.operation().name()` rather than from a free field, and const
assert **J6** checks name-vs-evidence agreement for every assembled row.

## 13.3 Ledger L1 is discharged by construction; its test is deleted

L1 said "added a variant, forgot `ALL`" was uncatchable and was covered by
`spec_operation_all_is_complete_and_dense`. That test could not detect its own
invariant: a 30th variant with all four exhaustive arms and no `ALL` row passed
J1, J3, J4 and J5 *and* passed the test, whose final assertion was the
hardcoded `ALL.len() == 29` that the omission preserves — and J4 cemented it.

`SpecOperationIr`, `ALL` and the complete `OperationDescriptor` are now
expansions of one `spec_operations!` row list, so the omission is
unrepresentable. The descriptor makes name, family, operand domain, normal
result and abrupt capability mandatory columns; the former parallel
`family`/`normal_result`/`abrupt` matches are deleted. The test is deleted.
`catalog_index` and const assert **J3** are deleted with it (density became
definitional); **J5** is restated as a real check on `build_catalog`'s row
order. §12.3 addition 16 no longer applies. The descriptor extension and its
bounded backend consumers are specified in `../operation-descriptors.md`.

## 13.4 Mistake class 7 / §10 item 7 were false as stated

`E0308` fired only for a transposition of already-*wrapped* values, which was
not the shape of the construction site: `IteratorSlot::new(next_binding),
NextMethodSlot::new(iterator_binding)` type-checked and miscompiled every
`for await`, exactly the original defect. The three `new`s are now
`pub(crate)`, and `ScriptLowerer::alloc_iterator_slot` /
`alloc_next_method_slot` / `alloc_done_slot` each wrap
`alloc_suspension_owned_binding` with its own fixed hint and return the slot
type. There is no `String` left in the expression, and §10 item 7 is now true.
Allocation order is unchanged, so the emitted binding names are byte-identical.

## 13.5 Mistake class 8 was aimed at the wrong place: there were already four

A `protocol` field on three `ForOf*` variants cannot catch a specialization
that is not spelled as a `ForOf*`. `for (x of arr) { … await … }` inside a
plain async function is desugared by
`lowering/for_of.rs::lower_async_for_of_array_with_body_await`
into `StatementIr::GeneratorLoop` with an explicit
`index < PropertyKeyIr::ArrayLength` test and `PropertyKeyIr::ArrayIndex`
element reads — an index walk on all the array premises, with no witness and no
way to demand one. §2's "for-of specialization construction sites: 3" was wrong.

The obligation is now attached to the **lowering of the head**. The private
`lowering/for_of.rs::lower_for_of_head` returns the module-private
`ForOfLoweringIr`, whose only constructor takes an `IteratorProtocolWitness`,
and every exit produces one
(`ForOfLoweringIr::no_iteration()` for the bail-outs, `NO_ITERATION` for the
`WebCompatCall` head that throws instead of iterating,
`ARRAY_INDEX_WALK_RESUMABLE` for the `GeneratorLoop` desugaring). The `ForOf*`
`protocol` field is now a consumer of that value rather than the only place it
is demanded. Spread, `yield*` and array destructuring reach the protocol by
other routes and are named as `EmissionSite`s instead (13.9).

The former public `ForOfLoweringIr::protocol()` accessor had no caller and was
deleted. The carrier now stays private with its lowering family, and
`into_statement_and_kind` consumes it, reads the witness and checks the two
locally decidable conditions before the statement crosses back to dispatch.

## 13.6 `IntactnessPremise` conflated three kinds of claim

`ArrayLengthReadOnce` and `ArrayElementReadBypassesGet` described what **our
code** does, not conditions under which doing that is correct, so
`ByAssumption(ArrayLengthReadOnce)` read as *discharged* while assuming nothing.
They are renamed to the program properties they stand for —
`ArrayLengthStableDuringBody` and `ArrayHasNoHolesOrIndexAccessors` — and every
variant now carries `IntactnessPremise::kind() -> PremiseKind`
(`ProgramProperty` | `ImplementationFact` | `Vacuous`), so the lane that closes
L3 knows which variants it must actually establish. `NoIterationLowered` is
added for 13.5's bail-out witness.

## 13.7 Ledger L3 was understated to the point of being misleading

L3 said no guard exists. A **partial** intactness guard already exists —
`ScriptLowerer::array_prototype_mutated` (`lowering.rs:544`), set at ten sites
including plain assignment to `Array.prototype[Symbol.iterator]`, `delete`,
aliasing, `defineProperty` and `Reflect` — and four sibling fast paths already
consult it (`:20960`, `:21458`, `:22213`, `:28497`). The for-of specialization
decision is the outlier that does not.

Corrected L3: *a guard exists and is not consulted at this decision.* Closing
`ArrayIteratorIntact` is one conjunct — `&& !self.array_prototype_mutated` at
the three-way `if` — plus an own-`@@iterator`/custom-prototype check for which
`Self::array_shape_has_custom_prototype` already exists. That change moves
emitted bytes, so it stays out of this pass. Also recorded: the same flag's
incompleteness bounds those four sibling fast paths, a larger blast radius than
for-of alone.

## 13.8 Ledger L4 had two factual errors

The `static_object_iterator_iife_source_values` oracle compacts with
`chars().filter(|ch| !ch.is_whitespace())` — **total** whitespace removal — so
`{[Symbol . iterator]:null}` compacts identically to `{[Symbol.iterator]:null}`
and trace A4's cited counterexample is not one. The real sensitivities are
`compact.find("let")` matching the substring `let` inside an earlier identifier
or string literal, and `const` instead of `let` returning `None`. Second, the
oracle does not select a for-of specialization at all: its output feeds
`Iterator.prototype.toArray` / `Array.from` constant folding, and the folded
expression has `possible_kinds` `{Object}`, so the for-of head reaches
`ForOfIterator` either way. L4 is restated as *an unsound source-text guard on
iterator constant-folding*, and A4's example is withdrawn.

## 13.9 `EmissionSite` is a set, not a single value; L6 is retired

`GetIterator`, `IteratorStep`, `IteratorValue` and `IteratorClose` are each
emitted by at least three arms — the sync for-of, the async for-of, and array
destructuring, which has no array fast path and always runs the real protocol
(`emit_get_iterator_from_value_locals`, `emit_destructuring_iterator_step`,
`emit_iterator_close`, `emit_iterator_close_preserving_current_throw`). As
written, §3 A6 read as if `for await` performed no `GetIterator`.

`StatementEmissionRow::site` is now `sites: &'static [EmissionSite]`,
`OperationLoweringStatus::StatementEmission` carries the slice, and const assert
**J7** rejects an empty one. `EmissionSite::ArrayDestructuring` thereby gains
construction sites and **ledger L6 is retired**.

## 13.10 Mistake class 4 is closed by the task enum

The first revision validated only shape and therefore accepted every `T` plus
two digits. The §13 dry run replaced that with an intermediate
`OwnerTaskId`/`BACKLOG_TASK_IDS` membership scan covering the then-current
T00–T27 ledger. That fixed `T99`, but duplicated the ledger as strings and fell
out of date when T28 and T29 landed.

The current implementation replaces both with a macro-generated, `#[repr(u8)]`
`TaskId` enum in `task.rs`. One registry generates its T00–T29 variants,
numeric discriminants, `ALL`, stable strings and parser. Catalog rows therefore
write `TaskId::T04`; `TaskId::T99` is an unknown variant and external text must
parse successfully before a task value exists. Matches over the task domain can
be exhaustive without a catch-all. `T26-unclassified` is explicitly not a
task: it remains a Test262 ownership fallback bucket and parsing it as `TaskId`
fails.

Note for the record: T04 *is* a real backlog task
(`tasks/04-spec-operations-and-completion-abi.md`); the original dry run's claim
that it did not exist came from checking `test262/backlog/ownership-map.tsv`,
which maps Test262 path prefixes to tasks and so legitimately omits a task that
owns no prefix. The twelve gap rows keep `TaskId::T04` and it is correct.

## 13.11 What `EmitterEvidence` proves, and how L2 must be scoped

Verified good news: both `lila-aot-wasm` matches on `SpecOperationIr` are
exhaustive with no top-level catch-all (the three `_ =>` arms in that range are
nested operand matches), so "add a variant, forget an arm" really is `E0004`.
But `emitter_evidence` is total on the variant, so the evidence proves only that
the name is a variant the backend must match on — an arm may satisfy both
matches with `Err(EmitError::unsupported(..))` and the catalog still reads
`SharedWasmEmitter`. §3 A1/A3 and mistake class 1 are amended accordingly, and
the doc comment on `emitter_evidence` now says so.

**L2 is rescoped**: the descriptor now gives each operation a closed
`AbruptCapability`, while named backend boundaries route `GetV` inside
`GetMethod` and the `ToNumber` of the `Number.prototype.toFixed` argument. The
former const-checked `MayThrowOperation` marker was later removed because its
finisher parameters were ignored and mismatched constants still compiled. The
named slices do not prove the other arms. Closing L2 still requires making
every arm's success value carry its capability *and* making
`EmitError::unsupported` unconstructible inside a `SpecOperationIr` arm. A
descriptive capability alone would leave the hole open.

## 13.12 The "emitter must not read `protocol`" rule is now a type

§7 R4 enforced it with a comment plus six hand-added `..` patterns, detectable
only by a ten-minute rung-G capture. Every reader of a witness's contents —
`get_iterator`, `iterator_step`, `iterator_value`, `iterator_close`,
`discharge`, `is_fully_emitted`, and `ObligationDischarge::is_emitted` — is now
`pub(crate)`. A backend arm that binds `protocol` and branches on it is `E0624`
at `cargo check`. §10 item 11 stops being the load-bearing check for it.

## 13.13 D4 holds, and the residual transposition inside the module is closed

§12.2 D4 was verified: outside `iterator_obligations` the only inhabitants of
`IteratorProtocolWitness` are its named constants. The residual the dry run
found — `assumed()`/`emitted_by()` took four same-typed `IntactnessPremise`
arguments positionally, so a transposition *inside* the module still
type-checked — is closed: each discharge newtype carries its own
`assumed`/`emitted` constructor and `IteratorProtocolWitness::new` takes the
four newtypes, so the constants are built per-obligation and swapping two is
`E0308` at the only place it can still be written. `Self::assumed` is deleted.

## 13.14 Two counting corrections

- §2's measured baseline says the 46 rows use **20** distinct `normal_result`
  strings. §12.2 D2 already recorded that the old table used **21** (a separate
  `"Completion"` alongside `"Completion Record"`). §2 is wrong and D2 is right.
- §3 A5 and the deleted test called `ALL` "one entry per variant". It is one
  entry per **operation**: `ToPrimitive` has three variants and one entry.
  Payload-carrying variants collapse to their canonical expression, which is
  correct for a catalog keyed by operation name.

## 13.15 Still open after this pass

- **I7.** `AsyncForOfPlanIr` still has zero construction sites and
  `compile_async_for_of_array` (449 lines) is still unreachable from the product
  path, which AGENTS.md says should fail to build.
- **L2**, rescoped as in 13.11; two backend uses consume the typed capability,
  while the remaining operation arms still do not.
- **L3**, with the corrected statement in 13.7 — one conjunct, but it moves
  emitted bytes.
- **L4/L5**, untouched; L4 restated as in 13.8.
- **L7**, unchanged: three `name()` renderers still have no product caller.

## 14. Integration record — the compile gate, and what it caught

This section is the integrator's, written after actually running the gate that
§10 item 1 and §13.1 could only assert. Commands run: `cargo check -p lila-ir
--all-targets`, `cargo xc` (`check --workspace --all-targets`), `cargo fmt --all
-- --check`. No tests, no golden capture, no Test262.

### 14.1 The gate was green, and that is the smaller half of the result

`cargo xc` exits 0 across all ten workspace crates with the three areas
integrated. Every const assertion in `operations.rs` (J1, J2, J4, J5, J6, J7) and
every one in `native_error.rs` / `well_known.rs` / `binding_names.rs` evaluated,
which is the first time any of them has been *evaluated* rather than argued: a
const assertion that never reaches a compiler is a comment. §13.1 was right that
the tree did not build; it does now, and the assertions hold as written.

### 14.2 §13.12's narrowing had a cost it did not price, and the fix upgrades it

Narrowing every witness reader to `pub(crate)` bought the `E0624` guarantee —
and left `is_emitted`, `get` ×4, the four slot accessors, `discharge` and
`is_fully_emitted` with **no caller outside `#[cfg(test)]`**. rustc reported all
ten as `dead_code`. That is the area's own defect reappearing one visibility
level down: `pub` stopped hiding callerless code, so `pub(crate)` inherited the
job of hiding it, and only the product build noticed.

The resolution keeps the guarantee and removes the decoration, because every
reader is a `const fn`:

- Four `#[test]`s asserted that the five witness constants say what their doc
  comments say. Three of those claims are now `const _: () = assert!(…)` —
  evaluated by `cargo check`, not by a test run. Editing `ARRAY_INDEX_WALK` to
  *claim* emission it does not perform, or reclassifying `ArrayIteratorIntact`
  from `ProgramProperty` to `ImplementationFact` so the index walk reads as
  discharged (the 13.6 defect), is now `E0080` at build time.
- `discharge` now reads through the four typed accessors instead of the four
  private fields, so the accessors have a product-path caller and the
  wrong-slot mistake stays `E0308`.
- `GetIteratorDischarge::obligation` and its three siblings are **deleted**.
  Each returned a hardcoded constant, so no edit to them could disagree with
  anything; the "cannot read a witness back through the wrong accessor"
  property is carried by the four distinct newtypes, not by these four methods.
  Their only caller was the one test that existed to call them; that test is
  deleted with them. This is AGENTS.md's own test applied to this area's own
  code: a plausible mistake did not become a compile error, so it was
  decoration.

Result: `lila-ir` goes from 12 warnings to 6, and the 6 that remain are all
in files this campaign never opened (`analysis.rs`, `lowering_helpers.rs`, and
four pre-existing clusters in `lowering.rs`, the largest being 27 callerless
generated-iterator and static-generator-folding helpers). Measured, not assumed:
the campaign's diff mentions none of those names.

§13.12 stands, strengthened — the readers still cannot be called from
`lila-aot-wasm`, and now they are not dead either.

### 14.3 I7 was deliberately not applied

`AsyncForOfPlanIr` still has zero construction sites. I7 was left undone for two
reasons that are about this batch, not about the merit of the deletion: it is a
449-line deletion in `lila-aot-wasm`, the crate batch 2 is verifying in
concurrently, and a deletion claimed to be behaviour-neutral wants the rung-G
golden diff that this pass is not permitted to spend. It stays open exactly as
13.15 states it, and it is still the right change.

### 14.4 The one warning this campaign accepts

`emission_sites.rs::emission_sites_are_backed` carries `#[allow(dead_code)]`. It
is uncalled *by design* — being uncalled is the whole mechanism, since its only
job is to make an `EmissionSite` variant name a real function or fail to
resolve. Recorded here so the allow is a decision rather than a leftover.

## 15. Raw catalog-input visibility closure (2026-08-13)

The assembled catalog is the API; its hand-written inputs are not.
`AbruptDiscipline`, `StatementEmissionRow`, `TrackedGapRow`,
`STATEMENT_EMISSION_ROWS` and `TRACKED_GAP_ROWS` are now crate-private and are
absent from `lila-ir`'s public re-export block. The public
`SPEC_OPERATION_CATALOG`, `SpecOperationCatalogEntry`, `RowSource`,
`OperationLoweringStatus`, `TrackedGapReason`, and the catalog lookup functions
remain unchanged.

The durable boundary is Rust visibility: another crate attempting to import a
raw row or table gets `E0603`, while `operations.rs` remains the sole assembly
site. This closes the callerless-public-surface residue without changing the 46
catalog rows, any lowering decision or emitted Wasm. No Cargo or Test262 command
ran in this follow-up; the central verifier owns `cargo check -p lila-ir` and
the rustdoc gate.

## 16. `ArraySpeciesCreate` backend-evidence checkpoint (2026-08-29)

This section supersedes the older three-status and `29 + 5 + 12` catalog
censuses in this document. It does not change the iterator-obligation evidence.

### 16.1 A dedicated backend operation can now carry implementation evidence

The macro-backed `BackendSpecOperation` domain records operations implemented
by a dedicated backend function rather than by the expression-shaped
`SpecOperationIr` emitter or a statement-shaped `EmissionSite`. It includes
`ArraySpeciesCreate`. The same macro rows generate the enum,
`ALL`, and the complete operation descriptor, so adding a member without a
name, family, operand domain, normal result, and abrupt capability is not
representable.

`BackendEmitterEvidence` has a private field and no public constructor.
`BackendSpecOperation::emitter_evidence` is its only producer.
`OperationLoweringStatus::SharedBackendEmitter` carries that evidence, and
`RowSource::DerivedFromBackendOperation` records its catalog origin. The
backend's exhaustive `backend_spec_operations_are_backed` match joins
`BackendSpecOperation::ArraySpeciesCreate` to
`FunctionBuilder::emit_array_species_create`. A renamed emitter or a new
backend operation without a join therefore fails to build.

The canonical `ArraySpeciesCreate` descriptor takes a value and an integer,
may throw, and has normal result `Object`. It must not say `Array`: a custom
`@@species` constructor may return an arbitrary object.

The current catalog census remains 46 rows:

- 29 expression rows backed by `SpecOperationIr` and `EmitterEvidence`;
- 2 backend rows backed by `BackendSpecOperation` and
  `BackendEmitterEvidence`;
- 5 statement-emission rows backed by `EmissionSite`;
- 10 tracked gaps.

Const assertion J4 pins `29 + 2 + 5 + 10 = 46`. Moving
`ArraySpeciesCreate` out of `TRACKED_GAP_ROWS` without adding its backend-derived
row, or changing any of the four domains without restating the census, fails at
compile time.

### 16.2 The promotion does not claim universal species ownership

`emit_array_species_create` contains the complete shared implementation and is
reachable from the product path. Its only callers are
`Array.prototype.slice` and `Array.prototype.splice`.

The exact current `builtins/array.rs` census is 9 direct
`Symbol.species` reads. One is inside `emit_array_species_create`. Eight remain
outside it:

- five live local `ArraySpeciesCreate` copies in `flat`, `concat`, `flatMap`,
  `map`, and `filter`;
- three distinct `TypedArraySpeciesCreate` paths in typed-array `slice`, `map`,
  and `filter`.

Those eight sites are not one homogeneous set of duplicate
`ArraySpeciesCreate` implementations. This checkpoint claims neither a single
species owner nor a universal caller migration. `SpeciesConstructor` remains a
tracked gap. `Completion` and `UpdateEmpty` also remain tracked gaps rather
than being promoted on the strength of their callerless Rust models.

Array `every` and `some` no longer contain their constant-false species copies
or the later unused result Array allocation and construction path. Their
bounded source guard, not the runtime non-observation fixture, distinguishes
that emitted-Wasm cleanup from the prior behavior.

### 16.3 Verification status

At the central checkpoint, `cargo check -p lila-aot-wasm` is green with only
the pre-existing vendored parser warning. The catalog-evidence and neighboring
splice structure targets pass `10/10`, the filtered IR operation units pass
`53/53`, and the Array slice core/species/Proxy CLI controls pass `3/3`.
Formatting, module-boundary, task-plan, exact-shortcut and diff checks are
green, with the shortcut inventory unchanged at 240 entries. This checkpoint
makes no Test262 result, published status count, or conformance-gain claim.
The two bounded Array-species and quantifier structure targets pass `10/10`.
The constructor/species non-observation fixture and four neighboring
Array/TypedArray core and resizable-buffer controls pass `5/5`.

## 17. `ToPropertyDescriptor` backend-evidence checkpoint (2026-08-29)

This section extends §16's backend-operation domain with its second member. It
does not change the iterator-obligation evidence or the species boundary.

### 17.1 The catalog row states the conversion's typed contract

`BackendSpecOperation::ToPropertyDescriptor` is generated by the same closed
macro as `ArraySpeciesCreate`. Its descriptor belongs to the Object family,
takes one `Value`, returns a `PropertyDescriptor`, and `MayThrow`:

```text
Value -> PropertyDescriptor / MayThrow
```

`BackendSpecOperation::emitter_evidence` supplies its unforgeable
`BackendEmitterEvidence`; `OperationLoweringStatus::SharedBackendEmitter` and
`RowSource::DerivedFromBackendOperation` carry that proof into the catalog.
The backend's exhaustive `backend_spec_operations_are_backed` match joins the
variant to `FunctionBuilder::emit_to_property_descriptor`. Adding another
backend operation without a corresponding real-function join therefore fails
to build.

The catalog remains 46 rows:

- 29 expression rows backed by `SpecOperationIr` and `EmitterEvidence`;
- 2 backend rows backed by `BackendSpecOperation` and
  `BackendEmitterEvidence`;
- 5 statement-emission rows backed by `EmissionSite`;
- 10 tracked gaps.

Const assertion J4 pins `29 + 2 + 5 + 10 = 46`. `ToPropertyDescriptor` is no
longer a gap row; the total did not change because its backend-derived row
replaced that gap.

### 17.2 The reserved carrier makes the local lifecycle explicit

There is exactly one `FunctionBuilder::emit_to_property_descriptor` definition
and exactly two direct source call sites. They belong to the Object static
builtins `Object.defineProperty` and `Object.defineProperties`. Each call is
immediately paired with `emit_from_present_property_descriptor`; there is no
third shared-converter or shared-materializer caller.

The converter reserves a presence, payload, and tag local for each of the six
descriptor fields, plus two field-key scratch locals. It releases the scratch
pair after validation and returns the remaining locals inside
`ReservedPropertyDescriptorLocals`. That crate-visible token is `#[must_use]`,
has a private descriptor field, derives no capabilities, and has no manual
`Clone` or `Copy` implementation. Its `ValidatedDescriptor` uses the private
`ReservedPropertyDescriptorCarrier`, so callers cannot extract, duplicate, or
forge the owned local set.

`emit_from_present_property_descriptor` takes the token by value. It allocates
the ordinary descriptor object, emits only fields whose presence says they
exist, releases its own field-key scratch local, and consumes the validated
descriptor with `into_partial()`. It then releases the six field groups in
reverse reservation order—`set`, `get`, `configurable`, `enumerable`,
`writable`, `value`—and releases each run-time group as tag, payload, then
presence. A successful caller therefore cannot retain or release the same
reserved descriptor locals a second time.

### 17.3 The present-only materializer is not `FromPropertyDescriptor`

The consuming materializer handles the present descriptor needed by those two
Object static builtins. It does not implement the general
`FromPropertyDescriptor` operation, including its `Undefined` result branch,
so `FromPropertyDescriptor` remains a `TrackedGapReason::NoImplementation`
catalog row.

Nor does the backend row claim universal descriptor-path migration.
`Reflect.defineProperty` still open-codes the six ordered field observations,
validation, and completed-descriptor object construction. Proxy
`[[GetOwnProperty]]` and other descriptor paths retain their separate
open-coded projections and materializers. Those paths may implement relevant
behavior, but they do not consume `ReservedPropertyDescriptorLocals` and are
not evidence for this catalog row or for `FromPropertyDescriptor`.

### 17.4 Verification status

At the 2026-08-29 checkpoint, the bounded evidence target passes `7/7`, the
existing Object descriptor fixture passes `1/1`, the filtered IR operation
units pass `53/53`, and `cargo check -p lila-aot-wasm` is green with only the
pre-existing vendored parser warning. This section claims no broad Reflect or
Proxy migration, no `FromPropertyDescriptor` promotion, no Test262 result, no
published-status change, and no conformance gain.

## 18. Synchronous Array `for-of` protocol checkpoint (2026-08-29)

This section supersedes the earlier live-tree descriptions of the synchronous
Array shortcut. The IR no longer has `StatementIr::ForOfArray`; the backend no
longer has `compile_for_of_array`; and
`IteratorProtocolWitness::ARRAY_INDEX_WALK` is gone. Direct synchronous Array
heads now construct `StatementIr::ForOfIterator` with
`ForOfIteratorHeadIr::Assignment`, `async_plan: None`, and
`IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL`.

The loop value on that generic path is `ValueKind::Dynamic` with every run-time
kind possible. A source Array's inferred heap shape cannot constrain what a
replaceable `@@iterator` yields. The focused fixture demonstrates the reason by
replacing `Array.prototype[Symbol.iterator]`, yielding the String `"4"`, and
requiring the loop body to produce `"41"` before `break` triggers one
`IteratorClose` `return` call. Its two native-iterator cases require a length
increase to extend iteration and a hole to resolve through an inherited indexed
getter.

This change closes the old direct-path L3 and L5 defects by deletion rather
than by an intactness guard. The four iterator-operation catalog rows keep their
existing `StatementEmission(SyncForOfIterator)` evidence, and the 46-row catalog
census does not change.

`cargo check -p lila-aot-wasm` passes. The two focused structure targets pass
`3/3` and `4/4`, the IR `for_of` target passes `16/16`, the planner and two CLI
targets each pass `1/1`, and the four pinned Array length-mutation leaves pass
`8/8` Wasm-AOT executions with every failure bucket at zero. This checkpoint
claims no semantic-golden result or published-status refresh.

`ARRAY_INDEX_WALK_RESUMABLE` remains live. A synchronous `for-of` with a body
that directly awaits inside a plain async function still becomes a
`StatementIr::GeneratorLoop` through
`lower_async_for_of_array_with_body_await`. That rewrite still assumes Array
iterator intactness, stable length, direct indexed reads, and no iterator object.
Migrating it to a resumable iterator record is separate required work.

## 19. Synchronous String `for-of` protocol checkpoint (2026-08-29)

This section supersedes the earlier live-tree descriptions of the synchronous
String shortcut. The IR no longer has `StatementIr::ForOfString`; the backend
no longer has `compile_for_of_string`; and
`IteratorProtocolWitness::STRING_CODE_POINT_WALK` is gone together with
`StringIteratorIntact` and `StringWalkIsCodePoint`. Ordinary direct synchronous
String heads now construct `StatementIr::ForOfIterator` with
`IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL`.

The loop value on that generic path is `ValueKind::Dynamic` with every run-time
kind possible. A source String cannot constrain what its replaceable
`@@iterator` yields. GetIterator boxes the primitive for property lookup with
the current function Realm's String prototype, but it keeps the original
primitive as the accessor and iterator-method receiver.

The focused fixture makes both mutable protocol points observable. A strict
`String.prototype[Symbol.iterator]` accessor returns a custom iterator whose
Number value makes the loop compute `5`; `break` must then call `return` once.
A separate `%StringIteratorPrototype%.next` replacement must supply the value
seen by an ordinary String loop. Each mutation restores the saved complete
property descriptor in `finally`.

This deletion retires the old direct-path L3 assumption instead of adding an
intactness guard. The four iterator-operation catalog rows retain their
existing `StatementEmission(SyncForOfIterator)` evidence, and the 46-row
catalog census does not change.

`cargo check -p lila-aot-wasm` passes. The String structure target passes `3/3`,
affected companion structures pass `19/19`, the IR `for_of` target passes
`17/17`, and the CLI witness passes `1/1`. The three pinned native String
controls pass `6/6` Wasm-AOT executions with every failure bucket at zero. This
checkpoint claims no semantic-golden, published-status, or broad conformance
result. It also does not claim complete Realm ownership for the generic loop's
iterator-protocol TypeErrors.

A synchronous String `for-of` with a body that directly awaits inside a plain
async function now uses the activation-backed synchronous Iterator Record
described in §20. The replacement no longer classifies the source as an Array.

## 20. Plain-async synchronous `for-of` protocol checkpoint (2026-08-29)

This section supersedes §18's live `ARRAY_INDEX_WALK_RESUMABLE` paragraph,
§19's directly awaiting String nonclaim, and §13.5 wherever it describes the
Array index rewrite as current source.

The IR now has a dedicated `StatementIr::AsyncFunctionForOfIterator` for one
direct body `await` inside a plain async function. Its required
`AsyncFunctionForOfIteratorPlanIr` has private fields and a crate-private
constructor. The plan owns the assignment binding, typed Iterator Record,
head and per-iteration environments, body statements before/at/after the
await, and ordered entry/resume/exit states. Construction rejects any split
statement other than `AsyncAwait`, mismatched suspend/resume states, exit-state
overflow, and a captured head TDZ. A captured lexical loop binding becomes
`ResumableLoopIterationEnvironmentIr::FreshPerIteration`.

Lowering allocates the iterator, next-method, and done roles through their
typed suspension-owned allocators. It records the yielded value as fully
`Dynamic`, with no heap shape or function targets, and no longer tests whether
the source is an Array. The former `AsyncForOfArrayWalkForm`,
`lower_async_for_of_array_with_body_await`, Array length/index synthesis, and
`ARRAY_INDEX_WALK_RESUMABLE` are deleted.

`RESUMABLE_SYNC_ITERATOR_PROTOCOL` discharges the four iterator obligations by
emission at `ResumableSyncForOfIterator`. The exhaustive backend join names
`compile_async_function_for_of_iterator` as that site's real function. Its
entry path performs `GetIterator` and caches `next` in the activation-owned
record. Resume paths reload that record. The planner budgets the emitter's
persistent locals additively with the maximum of iterable temporaries, every
body segment, and `FOR_OF_ITERATOR_HELPER_TEMP_LOCALS`; a nested child cannot
hide the persistent record below the old fixed local floor.

The runtime oracles distinguish all close boundaries. Natural exhaustion does
not call `return`. An await rejection closes once and preserves the original
Throw if close throws. A Return after the await closes once and is replaced by
a close error. Abrupt `next`, `done`, and `value` evaluation does not close.
The acquisition fixture reads Array `@@iterator` and the iterator's `next`
property once across two body awaits, accepts String values from a Number
Array, and runs a replaced synchronous String iterator through the same path.
It also observes a bare identifier assignment head receiving both custom
iterable values on both sides of the body await.
The existing six-closure oracle keeps each captured lexical binding distinct
after all iterations finish.

`cargo check -p lila-aot-wasm` passes. The five focused structure targets pass
`19/19`, the `lila-ir` `for_of` target passes `18/18`, and the four exact CLI
oracles pass `4/4`. All four fixtures pass `node --check`, and the two pinned
`Array.fromAsync` leaves pass `4/4` Wasm-AOT executions with every failure and
non-success bucket at zero. The complete 95-file `Array.fromAsync` leaf,
semantic golden, and published-status refresh were not run. The admitted form remains one plain
async function, one direct body `await`, and a simple single-name declaration
or bare identifier assignment head. Direct `break` and `continue`, pattern and
property heads, captured head TDZ, iterable suspension, async-generator owners,
and `for await` remain separate paths or explicit rejections.

## 21. Shared IteratorClose error-Realm checkpoint (2026-08-29)

The shared `emit_iterator_close` backend owner now creates both
algorithm-generated TypeErrors in the current function Realm. Its 67 external
entry routes are 16 direct, 48 preserving-current-Throw, and 3
preserving-saved-Throw routes. The preserving wrappers still restore the
incoming Throw. Entry code with `current_env_local == 0` still uses the main
Realm fallback.

This does not change the `IteratorClose` catalog row or the 46-row catalog
census. At this close-only checkpoint, ordinary direct `for-of` acquisition
and stepping errors remained a separate migration. The focused structure
target passes `4/4`, the exact created-Realm CLI test passes `1/1`, the
affected close sweep passes `6/6`, and the two direct `for-of` Test262 leaves
pass `4/4` with every failure and non-success bucket at zero. Section 22
supersedes that historical nonclaim. The exact close invariant, nonclaim, and
commands live in
[`iterator-close-error-realm.md`](./iterator-close-error-realm.md).

## 22. Direct synchronous `for-of` protocol-error Realm checkpoint (2026-08-29)

All three direct synchronous `for-of` execution owners now route their five
iterator acquisition and stepping TypeErrors through one closed body-Realm
projection, for 15 checks in total. Ordinary direct
`compile_for_of_iterator` and direct async-disposable
`compile_async_disposable_for_of_iterator` own five inline checks each.
Resumable plain-async `compile_async_function_for_of_iterator` delegates its
five checks to `emit_get_iterator_from_value_locals` and
`emit_sync_iterator_step_value`.

Each route selects `SyncIteratorConsumer::ForOf`. The exhaustive
`SyncIteratorProtocolError` projection owns the diagnostic and Realm choice;
no direct owner bypasses it. A standard-builtin body selects the trusted
self-backed current-function emitter. Main, user, host, and runtime-helper
bodies select the main-Realm emitter, so a nonzero lexical environment is
never read as function Realm metadata.
The five checks are a nullish source, a non-callable iterator method, a
primitive iterator-method result, a non-callable `next`, and a primitive
`next` result. The first two share the `NotIterable` error variant and message,
so five checks map through four error variants. Primitive property lookup in
both inline owners, including the async-disposable path, also boxes through the
current function Realm.

`crates/lila-cli/tests/fixtures/wasm_for_of_protocol_type_errors.js` exercises
all five errors and a valid control through an entry-Realm user function. That
fixture pins observable diagnostics, but it cannot distinguish the current
function Realm from the main Realm. A Realm-distinguishing witness would
require the compiled loop-owning user function itself to be defined in a
created Realm. Wasm AOT intentionally does not dynamically compile `eval`,
`Function`, or a cross-Realm Function constructor, so this runtime case
remains a stated limitation rather than an inferred pass.

This checkpoint does not absorb the two TypeErrors owned by shared
IteratorClose. At the time it did not cover `compile_async_for_of_iterator`,
Array destructuring, ArrayAccumulation, or `Math.sumPrecise`. Section 26
supersedes that synchronous-consumer nonclaim. It does not change user-thrown
accessor or callee completions, assignment or disposal errors. The later
callable-Proxy follow-up makes the ordinary direct owner use general
`IsCallable` and Proxy-aware `Call` for `@@iterator` and cached `next`, matching
the already-correct direct async-disposable and resumable owners. Its bounded
Rust guard forbids Function-tag gates and Function-only calls while pinning
both receivers and propagation-before-validation order. The source-free
fixture covers callable, throwing, non-callable, and revoked Proxy methods,
once-only `next` lookup, and no close on abrupt stepping. Proxy Call
completions propagate unchanged. Entry-Realm revoked proxies do not prove a
cross-Realm Proxy-internal TypeError result. This follow-up adds no operation
row or typed protocol error; the catalog census remains
`29 + 2 + 5 + 10 = 46`.

The focused and affected structure targets pass `37/37`, the exact error
fixture and four success-path CLI controls pass `5/5`, and the four pinned
direct leaves pass all `8/8` Wasm-AOT executions with every failure and
non-success bucket at zero. The invariant details and exact commands are recorded in
[`direct-synchronous-for-of-protocol-error-realm.md`](./direct-synchronous-for-of-protocol-error-realm.md).

The callable-Proxy/body-Realm follow-up passes the five directly affected
structure targets at `23/23`, five exact CLI controls at `5/5`, and eight
unchanged iterator/Proxy leaves at `16/16`, with every failure and non-success
bucket zero. It changes no catalog row, producer count, or shortcut entry.

## 23. Plain-async synchronous `for-of` member-head checkpoint (2026-08-29)

The property-head nonclaim in §20 is superseded for static, computed, and
private member References whose operands do not suspend. This does not add a
catalog row, witness, emission site, backend statement, or second iterator
owner. Lowering already stores IteratorValue in the activation-owned
`$forof.access` binding and lowers the member write as `PropertyWrite` or
`PrivateWrite`. That prefix now remains admitted when the body has one direct
await.

`AsyncFunctionForOfIteratorPlanIr::before_await` owns the prefix. The backend
executes it only in the entry state, inside the IteratorClose frame, after a
successful iterator step and before the body await. The next iteration
re-evaluates public bases and computed keys; async resumption does not repeat
the write. Base, key, setter, and private-brand failures therefore close, and
the existing preserving-Throw close path retains the original error when
`return` also throws. Capture analysis now scans member bases and computed keys
used only in the loop head.

`wasm_plain_async_sync_for_of_member_heads.js` covers static and computed
writes, changing targets and keys, assignment-before-await, no resumed write,
public setter failure, successful private assignment, wrong-brand failure,
IteratorClose counts, and Throw precedence. `cargo fmt --all -- --check` and
the relevant all-target compile pass. The IR `for_of` filter passes `21/21`,
the rejection matrix passes `1/1`, six focused and affected structure targets
pass `25/25`, and the exact member-head and retained capture CLI tests pass
`2/2`. The fixture passes `node --check`.
Patterns, resource heads, `super`, suspending member operands, direct
`break`/`continue`, nonlinear body suspension, captured head TDZ, suspending
iterables, async generators, and `for await` remain nonclaims. No pinned
Test262 leaf combines a member-reference head with this directly awaiting
plain-async body shape. The exact contract is
[`plain-async-synchronous-for-of-member-heads.md`](./plain-async-synchronous-for-of-member-heads.md).

## 24. Plain-async synchronous `for-of` nonlexical-pattern checkpoint (2026-08-29)

The pattern-head nonclaim in §§20 and 23 is superseded for assignment patterns
and `var` binding patterns. This changes neither the 46-row catalog nor its
four iterator rows: `RESUMABLE_SYNC_ITERATOR_PROTOCOL` and
`EmissionSite::ResumableSyncForOfIterator` still own the surrounding
GetIterator, IteratorStep, IteratorValue, and IteratorClose operations.

Lowering stores IteratorValue in the activation-owned `$forof` binding. The
ordinary typed Array/Object destructuring prefix then runs once in
`AsyncFunctionForOfIteratorPlanIr::before_await`, after a successful step,
inside the close frame, and before the body suspension. `var` BoundNames use
the async activation. Assignment References are fully prepared and consumed
by the prefix. Capture analysis now handles top-level object assignment
patterns plus both recursive nesting directions, including computed source
keys, defaults, identifier and public/private member targets, and rest
targets.

`wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js` covers Array and
Object `var` patterns across await, defaults, rest, computed/reference/getter
order, once-only effects, and an abrupt nested Array pattern whose inner and
outer `return` methods both throw. The original pattern Throw survives both
close attempts, and the loop body and await remain unentered.

The relevant all-target compile and formatting check pass. The IR `for_of`
filter and exact rejection witness pass `24/24` and `1/1`; six focused and
affected structure targets pass `25/25`; and the new plus three retained CLI
oracles pass `4/4`. The fixture passes `node --check` and its Node semantic
baseline.

At this checkpoint, `let` and `const` pattern heads remained explicitly
rejected rather than partially entering activation storage. Section 25
supersedes that historical limit with a complete fresh-environment and TDZ
model. Resource patterns, a captured TDZ for the older single-name
declaration, suspending head expressions, nonlinear body suspension, async
generators, and `for await` remain nonclaims. The pinned checkout has no
executable Test262 leaf combining a synchronous pattern head with a directly
awaiting plain-async body, so no Test262 result is attributed. The exact
boundary is
[`plain-async-synchronous-for-of-nonlexical-pattern-heads.md`](./plain-async-synchronous-for-of-nonlexical-pattern-heads.md).

## 25. Plain-async synchronous `for-of` lexical-pattern checkpoint (2026-08-29)

The remaining pattern-head nonclaim in §§20, 23, and 24 is superseded for
`let` and `const` array and object binding patterns. This adds no catalog row,
iterator witness, emission site, or second backend owner. The existing
`RESUMABLE_SYNC_ITERATOR_PROTOCOL` and
`EmissionSite::ResumableSyncForOfIterator` still own GetIterator,
IteratorStep, IteratorValue, and IteratorClose.

The private `AsyncFunctionForOfIteratorHeadIr` is a closed source-head input.
Its lexical-pattern case carries the source mode, an unspellable IteratorValue
entry local, the exact iteration-storage and TDZ-placeholder names, and the
BindingInitialization prefix. `AsyncFunctionForOfIteratorPlanIr` validates
that witness and derives the public closed storage lifetime:
`Activation`, `IterationEnvironment`, or `EntryLocal`. Invalid modes,
duplicate names or slots, incomplete name sets, out-of-range layouts,
assignment targets, and empty patterns claiming an iteration environment are
construction errors.

Capture analysis materializes every lexical BoundName before slot and
capture-hop finalization. Lowering predeclares all final iteration storage as
uninitialized before lowering any default or computed key. After every
successful outer step, the backend creates and publishes the complete fresh
Environment Record, initializes its cells to TDZ, opens the outer close frame,
writes IteratorValue to the entry local, and executes BindingInitialization.
The environment remains active across the body await and is left before
IteratorClose. Empty array and object patterns still perform their semantic
destructuring operations.

`wasm_plain_async_sync_for_of_lexical_pattern_heads.js` covers nested patterns,
defaults and rest, retained before- and after-await closures, uncaptured direct
reads, mutable `let`, forward and captured-head TDZ, post-await `const`
assignment, empty patterns, and inner plus outer preserving-Throw close
precedence. The all-target compile and formatting check pass. The IR `for_of`
filter and exact rejection witness pass `27/27` and `1/1`; the main structure
target and five companions pass `6/6` and `22/22`, for `28/28`; and the exact
lexical oracle plus four retained CLI controls pass `5/5`. The fixture passes
`node --check` and its Node semantic baseline.

Resource patterns, suspension in the iterable or pattern, direct
`break`/`continue`, nonlinear body suspension, async generators, and `for
await` remain nonclaims. The pinned checkout has no executable Test262 leaf
combining a lexical binding-pattern head with this directly awaiting
plain-async synchronous loop, so no Test262 result is attributed. The exact
boundary is
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](./plain-async-synchronous-for-of-lexical-pattern-heads.md).

## 26. Synchronous iterator consumer current-Realm checkpoint (2026-08-29)

The synchronous iterator diagnostic selector is now the private, non-`Copy`
`SyncIteratorConsumer::{ArrayDestructuring, ArrayAccumulation, ForOf,
MathSumPrecise}` domain. `SyncIteratorProtocolError` still has four variants.
Their exhaustive product has 16 diagnostic rows, including separate
destructuring and `array spread` messages. The confirmed source census is 17
typed projector calls and 35 error identifiers: the declaration, typed
projector parameter, 17 producers, and 16 mapping rows. This changes no
spec-operation catalog row or witness, so the catalog remains
`29 + 2 + 5 + 10 = 46`.

Consumer selection no longer chooses a Realm. Shared acquisition boxes every
non-nullish primitive source through the current function Realm. The 16
algorithm-created protocol errors use a separate exhaustive builder
Realm-source projection: standard builtins may read their trusted self-backed
current environment, while main, user, host, and runtime-helper bodies select
the main-Realm TypeError emitter. User-thrown accessor, Proxy,
iterator-method, `next`, `done`, and
`value` completions propagate unchanged.

Array destructuring keeps its custom step owner. It now receives the typed
consumer and produces `NextNotCallable` before the call and
`NextResultNotObject` after call-completion propagation. It then reads `done`
and reads `value` only for a value-bearing element. Array-literal spread uses
the separate `ArrayAccumulation` consumer and the shared no-close step helper.
Its abrupt `next`, `done`, and `value` paths do not call `return`, matching the
2026
[`IteratorStepValue`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-iteratorstepvalue)
and
[`ArrayAccumulation`](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-arrayaccumulation)
control shape.

The destructuring and ArrayAccumulation fixtures pin exact diagnostics,
completion identity, ordering, no-close behavior, and primitive String
prototype lookup. Both run the syntax-owning function in the entry Realm. They
cannot distinguish current-function Realm allocation from the main Realm
fallback. Doing so would require dynamically compiling the syntax-owning user
function in a created Realm, which is outside the Wasm-AOT dynamic-source
contract. No cross-Realm runtime result is inferred.

This checkpoint does not claim that a fresh Array literal or Array-rest result
uses the current function Realm's `%Array.prototype%`. Those allocation-Realm
questions remain separate work. The all-target compile and formatting check
pass. Nine structure targets pass `42/42`, seven exact Wasm-AOT CLI witnesses
pass `7/7`, and nine pinned Array-spread/destructuring leaves pass all `18/18`
sloppy/strict executions with every failure bucket at zero. No semantic golden,
published-status refresh, complete Test262 prefix, or broad workspace suite was
run. The complete invariant and commands are in
[`sync-iterator-consumer-capability.md`](./sync-iterator-consumer-capability.md).
