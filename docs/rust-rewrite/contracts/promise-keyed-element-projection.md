# Promise keyed-element projection ownership

Status: source-equivalent Wasm-AOT ownership invariant, focused-verified on
2026-08-28.

## Private projection authority

`builtins/promise/promise_keyed_element_projection.rs` privately owns
`PromiseKeyedElementProjection`, both semantic producers and the sole raw
consumer. `emit_promise_all_keyed_resolve_element` selects a fulfilled value;
`emit_promise_all_settled_keyed_element` carries the typed settlement direction
into a settlement-record projection. The parent and standard-builtin dispatcher
cannot name, import, re-export, construct or directly project the raw domain;
they can only call the unchanged `pub(crate)` semantic wrappers.

The private consumer matches the two-way projection exhaustively. Its
settlement-record arm separately matches `PromiseSettlement::{Fulfill, Reject}`
to the status and result-property pair, allocates one self-backed settlement
record, and then rejoins the common keyed-result publication and remaining-count
lifecycle. The raw consumer remains child-private, so a caller cannot combine a
settlement direction with the fulfilled-value route.

## Exact evidence

The pre-extraction four-line domain and 224-line producer/consumer lifecycle
retain SHA-256
`800bd5beb3809f1076d8ba44ad9ff1e1b4fbbe84a94a03cc48dc5ad40e013db2`
and
`a603dc1fc5222699b626849223d31816a5215428692e0a0034d756f1a09f812d`.
Their combined 228 selected lines retain SHA-256
`8933e0453be7da7baab292babfd052c663ffa6923cbdcb53715cd4d7bc9300df`.
The formatted 232-line child has SHA-256
`ddc8233e8b079fede9ea977e5cdda3cca449e79dae8a0943a42c910888651d24`
and reduces the concurrent Promise parent from 8,473 to 8,245 lines.

The recursive structure guard pins the private module, zero imports/re-exports,
six production authority mentions, exact wrapper choices, one owned consumer,
both exhaustive projection arms, typed settlement allocation and the absence of
the retired Boolean route. The include-only target passes `3/3`.

```sh
cargo test -p lila-aot-wasm --test promise_keyed_element_projection_structure --quiet
cargo test -p lila-engine tests::wasm_backend_promise_all_settled_keyed_uses_one_resolve_lookup_and_shared_guards -- --exact --test-threads=1
cargo test -p lila-cli --test cli functions::run_wasm_backend_distinguishes_all_promise_combinator_modes -- --exact --test-threads=1
```

After extraction, the engine and CLI commands each pass `1/1`. The shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks are green.
No semantic golden or broad suite was run for this source-equivalent move.

## Nonclaims

This source-equivalent move changes no projection, record property, keyed
enumeration, remaining-count, resolution, Realm or emitted instruction. It does
not add a Promise combinator mode, complete T14 or refresh conformance status.
