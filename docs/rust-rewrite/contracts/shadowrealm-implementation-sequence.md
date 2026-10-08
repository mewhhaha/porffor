# ShadowRealm source integration

Source batch: 2026-10-07. The constructor, finite `evaluate`, wrapped callable,
`importValue` and native continuations now have real IR/catalog, GC and Wasm
owners. Per-Realm module initialization and Realm-origin requests are written.
A compilation-local lowering session retains parsed finite sources while host
module discovery closes their requests, including aliases and nested Function
bodies. The accepted IR and complete graph precede dependency-cache identity;
the Wasm emitter consumes that same IR.
The all-feature/all-target workspace type check passes in `tasks-shadow-joined3`
under one CPU, 4096 MiB and zero swap. Its repair successor also passes the
workspace check. Affected IR regressions now pass after live-property and
module-catalog assertions were updated and ArraySetLength effects were retained.
`tasks-shadow-native2` passes four native gate functions: construction/subclass
branding and isolated globals, fresh evaluation lexical/variable lifetimes,
wrapped callable identity/execution, and isolated module caches. The first three
pass strict and sloppy modes; the checkpoint finishes in 1,050 seconds.
The remaining native cohorts and pinned results remain pending. The historical 124 failures are not a
refreshed result, and T24 acceptance remains open.

