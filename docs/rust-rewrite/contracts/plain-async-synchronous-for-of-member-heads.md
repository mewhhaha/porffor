# Plain-async synchronous `for-of` member heads

## Boundary

A synchronous `for-of` with one direct body `await` in a plain async function
admits static, computed, and private member-reference heads. The yielded value
first enters the activation-owned `$forof.access` binding. The existing
per-iteration prefix then evaluates the member Reference and performs its
`PropertyWrite` or `PrivateWrite` before the body suspension.

The prefix is part of `AsyncFunctionForOfIteratorPlanIr::before_await`. The
backend executes that list only in the entry state and inside the loop's
IteratorClose frame. Consequently each successful iterator step re-evaluates
the base and computed key exactly once, completes the write before the body
await, and does not repeat the write when the async function resumes. An abrupt
base, key, setter, or private-brand operation closes the iterator; the existing
Throw-completion precedence retains that original error if `return` also
throws.

Capture analysis scans the member base and computed key as part of the
per-iteration head. A nested async function therefore cannot silently treat an
outer binding used only by the member Reference as a global.

## Evidence

`wasm_plain_async_sync_for_of_member_heads.js` covers static writes, computed
base/key re-evaluation onto two different targets, assignment-before-await and
no-repetition-on-resume, public setter failure, successful private-field
writes, wrong-brand failure, IteratorClose counts, and Throw precedence. The
IR tests separately pin `PropertyWrite`, `PrivateWrite`, the activation value
binding, and captured computed-reference operands.

`cargo fmt --all -- --check` and
`cargo check -p lila-ir -p lila-aot-wasm -p lila-cli --all-targets` pass. The
IR `for_of` filter passes `21/21`, and the explicit rejection matrix passes
`1/1`. The main structure target plus five affected companion targets pass
`25/25`. The exact member-head CLI test and retained async iteration-capture
test pass `2/2`; the fixture passes `node --check`.

No exact Test262 run is claimed because the pinned suite has no leaf combining
this member head with the directly awaiting plain-async body. The complete
Test262 directory, semantic golden, published-status refresh, and broad
workspace test suite were not run for this checkpoint.

## Nonclaims

This checkpoint itself did not admit declaration or assignment patterns. The
later nonlexical-pattern checkpoint supersedes that historical limit for
assignment patterns and `var` binding patterns, and the lexical-pattern
checkpoint supersedes it for `let` and `const` patterns. Resource heads,
`super` References, suspension inside the member base or key, direct `break`
or `continue`, multiple or nested body suspensions, the older single-name
captured head TDZ, suspending iterables, async-generator owners, and `for
await` remain nonclaims. No pinned Test262 leaf combines a member-reference
head with this directly awaiting plain-async body shape, so the fixture is
source-free evidence rather than a full-suite conformance count. See
[`plain-async-synchronous-for-of-nonlexical-pattern-heads.md`](./plain-async-synchronous-for-of-nonlexical-pattern-heads.md)
and
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](./plain-async-synchronous-for-of-lexical-pattern-heads.md).
