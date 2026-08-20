# Execution backend routing is exhaustive

## Boundary

`ExecutionBackend` is the closed selector shared by the engine's public run and
observation APIs. `WasmAot` is the product backend. `SpecExec` is the explicit,
feature-gated differential oracle; it is never an implicit fallback.

Four engine dispatchers consume that selector:

- `Engine::run_script`;
- `Engine::run_module`;
- `Engine::observe_source`, which owns both public observation entry points;
  and
- `Engine::run_compiled_unit`.

Each dispatcher must exhaustively match `ExecutionBackend`. The `SpecExec` arm
calls the feature-gated oracle helper, while the `WasmAot` arm calls the
corresponding Wasm execution helper. An equality check followed by an implicit
`else` is not equivalent: after a future backend is added, that shape silently
routes the new variant through whichever backend owns the `else` branch.

There is no wildcard or `unreachable!` escape. Adding a backend therefore makes
every execution entry point fail to compile until its routing policy is stated
explicitly.

## Durable evidence

The four dispatchers use ordinary exhaustive Rust matches, so the compiler owns
variant coverage. A bounded structural regression pins the four match sites,
their two semantic arms, and the absence of implicit conditionals, wildcards,
or unreachable catch-alls. The existing behavioral regressions still prove
that default execution reports `WasmAot`, a product build rejects explicit
`SpecExec`, and a feature-enabled developer build can invoke the oracle.

## Nonclaims

This invariant does not add or remove an execution backend, alter the default,
change existing two-variant behavior or evaluation order, enable the oracle in
product builds, change CLI flags or publication policy, modify emitted Wasm, or
change conformance counts. Dependency quarantine and artifact inspection remain
the separate T27 gates.
