# Iterator-protocol obligation contract

This area's contract lives in one document, because the iterator obligations'
`EmissionSite` values are what witness the spec-operation catalog's
`StatementEmission` rows and the two would drift if split:

**[Spec-operation catalog evidence and the iterator-protocol obligation witness](./Spec-operation%20catalog%20evidence%20and%20the%20iterator-protocol%20obligation%20witness.md)**

Start at §1.2–§1.4 (spec basis) and §4 (type mapping, Part B) for the iterator
half. §9 holds the dry-run corpus and the three corrections to the area brief.

As built: `crates/lila-ir/src/iterator_obligations.rs`, with the
`EmissionSite` → real-function join in
`crates/lila-aot-wasm/src/emission_sites.rs`. §12 (encoder addendum) records
the deviations; D3 and D4 are the two that touch this half.

**§13 is the dry-run discrepancy pass and supersedes §§1–12 where they
disagree.** For this half, read 13.1 (the integration was not applied and now
is), 13.4 (the slot transposition was not actually `E0308`; the allocators fix
it), 13.5 (there was already a fourth for-of specialization, so the witness is
attached to the head lowering), 13.6 (`IntactnessPremise` conflated three kinds
of claim), 13.7 (a partial intactness guard exists and is not consulted), 13.9
(`EmissionSite` is a set; L6 retired) and 13.12 (the "emitter must not read
this" rule is now `pub(crate)`).

## Current synchronous Array checkpoint (2026-08-29)

Direct synchronous Array `for-of` no longer assumes the iterator protocol away.
`StatementIr::ForOfArray`, `compile_for_of_array`, and the synchronous
`ARRAY_INDEX_WALK` witness are gone. Exact Arrays now lower to
`StatementIr::ForOfIterator` with an assignment head, no async plan, and
`SYNC_ITERATOR_PROTOCOL`. Their yielded value is `Dynamic`, not the inferred
Array element type.

The focused witness covers Array growth during iteration, an inherited indexed
getter, and a replaced prototype `@@iterator` that yields a String and receives
one `return` call after `break`. The two structure targets pass `3/3` and `4/4`,
the IR `for_of` target passes `16/16`, the planner and two CLI targets each pass
`1/1`, and four pinned Array length-mutation leaves pass `8/8` Wasm-AOT
executions with every failure bucket at zero.

The former `ARRAY_INDEX_WALK_RESUMABLE` path is now deleted. See
[the focused contract](./synchronous-array-for-of-iterator-protocol.md) and
§20 of the combined evidence contract.

## Current plain-async synchronous `for-of` checkpoint (2026-08-29)

`StatementIr::AsyncFunctionForOfIterator` is the dedicated plain-async form for
a synchronous `for-of` whose body directly awaits. Its closed plan owns one
activation-backed `IteratorRecordIr`, the body split, environment lifecycle,
and ordered entry/resume/exit states. Lowering attaches
`RESUMABLE_SYNC_ITERATOR_PROTOCOL`; the exhaustive
`ResumableSyncForOfIterator` emission-site join names
`compile_async_function_for_of_iterator`.

The entry path performs `GetIterator` and reads `next` once. Resume paths reload
the stored record. The result is `Dynamic`, so the path no longer has an Array
classifier and also accepts synchronous String and custom iterables. Natural
exhaustion skips `return`. Body Throw and Return completions close once with
the required precedence; `next`, `done`, and `value` errors do not close.

The old `AsyncForOfArrayWalkForm`,
`lower_async_for_of_array_with_body_await`,
`ARRAY_INDEX_WALK_RESUMABLE`, and Array length/index synthesis are gone.
Focused evidence includes the existing six-capture per-iteration binding oracle
and a runtime bare-assignment oracle. The five structure targets pass `19/19`,
the IR `for_of` target passes `18/18`, and the four exact CLI oracles pass
`4/4`. The admitted head is a single-name declaration or bare identifier
assignment.
Direct `break`/`continue`, pattern and property heads, captured head TDZ,
iterable suspension, async-generator owners, and `for await` remain outside
this form. The combined
evidence contract records this as §20.

## Current plain-async member-head checkpoint (2026-08-29)

The historical property-head nonclaim above is superseded for non-suspending
static, computed, and private member References. Lowering stores IteratorValue
in `$forof.access`, lowers the Reference write through the existing
`PropertyWrite` or `PrivateWrite` prefix, and places it in
`AsyncFunctionForOfIteratorPlanIr::before_await`. The backend already executes
that prefix on entry only, inside IteratorClose and before the body await.
Capture analysis scans the base and computed key. The
`wasm_plain_async_sync_for_of_member_heads.js` oracle covers success, changing
targets and keys, public setter and private-brand failures, close counts, and
Throw precedence. Patterns, resource heads, `super`, suspending member
operands, and the other resumable-shape nonclaims remain outside this batch.
`cargo fmt --all -- --check` and the relevant all-target compile pass. The IR
`for_of` filter and explicit rejection matrix pass `21/21` and `1/1`; six
focused and affected structure targets pass `25/25`; and the exact member-head
and retained capture CLI tests pass `2/2`. The fixture passes `node --check`.
No matching pinned Test262 cohort or broad conformance result is claimed. The
exact boundary is in
[`plain-async-synchronous-for-of-member-heads.md`](./plain-async-synchronous-for-of-member-heads.md).

## Current plain-async nonlexical-pattern checkpoint (2026-08-29)

The historical pattern-head nonclaim above is superseded for assignment
patterns and `var` binding patterns. The activation-owned `$forof` slot receives
IteratorValue, and the existing Array/Object destructuring prefix runs entry
only inside IteratorClose before the body await. `var` BoundNames live in the
async activation; assignment References do not cross suspension. Capture
analysis exhaustively visits object and array assignment patterns at every
nesting level. The source-free
`wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js` oracle covers
defaults, rest, computed/reference order, once-only effects, and nested plus
outer close Throw precedence. The relevant all-target compile and formatting
check pass, the focused IR checks pass `25/25`, six focused and affected
structure targets pass `25/25`, and the new plus retained CLI oracles pass
`4/4`. The fixture passes `node --check` and its Node semantic baseline. The
lexical-pattern checkpoint below supersedes this checkpoint's historical
`let`/`const` rejection. No matching pinned Test262 cohort is claimed. See
[`plain-async-synchronous-for-of-nonlexical-pattern-heads.md`](./plain-async-synchronous-for-of-nonlexical-pattern-heads.md).

## Current plain-async lexical-pattern checkpoint (2026-08-29)

`let` and `const` array and object binding patterns now use the same
resumable synchronous iterator owner. The closed plan derives activation,
iteration-environment, or compiler-only entry-local IteratorValue storage from
the source head. A lexical pattern pairs the entry local with exact complete
iteration and TDZ name sets plus its BindingInitialization prefix. Analysis
materializes all BoundNames before capture hops are computed; lowering
predeclares their final storage before defaults; and the backend publishes the
fresh Environment Record before initialization and keeps it active across the
await.

`wasm_plain_async_sync_for_of_lexical_pattern_heads.js` covers complete fresh
environments, forward and captured-head TDZ, mutable `let`, `const` writes,
empty patterns, and inner plus outer close precedence. The relevant all-target
compile and formatting check pass; the IR filter and rejection witness pass
`27/27` and `1/1`; six structure targets pass `28/28`; and the exact plus
retained CLI controls pass `5/5`. The fixture passes `node --check` and its
Node semantic baseline. No matching pinned Test262 cohort is claimed. See
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](./plain-async-synchronous-for-of-lexical-pattern-heads.md)
and §25 of the combined evidence contract.

