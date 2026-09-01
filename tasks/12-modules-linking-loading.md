# T12 — Modules, linking, loading and namespace objects

**Status:** In progress — module IR/emission exists; graph linking and dynamic import are incomplete

**Parallel group:** Feature lane  
**Depends on:** T06, T07, T08, T09, T10  
**Blocks:** Module portion of T14, T23 and T26

## Current repository state

The Rust path now has a host loader, parse-once graph assembly, export
resolution, evaluation ordering, live-binding aliases and an AOT dynamic-import
registry. Dynamic-import components retain the full phaseful occurrence whose
phase-free key the host resolved, and the generated executor preserves the
specifier/options/coercion/property-read order. The original parse goal is also
retained as a closed root-`this` binding domain. In a flat eager synchronous
Module-entry graph, direct and lexical-arrow module-root reads lower to
`undefined` without changing ordinary function activations or Script-global
`this`; existing Script-entry, deferred and top-level-await wrappers retain
their activation route and obtain the same value from a bare strict call. The
source-text bridge also carries only closed span-stable edits: erased Unicode
module syntax cannot move later byte offsets, and an anonymous `export default`
may be split across lines without losing the original line-terminator sequence.
Those are foundations rather than completion: exact module namespace exotic
behavior, lazy dynamic target evaluation, all cyclic/deferred/async evaluation
cases and the `language/module-code` current-pin closure remain unverified.

## Objective

Compile complete ECMAScript module graphs ahead of time, with live bindings, cyclic linking and host-controlled resolution, without evaluating module source through an embedded interpreter.

## Compile-time model

Add module IR for:

- requested modules and import attributes present in the pin;
- local/import/indirect/star export entries;
- top-level declarations and module environment bindings;
- `import.meta` and dynamic import expressions;
- async/top-level-await status and dependency edges;
- source phase/module-source features if present in the pinned suite.

The CLI/library should accept an entry module plus a loader/resolver and produce a deterministic graph. Cache modules by normalized host key and reject inconsistent duplicate loads.

## Linking and evaluation

Implement spec-shaped phases:

1. parse all reachable modules;
2. resolve exports, including ambiguity and star cycles;
3. create module environments and namespace objects;
4. instantiate declarations/functions;
5. evaluate in dependency order with cycle handling;
6. coordinate async evaluation/top-level await through T14's job model.

Live imported bindings must reference exporter cells and remain read-only to the importer. Cyclic graphs must not be flattened into initialization-order guesses.

## Module namespace exotic object

Implement exact namespace behavior:

- sorted exported-string keys plus symbols in correct order;
- live getters/read-only semantics;
- null prototype, non-extensibility and `@@toStringTag`;
- custom internal methods and descriptor behavior;
- identity caching per module.

## Host loader contract

Define a Rust trait for resolve/load with referrer, attributes and module type. The Test262 loader may use repository files; product embedders may supply other sources. Prevent path traversal in the default filesystem loader. Do not bake Test262 paths into module semantics.

## Artifact strategy

Document whether a graph is emitted as one Wasm module or multiple linked modules. The first complete implementation may emit one module, but module records and live bindings must remain explicit so the design can evolve. `build wasm` must include compiled semantics, not source strings fed to a runtime parser.

**Decision (T12 foundation): one Wasm module per graph.**

The graph is linked at compile time into a single `ScriptIr`. Per-module identity survives in `ProgramIr::modules` (`ModuleGraphIr`), which carries the Source Text Module Records, the resolved import bindings, the namespace descriptors, the evaluation order and its strongly-connected components.

Every eagerly evaluated module's top-level bindings live side by side in one
merged activation environment. Source-spelled bindings keep their source name
(with collision rejection while the per-unit renaming pass is still open), an
anonymous default receives `$d<k>$`, and only compiler-owned cells use the
`$m<k>$` family. A cross-module binding is therefore an ordinary read of the
exporter's cell rather than a copied value, which keeps the binding live without
runtime indirection. Span-derived function ids remain unique because the
source units are concatenated before the single lowering pass.

