# Contract: Environment Record binding lifecycle

**Area.** TDZ as a per-binding typestate on `BindingInfo`, replacing the parallel
string-set stack (`tdz_scopes`) and the storage-name prefix (`$tdz.`).

**Stage.** FORMALIZER. No source file was edited to produce this document. Every
count below was obtained by reading the tree at
`claude/test-driven-rust-opus-pp6giw`; none is an estimate. The encoder
implements §2, §4 and §6 verbatim; the dry-runner verifies against §3 and §7.

---

## §0. Measurement ledger, and where the area brief is wrong

Read this section first. Five of the brief's numbers and two of its structural
claims do not survive contact with the code, and three of the corrections change
what the encoder must build.

| # | Brief says | Tree says | Consequence |
|---|---|---|---|
| **0.1** | "44 measured `BindingInfo` struct-literal sites" | **41** struct literals. `grep -c 'BindingInfo {'` excluding `VarBindingInfo` returns 44, but three of those lines are not literals: the definition `pub(crate) struct BindingInfo {` (`lowering.rs:14`) and two return types, `fn tdz_binding_info(..) -> BindingInfo {` (`:10735`) and `fn infer_catch_binding_info(..) -> BindingInfo {` (`:11791`). The brief's own line list contains all three. | The E0063 count in M1 is **41**, not 44. |
| **0.2** | "5 `mark_tdz_binding` and 11 `clear_tdz_binding` calls", "the 16 mark/clear sites" | **4** `mark` call sites (`:13786`, `:13804`, `:14811`, `:19920`) + 1 definition (`:37803`); **10** `clear` call sites (`:14854`, `:16446`, `:16512`, `:18475`, `:19938`, `:31660`, `:31726`, `:31783`, `:31817`, `:32145`) + 1 definition (`:37810`). 14 call sites total. | M2's "11 sites" is **10**. |
| **0.3** | The four `clear` sites at `:31660/:31726/:31783/:31817` are "the for-head clear sites … plus 14.7.4.4 CreatePerIterationEnvironment" | They are inside `lower_object_binding_pattern` (`:31628`) and `lower_array_binding_pattern` (`:31753`) — **binding-pattern** lowering, reached from `let`/`const` destructuring, from `var` destructuring, and from for-in/for-of heads alike. Neither function knows it is in a loop head. There is no CreatePerIterationEnvironment logic at any of the four. | §4's staging must treat them as the *destructuring* path, not a loop path. Corpus entry 9 is re-aimed accordingly (§7). |
| **0.4** | M5 is "the script/module top level" | `lower_root_statement_items_with_function_bindings` (`:9458`) is reached from **four** call sites: `:8368` (script prepass), `:8466` (script final body), `:14902` (`lower_function` body, via `lower_root_statement_items` `:9446`) and `:19993` (the second function-lowering path). **Every function body** is a statement-list scope with no predeclaration, not only the script top level. | M5's blast radius roughly doubles. Corpus entries 1–3 and 7 all land on it. |
| **0.5** | Item (4): the nine `analysis.rs` re-mintings "all route through one accessor on the binding state" | `analysis.rs` has no `BindingInfo` and cannot get one. All nine mints (`:1826`, `:1861`, `:2038`, `:2340`, `:2360`, `:2413`, `:2433`, `:4148`, `:4299`) feed *name sets*: `EnvironmentPlan::binding_storage_names` for `EnvironmentKind::ForInOfTdzHead`, and the `head_aliases` capture-rename map. Two of the four `is_tdz_binding_storage_name` callers (`:14692`, `:37235`) likewise hold only an analysis-minted `&str`, never a `BindingInfo`. | The prefix cannot be collapsed into the state. §2.3 splits the two roles explicitly and keeps a name-domain predicate for the three name-only sites. |
| **0.6** | Item (6): "the contract must state whether 9.1.1.1.5 step 3 is covered and, if not, name it as an open premise" | The write side is **closable this round** and must be closed. Six sites perform PutValue on an identifier and each already holds the resolved `BindingInfo`: `lower_identifier_assign_value` (`:31056`), `lower_array_assignment_identifier_target` (`:32037`), the arithmetic compound-assign arm (`:30584`), the logical compound-assign arm (`:30833`), the bitwise compound-assign arm (`:30934`), and `lower_update` (`:32935`). All six branch on `binding.mode == BindingMode::Const` and none tests initialization. Once `Initialization` is a field, the state is in scope at all six with no new plumbing. | M6 moves from "named premise" to **in scope**, §2.4. |
| **0.7** | — (not in the brief) | The compound-assign arms build `ExprIr::Identifier(storage_name)` **directly** (`:30646` and the corresponding lines in the logical and bitwise arms), bypassing `lower_identifier_name_inner` entirely; `lower_update` does the same at `:32953`. So the *read* half of `x += 1`, `x &&= 1`, `x |= 1` and `x++` is four further holes in 9.1.1.1.6 step 2, distinct from the write half. The three compound arms are **independent code**, not wrappers: `:30833` and `:30934` each have their own `lookup_binding`, their own `mode == Const` test, and their own `unsupported_expr("assignment to const binding")`. | Seven Reference-shaped sites in total; §4 Stage 3. |

Counts used throughout, all exact:

| Quantity | Count | How obtained |
|---|---|---|
| `BindingInfo` struct literals in `lowering.rs` | **41** | `grep -n 'BindingInfo {' lowering.rs \| grep -v VarBindingInfo \| grep -vE 'fn \|struct '` |
| `declare_binding` call sites | **50** | 51 matching lines − 1 definition (`:36619`) |
| `lookup_binding` call sites | **26** | 27 matching lines − 1 definition (`:37518`) |
| `mark_tdz_binding` / `clear_tdz_binding` call sites | **4** / **10** | §0.2 |
| `is_tdz_binding` call sites | **1** (`:17112`) | + 1 definition (`:37817`) |
| `is_tdz_binding_storage_name` call sites | **4** (`:7693`, `:14692`, `:17111`, `:37235`) | + 1 definition (`:10746`) |
| `tdz_binding_storage_name` call sites | **10** — 1 in `lowering.rs` (`:10738`) + 9 in `analysis.rs` | + 1 definition (`lowering_helpers.rs:1541`) |
| `TDZ_BINDING_STORAGE_PREFIX` uses | **4** | `names.rs:18` def; `lowering.rs:10747`; `lowering_helpers.rs:1542`; `lib.rs:127` (re-export) + `lib.rs:8376` (`#[cfg(test)]`) |
| `ExprIr::Identifier(` construction sites | **80** in `lila-ir`, **30** in `lila-aot-wasm` | used to price open obligation **O1** (§5) |
| `StatementIr::Lexical {` construction sites in `lila-ir` | **113** | used to refuse a "single exit" claim (§2.5) |
| `boa_ast::Declaration` variants | **6** | `vendor/boa_ast-0.21.1/src/declaration/mod.rs:41` |
| `boa_ast::LexicalDeclaration` variants | **4** | already matched exhaustively at `lowering.rs:13758-13765` |

---

## §1. Spec basis

### 1.1 The domain: a Declarative Environment Record binding is a three-state object

ECMA-262 9.1.1.1 gives a Declarative Environment Record's binding four
independent attributes, of which two are lifecycle and two are policy:

| Attribute | Set by | Domain |
|---|---|---|
| existence | CreateMutableBinding 9.1.1.1.2 / CreateImmutableBinding 9.1.1.1.3 | present / absent |
| **initialized** | InitializeBinding 9.1.1.1.4 (step 3: "Record that the binding … has been initialized") | uninitialized / initialized |
| mutable | which of 9.1.1.1.2 / 9.1.1.1.3 created it | mutable / immutable |
| deletable / strict | the `D` / `S` argument at creation | boolean |

The lifecycle is therefore the ordered chain

```
absent  --CreateMutableBinding / CreateImmutableBinding-->  uninitialized
        --InitializeBinding-->                              initialized
```

with **no edge back**. 9.1.1.1.4 step 2 asserts the binding "must be
uninitialized", so InitializeBinding is a one-shot transition; 9.1.1.1.7
DeleteBinding is the only removal and applies to `[[Deletable]]` bindings only,
which lexical declarations never are.

`mutable` and `initialized` are **orthogonal**. `const x = 1;` is immutable and,
between BlockDeclarationInstantiation and its LexicalBinding evaluation,
uninitialized. Lila conflates neither today — `BindingMode` carries the first —
but Lila also does not carry the second on the binding, which is the whole of
this area.

### 1.2 The two consumers, and why they are two obligations

**GetBindingValue (9.1.1.1.6).**

> 1. Assert: envRec has a binding for N.
> 2. If the binding for N in envRec is an uninitialized binding, throw a **ReferenceError** exception.
> 3. Return the value currently bound to N in envRec.

**SetMutableBinding (9.1.1.1.5).**

> 1. If envRec does not have a binding for N, then …
> 2. Let S be strict.
> 3. If the binding for N in envRec is an uninitialized binding, throw a **ReferenceError** exception.
> 4. Else if the binding for N in envRec is a strict binding, set S to true.
> 5. If the binding for N in envRec has not been initialized, throw a ReferenceError exception. *(editorially folded into 3 in current drafts)*
> 6. Else if the binding for N in envRec is a mutable binding, change its bound value to V.
> 7. Else … If S is true, throw a **TypeError** exception.

Two facts follow that the current tree does not honour:

- **Step 3 precedes step 6/7.** The uninitialized test comes *before* the
  immutability test. `const x = 1; { x = 2; const x = 3; }` throws
  **ReferenceError**, not TypeError, and `S` is irrelevant to step 3 — an
  uninitialized write throws in sloppy mode too. Lila's write sites test
  `mode == BindingMode::Const` and never test initialization (§0.6), so they
  reach step 6/7 with step 3 unevaluated.
- **The write obligation is not derivable from the read obligation.** A single
  "is this name in TDZ?" predicate consulted only on the read path leaves
  9.1.1.1.5 step 3 unenforced. That is exactly mistake class M6.

**Compound assignment and update evaluate both.** 13.15.4
ApplyStringOrNumericAssignment does GetValue (→ 9.1.1.1.6 step 2) then PutValue
(→ 9.1.1.1.5 step 3); 13.4.4 / 13.4.5 UpdateExpression likewise. So `x += 1` and
`x++` on an uninitialized binding must throw at the *read*, and a correct
implementation that only guarded the write would still be observably wrong when
the RHS has a side effect.

**`typeof` does not exempt.** 13.5.3 step 3 special-cases only an
*unresolvable* Reference. A Reference whose base is an Environment Record that
has the binding goes through GetValue, so `typeof x` where `x` is uninitialized
throws ReferenceError. The reader must therefore consume the binding's state,
not a nullability probe.

### 1.3 The moments: when uninitialized is entered and left

