# Contract: The Property Descriptor lattice — one closed 6.2.6 type and one derived ValidateAndApplyPropertyDescriptor, replacing a raw `u64` bitfield re-derived at eight sites

Area owner: FORMALIZER, theory-first campaign, round 3.
Implements: ECMA-262 6.2.6 (6.2.6.1 IsAccessorDescriptor, 6.2.6.2 IsDataDescriptor,
6.2.6.3 IsGenericDescriptor, 6.2.6.4 FromPropertyDescriptor, 6.2.6.5 ToPropertyDescriptor,
6.2.6.6 CompletePropertyDescriptor), 10.1.6.1 OrdinaryGetOwnProperty,
10.1.6.3 ValidateAndApplyPropertyDescriptor, 10.4.2.1 Array `[[DefineOwnProperty]]`,
10.4.4.2/10.4.4.3 mapped-arguments `[[DefineOwnProperty]]`, 10.5.6 Proxy `[[DefineOwnProperty]]`
(read-only, for the shape claim in §1.9).
Short-name pointer (to be written by the ENCODER after landing):
`docs/rust-rewrite/contracts/property-descriptor-lattice.md`.

---

## 0. How to read this, and what is measured

This document is the encoder's specification and the dry-runner's oracle. Every
line number is from `git rev-parse --short HEAD` = `5bb66a35a` on branch
`claude/test-driven-rust-opus-pp6giw`, working tree clean
(`git status --porcelain` empty). Every cited region was **opened and read**;
none was inferred from a grep count.

Three conventions, following the house style of
`Reference Records: one record, a carried [[Strict]], and a write that consumes it.md`
and `Numeric conversion codomains ....md`:

- **Invariants** are numbered `I1..I15`. §2 assigns each one either a Rust
  construct or a ledger row. There is no third option.
- **Ledger rows** are numbered `LN1..LN8`. A ledger row is a place where a test
  or a run-time check remains load-bearing, **with the reason a type cannot
  carry it**. A row without a reason is a defect in this document.
- **Measured** means counted from the tree at `5bb66a35a`. **Derived** means
  computed from the spec text or from the bit layout. **Estimated** appears
  nowhere in this document.

§5 lists **nine** places where this contract departs from the area brief, each
with the evidence. **Read §5 before implementing anything.** Four of the nine
would otherwise produce a wrong retrofit, a decoration type, or — in one case
(§5.2) — the deletion of a required 10.1.6.3 step-4 check.

### 0.1 What was measured

| Fact | Value | How |
|---|---|---|
| Lines in `objects.rs` | **20,865** | `wc -l` |
| Positional parameters of `emit_object_define_entry` | **16** (`objects.rs:13533-13551`), of which 15 carry descriptor data and 1 is `function` | read in full |
| Call sites of `emit_object_define_entry` | **4** | `objects.rs:13422`, `objects.rs:13511`, `standard.rs:11572`, `standard.rs:11905` |
| Call sites of `object_data_descriptor_kind` | **3** (`objects.rs:1250`, `objects.rs:1286`, **`functions.rs:3017`**) | `rg`; the brief's "4" counts the definition line |
| Call sites of `object_accessor_descriptor_kind` | **1** (`objects.rs:1403`) | `rg`; the brief's "2" counts the definition line |
| Call sites of `emit_validate_array_named_descriptor` | **2**, **both in `builtins/array.rs`** (`:4141`, `:4544`) | `rg` |
| Call sites of `emit_object_define_accessor` | **11**, **none in `objects.rs`** — `host.rs` ×9, `functions.rs` ×2 | `rg`, each site read |
| Call sites of `emit_object_define_enumerable_accessor` | **4**, all in `objects.rs` (`:2585`, `:2611`, `:2643`, `:2664`) | `rg`, each site read |
| Call sites of `emit_object_define_accessor_with_flag_local` | **3**, all in `objects.rs` (`:1418`, `:13456`, `:13484`) | `rg` |
| Accessor call sites passing **both** `getter: None` and `setter: None` | **0 of 15** | each of the 15 read; §5.1 |
| `DESCRIPTOR_KIND_OFFSET` references | **176 occurrences across 11 files**, incl. `builtins/intl_datetimeformat.rs` ×2 (batch-2 hold list) | `rg -o \| wc -l`; per-file `rg -c` |
| `OBJECT_DESCRIPTOR_*` references | **304 lines / 309 occurrences across 8 files**; `objects.rs` 121, `standard.rs` 76, `array.rs` 76 | `rg -c` (lines) and `rg -o` (occurrences) |
| Distinct `*_DESCRIPTOR_KIND_OFFSET` heap slots sharing this one word format | **9** | `heap.rs:292, 295, 653, 662, 666, 669, 676, 679, 696` |
| Descriptor-kind **write** sites (`store_*_at_offset(.., *_DESCRIPTOR_KIND_OFFSET, ..)`) | `objects.rs` 99, `array.rs` 48, `standard.rs` 21, `json.rs` 3, `control_flow.rs` 3, `heap.rs` 3 | `rg -B2` |
| Array/arguments `[[DefineOwnProperty]]` derivation sites the note routes | **8**, not six | `standard.rs:2454, 2741, 3007, 3432`; `array.rs:3709, 3821, 4062, 4463` |
| `IntrinsicPropertyAttributes` references outside `porffor-runtime` | **1**, a `pub use` re-export at `porffor-engine/src/lib.rs:984`. **No backend reads it.** | `rg` |
| `INTRINSIC_PROPERTY_DESCRIPTORS` rows | **46** | `porffor-runtime/src/lib.rs:485` |
| `porffor-runtime/Cargo.toml` `[dependencies]` | **absent entirely** | `cat` |
| `porffor-aot-wasm` → `porffor-ir` dependency | **present** (`porffor-aot-wasm/Cargo.toml`) | `cat` |
| test262 corpus files in §6 that exist at the current pin | **11 of 11** | checked by path |
| Sibling theory-first modules already landed in `porffor-ir` | **2** (`numeric_conversions.rs`, `reference.rs`), both declared `pub mod` **inside `ir.rs`** (`:34`, `:23`), not in `lib.rs` | `rg` |

### 0.2 The one-sentence claim

6.2.6 is a **partial record over six independently present-or-absent fields**
plus a **three-way partition** of that record, and every defect in this area is
either a *partition* error (a record classified into the wrong one of the three
cases, or into a case the code has no arm for) or a *presence* error (an absent
field materialised as a value, or a present field's presence encoded twice in
two independent `Option`s that can disagree) — so the two types this contract
adds are a closed partition enum and a closed presence enum, and everything else
is derived from them.

### 0.3 What this contract does **not** claim

It does not claim that any of M1–M7 is a currently-reachable wrong answer. §1.10
gives the reachability status of each, measured. Three of the seven
(M1, M2, M3, M4) are **already fixed by hand** in commit `fae75423a` and the
work here is to make the fix structural rather than commentary; M5 and M6 are
**representable but currently unreachable**, each blocked by a coincidence in a
different function; M7 is representable and its blast radius is invisible
(§1.8). A contract that overstated this would send the dry-runner hunting for
failures that are not there.

---

## 1. Spec basis

### 1.1 6.2.6 is a partial record, and "absent" is a first-class state

ECMA-262 6.2.6 defines the Property Descriptor specification type as a Record
with six possible fields — `[[Value]]`, `[[Writable]]`, `[[Get]]`, `[[Set]]`,
`[[Enumerable]]`, `[[Configurable]]` — and says explicitly that **any field may
be absent**. Table 3 lists the fields; the surrounding prose is what makes the
record *partial* rather than a six-tuple with defaults.

Three consequences the implementation must respect, and each is violated
somewhere in the tree today:

1. **Absent ≠ false.** "`[[Writable]]` is absent" and "`[[Writable]]` is
   present and is `false`" are different records with different behaviour under
   10.1.6.3 (step 5 leaves the existing attribute alone; a present `false`
   overwrites it). This is exactly the M2 defect as shipped.