### Module identity domain

`ModuleRequestKeyIr::specifier` is source spelling interpreted relative to a
referrer. The key combines that spelling with canonical attributes and is the
phase-free identity `ModuleRequestsEqual` and host resolution consume.
`ModuleRequestIr` adds occurrence phase for dispatch and evaluation, but never
becomes a module-map key. `ModuleKey` is the distinct, opaque identity returned
by the host after resolution/canonicalization. The engine retains `ModuleKey`
through its parse-once discovery map, `ModuleSourceIr`,
`SourceTextModuleRecordIr`, the IR graph key map and `DynamicComponentIr`; it is
never recovered by comparing a raw request spelling with a normalized key. This
prevents a missing host resolution from becoming an accidental match merely
because the two strings happen to be equal, without changing graph sharing or
evaluation order.

### Evaluation dependency domain

`ModuleEvaluationDependencyIr` is the closed edge type consumed by Tarjan
ordering and top-level-await propagation. Its private target can be constructed
only from an evaluation-phase request; `import defer` and `import source` remain
part of loading, linking and evaluation-mode classification but cannot become
ordinary evaluation dependencies after resolution erases the request context.
Non-eager units consequently have neither `[[AsyncEvaluation]]` nor pending
async dependencies.

### Runtime participation domain

Loading/linking participation and artifact participation are distinct. A unit
reached only through `import source` remains parsed and linked, and an active
referrer can receive its module source object, but the unit contributes no body
or runtime scaffolding of its own. `ModuleMaterializationModeIr` is the private
closed domain for the two artifact-present cases (`Eager` and `Deferred`),
derived once and exhaustively from `ModuleEvaluationModeIr`;
`ModuleGraphIr::materialized_units` is the common source for namespace and
module-source aliases, `import.meta` cells, dynamic-import dispatchers and
runtime-only collision checks. A namespace carries that typed mode rather than
a parallel deferred boolean and cannot be created for a source-only unit.

Dynamic components are discovered in full before evaluation-mode
classification, then components whose referrer does not materialize are
removed from the artifact registry. This preserves dynamic edges needed by the
fixed point without compiling an `import()` call site whose containing module
can never run. The precise invariants and regression shape live in
`docs/rust-rewrite/contracts/module-runtime-participation.md`.

### Root `this` binding domain

The merged graph is reparsed with the Script goal for one shared lowering, but
that implementation goal does not replace the source goal's Environment Record
semantics. `RootThisBinding` is derived once from the original goal and is
required by every lowerer construction. `CurrentThisBinding` then distinguishes
that root binding from a real function activation. Flat eager synchronous
Module-entry root reads, including nested lexical arrows, lower directly to
`undefined`; root Script reads retain the global-object operation; ordinary and
derived activations remain dynamic. Existing Script-entry module closures,
deferred thunks and top-level-await async wrappers continue to use activation
`this`, supplied as `undefined` by their bare strict invocation. Only
global-object root reads contribute to `ScriptIr::top_level_this_uses` and
therefore to AOT global bootstrap. The invariant and regression shape live in
`docs/rust-rewrite/contracts/module-root-this-binding.md`.

### Span-stable module-syntax rewriting

The linker erases module-goal-only syntax before its merged Script reparse.
Every edit is either byte-width-aware blanking or a replacement constructed
against the exact source slice it erases. Both preserve byte length and the
ordered ECMAScript LineTerminatorSequence list, including the distinction
between one CRLF and separated CR/LF sequences. A replacement reserves a
non-terminator barrier when relocation would fuse those sequences, including
across the edit boundary into the untouched initializer suffix. The same
lexical helper ends line comments at CR, LF, CRLF, U+2028 and U+2029.
Anonymous default exports therefore retain the byte offsets and line numbers
later passes consume even when `export` and `default` are separated by any of
those sequences or by comment trivia. The replacement may move those erased
sequences within their own span, so this is not a source-column mapping claim.
The invariant and regression shape live in
`docs/rust-rewrite/contracts/module-syntax-span-stability.md`.

