# T12 — Modules, linking, loading and namespace objects

**Status:** In progress — canonical Wasm GC graphs, native source intrinsics and genuine JSON default-export records are authored; full module acceptance and joined verification remain

**Parallel group:** Feature lane  
**Depends on:** T06, T07, T08, T09, T10  
**Blocks:** Module portion of T14, T23 and T26

## Current repository state

The 2026-10-07 source-phase scope audit separates required proposal behavior
from optional host extensions. The supported Source Text and JSON records have
empty `[[ModuleSource]]`; neither supplies a positive source representation.
The [Source Phase Imports draft](https://tc39.es/proposal-source-phase-imports/#sec-hostgetmodulesourcemodulerecord)
allows this boundary and defaults `HostGetModuleSourceModuleRecord` to
`not-a-source`. This pin tests the native `%AbstractModuleSource%` intrinsic and
source-phase rejection, including the distinct existing-JavaScript and
missing-target cases; it supplies no positive concrete non-JavaScript source
loader fixture. Implementing another host module kind is an optional extension,
not a missing T12 acceptance requirement. The intrinsic is already implemented;
full module acceptance and verification of the joined source remain open.

The joined source also classifies top-level `await using` as implicit TLA,
including lexical declarations and synchronous `for (await using ... of ...)`
heads with empty iteration or null resources. `ModuleBodyScan` publishes that
fact through the existing async graph lifecycle, so importers wait for disposal.
Record, graph and native Module controls are authored; this dry change makes no
runtime verification claim. See
[the async lifecycle contract](../docs/rust-rewrite/contracts/module-async-lifecycle.md).

The 2026-10-07 JSON batch adds a strict native parser and genuine default-only
synthetic records. Filesystem and typed embedded graphs preserve record kind,
exact bytes, canonical `type: "json"` attributes and shared identity. Evaluation
constructs data directly through the original Wasm GC JSON allocation/property
operations in the record Realm; no JSON text becomes a JavaScript wrapper.
Static malformed input rejects linking, dynamic malformed input remains an
import-job rejection, and JSON source phase remains `SourceUnavailable`.
Meaningful parser, graph, loader, oracle and Engine controls are authored but
unrun on this joined source. See
[the native JSON contract](../docs/rust-rewrite/contracts/json-module-native-default-export.md).
The historical JavaScript-only loader descriptions below retain their original
scope; this batch does not supply a positive proposal source representation.

The 2026-10-03 dry correction removes fabricated module source objects and the
retained merged driver. The current loader supplies JavaScript Source Text
Module Records, whose proposal source representation is empty. Static source
bindings and forwarded exports reject during linking; dynamic `import.source`
rejects with a fresh intrinsic SyntaxError after load/parse, before target
dependency loading or evaluation. Original host, parse and coercion failures
retain their own routes. Phaseful host requests load source-only targets once
and expand the cached target when Evaluation/Defer later needs it, including
agreeing duplicate host rows. Successful Module/Script/TLA/defer/cycle graphs
use canonical activations. Source review and authored controls are complete;
compilation and runtime verification remain pending. Canonical runtime graph
records now use the single semantic Wasm GC representation. The native
AbstractModuleSource constructor and prototype getter now have defining-Realm
intrinsic caches, actual native roots and a Test262-only host retrieval hook.
The harness consumes that intrinsic; the JavaScript substitute class is removed.
The constructor throws without observing a NewTarget prototype. Intrinsic and
created-Realm controls are authored; the original dry packet did not execute
them. Optional non-JavaScript source loader kinds are outside this pin's
required scope. See [the source-phase contract](../docs/rust-rewrite/contracts/source-text-module-source-phase-rejection.md).
Earlier focused results below retain their original source scope.

The prepared NEXT batch retains a valid one-node Script graph when its AST has
computed import calls but no literal discovery request. Operand/GetValue abrupt
completion, ordered coercion and fresh asynchronous rejection use the existing
AOT import-job dispatcher. Four paired Engine execution controls and the bounded
five-file/ten-mode pinned cohort are unexecuted on the joined source. This does not
claim historical missing-graph passes or close the separate source-phase boundary.
See [the computed import contract](../docs/rust-rewrite/contracts/script-computed-import-admission.md).

The active Script-entry batch replaces eager wrapper assembly for discovered
dynamic targets with canonical module owners and the same import-job lifecycle
used by Module entries. The original Script keeps its declarations, global
`this`, strictness and final completion; module discovery does not run target
bodies. Dynamic-only malformed targets and transitive link failures retain
their distinct load/dependency rejection stages without poisoning a valid
shared closure. The focused contract is
[`script-dynamic-import-jobs.md`](../docs/rust-rewrite/contracts/script-dynamic-import-jobs.md).
Its new engine target contains thirteen tests and eighteen graph executions,
including the unchanged pinned `update-to-dynamic-import.js` plus its original
fixture siblings in sloppy/strict Scripts, same-file Script/Module owners and
exact callable-source reflection after import rewriting. Compiler-owned byte
origins retain original function, arrow, method and whole-class text through
Unicode spans, default-export terminators and deferred wrappers.
The minimum real-suite cohort is
seven physical cases and fourteen executions. The focused Script tests and all
95 neighboring module tests pass. The current continuation rechecks all 42
module/eval tests, including exact source reflection after Unicode static-import
erasure and import-meta rewriting. Full IR verification passes 1,558 tests with
one ignored documentation example, and all-target checking passes. The seven
exact pinned cases pass all 14 executions on 2026-09-30. Broader workspace
verification remains pending; this
does not close T12 or publish new conformance counts.

The Rust path now has a host loader, parse-once graph assembly, export
resolution, evaluation ordering, live-binding aliases and an AOT dynamic-import
registry. Dynamic-import components retain the full phaseful occurrence whose
phase-free key the host resolved, and the generated executor preserves the
specifier/options/coercion/property-read order. Canonical graph execution gives
each materialized Module a private strict owner with module-root `this` fixed
to `undefined`, including nested lexical arrows. The original Script body
remains the root source tail after module instantiation; its global declarations,
strictness, `this` and completion are preserved. Ordinary calls keep their own
receiver. Module loading caches Modules separately from a Script at the same
host key, so importing the entry filename creates a distinct Module owner. The
source-text bridge also carries only closed span-stable edits: erased Unicode
module syntax cannot move later byte offsets, and an anonymous `export default`
may be split across lines without losing the original line-terminator sequence.
Those are foundations rather than completion: exact module namespace exotic
behavior, the complete cyclic/deferred/async evaluation surface
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

The graph is linked at compile time into a single `ScriptIr`. Per-module
identity survives in `ProgramIr::modules` (`ModuleGraphIr`), which carries the
Source Text Module Records, resolved import bindings, namespace descriptors,
evaluation order and strongly-connected components.

The canonical graph allocates a separate activation and binding environment
for each materialized Module. All environments and export cells are created
before imports and namespaces are linked; imported bindings alias the owning
exporter's cells rather than copying their values. Evaluation then follows the
phaseful requests through the module lifecycle. Source spellings can recur in
different owners without merging their declarations. A closed
`ModuleExecutionEntry::{Module, Script}` selects root execution: a Module entry
starts its canonical evaluation, while a Script remains outside the module
activation array and starts target evaluation only through import jobs. The
original Script record still occupies its graph unit slot, so module owner
identities keep their graph unit IDs. Source-phase Script graphs are explicitly
unsupported by this canonical artifact path.

### Module identity domain

`ModuleRequestKeyIr::specifier` is source spelling interpreted relative to a
referrer. The key combines that spelling with canonical attributes and is the
phase-free identity `ModuleRequestsEqual` and host resolution consume.
`ModuleRequestIr` adds occurrence phase for dispatch and evaluation, but never
becomes a module-map key. `ModuleKey` is the distinct, opaque identity returned
by the host after resolution/canonicalization. The engine retains `ModuleKey`
through its parse-once module discovery map, `ModuleSourceIr`,
`SourceTextModuleRecordIr`, the IR graph key map and `DynamicComponentIr`; it is
never recovered by comparing a raw request spelling with a normalized key. This
prevents a missing host resolution from becoming an accidental match merely
because the two strings happen to be equal, without changing graph sharing or
evaluation order. The root Script is outside the host module cache: the same
host key can identify its separately loaded Module source without reusing the
Script parse goal or harness-prefixed in-memory source.

### Evaluation dependency domain

`ModuleEvaluationDependencyIr` is the closed edge type consumed by Tarjan
ordering and top-level-await propagation. Its private target can be constructed
only from an evaluation-phase request; `import defer` and `import source` remain
part of loading, linking and evaluation-mode classification but cannot become
ordinary evaluation dependencies after resolution erases the request context.
Deferred and dynamically discovered owners do not begin async evaluation or
acquire pending async dependencies merely because they were discovered; those
runtime states arise only when their evaluation lifecycle starts.

### Runtime participation domain

Loading/linking participation and artifact participation are distinct. The
graph's source-only classification contributes no module body or runtime
scaffolding. Static source bindings reject at linking; dynamic source jobs in
Module and Script code reject after load/parse. A positive source-object loader
would be a future host extension. `ModuleMaterializationModeIr` is the private
closed domain for the two artifact-present cases (`Eager` and `Deferred`),
derived once and exhaustively from `ModuleEvaluationModeIr`;
`ModuleGraphIr::materialized_units` is the common source for namespace and
`import.meta` cells, dynamic-import dispatchers and
runtime-only collision checks. A namespace carries that typed mode rather than
a parallel deferred boolean and cannot be created for a source-only unit.

Dynamic components are discovered in full before evaluation-mode
classification, then components whose referrer does not materialize are
removed from the artifact registry. This preserves dynamic edges needed by the
fixed point without compiling an `import()` call site whose containing module
can never run. The precise invariants and regression shape live in
`docs/rust-rewrite/contracts/module-runtime-participation.md`.

### Root `this` binding domain

The assembled artifact is reparsed with the Script goal for one shared
lowering, but that implementation goal does not replace each source owner's
Environment Record semantics. `RootThisBinding` is derived from the original
goal and is required by every lowerer construction. `CurrentThisBinding` then
distinguishes that root binding from a real function activation. Canonical
`ModuleActivation` and `AsyncModuleActivation` owners carry module-root
`undefined` explicitly, including nested lexical arrows; root Script reads
retain the global-object operation. Ordinary and derived activations remain
dynamic, and arrows inside them inherit their actual receiver. Only
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
`docs/rust-rewrite/contracts/dynamic-component-authority.md`. Canonical target
owners are now evaluated lazily through their import jobs; complete namespace
exotic behavior remains a separate conformance obligation.

Dynamic-import targets are compiled into the same artifact, not separate Wasm
modules, and dispatch through the artifact's generated dynamic-import registry.
Instantiation creates environments and links cells without evaluating target
bodies. The registry starts a canonical owner's evaluation only when an import
executes, preserves namespace identity across fresh promises and reuses cached
normal or arbitrary abrupt evaluation results. Static dependencies and
top-level await complete through that lifecycle before ordinary import
fulfillment. Splitting a graph into several linked Wasm modules later remains
a backend change with no source-loading fallback.

`lila-ir` performs no IO. The host resolves and reads the finite transitive
closure up front (`lila_engine::load_module_graph`) and hands it to
`lila_ir::lower_module_graph` or `lila_ir::lower_script_graph` as
`ModuleGraphSources`. Dynamic-only malformed loads and dependency/link failures
are partitioned into phase-owned rejection rows, so valid shared closures
remain usable and the failures become observable only when their imports run.
Contradictory host rows and parse-goal misuse remain compilation errors.
Source-only requests preserve their phase through host loading and do not
expand target dependencies; dynamic JavaScript source requests reject natively.

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

The 2026-08-25 checkpoint removed the Boolean from dynamic-import call-site
rewriting and introduced a private two-variant dispatcher-reference domain.
Its focused ownership checks passed `4/4`, and both rewrite units passed `1/1`.
At 2026-09-29, the canonical Script import-job batch removes that alternate
Script-entry-export authority entirely: the eager wrapper, exported bindings,
alternate public rewriter, selector enum and dedicated structure target are
deleted. Module and Script calls now share `rewrite_dynamic_import_calls` and
the same exhaustive phase-to-dispatcher naming projection. Retained-AST source
record admission rejects private linker identifiers, including nested bindings,
before they can shadow rewritten imports; property and string data remain valid.
The refresh commands and pending central verification are recorded in
[the dispatcher contract](../docs/rust-rewrite/contracts/dynamic-import-dispatcher-reference-ownership.md).

The original 2026-08-25 ownership boundary was source-equivalent and made no
broader pinned Test262 claim. The 2026-09-29 Script job change has separate
semantic and runtime verification.

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

## Dry implementation: complete lexer-backed namespace name spelling

Module export readers and namespace aliases consume
the same Unicode character authority as the real lexer through `lila-front`.
The former alphabetic/alphanumeric approximation and separately listed
exceptions are removed, so legal combining marks, spacing marks, connector
punctuation and the lexer's Unicode 17 additions are no longer classified as
unspellable. Escapes remain decoded by the interner; generated readers preserve
exact code points and live cells without Unicode normalization.

`SourceName`/`MergedName`, anonymous-default minting, arbitrary export-name string
keys, reserved-word parser rules and existing module phase/identity behavior are
unchanged. Authored Front/IR/Engine regression sources cover these boundaries
and live Unicode exports plus namespace aliases. The later source-phase
correction retires ordinary source aliases and replaces their positive
expectations with rejection controls. No compilation, test
execution or pinned replay is claimed; T12 remains open. See [the contract
follow-up](../docs/rust-rewrite/contracts/module-binding-name-domains.md#13-shared-unicode-identifier-spelling-authority).
