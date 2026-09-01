# Wasm execution mode

The engine's Wasm path has two output contracts which must remain paired from
entry point through host-state construction and final result observation.

## Closed domain

`WasmExecutionMode` has exactly two private rows:

- `Legacy` delegates printed output and returns the historical rendered
  `RunOutcome`;
- `Structured` captures printed output and returns an
  `ObservedRunOutcome` with a typed ECMAScript completion.

The domain derives no cloning, copying, debugging, equality, ordering, hashing
or default capability. The three internal execution seams borrow the mode from
the entry point through host-state construction and final result projection.
Both semantic observations are exhaustive matches.

## Durable regressions

The structure guard pins the private two-row declaration, exact source census,
five entry-point producers, three borrowed execution seams, output-event
ownership table, result table, and the
required construction-before-result order. Four ordinary run paths select
`Legacy`; only the observation entry point selects `Structured`.

The existing `wasm_output_capture_is_enabled_only_for_structured_execution`
unit witnesses both output policies. Structured completion behavior is covered
through the public `observe_script` and `observe_module` tests rather than
through an internal equality assertion on the mode.

The dedicated structure target passes `3/3`. The exact output-ownership,
structured normal-completion and structured throw witnesses each pass `1/1`.
Independent dry review found the revised borrowed domain and swap-resistant
result guard clean. `cargo xc`, the full formatting and diff checks, and
repository boundary checks are green.

## Nonclaims

This capability closure changes no Wasm compilation, instantiation, execution,
completion rendering, output capture, public API or backend routing behavior.
It does not close the remaining shared completion-operation or Test262 work.