The replacement admission boundary now also carries its three failure meanings
as private, non-derived `SpanStableReplacementError` state. Its two invalid-span
producers, one generated-line-terminator producer and three checked-width
producers remain in their original order; the default-export rewriter maps all
three rows exhaustively to the existing diagnostics. A recursive structure
guard fixes the declaration, 13 source mentions, six producer conditions and
exact three-row projection, while the existing focused owner unit rejects a
generated line terminator. The dedicated structure target passes `3/3` and that
exact owner unit passes `1/1`. Independent review confirmed the capability
boundary, census, six ordered producers and diagnostic mapping. The coordinated
workspace checkpoint passes `cargo fmt --all -- --check`, `cargo xc`,
`git diff --check`, the module boundary check and the task-plan check; the
compile retains the repository's existing warnings. This is a source-equivalent
capability closure: it changes no module syntax, edit bytes, diagnostic text or
emitted Wasm. The invariant lives in
`docs/rust-rewrite/contracts/span-stable-replacement-error.md`.

The source scanner's slash context is also closed as private
`SlashMeaning::{Divide, Regexp}` state. Line and block comments remain ordered
before one borrowed exhaustive slash dispatch: regexp context consumes the
literal and enters divide context, while divide context consumes only the
operator and reopens expression context. The exact nine divide-context and ten
regexp-context producers and both transitions are pinned in structure, with
owner regressions for regexp and division spellings plus line and block
comments in both contexts. The guard also fixes every producer mapping,
comment-state preservation and the module scanner's exact 23-mention domain
beside the dynamic-source scanner's separate 18 mentions. Its three structure
checks and all four focused owner units pass, and independent dry review is
clean. The invariant lives in
`docs/rust-rewrite/contracts/module-source-slash-meaning.md`.

Dynamic-import component identity now has one construction authority.
`DynamicComponentIr` keeps its target key, phaseful request, referrer and target
fields private, and graph discovery constructs the only complete row from one
host-resolution decision. `ModuleGraphIr` keeps the resulting vector private
and exposes a read-only component slice, so callers cannot splice a request or
target from another graph into an already-linked artifact. Named accessors
preserve public inspection without reopening mutation. The invariant is
recorded in
`docs/rust-rewrite/contracts/dynamic-component-authority.md`; lazy target
evaluation and real namespace exotic objects remain open.

Dynamic-import targets are compiled into the same artifact, not separate Wasm
modules, and dispatch through the artifact's generated dynamic-import registry.
Target-body evaluation is not fully lazy yet: `StatementIr::ModuleUnitOnce` and
its Wasm guard exist, but the linker does not yet populate that seam for module
bodies. Splitting a graph into several linked Wasm modules later remains a
backend change with no source-loading fallback.

`lila-ir` performs no IO. The host resolves and reads the whole transitive closure up front (`lila_engine::load_module_graph`) and hands it to `lila_ir::lower_module_graph` as `ModuleGraphSources`.

### Artifact-local dynamic import

`import()` must work without runtime source compilation. Every statically
discoverable dynamic-import target is resolved with the graph and compiled as a
guarded module unit inside the same graph artifact. At runtime, the exact typed
occurrence — referrer, specifier, phase and attributes — must match an entry in
that artifact's precompiled registry. A runtime-computed specifier may select
only such a precompiled entry; a request with no exact match rejects its promise
with a host resolution error. There is no source-loading, parsing or evaluation
fallback inside the artifact. This keeps dynamic import out of T13's
unsupported dynamic-source bucket while preserving the one-Wasm-module-per-
graph decision.

### Dynamic-import request and options contract

