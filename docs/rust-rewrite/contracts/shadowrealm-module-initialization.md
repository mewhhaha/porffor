# Realm module initialization and import jobs

Source batch: 2026-10-07. The workspace type check passes all features and test
targets. Focused module graph, embedded loading, prepared-source cache and host
request tests pass. Native import execution passes the per-Realm module-cache
control in `tasks-shadow-native2`: root and separate ShadowRealms retain distinct
live cells, repeated imports evaluate once, and callable exports get fresh
wrappers. Remaining native cohorts and pinned conformance are still required
before closing acceptance.

`lila-ir/src/modules/synchronous_source.rs` puts module activation factories and
private import dispatchers inside one compiler-owned initializer. Trusted AST
positions distinguish its allocation body from the root initialization request.
`AnalysisBuilder` plans named environments for reusable module graphs: a module
in a new Realm cannot capture the entry Script's global slot indices. Private
dispatcher bindings retain their canonical cells without named eval exposure.

`lila-aot-wasm/src/planning/module_graph.rs` validates the initializer and import
dispatcher together. The `ModuleInitialize` runtime helper materializes that
compiled initializer with the selected Realm's global environment and its own
template-source execution. Startup and `ShadowRealm.prototype.importValue` call
the same helper. `RealmRecord.MODULES` caches fresh records, environments,
namespace cells, activations, evaluation promises and rejection state per Realm;
repeat imports never replace another Realm's registry or the current registry.

Programs without a validated module graph reserve each private module helper
as an unreachable three-byte body, including initialization. Their emitted
functions contain no call or function-reference edge to those slots. Both the
initialization facade and helper emitter require the actual graph plan before
emitting a call or body; absence cannot fabricate a normal initialization
completion. Prepared scripts in a graphless ShadowRealm still use their own
installed script hook and do not acquire a module graph.

`ModuleGraphSources.realm_requests` is a separate request domain. Loaded rows
name a real Module Record; rejected rows retain the host or parse/link failure.
`EmbeddedModuleReferrer::Realm` has its own validated and fingerprinted role,
including differential replay serialization. It never falls back to Script,
Module or Unlocated resolution rows. Ambient requests use
`HostModuleLoader::resolve(None, request)`, whose filesystem implementation uses
the entry's host base, independent of the module containing the native call.

Literal `.importValue` and `['importValue']` sites contribute potential requests
without claiming intrinsic identity or changing the actual call. The existing
finite-source/call analysis supplies alias and prepared-source candidates.
`LoweringSession` retains successful and failed prepared parses;
`lila-engine/src/prepared_source_catalog.rs` extends the host graph to a request
fixed point before dependency-cache identity is calculated. Existing loaded
sources are reused. Every added request acquires a Loaded or Rejected outcome,
including record-kind mismatch, so a failure cannot keep discovery cycling.

Admission partitions Realm-only parse and link failures into request rejections
while retaining static entry failures. Successful Realm targets participate in
module materialization and canonical eager namespace planning. The bridge in
`modules/realm_import.rs` obtains that namespace and starts the compiled private
async dispatcher. Its first suspension preserves the load-job boundary; its
ModuleEvaluate await uses the existing intrinsic module-Promise reaction owner,
without reading mutable Promise methods or constructors. The dispatcher returns
undefined after evaluation: export `then` is not assimilated. The native export
continuation captures the actual namespace separately.

Generated error constructor identifiers are trusted AST-pointer operations,
lowered to canonical intrinsic FunctionValues. Mutable globals or source
bindings cannot supply their constructors. The caller-Realm outer native
continuation still applies the proposal's error-copying boundary.

Authored controls cover source/Realm request separation, reusable initializer
reachability, retained parse/link/host rejection, cached host loading, fingerprint
separation, actual multi-Realm module state, top-level await, cycles, original
load failures and nested prepared-source discovery. Native controls also poison
the target Realm's global error constructors, export a callable `then`, and call
a custom `importValue` method whose candidate module has a syntax error. These
exercise intrinsic ownership, namespace handling and speculative discovery
without substituting the user's callee. The first three commands below pass in
the joined focused/IR checkpoints. The native module-cache control also passes;
the complete native import target remains pending.
Run these sequentially under the resource launcher, reusing build artifacts:

```sh
python3 scripts/limited_verification.py -- cargo test --locked -p lila-ir --test realm_module_catalog --test module_instantiation --test module_import_jobs --test complete_module_catalog
python3 scripts/limited_verification.py -- cargo test --locked -p lila-runtime --test embedded_module_graph
python3 scripts/limited_verification.py -- cargo test --locked -p lila-engine --lib discovered_realm_requests_keep_host_origin_and_closed_cached_outcomes
python3 scripts/limited_verification.py -- cargo test --locked -p lila-engine --test aot_realm_modules -- aot_shadow_realm_import_value::
```

Then run the affected module/Promise/prepared-source/native Realm regressions and
the original pinned ShadowRealm directory as part of the joined verification
ladder. These commands and source invariants do not refresh historical counts.