| Moment | Spec | What it fixes |
|---|---|---|
| Enter *uninitialized* — block/switch/case bodies | 14.3.1.2 BlockDeclarationInstantiation steps 3.b.i / 3.b.ii: for each element of `LexicallyScopedDeclarations`, `CreateImmutableBinding(dn, true)` for `const`, `CreateMutableBinding(dn, false)` for `let` and class declarations — **before any statement of the block runs** | The whole list is instantiated first; a read in statement 1 of a name declared in statement 5 sees an existing, uninitialized binding |
| Enter *uninitialized* — script top level | 16.1.7 GlobalDeclarationInstantiation step 17: for each element of `lexDeclarations`, `CreateImmutableBinding` / `CreateMutableBinding` on the *global lexical* Environment Record, again before evaluation | The same, for the whole script |
| Enter *uninitialized* — function body top level | 10.2.11 FunctionDeclarationInstantiation step 30: for each `d` of `lexDeclarations`, `CreateImmutableBinding(dn, true)` / `CreateMutableBinding(dn, false)` on `lexEnv`, before the body evaluates | The same, for every function body — §0.4 |
| Enter *uninitialized* — formal parameters | 10.2.11 step 21: `CreateMutableBinding(paramName, false)` for every `BoundName` of the formals, then step 24/27 initializes them left to right | `function f(a = b, b) {}` throws |
| Leave — declarator with initializer | 14.3.1.2 LexicalBinding : BindingIdentifier Initializer, steps 1-5: resolve the binding, **evaluate the Initializer**, then `InitializeReferencedBinding(lhs, value)` | The order: the initializer is evaluated *while the binding is still uninitialized* — this is why `let x = x;` throws |
| Leave — declarator without initializer | 14.3.1.2 LexicalBinding : BindingIdentifier, step 3: `InitializeReferencedBinding(lhs, undefined)` | `let x;` initializes to undefined |
| Leave — class declaration | 15.7.16 ClassDeclaration : Evaluation, step 2: `InitializeBoundName(className, value, env)` after ClassDefinitionEvaluation | The class body evaluates while the class name is uninitialized in the enclosing scope |
| Leave — for-of/for-in head | 14.7.5.7 ForIn/OfBodyEvaluation step 7.d, via 14.7.5.5 ForDeclarationBindingInstantiation: a fresh Environment Record per iteration, bindings created then initialized from `nextValue` | The head expression is evaluated in an environment where the loop's bound names exist and are uninitialized |
| Leave — per-iteration copy | 14.7.4.4 CreatePerIterationEnvironment steps 1.e-f: `CreateMutableBinding` then `InitializeBinding(bn, lastValue)` — created and initialized in the same step | A `for (let i = …)` per-iteration copy has **no** TDZ window |

The load-bearing shape for this contract: **creation is per statement-list, in
one sweep, before evaluation; initialization is per declarator, after its
initializer is evaluated.** Everything in §2 exists to make those two facts
unforgeable.

### 1.4 Where the spec leaves latitude, and what this contract chooses

**C1 — one state or two encodings.** The spec has one bit. Lila has two
encodings (`tdz_scopes` membership; the `$tdz.` storage-name prefix) OR-ed at one
consumer (`lowering.rs:17111-17113`). *Choice: one field on `BindingInfo`.* The
prefix survives, demoted to what it actually is — a reserved storage spelling —
and is never read as a lifecycle state again (§2.3).

**C2 — where the state lives.** The spec puts it on the binding record. Lila
could put it on the scope (a set) or on the binding (`BindingInfo`). *Choice:
on `BindingInfo`.* This is not a preference: `lookup_binding` (`:37518`) already
resolves a name by walking `self.scopes.iter().rev()` and returns a **clone** of
the found `BindingInfo`. Putting the state on the record makes the state travel
with the resolution for free and deletes `is_tdz_binding`'s positional
`zip(self.tdz_scopes.iter().rev())` (`:37821`) outright — M3 is closed by
deletion, not by discipline.

**C3 — a two-state enum, or three.** The spec has exactly two states for an
existing binding (absent is `Option::None` from `lookup_binding`). *Choice: two
variants, `Uninitialized(..)` and `Initialized`.* No `Default`, no `Unknown`, no
`#[non_exhaustive]`.

**C4 — the sub-state on `Uninitialized`.** Lila needs one distinction the spec
does not have: whether the uninitialized scope entry has already claimed its real
storage name, or is a placeholder whose `storage_name` is the unspellable `$tdz.`
form. `lexical_storage_name` (`:7689-7703`) reads exactly this today, via
`!Self::is_tdz_binding_storage_name(&binding.storage_name)`. *Choice: carry it as
a payload on `Uninitialized`, not as a third top-level variant and not as a
string test.* The domain stays two-state at the level the spec cares about,
`Initialized` gains no payload nobody reads, and the one site that needs the
distinction gets an exhaustive match instead of a prefix test.

**C5 — synthetic bindings.** Lila declares bindings the spec never mentions:
compiler temporaries (`alloc_temp_binding_name`), the `$arguments` lexical, the
`$derived.*` activation cells, captured aliases of an outer binding, `var`
mirrors. *Choice: they spell `Initialized`.* Each is created and stored in one
step and has no source-visible window; §4.3 lists all 26 such literals so the
encoder does not have to judge case by case.

**C6 — the `report_shadowed_namespace_globals` bail-out.** `namespace.rs:466-478`
and `:560-573` refuse to compile an eagerly evaluated module whose top level
binds `Object` or `Symbol`, on the stated ground that such a binding "poisons it
with a TDZ that no placement of the prelude can dodge." *Choice: do not remove
it, and annotate it with premise **P1** (§5).* The comment describes the correct
spec behaviour; today the compiler does not actually produce that TDZ at module
top level (that is M5). Removing the bail-out is only safe once §4 Stage 1 lands
*and* premise **P2** (`BindingStorage`, out of lane) is discharged, because a
merged-scope `Object` that lands in `BindingStorage::Dynamic` gets no runtime
uninitialized check at all.

---

## §2. Type mapping

New module: **`crates/lila-ir/src/binding_lifecycle.rs`** (exclusive to this
area). One `mod binding_lifecycle;` line in `lib.rs` immediately after
`mod binding_names;` (`lib.rs:57`), and `pub(crate) use binding_lifecycle::*;`
alongside the existing `pub(crate) use` block. Record the hub edit in the lane
note.

### 2.1 `Initialization` — the state, as a mandatory field

```rust
/// The lifecycle state of one Environment Record binding (ECMA-262 9.1.1.1).
///
/// Two variants because the spec has two: CreateMutableBinding (9.1.1.1.2) and
/// CreateImmutableBinding (9.1.1.1.3) leave a binding *uninitialized*, and
/// InitializeBinding (9.1.1.1.4) is the sole transition to *initialized*.
/// Absence is `Option::None` from `lookup_binding` and is not a variant here.
///
/// No `Default`. A binding whose state was never decided is not a binding whose
/// state is "probably initialized"; it is a declaration site that has not been
/// read against 14.3.1.2 / 16.1.7 step 17 / 10.2.11 steps 21 and 30. Omitting
/// the field is `error[E0063]` at all 41 `BindingInfo` literals, which is the
/// entire point.
///
/// No `Unknown`, no `#[non_exhaustive]`: `Initialization` is matched
/// exhaustively at the read and write sites, so a third state must be decided
/// there rather than fall into a `_` arm that answers "not in TDZ".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Initialization {
    /// Created, not yet initialized. GetBindingValue (9.1.1.1.6) step 2 and
    /// SetMutableBinding (9.1.1.1.5) step 3 both throw ReferenceError while a
    /// binding is in this state.
    Uninitialized(UninitializedStorage),
    /// InitializeBinding (9.1.1.1.4) has run, or the binding was created and
    /// stored in one step (see `Initialization::at_creation` for the closed
    /// list of spec steps that do that).
    Initialized,
}
```

`BindingInfo` (`lowering.rs:14-21`) gains it as the **last** field, so the 41
literals take a one-line addition each and diffs stay readable:

```rust
pub(crate) struct BindingInfo {
    pub(crate) mode: BindingMode,
    pub(crate) storage_name: String,
    pub(crate) kind: ValueKind,
    pub(crate) possible_kinds: KindSet,
    pub(crate) heap_shape: Option<Box<HeapShape>>,
    pub(crate) function_targets: BTreeSet<FunctionId>,
    /// ECMA-262 9.1.1.1. See `Initialization`.
    pub(crate) initialization: Initialization,
}
```

`BindingInfo` keeps `#[derive(Debug, Clone, PartialEq, Eq)]`; `Initialization`
and `UninitializedStorage` derive `Copy` in addition, so the field costs nothing
at the 26 `lookup_binding` clone sites.

### 2.2 `UninitializedStorage` — C4's payload

```rust
/// Which storage an uninitialized binding has already claimed.
///
/// Not a spec distinction: 9.1.1.1.2 gives the binding its slot at creation.
/// It is Lila's *name-allocation* question, and it exists here so that
/// `lexical_storage_name` (lowering.rs:7689) can ask it as an exhaustive match
/// on the binding's own state instead of testing its storage name for a `$tdz.`
/// prefix. A third storage disposition is `error[E0004]` at **both** of its
/// consumers: `lexical_storage_name` and `direct_lexical_storage_name`'s reuse
/// branch. The latter first shipped as
/// `.filter(|b| b.initialization == Uninitialized(Allocated))`, which would
/// have compiled silently past a new variant and quietly dropped the reuse
/// rule — reopening M2b for it. It is an exhaustive `match`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UninitializedStorage {
    /// The binding's real storage name is already allocated and is the name the
    /// eventual InitializeBinding will write. A later inner declaration of the
    /// same source name therefore *does* shadow it and must allocate a fresh
    /// name. Produced by BlockDeclarationInstantiation-shaped predeclaration.
    Allocated,
    /// The scope entry is a placeholder: its `storage_name` is
    /// `tdz_binding_storage_name(source_name)`, an unspellable name that backs
    /// no slot in this scope, and it exists only so a read of the source name
    /// resolves to *something uninitialized* rather than to an outer binding.
    /// It must not count as a shadowing declaration when the real binding
    /// allocates its name. Produced by `Lowerer::tdz_binding_info`.
    Placeholder,
}
```

**Why this variant pair earns its place, precisely.** Collapsing the two into a
bare `Uninitialized` and rewriting `lexical_storage_name:7693` as
`binding.initialization == Initialized` is a silent miscompile, and this is the
single most dangerous available mistake in the whole area: the top-level and
function-body scopes are not `direct_lexical_scopes` (`Lowerer::new` seeds
`direct_lexical_scopes: vec![false]`, `:7729`), so `direct_lexical_storage_name`
(`:7706`) falls through to `lexical_storage_name` (`:7710`), whose answer for a
source name is *stable only while nothing in scope claims it*. Predeclare a
top-level `let x` with storage name `x`, then let the declarator recompute:
`shadows_scope_binding` is now true, `alloc_temp_binding_name("lex")` returns
`$lexN`, and the program writes `$lexN` while every read of `x` before the
declaration reads slot `x`. §2.4's token exists to make that unwritable; this
variant pair is what stops the *other* half of the same trap, where a nested
block's `let x` stops shadowing an outer uninitialized `x` and the two share one
slot.

### 2.3 `TdzPlaceholderName` — the prefix, demoted to a name domain

The `$tdz.` prefix is a **wire name**, not a state. It reaches
`lila-aot-wasm` as `EnvironmentPlan::binding_storage_names` for
`EnvironmentKind::ForInOfTdzHead`, as `ForInOfEnvironmentIr::tdz_binding_names`
(`lowering.rs:10422-10431`), and as `CaptureBindingPlan::name` for a closure
created inside a for-in/for-of head. Nine `analysis.rs` sites mint it; three
`lowering.rs` sites test it against a bare `&str` with no `BindingInfo` in reach
(`:7693` is the fourth, and it is the one that stops testing strings).