`EvaluateImportCall` has two different abrupt-completion boundaries. The
specifier expression and options expression are evaluated, in that order,
before `%Promise%` creates the returned capability; an abrupt completion there
is therefore thrown to the caller. `ToString(specifier)`, `Get(options,
"with")`, enumerable-own-property discovery, and each attribute-value `Get`
happen after the capability exists; failures in those operations reject that
promise. Attribute values are required to be strings, and the resulting list
is sorted by key in UTF-16 code-unit order before host resolution.

The AOT graph preserves that boundary by leaving both source operands at the
rewritten call site and doing coercion and option inspection inside the
generated promise executor. The host accepts the phase-free
`ModuleRequestKeyIr`; a dynamic component retains the corresponding full
`ModuleRequestIr`, and referrer plus that occurrence is the runtime registry
identity. Phase therefore stays available to dispatch without splitting host
resolution into parallel keys.
Literal `{ with: { ... } }` attributes are carried into graph discovery and
therefore reach `HostModuleLoader::resolve`. An option shape whose eventual
attributes depend on runtime code discovers the attribute-free request as the
only safe baseline. At runtime it may resolve only to an exact request variant
already compiled into the component registry; no unknown module type triggers
runtime parsing or loading. The default filesystem host currently supports no
attributes, so attributed dynamic requests are retained and rejected honestly;
embedders that implement a module type can resolve the same typed requests
without changing compiler IR.

At 2026-08-25, dynamic-import call-site rewriting no longer carries the
module-local versus Script-entry-exported dispatcher choice as a Boolean.
`DynamicImportDispatcherReference` is the private, non-`Clone`, non-`Copy`
two-variant domain. Its six lexical mentions are the declaration, owned
parameter, two producers and two consumer arms; the two public rewriters pin
its variants and `rewrite_calls` has one exhaustive consuming projection. The
strengthened lexical structure target records that complete ownership boundary
and full rewrite-body fingerprint; it passes `4/4`. The exact module-local and
Script-entry-export rewrite units each pass `1/1`. The original checkpoint's
linker desugaring witness, workspace, golden and repository policy gates remain
the wider semantic baseline; this derive-only follow-up is source-equivalent and
adds no ABI, runtime-root or helper-count delta. A broader pinned Test262 module
run was not rerun.

The dynamic-import scanner's slash context is now the private, non-derived
`SlashMeaning::{Divide, Regexp}` domain. Line and block comments remain ordered
before one borrowed exhaustive dispatch: regexp context consumes the literal
and enters divide context, while divide context consumes only the operator and
reopens expression context. The structure boundary fixes the exact 20 owner
mentions, nine divide-context and seven regexp-context producers, every mapping,
both transitions and the separate module-source scanner census. The focused
contract and evidence live in
`docs/rust-rewrite/contracts/dynamic-import-slash-meaning.md`. This is a
source-equivalent scanner invariant; it changes no rewritten source, module
resolution, job order or emitted Wasm. The dedicated and neighboring structure
targets pass `3/3` each, and the three existing focused module units plus the added
division/rewrite witness pass `1/1` each. Scoped formatting, diff and task-plan
checks are green. Independent review confirmed the complete scanner-body and
lexical-state census. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings. Broader Test262 module verification was not rerun.

### Canonical request identity

Module request attributes cross graph and host boundaries only as
`ModuleRequestAttributesIr`: an immutable, duplicate-free list sorted by
UTF-16 key order. `ModuleRequestKeyIr` keeps specifier and attributes private
and is the sole phase-free identity used by `HostModuleLoader::resolve`, public
resolution rows and graph maps. `ModuleRequestIr` separately carries phaseful
occurrences for `[[RequestedModules]]`, entry tables, evaluation classification
and the artifact registry. Evaluation, defer and source occurrences with the
same key therefore share one host resolution but remain distinct at dispatch.

