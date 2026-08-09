# Contract: module binding-name domains — `[[LocalName]]` vs `[[ExportName]]` vs the merged storage name

Area: *Module binding-name domains: `[[LocalName]]` vs `[[ExportName]]` vs
merged storage name*
Stage: FORMALIZER. This document is normative for the encoder and is the
oracle the dry-runner checks against. No source code is edited in this stage.

> **Read §10 first.** A dry-run discrepancy pass amended this document after the
> encoding landed. §10 supersedes every claim it names — including §1.3, §1.4
> M2/M3, §2.4, §2.6 V6, §3 M4/M5, §4 K1/K2/K4, §5.4, §5.6, §9.3 and §9.4, and
> ledger entries R1/R2/R3. Do not cite §§1–9 without checking §10.

Owned files:

- `crates/porffor-ir/src/binding_names.rs` (new)
- `crates/porffor-ir/src/names.rs` (only the region at lines 19–125)
- `crates/porffor-ir/src/lib.rs` (only the `pub use modules::{…}` block at
  lines 76–83 and the `mod`/`pub use` lines for the new module)
- `crates/porffor-ir/src/modules/{mod,record,link,graph,namespace,source,dynamic,early}.rs`
- `docs/rust-rewrite/contracts/Module binding-name domains: [[LocalName]] vs [[ExportName]] vs merged storage name.md` (this file)

The area brief names this file `docs/rust-rewrite/contracts/module-binding-names.md`.
That path exists as a three-line pointer to this document; this document is
the contract.

Every count here was produced by a command, not an estimate. Where a count or
a claim in the area brief disagreed with the repository, the measured value is
used and the correction is stated in §0. Items marked **[dry-run obligation]**
are claims the dry-runner must confirm or refute before the encoder's work is
accepted.

---

## 0. Corrections to the area brief, measured

The brief is right about the shape of the problem — three name domains, one
of them minted, confused today because all three are `String` — and wrong
about which functions carry the mapping and which functions exist at all. Six
corrections, all load-bearing. **The encoder must work from the numbers here.**

| # | Brief says | Measured | Command / evidence |
|---|---|---|---|
| C1 | "the nine name-minting functions currently at `names.rs:35-125`" implement the source-name → merged-storage-name map, which "must be applied exactly once" | **`module_storage_prefix` is not that map.** It is applied to a *source-spelled* name in exactly **one** place workspace-wide — `graph.rs:631`, `format!("{}{name}", module_storage_prefix(*module))` — and that value feeds a field the product path never reads (§0/C4). The real `[[LocalName]]` → merged-name map is `modules::record::module_binding_reference` (`record.rs:251`), which applies **no prefix**: it is the identity except for `*default*`, which becomes `$d{unit}$`. Its 3 product call sites are `link.rs:495`, `link.rs:635`, `namespace.rs:244`. | `grep -rn "module_storage_prefix\|module_binding_reference" crates/ --include=*.rs`; `link.rs:20-27` and `link.rs:59-66` state the design: "the importer's name and the exporter's name are the same binding". |
| C2 | Nine minting functions to be given directional signatures | **Four of the nine have zero product call sites and one more writes a value nothing reads.** `module_function_id` (`names.rs:119`) has **0** callers workspace-wide; `module_function_id_prefix` (`names.rs:60`) and `is_user_function_id` (`names.rs:113`) are reached only from it; `module_import_meta_cell_name` (`names.rs:95`) has **0** callers. `module_component_completion_cell_name` (`names.rs:104`) has 1 caller (`dynamic.rs:244`) which writes `DynamicComponentIr::completion_cell`, a field with **0** readers. All five are `pub`, so none produces a dead-code warning. | `grep -rn "\bmodule_function_id\b" crates/ --include=*.rs` → 1 hit (the definition). Same for `module_import_meta_cell_name`. `grep -rn "completion_cell" crates/ --include=*.rs` → 4 hits: 2 doc comments, 1 declaration, 1 construction. |
| C3 | `module_import_meta_cell_name` is one of six per-unit cell minters to be retyped | It is not merely dead, it is **unusable**. It returns `$m{unit}$import.meta`, whose minimum length is 15 bytes; `rewrite_import_meta` (`record.rs:487-489`) must fit the replacement inside the `import.meta` span, minimum 11 bytes. It also contains `.`, so it is not an `IdentifierReference` and `namespace::is_binding_identifier` (`namespace.rs:211`) rejects it. The live function is `record::import_meta_binding_name` (`record.rs:386`), returning `$m{unit}$meta` (minimum 8 bytes). Two functions for one job, with different suffixes, one of them broken. | `record.rs:386-388` vs `names.rs:94-97`; length check at `record.rs:483-489`. |
| C4 | Three name domains | **Four.** There are two disjoint generators of merged names, and separately an *IR-level* cell name that is not a merged name at all. `ModuleNamespaceExportIr::cell` (`namespace.rs:100-109`) is documented in the source as "the *IR-level* cell name (`$m0$value`) … the generated namespace source reads `Self::target` through `namespace_target_reference` and never this field". Its only reader is `ModuleNamespaceIr::cell_for` (`namespace.rs:163`), whose only call site is a test (`namespace.rs:899`). See §1.3 and §5.2. | `grep -n "cell_for\|\.cell\b" crates/porffor-ir/src/modules/namespace.rs` |
| C5 | "All 16 call sites of the module name-minting functions … (link.rs 5, record.rs 4, namespace.rs 3, graph.rs 2, source.rs 1, dynamic.rs 1)" | **19** product call sites of the `names.rs` minters, distributed `namespace.rs` 10, `graph.rs` 3, `dynamic.rs` 3, `record.rs` 2, `link.rs` 1, `source.rs` 0, `early.rs` 0. Plus **5** product call sites of the two minters that live in `modules/record.rs` (`module_binding_reference` 3, `import_meta_binding_name` 2). **24 total.** Full table in §5.3. The containment claim itself is confirmed: zero call sites in `lowering.rs`, zero outside `crates/porffor-ir/src/modules/`. | §5.3, produced by splitting each file at its `#[cfg(test)]` line and excluding `use` lines and doc comments. |
| C6 | Commit `e27c01b1e` is a module-linking fix | Confirmed, as a **sub-item**. `e27c01b1e` is "Fix the unhandled-rejection swallow that scored failures as passes"; its message states verbatim: "`export default`, renamed import bindings and `export * from` all link. These were capped by module bodies being merged on source text, so same-named top-level bindings collided and a renamed binding had no distinct storage; fixing the storage naming closed all three." The `names.rs` diff in that commit is exactly the 16 lines that added `module_default_binding_name`. | `git show e27c01b1e -- crates/porffor-ir/src/names.rs` |

Two further facts the brief did not have, both strengthening the case:

- **The `export default` byte budget and the `import.meta` byte budget are
  both live, both tighter than the doc comment says, and both currently
  unenforced.** `names.rs:47-49` records only the first. The second is at
  `record.rs:483-489` and is *tighter*: 11 bytes, not 14. Both are derivable
  in `const` from the unit-id cap. §1.4.
- **`ModuleLinkErrorIr::AmbiguousExport.export_name` is filled from an
  `[[ImportName]]`** at `graph.rs:783` (`import_name_text(&entry.import_name)`).
  This is not a bug: §1.1/E2 shows `[[ImportName]]` and the target's
  `[[ExportName]]` are the *same* domain. The contract records it because it
  determines the type of `ImportNameIr::Name` (§2.3).

---

## 1. Spec basis

### 1.1 The domains, from ECMA-262

**§16.2.1.4 ImportEntry Records.** Table: `[[ModuleRequest]]` a String,
`[[ImportName]]` a String *or* `namespace-object`, `[[LocalName]]` a String.
The table's own prose for `[[LocalName]]`: "The name that is used to locally
access the imported value from within the importing module." For
`[[ImportName]]`: "The name under which the desired binding is exported by the
module identified by `[[ModuleRequest]]`."

**§16.2.1.5 ExportEntry Records.** Table: `[[ExportName]]` a String or `null`,
`[[ModuleRequest]]` a String or `null`, `[[ImportName]]` a String,
`all`, `all-but-default`, or `null`, `[[LocalName]]` a String or `null`.
`[[ExportName]]`: "The name used to export this binding by this module."
`[[LocalName]]`: "The name that is used to locally access the exported value
from within the importing module."

The two tables are the whole argument: the spec gives `[[LocalName]]` and
`[[ExportName]]` different *definitions* and lets them differ in value, and
`export { a as b }` is precisely the production that makes them differ.

**§16.2.1.6.2 ResolveExport(exportName, resolveSet).** Matches
`e.[[ExportName]]` against `exportName` (steps 4.a, 5.a), and returns a
ResolvedBinding Record whose `[[BindingName]]` is either `e.[[LocalName]]`
(step 4.a.i, the local case) or is recursively obtained from the *requested*
module (step 5.a.iii), or is `namespace` (step 5.a.ii). So a ResolvedBinding's
`[[BindingName]]` is a `[[LocalName]]` **of the resolving module**, not of the
module that asked — a third coordinate `(module, bindingName)`, which is why
`ResolvedBindingIr::Resolved` carries both.

**§8.2.2 BoundNames**, `ExportDeclaration : export default …`: for the
anonymous forms the result is `« "*default*" »`. §16.2.3.7's note and §8.2.2
together make the point the whole contract rests on: `*default*` is chosen
because no `BindingIdentifier` can produce it. Name-domain disjointness is
being used by the specification *as a correctness mechanism*.

**§9.1.1.1 Declarative Environment Records** — `CreateMutableBinding(N, D)`,
`CreateImmutableBinding(N, S)`, `InitializeBinding(N, V)`, `GetBindingValue(N,
S)` — all key on an `N` drawn from BoundNames, and each Environment Record has
its own name space. §16.2.1.6.4 `InitializeEnvironment` creates the module
environment's bindings from `[[ImportEntries]]` (`CreateImportBinding` with
`in.[[LocalName]]`, step 5.c.iii/5.d.iv) and from
`LexicallyScopedDeclarations`/`VarScopedDeclarations` of the body.

### 1.2 The choice this implementation makes, and what it costs

ECMA-262 gives each module its own Environment Record, so two modules may both
declare `let x` with no interaction. **This compiler does not build per-module
environments.** `modules::link` merges every unit's *source text* into one
Script and lowers it once (`link.rs:11-31`), so all units' top-level bindings
share one activation environment. The three reasons are stated at `link.rs:12-27`
and are sound: span-derived `FunctionId`s, per-lowering `owned_env_slots`
numbering, and — the one that is a *feature* — a cross-module read becoming an
ordinary read of the exporter's own cell, which is what makes an imported
binding live without runtime indirection.

The cost is a fourth name domain that the spec does not have: **the merged
name**, the name a binding is spelled by in the one merged Script scope. The
map from `[[LocalName]]` to it must be injective across the whole graph, and
today it is not: two units that both declare top-level `x` are *reported* as
unlinkable (`link.rs:495-503`) rather than renamed. That report is the
placeholder for the renaming pass, and it is the reason the merged-name domain
must be a distinct type now: the renaming pass will change the map, and
everything downstream of it must be forced to go through the map rather than
through a `String`.

