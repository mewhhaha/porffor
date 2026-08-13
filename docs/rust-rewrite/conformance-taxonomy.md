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

## Artifact Contract

Failure kind, outcome, and origin are closed domains in snapshots and generated
backlogs, including classification-count map keys. An unknown wire spelling is
invalid evidence; it must not be coerced to `Runtime`, `Bug`, or `unknown`, and
it must not disappear as an ignored count key. The explicit `unknown` origin is
a known taxonomy member and is distinct from an unrecognized wire spelling.

Snapshot version 4 predates outcomes, so read-only migration derives missing
failure outcomes and outcome counts from that version's recorded evidence.
Versions 5 and 6 require every failure to carry a recognized outcome and every
snapshot or aggregate entry to carry its outcome-count map. This compatibility
rule does not make legacy evidence publishable as current status.

## Done Means

- Full pinned Test262 run is green for chosen shell host profile.
- No permanent expected-fail list.
- No silent skip path.
- Every historical failure has owner, fix, or explicit temporary blocker written down.