2. **Absent ≠ undefined.** `{get: undefined}` **has** a `[[Get]]` field whose
   value is `undefined`; `{}` does not. 6.2.6.5 step 5 says "if
   *hasGet* is true … set desc.[[Get]] to *getter*", and *hasGet* comes from
   HasProperty, not from the retrieved value. The tree's
   `emit_object_append_accessor_property_with_flags`
   (`objects.rs:1328-1359`) materialises `ValueKind::Undefined` locals for a
   missing getter and then rebinds `getter` to `Some(..)` — which is *correct*
   for its callers (they are building complete accessor properties, where
   CompletePropertyDescriptor's default for `[[Get]]` genuinely is `undefined`),
   but it means the `Option` at that boundary has changed meaning from
   "the field is absent" to "the operand locals are absent". Two meanings, one
   type. §5.1.
3. **Presence is a property of the record, not of the emitter.** A field's
   presence may be known when the compiler runs (`Object.defineProperty(o,'x',{value:1})`
   lowered from a literal) or only when the program runs (a descriptor object
   the program computed). Both are "presence"; they are not two different
   questions. The tree spells them as two independent parameters that can
   disagree — `data: Option<(u32,u32)>` and `data_present_local: Option<u32>`
   at `objects.rs:13538` and `:13544`. §1.6.

### 1.2 The three-way partition (6.2.6.1 / 6.2.6.2 / 6.2.6.3), and its precondition

- **6.2.6.1 IsAccessorDescriptor(Desc)**: `false` if *Desc* is `undefined`;
  `false` if *Desc* has neither `[[Get]]` nor `[[Set]]`; otherwise `true`.
- **6.2.6.2 IsDataDescriptor(Desc)**: `false` if *Desc* is `undefined`; `false`
  if *Desc* has neither `[[Value]]` nor `[[Writable]]`; otherwise `true`.
- **6.2.6.3 IsGenericDescriptor(Desc)**: `false` if *Desc* is `undefined`;
  `true` if IsAccessorDescriptor(*Desc*) and IsDataDescriptor(*Desc*) are
  **both** `false`; otherwise `false`.

Read literally, these three are **not** a partition of all Property
Descriptors: a record carrying both `[[Value]]` and `[[Get]]` satisfies
6.2.6.1 and 6.2.6.2 simultaneously and 6.2.6.3 not at all. Such a record is
excluded upstream, by **6.2.6.5 ToPropertyDescriptor step 9**:

> If *desc* has a `[[Get]]` field or *desc* has a `[[Set]]` field, then
> if *desc* has a `[[Value]]` field or *desc* has a `[[Writable]]` field,
> throw a **TypeError** exception.

**Therefore the partition is a theorem about *validated* descriptors, not about
raw ones**, and the type mapping must say so: the three-case enum is the
codomain of a classification function whose domain is a *validated* descriptor
newtype, not a raw one. A type that made `{value, get}` unrepresentable would be
a spec error — 10.5.6 Proxy `[[DefineOwnProperty]]` and 10.4.4.x mapped
arguments both hand around descriptors that came from ToPropertyDescriptor, but
`Object.getOwnPropertyDescriptors` + `Object.defineProperties` round-trips make
the raw shape observable, and the TypeError is the observable behaviour.

The tree already emits this check, as Wasm, at `standard.rs:11363-11380`:
the accessor branch is entered when `getter_present != 0 || setter_present != 0`
(`:11356-11363`), and immediately throws if `value_present != 0 ||
writable_present != 0` (`:11364-11380`). That is 6.2.6.5 step 9 exactly. It is
also the reason the `data: None` + `data_present_local: Some(..)` pair at
`standard.rs:11572` does not currently produce a wrong answer — see §5.2, which
is the most important section of this document.

**Latitude, and this contract's choice.** The spec does not say where the
partition is computed. This contract fixes it: **exactly one function**,
`classify`, computes it, and it is a `const fn` over presence states with no
access to values. Nothing else in the workspace may re-derive it. Measured, the
tree derives it in **nine** distinct spellings today (§1.4).

### 1.3 6.2.6.6 CompletePropertyDescriptor, and why `Generic` cannot be a stored kind

6.2.6.6 takes a Property Descriptor and fills the gaps:

1. Assert *Desc* is a Property Descriptor.
2. Let *like* be Record { `[[Value]]`: undefined, `[[Writable]]`: false,
   `[[Get]]`: undefined, `[[Set]]`: undefined, `[[Enumerable]]`: false,
   `[[Configurable]]`: false }.
3. **If IsGenericDescriptor(*Desc*) is true, or IsDataDescriptor(*Desc*) is
   true**, then set the absent `[[Value]]` and `[[Writable]]` fields from
   *like*.
4. Else set the absent `[[Get]]` and `[[Set]]` fields from *like*.
5. Set the absent `[[Enumerable]]` and `[[Configurable]]` fields from *like*.

Step 3 is the spec's own theorem that **a generic descriptor completes to a
data descriptor**. It is why `Object.defineProperty(obj,"property",{})` on a
fresh key yields `{value: undefined, writable: false, enumerable: false,
configurable: false}` (test262 `15.2.3.6-4-52.js`, read in full in §6.1), and it
is why:

> `Generic` is a legal *classification* of an incoming partial descriptor and is
> **never** a legal *stored* kind.

That asymmetry is load-bearing for the type mapping. It gives two different
enums, not one:

- `PropertyDescriptorKind { Data, Accessor, Generic }` — the codomain of
  `classify`, three variants, matched exhaustively with no `_` arm.
- `CompleteDescriptor { Data { .. }, Accessor { .. } }` — what a property *is*,
  two variants, and **`Accessor` has no `writable` field at all**.

Collapsing these into one enum is the single most tempting simplification here
and it is wrong in both directions: it would either give `Generic` a stored
representation (which no heap word has) or force `classify` to lie about a
generic input.

**Latitude, and this contract's choice.** 6.2.6.6 says the defaults apply when
completing a descriptor; it does not say *when* an implementation must complete
one. 10.1.6.3 step 2 (current is `undefined`) is the only place completion is
observable on the ordinary-object path. This contract therefore requires
completion to happen **exactly at the create-a-new-entry path** and forbids it
anywhere else — concretely, at `objects.rs:14241-14356`, and not in the
existing-entry branch at `objects.rs:13669-14196`. Applying defaults on the
existing-entry path is precisely mistake class **M2**.

### 1.4 The nine spellings of the partition in this tree, measured

| # | Site | Spelling | Cases it can express |
|---|---|---|---|
| 1 | `objects.rs:45` `object_data_descriptor_kind(writable, enumerable, configurable) -> u64` | function name encodes `Data` | Data only |
| 2 | `objects.rs:63` `object_accessor_descriptor_kind(enumerable, configurable) -> u64` | function name encodes `Accessor` | Accessor only |
| 3 | `objects.rs:1450` `requested_data_descriptor: bool`, consumed at `:1532-1533` as `I32Const(i32::from(..))` compared `I32Eq` against "existing is accessor" | `bool` | Data, Accessor. **No `Generic`.** |
| 4 | `objects.rs:1504-1526` `kind_present_local` — a *runtime* OR-fold over the four presence locals, used to skip the kind check when the incoming descriptor is generic | i64 local | Generic vs not, at run time |
| 5 | `objects.rs:13575-13579` `let descriptor_kind = if has_data { DATA } else { ACCESSOR }` | `bool` (`data.is_some()`) | Data, Accessor. **No `Generic`.** |
| 6 | `objects.rs:14036-14064` the four-way `is_some()` conjunction that ORs the accessor bit back in when all four value-side presence flags are runtime-false | four `Option`s + runtime conjunction | a *correction*, only when all four locals are `Some` |
| 7 | `standard.rs:3441` `accessor: bool` (`fn emit_store_arguments_length_descriptor_kind`, `:3432`) | `bool` | Data, Accessor. **No `Generic`.** |
| 8 | `lowering.rs:25611` `generic_property_descriptor_shape()` — one object shape naming **all six keys** | shape literal | asserts Data ∧ Accessor simultaneously |
| 9 | `lowering.rs:26130-26163` `match property { ObjectShapeProperty::Data(..) => .., ObjectShapeProperty::Accessor{..} => .. }` | exhaustive 2-arm match | Data, Accessor — **and this one is correct**, see §5.6 |

Spelling 9 is the shape the whole contract is trying to generalise: it is an
exhaustive match over a closed two-case enum, and it produces the right key sets
for both cases without a single `&'static str` list assembled by hand. It is
also, measured, the only one of the nine that does.

A tenth, in a different crate: `porffor-runtime/src/lib.rs:159-164`
`IntrinsicPropertyAttributes { writable, enumerable, configurable }` — three
bare `bool`s, no accessor case, no presence notion. §5.8 gives its measured
reachability and this contract's declared decision.

### 1.5 10.1.6.3 ValidateAndApplyPropertyDescriptor, restated as the case tree the code must have

Arguments: *O* (an Object or `undefined`), *P*, *extensible*, *Desc*,
*current*. Returns a Boolean.

- **Step 2 — `current` is `undefined`.** If *extensible* is `false`, return
  `false`. If *O* is `undefined`, return `true`. Then:
  - 2.d.i If IsAccessorDescriptor(*Desc*) is true — create an accessor property
    with `[[Get]]`, `[[Set]]`, `[[Enumerable]]`, `[[Configurable]]` from *Desc*,
    **each absent field taking its 6.2.6.6 default**.
  - 2.d.ii Else — create a data property with `[[Value]]`, `[[Writable]]`,
    `[[Enumerable]]`, `[[Configurable]]`, same rule.
  - The `Else` is what makes a **generic** descriptor a data property. This is
    the single sharpest trace in the corpus (§6.1).
  - Return `true`.
- **Step 3 — assert `current` is a fully populated Property Descriptor.** This
  is the spec asserting exactly what §1.3 calls `CompleteDescriptor`.
- **Step 4 — `current.[[Configurable]]` is `false`.**
  - 4.a If *Desc* has `[[Configurable]]` and it is `true`, return `false`.
  - 4.b If *Desc* has `[[Enumerable]]` and it differs from
    `current.[[Enumerable]]`, return `false`.
  - 4.c If IsGenericDescriptor(*Desc*) is **false** and
    IsAccessorDescriptor(*Desc*) ≠ IsAccessorDescriptor(*current*), return
    `false`.
  - 4.d If IsAccessorDescriptor(*current*) is `true`:
    - if *Desc* has `[[Get]]` and `SameValue(Desc.[[Get]], current.[[Get]])` is
      `false`, return `false`;
    - same for `[[Set]]`.
  - 4.e Else if `current.[[Writable]]` is `false`:
    - if *Desc* has `[[Writable]]` and it is `true`, return `false`;
    - if *Desc* has `[[Value]]` and **`SameValue(Desc.[[Value]],
      current.[[Value]])` is `false`**, return `false`.

  (Edition numbering note: older editions and the area brief spell 4.e's second
  bullet as "step 4.a.ii". Both name the same clause — the `SameValue`
  comparison of `[[Value]]` on a non-configurable, non-writable data property.
  This document uses the current 4.e numbering and flags the equivalence here so
  the dry-runner is not hunting for a nonexistent 4.a.ii.)
- **Step 5 — IsGenericDescriptor(*Desc*) is `true`**: **do nothing** beyond the
  step-6/7 application below; no field of the existing property that *Desc* does
  not name may change. This is the clause M2 broke.
- **Step 6 — IsDataDescriptor(*current*) ≠ IsAccessorDescriptor(*Desc*)
  mismatch, data → accessor.** If IsDataDescriptor(*current*) is `true` and
  IsAccessorDescriptor(*Desc*) is `true`:
  - 6.a if `current.[[Configurable]]` is `false`, return `false`;
  - 6.b **convert the property to an accessor property, preserving ONLY
    `[[Enumerable]]` and `[[Configurable]]`, and setting the rest to their
    6.2.6.6 defaults.** `[[Writable]]` is not preserved — it ceases to exist.
- **Step 7 — the mirror, accessor → data.** Preserve only `[[Enumerable]]` and
  `[[Configurable]]`; `[[Writable]]` takes its default, **`false`**, and
  **not** whatever bit the accessor entry happened to be carrying. This is the
  clause M1 broke.
- **Step 8 — else, for each field of *Desc*, set the corresponding attribute of
  the property.** Fields *Desc* does not have are not touched.
- **Step 9 — return `true`.**

Two spec facts here are the whole reason for the `CompleteDescriptor` /
`PropertyDescriptorKind` split:

- Steps 6.b and 7 say the surviving attribute set is determined **entirely by
  the target kind**. An `Accessor` that can carry a `[[Writable]]` bit is not a
  representation of anything the spec defines.
- Step 3 asserts `current` is *fully populated*. There is no such thing as a
  stored generic property.

### 1.6 The presence encoding in `emit_object_define_entry`, read in full

`objects.rs:13533-13551`, verbatim parameter list (16 parameters):

```
object_local: u32,                       key_local: u32,
object_tag_local: Option<u32>,           data: Option<(u32, u32)>,
getter: Option<(u32, u32)>,              setter: Option<(u32, u32)>,
writable_payload_local: u32,             enumerable_payload_local: u32,
configurable_payload_local: u32,
data_present_local: Option<u32>,         getter_present_local: Option<u32>,
setter_present_local: Option<u32>,       writable_present_local: Option<u32>,
enumerable_present_local: Option<u32>,   configurable_present_local: Option<u32>,
function: &mut Function,
```

The three value-carrying fields are `Option<(payload, tag)>`; the three boolean
fields are bare `u32` payload locals. Independently, all six carry an
`Option<u32>` presence local. That is **two independent encodings of the same
question** for `[[Value]]`, `[[Get]]` and `[[Set]]`, and **one encoding plus a
hole** for `[[Writable]]`, `[[Enumerable]]`, `[[Configurable]]` (a bare `u32`
cannot say "absent" at all).

The four call sites, measured, each read in full:

| Site | `data` | `getter` | `setter` | six presence locals |
|---|---|---|---|---|
| `objects.rs:13422` (`emit_object_define_data_with_flag_locals`) | `Some` | `None` | `None` | all six `None` |
| `objects.rs:13511` (`emit_object_define_accessor_with_flag_local`) | `None` | forwarded `Option` | forwarded `Option` | all six `None` |
| `standard.rs:11572` (`Object.defineProperty`, accessor branch) | **`None`** | `Some` | `Some` | **all six `Some`** |
| `standard.rs:11905` (`Object.defineProperty`, data branch) | `Some` | `None` | `None` | all six `Some` |

Row 3 is the contradictory pair the brief names: static classification says
`Accessor` (`data.is_none()`), runtime classification says maybe-data
(`data_present_local = Some(value_present_local)`). §5.2 shows what that
actually does, which is **not** what the brief says, and why the naive fix
deletes a required check.

The presence locals are consumed at seven places inside the body, each guarded
by `if let Some(..)` or by an `is_some()` conjunction:

| Body site | Guard | Spec clause it implements |
|---|---|---|
| `:13683-13701` | `configurable_present_local` | step 4.a |
| `:13702-13725` | `enumerable_present_local` | step 4.b |
| `:13726-13777` | **all four of** `data`/`writable`/`getter`/`setter` present | steps 6.a and 7.a (kind change on a non-configurable property) |
| `:13778-13801` | `writable_present_local` | step 4.e first bullet |
| `:13802-13853` | `data_present_local` | step 4.e second bullet (`SameValue`) |
| `:13854-13896` / `:13897-13939` | `getter_present_local` / `setter_present_local` | step 4.d |
| `:13941-13963`, `:13973-13997`, `:13998-14014`, `:14015-14035`, `:14036-14065` | five separate presence guards | steps 5, 6.b, 7, 8 — the "leave it alone" clauses |

**Every one of these is an `if let` that silently emits nothing when the
`Option` is `None`.** There is no arm for "the caller does not know", because
"the caller does not know" and "the field is absent" are the same value.
That is mistake class M5's mechanism, and §5.2 shows that the four-way
conjunction at `:13726` makes it worse than the brief states: it couples two
*independent, required* obligations to a single four-way `is_some()` test.

### 1.7 The stored word, measured — three axes, not one

`heap.rs:948-955`:

```
OBJECT_DESCRIPTOR_ACCESSOR      = 1     bit 0
OBJECT_DESCRIPTOR_CONFIGURABLE  = 2     bit 1
OBJECT_DESCRIPTOR_WRITABLE      = 4     bit 2
OBJECT_DESCRIPTOR_ENUMERABLE    = 8     bit 3
OBJECT_DESCRIPTOR_DATA          = 0     the ABSENCE of bit 0
ARRAY_DESCRIPTOR_OWN_PROPERTY   = 16    bit 4
ARGUMENTS_DESCRIPTOR_MAPPED     = 32    bit 5
```

and `heap.rs:956-957`:

```
ARRAY_DESCRIPTOR_NORMAL_DATA = CONFIGURABLE | WRITABLE | ENUMERABLE   (= 14)
```

**A third axis the area brief does not mention, and which is decisive for the
encoding type.** `functions.rs:6404`:

```rust
ARGUMENTS_DESCRIPTOR_MAPPED as i64 | ((mapped_slot as i64) << 32),
```

and the reader, `functions.rs:6980-6988` (and again at `:7065-7073`):

```rust
LocalGet(descriptor_kind_local);  I64Const(32);  I64ShrU;  LocalSet(mapped_slot_local);
```

So the full layout of the word stored at every `*_DESCRIPTOR_KIND_OFFSET` is:

| Bits | Meaning | Axis |
|---|---|---|
| 0 | `[[Get]]`/`[[Set]]` kind: 1 = Accessor, 0 = Data | descriptor kind |
| 1 | `[[Configurable]]` | attribute |
| 2 | `[[Writable]]` | attribute |
| 3 | `[[Enumerable]]` | attribute |
| 4 | array exotic: this index has an own property record | exotic flag |
| 5 | mapped arguments: this index is mapped | exotic flag |
| 6..31 | unused | — |
| 32..63 | mapped-arguments environment slot index (`u32`) | exotic payload |

**Nine heap slots share this format** (`heap.rs:292, 295, 653, 662, 666, 669,
676, 679, 696`), and 176 references across 11 files read or write them.

Two facts follow, and the encoder must respect both:

1. **`ACCESSOR | WRITABLE` (= 5) is a representable, meaningless *value*** — an
   accessor property carrying a stale writable bit. That is M1. It is
   representable *because* `DATA` is the absence of bit 0, so nothing forces a
   choice.
2. **`ACCESSOR | WRITABLE` is also a perfectly legitimate *mask*.**
   `objects.rs:1560-1565` tests `existing & (ACCESSOR | WRITABLE) == 0`, i.e.
   "the existing property is a data property and is not writable" — one `I64And`
   instead of two. A type that made the bit pattern `5` unconstructible would
   break this **correct** code.

   Therefore the type mapping needs **two distinct newtypes** — a
   `DescriptorWord` (a value; constructors are exactly the two the spec
   licenses) and a `DescriptorMask` (a test; composites allowed) — and they must
   not be interconvertible. This is the sharpest single design constraint in the
   area and the brief does not contain it. §5.4.

### 1.8 6.2.6.4 FromPropertyDescriptor — the codomain is *not* "exactly four keys"

6.2.6.4, read literally:

1. If *Desc* is `undefined`, return `undefined`.
2. Let *obj* be OrdinaryObjectCreate(%Object.prototype%).
3. Assert: *obj* is an extensible ordinary object with no own properties.
4. **If *Desc* has a `[[Value]]` field**, CreateDataPropertyOrThrow(*obj*,
   `"value"`, *Desc*.`[[Value]]`).
5. **If *Desc* has a `[[Writable]]` field**, … `"writable"` …
6. **If *Desc* has a `[[Get]]` field**, … `"get"` …
7. **If *Desc* has a `[[Set]]` field**, … `"set"` …
8. **If *Desc* has an `[[Enumerable]]` field**, … `"enumerable"` …
9. **If *Desc* has a `[[Configurable]]` field**, … `"configurable"` …
10. Return *obj*.

Every step is conditional on presence. So the codomain is:

> **any subset** of `{value, writable, enumerable, configurable}`, **or any
> subset** of `{get, set, enumerable, configurable}` — never a set drawn from
> both halves, because *Desc* reached 6.2.6.4 through 6.2.6.5 step 9.

The area brief's "codomain is exactly `{value,writable,enumerable,configurable}`
OR `{get,set,enumerable,configurable}` — never six keys" is right about the
partition and **wrong about the cardinality**, and the difference decides the
type. The four-key form is the codomain only when *Desc* is *complete*, which is
true at exactly one caller:

- **10.1.8.1 `Object.getOwnPropertyDescriptor`** → 10.1.6.1
  OrdinaryGetOwnProperty, whose step 3 asserts a fully populated descriptor.
  Here, and only here, the answer has exactly four keys. This is what
  `15.2.3.3-4-1.js` and `15.2.3.3-4-239.js` pin (§6.8, §6.9).

and false at the other two callers in this tree's analysis:

- **10.5.6 Proxy `[[DefineOwnProperty]]` step 7**, `descObj` =
  FromPropertyDescriptor(*Desc*) where *Desc* is the caller's **partial**
  descriptor. `Object.defineProperty(proxy, 'x', {value: 1})` gives the trap a
  `descObj` with **one** key.
- **10.5.5 `[[GetOwnProperty]]`** likewise round-trips a partial descriptor when
  the trap result is re-normalised.

Consequently the type is not "an enum of two key sets". It is:

- a closed six-name field domain (`DescriptorField`), and
- two *derivations* from it: `PartialDescriptor::present_fields()` (any subset,
  partition-respecting by construction) and `CompleteDescriptor::keys() ->
  [DescriptorField; 4]` (the two four-key sets, and nothing else).

This distinction is what makes the retrofit of `lowering.rs:27816` and
`:28453` correct rather than merely different: those two sites are the **Proxy
trap argument**, so replacing the six-key shape with a four-key shape would swap
one false assertion for another. The right replacement there is
`heap_shape: None`. §4.5, §5.7.

### 1.9 Where `namespace.rs` actually sits — ToPropertyDescriptor's *domain*, not FromPropertyDescriptor's codomain

`crates/porffor-ir/src/modules/namespace.rs` builds module-namespace exotic
objects by concatenating JavaScript source text. Measured, there are **three**
descriptor builders in product code, not four:

- `:348-368` — the per-export accessor, rendered as
  `Object.defineProperty(<ns>, <name>, { get: () => <ref>, enumerable: true, configurable: false });`
  — **three keys**.
- `:371-378` — `@@toStringTag`, rendered as
  `{ value: "Module", writable: false, enumerable: false, configurable: false }`
  — **four keys**.
- `:742-749` — the module-source object's `@@toStringTag`, byte-identical shape
  to the previous.

The brief's fourth item, `:1109-:1126`, is a `#[test]`
(`namespace_source_creates_a_null_prototype_non_extensible_object`), not a
builder. §5.5.

These three are **arguments to `Object.defineProperty`**, i.e. inhabitants of
6.2.6.5's *domain* — arbitrary partial descriptors. They are not
FromPropertyDescriptor outputs. The `:348-368` builder proves it: a legal
three-key partial descriptor that no FromPropertyDescriptor-codomain type would
admit.

This matters for the deliverable. The key domain that `namespace.rs` consumes is
the **six-name field domain shared by both directions**; what it does *not*
consume is a four-key codomain type. Building the codomain type and then
"consuming" it here would be a type bent to fit a consumer — which is the
decoration AGENTS.md forbids. The correct owned consumer is a *partial*
descriptor rendered to source text, and §2.13 defines it that way.

### 1.10 Reachability of each mistake class, measured

| Class | Brief's status | Measured status at `5bb66a35a` | Evidence |
|---|---|---|---|
| M1 accessor carrying stale `[[Writable]]` | shipped, fixed by hand | **Fixed**, by a 21-instruction hand-written sequence at `objects.rs:13973-13997` guarded by a 5-line comment at `:13965-13972`. The word `ACCESSOR\|WRITABLE` remains representable. | read in full |
| M2 absent field materialised as explicit `false` | shipped, fixed by hand | **Fixed**, by the three "if absent, re-read the existing bit" blocks at `objects.rs:13998-14014`, `:14015-14035`, and the data re-read at `:13941-13963`. Each is an `if let Some(..)` that disarms silently. | read in full |
| M3 `F64Eq` instead of `SameValue` | shipped, fixed by hand | **Fixed** at both descriptor sites: `objects.rs:13834` and `objects.rs:1583`, both calling `emit_tagged_payload_same_value_i32`. That helper has **28 call sites workspace-wide**; only these two are 10.1.6.3 step 4.e. | `rg` + read |
| M4 ToPropertyDescriptor reading own slots | shipped, fixed by hand | **Fixed**. `objects.rs:11962-12014` loops the six fields in 6.2.6.5 order (enumerable, configurable, value, writable, get, set) calling `emit_object_has_property_with_key_tag_i32`, i.e. HasProperty, then Get. The order lives in a plain array literal; the obligation survives as the comment at `:11960-11961`. | read in full |
| M5 generic classified as accessor | "LATENT AND LIVE" | **LATENT, NOT REACHED.** All 15 accessor call sites pass at least one of `getter`/`setter` as `Some`, and `emit_object_append_accessor_property_with_flags` (`objects.rs:1328-1359`) rebinds both to `Some` before the only forwarding site. §5.1. | each site read |
| M6 six-key descriptor shape | "LIVE, CONSTRUCTED" | **Constructed at 3 sites, currently harmless.** The only fold that could act on it — the disjoint-singleton constant-fold at `operations.rs:10779-10787` — is disarmed for property reads by `expr_result_tag_is_runtime_dynamic` (`planning.rs:6178` lists `ExprIr::PropertyRead`). §5.7. |  traced end to end |
| M7 misspelt descriptor key | representable | **Representable; blast radius invisible.** A stray key in a `defineProperty` argument object is *ignored* by 6.2.6.5, so `configurabel: false` yields a non-configurable property anyway — the correct answer, by luck, with no diagnostic. In `property_descriptor_shape` a misspelt key yields a shape whose named property never exists at run time; per §5.7 that is currently also inert. | derived from 6.2.6.5 |

The honest summary: **this area has no currently-failing test to point at.** Its
value is that four defects were shipped here, all four were fixed by hand, and
all four fixes are held in place by comments and by `if let Some(..)` guards
that disarm without saying so. That is exactly the situation AGENTS.md's "Code
Invariants Before Test Invariants" section is written for, and it is the reason
the acceptance criteria in §7 are *structural* (rung 0 + an empty rung-G diff)
rather than a conformance delta.

---

## 2. Type mapping

### 2.0 Invariant index

| # | Invariant | Construct | Kills | Lives in |
|---|---|---|---|---|
| I1 | The 6.2.6.1–3 partition is closed and has three cases | `enum PropertyDescriptorKind { Data, Accessor, Generic }`, exhaustive matches, **no `_` arm anywhere in the workspace** | M5 | `porffor-ir` |
| I2 | A stored property is `Data` or `Accessor`, and `Accessor` has no `[[Writable]]` | `enum CompleteDescriptor<C> { Data{..}, Accessor{..} }` — `Accessor` has no `writable` field | M1 | `porffor-ir` |
| I3 | Presence is one closed 3-state question per field | `enum Presence<T, R> { Absent, Present(T), Runtime { present: R, value: T } }` | M2, M5 | `porffor-ir` |
| I4 | A partial descriptor is one value, not 15 positional parameters | `struct PartialDescriptor<C>` with six `Presence` fields | M2 | `porffor-ir` |
| I5 | The partition is a theorem about *validated* descriptors | `struct ValidatedDescriptor<C>(PartialDescriptor<C>)`, static constructor validates 6.2.6.5 step 9 | — | `porffor-ir` |
| I6 | `classify` is the only derivation of the partition | `fn classify(&ValidatedDescriptor<C>) -> DescriptorClassification` | M5 | `porffor-ir` |
| I7 | The stored word is a value with exactly two constructors | `struct DescriptorWord(u64)`; `of_data(writable, enumerable, configurable)`, `of_accessor(enumerable, configurable)` | M1 | `porffor-aot-wasm/heap.rs` |
| I8 | A mask is not a value | `struct DescriptorMask(u64)`, composites allowed, no conversion to `DescriptorWord` | (protects §1.7 fact 2) | `porffor-aot-wasm/heap.rs` |
| I9 | The exotic flags are an orthogonal axis with a payload | `struct DescriptorFlags`, `struct MappedSlot(u32)`, disjointness `const _: () = assert!(..)` | — | `porffor-aot-wasm/heap.rs` |
| I10 | The six legacy constants are derivations, and the wire format is pinned | `pub(crate) const OBJECT_DESCRIPTOR_* : u64 = <derivation>;` + `const _: () = assert!(.. == <literal>);` | — | `porffor-aot-wasm/heap.rs` |
| I11 | A runtime-built word cannot acquire a writable bit on an accessor path | `struct DescriptorWordEmitter<K: DescriptorKindMarker>`; `set_writable_if` exists **only** on `K = Data` and `K = Dynamic` | M1 (runtime half) | `porffor-aot-wasm/objects.rs` |
| I12 | The six field names are a closed domain | `enum DescriptorField { Value, Writable, Get, Set, Enumerable, Configurable }` with `const fn key(self) -> &'static str` | M7 | `porffor-ir` |
| I13 | FromPropertyDescriptor's two codomain shapes | `CompleteDescriptor::keys() -> [DescriptorField; 4]`, and `PartialDescriptor::present_fields()` for the partial case | M6 | `porffor-ir` |
| I14 | Descriptor source text is built from the field domain, not from string literals | `struct DescriptorSourceText` over `PartialDescriptor<SourceText>` where `SourceText::RuntimeFlag = core::convert::Infallible` | M7 | `porffor-ir` (consumed in `modules/namespace.rs`) |
| I15 | The 6.2.6.5 field-read order is a table, not a hand-written literal | `const TO_PROPERTY_DESCRIPTOR_ORDER: [DescriptorField; 6]` + `const _: () = assert!(..)` that every variant appears exactly once | partial M4 | `porffor-ir`; **consumer note-routed** |

Ledger:

| # | Row | Reason a type cannot carry it |
|---|---|---|
| LN1 | M3: which comparison a *body* emits (`SameValue` vs `F64Eq`) | The obligation is a property of the emitted Wasm instruction sequence. `emit_tagged_payload_same_value_i32` is a correctly *named* helper called at 28 sites; nothing in Rust distinguishes calling it from calling `emit_tagged_payload_equality_i32`, which is also correct at 3 of those sites (10.1.6.3 step 4.d compares `[[Get]]`/`[[Set]]`, which are objects, where the two agree). A type could only help by making the *operand* carry "this is a `[[Value]]`", and the operand is a pair of `u32` Wasm local indices. |
| LN2 | M4: HasProperty-then-Get, six times, in 6.2.6.5 order | I15 makes *omission* and *reordering* of the six fields a compile error. It cannot make "used an own-slot probe instead of `emit_object_has_property_with_key_tag_i32`" a compile error, because both are `&mut self` methods returning `Result<(), EmitError>` with identical shapes. |
| LN3 | `ValidatedDescriptor::from_runtime_checked` | Two call sites (`standard.rs:11572`, `:11905`) discharge 6.2.6.5 step 9 by an *emitted Wasm check* at `standard.rs:11363-11380`, in a file this lane may not edit. The escape hatch's value is that `rg from_runtime_checked` enumerates the obligation-by-convention sites, and the count is exactly two. Upgrading it to a witness token requires owning `standard.rs`. **Note-routed.** |
| LN4 | `porffor-runtime`'s `IntrinsicPropertyAttributes` | See §5.8. No dependency edge, no shared crate, and the tie would have to live in `porffor-engine`, which is unowned. |
| LN5 | The emitted Wasm's own arithmetic | `DescriptorWord` proves every *constant* seed. I11 proves that a writable bit cannot be OR'd in on a statically-accessor path. Neither proves the `I64Or`/`I64And` sequence in a `Dynamic`-kind body is the right one; that is one function's body, `DescriptorWordEmitter::<Dynamic>::set_writable_if`, and it must be right once. |
| LN6 | `emit_validate_array_named_descriptor`'s `requested_data_descriptor: bool` | Its **only** two call sites are `array.rs:4141` and `:4544`, in an unowned file. Changing the parameter type breaks them. **Note-routed**; the body's bit handling lands this round. |
| LN7 | The 8 array/arguments derivation sites | `standard.rs:2454, 2741, 3007, 3432`; `array.rs:3709, 3821, 4062, 4463`. Unowned. **Note-routed**, with the retrofit instructions and the §6.5 trace that proves them. |
| LN8 | `lowering.rs`'s three shape sites | Shared hub; batch 5 is in `standard.rs` this round and `lowering.rs` is the crate's largest contention surface. **Note-routed** with the exact replacement per site (§4.5). |

### 2.1 `DescriptorField` — I12

```rust
/// The six fields of a 6.2.6 Property Descriptor, and their property keys.
///
/// This is the *only* place any of these six strings is spelled in the
/// workspace. `"writeable"`, `"enumberable"` and every other plausible typo are
/// `E0599: no variant or associated item named ...`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DescriptorField {
    Value,
    Writable,
    Get,
    Set,
    Enumerable,
    Configurable,
}

impl DescriptorField {
    /// The 6.2.6 Table 3 property key for this field.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Writable => "writable",
            Self::Get => "get",
            Self::Set => "set",
            Self::Enumerable => "enumerable",
            Self::Configurable => "configurable",
        }
    }

    /// Which side of the 6.2.6.1/6.2.6.2 split this field determines.
    /// `[[Enumerable]]` and `[[Configurable]]` determine neither, which is why
    /// a descriptor carrying only those is *generic* (6.2.6.3).
    pub const fn side(self) -> Option<DescriptorSide> {
        match self {
            Self::Value | Self::Writable => Some(DescriptorSide::Data),
            Self::Get | Self::Set => Some(DescriptorSide::Accessor),
            Self::Enumerable | Self::Configurable => None,
        }
    }

    pub const ALL: [Self; 6] = [
        Self::Value, Self::Writable, Self::Get,
        Self::Set, Self::Enumerable, Self::Configurable,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorSide { Data, Accessor }
```

Both `match`es are exhaustive with no `_` arm. Adding a seventh field to the
enum — which the spec will not do, but a private-field proposal might — becomes
two compile errors rather than two silent holes.

`ALL` is pinned by a const assertion that it agrees with the match arms:

```rust
const _: () = {
    let mut seen = [false; 6];
    let mut i = 0;
    while i < 6 { seen[DescriptorField::ALL[i] as usize] = true; i += 1; }
    assert!(seen[0] && seen[1] && seen[2] && seen[3] && seen[4] && seen[5],
            "DescriptorField::ALL must list every variant exactly once");
};
```

### 2.2 `Presence<T, R>` — I3, the load-bearing item

```rust
/// Whether a 6.2.6 field is present, and where its value lives.
///
/// This replaces the pair (`data: Option<(u32,u32)>`, `data_present_local:
/// Option<u32>`) at `objects.rs:13538`/`:13544`, whose two `None`s meant
/// different things and whose `(None, Some(_))` combination said "statically
/// absent" and "maybe present at run time" at the same time.
///
/// `T` is the value carrier. `R` is the carrier for a *runtime* presence flag;
/// a carrier that has no such concept sets `R = core::convert::Infallible`,
/// which makes the `Runtime` variant unconstructible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence<T, R> {
    /// 6.2.6: the record does not have this field. Known when the compiler runs.
    Absent,
    /// The record has this field, and the compiler knows it.
    Present(T),
    /// Whether the record has this field is decided when the program runs:
    /// `present` is nonzero iff the field is there. `value` is meaningful only
    /// then, but is *always* supplied — which is what makes the old
    /// `(None, Some(_))` pair unspellable.
    Runtime { present: R, value: T },
}

