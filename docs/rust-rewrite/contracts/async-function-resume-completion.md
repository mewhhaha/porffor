# Async-function resume completion as a closed wire domain

## Specification boundary

ECMAScript `Await` resumes an async execution context with exactly one of two
completion shapes:

- a normal completion containing the fulfilled value;
- a throw completion containing the rejection reason.

The Wasm-AOT async-function activation stores that choice in one word. Its
stable encoding is:

| resume completion | wire word |
|---|---:|
| `Normal` | 0 |
| `Throw` | 1 |

These words describe the completion with which evaluation resumes. They are
not Promise lifecycle states and they are not the five-way async-generator
resume-kind domain.

## The bug class

The activation offset and both words were previously crate-visible integer
constants. Producers wrote them through the general heap store helper, while
consumers only tested whether the loaded word equalled the rejection word.
Consequently, a new producer could write an arbitrary integer and compile, and
an invalid runtime word was silently interpreted as a normal completion.

This is a record-integrity defect rather than a known valid-program failure.
The current producers write only 0 and 1, but a future emitter omission or a
linear-memory layout defect must fail closed instead of changing a throw into a
fulfilment.

## Producer invariant

`AsyncFunctionResumeCompletion::{Normal, Throw}` is the only source-level
domain for this field. The activation offset and numeric words remain private
to the heap boundary. The sole store operation accepts the closed Rust type,
so each producer must select a completion explicitly:

1. activation initialization selects `Normal` as the valid dormant value;
2. the async-function Promise reaction continuation selects `Normal` for its
   fulfilment arm and `Throw` for its rejection arm.

The enum, ordered set and stable words come from one macro row. Its
`is_throw()` policy is an exhaustive match, so adding a variant fails to
compile until its completion meaning is defined.

## Consumer invariant

The sole load operation reads the private field and decodes the closed domain
once. A known word becomes one normalized `is_throw` boolean. An unknown word
emits Wasm `unreachable`; it cannot fall through as `Normal`.

Ordinary async `await` consumes only that normalized boolean. The shared
`for-await-of` emitter has a closed activation-layout choice:

- `AsyncFunction` uses the strict async-function decoder;
- `AsyncGenerator` retains its separate resume-kind layout and policy.

Both the value-resume and iterator-close-resume paths normalize before any
completion, iterator-result validation or environment-unwind decision. No
ordinary async consumer compares the raw word.

## Durable evidence

The heap wire-domain test fixes the two words and their exhaustive throw
policy. A structural boundary test fixes the sole private offset owner, the
typed initializer and reaction stores, the ordinary-await decoder, both
`for-await-of` decoder sites, and the unknown-word trap.

Existing engine contracts exercise normal and rejected ordinary awaits plus
normal, rejected-next and iterator-close `for-await-of` paths. They remain
runtime verification for valid words; the structural contract guards the
illegal-state boundary that JavaScript cannot directly construct.

## Nonclaims

This invariant does not type Promise lifecycle state, async-generator resume
kinds, async-generator execution/body state, or module/finalization jobs. It
does not make the Promise-job queue realm- or agent-owned, change unhandled
rejection reporting, establish complete suspended-body support, or close the
T14 Test262 gate.
