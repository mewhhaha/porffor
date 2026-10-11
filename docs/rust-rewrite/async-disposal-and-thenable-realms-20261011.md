# Async disposal and thenable job realms — 2026-10-11

The interrupted all-feature baseline at `b447e2006` completed 171 of 245
inventoried targets: 4,959 passes, three failures and two schematic doctest
ignores. All 871 Engine library tests and all 485 builtin integration tests
passed. The async integration target completed at 242 passes and three failures.
This is diagnostic evidence, with no whole-workspace verdict.

The three failing helpers run SpecExec before Wasm AOT. Boa held a mutable stack
borrow while retrieving a user disposal getter, awaited each nullish resource
before invoking earlier disposers, and constructed suppression errors through a
mutable global constructor. Disposal registration now captures a shared List
handle before user code; moving a stack transfers that exact List. Async
disposal records the required final Await, keeps synchronous Call failures
synchronous, and uses its saved method realm and intrinsic error constructor.
The sync fallback creates the specified intrinsic Promise, discards the sync
method's return value and rejects on a synchronous throw. Both stack families
check disposed state before callback callability.

The original realm fixture incorrectly required a local thenable's resolving
functions to use the foreign disposal method's realm. Six direct release-product
probes passed the original fixtures in sloppy and strict mode. Both probes of
the corrected realm fixture failed. These direct CLI diagnostics use a
30-second outer process limit; they are not native-test acceptance.

`NewPromiseResolveThenableJob` selects its realm with `GetFunctionRealm(then)`
and creates resolving functions while that job runs. The awaited Promise's
realm does not select those functions. Wasm now materializes them using an
explicit realm context: the constructor entry uses the called constructor's
realm, and the thenable job uses its running realm. The old helper that selected
the Promise record's realm is removed. The fixture expectation follows this
rule, with added local/foreign, Proxy, bound, revoked and self-resolution
controls. Self-resolution errors use the resolving function's realm. Proxy Call
and Construct helpers also receive an explicit execution Realm: the runner's
lexical Environment cannot override the selected job. This covers revoked
thenable and reaction callbacks and trap argument Array ownership. Promise
capabilities and disposal suppression retain their separate
method-realm rules.

The normative audit used ECMA-262 source revision
`5345883164f463e87f8b40aca4956157ecba8783`, including
[DisposeResources](https://tc39.es/ecma262/#sec-disposeresources),
[Await](https://tc39.es/ecma262/#await),
[NewPromiseResolveThenableJob](https://tc39.es/ecma262/#sec-newpromiseresolvethenablejob)
and [CreateBuiltinFunction](https://tc39.es/ecma262/#sec-createbuiltinfunction).
The complete upstream Test262 source update is a separate pending batch;
this repair does not selectively remove the old Atomics.pause tests.

All ten native tests in the affected async-disposal module pass with zero
failures or ignores in 46.595 watched seconds. The four fixture
helpers cover SpecExec and Wasm AOT in sloppy and strict scripts; the other six
controls exercise the compiled Wasm path. Tests retain their original 30,000-ms
observation limits and run on one CPU with one libtest thread. Joined
all-feature/all-target workspace types pass in 130 watched seconds, and all
eleven repository guards pass. The [JSON receipt](async-disposal-and-thenable-realms-20261011.json)
binds the frozen source, selected tests, executable, commands and transcript
hashes, and retains the preceding nine-pass/one-failure diagnosis.

Two linking attempts failed with SIGBUS while the disk had less than 500 MiB
free. Neither ran tests. Removing obsolete incremental generations and a retired
CLI reclaimed 2.57 GB while preserving current caches; the unchanged-source
retry built in 2.020 seconds and passed. This diagnoses local
build pressure without establishing a general compiler speedup.

Fresh complete workspace acceptance, pinned Test262 publication and task closure
remain pending; historical published counts are unchanged.
