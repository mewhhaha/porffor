# Plain-async synchronous `for-of` nonlexical pattern heads

## Boundary

A synchronous `for-of` with one direct body `await` in a plain async function
admits assignment patterns and `var` binding patterns. Array and object forms
use the ordinary destructuring machinery, including nested patterns,
non-suspending defaults, rest elements and properties, identifier targets,
and public or private member targets.

The iterator value first enters the activation-owned `$forof` binding. The
pattern initialization or assignment is then part of
`AsyncFunctionForOfIteratorPlanIr::before_await`. The backend executes that
prefix only in the entry state, after opening the outer IteratorClose frame
and before the body suspension. A resume therefore observes the completed
destructuring without re-evaluating its computed keys, defaults, target
References, getters, setters, or rest copies.

`var` BoundNames use the plain async function's activation Environment Record,
so every assigned value remains available after the body await and the names
retain function-scoped `var` identity across iterations. An assignment pattern
creates no loop binding: all of its References are prepared and consumed by
the pre-await prefix.

Capture analysis exhaustively scans both array and object assignment patterns.
It records computed source keys, identifier targets, public and private target
bases and keys, defaults, rest targets, and both array-to-object and
object-to-array nesting. A nested async function cannot silently treat an
outer binding used only by the pattern head as a global.

## Abrupt completion

An abrupt computed key, getter, default, target Reference, setter, private
brand check, nested destructuring operation, or rest copy occurs inside the
outer loop's close frame and before the body begins. Array destructuring owns
its inner IteratorClose independently. If the pattern throws, the inner
iterator closes first when applicable, then the outer `for-of` iterator
closes; both preserving-Throw paths retain the original pattern error when a
`return` method also throws. Neither the body nor its await runs.

## Evidence

`wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js` covers array and
object `var` patterns across suspension, default and rest behavior, values
remaining visible after the await and after loop exhaustion, computed object
assignment keys, prepared member targets, getter order, rest targets,
once-only evaluation, and nested plus outer IteratorClose Throw precedence.
The focused IR regressions pin activation ownership, binding versus assignment
evaluation, typed identifier/property/private targets, and exhaustive capture
analysis in both nesting directions.

`cargo fmt --all -- --check` and
`cargo check -p lila-ir -p lila-aot-wasm -p lila-cli --all-targets` pass. The
IR `for_of` filter passes `24/24`, and its exact unsupported-shape rejection
witness passes `1/1`. Six focused and affected structure targets pass `25/25`.
The new CLI oracle and the retained iterator-record, member-head, and captured
iteration-environment oracles pass `4/4`. The fixture passes `node --check` and
prints its success marker under the Node semantic baseline.

The pinned Test262 checkout contains no executable leaf combining a
synchronous pattern head with a directly awaiting plain-async body. The
fixture is therefore source-free evidence; no Test262 numerator or denominator
is attributed to this checkpoint.

## Nonclaims

This checkpoint itself did not admit `let` or `const` pattern heads. The later
lexical-pattern checkpoint supplies their closed multi-binding input, full
fresh per-iteration Environment Record, and TDZ ownership; see
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](./plain-async-synchronous-for-of-lexical-pattern-heads.md).
Resource patterns, a captured TDZ for the older single-name declaration,
suspension in the iterable or pattern, `super` and dynamic `with` targets,
direct `break` or `continue`, multiple or nested body suspensions, async
generators, and `for await` remain outside this boundary. The inner
array-destructuring protocol-error Realm policy is a separate migration.
