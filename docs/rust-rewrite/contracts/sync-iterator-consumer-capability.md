# Sync iterator consumer capability

Status: focused verification passed on 2026-08-29.

## Specification basis

The shared backend path implements the error-bearing parts of the 2026
ECMAScript operations
[`GetMethod`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-getmethod),
[`GetIterator`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-getiterator),
[`GetIteratorFromMethod`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-getiteratorfrommethod),
[`IteratorNext`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-iteratornext),
[`IteratorComplete`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-iteratorcomplete),
[`IteratorValue`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-iteratorvalue),
and
[`IteratorStepValue`](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-iteratorstepvalue).
Array-literal spread follows
[`ArrayAccumulation`](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-runtime-semantics-arrayaccumulation).

## Closed consumer domain

`SyncIteratorConsumer::{ArrayDestructuring, ArrayAccumulation, ForOf,
MathSumPrecise}` is the crate-closed authority for synchronous iterator
diagnostics. The four-variant domain has no `Clone`, `Copy`, comparison,
formatting, default, conversion, or representation capability. Each semantic
owner constructs one value, and the structure guard pins the same borrow
through acquisition and stepping.

`SyncIteratorProtocolError` remains the private four-variant failure domain:

- `NotIterable`;
- `MethodResultNotObject`;
- `NextNotCallable`; and
- `NextResultNotObject`.

The sole projection consumes the error and exhaustively matches its product
with the borrowed consumer. The resulting 16 diagnostic rows are:

| Consumer | Not iterable | Iterator-method result | `next` | `next` result |
| --- | --- | --- | --- | --- |
| `ArrayDestructuring` | `destructuring value is not iterable` | `destructuring iterator method must return object` | `destructuring iterator next must be callable` | `destructuring iterator next result must be object` |
| `ArrayAccumulation` | `array spread value is not iterable` | `array spread iterator method must return object` | `array spread iterator next must be callable` | `array spread iterator next result must be object` |
| `ForOf` | `for-of target is not iterable` | `for-of iterator method must return object` | `for-of iterator next must be callable` | `for-of iterator next result must be object` |
| `MathSumPrecise` | `Math.sumPrecise input is not iterable` | `Math.sumPrecise iterator method must return an object` | `Math.sumPrecise iterator next method is not callable` | `Math.sumPrecise iterator next result must be an object` |

The confirmed source census is 17 typed projector calls and 35
`SyncIteratorProtocolError` identifiers. Those identifiers comprise the
declaration, the typed projector parameter, 17 producers, and 16 mapping rows.
Across the producers and mapping rows, the variants total 10 `NotIterable`, 7
`MethodResultNotObject`, 8 `NextNotCallable`, and 8
`NextResultNotObject` mentions.

## Realm invariant

The consumer controls diagnostic wording only. Every non-nullish primitive
source is boxed through the current function Realm before the backend reads
`@@iterator`. Algorithm-created protocol TypeErrors use a separate exhaustive
builder Realm-source projection. Standard builtins may read the trusted
self-backed current environment; main, user, host, and runtime-helper bodies
use the main-Realm error constructor. A nonzero lexical environment is not
Realm metadata. The source value, iterator object, iterator method, and `next`
method do not select the generated error's Realm.

Accessor, Proxy, iterator-method, `next`, `done`, and `value` throws propagate
their original values. This contract does not rebox user-thrown completions.
The ordinary direct `for-of` owner now reaches that rule for callable Proxy
iterator and `next` methods through general `IsCallable` and Proxy-aware
`Call`; non-callable Proxy methods still select the typed consumer diagnostic.
The callable-Proxy fixture deliberately retains 13 captured bindings and pins
the entry `%TypeError.prototype%` for primitive and non-callable Proxy
iterator methods. This follow-up does not change the 17-producer or
35-identifier census.

## Destructuring and ArrayAccumulation

