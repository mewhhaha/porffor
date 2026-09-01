# Synchronous Array `for-of` iterator protocol

Status: direct and plain-async paths verified on 2026-08-29.

## Current invariant

Direct synchronous `for-of` lowering has no Array index-walk statement. The IR
has no `StatementIr::ForOfArray`, and the Wasm backend has no
`compile_for_of_array`. An exact Array source that reaches direct statement
lowering uses:

```text
StatementIr::ForOfIterator {
    head: ForOfIteratorHeadIr::Assignment {
        async_plan: None,
        protocol: IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL,
        ..
    },
    ..
}
```

The generic iterator supplies the loop value, so lowering records that value as
`ValueKind::Dynamic` with every run-time kind possible, no heap shape, and no
function targets. The source Array's inferred element type is not evidence for
the result of a replaceable `@@iterator` method.

This removes four assumptions from the direct synchronous path. It no longer
assumes that `%Array.prototype%[@@iterator]` is intact, Array length is stable
while the body runs, indexed reads can bypass `[[Get]]`, or no iterator object
exists for `IteratorClose`.

## Observable witness

`crates/lila-cli/tests/fixtures/wasm_for_of_array_iterator_protocol.js` keeps
the three behaviors separate:

- a loop over `[1]` appends `2` during its first iteration and must visit both
  values, proving that Array iterator `next` observes the new length;
- a hole at index 1 resolves through a temporary `Array.prototype[1]` getter,
  proving that iteration performs an inherited indexed `Get` exactly once; and
- a temporary `Array.prototype[Symbol.iterator]` replacement yields the String
  `"4"`; the loop body computes `"41"`, then breaks, and the iterator's
  `return` method must run exactly once.

The last case covers acquisition, dynamic result typing, and `IteratorClose`
on one direct Array loop. The fixture restores both prototype changes before it
finishes.

## Callable-Proxy follow-up

Because the direct Array path is the generic `compile_for_of_iterator` owner,
a replaced Array `@@iterator` method and the resulting iterator's cached
`next` may now be callable Proxies. Both receive the same original receivers
and empty argument lists as ordinary functions. Apply-trap and revoked-Proxy
completions propagate without becoming iterator-protocol diagnostics. The
source-kind-independent witness lives in
`wasm_direct_for_of_callable_proxy_methods.js`; no Array index-walk exception
or Array-specific call path was added.

## Verification boundary

At the direct-path checkpoint, `cargo check -p lila-aot-wasm` passed and the
focused results were:

- the direct Array protocol structure target passed `3/3`;
- the now-retired resumable Array-walk ownership target passed `4/4`;
- the `lila-ir` `for_of` tests passed `16/16`, and the remaining resumable-walk
  obligation test passed `1/1`;
- the nested generic-iterator planner regression passed `1/1` above the old
  2,048-local floor;
- the new CLI witness and the existing ordinary Array lexical-environment
  control each passed `1/1`; and
- `array-expand.js`, `array-expand-contract.js`, `array-contract.js`, and
  `array-contract-expand.js` passed all `8/8` Wasm-AOT executions, with every
  failure bucket at zero.

The fixture also passes `node --check`. No semantic-golden run or published
status refresh is claimed by this checkpoint.

## Plain-async body-`await` checkpoint

The remaining Array index walk is deleted. A synchronous `for-of` with one
direct body `await` in a plain async function now lowers to
`StatementIr::AsyncFunctionForOfIterator`. Its required
`AsyncFunctionForOfIteratorPlanIr` couples these values behind private fields:

- the assignment binding and the activation-owned `IteratorRecordIr`;
- the head and per-iteration environment lifecycles;
- the body split before, at, and after the source `await`; and
- strictly ordered entry, resume, and exit states.

The constructor checks that the split statement is `AsyncAwait`, that its
suspend state equals the plan entry, and that its resume state is the next
state. It derives the exit state with checked addition. A captured lexical loop
binding becomes `FreshPerIteration`; a captured head TDZ remains rejected.

Lowering no longer asks whether the iterable is an Array. It allocates the
typed iterator, next-method, and done slots through the suspension-owned slot
allocators, assigns `ValueKind::Dynamic` to the yielded value, and attaches
`RESUMABLE_SYNC_ITERATOR_PROTOCOL`. The corresponding
`ResumableSyncForOfIterator` emission site names
`compile_async_function_for_of_iterator` as the owner of `GetIterator`,
stepping, value extraction, and close.

The backend acquires `@@iterator` and reads `next` only on the entry path. It
stores the Iterator Record in the async activation and reloads it after the body
await instead of restarting iteration. Natural exhaustion does not call
`return`. An await rejection closes once and keeps the original Throw even if
close throws. A Return after the await also closes once, but a close error
replaces that Return. Abrupt `next`, `done`, or `value` evaluation does not
close because no loop-body completion owns the iterator yet.

