# Grug Conformance Taxonomy

Every failure must fall into one bucket. No mystery pile.

## Failure Kinds
- `Parser`: source not parsed or grammar not recognized.
- `EarlyError`: parse okay, but static semantics and early errors wrong.
- `Lowering`: front-end semantics did not survive into spec IR or backend IR.
- `Runtime`: runtime semantics wrong after successful compilation.
- `WasmBackend`: Wasm emission or validation wrong.
- `HostHarness`: shell host shim or harness behavior wrong.
- `Unsupported`: feature not built yet, must burn down to zero.

## Done Means
- Full pinned Test262 run is green for chosen shell host profile.
- No permanent expected-fail list.
- No silent skip path.
- Every historical failure has owner, fix, or explicit temporary blocker written down.