## Current shared IteratorClose error-Realm checkpoint (2026-08-29)

The shared `emit_iterator_close` owner now creates its non-callable `return`
and primitive `return`-result TypeErrors in the current function Realm. Its 67
external entry routes comprise 16 direct calls, 48 preserving-current-Throw
calls, and 3 preserving-saved-Throw calls. The preserving wrappers still
restore the incoming Throw. Entry code with `current_env_local == 0` still uses
the main Realm fallback.

At this close-only checkpoint, ordinary direct `for-of` acquisition and
stepping errors remained a separate migration. The source-structure target
passes `4/4`, the exact created-Realm CLI test passes `1/1`, the affected close
sweep passes `6/6`, and the two direct `for-of` Test262 leaves pass `4/4`. The
later full boundary below supersedes that historical nonclaim. See
[the focused contract](./iterator-close-error-realm.md) and §21 of the combined
evidence contract.

## Current direct synchronous `for-of` protocol-error Realm checkpoint (2026-08-29)

The five synchronous iterator-protocol checks in each of three owners now use
one closed body-Realm projection, for 15 checks in total. Ordinary direct
`compile_for_of_iterator` and direct async-disposable
`compile_async_disposable_for_of_iterator` own five inline checks each.
Resumable plain-async `compile_async_function_for_of_iterator` delegates its
five checks to the shared acquisition and stepping emitters. Every route
selects `SyncIteratorConsumer::ForOf` and reaches the exhaustive
`SyncIteratorProtocolError` diagnostic projection. The two inline
owners, including the async-disposable path, also box primitive lookup through
the current function Realm. Their algorithm-created errors use the main Realm
because main and user lexical environments are not self-backed Realm records;
a standard-builtin body remains eligible for its trusted current Realm.

