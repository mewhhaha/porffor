# Synchronous String `for-of` iterator protocol

Status: direct and plain-async paths verified on 2026-08-29.

## Current invariant

Direct synchronous `for-of` lowering has no String code-point-walk statement.
The IR has no `StatementIr::ForOfString`, the Wasm backend has no
`compile_for_of_string`, and the iterator-obligation domain has no
`STRING_CODE_POINT_WALK`, `StringIteratorIntact`, or
`StringWalkIsCodePoint` member.

An ordinary direct String loop uses `StatementIr::ForOfIterator` with
`IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL`. The generic iterator
supplies the loop value, so lowering records that value as `ValueKind::Dynamic`
with every run-time kind possible, no heap shape, and no function targets. The
source String type does not constrain what a replaceable `@@iterator` yields.

`GetIterator` boxes a primitive for property lookup with the current
function Realm's `%String.prototype%`. The subsequent property read and method
call retain the original primitive as their receiver. A strict accessor on
`String.prototype[Symbol.iterator]` must therefore see the primitive String,
not the temporary wrapper.

## Observable witness

`crates/lila-cli/tests/fixtures/wasm_for_of_string_iterator_protocol.js`
contains two independent mutations:

- a temporary strict `String.prototype[Symbol.iterator]` accessor records its
  receiver and returns a custom iterator whose first value is the Number `4`;
  the loop computes `5`, breaks, and calls the iterator's `return` once; and
- a temporary replacement for `%StringIteratorPrototype%.next` returns the
  String `"replacement"`, which a loop over an ordinary String must observe.

Both cases save the complete original property descriptor and restore it with
`Object.defineProperty` in a `finally` block.

## Callable-Proxy follow-up

The generic direct owner now accepts a callable Proxy returned by String
`@@iterator` lookup and a callable Proxy in the resulting iterator's cached
`next` slot. The iterator method still receives the primitive String rather
than its lookup wrapper, and `next` receives the iterator; both calls have no
arguments. Apply-trap and revoked-Proxy completions propagate unchanged. The
source-kind-independent witness is
`wasm_direct_for_of_callable_proxy_methods.js`; this adds no String-specific
shortcut or call path.

## Verification boundary

At the direct-path checkpoint, `cargo check -p lila-aot-wasm` passed. The
bounded String structure target passed `3/3`; the affected Array,
synchronous-using, plain-async-using, and String-range companion targets passed
`19/19`; the `lila-ir` `for_of` target passed `17/17`; and the exact CLI witness
passed `1/1`. The fixture also passed `node --check`.

The pinned `string-bmp.js`, `string-astral.js`, and
`string-astral-truncated.js` controls pass all `6/6` Wasm-AOT executions, with
every failure bucket at zero. No semantic-golden run or published-status
refresh is claimed here.

## Error-Realm follow-up

Shared IteratorClose errors for a non-callable `return` method or primitive
`return` result now use the current function Realm; see
`iterator-close-error-realm.md`. At this String-deletion checkpoint, the
generic synchronous loop still retained separate acquisition and stepping
Realm work. The later full boundary routes all five such checks in the direct
String owner through `SyncIteratorConsumer::ForOf`; see
[`direct-synchronous-for-of-protocol-error-realm.md`](./direct-synchronous-for-of-protocol-error-realm.md).
The original String runtime witness exercises successful acquisition,
stepping, and a close call, not those acquisition or stepping throws. The new
entry-Realm error fixture covers the five messages but cannot prove the
created-Realm user-function case because Wasm AOT does not dynamically compile
such a function.

A synchronous String `for-of` whose body directly awaits inside a plain async
function now uses `StatementIr::AsyncFunctionForOfIterator`, the same
activation-backed synchronous Iterator Record plan as an Array or custom sync
iterable. The old Array-only classifier and index walk are deleted. The focused
fixture replaces `String.prototype[Symbol.iterator]`, yields a Number, awaits in
the body, and requires one acquisition and natural exhaustion. The shared
plain-async checkpoint passes `19/19` focused structure tests, `18/18` IR
`for_of` tests, and `4/4` exact CLI oracles; see the plain-async section of
`synchronous-array-for-of-iterator-protocol.md` for the complete verification
boundary.

The later member-reference checkpoint also applies when the synchronous source
is a String: static, computed, and private heads use the same pre-await write
lifecycle. Its source-free oracle is
`wasm_plain_async_sync_for_of_member_heads.js`; no String-specific shortcut or
new iterator owner was added. See
[`plain-async-synchronous-for-of-member-heads.md`](./plain-async-synchronous-for-of-member-heads.md).

The same plain-async String path now admits assignment patterns and `var`
binding patterns. The pattern prefix is source-kind independent: it consumes
the synchronous iterator's Dynamic value once before the body await, using the
same activation-backed record and close frame as Array or custom iterables.
The source-free oracle is
`wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js`. The later lexical
checkpoint below supersedes that checkpoint's historical `let`/`const`
rejection. The shared nonlexical-pattern verification
passes `25/25` focused IR checks, `25/25` focused and affected structure tests,
and `4/4` CLI oracles. See
[`plain-async-synchronous-for-of-nonlexical-pattern-heads.md`](./plain-async-synchronous-for-of-nonlexical-pattern-heads.md).

The same source-kind-independent prefix now admits `let` and `const` array and
object binding patterns. IteratorValue is held in a compiler-only entry local;
BindingInitialization targets one complete fresh Environment Record whose TDZ
is established before defaults are lowered and whose cells survive the body
await. The runtime oracle intentionally uses Array and custom iterables because
the yielded Dynamic value and head semantics do not depend on the synchronous
source kind. The shared lexical-pattern verification passes `27/27` focused IR
checks plus the `1/1` rejection witness, `28/28` focused and affected structure
tests, and `5/5` exact and retained CLI controls. The fixture passes
`node --check` and its Node semantic baseline. See
[`plain-async-synchronous-for-of-lexical-pattern-heads.md`](./plain-async-synchronous-for-of-lexical-pattern-heads.md).