```rust
/// The reserved storage spelling for a for-in/for-of head TDZ binding and a
/// pre-initialization formal parameter (`names.rs`'s
/// `TDZ_BINDING_STORAGE_PREFIX`, `"$tdz."`).
///
/// A newtype with one constructor, for the reason round 2 gave module binding
/// names: the prefix crosses the analysis -> lowering -> aot-wasm boundary as a
/// plain `String` in ten places, and "mint one more `$tdz.` name by hand" or
/// "test for the prefix by hand" are the two ways the domain drifts. There is
/// no `From<String>` and no public field.
///
/// This type says nothing about lifecycle. A binding *named* this way is always
/// `Initialization::Uninitialized(UninitializedStorage::Placeholder)` — that is
/// enforced by `BindingInfo::tdz_placeholder` being the only constructor that
/// accepts one — but the converse is deliberately false: a predeclared block
/// binding is uninitialized with an ordinary storage name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TdzPlaceholderName(String);

impl TdzPlaceholderName {
    /// The sole constructor. Replaces `lowering_helpers::tdz_binding_storage_name`.
    pub(crate) fn for_source_name(source_name: &str) -> Self { … }

    pub(crate) fn as_str(&self) -> &str { &self.0 }
    pub(crate) fn into_string(self) -> String { self.0 }

    /// The sole predicate. Answers a *name-domain* question — "was this storage
    /// name minted by `for_source_name`?" — for the three sites that hold an
    /// analysis-minted `&str` and no `BindingInfo`:
    /// `lower_function`'s capture-info arm (lowering.rs:14692),
    /// `is_script_global_var_capture` (lowering.rs:37235), and the
    /// `#[cfg(test)]` assertion at lib.rs:8376.
    ///
    /// It is NOT the TDZ predicate. `lower_identifier_name_inner` must not call
    /// it; see `Initialization`.
    pub(crate) fn names_a_placeholder(storage_name: &str) -> bool { … }
}
```

`names.rs:18` keeps `TDZ_BINDING_STORAGE_PREFIX` and its `lib.rs:127`
re-export — `lib.rs:8376`'s test asserts on it and `binding_lifecycle.rs` builds
the name from it — but it acquires a doc comment saying it is a storage spelling
and that the state lives on `BindingInfo`.
`lowering_helpers.rs:1541`'s `tdz_binding_storage_name` is **deleted**; its ten
callers call `TdzPlaceholderName::for_source_name(..)` and, where a `String` is
wanted, `.into_string()`.
`Lowerer::is_tdz_binding_storage_name` (`:10746`) is **deleted**; its four
callers are re-routed by §4.

### 2.4 `PendingInitialization` and `LoweredInitializer` — the ordering proof

This is the round-2 `ReferencePins::materialize(PendingReferenceWrite)` idiom
(`reference.rs:531`, `:625`), transposed.

```rust
/// A binding that CreateMutableBinding / CreateImmutableBinding has created and
/// InitializeBinding has not yet initialized: the *obligation* half of
/// 14.3.1.2's two-phase shape.
///
/// Neither `Clone` nor `Copy`, private fields, no `Default`. The only exits are
/// `initialize` and `initialize_value`, both of which consume `self`, so a
/// binding cannot be initialized twice (`E0382 use of moved value`) — which is
/// 9.1.1.1.4 step 2's assertion, made structural.
///
/// It carries `storage_name` because the predeclaration allocated it. That is
/// not a convenience: at a scope that is not a `direct_lexical_scope` —
/// every function body and the script top level — `lexical_storage_name`
/// (lowering.rs:7689) returns a *different* name once the predeclared entry is
/// in scope, so a declarator that recomputed its own name would write a slot
/// nothing reads. Taking the name from the token is what makes the two halves
/// of one binding provably the same slot.
#[derive(Debug)]
#[must_use = "a created binding that is never initialized stays in TDZ for the \
              rest of its scope"]
pub(crate) struct PendingInitialization {
    source_name: String,
    mode: BindingMode,
    storage_name: String,
}

impl PendingInitialization {
    /// InitializeBinding (9.1.1.1.4) for `LexicalBinding : BindingIdentifier
    /// Initializer` and `LexicalBinding : BindingIdentifier`.
    ///
    /// Takes the *lowered* initializer, so "clear the TDZ, then lower the
    /// initializer" cannot be written: there is no `LoweredInitializer` in
    /// scope until the initializer has been lowered, and no way to make one
    /// except `Lowerer::lower_declarator_initializer`. `let x = x;` therefore
    /// lowers its `x` read against the still-`Uninitialized` binding, which is
    /// 14.3.1.2 steps 3-5 and 9.1.1.1.6 step 2.
    ///
    /// Returns the `BindingInfo` the caller hands to `declare_binding`, and the
    /// storage name the caller puts in its `StatementIr::Lexical`.
    pub(crate) fn initialize(self, init: LoweredInitializer)
        -> (String, BindingInfo, TypedExpr);

    /// InitializeBinding for a name bound by a *pattern*, where one lowered
    /// initializer discharges every `BoundName` of the pattern (8.6.2
    /// BindingInitialization). Borrows the witness rather than consuming it,
    /// and takes the per-name `ValueInfo` the destructuring machinery derived.
    pub(crate) fn initialize_value(
        self,
        witness: &LoweredInitializer,
        info: ValueInfo,
    ) -> (String, BindingInfo);

    pub(crate) fn source_name(&self) -> &str;
    pub(crate) fn mode(&self) -> BindingMode;
}
```

```rust
/// Evidence that a declarator's Initializer has been evaluated.
///
/// `#[must_use]`, not `Clone`, private field. The sole producer is
/// `Lowerer::lower_declarator_initializer(&mut self, init: Option<&Expression>)`,
/// which takes the declarator's own initializer slot — so the absent case
/// (`let x;`, 14.3.1.2 step 3, InitializeReferencedBinding(lhs, undefined)) is
/// reached by passing `None` and not by a public `elided()` constructor that a
/// site with a real initializer could reach for by mistake.
#[derive(Debug)]
#[must_use = "a lowered initializer that discharges no PendingInitialization is \
              an evaluated expression with no consumer"]
pub(crate) struct LoweredInitializer(TypedExpr);

impl LoweredInitializer {
    pub(crate) fn value_info(&self) -> ValueInfo;
    /// For the destructuring path, which needs the value expression itself.
    pub(crate) fn into_expr(self) -> TypedExpr;
}
```

**What this closes and what it does not.** It closes M2 *for every path that
takes a token*: the transition cannot be written before the initializer exists,
and cannot be written twice. It does **not** make `Initialization::Initialized`
unspellable — 26 of the 41 literals legitimately spell it (§4.3) — so a future
site could hand-build an `Initialized` `BindingInfo` for a predeclared name and
skip the token. That residue is ledger entry **L1** (§5), not a claim of
totality. Round 2's finding 8 ("a doc comment asserting an invariant the type did
not carry") is the failure mode being avoided here by saying so.

### 2.5 `LexicalScopeInstantiation` — the typestate on the statement-list entry

```rust
/// One statement-list scope's BlockDeclarationInstantiation (14.3.1.2), from
/// the sweep that creates every lexically scoped binding to the last
/// LexicalBinding evaluation that initializes one.
///
/// This type exists so that **a statement-list lowering entry cannot be written
/// without predeclaring**, and so that the sweep runs into the scope the
/// statement list is lowered in. `lower_statement_items` and
/// `lower_root_statement_items_with_function_bindings` take one by value; the
/// constructors are `instantiate`, `instantiate_switch` and
/// `instantiate_in_current_scope`, each of which performs the sweep. A fifth
/// statement-list entry added later is `error[E0061]`/`error[E0308]` until its
/// author builds one — which is M5, closed structurally rather than by
/// remembering to call `predeclare_block_lexical_bindings`.
///
/// **M5b — the sweep landing in the wrong Environment Record.** The token as
/// first written witnessed that *a* sweep ran, not where: `instantiate` reaches
/// `create_lexical_binding` -> `declare_binding` -> `scopes.last_mut()`, and
/// the push was in a different function at every block-shaped call site, so
/// `let scope = LexicalScopeInstantiation::instantiate(self, items);
/// self.push_scope(); self.lower_statement_items(items, scope);` compiled,
/// satisfied M5's `E0061`, and declared every `let` of the inner list into the
/// **parent** scope — where it outlives the block and shadows its siblings.
/// 14.3.1.2 step 1's `env` is the *new* declarative record, so the push belongs
/// to the instantiation. It is now a private field:
///
/// ```rust
/// enum InstantiatedFrame { Pushed, Current }
/// ```
///
/// `instantiate` and `instantiate_switch` push (`Pushed`);
/// `instantiate_in_current_scope` does not (`Current`), for the three root
/// statement lists whose record — 16.1.7's global lexical record, 10.2.11's
/// `lexEnv` — the caller established and outlives the list. `finish(self,
/// lowerer)` consumes the token and pops iff `Pushed`, so the frame is popped
/// exactly once and only by the value that pushed it, and a caller that pushes
/// on its own gets a frame it has no token to pop.
///
/// `#[must_use]`, not `Clone`. Tokens are taken out by source name; whatever is
/// left at `finish` is the set of names whose declarators lowering did not
/// reach (unsupported binding forms, and the `var`/function items the sweep
/// deliberately skips), and those names stay `Uninitialized` for the scope —
/// which is the correct answer, not a leak.
#[must_use]
pub(crate) struct LexicalScopeInstantiation {
    pending: BTreeMap<String, PendingInitialization>,
}

impl LexicalScopeInstantiation {
    /// 14.3.1.2 / 16.1.7 step 17 / 10.2.11 step 30. Declares every lexically
    /// scoped name of `items` into the current scope as
    /// `Initialization::Uninitialized(UninitializedStorage::Allocated)` and
    /// returns one token per name.
    pub(crate) fn instantiate(
        lowerer: &mut ScriptLowerer<'_>,
        items: &[StatementListItem],
    ) -> Self;

    /// 14.3.1.2 for a switch: the union over every case's statement list
    /// (14.12.4 CaseBlockEvaluation instantiates the whole CaseBlock's
    /// LexicallyScopedDeclarations once, not per case).
    pub(crate) fn instantiate_switch(
        lowerer: &mut ScriptLowerer<'_>,
        switch: &AstSwitch,
    ) -> Self;