### 1.3 The four domains, named

| # | Domain | Spec anchor | In-repo values today |
|---|---|---|---|
| **D1** | `[[LocalName]]` — a name in one module's own environment | §16.2.1.4, §16.2.1.5, §8.2.2 | `ImportEntryIr::local_name`, `LocalExportEntryIr::local_name`, `ModuleEnvBindingIr::name`, `ResolvedBindingIr::Resolved{binding: Name(_)}`; the constant `MODULE_ANONYMOUS_DEFAULT_LOCAL_NAME` |
| **D2** | `[[ExportName]]` — a name in a module's export table | §16.2.1.5, §16.2.1.6.2 | `LocalExportEntryIr::export_name`, `IndirectExportEntryIr::export_name`, `ImportNameIr::Name(_)`, `ModuleNamespaceExportIr::export_name`, `resolve_export`'s parameter, `MODULE_DEFAULT_EXPORT_NAME` |
| **D3** | merged name — a name in the single merged Script top-level scope | none; forced by §1.2 | `module_binding_reference`'s result, `module_namespace_cell_name` &c., `ModuleNamespaceIr::cell`, both halves of the alias pairs |
| **D4** | unit-environment cell name — the `$m{unit}$`-prefixed spelling a future per-unit-environment backend would use | none; speculative | `graph.rs:631` only; stored in `ModuleNamespaceExportIr::cell`, read only by a test |

**D2 ⊇ the image of `[[ImportName]]`.** §16.2.1.4's own prose for
`[[ImportName]]` is "the name under which the desired binding is exported by
the module identified by `[[ModuleRequest]]`" — an `[[ExportName]]` of the
requested module. `ResolveExport(target, importName)` at `graph.rs:776` and
`graph.rs:588` passes it straight in as `exportName`. So `[[ImportName]]` is
**not** a fifth domain: it is a D2 value read from the other side. This is why
`graph.rs:783` filling `AmbiguousExport.export_name` from an `[[ImportName]]`
is correct, and why `ImportNameIr::Name` carries an `ExportName`.

**D3 has two disjoint generators, and this is the correction the brief needs.**

- **D3-of-local** — `merged(unit, local)` for a source binding:

  ```
  merged(unit, Source(s))          = s
  merged(unit, AnonymousDefault)   = "$d" ++ dec(unit) ++ "$"
  ```

  This is `module_binding_reference` (`record.rs:251-257`) exactly. It is
  total, and it applies **no** `$m{unit}$` prefix, because the merge shares
  cells by name on purpose (§1.2).

- **D3-minted** — `minted(unit, role)` for a compiler-owned per-unit cell:

  ```
  minted(unit, role) = "$m" ++ dec(unit) ++ "$" ++ suffix(role)
  ```

  `role` ranges over a *closed* set of compiler concepts, not over any source
  name: namespace object cell, deferred export-table cell, deferred evaluator
  function, module source object cell, `import.meta` object cell. Five roles;
  see §2.5 for why the sixth candidate is deleted.

The two generators have disjoint ranges by construction: `$` cannot begin a
source-spelled `BindingIdentifier` this compiler mints, `$d…` and `$m…` differ
in the second byte, and every `minted` name carries its unit id.

### 1.4 Invariants

**Local-name invariants.**

- **L1 (closed shape).** A `[[LocalName]]` is either source-spelled or the
  reserved `*default*` of §8.2.2. Exactly two cases, no third.
- **L2 (disjointness).** No source text produces `*default*`. §8.2.2 relies on
  this; the ParseModule path relies on it in the opposite direction at
  `record.rs:975-980`, where boa's spelling `default` for an anonymous default
  declaration is mapped to `*default*` and any other spelling is taken to be a
  real `BindingIdentifier` (the comment there says so).
- **L3 (spellability).** A source-spelled `[[LocalName]]` is an
  `IdentifierReference` the merged Script can write; `*default*` is not. Every
  emitter that writes a name into generated Script text must have discharged
  this, and today discharges it with a runtime predicate
  (`namespace::is_binding_identifier`, `namespace.rs:211`).
- **L4 (16.2 ExportedBindings ⊆ declared names).** For every
  LocalExportEntry, `[[LocalName]]` ∈ the names of the module environment.
  Checked at `early.rs:70-76` by `binding.name == entry.local_name` — a D1 = D1
  comparison. Writing `entry.export_name` there instead compiles today and
  produces a spurious `SyntaxError` for `export { x as y }`.

**Export-name invariants.**

