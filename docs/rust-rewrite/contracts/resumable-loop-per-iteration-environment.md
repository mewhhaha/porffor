# Resumable `for-of` per-iteration environments

## Scope

This contract owns a captured lexical binding in synchronous `for-of` when a
sequence of direct body awaits splits an iteration across plain-async
invocations. Each entered iteration needs a fresh Environment Record, and the
active record must survive until that iteration resumes.

The loop now performs the synchronous iterator protocol. It does not lower an
Array to length and index reads, and it does not infer the yielded value from
the source kind.

## Closed IR ownership

`StatementIr::AsyncFunctionForOfIterator` requires one
`AsyncFunctionForOfIteratorPlanIr`. Private fields couple:

- the constructor-derived IteratorValue storage and suspension-owned
  `IteratorRecordIr`;
- the complete for-in/of head environment;
- `ResumableLoopIterationEnvironmentIr::{StorageOnly, FreshPerIteration}`;
- the statements before the first `await`, that await, and its continuation; and
- entry, resume, and exit states.

The crate-private constructor requires an initial `AsyncAwait` whose suspend
state equals entry. Every direct await must resume at its suspend state plus
one, and the next await must suspend at that state. Eager statements may appear
between awaits; nested suspension is rejected. The plan retains the final
resume state and derives exit as `resume + 1` with checked arithmetic. Its
closed head input derives one of `Activation`, `IterationEnvironment`, or
`EntryLocal` storage. A lexical pattern additionally proves exact iteration and
TDZ name sets and a matching
BindingInitialization prefix. When capture analysis supplied an iteration
environment, construction clones that analyzed layout into
`FreshPerIteration`. A captured head TDZ on the older storage-only single-name
form remains a distinct rejection.

This prevents three false states: a captured binding cannot silently select
storage-only, the iterator roles cannot be transposed raw strings, and a body
split cannot disagree with the resume-state order while still producing IR.

The direct await sequence validator is also the async-generator dispatch
preflight's authority for resumable await loops. Dispatch compares the final
validated continuation with the loop's resume state; it does not equate that
state with the first await's continuation. A suspending prelude, nested
suspension region, discontinuous state sequence, or unmatched exit remains a
rejection. The separate single-yield loop checks retain their existing scope.

## Runtime lifecycle

The backend follows this order for `FreshPerIteration`:

1. On entry, it evaluates the iterable in the head TDZ scope, performs
   `GetIterator`, reads `next`, and stores the typed Iterator Record in the
   plain-async activation.
2. It reloads that record for each step. A `next`, `done`, or `value` error
   propagates without `IteratorClose`.
3. After a non-done result, it creates one fresh iteration record, chains it to
   the head's parent environment, initializes the loop binding, and publishes
   the active environment to the activation.
4. It runs the before-await statements and suspends.
5. On each resume, it reattaches the same iteration record and runs the next
   continuation segment. A later await suspends again with that record retained.
   Completing the body leaves the record and publishes its parent.
6. Normal completion resets the plan to its entry state for the next iterator
   step. Natural exhaustion reaches exit without reading `return`.
7. A body Throw or Return leaves the iteration environment before
   `IteratorClose`. Close preserves an existing Throw, while a close error
   replaces Return.

Closures keep references to their iteration records after loop execution has
restored the parent. The next entered iteration allocates a different record.

Uncaptured body bindings remain in the resumable activation. If their backend
storage is first registered after entering an iteration or nested lexical
environment, its slot address must include the current distance to that
activation. Lexical and dynamic binding allocation share
`activation_owned_binding_storage`; they must not use an activation slot index
with zero hops from a child record. Captured bindings already registered by
the lexical environment retain that record's own slots.

## Observable witness

`crates/lila-cli/tests/fixtures/wasm_async_for_of_closure_capture.js` retains
six closures and invokes them only after the loop. The required values are
`1,2,3,4,5,6`; one shared activation cell would produce
`6,6,6,6,6,6`.

The protocol, close, and protocol-error fixtures named in
`synchronous-array-for-of-iterator-protocol.md` separately prove that this
environment lifecycle is nested inside one persisted Iterator Record rather
than a restarted Array walk.

## Verification and boundary

The six-closure witness now passes through the Iterator Record implementation.
The five focused structure targets pass `19/19`, the IR `for_of` target passes
`18/18`, and the four exact CLI oracles, including this witness, pass `4/4`.
The two pinned `Array.fromAsync` leaves pass `4/4` Wasm-AOT executions with
every failure and non-success bucket at zero.

At this checkpoint, the admitted form was a plain async function with one
direct body `await` and a simple single-name declaration or bare identifier
assignment head. Later checkpoints admit non-suspending member References,
assignment patterns, `var` binding patterns, and `let`/`const` binding
patterns. The lexical-pattern path materializes every BoundName in the fresh
record, including uncaptured names read directly after resume, and admits its
pattern-head TDZ because the fresh record is complete. The current plan also
admits sequential direct body awaits. Direct `break`/`continue`, resource
patterns, suspension in the iterable or pattern,
nonlinear body suspension, async generators, and `for await` remain outside
the plan. See
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](./plain-async-synchronous-for-of-lexical-pattern-heads.md).
