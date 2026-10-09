# Async-function resume completion as a closed wire domain

## Current GC boundary — 2026-10-08

The current two-way policy is private, must-use, non-Copy `AsyncContinuationOwner`. Both `emit_load_async_continuation_resume` and `emit_async_continuation_await` borrow it. They project the actual typed GC Async/AsyncGenerator activation and strictly decode the accepted resume pair; the invocation frame owns the saved lexical Environment. Saved and active for-await iteration owners are consumed by reattachment and cleanup, with cleanup publishing the restored Environment before completion dispatch. The earlier four scalar-offset projections below describe the retired heap representation.

This source/guard repair does not establish a new runtime or conformance result.

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

Batch AB makes `ForAwaitActivationLayout` a must-use, capability-free owner of
the complete suspension policy. Its three offset projections borrow that one
owner. The two borrowed strict-decoder calls and four borrowed exhaustive
projections select the rejection Promise Realm and the ordinary-async versus
async-generator reaction sequence for the close and value suspension paths.
The former copied layout values and `is_async_generator` Boolean carrier are
gone, so those policies cannot drift onto independently copied authorities.
This Rust-only closure changes no emitted Wasm or runtime behavior. At the
Batch AB checkpoint, `cargo xc` is green, the dedicated structure target passes
`3/3`, and the exact ordinary-rejection, iterator-close and async-generator
rejection engine controls pass `3/3`. Test262 and semantic goldens were not
rerun for this capability-only closure.

## Durable evidence

The heap wire-domain test fixes the two words and their exhaustive throw
policy. A structural boundary test fixes the sole private offset owner, the
typed initializer and reaction stores, the ordinary-await decoder, both
`for-await-of` decoder sites, and the unknown-word trap.

Existing engine contracts exercise normal and rejected ordinary awaits plus
normal, rejected-next and iterator-close `for-await-of` paths. They remain
runtime verification for valid words; the structural contract guards the
illegal-state boundary that JavaScript cannot directly construct.

## Recorded verification

The exact `async_function_resume_completion` heap tests pass `2/2`. The engine
regressions for lexical-state `await`, rejected Array iteration and async
iterator-close validation each pass `1/1` on 2026-08-25. Three exact current-pin
Test262 leaves covering rejected `await`, thenables that throw and abrupt
`for-await-of` iterator-close lookup pass all `6/6` sloppy/strict Wasm-AOT
executions at vendored suite content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, with every failure and
non-success bucket at zero.

## Nonclaims

This invariant does not merge the distinct async-function and async-generator
resume domains. It does not type Promise lifecycle state, async-generator resume
kinds, async-generator execution/body state, or module/finalization jobs. It
does not make the Promise-job queue realm- or agent-owned, change unhandled
rejection reporting, establish complete suspended-body support, or close the
T14 Test262 gate.

## GC suspension return boundary — 2026-10-07

The GC body-entry ABI also distinguishes suspension from final falloff. A plain
async body returns a Normal completion with its committed, nonzero resume point
in the target word when it suspends. Normal with target zero means final
falloff. Resetting that target after registering an Await reaction prematurely
fulfilled the result Promise and marked the original activation completed;
queued resume jobs then correctly refused to reenter it.

`emit_return_async_suspension` consumes the actual typed body activation and
reads its committed `AsyncActivation.RESUME_POINT` into the returned completion.
Ordinary Await, module instantiation handoff, async resource disposal and both
iterator implementations' awaited next/close paths use this one return boundary.
The existing validated IR owns the nonzero suspension states. Async generators
keep their separate `BODY_STATUS` and execution-state protocol; a synchronous
generator is an explicit compiler error at this boundary. Registered reactions,
original lexical environments, pending abrupt completions and Promise settlement
remain owned by their existing implementations.

The original native mixed Array lifecycle fixture exposed the defect after the
main global-environment repair. Its execution returned normally with no expected
output. One diagnostic showed even `await 0` skipping its continuation while
the outer async Promise fulfilled. The shared source repair is written. The
`tasks-async-suspension2` checkpoint passes the whole-workspace type check,
immediate async return and async finalizer controls. Three controls remain red:
ordinary Await selects a main Atomics checkpoint with a missing branch condition,
iterator close fails method acquisition, and the unchanged mixed Array lifecycle
now throws instead of returning with empty output. The checkpoint completes all
five controls in 465 seconds with two passes, three failures and none ignored,
under one CPU and 4096 MiB. These failures have separate source owners and do not
establish complete suspension acceptance.

`tasks-symbol-progress1` then passes the current whole-workspace type check,
ordinary Await lexical-state/rejection control, awaited iterator closing and
the unchanged mixed Array lifecycle in both strict and sloppy modes. The joined
Atomics stack-result and actual Symbol identity repairs clear those three
failures. The separate waiter-progress and new Symbol-catalog controls remain
open at that checkpoint; no broad or pinned conformance claim follows.