`SourceTextModuleRecordIr::requested_modules` is the phaseful source-order list,
deduplicated by `(key, phase)`. `module_resolution_requests` is its separately
named phase-free projection for host discovery only. Evaluation and linking
walk the phaseful list, so `source m; eval n; eval m` retains evaluation order
`n, m` rather than being reordered by the first occurrence of key `m`.
Duplicate public rows for the same `(referrer, key, target)` coalesce; rows
naming two targets for one key produce `InconsistentResolution` with no
last-write winner.

The contract and public embedder regressions live in
`docs/rust-rewrite/contracts/module-request-identity.md`.

### Module-entry source authority

The entry source choice is now the closed `ModuleEntry` domain. `HostLoad`
requires the host loader to provide the entry, while `InMemory` carries the
exact embedder source beside the locator used for canonical identity and
relative dependency resolution. The already-parsed module and Script
handoffs accept only `entry_locator: &str` plus their typed parse product, so
an in-memory override cannot coexist there and be silently ignored. The
invariant and focused evidence live in
`docs/rust-rewrite/contracts/module-entry-source-authority.md`.
At 2026-08-27, the dedicated structure target passes `4/4`, the exact host and
in-memory behavior witnesses pass `1/1` each, the focused module-loader set
passes `14/14`, and `cargo check -p lila-engine` passes with the repository's
existing warnings. Scoped Rust formatting and diff checks are green; the wider
Test262 module suite was not rerun for this source-authority-only boundary.

Attributed re-export requests retain their full typed request from the Boa AST
through both Lila module-record passes. The public AST variant carries the
private-field `ReExportRequest`, whose sole constructor accepts only a
specifier and attributes and constructs an evaluation-phase `ModuleRequest`;
custom deserialization rejects other phases. Imports and re-exports share one
attribute parser, while the export parser has no phase parameter. Star,
namespace and named forms preserve the exact attributes in requested modules
and export entries. Canonical ordering lets an attributed import and re-export
deduplicate by one request key, and an in-memory graph witness makes the
host-resolution row's attributes load-bearing. The structural and semantic
evidence lives in `docs/rust-rewrite/contracts/module-request-identity.md`.
Boa's pre-existing attribute-order-sensitive `ModuleRequest` equality remains
unchanged; Lila's canonical IR boundary is the ordering authority. The
implementation and cheap static checks were completed on 2026-09-01. Product-path
`cargo check -p lila-ir --lib` is green, as is a disposable external crate
compiling the vendored `boa_ast` path with `serde` and `arbitrary` enabled; the
repository-root `cargo check -p boa_ast` form is invalid because that vendored
crate is not a workspace member. The full `lila-front` suite passes `152/152`,
its duplicate-attribute focus passes `3/3`, the attributed record and graph
witnesses pass `2/2`, and the surrounding record and graph groups pass `31/31`
and `56/56`. The new structure target passes `4/4`; its six adjacent targets
pass `20/20`. The exact export duplicate-key Test262 case passes `1/1`.
Formatting, diff, module-boundary, task-plan and scoped source-audit gates are
green. The default filesystem host still rejects attributed requests, and no
positive attributed-module execution or broad attribute-directory result is
claimed.

## Acceptance criteria

- Static import/export, re-export, namespace import and side-effect-only import cases pass.
- Cycles, live bindings, star ambiguity, TDZ and evaluation order pass.
- Module namespace descriptor/internal-method tests pass.
- Top-level `this`, strictness, `import.meta` and host resolution behavior are correct.
- Parse/link/evaluate failures are classified at the right phase.
- Dynamic import is integrated with promises/jobs rather than synchronous source evaluation.
- The pinned `language/module-code` and related module builtins reach zero failures.

## Required tests

```sh
cargo test -p lila-ir module_ --quiet
cargo test -p lila-engine module_ --quiet
cargo test -p lila-cli module_ --quiet
./target/debug/lila test262 run language/module-code --execution-backend wasm --timeout-ms 120000 --threads 4
```

Add filesystem-loader tests for cycles, missing modules, traversal rejection, duplicate normalized specifiers and import attributes.
