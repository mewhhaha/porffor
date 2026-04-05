# Grug Architecture Invariants

## Sacred Rule
Porffor ship real JavaScript-to-Wasm compilation. Porffor does **not** ship a JavaScript interpreter or VM compiled to Wasm as the execution strategy.

## Product Path
1. Parse source as script or module.
2. Apply early errors and semantic classification.
3. Lower into spec-shaped IR with explicit completion, environment, object, and job semantics.
4. Lower into backend IR.
5. Emit and validate Wasm.

## Allowed Hidden Tooling
- Tiny interpreter for debug bring-up.
- Differential executor for IR vs Wasm checks.
- Minimized repro runner for conformance triage.

## Forbidden Shipping Shapes
- `build wasm` outputs generic evaluator blob.
- `run` executes source by piping it into interpreter compiled inside Wasm.
- Embedding API exposes “load source into shipped VM Wasm module” as normal mode.