The checked sources are the repository's pinned
`test262/vendor/test262/test/built-ins/ShadowRealm` fixtures and the
[proposal algorithms](https://tc39.es/proposal-shadowrealm/). The latter is a
live proposal, not a separately pinned local specification. Pinned fixture
semantics remain explicit below; changes to that proposal require review.

## Consumed native and compiler owners

`builtins/shadow_realm/mod.rs` validates the receiver before source conversion,
selects NewTarget's prototype before allocating the inner Realm, and consumes
the [finite evaluate contract](shadowrealm-finite-evaluate.md). The new
`ShadowRealmObject` has the ordinary object header and one strong Realm edge.
The common constructor installer publishes the complete prototype family in
every Realm; bootstrap planning includes the bodies needed by fresh globals.

`builtins/shadow_realm/wrapping.rs` uses real nonconstructible native closures
with a closed strong target capture. Shared `CopyNameAndLength` retains the
observable descriptor/Get order; bind preserves its original abrupt completion,
while the ShadowRealm boundary replaces it. Native Call wraps arguments in
order, then the receiver, and restores the caller before wrapping its result.

`builtins/shadow_realm/import_value.rs` retains synchronous original specifier
conversion failures and uncoerced String export names. Its module bridge owns
the selected Realm's registry. The inner job signals evaluation completion
without returning a namespace through Promise resolution, so an exported
`then` cannot redirect that internal handoff. The export reaction captures the
canonical namespace and String name; rejection copying reads no user property.
The shared `emit_perform_promise_then` also serves `Promise.prototype.then` and
attaches intrinsic reactions without reading mutable Promise properties.

Authored native targets are `aot_shadow_realm_construction`,
`aot_shadow_realm_prepared_evaluate`, `aot_shadow_realm_wrapped_callables` and
`aot_shadow_realm_import_value`. They cover identity, lifecycle, borrowed
Realms, metadata order, abrupt cutoffs, real modules and asynchronous completion.
Their source and formatting checks are not runtime evidence.

## Production dependency map

Paths below are under `crates/`.

| Stage | Consumed owner |
| --- | --- |
| Realm-origin loading | `lila-engine/src/module_loader.rs` retains distinct Realm request resolutions and deferred failures; explicit embedded catalogs carry `EmbeddedModuleReferrer::Realm`. |
| Prepared-source closure | `lila-ir/src/prepared_source_cache.rs` retains exact grammar/context parse products; `lila-engine/src/prepared_source_catalog.rs` closes host requests before artifact identity and retains final IR. |
| Selected-Realm initialization | `lila-ir/src/modules/synchronous_definition.rs` owns the private initializer; `lila-aot-wasm/src/modules/initialization.rs` invokes it against the selected global environment. Root startup and ShadowRealm share this entry. |
| Native publication | `lila-ir/src/builtins/catalog.rs`, shared names and shape/effect facts drive `lila-aot-wasm/src/intrinsics/shadow_realm.rs`, Realm bootstrap, planning and exhaustive native dispatch. |
| Callable boundary | `lila-aot-wasm/src/gc_types/{layouts,value}.rs` owns closed captures; `builtins/shadow_realm/wrapping.rs` consumes the target, actual native Call and destination Realm. |
| Evaluation | `lila-ir/src/lowering/prepared_script.rs` and `lila-aot-wasm/src/prepared_script.rs` retain the distinct finite evaluation kind and separate syntax/execution outcomes. |
| Import jobs | `lila-ir/src/modules/realm_request.rs` emits the private async dispatcher; `lila-aot-wasm/src/modules/realm_import.rs` consumes canonical namespace and evaluation owners. Trusted module awaits bypass public Promise properties, and native continuations attach through shared intrinsic reactions. |

The [module initialization contract](shadowrealm-module-initialization.md)
records graph, closure and cache ownership. The branded ShadowRealm object
retains its actual ordinary header; bounded completion snapshots continue to
reject its unsupported internal state explicitly rather than flattening it.

## Observable requirements retained by the implementation

The pinned `prototype/evaluate/throws-when-argument-is-not-a-string.js` requires
String input without coercion. `throws-error-from-ctor-realm.js` and
`wrapped-function-throws-typeerror-from-caller-realm.js` require errors from the
active method or wrapper's defining Realm, including borrowed methods.
`no-conditional-strict-mode.js` requires independent source strictness.

`WrappedFunction/{name,length}.js` requires observable own-length lookup before
name lookup, correct numeric length normalization and configurable metadata.
The evaluate wrapper fixtures require a fresh identity at each crossing,
primitive values unchanged, callable object arguments/results wrapped, and
ordinary objects rejected. The native wrapper must preserve argument order,
wrap the this value after arguments, restore the caller context, and replace
target abrupt completions without invoking user code on the thrown value.

`prototype/importValue/specifier-tostring.js` retains synchronous original
specifier coercion failures. `throws-if-exportname-not-string.js` forbids
coercing the export name. The import fixtures require actual module execution,
missing-export rejection and caller-Realm TypeErrors for module syntax and
evaluation failures. An unresolved module cannot be converted into a fulfilled
placeholder, nor can the primary Realm's namespace supply the result.

## Joined acceptance sequence

1. Review the Realm request catalog and reusable module initialization entry.
   Both existing root module execution and a second isolated Realm must consume
   the same owner. Retain controls for repeated imports in one Realm, independent
   state across Realms, a Script and Module referrer, cycles, top-level await,
   missing modules, and original parse/link failures. Filesystem and embedded
   catalogs must agree about the resolved request while retaining their distinct
   loading policies.
2. Review complete ShadowRealm native state, wrapped-value/call algorithms and
   prepared evaluate admission together. Retain finite Engine controls for
   repeated lexical declarations, strict/sloppy variable lifetime, nested
   Realms, borrowed methods, callable Proxies, metadata getter order/abrupt
   exits, primitives including Symbol/BigInt, original source SyntaxError,
   wrapper TypeError, and unmatched dynamic source. Global publication and the
   actual native family are joined in this source batch.
3. Review importValue against the same module and Promise owners and the
   full catalog/prototype family. Retain controls for receiver-before-coercion
   ordering, original coercion throws, uncoerced export names, fresh promises,
   callable exports, rejected ordinary-object exports, missing exports,
   module throws, cycles, suspended evaluation and cross-Realm prototype/error
   identity. Mutating Promise globals and methods must not redirect internal
   reactions.
4. At the joined capped checkpoint, compile once, run those semantic cohorts,
   then the original pinned ShadowRealm directory and the affected module,
   Promise, dynamic-source, Function/Proxy and Realm regressions sequentially.
   Refresh original-source evidence before making any conformance claim.

The deferred checkpoint must verify compiler/host integration and observable
semantics. No part of this source batch enables weak reachability;
T05/T21 retain their separately typed runtime capability blocker.