impl<T, R> Presence<T, R> {
    /// 6.2.6-level "has this field", as a three-valued answer. There is no
    /// two-valued form, deliberately.
    pub const fn known(&self) -> KnownPresence {
        match self {
            Self::Absent => KnownPresence::No,
            Self::Present(_) => KnownPresence::Yes,
            Self::Runtime { .. } => KnownPresence::AtRuntime,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownPresence { No, Yes, AtRuntime }
```

Three things this buys, each checkable:

1. `Presence::Runtime` **requires** a value carrier. `standard.rs:11572`'s
   `data: None, data_present_local: Some(value_present_local)` has no
   translation that keeps both halves; the caller must choose, and §5.2 shows
   which choice is correct and why it is not the obvious one.
2. `KnownPresence` has three variants, so every `match` on presence has an
   `AtRuntime` arm. Today those arms are `if let Some(..) { .. }` with **no
   else**, which is the M5 mechanism.
3. `R = Infallible` gives `namespace.rs` a `Presence` whose `Runtime` variant
   cannot be constructed — an emitted-at-compile-time descriptor cannot acquire
   a runtime-conditional key. That is I14's compile error and it costs nothing.

### 2.3 `DescriptorCarrier` — the one generic parameter

```rust
/// What a descriptor's fields are made of, in one lowering context.
///
/// Deliberately not sealed: `porffor-aot-wasm` adds its own impl. The trait has
/// no methods, so an impl cannot misbehave.
pub trait DescriptorCarrier {
    /// Carrier for `[[Value]]`, `[[Get]]`, `[[Set]]`.
    type Value;
    /// Carrier for `[[Writable]]`, `[[Enumerable]]`, `[[Configurable]]`.
    type Flag;
    /// Carrier for a runtime presence flag. `Infallible` if there is none.
    type RuntimeFlag;
}

/// Descriptors rendered as JavaScript source text (`modules/namespace.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceText;

impl DescriptorCarrier for SourceText {
    type Value = String;                     // an expression, already rendered
    type Flag = bool;                        // `true` / `false`
    type RuntimeFlag = core::convert::Infallible;
}
```

and, in `porffor-aot-wasm` (`objects.rs`, owned region):

```rust
/// A tagged value living in two Wasm locals: `(payload, tag)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TaggedLocals { pub payload: u32, pub tag: u32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WasmLocals;

impl DescriptorCarrier for WasmLocals {
    type Value = TaggedLocals;   // exactly today's `(u32, u32)`
    type Flag = u32;             // exactly today's `*_payload_local`
    type RuntimeFlag = u32;      // exactly today's `*_present_local`
}
```

Note that `TaggedLocals { payload, tag }` also fixes a live hazard that no
mistake class named: `emit_object_define_entry` destructures its parameters as
`(data_payload_local, data_tag_local)` at `:13613` but as
`(getter_payload, getter_tag)` at `:13624` — consistent — while
`emit_object_append_accessor_property_with_flags` builds them as
`(payload, tag)` at `:1349-1352`. All four agree today, but the tuple gives no
reason they must. Named fields make a transposition `E0560`.

### 2.4 `PartialDescriptor` and `ValidatedDescriptor` — I4, I5

```rust
/// A 6.2.6 Property Descriptor: six independently present-or-absent fields.
///
/// Every combination is representable, *including* one carrying both
/// `[[Value]]` and `[[Get]]` — 6.2.6.5 step 9 makes that a **TypeError**, not
/// an unrepresentable state, and a type that banned it would be a spec error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartialDescriptor<C: DescriptorCarrier> {
    pub value:        Presence<C::Value, C::RuntimeFlag>,
    pub writable:     Presence<C::Flag,  C::RuntimeFlag>,
    pub get:          Presence<C::Value, C::RuntimeFlag>,
    pub set:          Presence<C::Value, C::RuntimeFlag>,
    pub enumerable:   Presence<C::Flag,  C::RuntimeFlag>,
    pub configurable: Presence<C::Flag,  C::RuntimeFlag>,
}

impl<C: DescriptorCarrier> PartialDescriptor<C> {
    /// The empty descriptor, `{}` — generic, and the 6.2.6.6 completion of it
    /// is the all-defaults data property. This is `15.2.3.6-4-52.js`.
    pub const fn empty() -> Self { /* six `Presence::Absent` */ }

    /// The fields 6.2.6.4 would emit, in 6.2.6.4 step order. Never yields a
    /// data-side and an accessor-side field from the same descriptor unless
    /// this descriptor failed 6.2.6.5 step 9 — which `ValidatedDescriptor`
    /// makes impossible to reach.
    pub fn present_fields(&self) -> impl Iterator<Item = DescriptorField> + '_;
}
```

and the validated newtype:

```rust
/// A `PartialDescriptor` that has passed **6.2.6.5 step 9**: it is not both a
/// data and an accessor descriptor. The three-way partition of 6.2.6.1–3 is a
/// partition only on this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedDescriptor<C: DescriptorCarrier>(PartialDescriptor<C>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BothDataAndAccessor {
    pub data_side: DescriptorField,      // Value or Writable
    pub accessor_side: DescriptorField,  // Get or Set
}

impl<C: DescriptorCarrier> PartialDescriptor<C> {
    /// Statically discharge 6.2.6.5 step 9. Available when every presence is
    /// `Absent` or `Present`; a `Runtime` presence on both sides cannot be
    /// decided here and returns `Err(NeedsRuntimeCheck)`.
    pub fn validate(self) -> Result<ValidatedDescriptor<C>, ValidateError>;

