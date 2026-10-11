# Weak collector foundation — 2026-10-11

The pinned Wasmtime copying collector now supports Store-scoped weak targets
and owner/key-gated ephemerons. Ten forced-GC controls and one cancellation
control pass, including thirteen cancellation boundaries. Eleven native image
controls also pass across all-feature and default builds: **22 passes, zero
failures, zero ignores**. Joined all-feature/all-target workspace types pass.
The [receipt](weak-collector-foundation-20261011.json) records exact inventories,
commands, source snapshots, executable digests and logs.

The collector traces conditional values to a fixed point, updates or clears weak
edges before ExternRef sweeping, and finishes every phase if collection is
cancelled after the space flip. Controls cover reverse chains, scans across
increment boundaries, dead owners with live keys, value/key cycles, moving
identity, stale and foreign handles, kept roots and null conditional values.
The [collector contract](contracts/collector-weak-edges.md) defines ownership
and explicit deregistration requirements.

The runtime image archive requests `static:-bundle`, reducing the leaf Rust
archive from about 287 MB to **10,718 bytes** while final executables still link
and admit the immutable images. Three producers have identical digests for the
41,071,134-byte package, 246,324,784-byte native image, manifest, native archive
and bindings. All-feature and default fresh-program execution, admission,
reused stencils, cache fingerprinting and oracle-unlinked behavior pass.

Failed builds are retained separately. Two exhausted the cloud disk; derived
obsolete caches were pruned before unchanged-source retries. The standalone
collector test exposed three upstream test feature guards that did not match
their required methods. Correcting those guards allowed the cancellation
control to compile with the product's narrow features plus `wat`; this is not
a full upstream Wasmtime test run. An intermediate expanded-feature build also
failed and remains recorded.

JavaScript's weak capability remains unavailable until WeakMap, WeakSet,
WeakRef, FinalizationRegistry, their typed host ABI and real job integration are
complete. Complete workspace, pinned-suite, task and independent-host acceptance
remain open. Historical suite counts are unchanged by this focused checkpoint.
