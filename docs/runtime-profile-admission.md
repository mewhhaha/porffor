# Native source runtime/profile admission

The three native source callbacks share one `ValidatedRuntimeArtifact` owned by
`NativeSourceCompilerContext`. Its private fields bind the exact immutable R Arc
to a selected physical Intl image. Construction checks the recorded current
layout admission and parses the actual R bytes once with the existing strict
Intl profile/frame validator. No per-callback image scan remains in the three
proof-consuming emitter APIs.

Layout authority comes from the current compiler's R emission or from a cache
or build-package envelope whose encoded version was checked before full payload
and Wasm validation. The version token is passed into payload admission; a raw
payload cannot acquire the current version by default. Synthetic unit fixtures
are explicitly unadmitted until the cache decoder admits their checked envelope.

The original public raw emission APIs remain checked entrypoints for standalone
callers and negative tests. They preserve program diagnostics and direct-eval
source-witness validation before runtime admission, then share the same emission
body as the proof APIs. The proof APIs accept neither a replacement profile nor
a separate runtime owner. Artifacts retain the exact R Arc. They preserve direct
eval's complete caller/private-floor/source/root witness and Wasm GC nominal-type
linking. This change adds no Wasm GC fields or layout-version bump.

Context construction is fallible and checks the original execution control both
before and after profile admission. All callbacks retain that execution Arc and
its armed deadline; no fresh compilation budget or ambient program is added.
Admission errors remain Engine errors rather than JavaScript SyntaxError results.

The focused Rust controls in this component are authored but have not been
compiled or executed. Source-only checks cover strict patch composition, exact
predecessor bytes, formatting, retained historical selectors, opaque ownership,
all three actual fast consumers, checked raw entrypoints, and unchanged roots.
No measured performance gain is claimed until the coherent native replay runs.