    /// 6.2.6.5 step 9 has been discharged by an **emitted run-time check**
    /// elsewhere. NOT an invariant — a named escape hatch. Ledger row LN3.
    ///
    /// Declared call sites, and the lines that discharge the obligation:
    ///   * `builtins/standard.rs:11572` — discharged at `standard.rs:11364-11380`
    ///   * `builtins/standard.rs:11905` — discharged at `standard.rs:11364-11380`
    ///     (the `Else` of the same `If`, so `getter_present == setter_present == 0`)
    /// Adding a third caller without adding a row here is the defect this
    /// doc-comment exists to make visible to `rg`.
    pub fn from_runtime_checked(self) -> ValidatedDescriptor<C>;
}
```

`ValidatedDescriptor`'s field is private and it is **not** `DerefMut`; the only
read access is `fn as_partial(&self) -> &PartialDescriptor<C>`. Mutating a
validated descriptor back into an invalid one requires re-validating.

### 2.5 `PropertyDescriptorKind` and `classify` — I1, I6

```rust
/// The 6.2.6.1 / 6.2.6.2 / 6.2.6.3 partition. Three cases, and `Generic` is one
/// of them rather than a hole patched afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyDescriptorKind { Data, Accessor, Generic }

/// The partition, evaluated against presences that may not all be static.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorClassification<C: DescriptorCarrier> {
    /// Every relevant presence is `Absent` or `Present`: the case is decided.
    Static(PropertyDescriptorKind),
    /// At least one of the four kind-determining fields has a `Runtime`
    /// presence. The disjunctions are named so a consumer emits the *spec's*
    /// predicate rather than re-deriving one.
    Dynamic {
        /// Presence flags whose disjunction is 6.2.6.1 IsAccessorDescriptor.
        /// Empty iff the accessor side is statically absent.
        accessor_terms: KindTerms<C>,
        /// Presence flags whose disjunction is 6.2.6.2 IsDataDescriptor.
        data_terms: KindTerms<C>,
    },
}

/// The statically-known part and the runtime part of one side's disjunction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KindTerms<C: DescriptorCarrier> {
    /// `true` iff some field on this side is `Presence::Present` — the side is
    /// then unconditionally true and `runtime` need not be consulted.
    pub statically_true: bool,
    /// Runtime presence flags, in `DescriptorField::ALL` order. At most two.
    pub runtime: [Option<C::RuntimeFlag>; 2],
}

/// The one derivation of the partition in the workspace.
pub fn classify<C: DescriptorCarrier>(
    desc: &ValidatedDescriptor<C>,
) -> DescriptorClassification<C>;
```

**Rule, and it is the rule the whole area exists for:** every `match` on
`DescriptorClassification` or on `PropertyDescriptorKind` in the workspace is
exhaustive with **no `_` arm**. `#[deny(clippy::wildcard_enum_match_arm)]` is
*not* sufficient (it is allow-by-default and lanes disable lints); the contract's
enforcement is that both enums are `#[non_exhaustive]`-free and the reviewer
greps for `_ =>` in the same statement as a `PropertyDescriptorKind`.

Since `classify` takes `&ValidatedDescriptor`, the "both sides true" case cannot
arrive, and `Static` has exactly three inhabitants — which is what makes the
`Generic` arm mandatory at every consumer. Today, `objects.rs:13575-13579` has no
such arm; it is a two-way `if`.

### 2.6 `CompleteDescriptor` — I2, and the reason `Accessor` has no `writable`

```rust
/// 6.2.6.6's output, and 10.1.6.3 step 3's assertion: a **fully populated**
/// Property Descriptor. This is what a stored property is.
///
/// There is no `Generic` variant. 6.2.6.6 step 3 says a generic descriptor
/// completes to a data descriptor, so `Generic` is a classification of an
/// *incoming* descriptor and never a stored kind.
///
/// `Accessor` has **no `writable` field**. 10.1.6.3 steps 6.b and 7 say the
/// conversion between kinds preserves only `[[Enumerable]]` and
/// `[[Configurable]]`; an accessor that could carry `[[Writable]]` is not a
/// representation of anything 6.2.6 defines, and the stale bit it would carry
/// is mistake class M1 — shipped once already, in commit `fae75423a`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompleteDescriptor<C: DescriptorCarrier> {
    Data {
        value: C::Value,
        writable: C::Flag,
        enumerable: C::Flag,
        configurable: C::Flag,
    },
    Accessor {
        get: C::Value,
        set: C::Value,
        enumerable: C::Flag,
        configurable: C::Flag,
    },
}

impl<C: DescriptorCarrier> CompleteDescriptor<C> {
    /// 6.2.6.4's codomain when the input is complete: exactly four keys, one of
    /// exactly two sets. This is what `Object.getOwnPropertyDescriptor` returns
    /// (10.1.8.1 → 10.1.6.1), and it is the direct refutation of
    /// `lowering.rs:25611`'s six-key claim.
    pub const fn keys(&self) -> [DescriptorField; 4] {
        match self {
            Self::Data { .. } => [
                DescriptorField::Value, DescriptorField::Writable,
                DescriptorField::Enumerable, DescriptorField::Configurable,
            ],
            Self::Accessor { .. } => [
                DescriptorField::Get, DescriptorField::Set,
                DescriptorField::Enumerable, DescriptorField::Configurable,
            ],
        }
    }

    pub const fn kind(&self) -> PropertyDescriptorKind {
        match self {
            Self::Data { .. } => PropertyDescriptorKind::Data,
            Self::Accessor { .. } => PropertyDescriptorKind::Accessor,
        }
    }
}
```

and 6.2.6.6 itself, as the only bridge from partial to complete:

```rust
/// 6.2.6.6 CompletePropertyDescriptor.
///
/// Consumes the validated descriptor: completing twice is meaningless and
/// `E0382` (use of moved value) says so. The `defaults` argument supplies the
/// carrier-specific spellings of `undefined` and `false` (Wasm locals holding
/// them; the strings `"undefined"` / `"false"`).
///
/// May be called only on the 10.1.6.3 **step 2** path — the create-a-new-entry
/// path. Calling it on the existing-entry path is mistake class M2.
pub fn complete_property_descriptor<C: DescriptorCarrier>(
    desc: ValidatedDescriptor<C>,
    defaults: &CompletionDefaults<C>,
) -> CompleteDescriptor<C>;
```

Its body is a `match classify(&desc)` with three arms, and the `Generic` and
`Data` arms are the *same* arm by 6.2.6.6 step 3 — written as
`PropertyDescriptorKind::Data | PropertyDescriptorKind::Generic =>`, an
or-pattern, so adding a fourth kind is still a compile error.

### 2.7 `DescriptorWord`, `DescriptorMask`, `DescriptorFlags`, `MappedSlot` — I7, I8, I9

These live in `crates/porffor-aot-wasm/src/heap.rs`, **additively**: the six
existing `pub(crate) const`s keep their names, their types (`u64`) and their
values, and gain derivations.

```rust
/// One bit of the descriptor-kind word. Closed; see the layout table in the
/// contract, §1.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescriptorBit {
    Accessor = 0,
    Configurable = 1,
    Writable = 2,
    Enumerable = 3,
    ArrayOwnProperty = 4,
    ArgumentsMapped = 5,
}

impl DescriptorBit {
    pub(crate) const fn word(self) -> u64 { 1u64 << (self as u32) }
}

/// A descriptor-kind word as **stored** in a heap slot.
///
/// Constructors are exactly the two 6.2.6.6 licenses. `of_accessor` takes no
/// `writable` argument, so the bit pattern `ACCESSOR | WRITABLE` (= 5) has no
/// constructor — which is mistake class M1, made unspellable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DescriptorWord(u64);

impl DescriptorWord {
    pub(crate) const fn of_data(writable: bool, enumerable: bool, configurable: bool) -> Self;
    pub(crate) const fn of_accessor(enumerable: bool, configurable: bool) -> Self;

    /// Attach the orthogonal exotic axis. Separate from the two constructors so
    /// the kind decision is never made *by* a flag.
    pub(crate) const fn with_flags(self, flags: DescriptorFlags) -> Self;

    pub(crate) const fn bits(self) -> u64;
    pub(crate) const fn as_i64(self) -> i64 { self.bits() as i64 }
}

/// A **test** against a descriptor word. Deliberately a different type from
/// `DescriptorWord`: composites like `ACCESSOR | WRITABLE` are legal and
/// *needed* as masks (`objects.rs:1560-1565` uses exactly that one) while being
/// illegal as values. There is no `From<DescriptorMask> for DescriptorWord` and
/// no `From<DescriptorWord> for DescriptorMask`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DescriptorMask(u64);

impl DescriptorMask {
    pub(crate) const ACCESSOR: Self;
    pub(crate) const WRITABLE: Self;
    pub(crate) const ENUMERABLE: Self;
    pub(crate) const CONFIGURABLE: Self;
    /// `objects.rs:1562`: "existing is a data descriptor **and** is not
    /// writable", in one `I64And`. A legal mask; not a legal word.
    pub(crate) const ACCESSOR_OR_WRITABLE: Self;
    pub(crate) const KIND_AND_ATTRIBUTES: Self;   // bits 0..3
    pub(crate) const EXOTIC_FLAGS: Self;          // bits 4..5
    pub(crate) const fn bits(self) -> u64;
    pub(crate) const fn as_i64(self) -> i64;
    pub(crate) const fn union(self, other: Self) -> Self;
}

/// The exotic axis (§1.7). Orthogonal to the descriptor kind: an array's
/// own-property marker and an arguments object's mapping are not descriptor
/// kinds and must not share the kind namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DescriptorFlags {
    pub(crate) array_own_property: bool,
    /// `Some(slot)` sets bit 5 **and** packs `slot` into bits 32..63.
    /// The two cannot be set independently, which is what
    /// `functions.rs:6404` does by hand today.
    pub(crate) mapped: Option<MappedSlot>,
}

/// A mapped-arguments environment slot index, living in bits 32..63.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MappedSlot(u32);

impl MappedSlot {
    pub(crate) const SHIFT: u32 = 32;
    pub(crate) const fn new(slot: u32) -> Self { Self(slot) }
    pub(crate) const fn packed(self) -> u64 { (self.0 as u64) << Self::SHIFT }
}
```

with the disjointness the layout depends on, asserted at build time:

```rust
const _: () = assert!(
    DescriptorMask::KIND_AND_ATTRIBUTES.bits() & DescriptorMask::EXOTIC_FLAGS.bits() == 0,
    "the descriptor kind bits and the exotic flag bits must not overlap",
);
const _: () = assert!(
    (DescriptorMask::KIND_AND_ATTRIBUTES.bits() | DescriptorMask::EXOTIC_FLAGS.bits())
        < (1u64 << MappedSlot::SHIFT),
    "every flag bit must sit below the mapped-slot payload at bit 32",
);
const _: () = assert!(
    MappedSlot::SHIFT == 32,
    "functions.rs:6987 and :7072 shift the stored word right by a literal 32",
);
```

The third assertion is the one that earns its place: the *reader* of the mapped
slot is a bare `I64Const(32); I64ShrU` in an **unowned** file
(`functions.rs:6986-6988`, `:7071-7073`). Changing `SHIFT` here without changing
those two would be silent today; with the assertion it is at least pinned to the
literal the readers use, and §4.6 routes the reader-side change.

### 2.8 The heap constants as derivations — I10

`heap.rs:948-957` becomes, with **the same names, types and values**:

```rust
pub(crate) const OBJECT_DESCRIPTOR_ACCESSOR: u64 = DescriptorBit::Accessor.word();
pub(crate) const OBJECT_DESCRIPTOR_CONFIGURABLE: u64 = DescriptorBit::Configurable.word();
pub(crate) const OBJECT_DESCRIPTOR_WRITABLE: u64 = DescriptorBit::Writable.word();
pub(crate) const OBJECT_DESCRIPTOR_ENUMERABLE: u64 = DescriptorBit::Enumerable.word();
/// Data is the **absence** of the accessor bit. Named so the eight files that
/// spell `OBJECT_DESCRIPTOR_DATA` keep compiling; `DescriptorWord::of_data` is
/// the constructor that makes the absence a decision rather than a default.
pub(crate) const OBJECT_DESCRIPTOR_DATA: u64 = 0;
pub(crate) const ARRAY_DESCRIPTOR_OWN_PROPERTY: u64 = DescriptorBit::ArrayOwnProperty.word();
pub(crate) const ARGUMENTS_DESCRIPTOR_MAPPED: u64 = DescriptorBit::ArgumentsMapped.word();
pub(crate) const ARRAY_DESCRIPTOR_NORMAL_DATA: u64 =
    DescriptorWord::of_data(true, true, true).bits();
```

pinned:

```rust
const _: () = assert!(OBJECT_DESCRIPTOR_ACCESSOR == 1);
const _: () = assert!(OBJECT_DESCRIPTOR_CONFIGURABLE == 2);
const _: () = assert!(OBJECT_DESCRIPTOR_WRITABLE == 4);
const _: () = assert!(OBJECT_DESCRIPTOR_ENUMERABLE == 8);
const _: () = assert!(OBJECT_DESCRIPTOR_DATA == 0);
const _: () = assert!(ARRAY_DESCRIPTOR_OWN_PROPERTY == 16);
const _: () = assert!(ARGUMENTS_DESCRIPTOR_MAPPED == 32);
const _: () = assert!(ARRAY_DESCRIPTOR_NORMAL_DATA == 14);
```

These eight are not tautologies. They pin the **wire format** of a word written
at 9 heap offsets by 177 store sites and read by 176 references across 11 files,
so reordering `DescriptorBit`'s variants is a compile error rather than a
silent, total corruption of every object's property attributes.

**The hard constraint, restated as an acceptance test.** This edit must be
purely additive: no existing constant may change name, type or value. The check
is `cargo check -p porffor-aot-wasm` producing zero errors **without any edit to
`builtins/intl_datetimeformat.rs` (2 refs), `builtins/array.rs` (76 refs),
`builtins/standard.rs` (76 refs), `functions.rs` (17 refs), `control_flow.rs`
(3), `builtins/json.rs` (2), `emit.rs` (1), `builtins/string.rs` (11),
`builtins/intl.rs` (1)** — the first of which is on batch 2's hold list.

### 2.9 `DescriptorWordEmitter<K>` — I11, the runtime half of M1

`DescriptorWord` proves every *constant*. It proves nothing about
`objects.rs:13580-13612`, where the word is assembled at run time by OR-ing bits
into `descriptor_kind_local` under `If` guards. The typestate closes that.

