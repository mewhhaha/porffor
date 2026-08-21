# Plain async function classic-for `await using`

## Evidence boundary

At committed Lila head `bca90f2ff9`, the existing Wasm-AOT binary reports
`Runtime/NotImplemented: unsupported in lila wasm-aot first slice: await using
declaration` for both strictness executions of each of these Test262 files at
revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`:

- `language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-forstatement.js`
- `language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-forstatement.js`
- `language/statements/await-using/initializer-Symbol.asyncDispose-called-if-subsequent-initializer-throws-in-forstatement-head.js`
- `language/statements/await-using/initializer-Symbol.dispose-called-if-subsequent-initializer-throws-in-forstatement-head.js`

That is an exact physical cohort of four files and eight executions, freshly
measured `0/8`. There is no rewrite or known-failure mask for these files.

## Normative lifecycle

This batch admits only a classic `for` whose lexical initializer is a non-empty
list of `await using BindingIdentifier = Initializer` declarations inside a
plain async function. The direct `StatementIr::For` remains the label and loop
control target.

1. Enter the classic-for lexical environment and create every head binding as
   an immutable, uninitialized binding before evaluating any initializer.
2. Evaluate resources in declaration order. For each initializer, perform the
   async-dispose-first, sync-dispose-fallback protocol, register the validated
   disposer, and only then initialize that declarator's binding. If a later
   initializer or protocol lookup throws, only already registered resources are
   disposed.
3. One activation-backed async DisposeCapability remains live across the test,
   every body execution, and every update. A local `continue` begins the update
   and next test without disposing that capability.
4. Test-false, a local `break`, `return`, `throw`, and abrupt completion from
   initialization, test, body, or update all leave through the same finalizer.
   Resources are disposed exactly once in reverse registration order and
   asynchronous disposal settles before the saved completion continues.
5. The finalizer owns a strictly ordered entry, disposal, resume, and exit state
   chain. Its activation binding is minted by the suspension-owned binding
   allocator and cannot be substituted with an arbitrary string.
6. `await using` head bindings are immutable and belong to the loop's lexical
   environment. Unlike `let`, they have no per-iteration replacement slots.

## Closed IR contract

`ForInitIr` has an exhaustive `AsyncDisposable(AsyncDisposableForInitIr)` case.
`AsyncDisposableForInitIr` has private fields and is non-`Copy`/`#[must_use]`;
its sole crate constructor requires both an
`AsyncFunctionAsyncDisposableCapabilityIr` and the already non-empty
`AsyncDisposableResourcesIr`. Consequently, an async-disposable initializer
cannot be empty, cannot carry a generator owner, and cannot be emitted as an
ordinary lexical initializer.

Lowering holds an incomplete initializer only in the private non-`Clone`,
non-`Copy`, `#[must_use]` `PendingAsyncDisposableForInitIr`. It captures the
plain-async entry state, suspension-owned capability binding, and declaration-
ordered resources before lowering the test, update, and body. Only after those
regions have allocated their source states may the pending value be consumed
into `AsyncDisposableForInitIr`, which allocates the finalizer states after the
whole loop region. Public IR never represents an unfinished finalizer.

Analysis treats the classic-for `AwaitUsing` head as `Const`/`Using`: it
registers the head environment and aliases, scans initializer/test/update/body,
and retains captured slots, while the IR environment derives an empty
per-iteration slot set from the dedicated initializer case.

## Explicit nonclaims

This batch does not admit async generators, plain generators, ordinary
functions, modules or dynamic source, binding patterns, `for-of` or
`for-await-of` heads, `await using` outside a classic-for initializer, or any
source `await`/`yield` in the initializer, test, update, or body. It also does
not admit an outer `continue`, an enclosing-loop shape that can dynamically
re-enter the async-disposable loop, or any other repeated/nonlinear execution
of the same classic-for IR node within one activation: its finalizer-state plan
is one-shot. Outer control through an enclosing labelled block, including an
outer `break`, is also excluded. Direct label chains whose target is the
async-disposable `StatementIr::For` remain admitted. Nor does this batch add
asynchronous resource support to synchronous `using` heads. Those forms keep
explicit diagnostics rather than entering this capability domain.

## Focused verification

- `cargo test -p lila-ir plain_async_classic_for_await_using_owns_closed_initializer_capability`
- the four exact Test262 paths above with the Wasm-AOT execution backend
- `cargo fmt --all -- --check`
- `git diff --check`
- `bash scripts/check-module-boundaries.sh`

Broad Test262 and status publication remain separate batch-verification steps.