The entry-Realm CLI fixture covers the nullish source, non-callable iterator
method, primitive iterator result, non-callable `next`, and primitive `next`
result branches plus a valid control. It cannot distinguish a legitimate
current-function Realm from the main Realm. A Realm-distinguishing runtime witness
would require a compiled user function defined in a created Realm, and dynamic
function compilation remains outside the Wasm-AOT contract. The focused and
affected structure targets pass `37/37`, the CLI cohort passes `5/5`, and four
pinned direct leaves pass all `8/8` Wasm-AOT executions. At that checkpoint it
did not absorb the two shared IteratorClose errors, the `for await` path, Array
destructuring, ArrayAccumulation, or `Math.sumPrecise`. The next checkpoint
supersedes the synchronous-consumer part of that nonclaim. The catalog remains
46 rows. See
[the focused contract](./direct-synchronous-for-of-protocol-error-realm.md) and
§22 of the combined evidence contract.

The ordinary direct owner now also uses the general callability operation and
Proxy-aware Call for both `@@iterator` and cached `next`. The source-free
follow-up fixture covers exact receivers, empty argument lists, apply-trap
completion identity, non-callable Proxy diagnostics, revoked callable
Proxies, once-only `next` lookup, and no close on abrupt stepping. A bounded
Rust guard forbids Function-tag gates and Function-only calls in this owner.
The direct async-disposable and resumable shared owners were already
Proxy-aware. This changes neither the 15 typed checks nor the catalog census,
and the entry-Realm fixture makes no cross-Realm Proxy-error claim. The
callable-Proxy/body-Realm follow-up passes `23/23` affected structure tests,
`5/5` exact CLI controls, and `16/16` unchanged iterator/Proxy Test262
executions with every failure bucket zero.

## Current synchronous iterator consumer checkpoint (2026-08-29)

The non-`Copy`
`SyncIteratorConsumer::{ArrayDestructuring, ArrayAccumulation, ForOf,
MathSumPrecise}` domain now selects diagnostics only. Its product with the four
`SyncIteratorProtocolError` variants is one exhaustive 16-row projection.
There are 17 typed producers and 35 error identifiers in the confirmed source
census. Primitive acquisition still boxes through the current function Realm.
Algorithm-created protocol TypeErrors instead match the closed builder
Realm-source domain: trusted standard builtins use their self-backed current
Realm, while main, user, host, and runtime-helper bodies use the main Realm.
Nonzero lexical environments are never treated as function Realm metadata.

Array destructuring threads its named consumer through the custom step owner,
including typed checks before and after calling `next`, then observes `done`
before any value-bearing read. Array-literal spread has separate `array spread`
diagnostics and keeps the direct-propagation, no-close behavior required by
2026
[`ArrayAccumulation`](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-arrayaccumulation).
The entry-Realm fixtures cannot distinguish current-function from main-Realm
error identity, so no created-Realm runtime result is claimed. This checkpoint
also does not claim the current function Realm's `%Array.prototype%` for fresh
Array literals or Array-rest results.

The all-target compile and formatting check pass. Nine structure targets pass
`42/42`; seven exact Wasm-AOT CLI witnesses pass `7/7`; and nine pinned
Array-spread/destructuring leaves pass all `18/18` sloppy/strict executions
with every failure bucket at zero. See
[`sync-iterator-consumer-capability.md`](./sync-iterator-consumer-capability.md)
and §26 of the combined evidence contract.

## Current synchronous String checkpoint (2026-08-29)

Direct synchronous String `for-of` no longer assumes the iterator protocol
away. `StatementIr::ForOfString`, `compile_for_of_string`,
`STRING_CODE_POINT_WALK`, `StringIteratorIntact`, and
`StringWalkIsCodePoint` are gone. Ordinary String heads lower to
`StatementIr::ForOfIterator` with `SYNC_ITERATOR_PROTOCOL`, and their yielded
value is `Dynamic`.

The generic GetIterator path boxes primitive property lookup with the current
function Realm's String prototype while retaining the primitive as the
observable accessor and method receiver. The focused witness temporarily
replaces `String.prototype[Symbol.iterator]` and
`%StringIteratorPrototype%.next`, requires a Number value from the former, and
requires one `return` call after `break`. The String structure target passes
`3/3`, affected companion structures pass `19/19`, the IR `for_of` target passes
`17/17`, and the CLI witness passes `1/1`. Three pinned native String controls
pass `6/6` Wasm-AOT executions with every failure bucket at zero.

This direct-path checkpoint did not claim complete iterator-protocol error
Realm ownership; the later full-boundary checkpoint above supersedes that
historical nonclaim. A synchronous String loop with a directly awaiting body
in a plain async function now uses the activation-backed Iterator Record
checkpoint described above. See
[the focused contract](./synchronous-string-for-of-iterator-protocol.md) and
§§19–20 of the combined evidence contract.