```rust
mod sealed { pub trait Sealed {} }

pub(crate) trait DescriptorKindMarker: sealed::Sealed {
    const KIND: Option<PropertyDescriptorKind>;   // `None` for `Dynamic`
}

pub(crate) struct Data;      // impl DescriptorKindMarker { KIND = Some(Data) }
pub(crate) struct Accessor;  // ..                          Some(Accessor)
pub(crate) struct Generic;   // ..                          Some(Generic)
pub(crate) struct Dynamic;   // ..                          None

/// Builds the run-time descriptor-kind word in a Wasm local.
///
/// `K` is the *statically known* kind, from `classify`. Seeding requires
/// naming it, so a two-way `if has_data { DATA } else { ACCESSOR }`
/// (`objects.rs:13575-13579`) has no translation: `classify` returns three
/// static cases plus `Dynamic`, and the caller must have an arm for each.
pub(crate) struct DescriptorWordEmitter<K: DescriptorKindMarker> {
    local: u32,
    _kind: core::marker::PhantomData<K>,
}

impl<K: DescriptorKindMarker> DescriptorWordEmitter<K> {
    pub(crate) fn seed(builder: &mut FunctionBuilder, local: u32, f: &mut Function) -> Self;
    pub(crate) fn set_bit_if_nonzero(&mut self, bit: DescriptorBit, flag_local: u32, f: &mut Function);
    pub(crate) fn local(&self) -> u32;
}

impl DescriptorWordEmitter<Data> {
    /// 10.1.6.3 step 8 / 6.2.6.6: OR in `[[Writable]]`. Exists only here.
    pub(crate) fn set_writable_if(&mut self, flag_local: u32, f: &mut Function);
}

impl DescriptorWordEmitter<Dynamic> {
    /// The kind is a run-time value, so the writable OR-in must itself be
    /// guarded by the run-time "is this a data descriptor" predicate, and the
    /// signature demands that predicate rather than trusting the caller to
    /// have opened an `If`. Ledger row LN5: this body must be right once.
    pub(crate) fn set_writable_if_data(
        &mut self, is_data: RuntimeKindPredicate, flag_local: u32, f: &mut Function,
    );
}
```

There is **no** `set_writable_if` on `DescriptorWordEmitter<Accessor>` and none
on `<Generic>`. `objects.rs:13973-13997`'s 21 hand-written instructions — the
by-hand M1 repair, guarded by a comment — become either an arm that cannot
mention writability (`Accessor`) or a call to the one method that can (`Data`).
That is what turns the comment at `:13965-13972` into a type.

