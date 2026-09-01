# Atomics wait outcome domain

Status: implemented as a source-equivalent Wasm-AOT invariant boundary.

## Closed outcome

`Atomics.wait` returns one of `"ok"`, `"not-equal"`, and `"timed-out"`.
`Atomics.waitAsync` uses the same strings for its synchronous result and for the
value that its asynchronous result Promise fulfills with. These result spellings
now cross the private, non-derived, non-copyable
`AtomicsWaitOutcome::{Ok, NotEqual, TimedOut}` domain.

One borrowed exhaustive projection owns the exact strings. It has all three
named arms and no wildcard, assertion, default, numeric code, or unreachable
fallback. The two result helpers accept only `AtomicsWaitOutcome`; neither can
receive an arbitrary string.

## Producers

The thirteen semantic producers retain their source and emission order:

| Emitter region | `Ok` | `NotEqual` | `TimedOut` |
| --- | ---: | ---: | ---: |
| `Atomics.notify` asynchronous-waiter settlement | 1 | 0 | 1 |
| immediate `Atomics.waitAsync` result | 0 | 1 | 1 |
| asynchronous timeout checkpoint | 2 | 0 | 2 |
| `Atomics.wait` result | 1 | 2 | 2 |
| Total | 4 | 3 | 6 |

Six producers write a string payload directly and seven enter one of the two
typed result helpers. The static string pool retains the existing
`"not-equal"`, `"timed-out"`, `"ok"` order, so the invariant migration does not
renumber later string payloads.

## Numeric boundaries

The outcome domain is not a wire protocol. Wasm `memory.atomic.wait32` and
`memory.atomic.wait64` still produce numeric status codes, and the `agent_call`
host boundary still exchanges numeric waiter identifiers, notification counts,
and poll/cancel statuses. Internal waiter state also remains numeric. Each
numeric boundary is decoded before the selected ECMAScript result is
materialized or a Promise is fulfilled; no host status is stored in
`AtomicsWaitOutcome`.

## Durable evidence

`atomics_wait_outcome_structure.rs` pins the exact domain and spelling
projection, lack of capabilities, string-pool order, both typed consumers, all
thirteen producers in their four emitter regions, the `4/3/6` variant census,
the six direct payload projections, the recursive source-wide census, and the
absence of raw outcome literals outside the projection and static string pool.
It separately pins the numeric Wasm and host-status decoding sequences.

The retained `wasm_atomics_wait_core.js`, `wasm_atomics_wait_async_core.js`, and
`wasm_atomics_wait_async_timeouts.js` fixtures remain the focused runtime
witnesses. They cover immediate not-equal and timed-out results, blocking wait
status decoding, Promise fulfillment with ok and timed-out, notification, and
finite timeout checkpoints.

At the 2026-08-27 focused checkpoint, the bounded structure target and the CLI
`atomics_wait` filter each pass `4/4`. The exact current-pin
`returns-result-object-value-is-string-not-equal.js`,
`returns-result-object-value-is-string-timed-out.js`,
`returns-result-object-value-is-promise-resolves-to-ok.js`, and
`returns-result-object-value-is-promise-resolves-to-timed-out.js` Test262
leaves each pass both sloppy and strict Wasm-AOT variants, for `8/8` aggregate
under `--jobs 1 --threads 1`. Every reported parser, early-error, lowering,
runtime, Wasm-backend, host-harness, unsupported, not-implemented, crash, and
bug bucket is zero.

## Scope

This boundary adds no fixture or Test262 rewrite and changes no host ABI word.
It does not claim multi-agent stress correctness, all waitAsync scheduling
semantics, full Atomics conformance, or T17 closure. Focused execution and broad
verification beyond the recorded witnesses remain deferred for the coordinated
batch checkpoint.
