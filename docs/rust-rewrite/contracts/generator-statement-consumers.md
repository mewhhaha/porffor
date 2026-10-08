# Generator statement consumers

Ordinary generator plain identifier assignments use their actual retained
WriteOnly Reference owner. The existing iterator and async-generator linear
contexts keep their predecessor RHS-only assignment route when identifier
resolution is static and no With environment is active. Its explicit
`LinearOnly` arm stages the complete RHS, then performs the existing identifier
write. It cannot acquire the new ordinary reference record or admit a runtime
environment that the predecessor refused.

The real iterator IR regression retains two distinct call-argument resume cells
and performs the target write afterward. The strict/sloppy Wasm AOT fixture
requires one call after both resumes, real GC at the boundaries, and exact
identity of both arguments and the resulting assignment value.

Discarded object literal expressions now reach the same complete expression
source plan and staged property definition owner as value-consuming contexts.
The two statement catch-alls delegate to those actual existing owners. The
semantic discarded fixture observes key conversion, comma-value effects,
immediate spread Gets and later property order across GC and resumption.

These are source corrections with authored controls. Compilation, Wasm
validation, execution and broad conformance remain pending for the joined batch.