- **E1 (open domain).** `[[ExportName]]` comes from `ModuleExportName`, which
  §16.2.3.1's grammar allows to be a `StringLiteral`
  (`export { a as "any \u{10000} text" }`). It is an arbitrary String. It is
  **not** validated, and this contract deliberately adds no validation — the
  repository already commits to surviving unpaired surrogates here
  (`namespace.rs:172-176`, "the only encoding that survives an unpaired
  surrogate"). The type earns its place by *separation*, not by validation;
  see §3, note (a).
- **E2 (`[[ImportName]]` is an `[[ExportName]]`).** §1.3. `ImportNameIr::Name`
  carries `ExportName`.
- **E3 (uniqueness).** §16.2.3.1: "It is a Syntax Error if the ExportedNames
  of ModuleItemList contains any duplicate entries." Enforced at
  `record.rs:323-341` / `early.rs:44-55` over D2 values only.
- **E4 (`default` is reserved, not minted).** `MODULE_DEFAULT_EXPORT_NAME`
  (`"default"`) is a D2 value, and it is *spellable from source* — `export
  { x as default }` produces it (§16.2.3.7). It is therefore **not** the
  analogue of `*default*`, and the two constants must not be given a common
  type. `export *` never re-exports it (§16.2.1.6.1 step 7 / `graph.rs:531`,
  `graph.rs:590-592`).

**Merged-name invariants.**

- **M1 (totality).** `merged(unit, ·)` is total on D1: every `[[LocalName]]`,
  including `*default*`, has a merged spelling. `module_binding_reference`
  already is total; the type must keep it so.
- **M2 (apply once).** The map is applied exactly once on any path from a
  `[[LocalName]]` to emitted Script text. Applying it twice yields
  `$d{u}$` for a name already `$d{u}$`, or (in D4) `$m{u}$$m{u}$x`; applying
  it zero times emits `*default*`, which does not lex.
- **M3 (injectivity, currently partial).** `merged` must be injective over the
  whole graph. It is not: two eager units declaring the same source name map
  to the same merged name. This is *detected* at `link.rs:495-503` and
  `namespace.rs:492-502` and reported as unsupported. **Ledger entry R2.**
- **M4 (minted names are identifiers).** Every `minted(unit, role)` must be a
  legal `IdentifierReference`, because every one of them is emitted into
  generated Script as a declaration or a read (`namespace.rs:260-263`,
  `namespace.rs:573-576`, `namespace.rs:612`, `namespace.rs:630-637`,
  `record.rs:399-402`). The two dead minters violate this: `$m{u}$import.meta`
  and `$m{u}$component.completion` both contain `.`.
- **M5 (roles are closed).** The set of compiler-owned per-unit cells is a
  closed set fixed by the linker's design, not an open string namespace.
- **M6 (D4 is not D3).** A D4 name must never reach an emitter. Today the only
  thing stopping it is the doc comment at `namespace.rs:104-108`.

**Byte-budget invariants** (implementation-forced, not spec).

- **B1 (`export default`).** `modules::source` rewrites the two keywords in
  place and must not change the unit's byte length
  (`source.rs:19-22`, `source.rs:401-433`). The replacement is
  `keyword ++ name ++ padding ++ "="` with `keyword ∈ {"let ", "var "}`, both
  4 bytes. The narrowest span the keywords can occupy is
  `"export" ++ " " ++ "default"` = **14** bytes. Hence
  `len(merged_anonymous_default(unit)) ≤ 14 − 4 − 1 = 9`.
  `"$d" ++ dec(u) ++ "$"` has length `3 + len(dec(u))`, so **B1 holds iff
  `len(dec(u)) ≤ 6`, i.e. `u ≤ 999_999`**.
- **B2 (`import.meta`).** `rewrite_import_meta` replaces the meta-property in
  place, same length (`record.rs:437-448`, check at `record.rs:483-489`). The
  narrowest span is `"import.meta"` = **11** bytes.
  `"$m" ++ dec(u) ++ "$" ++ "meta"` has length `7 + len(dec(u))`, so **B2
  holds iff `len(dec(u)) ≤ 4`, i.e. `u ≤ 9_999`**.

B2 is strictly tighter than B1. Both hold for every unit id iff the graph
caps unit ids at **9 999**. At exactly 9 999 the `import.meta` replacement is
11 bytes with **zero** padding — the assertion is tight, which is what makes
it worth writing.

---

## 2. The types

One new file, `crates/porffor-ir/src/binding_names.rs`, declared in
`crates/porffor-ir/src/lib.rs` next to `mod names;` (lib.rs:64) with
`mod binding_names;` and `pub use binding_names::*;` next to
`pub use names::*;` (lib.rs:109).

All five types derive `Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash` so
they can key the `BTreeMap`/`BTreeSet` collections at `link.rs:471`,
`link.rs:525`, `link.rs:626`, `namespace.rs:437`, `namespace.rs:453`,
`record.rs:1002` without those call sites changing shape. `Copy` where the
payload allows (`UnitCellRole` only).

### 2.1 `SourceName` — a `[[LocalName]]` that came from source text

```rust
/// A `BindingIdentifier` as written, resolved out of the interner.
///
/// The only constructor rejects the one name 8.2.2 reserves, so
/// `LocalName::Source` and `LocalName::AnonymousDefault` cannot alias.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceName(String);

impl SourceName {
    /// The only way to build one. `None` for `*default*` (invariant L2) and
    /// for the empty string.
    pub fn new(name: impl Into<String>) -> Option<Self>;
    pub fn as_str(&self) -> &str;
}
```

`new` is the validating constructor the standard asks for: it enforces L2 and
it is the single place the `*default*` spelling is compared against anything.
It returns `Option`, not a panic: the input comes from an interner and the
check is cheap, but ParseModule already discriminates the two cases (§5.4,
`record.rs:975-980`) and can pass the discrimination through rather than
re-deriving it, so `None` is reachable only from a caller that got it wrong.

### 2.2 `LocalName` — domain D1

```rust
/// `[[LocalName]]` (16.2.1.4, 16.2.1.5). Exactly two shapes — invariant L1.
///
/// This generalises `ImportNameIr` (`modules/record.rs:96`), which already
/// distinguishes `namespace-object` from a String by variant rather than by
/// a sentinel value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LocalName {
    /// A name the module's own text spells.
    Source(SourceName),
    /// `*default*` (8.2.2). No source text can produce it.
    AnonymousDefault,
}

impl LocalName {
    /// The spec spelling. `"*default*"` for `AnonymousDefault`.
    ///
    /// For diagnostics and for the 16.2 ExportedBindings check only. It is
    /// deliberately NOT what an emitter writes — see `merged_in`.
    pub fn spec_name(&self) -> &str;

    /// Domain D1 -> D3, applied exactly once. This is the whole of
    /// `module_binding_reference` (`modules/record.rs:251`), which is deleted.
    pub fn merged_in(&self, unit: ModuleUnitId) -> MergedName;
}
```

`merged_in` is a two-arm `match` with no catch-all: adding a third
`[[LocalName]]` shape (there will not be one, but a *refactor* can add a
variant) is `E0004` at the one place the mapping lives.

Deliberately absent, and the encoder must not add them: `Display`,
`AsRef<str>`, `Deref<Target = str>`, `From<LocalName> for String`, `FromStr`,
`Default`. A stringification must name `spec_name()` or `merged_in(..)` at
the call site — that is what stops `format!("{name}")` silently reintroducing
the `String` domain, and here it would silently reintroduce it *with the wrong
answer*, since `spec_name()` and `merged_in()` differ exactly on the case that
matters.

### 2.3 `ExportName` — domain D2

```rust
/// `[[ExportName]]` (16.2.1.5), and — read from the requested module's side —
/// `[[ImportName]]` (16.2.1.4). One domain, two viewpoints; see contract E2.
///
/// Not validated: `ModuleExportName` admits an arbitrary `StringLiteral`
/// (16.2.3.1), including one with unpaired surrogates, which this compiler
/// deliberately round-trips. The type carries *separation*, not validation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExportName(String);

impl ExportName {
    pub fn new(name: impl Into<String>) -> Self;
    pub fn as_str(&self) -> &str;
    /// `"default"` (16.2.3.7). Spellable from source — see contract E4.
    pub const DEFAULT: &'static str = "default";
    pub fn is_default(&self) -> bool;
}
```

Same absent impls, same reason.

### 2.4 `MergedName` — domain D3

```rust
/// A name in the single merged Script top-level scope.
///
/// Two disjoint generators and no other constructor: `LocalName::merged_in`
/// for a source binding, `MergedName::minted` for a compiler-owned cell.
/// There is no `From<String>`, so a bare `String` cannot become one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MergedName(String);

impl MergedName {
    /// `$m{unit}${suffix}` — the compiler-owned per-unit cells.
    pub fn minted(unit: ModuleUnitId, role: UnitCellRole) -> Self;
    /// `$d{unit}$` — the merged spelling of an anonymous `export default`.
    ///
    /// `pub(crate)`. Reached from `LocalName::merged_in`'s `AnonymousDefault`
    /// arm and from `link.rs`'s `DefaultExportRewrite::Bind`; those are the
    /// only two, and they agree by construction.
    pub(crate) fn anonymous_default(unit: ModuleUnitId) -> Self;
    pub fn as_str(&self) -> &str;
}
```

Note what is *not* offered: no `MergedName::prefixed(unit, &LocalName)`. There
is no function anywhere that takes a `[[LocalName]]` and returns a
`$m{unit}$`-prefixed name. That is the direct compile-time answer to the
brief's headline mistake class (§4, K1).

### 2.5 `UnitCellRole` — the closed role set (M5)

```rust
/// The compiler-owned per-unit cells. Closed by the linker's design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitCellRole {
    /// Identity-cached namespace exotic object (16.2.1.10).
    Namespace,
    /// `import defer` export table; `undefined` until the body has begun.
    DeferCells,
    /// `import defer` evaluator thunk.
    DeferEvaluate,
    /// `import source` module source object.
    ModuleSource,
    /// `import.meta` object (13.3.12, 16.2.1.9).
    ImportMeta,
}

impl UnitCellRole {
    pub const ALL: [UnitCellRole; 5];
    pub const fn suffix(self) -> &'static str;
}
```

| Variant | `suffix()` | Replaces |
|---|---|---|
| `Namespace` | `"namespace"` | `names::module_namespace_cell_name` |
| `DeferCells` | `"defer$cells"` | `names::module_defer_cells_cell_name` |
| `DeferEvaluate` | `"defer$evaluate"` | `names::module_defer_evaluate_function_name` |
| `ModuleSource` | `"source"` | `names::module_source_cell_name` |
| `ImportMeta` | `"meta"` | `record::import_meta_binding_name` |

**There is no sixth variant.** `module_import_meta_cell_name` is deleted (C3:
dead, and both budget-violating and identifier-violating).
`module_component_completion_cell_name` is deleted with its only consumer
(C2: `DynamicComponentIr::completion_cell`, written and never read; its
spelling `$m{u}$component.completion` violates M4). The design note at
`dynamic.rs:29-33` and `dynamic.rs:142-146` moves into that file's module doc
as prose, and must state that a future memoisation cell has to be added as a
`UnitCellRole` variant, which const assertion V3 will then check.
**[dry-run obligation D5]**

### 2.6 Constants and const assertions

```rust
/// Largest module unit id the source-text linker can name. Derived, not
/// chosen: see contract B1/B2.
pub const MAX_LINKABLE_MODULE_UNIT_ID: ModuleUnitId = 9_999;

pub(crate) const EXPORT_KEYWORD: &str = "export";
pub(crate) const DEFAULT_KEYWORD: &str = "default";
pub(crate) const IMPORT_META_TEXT: &str = "import.meta";
pub(crate) const DEFAULT_BINDING_LET: &str = "let ";
pub(crate) const DEFAULT_BINDING_VAR: &str = "var ";
```

`modules/source.rs` must use `EXPORT_KEYWORD` / `DEFAULT_KEYWORD` at its
`word_at` sites (`source.rs:275`, `source.rs:385-386`) and
`DEFAULT_BINDING_LET` / `DEFAULT_BINDING_VAR` at `source.rs:422`;
`modules/record.rs` must use `IMPORT_META_TEXT`'s length in the
`starts_with("import") && ends_with("meta")` guard at `record.rs:480`. Without
that the assertions below tie to nothing.

Private `const fn` helpers in `binding_names.rs`: `decimal_len(u32) -> usize`,
`str_len(&str) -> usize`, and `is_identifier_body_ascii(&str) -> bool`
(true iff every byte is `[A-Za-z0-9_$]`). All three are const-stable
(`str::as_bytes` + a `while` loop), matching the `str_eq` pattern already used
in `closed-name-domains.md` §2.1.

| # | `const _: () = assert!(…)` | Catches |
|---|---|---|
| **V1** | `DEFAULT_BINDING_LET.len() == DEFAULT_BINDING_VAR.len()` | the two rewrite heads drifting apart, which would make B1 depend on `hoisted` |
| **V2** | `DEFAULT_BINDING_LET.len() + (2 + decimal_len(MAX_LINKABLE_MODULE_UNIT_ID) + 1) + 1 <= EXPORT_KEYWORD.len() + 1 + DEFAULT_KEYWORD.len()` | **B1**: byte-length drift in `MergedName::anonymous_default` — the mistake class the brief named, now a build failure rather than a doc comment |
| **V3** | `∀ r ∈ UnitCellRole::ALL: is_identifier_body_ascii(r.suffix())` | **M4**: a minted cell name that is not an `IdentifierReference`. Would have rejected both `"import.meta"` and `"component.completion"` |
| **V4** | `∀ r ∈ UnitCellRole::ALL: 2 + decimal_len(MAX_LINKABLE_MODULE_UNIT_ID) + 1 + r.suffix().len() <= IMPORT_META_TEXT.len()` **for `r == ImportMeta` only**, written as an explicit single assertion naming `UnitCellRole::ImportMeta` | **B2**: the `import.meta` replacement outgrowing the span it replaces. Tight at the cap: 11 ≤ 11 |
| **V5** | `∀ i < j ∈ ALL: !str_eq(ALL[i].suffix(), ALL[j].suffix())` | two roles sharing a cell |
| **V6** | `∀ i: ALL[i] as u8 == i as u8` and `ALL.len() == 5` | `ALL` short or out of order |

V4 is deliberately *not* quantified over all roles: `DeferEvaluate`'s suffix is
14 bytes and would fail, correctly — only `ImportMeta` is written into a
fixed-width span. The assertion names the variant, so adding a second
span-constrained role means editing the assertion, which is the point.

---

## 3. Type mapping: invariant → construct

| Invariant | Rust construct | Why it holds |
|---|---|---|
| **L1** two shapes | `enum LocalName { Source, AnonymousDefault }` | a third shape is `E0004` at `merged_in` and at `early.rs`'s L4 check |
| **L2** disjointness | `SourceName::new` returning `None` for `*default*` — the only constructor | the sentinel comparison exists once, in a constructor, instead of at `record.rs:252` and `record.rs:975` |
| **L3** spellability | `LocalName::Source(_)` is spellable by construction; `AnonymousDefault` reaches an emitter only through `merged_in`, which returns `MergedName` | `namespace::is_binding_identifier` (`namespace.rs:211`) keeps only its D3-of-source role — ledger **R1** |
| **L4** ExportedBindings ⊆ declared | `early.rs:70-76` compares `ModuleEnvBindingIr::name: LocalName` with `LocalExportEntryIr::local_name: LocalName` | writing `entry.export_name` there is `E0308: expected LocalName, found ExportName` |
| **E1** open domain | `ExportName` newtype, no validation, documented | separation, not validation — note (a) |
| **E2** ImportName ≡ ExportName | `ImportNameIr::Name(ExportName)`; `resolve_export(&self, ModuleUnitId, &ExportName)` | passing a `LocalName` to `resolve_export` is `E0308` |
| **E3** uniqueness | `duplicate_export_names(&self) -> Vec<ExportName>` over D2 fields only | mixing a `local_name` into that iterator chain is `E0308` |
| **E4** `default` is not `*default*` | `ExportName::DEFAULT: &'static str` vs `LocalName::AnonymousDefault`, different types entirely | `LocalName::AnonymousDefault == ExportName::DEFAULT` does not compile |
| **M1** totality | `merged_in` returns `MergedName`, not `Option<MergedName>` | no call site can forget a failure case, because there is none |
| **M2** apply once | `MergedName` has no constructor taking a `MergedName`, and none taking a bare `String` | `merged_in(merged_in(..))` is `E0308`; so is `MergedName::minted(unit, role)` fed to anything expecting a `LocalName` |
| **M4** minted names are identifiers | const assertion **V3** over `UnitCellRole::ALL` | a `.` in a suffix is a build failure |
| **M5** roles closed | `enum UnitCellRole` + `ALL` + **V5/V6** | a new cell must be a variant; a `format!` cannot mint one |
| **M6** D4 ≠ D3 | §5.2: D4 is **deleted**, not typed | with `graph.rs:631` gone, no function produces a D4 name at all |
| **B1** | const assertion **V2** | |
| **B2** | const assertion **V4** | |

Note (a) — **why `ExportName` earns its place without a validating
constructor.** AGENTS.md's test is "a plausible mistake becomes a compile
error", not "the constructor validates". The mistake is the swap, and the
swap becomes `E0308`. The brief's phrasing ("newtypes whose only constructor
validates") describes the usual case; `ExportName`'s domain is genuinely open
(E1), so validating would be *false*, and the type still passes the test.
`SourceName` does validate, and `LocalName`'s two-variant shape is what makes
that validation meaningful.

### 3.1 Runtime-checked ledger

These are the only places a test remains load-bearing. Each entry states why a
type cannot carry the invariant. The encoder must not "fix" these by inventing
a type; the dry-runner must confirm each reason still holds.

| # | Invariant | Where | Why no type carries it | What must check it |
|---|---|---|---|---|
| **R1** | **L3**, "this merged name is an `IdentifierReference`". | `namespace::is_binding_identifier` (`namespace.rs:211`), called at `namespace.rs:244-246`, `namespace.rs:472`, `namespace.rs:686`. | After the retype the remaining inputs are `MergedName::of_local` results, i.e. arbitrary source-spelled identifiers *as the source wrote them*. boa has already accepted them as `BindingIdentifier`s, so the predicate is nearly always true — but "nearly" is not "always": a `\u`-escaped identifier and an astral-plane identifier both parse and neither is ASCII-spellable by the current emitter. Encoding that in a type would mean a `SpellableName` newtype whose constructor runs a full `IdentifierPart` scan, duplicating boa. The honest operation is the predicate. **What the retype does buy:** the `AnonymousDefault` case can no longer reach it, because `merged_in` has already replaced it — so the predicate's `*default*` branch (its documented purpose, `namespace.rs:206-210`) becomes dead and the encoder must delete that sentence from its doc comment. | `cargo test -p porffor-ir` (rung 1); the existing `namespace.rs` unsupported-diagnostic tests |
| **R2** | **M3**, injectivity of `merged` over the graph. | `link.rs:471-504` (`report_cross_unit_binding_collisions`), `link.rs:625-644` (`merged_lexical_names`), `namespace.rs:437-449`. | Injectivity is a property of a *set* of units, not of one name. No per-value type can carry it; a typestate that refused to emit until the whole graph was checked is possible in principle but would have to thread the whole `ModuleGraphIr` through every `MergedName` construction, which is not worth it while the resolution is "report as unsupported" rather than "rename". **This is the invariant that becomes a real typestate obligation when the renaming pass lands**, and the contract records that now so the encoder does not build the wrong thing first. | the existing collision tests in `link.rs`'s `#[cfg(test)]` block; test262 `language/module-code/instn-*` |
| **R3** | **B1/B2** at run time for a unit id above the cap. | `graph.rs:669`, `let id = ModuleUnitId::try_from(graph.units.len()).unwrap_or(ModuleUnitId::MAX);` | `ModuleUnitId` stays `pub type ModuleUnitId = u32` (§6). Newtyping it with a bounded constructor would ripple into `ir.rs:1345,1349,1353,1846` and `lib.rs:81`, which are not owned by this area. So the cap is enforced at the one place ids are minted, not in the type. **The current line is worse than unchecked**: it *saturates* to `u32::MAX`, whose decimal length is 10, which violates B1 and B2 silently and then fails downstream with a confusing `StripError`. The encoder replaces the `unwrap_or` with a `ModuleLinkErrorIr`-shaped rejection at `> MAX_LINKABLE_MODULE_UNIT_ID`. | one new unit test in `graph.rs` asserting the rejection; V2/V4 carry the *format* half of B1/B2 at compile time |
| **R4** | boa's `private_name()` accessor means `[[LocalName]]` in `ExportDeclaration::List` and `[[ImportName]]` in `ReExportKind::Named`. | `record.rs:838` vs `record.rs:862`. | The accessor is boa's, on a foreign type; its return is a `Sym`. This contract cannot change boa's naming. **What the retype buys:** the two sites now convert through `SourceName::new(..)` and `ExportName::new(..)` respectively, so the *conversion* names the domain even though the accessor does not, and swapping the two conversions is `E0308` one line later at the struct literal. | trace T3 (§7) |
| **R5** | **L2's second half**: `SourceName::new` rejecting the *empty* spelling. | `SourceName::new` (`binding_names.rs`). | **Encoder deviation from §2.1, recorded rather than hidden.** §2.1 asked `new` to return `None` for `*default*` *and* for the empty string. Two rejection reasons make `None` ambiguous at a call site, and — decisively — they make the classifier total only by accident: every product conversion site is a BoundNames position, so the classification must be *total*, and `LocalName::from_bound_name` can only be total if `None` means exactly one thing. `new` therefore rejects `*default*` and nothing else, and `from_bound_name` reads that single `None` as "this is the anonymous default". The empty spelling is not a domain question — it is a spellability question, and `is_binding_identifier` already rejects it (its first act is `chars.next()` returning `None`). No product path can produce it: boa's interner never resolves a `BindingIdentifier` to `""`. | `is_binding_identifier`'s empty-name arm; `binding_names.rs`'s `source_name_rejects_only_the_reserved_spelling` |
| **R6** | **The 24 minting call sites and every field retype are load-bearing for byte-identical output**, and this lane ran no compiler. | whole area. | Not an invariant a type can carry: it is the claim that a mechanical retype changed no emitted byte. Every rewrite in §5.3 is spelling-preserving by construction (`MergedName::minted(u, r)` is `format!("$m{u}${suffix}")`, the same literal the deleted function built), and `merged_in` is `module_binding_reference` verbatim — but "by construction" is an argument, not a measurement. | `cargo check -p porffor-ir`, `cargo test -p porffor-ir --lib`, and **rung G** (`diff -r target/golden/before target/golden/after` empty). §8. |

---

## 4. Mistake-class table

| # | Mistake | Today | After |
|---|---|---|---|
| **K1** | Pass a source name where a merged name belongs; apply the prefix zero times or twice. *This has shipped here* (C6). | `String` everywhere; `format!("{}{name}", module_storage_prefix(u))` at `graph.rs:631` and `module_binding_reference` at `record.rs:251` are both `&str -> String` and are indistinguishable at a call site. | `E0308: expected MergedName, found LocalName` (or `&str`). `MergedName` has no `From<String>` and no `prefixed(unit, &LocalName)` constructor, so the double-prefix expression cannot be written. `module_storage_prefix` ceases to exist as a public function; its literal moves inside `MergedName::minted`. |
| **K2** | Swap `[[LocalName]]` and `[[ExportName]]` in a `LocalExportEntryIr`. | Two adjacent `String` fields (`record.rs:120-125`); `push_local_export` (`record.rs:962-967`) writes `local_name: name.to_string(), export_name: name.to_string()` — the coincident case that hides the swap. | `E0308: expected LocalName, found ExportName` at the struct literal. In `push_local_export` the one `&str` must be converted **twice**, by `SourceName::new(name).map(LocalName::Source)` and by `ExportName::new(name)`; one shared conversion no longer type-checks. |
| **K3** | Look a name up in the wrong domain. | `resolve_export(module, &str)`; `graph.rs:814` passes `entry.export_name`, `graph.rs:776` passes an `[[ImportName]]`, `early.rs:73` compares `binding.name == entry.local_name`. All four are `String`, so any pair is interchangeable. | `resolve_export(&self, ModuleUnitId, &ExportName)`. Passing a `LocalName` is `E0308`. `early.rs:73` is `LocalName == LocalName`; substituting `entry.export_name` is `E0308`. |
| **K4** | Mint a per-unit cell without the prefix, or with it already applied, or with a name that is not an identifier. | Six functions returning bare `String` from a bare `u32`, with no relation between them; `$m{u}$import.meta` and `$m{u}$component.completion` both exist and both contain `.`. | One constructor `MergedName::minted(unit, UnitCellRole)`. The prefix appears in exactly one `format!` in the crate. A new cell is a new enum variant; const assertion **V3** rejects a non-identifier suffix at build time, **V5** rejects a duplicate. |
| **K5** | Byte-length drift in the anonymous-default name. | A doc comment on a `String`-returning function (`names.rs:47-49`). | Const assertion **V2** ties `MergedName::anonymous_default`'s format to `EXPORT_KEYWORD`/`DEFAULT_KEYWORD`/`DEFAULT_BINDING_LET`, the same constants `modules/source.rs` matches on. Widening `$d` to `$default` fails to build. |
| **K6** | Byte-length drift in the `import.meta` cell name. *Unrecorded in the brief.* | Nothing. `import_meta_binding_name` (`record.rs:386`) and the 11-byte budget at `record.rs:483-489` are 100 lines apart with no tie. | Const assertion **V4**, naming `UnitCellRole::ImportMeta` explicitly. |
| **K7** | Emit a D4 (unit-environment) cell name into generated Script text. | Prevented only by the doc comment at `namespace.rs:104-108`. | The D4 producer is deleted (§5.2); no function returns such a name. |
| **K8** | Module-qualify a builtin `FunctionId`. | Guarded by `is_user_function_id`'s `starts_with` test (`names.rs:113`), whose doc calls itself "the single authority". | **Not applicable after §5.2.** `module_function_id`, `module_function_id_prefix` and `is_user_function_id` are deleted: they have zero product call sites (C2), and the design that needed them was replaced — `link.rs:12-17` states that merging on source text makes every span-derived `FunctionId` unique by construction, so no module qualification is performed anywhere. Nothing can module-qualify a builtin id because nothing module-qualifies any id. The brief's `UserFunctionId` newtype is therefore **not built**: guarding a function with no callers is decoration by AGENTS.md's own test. **[dry-run obligation D4]** |
| **K9** | Add a per-unit cell and forget one of the places it must be declared. | Six unrelated functions. | Partially addressed: `UnitCellRole::ALL` gives one enumeration point. It does **not** force the declaration to be emitted — that is prelude-assembly ordering, a different area. Stated so the encoder does not overclaim. |

---

## 5. Retrofit map

### 5.1 Order of operations

Strictly this order. Steps 1, 2 and 6 leave the tree compiling; steps 3–5 do
not, and must land as one edit.

1. **Add `crates/porffor-ir/src/binding_names.rs`** with all five types, the
   constants and V1–V6. Add `mod binding_names;` / `pub use binding_names::*;`
   to `lib.rs`. Nothing consumes it yet.
   **`cargo check -p porffor-ir` here validates V1–V6 before any call site
   depends on the types.** This is the cheapest possible confirmation that B1
   and B2 as derived in §1.4 are arithmetically right.
2. **Delete the five dead functions** (§5.2). Independent of everything else;
   `cargo check -p porffor-ir` must stay green, which is itself the proof that
   they were dead.
3. **Retype the record fields** (§5.4) and fix `modules/record.rs`,
   `modules/early.rs` in the same edit.
4. **Retype the graph surface** — `resolve_export`, `exported_names`,
   `push_unique_name`, `ModuleBindingNameIr::Name`, `ModuleLinkErrorIr`'s two
   name fields — and fix `modules/graph.rs`.
5. **Retype the emitters** — `modules/namespace.rs`, `modules/link.rs`,
   `modules/dynamic.rs`, `modules/source.rs`.
6. **Fix the `#[cfg(test)]` blocks** (§5.6) and the `lib.rs` re-export list.

### 5.2 Deletions

| Symbol | File:line | Product call sites | Reason |
|---|---|---|---|
| `module_function_id` | `names.rs:117-125` | **0** | C2. No `FunctionId` is module-qualified anywhere; `link.rs:12-17` explains why none needs to be. |
| `module_function_id_prefix` | `names.rs:55-62` | **0** | reached only from the above |
| `is_user_function_id` | `names.rs:108-115` | **0** | reached only from the above. Its "single authority" doc comment is the comment-doing-a-type's-job the area was chartered to fix; the fix is that there is no longer a job. |
| `module_import_meta_cell_name` | `names.rs:93-97` | **0** | C3. Dead, 15 bytes into an 11-byte span, and not an identifier. Superseded by `import_meta_binding_name`, which becomes `UnitCellRole::ImportMeta`. |
| `module_component_completion_cell_name` | `names.rs:99-106` | 1, feeding a field with 0 readers | C2/M4. Deleted together with `DynamicComponentIr::completion_cell` (`dynamic.rs:206`) and its construction (`dynamic.rs:244`). |
| `ModuleGraphIr::cell_name` | `graph.rs:619-641` | 1 (`namespace.rs:738`), feeding a field whose only reader is a test | C4/M6. The D4 producer. |
| `ModuleNamespaceExportIr::cell` | `namespace.rs:100-109` | written at `namespace.rs:739`, read only at `namespace.rs:899` (test) | C4 |
| `ModuleNamespaceIr::cell_for` | `namespace.rs:158-168` | **0** product | C4 |
| `module_binding_reference` | `record.rs:246-257` | 3 | replaced verbatim by `LocalName::merged_in`; not a semantic deletion |
| `import_meta_binding_name` | `record.rs:381-388` | 2 | replaced by `MergedName::minted(unit, UnitCellRole::ImportMeta)`; same bytes |
| `module_storage_prefix`, `module_default_binding_name`, `module_namespace_cell_name`, `module_defer_cells_cell_name`, `module_defer_evaluate_function_name`, `module_source_cell_name` | `names.rs:28-91` | 19 total | the format strings move into `MergedName::minted` / `MergedName::anonymous_default`; the `pub fn`s are removed from `names.rs` |

AGENTS.md, "If something is unreachable from the product path, that should
fail to build, not merely fail to run. Code with no call site has been written
here more than once; it compiled, formatted cleanly and produced no dead-code
warning because it was `pub`." Five such functions are in this one 90-line
region. **[dry-run obligation D4]**

Design notes that must survive deletion, moved to prose:

- `dynamic.rs`'s module doc gains the `completion_cell` rationale
  (`dynamic.rs:29-33`, `dynamic.rs:142-146`), rewritten to say what a future
  memoisation cell would be — a `UnitCellRole` variant with an
  identifier-legal suffix.
- `namespace.rs`'s module doc gains the D4 note (`namespace.rs:104-108`),
  rewritten to say that a per-unit-environment backend would reintroduce
  `$m{unit}$` + `[[LocalName]]` as a distinct type, and that it must not be
  spelled `MergedName`.

### 5.3 Every product call site of a minting function, measured

Produced by splitting each file at its `#[cfg(test)]` line — `link.rs:686`,
`graph.rs:1147`, `namespace.rs:828`, `record.rs:1303`, `dynamic.rs:1301`,
`source.rs:711`, `early.rs:186` — and excluding `use` lines and doc comments.
19 `names.rs` minters + 5 `modules/`-local minters = **24**.

| File:line | Today | Becomes |
|---|---|---|
| `graph.rs:628` | `module_namespace_cell_name(*module)` | **deleted with `cell_name`** |
| `graph.rs:629` | `module_source_cell_name(*module)` | **deleted with `cell_name`** |
| `graph.rs:631` | `format!("{}{name}", module_storage_prefix(*module))` | **deleted with `cell_name`** — the sole D4 producer |
| `namespace.rs:233` | `module_namespace_cell_name(*module)` | `MergedName::minted(*module, UnitCellRole::Namespace)` |
| `namespace.rs:237` | `module_source_cell_name(*module)` | `MergedName::minted(*module, UnitCellRole::ModuleSource)` |
| `namespace.rs:244` | `module_binding_reference(*module, name)` | `name.merged_in(*module)` where `name: &LocalName` |
| `namespace.rs:288` | `module_defer_evaluate_function_name(namespace.module)` | `MergedName::minted(namespace.module, UnitCellRole::DeferEvaluate)` |
| `namespace.rs:504` | `module_namespace_cell_name(*module)` | `…UnitCellRole::Namespace` |
| `namespace.rs:558` | `module_defer_cells_cell_name(module)` | `…UnitCellRole::DeferCells` |
| `namespace.rs:559` | `module_defer_evaluate_function_name(module)` | `…UnitCellRole::DeferEvaluate` |
| `namespace.rs:612` | `format!("let {};\n", module_defer_cells_cell_name(module))` | `…UnitCellRole::DeferCells` + `.as_str()` |
| `namespace.rs:630` | `module_source_cell_name(module)` | `…UnitCellRole::ModuleSource` |
| `namespace.rs:703` | `module_source_cell_name(*module)` | `…UnitCellRole::ModuleSource` |
| `namespace.rs:721` | `module_namespace_cell_name(module)` | `…UnitCellRole::Namespace` |
| `dynamic.rs:244` | `module_component_completion_cell_name(module)` | **deleted with the field** |
| `dynamic.rs:435` | `module_namespace_cell_name(component.module)` | `…UnitCellRole::Namespace` |
| `dynamic.rs:437` | `module_source_cell_name(component.module)` | `…UnitCellRole::ModuleSource` |
| `record.rs:253` | `module_default_binding_name(unit)` | body of `LocalName::merged_in`'s `AnonymousDefault` arm → `MergedName::anonymous_default(unit)` |
| `record.rs:387` | `format!("{}meta", module_storage_prefix(unit))` | `MergedName::minted(unit, UnitCellRole::ImportMeta)` |
| `record.rs:398` | `import_meta_binding_name(unit)` | same |
| `record.rs:460` | `import_meta_binding_name(record.id)` | same |
| `link.rs:244` | `module_default_binding_name(unit_id)` | `MergedName::anonymous_default(unit_id)` |
| `link.rs:495` | `module_binding_reference(*unit_id, &binding.name)` | `binding.name.merged_in(*unit_id)` |
| `link.rs:635` | `module_binding_reference(unit.record.id, &binding.name)` | `binding.name.merged_in(unit.record.id)` |

### 5.4 Field and signature retypes

**`modules/record.rs`**

| Item | Today | Becomes |
|---|---|---|
| `ImportNameIr::Name` (`:102`) | `Name(String)` | `Name(ExportName)` — E2 |
| `ImportEntryIr::local_name` (`:113`) | `String` | `LocalName` |
| `LocalExportEntryIr::local_name` (`:122`) | `String` | `LocalName` |
| `LocalExportEntryIr::export_name` (`:124`) | `String` | `ExportName` |
| `IndirectExportEntryIr::export_name` (`:135`) | `String` | `ExportName` |
| `ModuleEnvBindingIr::name` (`:166`) | `String` | `LocalName` |
| `ModuleEnvBindingIr::indirect` (`:177`) | `Option<(ModuleRequestIr, ImportNameIr)>` | unchanged (`ImportNameIr` now carries `ExportName`) |
| `ImportMetaBindingIr::name` (`:372`) | `String` | `MergedName` |
| `own_exported_names` (`:272`) | `-> Vec<String>` | `-> Vec<ExportName>` |
| `duplicate_export_names` (`:323`) | `-> Vec<String>` | `-> Vec<ExportName>` |
| `push_unique_name` (`:551`) | `(&mut Vec<String>, &str)` | `(&mut Vec<ExportName>, &ExportName)` — all 6 call sites (`record.rs:275,278,337`, `graph.rs:519,522,533`) are D2 |
| `module_environment` (`:993`) `default_local` | `Option<&str>` | `Option<&LocalName>` |
| `push_declaration_bindings` / `push_var_bindings` / `push_default_binding` `declared: &mut BTreeSet<String>` | | `BTreeSet<LocalName>` |

ParseModule conversion sites — the ones where the domain is decided:

| Line | Accessor | Domain | Conversion |
|---|---|---|---|
| `760` | `default.sym()` | D1 | `LocalName::Source(SourceName::new(..)?)` |
| `759` | literal `MODULE_DEFAULT_EXPORT_NAME` | D2 | `ExportName::new(ExportName::DEFAULT)` |
| `770` | `binding.sym()` | D1 | `LocalName::Source(..)` |
| `778-781` | `name.export_name()` / `name.binding().sym()` | D2 / D1 | the `import { a as b }` pair |
| `825` | `*name` (`export * as ns from`) | D2 | `ExportName::new(..)` |
| `837-840` | `entry.private_name()` / `entry.alias()` | **D2 / D2** | `ReExportKind::Named`: `private_name()` is the `[[ImportName]]` — R4 |
| `862-863` | `entry.private_name()` / `entry.alias()` | **D1 / D2** | `ExportDeclaration::List`: `private_name()` is the `[[LocalName]]` — R4. The two rows above and below are the whole reason R4 exists. |
| `883-884` | | | `LocalExportEntryIr { local_name: local, export_name }` — the swap site |
| `936`, `940` | `bound_names(..)` | D1, then D2 | via `push_local_export` — K2 |
| `975-980` | boa's `default` sentinel | D1 | `if declared_name == ExportName::DEFAULT { LocalName::AnonymousDefault } else { LocalName::Source(..) }` — the one place L2 is decided, and the only remaining caller of `SourceName::new` that can plausibly see `None` |
| `986-988` | | D1 / D2 | `LocalName::AnonymousDefault`, `ExportName::new(ExportName::DEFAULT)` |

`MODULE_ANONYMOUS_DEFAULT_LOCAL_NAME` (`names.rs:26`) is kept as the spelling
returned by `LocalName::spec_name()` and used by `SourceName::new`'s rejection
test, and by nothing else. `MODULE_DEFAULT_EXPORT_NAME` (`names.rs:20`) is
kept; `ExportName::DEFAULT` is defined *from* it, so there is one literal.

**`modules/graph.rs`**

| Item | Today | Becomes |
|---|---|---|
| `ModuleBindingNameIr::Name` (`:73`) | `Name(String)` | `Name(LocalName)` — §16.2.1.6.2's `[[BindingName]]` is a `[[LocalName]]` of the *resolving* module |
| `ModuleLinkErrorIr::MissingExport::import_name` (`:117`) | `String` | `ExportName` — E2 |
| `ModuleLinkErrorIr::AmbiguousExport::export_name` (`:123`) | `String` | `ExportName` |
| `ModuleLinkErrorIr::DuplicateExport::export_name` (`:130`) | `String` | `ExportName` |
| `exported_names` (`:498`) | `-> Vec<String>` | `-> Vec<ExportName>` |
| `collect_exported_names` (`:505`) `names` | `&mut Vec<String>` | `&mut Vec<ExportName>` |
| `resolve_export` (`:540`) | `(ModuleUnitId, &str)` | `(ModuleUnitId, &ExportName)` |
| `resolve_export_inner` (`:545`) `resolve_set` | `&mut Vec<(ModuleUnitId, String)>` | `&mut Vec<(ModuleUnitId, ExportName)>` |
| `import_name_text` (`:1016`) | `-> String` | `-> ExportName`; the `Namespace` arm keeps `"*"`, which is a diagnostic spelling and is documented as such |
| `cell_name` (`:625`) | | **deleted** (§5.2) |
| `build_graph` unit mint (`:669`) | `unwrap_or(ModuleUnitId::MAX)` | reject above `MAX_LINKABLE_MODULE_UNIT_ID` — ledger **R3** |

**`modules/namespace.rs`**

| Item | Today | Becomes |
|---|---|---|
| `ModuleNamespaceExportIr::export_name` (`:97`) | `String` | `ExportName` |
| `ModuleNamespaceExportIr::cell` (`:109`) | `String` | **deleted** |
| `ModuleNamespaceIr::cell` (`:126`) | `String` | `MergedName` |
| `own_property_keys` (`:151`) | `-> Vec<&str>` | `-> Vec<&ExportName>` |
| `cell_for` (`:163`) | | **deleted** |
| `utf16_sort_key` (`:174`) | `(&str)` | `(&ExportName)` |
| `namespace_target_reference` (`:227`) | `-> Option<String>` | `-> Option<MergedName>` |
| `is_binding_identifier` (`:211`) | `(&str) -> bool` | `(&MergedName) -> bool`; its `*default*` sentence is deleted — ledger **R1** |
| `ensure_namespace` (`:720`) | `-> String` | `-> MergedName` |
| `deferred_cells_declaration` (`:617`) | `-> String` | unchanged (`String` of generated *source text*, not a name) |
| alias vectors (`:452`, `:667`) | `Vec<(String, String)>` | `Vec<(MergedName, MergedName)>` |
| `declared` / `owners` maps (`:437`, `:453`) | `BTreeMap<&str, &str>` | `BTreeMap<MergedName, &str>` (the key was `binding.name`, a D1 value, compared against `entry.local_name`, also D1 — but it is compared against *alias* names, which are D3, so it must be a D3 map. **This is a live domain confusion the retype exposes**: `namespace.rs:447` inserts `binding.name` (D1) and `namespace.rs:496` looks up `local` (D1, from an import entry) — both D1, so the current code is *self-consistent*, but `namespace.rs:504` then emits the alias as a merged declaration. Under the types the map becomes D3 on both sides via `merged_in`. **[dry-run obligation D6]**) |

**`modules/link.rs`**

| Item | Today | Becomes |
|---|---|---|
| `collect_binding_aliases` (`:519`) | `-> Vec<(String, String)>` | `-> Vec<(MergedName, MergedName)>` |
| `merged_lexical_names` (`:625`) | `-> BTreeMap<String, String>` | `-> BTreeMap<MergedName, String>` |
| `owners` in `report_cross_unit_binding_collisions` (`:471`) | `BTreeMap<String, &str>` | `BTreeMap<MergedName, &str>` |
| `binding_alias_prelude` | `&[(String, String)]` | `&[(MergedName, MergedName)]` |
| `link.rs:566` `reference == entry.local_name` | `String == String` | `reference == entry.local_name.merged_in(unit.record.id)` — **this comparison is currently cross-domain** (a D3 `reference` against a D1 `local_name`) and happens to be right only because `merged_in` is the identity on source names. Under the types it must be written out, which makes the reasoning visible. **[dry-run obligation D7]** |
| `link.rs:570-573`, `:582`, `:588`, `:594` | `entry.local_name.as_str()` compared against `OBJECT_NAME` &c., used in messages | `.spec_name()` for messages; the `OBJECT_NAME`/`SYMBOL_NAME`/`GLOBAL_THIS_NAME` comparison is against merged names, so `merged_in(..).as_str()` |

**`modules/source.rs`**

`DefaultExportRewrite::Bind::name` (`:56`) becomes `&'a MergedName`; the
`head` computation at `source.rs:423` uses `name.as_str().len()`. The three
keyword literals become the `binding_names.rs` constants (§2.6) so V2 ties to
them.

**`modules/early.rs`**

`early.rs:73` `binding.name == entry.local_name` is `LocalName == LocalName`,
unchanged in shape. `early.rs:83` interpolates `entry.local_name` into a
message → `.spec_name()`.

**`modules/dynamic.rs`**

`DynamicComponentIr::completion_cell` deleted. `component_resolution_cell`
(`:433`) returns `MergedName`.

### 5.5 `crates/porffor-ir/src/lib.rs`

The `pub use modules::{…}` block at lines 76–83 keeps every name it lists
except none — no exported type is removed, only fields and one method. Add
`mod binding_names;` after `mod analysis;` (alphabetical, line 57) and
`pub use binding_names::*;` next to `pub use names::*;` (line 109).

`names.rs` loses the eleven module functions and keeps
`MODULE_DEFAULT_EXPORT_NAME` and `MODULE_ANONYMOUS_DEFAULT_LOCAL_NAME`. The
`pub(crate) use names::{…}` list at lines 110–112 is unaffected.

### 5.6 Tests in `#[cfg(test)]` blocks that must change

These are mechanical but must not be skipped; three of them assert the deleted
D4 mapping and would otherwise be deleted silently.

| File:line | Assertion | Action |
|---|---|---|
| `graph.rs:1559-1562` | `graph.cell_name(&resolved) == Some(format!("{}x", module_storage_prefix(c)))` | **Delete.** It is the only assertion that the D4 mapping exists, and D4 is deleted. Replace with `namespace_target_reference(&resolved) == Some(MergedName::…)` — the D3 value, which is `"x"`. |
| `graph.rs:1297-1300` | `graph.cell_name(..) == Some(module_source_cell_name(target))` | rewrite through `namespace_target_reference`; the value is unchanged |
| `graph.rs:1679-1682` | `graph.cell_name(&binding) == Some(module_namespace_cell_name(a))` | rewrite through `namespace_target_reference`; value unchanged |
| `namespace.rs:896-902` | `namespace.cell_for("value") == Some("$m0$value")` | **Delete** with `cell_for` |
| `namespace.rs:1014-1021` | asserts the generated source does **not** contain `$m0$value` | **Keep** — it becomes a stronger statement once nothing can produce that name, and it is the regression guard for K7 |
| `namespace.rs:910-916`, `:985-990`, `:1105-1112`, `:1150-1155`, `:1298-1310`, `:1340-1345`, `:1380-1386`, `:1420-1427` | construct or compare minted names | `MergedName::minted(..)` |
| `namespace.rs:1120-1127`, `dynamic.rs:1720-1727`, `link.rs:970-980`, `:1010-1025` | `module_default_binding_name(n)` | `MergedName::anonymous_default(n)` |
| `dynamic.rs:1360-1370`, `:1620-1640`, `:1740-1750`, `:1765-1775`, `link.rs:1100-1110`, `:1130-1140`, `:1180-1190`, `:1235-1245`, `graph.rs:1290-1300` | minted-name comparisons | `MergedName::minted(..)` |
| `record.rs:1320-1360` | the `import` / `local_export` / `indirect_export` test constructors | take `&str`, convert inside; **`local_export(local_name, export_name)` must convert its two arguments through the two different constructors** — this is the adversarial trace T6's compile-time witness |
| `record.rs:1800-1860` | `import_meta_binding_name` | `MergedName::minted(.., ImportMeta)` |
| `early.rs:220-320` | constructs `LocalExportEntryIr` / `IndirectExportEntryIr` / `ModuleEnvBindingIr` | convert per field |
| `graph.rs:1600-1610`, `:1636-1656` | `resolve_export(a, "x")`, `exported_names(a) == vec!["z"]` | `&ExportName::new("x")`, `vec![ExportName::new("z")]` |
| `graph.rs:1290-1300`, `link.rs` link-error assertions | `import_name: "nope".to_string()` | `ExportName::new("nope")` |

---

## 6. What stays untouched, and why

- **`pub type FunctionId = String` (`ir.rs:17`).** Newtyping it reaches
  `crates/porffor-aot-wasm`, outside this campaign's crate scope. Nothing in
  this contract touches it: `UserFunctionId` is not built (K8).
- **`pub type ModuleUnitId = u32` (`record.rs:25`).** It appears in `ir.rs` at
  lines 1345, 1349, 1353, 1846 and in the `lib.rs` re-export at line 81.
  Newtyping it would be the natural home for the B1/B2 cap; it is deferred, and
  the cap lives at the single mint site instead — ledger **R3**.
- **`$tdz.` (`names.rs:17`), `tdz_binding_storage_name` /
  `for_of_loop_binding_storage_name` (`lowering_helpers.rs:1541,1545`),
  `ForInOfEnvironmentIr::tdz_binding_names` (`ir.rs:1750`).** They live in
  `lowering.rs` / `lowering_helpers.rs` / `ir.rs`. This area's value is that it
  never opens those files.
- **The general `source_name` / `storage_name` retype.** Same contract applied
  to the non-module half; a later lane, once `lowering.rs` is free.
- **`ModuleRequestIr`, `ImportAttributeIr`, `ImportPhaseIr`,
  `StarExportEntryIr`, `ModuleBindingKindIr`, `DefaultExportFormIr`,
  `ModuleEvaluationModeIr`.** Specifier and phase are a different domain
  (a *module* name, not a *binding* name) and out of scope. `StarExportEntryIr`
  holds no binding name at all — which is itself evidence for the domain split
  and is why it is one of the corpus traces.
- **`crates/porffor-aot-wasm`.** Zero consumers: `ImportEntryIr`,
  `LocalExportEntryIr`, `IndirectExportEntryIr`, `ImportNameIr`,
  `StarExportEntryIr`, `ModuleEnvBindingIr`, `ModuleNamespaceExportIr` and all
  eleven minting functions appear only in `crates/porffor-ir`. Verified by
  `grep -rn … crates/ --include=*.rs`. The batch-2 files
  (`intl_datetimeformat.rs`, `temporal*.rs`, `emitted_function.rs`,
  `runtime_helpers.rs`) are not reached.

---

## 7. Dry-run corpus: what each trace must establish

Each trace is a symbolic execution on paper of the spec steps against the
post-retrofit code. Any trace that cannot be completed is a defect in this
contract, not in the encoder's work.

| # | Case | Must establish |
|---|---|---|
| **T1** | `instn-named-bndng-dflt-fun-anon.js` — anonymous `export default function () {}` | ParseModule reaches `record.rs:975-980` with boa's `default`, produces `LocalName::AnonymousDefault` (not `SourceName`), `ExportName::DEFAULT`; `default_export_form()` returns `Anonymous { hoisted: true }`; `link.rs:244` mints `MergedName::anonymous_default(u)`; `source.rs:406-433` binds `var $d0$ = …` in 14 bytes with 5 spaces of padding. **`SourceName::new("*default*")` is never called on this path.** |
| **T2** | `instn-named-bndng-dflt-fun-named.js` — `export default function f() {}` | `[[LocalName]]` is `LocalName::Source("f")` while `[[ExportName]]` is `ExportName::DEFAULT`; `default_export_form()` is `Named`; `DefaultExportRewrite::DeleteKeywords`, so `MergedName::anonymous_default` is **not** called. Together with T1 this separates D1 from D2 on one production. |
| **T3** | `instn-named-bndng-dflt-named.js` — `export { x as default }` | `record.rs:862-863`: `private_name()` → `SourceName("x")` → D1; `alias()` → `ExportName("default")` → D2. Neither is `*default*` (E4). The `LocalExportEntryIr` literal at `:883` receives two different types. **Swap-detection case.** |
| **T4** | `eval-export-dflt-cls-anon.js` — anonymous default class | Same `*default*` path as T1 through `DefaultExportFormIr::Anonymous { hoisted: false }` → `let` rather than `var`; V1 guarantees the byte budget is the same either way. |
| **T5** | `instn-local-bndng-export-let.js` — `export let x` | `push_local_export` (`record.rs:962-967`): one `&str` converted **twice**, once to `LocalName::Source` and once to `ExportName`. Confirm the two conversions are visible in the source and that a single shared conversion does not type-check. The coincident case. |
| **T6** | `instn-named-bndng-let.js` — `import { x } from './m.js'` | `ImportEntryIr { import_name: ImportNameIr::Name(ExportName("x")), local_name: LocalName::Source("x") }`; `graph.rs:776` passes the `ExportName` to `resolve_export`. The precedent `ImportNameIr` generalises. |
| **T7** | `instn-iee-bndng-let.js` — `export { x } from './m.js'` | `IndirectExportEntryIr` has `import_name: ExportName` and `export_name: ExportName` and **no `[[LocalName]]` field at all**. Confirms three domains are three: an entry that carries only D2 exists. `graph.rs:814` resolves through `entry.export_name`, not `import_name` — the spec's own step 5 wording. |
| **T8** | `instn-star-props-nrml.js` — `export * from` | `graph.rs:524-534` walks `star_export_entries` collecting `ExportName`s; `push_unique_name` is `Vec<ExportName>`; `graph.rs:590-592` excludes `ExportName::DEFAULT`. Then `namespace.rs:738` resolves each to a `ResolvedBindingIr` and `namespace_target_reference` maps to `MergedName`. One of the three defects of `e27c01b1e`. |
| **T9** | `instn-named-err-not-found.js` | `resolve_export` returns `NotFound`; `ModuleLinkErrorIr::MissingExport { import_name: ExportName }`. Confirm the lookup side: constructing that error from a `LocalName` is `E0308`. |
| **T10** | **Adversarial, paper trace of `e27c01b1e`.** Two units each with top-level `let x`, one re-exporting the other. | Trace both units through `link.rs:495` and `namespace.rs:244`. Confirm (i) `merged_in` applies no prefix, so both map to `MergedName("x")` and `link.rs:499-503` reports the collision — invariant M3 / ledger R2, **not** silently mislinked; (ii) `MergedName::minted(0, ..)` and `MergedName::minted(1, ..)` differ; (iii) there is no expression that produces `$m0$$m0$x`, because no constructor accepts a `MergedName` or a `LocalName` and returns a prefixed name. **This is the trace that decides whether the brief's "apply exactly once" is carried; per C1 it is carried by there being no such function at all, which is stronger.** |
| **T11** | **Adversarial, compile-time.** Swap the two field initialisers at `record.rs:883-884`, then at `record.rs:963-966`. | Both must be `E0308: expected LocalName, found ExportName` / vice versa. The second is the coincident-name construction and is the one that fails today. |
| **T12** | **Adversarial, compile-time.** Feed `BUILTIN_OBJECT_ASSIGN_FUNCTION_ID` to `module_function_id(0, ..)`. | The call must fail with `E0425: cannot find function` — the function is deleted (K8). Confirm no other call site was module-qualifying a `FunctionId`, i.e. that deleting it did not remove a needed guard. **[dry-run obligation D4]** |
| **T13** | **Adversarial, compile-time.** Widen `MergedName::anonymous_default` from `$d{u}$` to `$default{u}$`; separately raise `MAX_LINKABLE_MODULE_UNIT_ID` to 10 000. | The first must fail assertion **V2**; the second must fail **V4** (the `import.meta` budget binds first, at 11 = 11). Both at `cargo check`, with no test run. |
| **T14** | **Adversarial, compile-time.** Add a `UnitCellRole` variant with suffix `"component.completion"`. | Must fail assertion **V3**. This is the deleted function reconstructed as a would-be regression. |

Additional obligations recorded above: **D4** (§5.2, K8, T12), **D5** (§2.5),
**D6** (§5.4 `namespace.rs` maps), **D7** (§5.4 `link.rs:566`).

---

## 8. Verification ladder for the encoder

| After | Command | Expected |
|---|---|---|
| §5.1 step 1 | `cargo check -p porffor-ir` | green; V1–V6 evaluated |
| §5.1 step 2 | `cargo check -p porffor-ir` | green — this *is* the proof the five functions were dead |
| §5.1 steps 3–5 | `cargo check -p porffor-ir` | green |
| §5.1 step 6 | `cargo test -p porffor-ir --lib` | no new failures against the pre-batch count |
| whole area | `cargo check -p porffor-aot-wasm` | green with **zero** edits in that crate — the containment claim (§6) |

Rung G (`emit_golden`) is **required** before this is called done: this is a
pure refactor of `porffor-ir`, so `diff -r target/golden/before
target/golden/after` must be empty. `batch-workflow.md` §"Rung G — the
refactor gate". Note the untracked-file caveat there: `binding_names.rs` must
be moved aside, not stashed, when capturing the baseline.


---

## 9. Encoder's record

Written by the ENCODER stage. Everything below is what the code now does, and
where it departs from §2–§6 it says so.

### 9.1 What was built

`crates/porffor-ir/src/binding_names.rs`, declared as `mod binding_names;` next
to `mod analysis;` in `lib.rs` and re-exported by `pub use binding_names::*;`
next to `pub use names::*;`. A second, `pub(crate) use binding_names::{…}` line
carries the seven `pub(crate)` spelling constants to `modules::source` and
`modules::record`, which is what ties V1/V2/V4/V7 to text those files actually
match on.

Five types, as §2: `SourceName`, `LocalName`, `ExportName`, `MergedName`,
`UnitCellRole`. Seven const assertions, V1–V6 as specified plus one addition:

- **V7** ties `IMPORT_META_HEAD` (`"import"`) and `IMPORT_META_TAIL` (`"meta"`)
  — the two fragments `rewrite_import_meta`'s span guard checks — to
  `IMPORT_META_TEXT` (`"import.meta"`), whose length V4 budgets against. Without
  it V4 constrains a constant no product code reads, and the tie §2.6 asks for
  is nominal. With it, changing the guard to accept a differently-shaped
  meta-property fails to build.

Two additions to §2's API, both to keep a *total* operation total:

- `LocalName::from_bound_name(name) -> LocalName` — see ledger **R5**. It is
  the single L2 decision point in the crate.
- `ExportName::default_export() -> ExportName` — a named constructor for the
  one `[[ExportName]]` the specification fixes, so the four sites that build it
  do not each write `ExportName::new(ExportName::DEFAULT)`.

`UserFunctionId` was **not** built, per §4/K8.

### 9.2 What was deleted

All eleven `names.rs` minting functions (`names.rs:29–126`), `ModuleGraphIr::cell_name`,
`ModuleNamespaceExportIr::cell`, `ModuleNamespaceIr::cell_for`,
`DynamicComponentIr::completion_cell`, `modules::record::module_binding_reference`
and `modules::record::import_meta_binding_name`. The two design notes §5.2 asks
to preserve are now prose in `modules::dynamic`'s and `modules::namespace`'s
module docs, and both name the `UnitCellRole` variant a future cell would have
to become.

The ad-hoc re-validation the types replace is gone with them: the `*default*`
sentinel comparison existed at `record.rs:252`, `record.rs:308`, `record.rs:316`
and `record.rs:981` and now exists once, inside `SourceName::new`.

### 9.3 Departures from the retrofit map, and why

1. **`is_binding_identifier` keeps its empty-name arm.** §5.4 says only that its
   `*default*` sentence is deleted, which it is. The arm stays because it is now
   the only thing standing between an empty spelling and emitted Script text —
   ledger **R5**.
2. **`ensure_namespace`'s export filter** was `graph.cell_name(&target)?`, which
   returned `None` exactly for `Ambiguous`/`NotFound`. With `cell_name` deleted
   the filter is written as that condition directly
   (`matches!(&target, ResolvedBindingIr::Resolved { .. })`) rather than through
   `namespace_target_reference`, which would additionally have dropped
   unspellable names — a *behaviour change*, since `namespace_object_source`
   reports those as an unsupported diagnostic rather than silently omitting the
   key. Preserving the old partition is what keeps rung G honest.
3. **A new `ModuleLinkErrorIr` variant, `TooManyUnits`,** carries ledger **R3**'s
   rejection. `ModuleLinkErrorIr::code` and `::message` are exhaustive matches
   with no catch-all, so the variant forced both to be updated — which is the
   reason it is a variant rather than a bare `IrDiagnostic`.
4. **`ModuleUnitId` stays `pub type … = u32`,** as §6 requires.

### 9.4 Mistake classes: discharged or moved

| # | Outcome |
|---|---|
| **K1** | **Discharged.** `MergedName` has no `From<String>` and no constructor taking a `LocalName` or a `MergedName`. `format!("{prefix}{name}")` over a merged name cannot be written because `module_storage_prefix` no longer exists. |
| **K2** | **Discharged.** `LocalExportEntryIr`'s two fields are different types; `push_local_export` converts its one `&str` twice. |
| **K3** | **Discharged.** `resolve_export(&self, ModuleUnitId, &ExportName)`; `early.rs`'s check is `LocalName == LocalName`. |
| **K4** | **Discharged.** One `format!`, one closed role enum, V3 and V5. |
| **K5** | **Discharged** by V2, now tied to `DEFAULT_BINDING_LET` / `DEFAULT_BINDING_ASSIGN` / `EXPORT_KEYWORD` / `DEFAULT_KEYWORD`, the constants `modules::source` matches on. |
| **K6** | **Discharged** by V4, tied through V7 to the fragments `rewrite_import_meta` checks. |
| **K7** | **Discharged by deletion.** No function in the crate returns a `$m{unit}$`-prefixed local name. |
| **K8** | **Not applicable**, as §4 predicted; the three functions are deleted and no `UserFunctionId` was built. |
| **K9** | **Partial**, as §4 states. `UnitCellRole::ALL` is one enumeration point; prelude-assembly ordering is a different area. |

Nothing was moved from the mistake-class table to the ledger. Two *new* ledger
entries were added (**R5**, **R6**) for obligations the encoder found rather
than inherited.

---

# 10. Amendments from the dry-run discrepancy pass (applied)

**This section supersedes every claim it names.** The code changes it describes
are in the tree.

## 10.1 V6 could not catch what it claimed; K4 was not discharged

`ALL`'s declared type `[UnitCellRole; 5]` hardcoded its own length, so
`assert!(UnitCellRole::ALL.len() == 5)` compared 5 against 5 by construction.
Adding a sixth variant forced an edit to `suffix()` (`E0004`) but **not** to
`ALL`: the tree still compiled with a five-element `ALL`, V6 still passed, and
the new role's suffix was checked by neither V3 (identifier-legality, M4) nor V5
(distinctness) — the exact regression K4 exists to prevent, since `import.meta`
and `component.completion` are the two suffixes deleted for failing V3.
§9.4's "K4 Discharged" was wrong.

The enum, `ALL` and `suffix()` are now three expansions of one
`unit_cell_roles!` row list, so a short, long or out-of-order `ALL` is
unrepresentable and V3/V5 quantify over the whole domain by construction. `ALL`
becomes `&'static [UnitCellRole]`. **V6 and `all_is_in_declaration_order` are
deleted** — both had become unfalsifiable — and what V6 used to check is
recorded in the macro's doc comment so nobody reinstates it. **K4 is discharged
by construction**, not by an assertion.

## 10.2 The minted and source ranges are *not* disjoint by construction

§1.3, ledger R2 and `MINTED_PREFIX`'s doc all asserted disjointness. The
quantifier ranges over what the *compiler* mints, not over what *source text*
may spell: `merged_in` is the identity on a source name, and `$m0$namespace`,
`$m1$meta` and `$d0$` are all legal `BindingIdentifier`s. Nothing checked it —
`check_linkable`'s collision map holds only per-unit bindings, and the one
prefix guard that existed covers `LINKER_NAME_PREFIX` (`$porffor$module$`), a
different family. A module that *declares* `$m0$namespace` produced a
duplicate-declaration SyntaxError from the merged script for a legal module; a
module that merely *reads* one had the read silently captured by the prelude's
own cell.

Added: `MergedName::is_minted_shaped`, derived from `MINTED_PREFIX` /
`ANONYMOUS_DEFAULT_PREFIX` / `UNIT_ID_TERMINATOR` so predicate and generators
cannot drift, and a second guard in `check_dynamic_import_linkable` — which
already runs unconditionally from `linked_script_source` and already walks every
unit's environment in the merged spelling. The **read**-of-an-undeclared-global
case remains: a program that reads a `$m<u>$…`-shaped global it never declares
still resolves to the prelude's cell. Recorded as new ledger entry **R7**;
closing it needs the per-unit renaming pass R2 already defers.

## 10.3 Ledger R1's stated reason does not survive the retype

R1 justified keeping `is_binding_identifier` with two examples, and neither
holds. A `\u`-escaped identifier is resolved to its code points by boa's
interner long before it reaches a `SourceName`. An astral-plane identifier
**passes**, because `char::is_alphabetic` accepts astral letters. Nor does the
emitter need ASCII: identifiers are written raw into UTF-8 merged source that
boa re-parses — only `push_js_string_literal` escapes to ASCII, and that is for
string keys. Its documented `*default*` job is genuinely dead, as §5.4 says, and
that was its only reachable job.

What it actually did was produce **false rejections**: legal `IdentifierName`s
containing `Other_ID_Start`/`Other_ID_Continue` characters or ZWNJ/ZWJ (U+2118,
U+212E, U+309B, U+309C, U+00B7, U+200C, U+200D) turned a conformant module into
an "unsupported" diagnostic. Option (b) of the fix is applied: the predicate is
kept as a conservative guard, its doc is rewritten to say all of the above, and
`is_identifier_start_char` / `is_identifier_part_char` widen it to cover
`Other_ID_Start`, `Other_ID_Continue`, ZWNJ and ZWJ. It **remains** conservative
for `IdentifierPart`'s `Mn`, `Mc` and `Pc` categories, which would need Unicode
tables this crate does not carry; that residual is stated in the doc comment and
is a *report*, never a miscompilation.

## 10.4 The unit/name pairing is now carried by the record

`LocalName::merged_in(&self, unit: ModuleUnitId)` took a bare `u32`, so the
"which unit does this name belong to" coordinate — the exact one commit
`e27c01b1e` got wrong — was carried by loop structure. Passing the importer's id
where the exporter's was required compiled, was silent for `LocalName::Source`
(`merged_in` ignores the unit there) and produced the wrong `$d<unit>$` cell for
`AnonymousDefault`. All ten call sites were correct; nine were correct only
because name and id came from the same loop variable.

`SourceTextModuleRecordIr::merged(&self, &LocalName)` is added, and the nine
sites that already hold the record now use it, so the id cannot be supplied
independently of the name's owner. The tenth (`namespace_target_reference`)
keeps `merged_in`: `module` and `binding` are destructured from the same
`ResolvedBindingIr::Resolved`, so its pairing is structural already, and the
code now says so. `merged_in` stays `pub` (making it `pub(crate)` would break
the public intra-doc links in this domain's module header for no gain).

## 10.5 K1 and K2 were overclaimed; the load-bearing halves hold

§2.4's "there is no `From<String>`, so a bare `String` cannot become one" is
literally false. `LocalName::from_bound_name(arbitrary).merged_in(0)` yields
`MergedName(arbitrary)`; `ExportName::new(local.spec_name())` yields an
`ExportName` holding a `[[LocalName]]`, `*default*` included;
`LocalName::from_bound_name(export.as_str())` yields the converse. All three
compile because every constructor takes `impl Into<String>` / `&str`.

What does hold, and is the part that matters: the *implicit* mixing that shipped
is `E0308`, and no double-prefix expression exists at all (T10.iii). The claims
are restated as **"no implicit conversion exists; a deliberate re-derivation
still has to name `spec_name()`/`as_str()` at the call site."** Closing the
`spec_name()` route would need a `SpecName<'_>` wrapper with no `Into<String>`;
not done, and the reverse direction is not worth closing.

## 10.6 New ledger entry: nothing forces an emitted identifier to be a `MergedName`

The four emitters accumulate generated Script text with
`String::push_str(&str)`, so `text.push_str(entry.local_name.spec_name())` —
emitting `*default*`, the zero-prefix half of K1 — compiles at any of them.
Every current site is correct (all go through `MergedName::as_str()` or
`namespace_target_reference`), but the guarantee lives at the `push_str` as a
convention rather than in a type, and §1.4 M2 did not record it.

Recorded as ledger **R8**. The fix is a thin `ScriptText` builder in
`binding_names` with `push_literal(&'static str)`,
`push_identifier(&MergedName)` and `push_string_literal(&str)`, replacing the
raw `String` those four functions accumulate into — an identifier position would
then accept only a `MergedName`. Not applied here: it rewrites the emitters that
produce merged source text and so belongs to a lane with a rung-G budget.

## 10.7 `import_name_text`'s `"*"` is not a name no module could export under

Its doc claimed exactly that. 16.2.3.1 makes `ModuleExportName : StringLiteral`
legal, so `export { x as "*" } from "m"` spells it, and invariant E1 relies on
that openness. A `MissingExport`/`AmbiguousExport` diagnostic naming `*` is
therefore ambiguous between a namespace import and a literal `"*"` export.
Diagnostic text only, no miscompilation. The false half of the sentence is
deleted and replaced with the ambiguity it actually has.

## 10.8 The `unwrap_or(ModuleUnitId::MAX)` saturations: scope corrected

§9.3 item 3 said the saturation "is gone". Six such expressions remained in
product code (`graph.rs` ×5, `dynamic.rs` ×1). All six are provably unreachable
— `build_graph` rejects `units.len() > MAX_LINKABLE_MODULE_UNIT_ID` at the one
mint site, so every later `try_from(index)` over `0..units.len()` succeeds — so
this was a scope error in the claim rather than a live defect. It is worth
recording because the argument that makes them safe is exactly the cap R3
introduced, and a future change to the cap silently re-arms them.

Restated: **gone at the mint site, and thereby unreachable at the six
index-to-id conversions that remain.** All six are now `.expect("unit index is
capped by build_graph …")`, so the reliance on the cap is stated where it is
relied on instead of hidden behind a saturating default.

## 10.9 The three prelude-global guards disagreed

`link.rs`'s renamed-import check tested `OBJECT_NAME | SYMBOL_NAME |
GLOBAL_THIS_NAME`; `namespace.rs`'s two alias checks and its shadowed-globals
check tested only `OBJECT_NAME | SYMBOL_NAME`. So
`import * as globalThis from './m.js'` emitted
`const globalThis = $m0$namespace;` into the merged scope ahead of
`binding_alias_prelude`'s `Object.defineProperty(globalThis, …)`, which then
defined every renamed-import alias on the namespace object — a silent wrong
answer where the other two names give a diagnostic. Pathological input, but the
same merged-name hazard M3 governs, and the asymmetry was invisible because the
three lists were three literals.

One `PRELUDE_GLOBALS: [&str; 3]` and one `shadows_prelude_global` now back all
four sites.

## 10.10 The `local_export` test helper's witness was half-real

§5.6 called it "the adversarial trace T6's compile-time witness". It was that
for the field initialisers *inside* it, not for its call sites: both parameters
were `&str`, so `local_export("x", "y")` and `local_export("y", "x")` both
compiled. The parameters are now `LocalName` and `ExportName`, with `local(..)`
and `ExportName::new(..)` at each call site, so the swap is `E0308` where it
would actually be written.

## 10.11 Obligations that resolved in the contract's favour

- **E1** (`[[ExportName]]` is open) — discharged by design, correctly not by
  validation; the UTF-16 escaping path still round-trips unpaired surrogates.
- **M2** (apply exactly once) — discharged by type in the strong form §1.4
  predicts: there is no prefixing function at all, `MergedName`'s only
  constructors take no name-shaped argument, and `$m0$$m0$x` is inexpressible.
  The zero-times direction is the weaker one — see R8 in 10.6.

## 11. Integration record — the compile gate

Integrator's section, written after running the gate. Commands: `cargo check -p
porffor-ir --all-targets`, `cargo xc`, `cargo fmt --all -- --check`.

**Green, with zero integrator edits to this area.** §9's containment claim held
under the compiler, which is the first real test it has had: retyping the seven
module IR types and deleting the eleven minting functions required **no** edit
in `porffor-aot-wasm` or any other crate. The encoder's §6.2 worry — a fourth
`is_binding_identifier` call site that grep missed — did not materialise; there
are three, all passing merged names.

The ~1,000-line blind retype across seven files produced **no** type errors at
integration, and the predicted `&ExportName`-vs-`ExportName` mismatches at test
call sites did not appear either. Every const assertion (V1–V7) evaluated,
including the tight V4 (11 ≤ 11 at the unit-id cap) and V7 tying it to the text
`rewrite_import_meta` reads.

The deletions are the load-bearing result and they are now *proved* rather than
measured: five minting functions, one field, one method and `cell_name` are gone
and the workspace still builds, which is what "these had no product call site"
means when a compiler says it.

Formatting note: `binding_names.rs`, `link.rs`, `namespace.rs` and `record.rs`
needed `cargo fmt` (import ordering and one over-wrapped boolean chain); the
tree is now clean workspace-wide.

**Unchanged and still open** after the gate: **R5** (`SourceName::new` rejects
only `*default*`, not the empty string — the deviation stands, with the
encoder's reasoning), **R8** (§10.6, nothing forces an emitted identifier to be
a `MergedName`), and the R3 unit test that builds 10,001 module records, which
was not run here and may still want `#[ignore]`.