The protocol fixture also runs a directly awaiting String loop with a replaced
`String.prototype[Symbol.iterator]`. The old Array classifier rejected that
source before emission; the new synchronous protocol path accepts it. The same
fixture runs a custom iterable through a bare identifier assignment head and
observes each assigned value on both sides of the body await.

## Focused verification

The bounded structure target and four runtime oracles cover
once-only Array `@@iterator` and `next` acquisition across awaits, a custom
String result from a Number Array, natural exhaustion, directly awaiting String
iteration, bare identifier assignment, close precedence, protocol errors that
do not close, and six fresh captured loop bindings. All four fixtures pass
`node --check`. `cargo check -p lila-aot-wasm` passes. The five focused
structure targets pass `19/19`, the `lila-ir` `for_of` target passes `18/18`,
and the four exact CLI oracles pass `4/4`. The two pinned `Array.fromAsync`
leaves pass `4/4` Wasm-AOT executions with every failure and non-success bucket
at zero. The complete 95-file `Array.fromAsync` leaf, semantic golden, and
published-status refresh were not run.

This form remains limited to a plain async function, one direct body `await`,
and a simple single-name declaration or bare identifier assignment head.
Direct `break` and `continue`, pattern and property heads, a captured head TDZ,
an iterable that suspends, async generators, and `for await` do not use this
plan.

## Member-reference head checkpoint (2026-08-29)

The historical property-head limit above is superseded for static, computed,
and private member-reference heads whose base and key do not suspend. They use
the same `AsyncFunctionForOfIteratorPlanIr`: IteratorValue is stored in the
activation-owned `$forof.access` binding, then the pre-await prefix performs
the member write once per entered iteration inside IteratorClose. Capture
analysis now scans bases and computed keys used only in the head. The
`wasm_plain_async_sync_for_of_member_heads.js` oracle covers reevaluation,
assignment-before-await, no write on resume, public and private failures, close
counts, and Throw precedence. The relevant all-target compile, `21/21` IR
`for_of` filter, `1/1` rejection matrix, `25/25` focused and affected structure
tests, and `2/2` exact member-head and retained capture CLI tests pass. The
fixture passes `node --check`. No matching pinned Test262 cohort or broad
conformance result is claimed. Patterns,
resource heads, `super`, suspending member operands, and the remaining shape
limits above are unchanged; see
[`plain-async-synchronous-for-of-member-heads.md`](./plain-async-synchronous-for-of-member-heads.md).

## Nonlexical pattern-head checkpoint (2026-08-29)

The historical all-pattern limit above is superseded for assignment patterns
and `var` binding patterns. The iterator value enters an activation-owned
synthetic slot, then existing Array/Object destructuring runs once in
`before_await`, inside IteratorClose and before suspension. Assignment-pattern
capture analysis now covers both shapes and nesting directions. The
source-free `wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js` oracle
covers Array and Object `var` forms, computed target order, defaults, rest,
once-only effects, and nested plus outer close precedence. The later lexical
checkpoint below supersedes this historical `let`/`const` rejection. The
relevant compile and formatting check pass; the focused IR checks, six
structure targets, and four CLI oracles pass `25/25`, `25/25`, and `4/4`,
respectively. The fixture passes `node --check` and its Node semantic baseline.
See
[`plain-async-synchronous-for-of-nonlexical-pattern-heads.md`](./plain-async-synchronous-for-of-nonlexical-pattern-heads.md).

## Lexical pattern-head checkpoint (2026-08-29)

The same source-kind-independent iterator plan now admits `let` and `const`
array and object binding patterns. IteratorValue enters an unspellable local,
then BindingInitialization writes the exact complete fresh iteration
Environment Record before the body await. Every BoundName is predeclared in
TDZ before defaults are lowered, and captured as well as uncaptured cells
survive suspension. Empty patterns retain their Array iterator or Object
coercion semantics.

The source-free `wasm_plain_async_sync_for_of_lexical_pattern_heads.js` oracle
uses Arrays, custom outer iterables, and nested inner iterators to cover fresh
cells, TDZ, `const`, empty patterns, and close precedence. The relevant compile
and formatting check pass; the IR filter and rejection witness pass `27/27`
and `1/1`, six structure targets pass `28/28`, and five exact and retained CLI
controls pass `5/5`. The fixture passes `node --check` and its Node semantic
baseline. No matching pinned Test262 cohort is claimed. See
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](./plain-async-synchronous-for-of-lexical-pattern-heads.md).