    pub(crate) fn take(&mut self, source_name: &str) -> Option<PendingInitialization>;
    pub(crate) fn is_empty(&self) -> bool;
}
```

`instantiate` subsumes `predeclare_block_lexical_bindings` (`:13738`) and
`predeclare_direct_lexical_binding` (`:13752`); `instantiate_switch` subsumes
`predeclare_switch_lexical_bindings` (`:13744`). The `Declaration` match inside
becomes **exhaustive over all 6 boa variants** — today it is
`Declaration::Lexical`, `Declaration::ClassDeclaration`, `_ => {}` (`:13806`),
and a seventh boa variant that bound a name lexically would be silently skipped.
The four function forms are named with the reason they are not in TDZ (10.2.11
step 30 / 16.1.7 step 17.b: a hoisted function declaration is created **and
initialized** at instantiation), so a reader sees a decision rather than a hole.

**Refused: a "single exit" claim over `StatementIr::Lexical`.** Making the token
the sole producer of the lexical-declaration IR node would give an `E0308` for
"initialized without the token", but there are **113** `StatementIr::Lexical {`
construction sites in `lila-ir` and the overwhelming majority are compiler
temporaries with no source-level binding at all. Routing 113 sites through a
lifecycle type to constrain the ~6 that are lexical declarations is the
decoration AGENTS.md warns about. The claim this contract makes is the narrower
true one: the *token* cannot be discharged out of order, and its absence is
ledger **L1**.

### 2.6 `BindingResolution` — the read/write obligation, as an exhaustive match

```rust
/// The result of ResolveBinding (9.1.2.1) at a site that is about to perform
/// GetBindingValue (9.1.1.1.6) or SetMutableBinding (9.1.1.1.5).
///
/// Three variants, matched exhaustively at every Reference-shaped site. The
/// point is the `Uninitialized` arm: with `lookup_binding`'s bare
/// `Option<BindingInfo>` a site that forgets step 2 / step 3 compiles and
/// answers `undefined`; with this, the arm must be written, and the only thing
/// that can be done with a `TdzViolation` is turn it into the throw.
#[must_use]
pub(crate) enum BindingResolution {
    /// 9.1.1.1.6 step 2 / 9.1.1.1.5 step 3.
    Uninitialized(TdzViolation),
    Initialized(BindingInfo),
    /// No Environment Record in the chain has the binding; the caller falls
    /// through to its own global / builtin / unresolvable handling.
    Unresolvable,
}

/// An uninitialized binding reached by a Reference. Not `Clone`; the single
/// consumer is `into_throw`, so the arm cannot be written as a no-op that falls
/// through to the ordinary read.
#[derive(Debug)]
#[must_use = "an uninitialized binding that is read or written must throw"]
pub(crate) struct TdzViolation { /* private */ }

impl TdzViolation {
    /// The ReferenceError of 9.1.1.1.6 step 2 and 9.1.1.1.5 step 3. One
    /// message, one `NativeErrorKind`, one `ValueInfo`, produced here rather
    /// than re-spelled at each of the six sites.
    pub(crate) fn into_throw(self) -> TypedExpr;
}
```

`ScriptLowerer` gains **one** new accessor, in `lowering.rs`'s scope-helper
region:

```rust
/// ResolveBinding + the 9.1.1.1.5/9.1.1.1.6 state test, for the seven sites
/// that perform GetValue or PutValue on an Environment Record Reference.
///
/// `lookup_binding` stays as it is for the other 19 call sites, which ask
/// metadata questions (`is_some`, `possible_kinds`, `storage_name` for a map
/// key) and must not be forced to decide what TDZ means for them.
fn resolve_binding_reference(&self, name: &str) -> BindingResolution;
```

**Honest limit.** This makes the obligation a compile error at the seven sites
that call it, and makes a *new state* an `E0004` at all seven. Three things it
does **not** do, each a ledger entry rather than a claim:

- It does not make an *eighth, newly written* Reference site fail to build,
  because `ExprIr::Identifier(String)` can still be constructed from any `&str`.
  Measured cost in §5, open obligation **O1**, ledger **L3**.
- Rust does not generally force an `Uninitialized` arm to *use* its
  `TdzViolation`: `#[must_use]` does not fire for a value wildcarded in a match.
  The known exception is now closed, however:
  `lower_array_assignment_identifier_target` must consume the witness to build
  `IdentifierWriteReferenceIr::uninitialized_binding`, and the backend's
  exhaustive write disposition emits the runtime ReferenceError at PutValue.
  A future arm can still discard a witness; that narrower residue is ledger
  **L8**.
- It does not stop a new site from classifying the record *by hand*.
  `BindingInfo::initialization` is `pub(crate)` and `Initialization` derives
  `PartialEq`, so
  `self.lookup_binding(&name).is_some_and(|b| b.initialization == Initialization::Initialized)`
  gets the classification right, never obtains a `TdzViolation`, and therefore
  never emits the throw. Adjacent to **L3** but distinct: L3 covers sites that
  do not test at all; this covers sites that test and then do nothing. Ledger
  **L8**.

### 2.7 Refused types, and why

Recording these so the encoder does not re-derive them and the reviewer can
check the reasoning.

- **`InitializedAtCreation` witness enum** (a closed enum naming *which* spec
  step initialized a binding that skips 9.1.1.1.4: `VarInstantiation`,
  `FormalParameter`, `CatchParameter`, `CapturedAlias`, `CompilerTemporary`,
  `HoistedFunction`, `ImportBinding`, required as an argument to an
  `Initialization::initialized(..)` constructor). **Refused.** Nothing reads it.
  With the mandatory field, `Initialization::Initialized` already has to be
  written at all 26 such sites; the witness adds a second token to write at the
  same sites and detects no additional mistake. It is deliberation, not
  proof — round-2 finding 3's rule ("a field constructed at 3 sites and read at
  0") applied prospectively. The spec justification per site is carried instead
  by §4.3's table and by one-line comments at the six non-obvious literals.
- **Replacing `lookup_binding`'s return type wholesale.** 20 of its 26 call
  sites are metadata probes; forcing each to spell a `BindingResolution` arm
  would add 20 arms that all mean "I do not care" and would dilute exactly the
  six that do. Refused in favour of the second accessor in §2.6.
- **A `BindingStorageName` enum over `BindingInfo::storage_name`.** Would make
  `UninitializedStorage` redundant, but changes a field read at 41 literals and
  ~60 further sites inside and outside this area's owned regions, for a
  distinction §2.2 already carries at one site. Refused as out of proportion.

---

## §3. Mistake-class table

| # | Mistake | Today | After | Compile error |
|---|---|---|---|---|
| **M1** | Declare a lexical binding and forget it starts uninitialized. 50 `declare_binding` call sites, 41 `BindingInfo` literals, governed by 4 `mark_tdz_binding` calls. | Compiles silently; the binding reads as initialized. | Every literal must spell `initialization:`. `Var`, catch, capture and temporary literals spell `Initialized` explicitly rather than inherit it by omission. | `error[E0063] missing field 'initialization' in initializer of 'BindingInfo'` — 41 sites. |
| **M2** | Clear the TDZ before the initializer is lowered. `declare_binding` and `clear_tdz_binding` are separate calls whose order is convention at 10 sites. `let x = x;` must throw (9.1.1.1.6 step 2); the wrong order reads undefined. | Order is a review item. | The separate `clear_tdz_binding` call — the thing whose *order* was the hazard at ten sites — no longer exists: the transition **is** the `declare_binding`, and it is built out of a `LoweredInitializer`. | `error[E0382] use of moved value` on a second initialization, and `error[E0061]`/`error[E0308]` on an `initialize` with no `LoweredInitializer` to hand it. **Not** `E0425`, and §2.4's "there is simply no value of this type in scope until then" is false as landed: `LoweredInitializer::evaluated(TypedExpr)` is `pub(crate)` and accepts any `TypedExpr`, so `initialize(evaluated(TypedExpr::undefined()))` followed by lowering the real initializer compiles and reproduces the exact M2 defect. That half is ledger **L6**, whose eight call sites are enumerated there. |
| **M2b** | Initialize a predeclared binding under a *recomputed* storage name. Not in the brief; §2.2 shows it is the trap a naive M5 fix walks into at every non-`direct_lexical` scope. | Would compile and write a slot nothing reads. | `PendingInitialization::initialize` returns an **`InitializedBinding`** with private fields whose only exit is `declare(&mut ScriptLowerer) -> StatementIr` — it performs the scope-record write *and* emits the `StatementIr::Lexical`, so the declarator never holds the storage name as a `String`. | Structural: on a tokened path there is no `String` in the caller's hands to substitute. The `error[E0616]` this row used to claim fires only for `pending.storage_name`, which nobody would write; while `initialize` returned a bare `(String, BindingInfo, TypedExpr)` a declarator could take the token, `declare_binding` the returned record (flipping the entry to `Initialized`, so `direct_lexical_storage_name`'s `Uninitialized(Allocated)` reuse filter stops matching) and recompute the name for the emitted node — and `lower_class_declaration` held both spellings in scope simultaneously. The **untokened** paths (the four L2 destructuring sites, and the four `InitializedBinding::without_creation` fallbacks) are still governed by `direct_lexical_storage_name`'s reuse rule, a compiler-runtime rule and not a type: ledger **L2**. |
| **M3** | The two stacks fall out of step. `is_tdz_binding` (`:37817-37831`) zips `scopes.iter().rev()` against `tdz_scopes.iter().rev()` positionally; `mark`/`clear` write `last_mut()` behind an `expect`. A third scope-push entry point that pushes one stack and not the other is a panic or a silent misalignment. | Two entry points (`push_scope` `:37783`, `push_direct_lexical_scope` `:37789`), one shared invariant, no type. | `tdz_scopes`, `mark_tdz_binding`, `clear_tdz_binding` and `is_tdz_binding` are **deleted**. There is one stack. | `error[E0609] no field 'tdz_scopes' on type 'ScriptLowerer'` for any re-introduction; the misalignment has no representation. |
| **M4** | Two spellings of the same state drift apart. `is_tdz_binding_storage_name` (prefix match) OR `is_tdz_binding` (set membership), OR-ed at `:17111-17113`; nine `analysis.rs` sites re-mint the prefixed name with no relation to either. | A change to what counts as TDZ, or to the prefix, must be made in two files' worth of unrelated string logic. | The reader consumes `Initialization` and nothing else. The prefix is minted and tested only through `TdzPlaceholderName`, whose only constructor is `for_source_name`. | `error[E0599] no function or associated item named 'is_tdz_binding_storage_name'`; `error[E0308]` on any hand-built `format!("$tdz.{…}")` passed where a `TdzPlaceholderName` is wanted. |
| **M5** | A statement-list scope with no predeclaration. `lower_root_statement_items_with_function_bindings` (`:9458`) calls neither predeclare wrapper, and is the entry for the script top level **and every function body** (§0.4). `x; let x;` reads slot `x` where 9.1.1.1.6 step 2 requires a ReferenceError. | Verified live. Corroborated by the team's own workaround at `namespace.rs:466-478`. | Both statement-list entries take a `LexicalScopeInstantiation` by value; the only constructor performs the sweep. | `error[E0061] this function takes 3 arguments but 2 were supplied` at any new or un-updated statement-list entry. |
| **M6** | Read-side check without a write-side check. 9.1.1.1.5 step 3 makes an assignment to an uninitialized binding throw, *before* the immutability test of step 6/7. | Seven sites hold a resolved `BindingInfo`; **one** tests TDZ, and it tests it as a string-OR-set disjunction. The other six test `mode == Const` and nothing else. | All seven route through `resolve_binding_reference` and match `BindingResolution` exhaustively. `TdzViolation`'s only *method* is `into_throw`. | `error[E0004] non-exhaustive patterns: 'Uninitialized(_)' not covered` — the **arm must exist**, which is what this row discharges. It does **not** discharge the arm's *content*: `#[must_use]` on a struct does not fire for a value bound (or `_`-bound) in a `match` pattern, so `Uninitialized(_) => {}` compiles with no warning, and `lowering.rs:32402` legitimately writes that shape. The `error[E0599]` this row used to claim never fires. The seven arms' contents are ledger **L8**. |
| **M6b** | Compound assignment and update skip the *read* check too: the three compound arms and `lower_update` build `ExprIr::Identifier(storage_name)` directly (`:30646`, `:32953`, and the corresponding lines in `:30833`/`:30934`), never passing through `lower_identifier_name_inner`. Not in the brief (§0.7). | `x += 1`, `x &&= 1`, `x \|= 1`, `x++` on an uninitialized binding all read undefined. | Same mechanism as M6; these are four of the seven sites. | As M6. |
| **M7** | Storage-class dependence. Nothing relates `Initialization` in `lila-ir` to `allocate_binding`'s `BindingStorage` choice in `lila-aot-wasm/src/environments.rs:900`. `EnvSlot` gets the runtime uninitialized check (`:876-895`); `Fixed` and `Dynamic` get none, and a `Dynamic` tag local starts at 0 = `ValueKind::Undefined.tag()`. | The same program's answer depends on an unrelated capture-analysis decision. | **Not a compile error this round.** `BindingStorage` is in a crate this area does not edit. Named premise **P2** + integration note (§5). | none — by design, and stated as such. |
| **M8** | A new `boa_ast::Declaration` variant that binds a name lexically is silently not predeclared. Not in the brief; the `_ => {}` at `:13806` covers 4 of 6 variants. | Silent. | `LexicalScopeInstantiation::instantiate`'s match is exhaustive over all 6. | `error[E0004] non-exhaustive patterns`. |