Array destructuring uses the shared typed acquisition checks, then passes
`ArrayDestructuring` into its custom step owner. That owner checks
`NextNotCallable`, invokes `next`, propagates the call completion, checks
`NextResultNotObject`, reads `done`, and reads `value` only for a value-bearing
element. An elision does not read `value`. The checks use typed error variants,
not raw diagnostic strings.

Array-literal spread passes `ArrayAccumulation` through shared acquisition and
`IteratorStepValue`. Its four diagnostics remain distinct from destructuring.
Abrupt `next`, `done`, or `value` evaluation, a non-callable `next`, and a
primitive next result propagate directly without calling `return`. This is the
no-close control shape required by `ArrayAccumulation`.

## Runtime witness boundary

`wasm_array_destructuring_iterator_abrupt.js` covers destructuring step errors,
completion identity, close and no-close behavior, and elision ordering.
`wasm_array_accumulation_iterator_errors.js` separately covers all four Array
spread diagnostics, no-close behavior for step failures, abrupt `done` and
`value` identity, and a primitive String whose inherited `@@iterator` is
overridden.

Both fixtures execute their syntax in an entry-Realm user function. They can
pin diagnostics, propagation, no-close behavior, and entry-Realm prototype
lookup. They cannot distinguish current-function Realm allocation from the
main Realm fallback. That would require the syntax-owning compiled function to
be defined in a created Realm. Wasm AOT does not dynamically compile `eval`,
`Function`, or cross-Realm Function-constructor source, so no such runtime
result is claimed.

This change does not establish the Realm of `%Array.prototype%` used by a fresh
Array literal or by an Array-rest result. Those prototype-Realm questions remain
separate work.

## Source and verification checkpoint

The extracted async owner is 416 method lines and 420 raw child lines with
SHA-256
`d722dc0abbfda6aea0f1bec2b8fd15cd40f32c34eb443ac082e62744950dcec5`.
Its parent source is 13,224 raw lines. These measurements describe the child
owner after consumer threading; older recorded hashes remain historical
evidence for their checkpoints.

This contract's stable path is
`docs/rust-rewrite/contracts/sync-iterator-consumer-capability.md`.

The all-target compile and formatting check pass. The consumer-capability,
protocol-error, direct `for-of`, `Math.sumPrecise`, resumable plain-async,
iterator-local, destructuring-step, destructuring-local, and IteratorClose
structure targets pass `42/42`. The new ArrayAccumulation fixture plus six
retained exact Wasm-AOT CLI witnesses pass `7/7`. The new fixture also passes
`node --check`, and the module boundary guard is green.

Five pinned Array-spread leaves and four Array-destructuring leaves pass all
`18/18` sloppy/strict Wasm-AOT executions with every failure and non-success
bucket at zero. The callable-Proxy/body-Realm follow-up reran the five directly
affected structure targets at `23/23`, five exact CLI controls at `5/5`, and eight
unchanged direct iterator/Proxy leaves at `16/16`, with every failure and
non-success bucket zero. The all-target compile, formatting, module-boundary,
task-plan, shortcut-inventory, and diff checks remain green.

The focused commands include:

```sh
cargo test -p lila-aot-wasm --test sync_iterator_consumer_capability_structure -- --test-threads=1
cargo test -p lila-aot-wasm --test sync_iterator_protocol_error_ownership_structure -- --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_preserves_array_destructuring_iterator_abrupt_completions -- --exact
cargo test -p lila-cli --test cli array::run_wasm_backend_preserves_array_accumulation_iterator_errors -- --exact
./target/debug/lila --jobs 1 test262 run language/expressions/array/spread-err-sngl-err-itr-step.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run language/statements/variable/dstr/ary-ptrn-elem-id-iter-step-err.js --suite-root test262/vendor/test262 --execution-backend wasm-aot --timeout-ms 180000 --threads 1
```

No semantic golden, published-status refresh, complete Test262 prefix, or broad
workspace suite was run for this checkpoint.