`set_bit_if_nonzero` stays available on all four markers because
`[[Enumerable]]` and `[[Configurable]]` are legal on every kind — which is
exactly 10.1.6.3 steps 6.b and 7 ("preserve only `[[Enumerable]]` and
`[[Configurable]]`"), now readable off the API surface.

### 2.10 What is *not* typed, and why — the four honest negatives

1. **The comparison operator (LN1).** No descriptor type distinguishes
   `emit_tagged_payload_same_value_i32` from `emit_tagged_payload_equality_i32`.
   Both take four `u32` locals and a `&mut Function`. And both are correct
   *somewhere* in this very function: `SameValue` at `:13834` for `[[Value]]`
   (step 4.e), plain equality at `:13877` and `:13920` for `[[Get]]`/`[[Set]]`
   (step 4.d), where the operands are objects and the two agree. A newtype on
   the operand would have to be threaded through `TaggedLocals`, and
   `TaggedLocals` is what the *whole emitter* passes around; making it
   descriptor-field-indexed would spread this area's types into every unrelated
   caller. That is the decoration test failing. **Ledger.** §6.6 and §6.14 give
   the two counterexamples that pin the operator from opposite directions.
2. **The read primitive in 6.2.6.5 (LN2).** I15 makes the *table* of six fields
   compile-checked; the *primitive* used to read each one is a method call. A
   type would have to distinguish "this method performs a `[[Get]]`-visible
   lookup" from "this method reads an own slot", which is a property of the
   emitted Wasm.
3. **`porffor-runtime`'s parallel model (LN4).** §5.8.
4. **The array/arguments derivations (LN7) and the `lowering.rs` shapes
   (LN8).** Unowned files. The types land; the retrofit is note-routed with
   per-site instructions and a dry-run trace (§6.5) that proves them before
   another batch executes them.

---

## 3. The mistake-class table

| # | Mistake, as a program or an edit | Today | Under this contract | Named construct |
|---|---|---|---|---|
| **M1** | An accessor entry carries a `[[Writable]]` bit, so a later accessor→data conversion yields `writable: true`. `var o={}; Object.defineProperty(o,'x',{get(){return 1},configurable:true}); Object.defineProperty(o,'x',{value:2})` | Fixed by 21 hand-written instructions at `objects.rs:13973-13997` under a comment; the word `5` is still constructible | **E0599** — `no method named set_writable_if found for struct DescriptorWordEmitter<Accessor>`; and the constant form has no constructor at all, because `DescriptorWord::of_accessor` takes no `writable` parameter | `DescriptorWordEmitter<Accessor>`, `DescriptorWord::of_accessor`, `CompleteDescriptor::Accessor` |
| **M2** | A field absent from the incoming descriptor is written as an explicit value on an existing property. `Object.defineProperties(o,{x:{enumerable:false}})` clearing `writable` | Fixed by three `if let Some(present) { .. }` re-read blocks (`:13941`, `:13998`, `:14015`) that emit **nothing** when the caller passed `None` | **E0004** — `match` on `Presence::known()` must have an arm for each of `No`, `Yes`, `AtRuntime`; the "caller passed `None`" state no longer exists, because `Absent` and `AtRuntime` are different values | `Presence<T, R>`, `KnownPresence` |
| **M3** | `[[Value]]` compared with `F64Eq` instead of `SameValue` in 10.1.6.3 step 4.e | Correct, by calling a correctly-named helper at 2 sites | **LEDGER (LN1)** — with counterexamples `15.2.3.6-4-131.js` (`-0`/`+0`: `SameValue` false, `F64Eq` true) and §6.14 (`NaN`: `SameValue` true, `F64Eq` false), which fail in **opposite** directions | — |
| **M4** | 6.2.6.5 reads own data slots, or reads the six fields in the wrong order / drops one | Correct; order lives in a plain array literal at `objects.rs:11962-11999` under a comment at `:11960` | **Split.** Dropping or reordering a field → **compile error** via `TO_PROPERTY_DESCRIPTOR_ORDER`'s const assertion (I15). Using an own-slot probe instead of HasProperty → **LEDGER (LN2)** | `DescriptorField`, `TO_PROPERTY_DESCRIPTOR_ORDER` |
| **M5** | A generic descriptor — `Object.defineProperty(o,'x',{enumerable:true})` on a fresh key — classified as an accessor | `objects.rs:13575-13579` makes it `ACCESSOR` by construction; the correction at `:14036-14064` is gated on **all four** presence locals being `Some`, so a future caller that statically knows one field is absent disarms it silently. Currently unreached (§5.1) | **E0004** — `classify` returns `DescriptorClassification`, whose `Static` payload has three inhabitants; a two-arm `if` has no translation and a `match` without a `Generic` arm does not compile | `PropertyDescriptorKind::Generic`, `DescriptorClassification` |
| **M5′** | The *coupling* defect §5.2 found: two independent 10.1.6.3 step-4 obligations gated on one four-way `is_some()` conjunction | `objects.rs:13726-13777` | **E0004 twice** — each obligation is emitted from its own `match` on its own side's `KindTerms`, so dropping one is a missing arm, not a silently-false conjunct | `KindTerms`, `DescriptorClassification::Dynamic` |
| **M6** | A shape naming a key `FromPropertyDescriptor` never emits — `writable` and `get` in one object shape | `lowering.rs:25611-25626` lists all six; 3 call sites. Currently inert (§5.7) | **Unconstructible** — the only shape builders are `CompleteDescriptor::keys()` (exactly 4, one of 2 sets) and `PartialDescriptor::present_fields()` (partition-respecting). A six-key list has no producer | `CompleteDescriptor::keys`, `PartialDescriptor::present_fields` |
| **M7** | A misspelt descriptor key: `vec![("writeable", ..)]`, or `push_str(", configurabel: false })")` | Compiles. In a shape: a property that never exists. In source text: 6.2.6.5 ignores the stray key, so the property silently takes the *default*, which is usually the value the author wanted — no diagnostic, ever | **E0599** — `no variant or associated item named Writeable found for enum DescriptorField` | `DescriptorField` |

---

## 4. The retrofit map

### 4.1 Order

The order is chosen so that every step compiles on its own — the lane has no
build access, so a step that only compiles once a later step lands is a step
that fails silently.

| Step | File | Change | Compiles alone? |
|---|---|---|---|
| **1** | `crates/porffor-ir/src/property_descriptor.rs` (NEW) | I1–I6, I12–I15: `DescriptorField`, `Presence`, `DescriptorCarrier`, `SourceText`, `PartialDescriptor`, `ValidatedDescriptor`, `PropertyDescriptorKind`, `DescriptorClassification`, `classify`, `CompleteDescriptor`, `complete_property_descriptor`, `TO_PROPERTY_DESCRIPTOR_ORDER`, `DescriptorSourceText` | Yes — depends on nothing |
| **2** | `crates/porffor-ir/src/lib.rs` | **Two lines only**: `mod property_descriptor;` in the `mod` block at `:56-78`, and one appended `pub use crate::property_descriptor::{..};` block after `:151` | Yes |
| **3** | `crates/porffor-ir/src/modules/namespace.rs` | Route the three builders (`:348-368`, `:371-378`, `:742-749`) through `DescriptorSourceText`. **Byte-identical output required.** | Yes |
| **4** | `crates/porffor-aot-wasm/src/heap.rs` | I7–I10, **additive**: `DescriptorBit`, `DescriptorWord`, `DescriptorMask`, `DescriptorFlags`, `MappedSlot`, the 8 derivations, the 11 `const _` assertions | Yes |
| **5** | `crates/porffor-aot-wasm/src/objects.rs` §A | `WasmLocals` carrier + `TaggedLocals`; rewrite `object_data_descriptor_kind` (`:45-61`) and `object_accessor_descriptor_kind` (`:63-72`) as one-line delegations to `DescriptorWord`. **Signatures unchanged** — `functions.rs:3017` is unowned | Yes |
| **6** | `crates/porffor-aot-wasm/src/objects.rs` §B | `emit_validate_array_named_descriptor` (`:1446-1650`) body only: replace the three raw bit expressions at `:1528`, `:1543`, `:1562` with `DescriptorMask` tests. **Signature unchanged** — `array.rs:4141`/`:4544` are unowned (LN6) | Yes |
| **7** | `crates/porffor-aot-wasm/src/objects.rs` §C | `DescriptorWordEmitter<K>` (I11) | Yes |
| **8** | `crates/porffor-aot-wasm/src/objects.rs` §D | `emit_object_define_entry` (`:13533-14372`): 15 descriptor parameters → `ValidatedDescriptor<WasmLocals>`; the four-way conjunction at `:13726` and the corrective block at `:14036` → exhaustive `match`es | **No** — needs step 9 in the same patch |
| **9** | `crates/porffor-aot-wasm/src/objects.rs` §E + `builtins/standard.rs` **(2 lines each, adapter only)** | The four call sites. `objects.rs:13422` and `:13511` are owned. `standard.rs:11572` and `:11905` are **not** — see §4.4 for the seam that keeps this lane out of that file | See §4.4 |
| **10** | `docs/rust-rewrite/contracts/…` (this file) + `target/lane-notes/property-descriptor-lattice-theory-integration.md` | The note carries LN6, LN7, LN8, LN3 and the §4.6 reader-side change | Yes |

### 4.2 Step 3 — `namespace.rs`, byte-identity required

```rust
/// A 6.2.6 partial descriptor rendered as the `{ … }` argument to
/// `Object.defineProperty`. Every key comes from `DescriptorField::key()`, so a
/// typo is `E0599` instead of a silently-ignored property that leaves the
/// attribute at its 6.2.6.6 default.
///
/// `SourceText::RuntimeFlag = Infallible`, so `Presence::Runtime` is
/// unconstructible here: a compile-time-emitted descriptor cannot acquire a
/// run-time-conditional key.
pub struct DescriptorSourceText(PartialDescriptor<SourceText>);

impl DescriptorSourceText {
    pub fn new() -> Self;                                   // all six Absent
    pub fn get(self, expr: String) -> Self;
    pub fn set(self, expr: String) -> Self;
    pub fn value(self, expr: String) -> Self;
    pub fn writable(self, flag: bool) -> Self;
    pub fn enumerable(self, flag: bool) -> Self;
    pub fn configurable(self, flag: bool) -> Self;

    /// Renders `{ k: v, … }` in **`DescriptorField::ALL` order** — value,
    /// writable, get, set, enumerable, configurable — which is 6.2.6.4's own
    /// step order and, verified below, the order the three existing builders
    /// already use.
    ///
    /// Returns `Err(BothDataAndAccessor)` rather than emitting source text that
    /// 6.2.6.5 step 9 would make throw.
    pub fn render(self) -> Result<String, BothDataAndAccessor>;
}
```

**Verification that byte-identity is achievable**, checked against the three
builders:

| Builder | Current literal | `DescriptorField::ALL` order | Match? |
|---|---|---|---|
| `:355`+`:368` | `{ get: <expr>, enumerable: true, configurable: false }` | get(3) < enumerable(5) < configurable(6) | **yes** |
| `:376`+`:378` | `{ value: <lit>, writable: false, enumerable: false, configurable: false }` | value(1) < writable(2) < enumerable(5) < configurable(6) | **yes** |
| `:747`+`:749` | same as above | same | **yes** |

so `render()` in `ALL` order reproduces all three exactly. The check is the
existing test assertions in the same file, which are substring matches on the
rendered text and will fail loudly on any spacing change:
`:1144` `"get: () => value,"`, `:1167` `"\"b\", { get: () => a,"`,
`:1186` the full `Symbol.toStringTag` line, `:1259` `"get: () => {}"`,
`:1294`, `:1447`, `:1453`, `:1566`. **Eight assertions**; `cargo test -p
porffor-ir modules::namespace` is the check, and it is a rung-1 command the
integrator runs, not this lane.

One deliberate non-change: the `defineProperty` **call** wrapper
(`Object.defineProperty(<ns>, <key>, ` … `);\n`) stays as `push_str` calls.
`DescriptorSourceText` renders the descriptor object only. Absorbing the call
wrapper would give the type a second job and no second compile error.

### 4.3 Steps 5 and 6 — the two signature freezes, stated as constraints

**`object_data_descriptor_kind` and `object_accessor_descriptor_kind` keep
`-> u64`.** Measured, `object_data_descriptor_kind` has a call site at
`functions.rs:3017`, in an unowned file:

```rust
crate::objects::object_data_descriptor_kind(false, false, meta.length_name_configurable)
```

Changing the return type to `DescriptorWord` breaks it. The bodies become:

```rust
pub(crate) fn object_data_descriptor_kind(writable: bool, enumerable: bool, configurable: bool) -> u64 {
    DescriptorWord::of_data(writable, enumerable, configurable).bits()
}
pub(crate) fn object_accessor_descriptor_kind(enumerable: bool, configurable: bool) -> u64 {
    DescriptorWord::of_accessor(enumerable, configurable).bits()
}
```

Which is a *small* win — but a real one: it removes the `|=`-into-a-`mut`
accumulator (`:50-60`, `:64-71`) that is the shape a `writable` line could be
pasted into. `object_accessor_descriptor_kind` currently differs from
`object_data_descriptor_kind` only by *not having* three lines; after the change
it differs by calling a constructor that has no such parameter.

**`emit_validate_array_named_descriptor` keeps `requested_data_descriptor: bool`
(LN6).** Its only two call sites are `array.rs:4141` and `:4544`. In scope this
round: the three raw-bit expressions in the body.

| Line | Today | After |
|---|---|---|
| `:1528` | `I64Const(OBJECT_DESCRIPTOR_ACCESSOR as i64)` then `I64And` | `I64Const(DescriptorMask::ACCESSOR.as_i64())` |
| `:1543` | `I64Const(OBJECT_DESCRIPTOR_WRITABLE as i64)` | `I64Const(DescriptorMask::WRITABLE.as_i64())` |
| `:1562` | `I64Const((OBJECT_DESCRIPTOR_ACCESSOR \| OBJECT_DESCRIPTOR_WRITABLE) as i64)` | `I64Const(DescriptorMask::ACCESSOR_OR_WRITABLE.as_i64())` |

All three emit identical bytes. `:1562` is the site §1.7 fact 2 exists for: it is
a **mask** spelling the bit pattern `5`, and it is correct.

The `requested_data_descriptor` → `PropertyDescriptorKind` change, and the
`kind_present_local` runtime fold at `:1504-1526` (which is a hand-rolled
`IsGenericDescriptor`), are in the note.

### 4.4 Steps 8 and 9 — the `emit_object_define_entry` seam

The new signature:

```rust
pub(crate) fn emit_object_define_entry(
    &mut self,
    object_local: u32,
    object_tag_local: Option<u32>,
    key_local: u32,
    descriptor: ValidatedDescriptor<WasmLocals>,
    function: &mut Function,
) -> Result<(), EmitError>
```

16 parameters → 5. The two owned call sites translate mechanically:

```rust
// objects.rs:13422, was: data Some, everything else None
PartialDescriptor {
    value:        Presence::Present(TaggedLocals { payload: payload_local, tag: tag_local }),
    writable:     Presence::Present(writable_payload_local),
    get:          Presence::Absent,
    set:          Presence::Absent,
    enumerable:   Presence::Present(enumerable_payload_local),
    configurable: Presence::Present(configurable_payload_local),
}.validate().expect("a data-only descriptor cannot fail 6.2.6.5 step 9")
```

```rust
// objects.rs:13511, was: getter/setter forwarded, data None, all presence None
PartialDescriptor {
    value:        Presence::Absent,
    writable:     Presence::Absent,             // <-- see below
    get:          presence_of(getter),
    set:          presence_of(setter),
    enumerable:   Presence::Present(enumerable_payload_local),
    configurable: Presence::Present(configurable_payload_local),
}.validate().expect("an accessor-only descriptor cannot fail 6.2.6.5 step 9")
```

The `writable: Absent` line is the change that makes I2 real. Today this helper
**reserves a temp local, stores `0` into it, and passes it as
`writable_payload_local`** (`objects.rs:13508-13510`, `:13518`) — a
`[[Writable]]` operand supplied to a descriptor that has no `[[Writable]]`
field, purely so the positional slot has something in it. Under `Presence` the
slot does not need filling, so those three lines and the
`release_temp_local(writable_payload_local)` at `:13529` **delete**. That is the
first place the type pays for itself in deleted code rather than added code.

**Byte-identity for these two:** the emitted Wasm is unchanged. `Absent` and
today's `None` produce the same (empty) emission at every one of the seven
guarded body sites, and the `I64Const(0); LocalSet(writable_payload_local)`
pair that disappears is dead — `writable_payload_local` is read at `:13583`
only under `if has_data`, which is false here, and at `:13782`/`:13975` only
under `if let Some(writable_present_local)`, which is `None` here.
**Three instructions removed, all provably unreachable.** The dry-runner must
confirm this against the rung-G diff (§7).

**The two unowned call sites.** `standard.rs:11572` and `:11905` cannot be
edited by this lane. The seam is a thin adapter **in `objects.rs`**, keeping the
old 16-parameter name and signature:

```rust
/// Adapter for the two `Object.defineProperty` call sites in
/// `builtins/standard.rs`, which this lane does not own. Delete when
/// `standard.rs` is retrofitted; the note carries the instruction and the
/// exact `PartialDescriptor` literal for each site.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_object_define_entry_positional(
    &mut self, /* the existing 16 parameters, unchanged */
) -> Result<(), EmitError>
```

and `emit_object_define_entry`'s **name is reused for the new signature**, so
the two `standard.rs` sites must be renamed to `_positional` — a **2-line edit
in an unowned file**.

> **Coordination requirement, and it is not optional.** Two identifier renames
> at `standard.rs:11572` and `standard.rs:11905` are the *entire* footprint of
> this lane in that file. They do not touch
> `emit_create_data_property_or_throw` (`objects.rs:14374-14674`), the
> `Option<IteratorCloseOnThrowLocals>` parameter at `:14383`, or any Iterator
> helper. The integrator applies them; the lane does not. If the integrator
> judges even two lines too much contention with batch 5, the fallback is to
> name the new function `emit_object_define_entry_typed` and leave
> `emit_object_define_entry` untouched — at the cost of the *old* name
> surviving, which the note must then carry as a deletion instruction.

The adapter body is the *only* place `from_runtime_checked` is called (LN3):

```rust
let descriptor = PartialDescriptor {
    value: match data {
        Some(TaggedLocals { .. }) => /* Runtime, with the presence local */,
        None => Presence::Absent,     // <-- §5.2: NOT `Runtime` with no value
    },
    /* … */
}.from_runtime_checked();
```

and §5.2 is the section that says why, and what must be added so this is not a
regression.

### 4.5 What is note-routed, per site, with the replacement

`target/lane-notes/property-descriptor-lattice-theory-integration.md` carries:

**(a) The 8 array/arguments derivation sites (LN7).**

| Site | Current classification spelling | Replacement |
|---|---|---|
| `standard.rs:2454` `emit_arguments_define_data_index` | positional; kind implied by name | `ValidatedDescriptor<WasmLocals>` + `classify` |
| `standard.rs:2741` `emit_arguments_define_accessor_index` | ditto | ditto |
| `standard.rs:3007` `emit_arguments_define_callee` | ditto | ditto |
| `standard.rs:3432` `emit_store_arguments_length_descriptor_kind` | **`accessor: bool` at `:3441`** | `PropertyDescriptorKind` — the `Generic` arm is the one that is missing today |
| `array.rs:3709` `emit_array_define_data_index` | name | `ValidatedDescriptor` |
| `array.rs:3821` `emit_array_define_accessor_index` | name | ditto |
| `array.rs:4062` `emit_array_define_named_data_descriptor` | name + `emit_validate_array_named_descriptor(.., true, ..)` at `:4141` | `PropertyDescriptorKind::Data` |
| `array.rs:4463` `emit_array_define_named_accessor_descriptor` | name + `emit_validate_array_named_descriptor(.., false, ..)` at `:4544` | `PropertyDescriptorKind::Accessor` |

The last two are the LN6 flip, and §6.5's trace
(`15.2.3.6-4-199.js`) is what proves the instruction before another batch
executes it. Two sites in `array.rs` construct the word
`ARRAY_DESCRIPTOR_OWN_PROPERTY | OBJECT_DESCRIPTOR_ACCESSOR` directly
(`array.rs:3859`, `standard.rs:3301`); those become
`DescriptorWord::of_accessor(..).with_flags(DescriptorFlags { array_own_property: true, mapped: None })`.

**(b) The three `lowering.rs` shape sites (LN8), and they get three *different*
replacements.**

| Site | Role | Descriptor is | Replacement |
|---|---|---|---|
| `lowering.rs:26177` | `Object.getOwnPropertyDescriptor` result | **complete** (10.1.6.1 step 3) | The two 4-key shapes, selected by the stored kind. Where the kind is unknown, `heap_shape: None` — **not** a six-key union |
| `lowering.rs:27816` | Proxy `defineProperty` trap **argument** | **partial** (10.5.6 step 7 of a caller's partial `Desc`) | **`heap_shape: None`.** §1.8: a 4-key shape here is as false as a 6-key one |
| `lowering.rs:28453` | same, other entry point | same | **`heap_shape: None`** |

and `property_descriptor_shape(fields: Vec<(&'static str, ValueInfo)>)`
(`:25598`) → `fn property_descriptor_shape(fields: Vec<(DescriptorField, ValueInfo)>)`,
with `generic_property_descriptor_shape()` (`:25611-25626`) **deleted**, since
after the three replacements it has no call site. The three current
`property_descriptor_shape` call sites (`:25618`, `:26120`, `:26167`) carry three
different key sets today; `:25618` disappears with `generic_…`, and `:26120`
(species: get/set/enumerable/configurable) and `:26167` (the correct
`match`-derived pair) both become `DescriptorField` lists.

**(c) `emit_validate_array_named_descriptor`'s classification parameter (LN6)**,
and its hand-rolled `IsGenericDescriptor` at `:1504-1526`.

**(d) 6.2.6.5's field-order table (LN2 / I15).** `objects.rs:11962-11999` is
outside this lane's declared region (`~45-72`, `~1446-1650`, `~13498-14100`).
The note carries: replace the six-element array literal with
`TO_PROPERTY_DESCRIPTOR_ORDER` and a `DescriptorField::key()` call, keeping the
loop body byte-identical.

**(e) The mapped-slot readers (§4.6).**

**(f) `porffor-runtime` (LN4, §5.8).**

### 4.6 The `MappedSlot::SHIFT` readers

`functions.rs:6986-6988` and `:7071-7073` each read the mapped slot with a bare
`I64Const(32); I64ShrU`, and `functions.rs:6404` writes it with a bare
`<< 32`. `functions.rs` is unowned. The note instructs: replace all three with
`MappedSlot::SHIFT` and `MappedSlot::packed()`. Until then, the `const _: () =
assert!(MappedSlot::SHIFT == 32)` in `heap.rs` is what keeps the two sides from
drifting — an assertion whose whole job is to fail if someone changes the
constant without changing the three literals.

### 4.7 What stays untouched, said out loud

- **`emit_create_data_property_or_throw`** (`objects.rs:14374-14674`), and in
  particular its `iterator_close_on_throw: Option<IteratorCloseOnThrowLocals>`
  parameter at `:14383`. This is batch 5's contention surface in this file. It
  builds a descriptor object and calls `defineProperty`; it is a *caller* of the
  descriptor machinery and it is **not** in this lane.
- **`builtins/intl_datetimeformat.rs`, `temporal*.rs`, `emitted_function.rs`,
  `runtime_helpers.rs`** — batch 2's hold list. `intl_datetimeformat.rs` has 2
  `DESCRIPTOR_KIND_OFFSET` references and 2 `ARRAY_DESCRIPTOR_NORMAL_DATA`
  references, which is exactly why step 4 is additive.
- **`builtins/host.rs`** (9 `emit_object_define_accessor` call sites) and
  **`crates/porffor-aot-wasm/src/functions.rs`** (2 more, plus
  `object_data_descriptor_kind` at `:3017`, plus the three mapped-slot sites).
  Signatures frozen; §4.3, §4.6.
- **Proxy `[[DefineOwnProperty]]` invariant checks (10.5.6 steps 10–16).**
  Declared out of scope by the brief and by this contract. The *shape* claims at
  `lowering.rs:27816`/`:28453` are note-routed (§4.5b); the invariant checks are
  not touched at all.
- **`porffor-spec-exec`.** No descriptor work this round; the crate is named in
  the campaign scope but has no site in this area.

### 4.8 Region disjointness inside `objects.rs`

| Region | Lines | This lane |
|---|---|---|
| `object_*_descriptor_kind` | 45–72 | **owns** |
| `emit_object_append_*_property_with_flags` | ~1200–1444 | **reads only** (§5.1 evidence); the `object_*_descriptor_kind` call sites at `:1250`, `:1286`, `:1403` are unchanged text |
| `emit_validate_array_named_descriptor` | 1446–1650 | **owns**, body only |
| `emit_tagged_payload_same_value_i32` call site | 1583 | **reads only** (LN1) |
| 6.2.6.5 field loop | 11925–12020 | **reads only**; note-routed (§4.5d) |
| `emit_object_define_*` family | 13398–13531 | **owns** |
| `emit_object_define_entry` | 13533–14372 | **owns** |
| `emit_create_data_property_or_throw` | 14374–14674 | **must not touch** |

Total owned span: `45–72` (28) + `1446–1650` (205) + `13398–14372` (975) =
**1,208 of 20,865 lines (5.8%)**, and the largest contiguous owned block ends **2 lines before**
`emit_create_data_property_or_throw` begins.

---

## 5. Deviations from the area brief, with evidence

### 5.1 M5 is latent and **not reached**; the brief says "LATENT AND LIVE"

The brief states that `objects.rs:13574-13579` "classifies every value-less
descriptor as an accessor". True. It then states M5 is live. Measured, it is
not, and the reason is worth recording because it is fragile.

`descriptor_kind = if has_data { DATA } else { ACCESSOR }` is wrong only when
the descriptor is *generic*, i.e. `data`, `getter` and `setter` are all `None`.
That reaches `emit_object_define_entry` only through
`emit_object_define_accessor_with_flag_local` (`:13498`), whose three callers
are `objects.rs:1418`, `:13456`, `:13484`. Following each:

- **`:13456`** — `emit_object_define_accessor(object, key, getter, setter)`, 11
  call sites, **all in unowned files**, each read: `host.rs:4402, 4512, 4588,
  4794, 4834, 5578, 5662, 5755, 6459` and `functions.rs:986, 1009`. Nine pass
  `Some(getter), None`; two pass `Some, Some`; one (`functions.rs:1009`) passes
  `None, Some(setter)`. **None passes both `None`.**
- **`:13484`** — `emit_object_define_enumerable_accessor`, 4 call sites, all in
  `objects.rs` (`:2585, 2611, 2643, 2664`), each read: one `Some, <forwarded>`
  and three with exactly one `Some`. **None passes both `None`.**
- **`:1418`** — inside `emit_object_append_accessor_property_with_flags`. Its
  parameters `getter`/`setter` *are* `Option`, so both could be `None` — except
  that lines **`1328-1359`** rebind both to `Some(..)`, materialising
  `ValueKind::Undefined` locals for whichever is missing:

  ```rust
  let getter = getter.or_else(|| Some((
      missing_getter_payload_local.expect("missing getter payload local"),
      missing_getter_tag_local.expect("missing getter tag local"),
  )));
  ```

  So by `:1418`, both are unconditionally `Some`.

**Corroborating evidence that the tree already knows this is a hazard:** the
*other* branch of that same function, at `:1362-1365`, discharges it with a
runtime panic —

```rust
let (getter_payload_local, getter_tag_local) = getter.expect("getter locals must be materialized");
let (setter_payload_local, setter_tag_local) = setter.expect("setter locals must be materialized");
```

— two `.expect()`s that are, after `:1348-1359`, unreachable. One branch panics
on a state the other branch silently miscompiles. Under
`DescriptorClassification`, the `:1418` branch acquires a `Generic` arm and the
`.expect()`s become provably dead.

**Consequence for the encoder:** do not write a test expecting a wrong answer
from M5 today. The invariant's value is prospective, and §6.3's trace (A3) is a
*Rust-level* trace, not a JavaScript one. This matches the brief's own framing
of A3.

### 5.2 The `standard.rs:11572` "contradictory pair" — the naive fix **deletes a required check**

This is the most consequential correction in this document.

The brief says: "`builtins/standard.rs:11572` passes `data: None` together with
`data_present_local: Some(..)`, so the static classification says ACCESSOR while
the runtime one says maybe-data", and asks for the pair to be made
unspellable. It is unspellable under `Presence` — but the *obvious* translation,
`value: Presence::Absent`, silently removes two required 10.1.6.3 step-4 checks.

**Why.** `objects.rs:13726-13777` is guarded by

```rust
if data_present_local.is_some()
    && writable_present_local.is_some()
    && getter_present_local.is_some()
    && setter_present_local.is_some()
```

and its body emits **two independent throws**:

- `:13731-13752` — `(getter_present || setter_present) && existing is DATA`
  → TypeError. This is **10.1.6.3 step 6.a**: converting a non-configurable
  **data** property to an accessor must fail. **It does not read
  `data_present_local` at all.**
- `:13754-13776` — `(data_present || writable_present) && existing is ACCESSOR`
  → TypeError. This is **step 7.a**, the mirror.

At `standard.rs:11572` (the accessor branch) the first throw is **live and
required**; the second is runtime-unreachable, because `standard.rs:11364-11380`
has already thrown if `value_present || writable_present`. Setting
`value: Presence::Absent` makes `data_present_local.is_some()` false, the whole
block disappears, and **step 6.a stops being emitted**.

That would be an immediate, reachable conformance regression:
`8.12.9-9-b-i_1.js` (§6.4) redefines a **non-configurable** data property as an
accessor and requires a `TypeError`.

**The contract's resolution, and it is the reason M5′ exists as a mistake
class.** The four-way conjunction is not one obligation; it is two, and each
needs only *its own side's* terms. Under `DescriptorClassification::Dynamic`:

```rust
match classify(&descriptor) {
    DescriptorClassification::Static(PropertyDescriptorKind::Data)     => { /* step 7.a only */ }
    DescriptorClassification::Static(PropertyDescriptorKind::Accessor) => { /* step 6.a only */ }
    DescriptorClassification::Static(PropertyDescriptorKind::Generic)  => { /* neither: step 4.c exempts generic */ }
    DescriptorClassification::Dynamic { accessor_terms, data_terms } => {
        // step 6.a, from `accessor_terms` alone
        self.emit_kind_change_throw(DescriptorSide::Accessor, &accessor_terms, existing, function)?;
        // step 7.a, from `data_terms` alone
        self.emit_kind_change_throw(DescriptorSide::Data, &data_terms, existing, function)?;
    }
}
```

At `standard.rs:11572` with `value: Absent, writable: Absent, get: Runtime,
set: Runtime`, `classify` returns `Dynamic { accessor_terms: { statically_true:
false, runtime: [Some(getter_present), Some(setter_present)] }, data_terms: {
statically_true: false, runtime: [None, None] } }`. The step-6.a throw is
emitted from `accessor_terms` — **unchanged bytes**. The step-7.a throw has no
terms, so it emits nothing — **and that is the byte delta**: the 15
`function.instruction` calls at `:13754-13776` plus the expansion of the
`emit_throw_runtime_error` / `emit_return_current_completion` pair inside them,
all of which are provably runtime-dead given the guard at
`standard.rs:11364-11380`.

**Dry-run obligation, and it is mandatory before this lands (§6.3, §7):**
enumerate every instruction the rung-G diff shows at these two call sites and
show each is in the provably-dead set. The expected sets are:

| Site | Instructions expected to disappear | Why dead |
|---|---|---|
| `standard.rs:11572` | `:13754-13776` (step 7.a throw) | `standard.rs:11364-11380` already threw if `value_present \|\| writable_present` |
| `standard.rs:11572` | `:13778-13801` (step 4.e writable check) | same |
| `standard.rs:11572` | `:13802-13853` (step 4.e `SameValue`) | same |
| `standard.rs:11572` | `:13941-13963` (re-read stored data) | result stored only under the `descriptor_kind & ACCESSOR == 0` branch at `:14066`, which is false here |
| `standard.rs:11905` | `:13731-13752` (step 6.a throw) | this is the `Else` of `standard.rs:11363`, so `getter_present == setter_present == 0` |
| `objects.rs:13511` | `I64Const(0); LocalSet(writable_payload_local)` at `:13509-13510` | §4.4 |

**If the dry run cannot discharge every row, the encoder must keep the byte and
route the simplification to the note.** A byte the dry-runner cannot account for
is a defect, not a cleanup.

### 5.3 The derivation count is **nine**, not eight, and the array/arguments count is **eight**, not six

The area title says "re-derived at eight sites". Measured (§1.4): **nine**
spellings of the partition, in three crates, one of which
(`lowering.rs:26130-26163`) is correct. The brief's "six derivation sites in
`builtins/array.rs` and `builtins/standard.rs`" measures as **eight**:
`standard.rs:2454, 2741, 3007, 3432` and `array.rs:3709, 3821, 4062, 4463`.
Both counts are used in §4.5's note table; neither changes the design.

### 5.4 A mask is not a word — the constraint the brief's scope item (3) would break

Brief scope item (3): "the illegal word `OBJECT_DESCRIPTOR_ACCESSOR |
OBJECT_DESCRIPTOR_WRITABLE` (= 5, today a perfectly representable stored value
…) has no constructor."

Correct as stated about *stored values*. But `objects.rs:1560-1565` computes

```rust
I64Const((OBJECT_DESCRIPTOR_ACCESSOR | OBJECT_DESCRIPTOR_WRITABLE) as i64); I64And; I64Eqz;
```

which is the **mask** for "existing is a data descriptor and is not writable" —
correct, and one instruction cheaper than two separate tests. An implementation
that banned the bit pattern `5` outright would have to rewrite this into two
`I64And`s, changing emitted bytes for no reason, or would tempt the encoder to
add a `DescriptorWord::from_bits_unchecked` escape hatch that reopens M1.

Hence I8: `DescriptorMask` is a **separate newtype** with no conversion to or
from `DescriptorWord`, and `ACCESSOR_OR_WRITABLE` is one of its named
constants. This constraint is not in the brief and it is the difference between
a landable retrofit and one that fights the existing code.

### 5.5 `namespace.rs` has **three** builders, and it is a `ToPropertyDescriptor`-domain consumer

The brief names "four builders (`:349`, `:372`, `:743`, and the assembly at
`:1109-:1126`)". Measured: `:348-368`, `:371-378`, `:742-749` are builders;
`:1107-1132` is `#[test] fn
namespace_source_creates_a_null_prototype_non_extensible_object`, which asserts
on the *output* of the builders. It is an oracle, not a fourth builder — and a
useful one, since it pins the `preventExtensions`-last ordering that §4.2 must
not perturb.

Separately, the brief describes this as "the owned consumer of the
FromPropertyDescriptor key domain". It is not: these are `defineProperty`
**arguments**, i.e. 6.2.6.5's domain, and the `:348-368` builder emits a legal
**three-key** partial descriptor (`get`/`enumerable`/`configurable`) that no
four-key codomain type would accept. §1.8, §1.9. The type it actually consumes
is `PartialDescriptor<SourceText>` over the shared six-name field domain — which
is the right type anyway, and is what makes I14 a real compile error (a misspelt
key in source text is otherwise **completely invisible**: 6.2.6.5 ignores stray
keys, so `configurabel: false` yields a non-configurable property by default and
the program behaves correctly for the wrong reason).

### 5.6 `lowering.rs:26130-26163` is **already correct** and the retrofit must not "unify" it

```rust
let fields = match property {
    ObjectShapeProperty::Data(value) => vec![("value", value), ("writable", ..), ("enumerable", ..), ("configurable", ..)],
    ObjectShapeProperty::Accessor { getter, setter } => vec![("get", ..), ("set", ..), ("enumerable", ..), ("configurable", ..)],
};
```

This is an exhaustive two-arm match over a closed enum producing exactly the two
legal four-key sets. It is the shape §2.6's `CompleteDescriptor::keys()`
generalises, and the retrofit's only change here is `&'static str` →
`DescriptorField` (I12). Do not fold it into the generic path; the generic path
is the one that is wrong.

### 5.7 M6 is **inert today**, and the reason is a guard in a different crate

The brief calls `generic_property_descriptor_shape()` "a false assertion about
the heap, not a conservative one", and asks which fold it licenses. Traced end
to end:

1. `read_own_object_shape_property` (`lowering.rs:35796-35808`) returns the
   shape's `ValueInfo` for a named key, and `None` for an absent one, which
   falls back to `ValueKind::Dynamic` / `KindSet::all_runtime_tags()`.
   The six-key shape's `value`, `writable`, `get`, `set` all carry exactly that
   `Dynamic`/all-tags info (`:25612-25617`), so those four keys are
   indistinguishable from absent. **No fold.**
2. `enumerable` and `configurable` carry `boolean_value_info()`
   (`:25628-25630`) = `ValueInfo::new(ValueKind::Boolean)`, whose
   `possible_kinds` is the **singleton** `{Boolean}`. That *is* a real claim.
3. The consumer that acts on a singleton is
   `compile_strict_equality_i32` (`operations.rs:10771-10787`):

   ```rust
   if !lhs_tag_dynamic && !rhs_tag_dynamic
       && lhs.possible_kinds.is_singleton() && rhs.possible_kinds.is_singleton()
       && lhs.kind != rhs.kind
   { function.instruction(&Instruction::I32Const(0)); return Ok(()); }
   ```

   So inside a Proxy `defineProperty` trap, `desc.enumerable === undefined`
   would fold to `false` — while at run time, `Object.defineProperty(proxy,'x',{value:1})`
   gives the trap a `descObj` with **no** `enumerable` key (§1.8), so the
   correct answer is `true`. That would be a live wrong answer.
4. **It does not fire.** `lhs_tag_dynamic` is
   `expr_result_tag_is_runtime_dynamic(&lhs.expr)`, and
   `planning.rs:6178` lists `ExprIr::PropertyRead { .. }` among the arms
   returning `true`. `desc.enumerable` is a `PropertyRead`. The guard is
   `!lhs_tag_dynamic`, so the fold is skipped.
5. **Every** `is_singleton()` consumer in the backend is paired with the same
   guard. Measured: 19 `is_singleton()` occurrences workspace-wide;
   the backend ones are `expressions.rs:149, 2680`,
   `operations.rs:387, 3853, 3882, 7330, 10781, 10791, 10901, 10939`,
   `control_flow.rs:2057`, and each is conjoined with
   `expr_result_tag_is_runtime_dynamic` or with `emits_own_dynamic_result`.

**And the tree has shipped this exact defect before.** `operations.rs:7325-7331`
carries the comment:

> "…a call whose inferred kind was a (wrong) singleton got constant-folded into
> a literal type name instead of re-reading the runtime tag."

So the six-key shape is a false assertion held harmless by a defence-in-depth
guard in another crate, whose companion defect has already shipped once. That is
a strong reason to fix it and a poor reason to call it urgent. The mistake class
stays; the claim "LIVE" becomes "constructed, inert, one guard away".

**Consequence for the dry-runner:** adversarial A4
(`Object.getOwnPropertyDescriptor({get x(){return 1}},'x').writable` must be
`undefined`) will **pass today**. Do not treat a pass as evidence the shape is
correct. The check that actually discriminates is the *shape*, not the program:
after the retrofit, `generic_property_descriptor_shape` must have **no call
site**, which is a compile error (`dead_code`) rather than a test.

### 5.8 `porffor-runtime`'s second model: the declared decision is **no dependency edge**, and the reason is that the model is **unconsumed**

Measured facts, all verified:

- `crates/porffor-runtime/Cargo.toml` has **no `[dependencies]` section at
  all**. It is a leaf crate.
- `porffor-ir` depends on `boa_ast`, `boa_interner`, `boa_parser`,
  `icu_properties`, `num-bigint`, `num-traits`, `porffor-front`, `regress`,
  `serde_json` — including a **JavaScript parser**.
- `porffor-runtime` is depended on by exactly **one** crate: `porffor-engine`.
- `IntrinsicPropertyAttributes` has **13 references**, of which **12 are inside
  `porffor-runtime/src/lib.rs`** (2 of those in `#[cfg(test)]` at `:1167` and
  `:1226`) and **1 is a `pub use` re-export** at `porffor-engine/src/lib.rs:984`.
- `INTRINSIC_PROPERTY_DESCRIPTORS` (46 rows) is likewise only re-exported
  (`porffor-engine/src/lib.rs:987`). **No backend reads it.**
- The AOT backend derives the same spec fact independently, at
  `functions.rs:3017`:
  `object_data_descriptor_kind(false, false, meta.length_name_configurable)` —
  which is 10.2.x's `{[[Writable]]: false, [[Enumerable]]: false,
  [[Configurable]]: true}` for a builtin function's `length` and `name`, exactly
  what `IntrinsicPropertyAttributes::BUILTIN_FUNCTION_LENGTH_NAME_CONFIGURABLE`
  (`:167-171`) says.

**The decision.** Do **not** give `porffor-runtime` a dependency on
`porffor-ir`. Buying a `boa_parser` edge for a leaf table crate, in order to
share a three-`bool` struct that no consumer reads, inverts the dependency
graph for no correctness gain.

**And do not "tie the two by const assertion" either**, because a const
assertion cannot cross a crate boundary in the absence of a dependency, and the
only crate that can see both types is `porffor-engine` — which this lane does
not own. Placing the tie there is a real option; it is **note-routed**, not
performed here.

**Why not simply make the second model safer in place?** Because in Rust,
privacy is module-scoped: making `IntrinsicPropertyAttributes`'s fields private
blocks nothing, since all 46 rows are built by `const fn`s in the same module
(`:429-483`) and the tests are a descendant module. There is **no plausible
mistake that becomes a compile error** from any edit confined to
`porffor-runtime/src/lib.rs`. Per AGENTS.md — "If it does not, the type is
decoration and a plain function is better" — the honest answer is a ledger row.

**LN4, in full.** `porffor-runtime`'s `IntrinsicPropertyAttributes` and its 46
`INTRINSIC_PROPERTY_DESCRIPTORS` rows are a **second, parallel, currently
unconsumed** encoding of the same spec facts the AOT backend derives at
`functions.rs:3017` and at 4 other sites. They are data-only, have no accessor
case, and cannot express field presence. Two follow-ups, both note-routed, in
preference order:

1. **Preferred: delete.** The table has no product-path consumer. AGENTS.md:
   "If something is unreachable from the product path, that should fail to
   build, not merely fail to run. Code with no call site has been written here
   more than once; it compiled … because it was `pub`." Deleting requires
   editing `porffor-engine/src/lib.rs:984, 987`.
2. **If a consumer is imminent: tie in `porffor-engine`.** A `const _: () =
   assert!(...)` there can see `porffor_runtime::IntrinsicPropertyAttributes`
   and `porffor_ir::property_descriptor::CompleteDescriptor` simultaneously, and
   can pin `BUILTIN_FUNCTION_LENGTH_NAME_CONFIGURABLE` against the canonical
   10.2.x triple.

Until one of those lands, the divergence risk is real and is recorded here with
its exact content: **two independent statements of "a builtin function's
`length` and `name` are non-writable, non-enumerable, and configurable iff
`length_name_configurable`", 3 crates apart, with nothing checking they agree.**

### 5.9 `porffor-ir/src/lib.rs` vs `ir.rs` — following ownership, against local convention

The two sibling theory-first modules already in this crate declare themselves
**inside `ir.rs`**: `pub mod reference;` at `ir.rs:23`, `pub mod
numeric_conversions;` at `ir.rs:34`. Because `lib.rs:84` is `pub use ir::*;`,
both are reachable as `porffor_ir::reference::…` and
`porffor_ir::numeric_conversions::…`.

This contract places `mod property_descriptor;` in **`lib.rs`** instead, because
`ir.rs` is not in this area's `files_owned` and `lib.rs` is (line-scoped). The
public path is then `porffor_ir::property_descriptor::…` via the appended
`pub use`, which is the same shape from a consumer's point of view.

**Merge hazard, stated so the integrator sees it:** other lanes append to the
same two regions of `lib.rs` (the `mod` block at `:56-78` and the `pub use`
region after `:151`). The edit must be exactly two additions, no reflowing, no
re-sorting. If a later batch normalises module declarations into `ir.rs`, this
module moves with them; nothing else changes.

---

## 6. Dry-run corpus, with the traces the dry-runner must reproduce

All 11 test262 paths verified present at the current pin under
`test262/vendor/test262/test/`. Each trace is a **symbolic execution of the
named spec steps against the named lines**, on paper. Actual execution is out of
scope for this campaign and belongs elsewhere.

### 6.1 `built-ins/Object/defineProperty/15.2.3.6-4-52.js` — M5, the sharpest trace

Source (read in full): `var obj = {}; Object.defineProperty(obj, "property", {});`
then `verifyProperty(obj, "property", {value: undefined, writable: false,
enumerable: false, configurable: false})`.

Spec path: 6.2.6.5 yields the **empty** descriptor (all six absent) — no
TypeError, since step 9's antecedent is false. 10.1.6.3 step 2: `current` is
`undefined`, `extensible` is true, so step 2.d. IsAccessorDescriptor is
**false** (no `[[Get]]`, no `[[Set]]`), so step 2.d.ii creates a **data**
property, and 6.2.6.6 supplies `value: undefined, writable: false, enumerable:
false, configurable: false`.

**Trace obligation, today.** Follow it to `standard.rs`: no get/set, so the
guard at `:11356-11363` is false and control reaches the data branch,
`emit_object_define_entry` at `:11905` with `data: Some((value_payload_local,
value_tag_local))`. `has_data` is true at `objects.rs:13572`, so
`descriptor_kind = OBJECT_DESCRIPTOR_DATA`. The correct answer, reached through
the *wrong reason*: it is a data property because a value **operand** was
supplied, not because IsAccessorDescriptor is false. Confirm that
`value_payload_local`/`value_tag_local` hold `undefined` when
`value_present_local` is 0 — `objects.rs:12000-12003` initialises every field's
payload to `0` and tag to `ValueKind::Undefined` **before** the HasProperty
probe, so they do.

**Trace obligation, after.** `classify` returns
`Dynamic { accessor_terms: [getter_present, setter_present], data_terms:
[value_present, writable_present] }` (this call site keeps all four as
`Runtime`), so the run-time kind decision is emitted, and at run time all four
flags are 0 → `Generic` → the new-entry path completes it to `Data` per 6.2.6.6
step 3. **Same answer, now for the spec's reason.** Show the emitted bytes are
unchanged.

**And the negative trace:** exhibit that
`emit_object_define_accessor_with_flag_local` with `getter = None, setter =
None` — the state §5.1 shows is unreachable but representable — today produces
`descriptor_kind = OBJECT_DESCRIPTOR_ACCESSOR` and an accessor entry with two
`undefined` accessors, and after the change is an **E0004** on the missing
`PropertyDescriptorKind::Generic` arm.

### 6.2 `built-ins/Object/defineProperty/15.2.3.6-4-82-1.js` — M2 and step 5

`foo` is defined `{value: 1001, writable: true, enumerable: true, configurable:
true}`, then redefined with the generic `{enumerable: false}`. Required result:
`{value: 1001, writable: true, enumerable: false, configurable: true}`.

Spec path: `current` is configurable, so step 4 is skipped. IsGenericDescriptor
is **true** → step 5 → step 8 sets only `[[Enumerable]]`. `[[Value]]`,
`[[Writable]]` and `[[Configurable]]` are **untouched**, not rewritten and not
defaulted.

**Trace obligation.** Through `objects.rs`: `has_data` is true (site
`standard.rs:11905` always passes value locals), so `descriptor_kind` starts at
`DATA`; `:13582-13592` does not OR in `WRITABLE` because
`writable_payload_local` is 0; `:13593-13612` does not OR in `ENUMERABLE`
(payload 0) or `CONFIGURABLE` (payload 0). The existing bits are recovered by
the three "if absent, re-read" blocks: `:13973-13997` (writable),
`:13998-14014` (enumerable — **but the incoming descriptor *has*
`[[Enumerable]]`, so `enumerable_present_local` is 1 and this block correctly
does nothing**), `:14015-14035` (configurable), and the stored value by
`:13941-13963`. Show that under `Presence`, each of those four becomes an arm of
a `match … { No | Yes | AtRuntime }` and that the emitted bytes are identical.

Then show the M2 failure mode explicitly: if any one of the four presence locals
were `None` — which is exactly the state `objects.rs:13422` passes — the
corresponding block emits **nothing**, and the attribute is written from the
incoming payload, i.e. `false`. Today that is safe only because `:13422`'s
caller supplies a complete descriptor. Under `Presence` the two states are
`AtRuntime` and `Present`, and neither is `Absent`.

### 6.3 ADVERSARIAL A3 — the disarming correction, at the Rust level

Two halves.

**(a)** `standard.rs:11572`'s `data: None` + `data_present_local:
Some(value_present_local)`. Show there is **no** `Presence` value with
`Absent`'s value-carrier and `Runtime`'s presence flag: `Runtime` requires
`value: T`. Then show that the correct translation is
`value: Presence::Absent` — **and** that per §5.2 this drops
`objects.rs:13754-13776`, `:13778-13801`, `:13802-13853` and `:13941-13963`,
and account for every one of those instructions as runtime-dead given
`standard.rs:11364-11380`. **This is the single most important dry-run
obligation in the corpus.** If any row cannot be discharged, the byte stays.

**(b)** A hypothetical future call site passing `data: None, data_present_local:
None` for a statically-known-absent `[[Value]]`. Today: `has_data` is false at
`:13572` → `ACCESSOR` at `:13575-13579`; **and** the correction at `:14036`
requires all four presence locals `Some`, so it does not emit; **and** the
step-6.a/7.a throws at `:13726` likewise do not emit. Three silent omissions
from one `None`. After: `classify` yields `Static(Generic)` and the consumer
`match` has no `Generic` arm → **E0004**.

### 6.4 `built-ins/Object/defineProperty/8.12.9-9-c-i_1.js` and `8.12.9-9-b-i_1.js` — M1, both directions

`8.12.9-9-c-i_1.js`: a configurable **accessor** redefined as a **data**
property (10.1.6.3 step 7). Only `[[Enumerable]]` and `[[Configurable]]` carry
over; `[[Writable]]` takes its 6.2.6.6 default, `false`. A surviving writable
bit on the accessor entry is the illegal word.

`8.12.9-9-b-i_1.js`: the mirror, data → accessor (step 6.b).

**Trace obligation.** Show that `objects.rs:13973-13997` — 21 instructions
under the comment at `:13965-13972` — carries the whole step-7 obligation
today, and that it is guarded by `if has_data { if let Some(writable_present) {
… } }`, i.e. by **two** conditions neither of which is the spec's. Then show
that under I11 the accessor arm is
`DescriptorWordEmitter<Accessor>`, on which `set_writable_if` does not exist
(E0599), so the entry cannot acquire the bit in the first place and the 25
instructions have nothing to repair.

Also confirm the **`8.12.9-9-b-i_1.js` direction is what proves `Accessor { get,
set }` having no `writable` field is *sufficient*, not merely convenient**: with
no such field, step 6.b's "preserve only `[[Enumerable]]` and
`[[Configurable]]`" is the *only* thing `CompleteDescriptor::Accessor` can be
built from.

### 6.5 `built-ins/Object/defineProperty/15.2.3.6-4-199.js` — M5 through the array path (note-routed)

`Object.defineProperty([], "0", {enumerable: true})` must define a **data**
property `{value: undefined, writable: false, enumerable: true, configurable:
false}` — 10.4.2.1 → 10.1.6.3 step 2.d.ii via 6.2.6.6 step 3.

**Trace obligation.** Follow it to `emit_validate_array_named_descriptor`
(`objects.rs:1446`) and `emit_array_define_data_index`
(`array.rs:3709`). Show that `:1504-1526`'s `kind_present_local` fold is a
hand-rolled `IsGenericDescriptor`, that it correctly skips the kind check at
`:1527-1537` when the descriptor is generic, and that
`requested_data_descriptor: bool` at `:1450` therefore has **no bearing** on the
generic case — which is why the array path gets the right answer today and the
ordinary-object path (§6.1) gets it for the wrong reason.

This trace exists to prove the note's retrofit instruction for `array.rs:4062`
and `:4463` (§4.5a) **before** another batch executes it. Half this path is
note-routed; the trace is the whole deliverable for that half.

### 6.6 `built-ins/Object/defineProperty/15.2.3.6-4-131.js` — LN1, first direction

`Object.defineProperty(arrObj, "length", {value: -0})` must **not** throw
`RangeError` and must leave `length` at `0`. 10.1.6.3 step 4.e compares with
`SameValue`: `SameValue(-0, +0)` is **false**, `F64Eq(-0, +0)` is **true**. So
here the *wrong* operator is the *permissive* one.

**Trace obligation.** Confirm `objects.rs:1583` and `:13834` both call
`emit_tagged_payload_same_value_i32`, and record that nothing in the type system
distinguishes that call from `emit_tagged_payload_equality_i32` — which is
called at `:13877` and `:13920` for `[[Get]]`/`[[Set]]` and is **correct
there** (step 4.d, object operands). Two adjacent obligations, two different
correct operators, one helper signature. That is LN1's justification, verbatim.

### 6.7 `built-ins/Object/defineProperty/15.2.3.6-3-25.js` — LN2

The `enumerable` field of the Attributes object is an own data property
shadowing an **inherited accessor**. 6.2.6.5 reads each field with HasProperty
then Get, both of which walk the prototype chain, so the inherited getter is
reachable in general and shadowed here.

**Trace obligation.** Confirm `objects.rs:11962-12014` loops the six fields in
6.2.6.5 order and calls `emit_object_has_property_with_key_tag_i32`, not an
own-slot probe. Then state the residual: I15 makes dropping or reordering a
field a compile error; the choice of *read primitive* is a method call with the
same signature as its wrong alternative. LN2.

### 6.8 `built-ins/Object/getOwnPropertyDescriptor/15.2.3.3-4-1.js` — M6, positive direction

The descriptor returned for a **data** property must carry exactly
`value`/`writable`/`enumerable`/`configurable`.

**Trace obligation.** `lowering.rs:26131-26136` already produces exactly this
four-key list. Show `CompleteDescriptor::keys()`'s `Data` arm reproduces it
element for element, in the same order.

### 6.9 `built-ins/Object/getOwnPropertyDescriptor/15.2.3.3-4-239.js` — M6, the direction that kills the six-key shape

The returned descriptor's `get` must itself be a **data** property with the
right value — so a descriptor for an accessor has `get`/`set` and has **no
`writable` key at all**.

**Trace obligation.** `lowering.rs:26137-26162` produces the correct four-key
accessor list. Show that `:26177`'s fallback — reached when
`read_own_object_shape_property` returns `None` — claims **six** keys,
i.e. asserts `writable` present on a descriptor the spec says has no such key,
and that this is a *false* claim rather than a conservative one. Per §5.7 it is
currently inert; the check that the fix landed is that
`generic_property_descriptor_shape` has no call site and therefore fails to
build.

### 6.10 ADVERSARIAL A4 — `'writable' in Object.getOwnPropertyDescriptor({get x(){return 1}},'x')`

Must be `false`, and `.writable` must be `undefined`.

**Trace obligation, and read §5.7 first: this passes today.** The six extra keys
in `generic_property_descriptor_shape` carry `ValueKind::Dynamic` /
`all_runtime_tags`, which is identical to the absent-key fallback, and the one
fold that could act on a singleton is disarmed for `PropertyRead` by
`planning.rs:6178`. The dry-runner must record **why** it passes, not merely
that it does — and must additionally exhibit the *Proxy* variant, which is the
one the shape is actually wrong about:

```js
var log = [];
var p = new Proxy({}, {
  defineProperty(t, k, desc) { log.push(desc.enumerable === undefined); return Reflect.defineProperty(t, k, desc); }
});
Object.defineProperty(p, 'x', { value: 1 });   // Desc has no [[Enumerable]]
// log[0] must be true
```

10.5.6 step 7 gives the trap `FromPropertyDescriptor(Desc)`, which per §1.8 has
**one** key. `lowering.rs:27816`/`:28453` claim six, including `enumerable:
Boolean` — a singleton. Show the fold at `operations.rs:10779-10787` would emit
`I32Const(0)` if `lhs_tag_dynamic` were false, and that it is true because
`desc.enumerable` is an `ExprIr::PropertyRead`. That is the whole distance
between "inert" and "wrong answer", and it is one `planning.rs` arm wide.

### 6.11 ADVERSARIAL A1 — M1 as a two-line program

```js
var o = {};
Object.defineProperty(o, 'x', { get() { return 1 }, configurable: true });
Object.defineProperty(o, 'x', { value: 2 });
Object.getOwnPropertyDescriptor(o, 'x');   // {value:2, writable:false, enumerable:false, configurable:true}
```

**Trace obligation.** At the Rust level, exhibit that no sequence of calls to
`DescriptorWord::of_data` / `::of_accessor` / `::with_flags` produces a word
with both bit 0 and bit 2 set, and that
`DescriptorWordEmitter<Accessor>` has no method that can set bit 2. Then show
that `objects.rs:13973-13997` becomes **deletable** — not merely commented —
because the `Accessor` arm cannot mention writability and the `Data` arm's
`set_writable_if` is the spec's step 8, not a repair.

### 6.12 ADVERSARIAL A2 — M2 as a program, traced through `PartialDescriptor`

```js
var o = {};
Object.defineProperty(o, 'x', { value:1, writable:true, enumerable:true, configurable:true });
Object.defineProperties(o, { x: { enumerable:false } });
Object.getOwnPropertyDescriptor(o, 'x').writable;   // must still be true
```

**Trace obligation.** Trace it through the single `PartialDescriptor` value
rather than through 15 positional parameters: `writable` is
`Presence::Runtime { present: writable_present_local, .. }` with
`writable_present_local == 0` at run time, so the step-8 application must not
touch `[[Writable]]` and the `:13973-13997` carry-over must fire. Show the
`match` arm structure that replaces the nested
`if has_data { if let Some(present) { … } }`.

### 6.13 `language/module-code/namespace/internals/define-own-property.js` and `language/arguments-object/mapped/nonconfigurable-descriptors-define-failure.js`

The first exercises module-namespace exotic `[[DefineOwnProperty]]`, whose
descriptors this tree builds as **source text** in `modules/namespace.rs`
(§1.9). Trace obligation: show `DescriptorSourceText::render()` reproduces all
three builders byte for byte (§4.2's table), and that a misspelt key is E0599
rather than a silently-ignored property that leaves the attribute at its 6.2.6.6
default.

The second exercises a mapped arguments element whose descriptor round-trips
through the **same** `u64` word that carries `ARGUMENTS_DESCRIPTOR_MAPPED` (bit
5) **and the mapped slot index in bits 32..63** (§1.7). Trace obligation: show
`DescriptorFlags { mapped: Some(MappedSlot::new(n)) }` reproduces
`functions.rs:6404`'s `ARGUMENTS_DESCRIPTOR_MAPPED | ((n as i64) << 32)`
exactly, that the three `const _` disjointness assertions in §2.7 hold, and that
`DescriptorWord::of_data(..)` alone can never set bits 4, 5, or 32..63.

### 6.14 ADVERSARIAL A6 — LN1, the opposite direction

On a non-configurable, non-writable property whose value is `NaN`,
`Object.defineProperty(o, 'x', { value: NaN })` must **not** throw, because
`SameValue(NaN, NaN)` is `true` while `F64Eq(NaN, NaN)` is `false`.

Together with §6.6 this pins the operator from both sides: `-0`/`+0` makes
`F64Eq` too permissive, `NaN` makes it too strict. **No descriptor type
distinguishes them**, which is the honest reason M3 is ledgered rather than
typed. State that conclusion explicitly in the dry-run record; it is the
justification for LN1 and a future reader will otherwise try to type it away.

### 6.15 ADVERSARIAL A5 — M7 at all three `property_descriptor_shape` call sites

`Self::property_descriptor_shape(vec![("writeable", dynamic)])` must fail to
compile. Trace at `lowering.rs:25618` (six keys, deleted by the retrofit),
`:26120` (`get`/`set`/`enumerable`/`configurable`) and `:26167` (the
`match`-derived pair) — three different key sets today, one closed domain after.
Note that this half is **note-routed** (LN8); the same E0599 lands this round in
`modules/namespace.rs` via I14, which is what stops `DescriptorField` from being
dead `pub` code in the interim.

---

## 7. Acceptance criteria

The encoder's work is done when all of the following hold. Each is checkable
without a conformance run.

1. **Rung 0 is clean.** `cargo check -p porffor-ir` and `cargo check -p
   porffor-aot-wasm` produce zero errors, with **no edit** to any file outside
   this area's `files_owned` except the two identifier renames named in §4.4 —
   and those are applied by the integrator, not the lane.
2. **The heap change is additive.** All eight constants in §2.8 keep their
   names, types and values, and all eleven `const _: () = assert!(..)` lines in
   §2.7 and §2.8 are present. `builtins/intl_datetimeformat.rs` is untouched.
3. **No `_` arm over a descriptor domain.** `rg '_ =>'` in the same `match` as
   `PropertyDescriptorKind`, `DescriptorClassification`, `KnownPresence`,
   `DescriptorField` or `CompleteDescriptor` returns nothing, workspace-wide.
4. **`classify` is the only derivation.** Of the nine spellings in §1.4,
   1, 2, 5 and 6 are gone; 3, 4, 7, 8 are note-routed with per-site
   instructions; 9 is unchanged except `&'static str` → `DescriptorField`.
5. **The rung-G diff is fully accounted for.** It will **not** be empty. Every
   differing byte must map to a row of §5.2's dead-instruction table or to
   §4.4's three-instruction deletion. **A byte the dry-runner cannot account for
   is a defect, and the change does not land.** This is the reason §5.2 is
   mandatory reading: it is the difference between a justified 5-row diff and a
   silently dropped 10.1.6.3 step-6.a check.
6. **`namespace.rs` output is byte-identical**, evidenced by the eight existing
   substring assertions in that file (§4.2) passing unchanged.
7. **`generic_property_descriptor_shape` has no call site**, and therefore does
   not build — *when* LN8 is executed. This round it is note-routed, so the
   criterion for this round is that the note carries the three per-site
   replacements of §4.5b, including the fact that `27816` and `28453` become
   `heap_shape: None` and **not** a four-key shape.
8. **Every ledger row has a reason.** LN1–LN8 are stated in §2.0 with the reason
   a type cannot carry each. A row acquiring a type later is progress; a row
   without a reason is a defect in this document.
9. **Nothing in `emit_create_data_property_or_throw` changed.**
   `git diff --stat` shows no hunk overlapping `objects.rs:14374-14674`.