---

## §4. Retrofit map

Four stages. Stages 1-3 are one landing; stage 4 is optional within this round
and explicitly flagged. Each stage names what it touches and what it leaves
alone.

### Stage 0 — the module, no behaviour change

1. Create `crates/lila-ir/src/binding_lifecycle.rs` with §2.1, §2.2, §2.3,
   §2.4, §2.5, §2.6.
2. `lib.rs:57`: add `mod binding_lifecycle;` after `mod binding_names;`. Add
   `pub(crate) use binding_lifecycle::*;` to the existing `pub(crate) use` block
   (the crate uses a glob-import discipline; `lowering.rs:1` is `use super::*;`).
   **This is the only `lib.rs` edit and this area is the sole `lib.rs` owner this
   round — record it in the lane note.**
3. `names.rs:18`: keep `TDZ_BINDING_STORAGE_PREFIX`; add the doc comment from
   §2.3. No other change; the `lib.rs:127` re-export stays.
4. `lowering_helpers.rs:1541`: delete `tdz_binding_storage_name`.

### Stage 1 — the field, and the 41 literals (closes M1)

5. `lowering.rs:14-21`: add the `initialization` field.
6. Fill all 41 literals. The classification is fixed here so the encoder makes
   no judgement calls:

**`Initialization::Initialized` — 26 literals.**

| Lines | What | Spec ground |
|---|---|---|
| `:7619`, `:9625`, `:10286`, `:15647`, `:15759`, `:15886` | compiler temporaries (`alloc_suspension_owned_binding`, `alloc_temp_binding_name`, `with.object.`, `object.spread.`, `yield.condition.`, `yield.template.`) | no source-level binding; created and stored in one step (C5) |
| `:9938` | root function binding | 16.1.7 step 17.b.i.1 / 10.2.11 step 30: created **and initialized** at instantiation |
| `:16020`, `:16050`, `:16083`, `:16116` | block-level function declarations (function, generator, async, async generator) | 14.2.3 / Annex B.3.3: initialized at BlockDeclarationInstantiation |
| `:10595` | for-in head, `mode == Var` arm | 10.2.11 step 28: `var` is created and initialized to undefined together |
| `:10607`, `:13439` | for-in / for-of *iteration* binding | 14.7.5.5 ForDeclarationBindingInstantiation: created then immediately initialized from `nextValue` |
| `:11798` | `infer_catch_binding_info` | 14.15.3 step 4: BindingInitialization runs before the catch block; also a value-info carrier with `storage_name: String::new()` |
| `:14024` | for-loop lexical head per-iteration | 14.7.4.4 CreatePerIterationEnvironment steps 1.e-f: `CreateMutableBinding` then `InitializeBinding(bn, lastValue)` — **no TDZ window** |
| `:14669`, `:19884` | function self-binding (`self_binding_name`) | 10.2.11 step 18: the named function expression's own binding is initialized to the function object at instantiation |
| `:14773`, `:19866`, `:20252` | captured aliases of an outer binding | 9.1.1.1.4 already ran in the owner scope; this entry is a view of an initialized cell |
| `:14787`, `:17826`, `:19897`, `:20264` | `LEXICAL_ARGUMENTS_NAME` | 10.2.11 step 22: `arguments` is created and initialized in one step |
| `:14875`, `:17853`, `:19943` | a formal parameter, *after* its `clear_tdz_binding` | this is the InitializeBinding of 10.2.11 step 24/27; see Stage 2 |
| `:18576` | class-expression name binding inside the class scope | 15.7.14 ClassDefinitionEvaluation step 6.a: `classEnv` binds `classBinding` and step 17 initializes it before the body's methods are visible |
| `:37532` | `lookup_binding`'s `var_bindings` fallback | 10.2.11 step 28. **See the caveat below.** |

**`Initialization::Uninitialized(UninitializedStorage::Allocated)` — 2 literals.**

`:13777` and `:13795`, the two predeclaration literals (lexical and class), which
Stage 2 moves into `LexicalScopeInstantiation::instantiate`.

**`Initialization::Uninitialized(UninitializedStorage::Placeholder)` — 1 literal.**

`:10736`, inside `tdz_binding_info`. Rename it
`BindingInfo::tdz_placeholder(mode: BindingMode, name: TdzPlaceholderName)`, move
it to `binding_lifecycle.rs`, and give it the `TdzPlaceholderName` parameter so
the placeholder state and the placeholder name are minted together. Its five
callers (`:10550`, `:10729`, `:12865`, `:13353`, and the three destructuring
predeclares `:31310`, `:31335`, `:31377`) change only their argument spelling.

**Determined by their site — 12 literals.** These are the sites that today are
preceded by a `clear_tdz_binding` call, plus the two whose state depends on the
declarator form:

`:16449` (`lower_lexical_declaration`, identifier declarator), `:16515`
(`lower_using_declaration`), `:18478` (`lower_class_declaration`), `:32148`
(`lower_lexical_binding_value`), `:31663`, `:31729`, `:31786`, `:31820`
(the four binding-pattern sites). All spell `Initialized` — they *are* the
InitializeBinding — and Stage 2 rewrites them to obtain the value from
`PendingInitialization::initialize`/`initialize_value` rather than to spell it.

> **Caveat on `:37532`.** This literal is not a declaration; it manufactures a
> `BindingInfo` from `var_bindings` when no scope entry matched. `var_bindings`
> contains two populations: genuine `var`s (initialized to undefined at
> instantiation) and, at script owner only, **lexical metadata** entries inserted
> by `hoist_lexical_declaration_metadata` (`:16826-16866`, `is_lexical_metadata:
> true`) for names whose real declaration has not been lowered yet. Spelling
> `Initialized` here is correct *only because Stage 2 puts a real, uninitialized
> scope entry in front of it for every lexical name*: `lookup_binding` searches
> `scopes` before `var_bindings` (`:37519-37523`), so once the top level
> predeclares, the metadata entry is never the answer for a name in TDZ. **The
> encoder must not reverse the order of Stage 1 and Stage 2 for this reason**, and
> must not "fix" `:37532` by reading `is_lexical_metadata` — that flag is a
> value-info provenance marker, not a lifecycle state, and it stays `true` for
> the whole script including after initialization.

### Stage 2 — instantiation and initialization (closes M2, M2b, M3, M5, M8)

7. `LexicalScopeInstantiation::instantiate` absorbs `:13738-13808`. Delete
   `predeclare_block_lexical_bindings`, `predeclare_switch_lexical_bindings`,
   `predeclare_direct_lexical_binding`. Make the `Declaration` match exhaustive
   (M8).
8. `lower_block` (`:10314-10324`): replace the `predeclare_block_lexical_bindings`
   call with `let scope = LexicalScopeInstantiation::instantiate(self, items);`
   and pass it to `lower_statement_items`. **The constructor performs the
   `push_direct_lexical_scope` and `lower_statement_items` performs the matching
   `finish`**, so `lower_block`'s four callers — the `Statement::Block` arm and
   `lower_try`'s try/finally pair — drop the push/pop they used to wrap it in.
   `lower_try`'s *catch* push stays: 14.15.3 step 2's `catchEnv` holds the catch
   parameter and is a different Environment Record from the catch Block's.
9. `lower_statement_items` (`:9505`) gains a
   `mut scope: LexicalScopeInstantiation` parameter.
10. `lower_switch` (`:13662`): `instantiate_switch`, passed to whatever lowers the
    case bodies. The CaseBlock's `push_direct_lexical_scope()` moves **into**
    `instantiate_switch` and the matching `pop_scope()` becomes `scope.finish(self)`,
    for the M5b reason in §2.5: keeping the push at the call site is what let the
    sweep and the lowering address different frames.
11. **`lower_root_statement_items_with_function_bindings` (`:9458-9503`) gains the
    same parameter.** Its four call sites (`:8368`, `:8466`, `:14902` via
    `:9446`, `:19993`) build one, with `instantiate_in_current_scope` — these
    are the three root statement lists whose Environment Record already exists.
    This is M5's fix, and it fixes the script top level and every function body
    in the same edit.

    Order at each: the sweep must run *after*
    `prepare_root_function_binding_ids` / `root_function_init_statements`
    (`:9468-9469`) so a `let` that shadows a hoisted function name allocates its
    storage against the function's, and *before* the statement loop (`:9472`).
    Simplest correct placement: build it inside
    `lower_root_statement_items_with_function_bindings` itself between `:9469`
    and `:9470`, and give the function the *items* it already has — but then the
    parameter does not exist and M5 is not closed for a *new* entry. **Do it the
    other way**: the parameter is the point. Callers build the token map, the
    function's first act is `debug_assert!(!scope.is_empty() || …)`-free
    consumption. If a call site cannot build one before calling (borrow
    conflict), the constructor takes `&mut ScriptLowerer` and returns an owned
    value, so `let scope = LexicalScopeInstantiation::instantiate(self, items);
    self.lower_root_statement_items_with_function_bindings(items, fns, scope)`
    compiles — the borrow ends at the semicolon.

12. `lower_lexical_declaration` (`:16416-16462`): the identifier-declarator arm
    becomes

    ```rust
    let init = self.lower_declarator_initializer(variable.init());   // 14.3.1.2 step 4
    let (storage_name, info, value) = match scope.take(&name) {
        Some(pending) => pending.initialize(init),                   // 14.3.1.2 step 5
        None => /* not a predeclared lexical: for-head, generated, or
                   unsupported form. Fall back to the existing
                   direct_lexical_storage_name path, spelling
                   Initialization::Initialized. */,
    };
    self.declare_binding(name.clone(), info);
    statements.push(StatementIr::Lexical { mode, name: storage_name, init: value });
    ```

    The static-analysis side effects currently interleaved at `:16403-16443`
    (`static_generator_values`, `static_string_bindings`,
    `static_to_string_regexp_object_bindings`, `static_iterator_binding_values`)
    are unchanged and stay where they are; they read `variable.init()` and the
    lowered value, both of which are still available. **Do not fold them into
    `lower_declarator_initializer`** — `array_iterator_from_static_generator_values`
    (`:16417`) replaces the lowered expression entirely, so
    `lower_declarator_initializer` must be the function that decides between the
    two, or the static path must be applied before the witness is minted. Prefer
    the latter: `lower_declarator_initializer` takes `Option<&Expression>` and
    nothing else, and the static-generator substitution stays in
    `lower_lexical_declaration` producing a `TypedExpr` that is then wrapped.
    That requires a second, deliberately awkward constructor —
    `LoweredInitializer::from_substituted(TypedExpr)` — whose doc comment names
    its single call site. Naming one loophole beats leaving three.

