# Persistent raw runtime artifacts

Heap programs contain a small program module P and attach a compiler-owned
runtime module R. Fresh Engine processes may reuse raw R through the existing
program-Wasm disk cache. R and P share its directory, storage budget, atomic
publication, access refresh and pruning. No new cache tier, environment flag,
native deserializer or deadline allowance is introduced.

`RuntimeArtifactCache` supplies storage and the compiler artifact identity. The
Engine provider uses the existing fingerprint over compiler inputs and the
running executable. The domain-separated runtime key also binds the R/P ABI
version, architecture and every admitted Intl custom section, including service
selection and physical component images. Intl admission runs before either the
memory or disk lookup; cached bytes cannot select a different provider.

The versioned entry contains the key, bounded compiler-owned pool dimensions,
raw Wasm and a SHA-256 digest over the complete entry. A hit requires exact
lengths and integrity, full Wasm type/body validation, the expected host import
namespace, indexed exports for every runtime function and global, the two
runtime data segments at their declared boundaries, and exact admitted Intl
sections. Function signatures and global types are reconstructed from validated
Wasm, never accepted as independent serialized layout vectors. Each newly
emitted P still checks its freshly collected compiler-owned pool against R's
boundary before compiling against it. Missing, stale, corrupt or incompatible
entries are removed and regenerated; storage errors leave emission available.

The digest detects corruption; it is not an authentication mechanism against a
writer controlling the user's cache directory. This is the same local storage
trust boundary as the existing program and native caches. Wasmtime's normal
module compilation/cache path and the Engine's Intl artifact admission remain
in force after raw R is loaded.

Native R and P share the existing bounded memory module cache. Both ordinary
retention bypass and agent execution avoid retaining either module. Live
execution may own both even when admitting P evicts R from the cache. There is
no permanent strong runtime-module map outside the configured entry and actual
compiled-image byte limits. Ancillary raw-data and Intl-kernel caches have their
existing separate ownership and are not described by those native-image limits.

The pinned Wasmtime cache package identifies published source from Cargo's
retained packaging metadata. Such builds share the upstream release namespace
across executable mtimes; genuine development checkouts retain Git revision and
executable-mtime invalidation. Native compiler compatibility checks and cache
budgets are unchanged. See the vendored package's
[`LILA-PATCH.md`](../../../vendor/wasmtime-internal-cache-47.0.0/LILA-PATCH.md)
for source provenance and the separate identity review required before changing
Wasmtime's native compiler or runtime source.

The cache removes repeated R emission after one successful producer. It does
not promise a five-second first run with an empty native cache, and P still
constructs its own compiler-owned pool before adding source data. Timing claims
must report the original worker deadline and actual cache state.

Verification controls (run serially through `scripts/limited_verification.py`):

```sh
cargo test --locked -p lila-aot-wasm --lib runtime_artifact::cache::tests -- --test-threads=1
cargo test --locked -p lila-engine --test runtime_cache -- runtime_artifact_cache:: --test-threads=1
cargo test --locked -p lila-engine --lib linked_runtime_native_modules_honor_both_bypass_retention_paths -- --test-threads=1
cargo test --locked -p lila-engine --lib memory_module_cache::tests -- --test-threads=1
```

The controls cover integrity and ABI rejection, replacement after corruption,
real minimal and selected custom Intl runtime round trips with an emission
closure that fails if called on a hit, independent Engine processes compiling
different P modules against identical cached R bytes, and both native retention
bypass paths. A native process control also copies the same executable bytes,
verifies distinct modification times, and checks that unchanged R hits while
different P misses and executes to its expected result. Source changes and
these controls require execution before any cache-performance or conformance
result is claimed. The AOT cache identity workflow runs both the standalone
vendored identity regressions and the Engine process target in a serial job
on the existing `lila-conformance` runner, using the 4096-MiB launcher.
