# Embedded runtime image metadata boundary — 2026-10-11

Engine's immutable runtime byte initializers occupied approximately 288 MB in
each normal/check compiler metadata image. Their values were also repeated in
Engine incremental compilation data. Static visibility did not remove that
cost. A one-MiB private experiment confirmed that a non-inline function in a
separate image crate keeps its initializer out of a downstream crate's metadata.

The first leaf-crate candidate reduced Engine metadata to 1.29 MB in the check
profile and 1.77 MB normally; nine focused native controls and joined types
passed. Its own large Rust initializer still consumed substantial metadata and
incremental cache space. Native/default links hit disk exhaustion before tests;
the failed receipts remain separate from executed test results.

`lila-runtime-image` now owns the three immutable Cargo-produced images as linked
data and exposes non-inline slice accessors. Its build script invokes the existing checked
raw-runtime producer with the same target, configuration, modes and bounded
Cranelift function cache. It performs fresh Wasmtime precompilation of the
current raw runtime; no native cache file replaces that producer.

The Cargo target Rust compiler emits a data-only object with three immutable
arrays and matching exact declarations. It emits no Rust metadata, incremental
cache or duplicate bitcode for those initializers. The target archiver packages
the object for Cargo's normal static linking, then the duplicate object is
retired. Ordinary Rust metadata contains only the extern array declarations and
accessors. Unsupported targets keep three empty slices and the existing ordinary
runtime path. A one-MiB private object experiment verifies exact linked-byte
parity and 2,520 bytes of accessor metadata; it establishes no product speedup.

Engine independently checks the exact source, target, configuration, modes,
package/native digests and raw-runtime key before deserializing into the caller's
Engine. The existing bounded R/P retention owner is unchanged. No module is
retained by the image crate and no external bytes or paths enter native admission.

Both build scripts use the same compiler fingerprint, including the new image
crate. Image production watches source and cache settings; only Engine's light
identity producer watches Git revision metadata. A commit that changes no
compiler inputs therefore need not regenerate the runtime image.

The final data-object batch passes joined all-feature/all-target workspace types,
all twelve repository guards and eleven focused native controls, with no failures
or ignores. Three controls verify complete raw-R parity and manifest admission;
five verify compiler-source invalidation, including all four new crate files.
The fresh-program process control passes in both graphs. The default graph also
proves the oracle remains unlinked; its actual Cargo feature lists are empty.
Native tests retain one CPU, original worker counts and original deadlines.

| Artifact | Before (bytes) | Final (bytes) |
| --- | ---: | ---: |
| Engine check metadata | 288,071,282 | 1,288,013 |
| Engine normal metadata | 288,563,052 | 1,766,601 |
| Engine normal archive | 594,863,518 | 21,287,870 |
| Image check metadata | — | 3,804 |
| Image normal metadata | — | 3,826 |

The image archive still owns the real 286,778,811 bytes of immutable data; those
bytes have moved out of Rust metadata rather than disappeared from the program.
Its three measured incremental graphs occupy 43,210–57,794 bytes each. Engine's
normal incremental graph occupies 65,640,457 bytes, versus 922,671,129 previously;
these are artifact-size observations, not a controlled general build-speed claim.
All three current producer profiles publish byte-identical packages, native images,
manifests and linked archives. Their duplicate data objects are retired.

The first final-candidate default build exhausted disk while creating the unit
incremental directory, before any test ran. After retiring unused incremental
graphs and older generations, both default native controls pass. The failed
build receipt is retained alongside the passing retry. Earlier static-candidate
link failures are also preserved. No failed build is counted as executed tests.

The [machine-readable receipt](runtime-image-metadata-boundary-20261011.json)
records source bindings, executable/log digests, actual counts and measurements.
Git-only image reuse, complete workspace/default sweeps, pinned Test262 and task
completion remain pending. macOS/Windows linking and runtime execution still
require their own native evidence.