13. `lower_lexical_binding_value` (`:32133-32162`) and `lower_using_declaration`
    (`:16512-16523`) and `lower_class_declaration` (`:18475-18486`): same shape.
    `lower_class_declaration`'s "initializer" is the ClassDefinitionEvaluation
    result already computed at `:18467-18474`, so it uses
    `LoweredInitializer::from_substituted` too, with the 15.7.16 step 2 citation.
14. Formal parameters (`:14810-14812`/`:14852-14856` and
    `:19918-19921`/`:19936-19940`): the `$tdz.` placeholder declaration stays
    (10.2.11 step 21 creates the bindings before any default evaluates), and the
    `clear_tdz_binding` loop is deleted — the subsequent `declare_binding` at
    `:14873`/`:19941` already overwrites the entry, and now overwrites it with
    `Initialized`. **The existing ordering is correct and must be preserved**:
    `default_init` is lowered at `:14849-14851`/`:19933-19935`, before the
    overwrite. Do not reorder.
15. Delete `tdz_scopes` (`:478`), its initializer (`:7730`), the two stack
    pushes/pops (`:37786`, `:37800`), and `mark_tdz_binding` / `clear_tdz_binding`
    / `is_tdz_binding` (`:37803-37832`). The 14 call sites are gone by now.

### Stage 3 — the seven Reference sites (closes M4, M6, M6b)

16. Add `resolve_binding_reference` (§2.6) next to `lookup_binding` (`:37518`).
17. Convert, in this order. All seven are independent code paths — verified by
    reading: `:30584`, `:30833` and `:30934` each have their own
    `lookup_binding`, their own `binding_storage_name` clone, their own
    `mode == Const` test and their own
    `unsupported_expr("assignment to const binding")`. None is a wrapper over
    another.

| Site | Function | Obligation | Today |
|---|---|---|---|
| `:17110` | `lower_identifier_name_inner` | 9.1.1.1.6 step 2 | the string-OR-set disjunction at `:17111-17113` — becomes one `Uninitialized` arm |
| `:31056` | `lower_identifier_assign_value` | 9.1.1.1.5 step 3, **before** the `mode == Const` test at `:31058` | absent |
| `:32037` | `lower_array_assignment_identifier_target` | step 3 (13.15.5.3) | absent |
| `:30584` | compound-assign, arithmetic (`+= -= *= /= %= **=`) | steps 2 **and** 3 (13.15.4 GetValue then PutValue) | absent, both halves |
| `:30833` | compound-assign, logical (`&&= \|\|= ??=`) | steps 2 and 3 (13.15.3) | absent, both halves |
| `:30934` | compound-assign, bitwise (`&= \|= ^= <<= >>= >>>=`) | steps 2 and 3 | absent, both halves |
| `:32935` | `lower_update` | steps 2 and 3 (13.4.4/13.4.5) | absent, both halves |

    In every case the `Uninitialized` arm is `return violation.into_throw();`.

    **One sequencing hazard, recorded not fixed.** All three compound arms lower
    the RHS *before* they resolve the binding (`:30583` before `:30584`, and the
    same shape at `:30832` and `:30933`). 13.15.4 evaluates the LHS Reference and
    performs GetValue at steps 1-2, *before* evaluating the RHS at step 3, so the
    ReferenceError should precede the RHS's side effects and today would follow
    them. Placing `into_throw()` at the existing `lookup_binding` line preserves
    the current (wrong) order; sequencing it correctly means hoisting the
    resolution above the RHS lowering in three ~300-line arms. **Do the former
    this round** and record the latter as a separate finding in the lane note —
    the observable difference is confined to `f() += x` shapes where `f()` has an
    effect and `x` is in TDZ, and reordering three large arms without a runtime
    oracle is how a second wrong answer gets introduced.
18. Delete `is_tdz_binding_storage_name` (`:10746`). Re-route its four callers:
    `:7693` → match on `binding.initialization` (§2.2); `:14692` and `:37235` →
    `TdzPlaceholderName::names_a_placeholder(name)`; `:17111` → gone, absorbed by
    17.
19. `analysis.rs`: the nine `tdz_binding_storage_name(..)` calls become
    `TdzPlaceholderName::for_source_name(..).into_string()`. No other change to
    that file; the two helper pairs (`for_of_tdz_binding_storage_names` `:2330`,
    `for_of_tdz_binding_modes` `:2347`, `for_in_tdz_binding_storage_names`
    `:2403`, `for_in_tdz_binding_modes` `:2420`) keep their signatures.

### Stage 4 — destructuring token threading (optional this round)

