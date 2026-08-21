# Plain async-function `for-of` `await using`

## Status and exact boundary

This contract owns the synchronous-iterator form of an `await using` resource
head in a plain async function:

```js
async function f(iterable) {
  for (await using resource of iterable) {
    // no source suspension in this bounded slice
  }
}
```

At committed HEAD `009219b28` and pinned Test262
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, the following five physical files
are raw-red in both applicable async executions (0/10 total):

- `language/statements/await-using/initializer-Symbol.asyncDispose-called-at-end-of-each-iteration-of-forofstatement.js`
- `language/statements/await-using/initializer-Symbol.dispose-called-at-end-of-each-iteration-of-forofstatement.js`
- `language/statements/await-using/syntax/await-using-invalid-assignment-statement-body-for-of.js`
- `language/statements/await-using/syntax/await-using-valid-for-await-using-of-of.js`
- `language/statements/for-of/head-await-using-bound-names-fordecl-tdz.js`

Every execution reports the same current product diagnostic:
`unsupported in lila wasm-aot first slice: await using declaration in for-of`.
These are raw source results, not rewrites, masks, interpreter results, or a
focused synthetic-suite projection.

## Closed producer domain

The generic iterator IR has one dedicated resource-head case:

```text
ForOfIteratorHeadIr::AsyncDisposable(AsyncDisposableForOfHeadIr)
```

`AsyncDisposableForOfHeadIr` has private fields and owns exactly:

- one immutable identifier binding;
- one `AsyncFunctionAsyncDisposableForOfCapabilityIr`; and
- one activation-backed `IteratorRecordIr`.

The capability has private fields, is non-`Copy`, is `must_use`, and owns the
hidden suspension binding plus one validated `AsyncDisposableFinalizerPlanIr`.
Only lowering can construct either carrier. The generic head derives the
synchronous iterator protocol; it carries neither an async-iterator plan nor a
caller-selected protocol witness. Array and String index-walk nodes accept only
`ForOfAssignmentIr`, so a resource head cannot enter either specialization.

A lowering-only, non-`Clone`, non-`Copy`, `must_use`
`PendingAsyncDisposableForOfHeadIr` holds the head after the iterable and
resource protocol have been admitted but before the body is lowered. Consuming
that pending value after the body is the only constructor for the public head,
so finalizer state assignment cannot accidentally precede source-state
allocation.

## Normative lifecycle

1. The lexical head Environment Record is created with the immutable binding
   uninitialized before evaluating the iterable. A read of that name from the
   iterable therefore observes the TDZ.
2. `GetIterator(iterable, sync)` runs exactly once. Its Iterator, NextMethod,
   and Done roles live in the capability owner's activation and survive every
   disposal suspension.
3. Each successful `IteratorStepValue` creates a fresh iteration Environment
   Record with a new immutable cell. The value is not published to the cell
   until resource acquisition succeeds.
4. Acquisition performs `GetMethod(value, @@asyncDispose)` first. If absent it
   performs `GetMethod(value, @@dispose)` and wraps that synchronous method.
   Registration is committed before `InitializeBinding`, so getter/call errors
   and self-references see the required uninitialized cell.
5. A direct `@@asyncDispose` normal result is awaited. The synchronous
   `@@dispose` fallback wrapper ignores a normal return value, including a
   thenable, and supplies fulfilled `undefined`; a synchronous throw becomes
   the wrapper rejection.
6. Body completion is saved, resources are disposed in reverse registration
   order, and disposal completion is folded with the body completion before
   leaving the iteration Environment Record. Only then may `LoopContinues`
   normalize a local `continue`. Other abrupt completions perform
   `IteratorClose` after disposal and before final dispatch. Assignment to the
   immutable head is therefore a caught `TypeError` only after disposal.
7. Normal iteration and local `continue` request the next value using the same
   activation-backed Iterator Record. The finalizer's dispose/resume roles are
   deliberately reused for each iteration; its exit state is entered only
   when the whole loop finishes. `done` exits without acquiring a resource.

Nullish values still register the async-disposal protocol's empty resource
entry and pass through the required await boundary for the iteration.

## Analysis and lowering obligations

- `AwaitUsing` participates in every for-of lexical-head analysis path that
  already admits `Const` and synchronous `Using`: outer TDZ bindings, fresh
  iteration bindings, alias scanning, capture analysis, and immutable modes.
- The sole admitted source head is `AwaitUsing(Binding::Identifier(_))` in a
  plain async function and a non-async `for-of` loop.
- The iterable is lowered while the head binding is in TDZ. The iteration
  binding is initialized only by the async-resource acquisition operation.
- The iterable and body must contain no source `await` or `yield`. This keeps
  the batch's implicit disposal suspension states distinct from unplanned
  source suspension states.
- All closed head and owner domains are handled with exhaustive matches, not
  catch-all arms.

## Explicit nonclaims

This batch does not claim `for-await-of`, binding patterns, ordinary functions,
plain generators, async generators, modules, dynamic source evaluation,
resource initializers or bodies containing source `await`/`yield`, resource
loop heads outside this direct `for-of` form, or nonlinear enclosing control
and repeated execution of the same IR node within one activation. Optional,
property, private, and destructuring assignment heads remain ordinary for-of
assignment grammar, not resource bindings.

The five-file/ten-execution cohort above is the complete publication claim;
larger await-using inventories remain regression evidence until measured on the
same committed product.

## Verification ladder

After the full producer/backend/evidence batch is assembled:

1. `cargo fmt --all -- --check`
2. `git diff --check`
3. `bash scripts/check-module-boundaries.sh`
4. focused `lila-ir` invariant test for the async-disposable for-of head
5. focused CLI lifecycle fixture
6. exact five-file Test262 filter in Wasm-AOT mode (expect 10/10)
7. retained resource-management and iterator-close regressions
8. the repository's broad verification checkpoint and pinned status refresh