The four binding-pattern literals (`:31663`, `:31729`, `:31786`, `:31820`) sit in
`lower_object_binding_pattern` / `lower_array_binding_pattern`, which are shared
by `let`/`const`, `var`, and for-in/for-of heads and take no scope parameter.
Threading `&mut LexicalScopeInstantiation` through them closes M2 for
`let {a} = expr;`. Their **current order is already correct** (the `default` and
the pattern's initializer are lowered before each `clear`/`declare`), so the
value of Stage 4 is preventing future regression, not fixing a live defect. If it
is not done this round, say so in the lane note and add ledger entry **L2**; do
not leave the four sites spelling `Initialized` with no comment.

### What stays untouched

- `crates/lila-aot-wasm/**` — nothing. `environments.rs` and `heap.rs` are
  read-only integration-note targets. Batch 2's held files are not in this area's
  path at all.
- `modules/namespace.rs:466-478` and `:560-573` — **annotate only**. Add the
  premise **P1** reference to both comments. Do not remove the bail-out (C6).
- `hoist_lexical_declaration_metadata` (`:16826`) and `VarBindingInfo`
  (`:23-31`) — untouched. `is_lexical_metadata` is not a lifecycle flag; see the
  Stage 1 caveat.
- `lower_for_head_expression_with_tdz` (`:10718-10733`) — its `push_scope` /
  `declare_binding(tdz_placeholder)` / `lower_expression` / `pop_scope` shape is
  already 14.7.5.5's, and is correct. Only the constructor's spelling changes.
- The `#[cfg(test)]` assertions at `lib.rs:1005`, `:1023`, `:1082`, `:1083`,
  `:8376` — they assert on the `$tdz.` *storage names* in environment plans,
  which survive unchanged. If any of them fails to compile it is because the
  `TDZ_BINDING_STORAGE_PREFIX` re-export moved; restore the re-export rather than
  weakening the assertion.
- `binding_names.rs`, `early_error_code.rs`, `iterator_obligations.rs`,
  `reference.rs` — no relation to this area.

---

## §5. The runtime-checked ledger and the open premises

These are the only places where a test, an assertion or a human remains
load-bearing. Each says why a type cannot carry it.

| ID | What is not compile-enforced | Why | Where it is enforced instead |
|---|---|---|---|
| **L1** | A future site could hand-build `BindingInfo { …, initialization: Initialized }` for a name that has a live `PendingInitialization`, bypassing the token. | `Initialized` must be spellable — 26 of 41 literals legitimately spell it (§4.3). Making it unspellable would need a witness type that nothing reads (§2.7). | The token is `#[must_use]` and the leftover set is inspectable via `LexicalScopeInstantiation::is_empty`. Reviewable, not provable. |
| **L2** | Stage 4 not done: the four binding-pattern initializations spell `Initialized` directly rather than discharging a token. | Threading a scope parameter through two functions shared by `var` and loop heads is a larger change than the live defect justifies. | The four sites' current ordering is correct and is asserted by corpus entry 12. Delete this entry when Stage 4 lands. |
| **L3** | An *eighth* Reference-shaped site, newly written, that builds `ExprIr::Identifier(name)` from a `&str` without calling `resolve_binding_reference`. | See obligation **O1**. | The seven converted sites are exhaustive over today's tree; a new one is a review item. |
| **L4** | The three compound-assign arms resolve the binding after lowering the RHS, so the 9.1.1.1.6 step 2 throw follows the RHS's side effects where 13.15.4 puts it before them. | Fixing it means hoisting the resolution above ~300 lines of arm in three places. | Stage 3's recorded finding. Delete this entry when the hoist lands. |
| **P1** | `report_shadowed_namespace_globals` (`namespace.rs:471`) and the namespace-alias arm (`:561`) refuse to compile a module that shadows `Object`/`Symbol`, on the ground that such a binding is in TDZ for the whole merged scope. | The premise is *the spec behaviour*, which Stage 2 makes real for the first time. Removing the bail-out additionally requires **P2**, because a merged-scope `Object` whose storage is `Dynamic` gets no runtime check. | Annotated comment referencing this contract. Removal is a later lane, gated on P2. |
| **P2-read** | Nothing in `lila-ir` relates `Initialization::Uninitialized` to `allocate_binding`'s `BindingStorage` choice (`lila-aot-wasm/src/environments.rs:900-945`). `read_binding_to_locals` (`:850`) emits the `ENV_SLOT_UNINITIALIZED_TAG` comparison (`:876-895`) **only** for `BindingStorage::EnvSlot`. `Fixed` and `Dynamic` have no check, and a `Dynamic` tag local is zero-initialized — and `ValueKind::Undefined.tag()` returns `0` (`lila-ir/src/ir.rs:225`). | `BindingStorage` is defined in a crate this round does not edit; batch 2 holds files in it. | Integration note (see below), plus the fact that the compile-time throw this contract installs at the seven IR sites covers every *statically resolved* read. P2-read governs only the reads that reach a runtime slot: closures over a binding whose storage the capture analysis did not promote to `EnvSlot`. |
| **P2-write** | `write_binding_from_locals` (`environments.rs:320-348`) emits **no** `ENV_SLOT_UNINITIALIZED_TAG` comparison in *any* arm, `EnvSlot` included — unlike `read_binding_to_locals`. So 9.1.1.1.5 step 3 has no runtime backstop at all for a **closure write** to an uninitialized binding, at every storage class. Nor does the compile-time layer reach it: a captured name enters the inner function as a capture alias declared `Initialization::Initialized` (`lowering.rs:14890-14901`, `:20051`, `:20070`), so `lower_identifier_assign_value` (`:31353`) takes the `Initialized` arm. This is where §7 corpus entry 7 actually lands, and it is **not** covered by P2-read, which scopes itself to reads. | Same crate boundary. | Integration note. The fix is the same `ENV_SLOT_UNINITIALIZED_TAG` comparison-and-throw in `write_binding_from_locals`'s `EnvSlot` arm, with the *initializing* write exempt — 9.1.1.1.4 must be able to write the slot while it is still tagged `-1`. The IR already separates the two: an InitializeBinding is `StatementIr::Lexical` and a SetMutableBinding is `ExprIr::AssignIdentifier`, so the exemption costs nothing to express at the aot-wasm boundary. |
| **O1** | The obligation "every Environment Record read/write goes through the state" is not total, because `ExprIr::Identifier(String)` and `ExprIr::AssignIdentifier { name: String, .. }` accept a bare `String`. | `ir.rs` is not owned by this area, and the payload change ripples into `lila-aot-wasm`. **Measured price**: `ExprIr::Identifier(` has **80** construction sites in `lila-ir` and **30** in `lila-aot-wasm`; `ExprIr::AssignIdentifier` has **17** and **10**. 137 sites across two crates. | Named here as an open proof obligation with its price, in the round-2 style. Not attempted this round. |

**Integration note (goes in `target/lane-notes/environment-record-tdz-theory-integration.md`).**
For a later batch that owns `lila-aot-wasm`: `allocate_binding`
(`environments.rs:900`) picks `EnvSlot` only when `owned_env_slot(&name)` is
`Some` — i.e. only when the capture analysis decided the binding is captured. The
uninitialized tag is therefore a property of *capture*, not of *lexicality*. The
clean fix is to make the choice a function of the binding's `Initialization` as
well: a lexical binding that has any uninitialized window needs a slot that can
hold the sentinel, which `Fixed` (payload local + static kind) structurally
cannot. Either force `Dynamic` for such bindings and give `Dynamic` the same tag
comparison, or force `EnvSlot`. `ENV_SLOT_UNINITIALIZED_TAG` is `-1`
(`heap.rs:947`), which no `ValueKind::tag()` returns, so the sentinel is safe in
a `Dynamic` tag local; the missing piece is initializing that local to `-1`
rather than leaving it zero, and emitting the comparison in the `Dynamic` arm of
`read_binding_to_locals`.

---

## §5b. ENCODER RECORD — what was built, and what moved to the ledger

Written blind (no `cargo`/`rustc` was run; the integrator owns the compile gate).
Files touched: `crates/lila-ir/src/binding_lifecycle.rs` (new),
`lowering.rs`, `analysis.rs`, `names.rs`, `lowering_helpers.rs`, `lib.rs`,
`modules/namespace.rs`.

### Mistake classes discharged as promised

| # | Discharged | Mechanism as landed |
|---|---|---|
| **M1** | yes | `BindingInfo::initialization: Initialization`, no `Default`. All 41 literals now spell it; omitting it at a 42nd is `E0063`. |
| **M3** | yes | `tdz_scopes`, `mark_tdz_binding`, `clear_tdz_binding`, `is_tdz_binding` deleted. A re-introduction is `E0609`. |
| **M4** | yes | `is_tdz_binding_storage_name` and `lowering_helpers::tdz_binding_storage_name` deleted. `TdzPlaceholderName::for_source_name` is the only mint (9 `analysis.rs` sites), `::names_a_placeholder` the only predicate (2 name-only sites). No read/write site consults a prefix. |
| **M5** | yes | `LexicalScopeInstantiation`, `#[must_use]`, private field, constructors `instantiate`/`instantiate_switch` only. All three statement-list entries take one; a fourth is `E0061`. This lands the top-level *and* every-function-body sweep of §0.4. |
| **M6 / M6b** | yes | `BindingResolution` + `TdzViolation`, produced only by `BindingResolution::of`, which is a total function of the resolved record. All seven Reference-shaped sites match it exhaustively; a new state is `E0004` at all seven. |
| **M8** | yes | The `boa_ast::Declaration` match in `LexicalScopeInstantiation::instantiate_item` is exhaustive over all six variants; the four function forms are named with their spec ground. |
| **M2** | yes, by a different mechanism than §2.4 predicted | `PendingInitialization::initialize` takes a `LoweredInitializer` and consumes `self`, so double-initialization is `E0382` and initialization-before-lowering has no value to pass. **Additionally** the separate `clear_tdz_binding` call — the thing whose *order* was the hazard at ten sites — no longer exists: the transition is the `declare_binding` that already had to be built out of the lowered initializer. M2's residue is L1 and L2 below. |
| **M2b** | yes, by two mechanisms (**strengthened by the discrepancy-fixer pass — see §5c**) | The token carries the created storage name, and since §5c it does not release it: `initialize` returns an `InitializedBinding` whose only exit performs both the `declare_binding` and the `StatementIr::Lexical`. `ScriptLowerer::direct_lexical_storage_name` additionally returns the created name whenever the current scope already holds an `Uninitialized(Allocated)` entry, which is what makes the untokened destructuring paths agree. This is not decorative duplication: the token proves the *ordering* and now the *name*, the accessor covers the paths that take no token (L2). |
| **M7** | not attempted, as designed | Premise **P2**; integration note written. |

### Moved to the ledger, with reasons

| ID | What |
|---|---|
| **L1** (unchanged) | `Initialization::Initialized` is spellable; 38 literals legitimately spell it. |
| **L2** (was conditional, now live) | Stage 4 not done. `lower_object_binding_pattern` / `lower_array_binding_pattern` take no `&mut LexicalScopeInstantiation`. They are shared by `let`/`const`, `var` and for-in/of heads, and their four initialization sites carry a comment naming this entry. Their ordering is already correct and their storage name is the created one via the accessor rule, so what is missing is the *proof*, not the check. |
| **L3** (unchanged) | An eighth, newly written Reference site. Obligation **O1**. |
| **L4** (unchanged) | The arithmetic compound-assign arm resolves after lowering the RHS, so its throw follows the RHS's effects where 13.15.4 puts it before them. Recorded at the site. The *logical* and *bitwise* arms resolve before the RHS is lowered, so they are already correct — this entry now covers one arm, not three. |
| **L5** (closed by the T08 identifier-reference follow-up) | `DestructuringTargetIr::AssignmentIdentifier` now owns an `IdentifierWriteReferenceIr`. Its uninitialized constructor requires and consumes the `TdzViolation`; its `Throw` disposition is emitted only after destructuring has evaluated the extracted value and any default initializer. The former lowering `unsupported` is gone, so 9.1.1.1.5 step 3 reaches a runtime ReferenceError without moving it ahead of 13.15.5.3's earlier work. |
| **L6** (NEW, corrected) | `LoweredInitializer::evaluated` is `pub(crate)` and accepts any `TypedExpr`, so it is a **general constructor**, not a closed list — the doc comment that claimed six call sites was a doc comment asserting an invariant the type does not carry, which is round 2's finding 8. The measured count is **8**: `lowering.rs:16376`, `:16474`, `:16497`, `:16546` (the four async/generator staging paths), `:16602` (the ordinary identifier declarator, which the old list omitted), `:16679` (`lower_using_declaration`, omitted), `:18657` (class), `:31937` (`lower_object_pattern_lexical_binding_from_value`, omitted). What this leaves open is precisely M2's ordering half: `initialize(evaluated(TypedExpr::undefined()))` followed by lowering the real initializer compiles. Each of the eight bodies has been read and does produce its value first; that is review, not proof. §4 item 12 anticipated one such constructor (`from_substituted`); a second constructor differing only in name would have been the decoration AGENTS.md warns about. |
| **L8** (narrowed by the T08 identifier-reference follow-up) | Rust cannot make a value undroppable: a future `BindingResolution::Uninitialized(_) => {}` still compiles, and `BindingInfo::initialization` remains inspectable inside the crate. The seven current arms either consume the witness with `into_throw()` or, for deferred destructuring, into `IdentifierWriteReferenceIr`; the former L5 wildcard/unsupported exception no longer exists. Review-checked, not globally proved. |
| **L9** (NEW) | The state is **flow-insensitive**. `Initialization` tracks *lowering order*, not control flow, so a `switch` case body that falls through past an earlier case's declarator reads the binding as initialized: `switch (x) { case 1: let a = 1; case 2: a; }` — case 1's declarator runs `scope.take("a")` and flips the shared CaseBlock entry to `Initialized` (`lowering.rs:13840` is one token map; `:13881` lowers the cases in source order into one `scopes.last_mut()` frame), so case 2's read at `:17285` takes the `Initialized` arm. With `x === 2` the binding is uninitialized at run time and 9.1.1.1.6 step 2 requires a ReferenceError. | A compile-time per-binding state cannot be flow-sensitive without a dataflow merge. Pre-existing — the deleted `mark`/`clear` pair had exactly the same order sensitivity — and not a regression. | Nothing, today. Closing it needs either **P2-read**'s runtime sentinel for every lexical binding in a CaseBlock with more than one reachable entry, or a merge over case bodies that re-marks a name `Uninitialized` for cases that do not dominate its declarator. Not attempted in this lane. |
| **L7** (NEW) | The sweep for `lower_root_statement_items_with_function_bindings` is built by the **caller**, so it runs *before* `prepare_root_function_binding_ids` rather than after, as §4 item 11 asked. The parameter is the point (M5's `E0061`) and the caller-side construction is what makes it one. The order is observationally equivalent: a lexical declaration and a hoisted function declaration that bind the same name in the same scope are an early error, so no name can be reached by both sweeps. |

### Deviations from §2 and §4 worth the reviewer's attention

1. **`PendingInitialization` has no `initialize_value`.** §2.4 gave it one; nothing read it once Stage 4 moved to L2, and a field constructed at N sites and read at 0 is round-2 finding 3. It carries `source_name`, `mode` and `storage_name` — `source_name` was dropped in the encoder pass for the same reason and **restored in §5c**, where `InitializedBinding::declare` gave it a reader.
2. **`LexicalScopeInstantiation` has no `is_empty`.** Same reason — no consumer.
3. **The token is threaded through `lower_statement_list_item` -> `lower_declaration` -> `lower_lexical_declaration` / `lower_using_declaration` / `lower_class_declaration`.** §4 assumed this plumbing without pricing it; it is cheap because `lower_statement_list_item` has exactly three call sites (the three statement-list entries) and `lower_declaration`, `lower_lexical_declaration`, `lower_using_declaration` and `lower_class_declaration` have exactly one each.
4. **`lower_lexical_binding_value` is the single InitializeBinding site for the five identifier-declarator paths** and takes `LoweredInitializer` + `Option<PendingInitialization>`, rather than each path re-spelling the transition.
5. **`ScriptLowerer` gained two `pub(crate)` methods**, `create_lexical_binding` and `interner`, so the exhaustive `Declaration` match can live in `binding_lifecycle.rs` while the private-field constructor property of `LexicalScopeInstantiation` is preserved.
6. **`lower_class_declaration` still allocates its storage name before ClassDefinitionEvaluation**, not after, because `direct_lexical_storage_name` can consume a `$lexN` temporary index and moving that consumption past `lower_class_common` would renumber every generated name inside the class body.

---

---

## §5c. DISCREPANCY-FIXER RECORD — what the dry run moved

Written blind (no `cargo`/`rustc`; the integrator owns the compile gate). Files
touched: `crates/lila-ir/src/binding_lifecycle.rs`,
`crates/lila-ir/src/lowering.rs`.

### Closed by a type this pass

| Obligation | What landed |
|---|---|
| **M5b** — the sweep running into the wrong Environment Record | `LexicalScopeInstantiation` gained a private `frame: InstantiatedFrame` field. `instantiate` and `instantiate_switch` perform the `push_direct_lexical_scope`; `instantiate_in_current_scope` (new, three root call sites) does not; `finish(self, lowerer)` consumes the token and pops iff it pushed. `lower_statement_items` and `lower_root_statement_items_with_function_bindings` call `finish` at the end, and `lower_switch` calls it where its `pop_scope()` was. Five caller-side pushes were deleted (`Statement::Block`, `lower_try`'s try and finally, `lower_switch`); the catch push stays, annotated as 14.15.3's `catchEnv`. The pop of an instantiation frame is now reachable only through the token. |
| **M2b-name** — initializing a predeclared binding under a recomputed name | `PendingInitialization::initialize` returns `InitializedBinding` (private fields, `#[must_use]`) instead of `(String, BindingInfo, TypedExpr)`. Its only exit, `declare(&mut ScriptLowerer) -> StatementIr`, performs the scope write and emits the node, so the tokened paths have no `String` to substitute. Four call sites converted; the untokened halves go through the named `InitializedBinding::without_creation`, which is where L2 now sits. `ScriptLowerer::declare_initialized_binding` is the one new `pub(crate)` accessor this needed. |
| **`UninitializedStorage`'s second consumer** | `direct_lexical_storage_name`'s reuse branch was `== Uninitialized(Allocated)`; it is an exhaustive `match`, so a third variant is `E0004` at both consumers rather than at one. |

### Corrected in this document, not in the code

- §3 **M2** (`E0425` claim), §3 **M2b** (`E0616` claim), §3 **M6** (`E0599`
  claim) and §2.4's "no value of this type in scope" all overstated what the
  types carry. Restated, with the residues moved to ledger **L6** and **L8**.
- §7 corpus **entry 7**'s trace was wrong end to end (it is a capture-alias
  write, not a same-scope one) and its verdict changes from "ReferenceError" to
  "unchanged, open". Premise **P2** is split into **P2-read** and **P2-write**,
  because the write path has no runtime check in *any* storage arm.
- §7's two non-program checks both failed as literally written. Restated and
  re-measured: `grep -c 'initialization:'` is 36 in `lowering.rs` and 3 in
  `binding_lifecycle.rs`; the tombstone grep is an `-E` word-boundary form that
  returns only comment lines.
- New ledger rows **L8** (the arm's content, and the by-hand classification
  route) and **L9** (flow insensitivity, with the `switch` fall-through
  witness).

## §6. Deviations from the area brief

Restated compactly, because these are the points where following the brief
verbatim produces a defect or a decoration. All are argued above.

- **§6.1** — 41 `BindingInfo` literals, not 44; 4 `mark` and 10 `clear` sites, not
  5 and 11. (§0.1, §0.2)
- **§6.2** — M5 is not confined to the script/module top level. Every **function
  body** goes through the same unpredeclared entry. (§0.4)
- **§6.3** — Item (4) cannot be implemented as written. The nine `analysis.rs`
  mints and two of the four prefix tests have no `BindingInfo` to route through;
  the prefix is a name domain and stays one, closed by a newtype rather than
  collapsed into the state. (§0.5, §2.3)
- **§6.4** — Item (6) understates what is reachable. The write side is **in
  scope** and must be closed at six sites, and the *read* side has four further
  holes the brief does not mention (three compound-assign arms and update).
  Seven Reference-shaped sites in total, of which one is checked today. (§0.6,
  §0.7)
- **§6.5** — The `Initialization` enum needs a payload on its `Uninitialized`
  variant. A bare two-state field makes `lexical_storage_name:7693` wrong, and
  the resulting mis-shadowing is silent. (§2.2)
- **§6.6** — The four `clear` sites at `:31660`/`:31726`/`:31783`/`:31817` are
  destructuring-pattern sites, not for-head sites, and there is no
  CreatePerIterationEnvironment logic at any of them. Corpus entry 9 is re-aimed.
  (§0.3, §7)
- **§6.7** — A naive M5 fix (just call the predeclare wrapper at the root)
  **introduces** a storage-name split at every non-`direct_lexical` scope. This
  is not in the brief and is the strongest argument for the token carrying the
  name. (§2.2, M2b)
- **§6.8** — Two additions the brief does not have: M8 (the `_ => {}` over
  `boa_ast::Declaration`) and the exact price of obligation O1.

---

## §7. Dry-run corpus, with the trace each entry must produce

Twelve entries. For each: the mistake class, the path through the code, the
answer today, and the answer the contract requires. The dry-runner executes these
symbolically against the encoder's landing; no case is run.

| # | Source | Class | Trace | Today | Required |
|---|---|---|---|---|---|
| 1 | `test/language/statements/let/global-use-before-initialization-in-prior-statement.js` — `x; let x;` | **M5** | `lower_root_statement_items_with_function_bindings:9458` → no sweep → `lower_identifier_name_inner:17110` → `lookup_binding` misses `scopes`, hits the `var_bindings` **lexical-metadata** entry that `hoist_lexical_declaration_metadata:16826` inserted → `:37532` builds `BindingInfo { mode: Var, storage_name: "x" }` → `ExprIr::Identifier("x")` | reads slot `x` | Stage 2's sweep puts an `Uninitialized(Allocated)` entry in `scopes.last()`; `lookup_binding` finds it first (`:37519` before `:37523`); `resolve_binding_reference` returns `Uninitialized`; ReferenceError |
| 2 | `…/const/global-use-before-initialization-in-prior-statement.js` | **M5** on the `const` arm | Same, through `LexicalDeclaration::Const` → `BindingMode::Const` at `:13762-13764` | same | same. Confirms the gap is not `let`-specific and that `instantiate`'s mode mapping survives the move |
| 3 | `…/let/global-use-before-initialization-in-declaration-statement.js` — `let x = x;` at top level | **M2 × M5** | Sweep mints the token; `lower_declarator_initializer(Some(x))` lowers the `x` read **while the entry is still `Uninitialized`**; `initialize` then runs | reads undefined | ReferenceError. This is the case a fix to M5 alone would get wrong by clearing first |
| 4 | `…/let/block-local-use-before-initialization-in-prior-statement.js` | **M1/M4**, regression anchor | `lower_block:10314` → sweep → `:17110` → `Uninitialized(Allocated)` | **already ReferenceError** | unchanged. The working sibling of entry 1 — same program, different scope, different answer today, same answer after |
| 5 | `…/let/block-local-use-before-initialization-in-declaration-statement.js` | **M2** in a block | The order `PendingInitialization` must preserve: `:16416-16423` lowers, `:16446` clears, `:16447` declares | already correct | unchanged, now unforgeable |
| 6 | `…/let/global-closure-get-before-initialization.js` — `function f(){ return x+1 } … let x;` | **M7 / P2** | `x` is captured → `allocate_binding:900` picks `EnvSlot` → the runtime check at `:876-895` fires inside `f` | ReferenceError, from the **runtime** check | unchanged. Pair with entry 1: same source-level state, one enforced at compile time and one at run time, by two mechanisms that nothing relates. This pair is why P2 is a named premise |
| 7 | `…/let/function-local-closure-set-before-initialization.js` — `function f(){ x = 1 } … let x;` inside an IIFE | **P2-write** (not M6) — **trace corrected** | The `let x` is in the enclosing IIFE body, **not** in `f`'s body, so `f`'s own sweep never creates it. `x` reaches `f` as a **capture alias**, declared `Initialization::Initialized` (`lowering.rs:14890-14901`, `:20051`, `:20070`), so `lower_identifier_assign_value` (`:31353`) takes the `Initialized` arm and the compile-time check cannot fire. Nor is there a runtime backstop: `write_binding_from_locals` (`environments.rs:320-348`) emits no `ENV_SLOT_UNINITIALIZED_TAG` comparison in **any** arm, `EnvSlot` included | writes the slot; no throw | **still writes the slot; no throw.** This entry is not closed by this lane and the earlier claim that it was — "f's body is a statement-list scope too (§0.4)" — was wrong: it confused the scope the *read/write* is in with the scope the *declaration* is in. 9.1.1.1.5 step 3 is unenforced at both layers for every closure write to an uninitialized binding. Ledger **P2-write**. Its sibling entry 6 is the read half, which *is* enforced, by the runtime `EnvSlot` check — the asymmetry between the two is the whole content of the premise |
| 8 | `…/switch/scope-lex-const.js` | **M3** | `lower_switch:13660` `push_direct_lexical_scope` → `instantiate_switch` → case bodies. The second predeclaration entry point, and the reason the two-stack invariant had two producers | works today | unchanged, with `tdz_scopes` gone. Verify the sweep spans **all** cases (14.12.4 instantiates the CaseBlock once), not per case |
| 9 | `…/for-of/head-let-bound-names-fordecl-tdz.js` | **M2/M4** on the placeholder path — **re-aimed** (§0.3) | `lower_for_head_expression_with_tdz:10718` pushes a scope, declares a `Placeholder`, lowers the iterable, pops. The head's read of the loop name resolves to the placeholder | ReferenceError via the `$tdz.` prefix test at `:17111` | ReferenceError via `Initialization::Uninitialized(Placeholder)` — **the prefix must no longer be consulted at `:17111`**. This is the entry that proves M4 closed |
| 10 | **ADVERSARIAL** `print(Object); let Object = 1;` at script top level | **M5** + premise **P1** | As entry 1, but the name also resolves as a builtin global. Order in `lower_identifier_name_inner`: `lookup_binding:17110` runs **before** the `StandardBuiltinId::all_globals()` fallback at `:17149`, so a predeclared scope entry wins | prints the `Object` constructor | ReferenceError. Trace must also state what changes for the module path: `namespace.rs:471` still refuses the program (C6), so the *module* answer stays "unsupported" and only the *script* answer changes. Both must be reported |
| 11 | **ADVERSARIAL** `let x = x;` at script top level, traced against `{ let x = x; }` | **M2 × M5** | Two lowering entries: `lower_root_statement_items_with_function_bindings:9458` and `lower_block:10314` → `lower_statement_items:9505`. Both must reach the same state through `resolve_binding_reference` | outer: undefined; inner: ReferenceError | both ReferenceError. Additionally: the outer case must confirm **M2b** — the storage name in the emitted `StatementIr::Lexical` is the token's, and equals the one every earlier read of `x` resolved to. Compare against `lexical_storage_name:7689`'s answer *with* the predeclared entry present, which is `$lexN` — that difference is the defect the token prevents |
| 12 | **ADVERSARIAL** `{ let a = 1; { typeof a; let a; } }` and `{ let a = 1; { a = 2; let a; } }` | **M4 / M6** | First: 13.5.3 exempts only *unresolvable* References, so `typeof` on an uninitialized binding throws — the reader must consume one state, not OR two encodings. Second: the write-side counterpart at `:31056`. Also exercises `UninitializedStorage::Allocated`'s shadowing role — the inner `let a` must allocate a name distinct from the outer's | first: ReferenceError (block path works); second: **assigns to the outer `a`'s slot silently** | both ReferenceError, and the inner binding's storage name must differ from the outer's |

Two further checks the dry-runner must perform that are not programs:

- **Count check.** Both of the checks this section first stated fail as written
  and must not be run in that form; a reviewer who runs them concludes the
  landing is broken when it is not.

  - `grep -c 'initialization:'` over the literal sites **cannot** be 41: six of
    the 41 literals became constructors (`BindingInfo::tdz_placeholder`,
    `BindingInfo::initialized`, `create_lexical_binding`,
    `PendingInitialization::initialize`, and the two the sweep absorbed). The
    measured invariant is
    `grep -c 'initialization:' crates/lila-ir/src/lowering.rs` == **36**
    (35 literals + the struct-field declaration) and == **3** in
    `binding_lifecycle.rs`; 35 + 6 = 41.
  - `grep -rn 'tdz_binding_storage_name'` **cannot** be zero: it collides by
    substring with `for_of_tdz_binding_storage_names` /
    `for_in_tdz_binding_storage_names` (`analysis.rs:2339`, `:2419`, `:4153`,
    `:4303`), which §4 item 19 explicitly preserves. The correct form is

    ```sh
    grep -rnE '\btdz_scopes\b|\bmark_tdz_binding\b|\bclear_tdz_binding\b|\bis_tdz_binding\b|\bis_tdz_binding_storage_name\b|(^|[^_])\btdz_binding_storage_name\b' crates/lila-ir/src/
    ```

    which must return only comment lines (the two tombstone comments that name
    the deleted helpers). Both restated checks pass on the current tree.
- **Prefix-role check.** `TDZ_BINDING_STORAGE_PREFIX` must have exactly the uses
  §0's table lists, plus its uses inside `binding_lifecycle.rs`. Any occurrence in
  a conditional that decides whether to throw is M4 reopening.
