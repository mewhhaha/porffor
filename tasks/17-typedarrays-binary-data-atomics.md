# T17 — ArrayBuffer, DataView, TypedArray, SharedArrayBuffer and Atomics

**Status:** In progress — broad binary-data support exists; GC/agents and full-tree closure remain

**Parallel group:** Feature lane; split internally by API family  
**Depends on:** T03, T04, T05, T06, T10; iterator paths use T15; `waitAsync` uses T14  
**Blocks:** Binary-data and concurrency portions of T26

## Current repository state

ArrayBuffer, SharedArrayBuffer, DataView, TypedArray and Atomics have dedicated
backend implementations, including resizable/growable backing-store and
`waitAsync` work with focused fixtures. Binary-data-specific harness rewrites
remain, real GC is unavailable, and the shortcut-free real-agent/full-tree
acceptance criteria have not been demonstrated on a current complete matrix.

The cross-instance async-waiter transport now shares the closed
`lila-runtime::AgentHostOperation` wire domain with the rest of the Wasm agent
ABI. Registration, polling, notification and cancellation are typed at every
AOT producer and exhaustively dispatched by the engine; their stable wire
values remain 10 through 13. This prevents producer/consumer opcode drift but
does not by itself prove waiter semantics or multi-agent stress safety.

Resizable-buffer observation now has a typed AOT seam for callback and
search/access consumers. A private TypedArray view record keeps the stored fixed
byte extent immutable, while a fresh buffer witness derives out-of-bounds
state, element length and an element-aligned index bound from one cached
backing-store length. Its closed use domain distinguishes validated TypedArray
method entry, generic Array length snapshots, live integer-indexed property
observations and the three-kind view-accessor projection. The callback families
shared with T16 use that seam,
including both `reduce` property checks; so do `at`, the generic Array index
searches and the non-generic TypedArray search methods. TypedArray search length
is validated and snapshotted once at method entry, while generic Array search
keeps its `LengthOfArrayLike` and live integer-indexed behavior. Focused
contracts cover fixed-view out-of-bounds/regrow behavior and the Uint16
odd-byte floor.

The shared integer-index validity predicate now consumes the closed
`IntegerIndexedProperty` projection of that same witness. It classifies the
numeric index before loading one immutable view and making one non-throwing
backing-store observation; detached, fixed/tracking out-of-bounds and
index-at-or-above-current-length states all project to an absent property.
Current `Get`, `HasProperty`, `GetOwnProperty`, `DefineOwnProperty`, `Set`,
`Delete` and method callers inherit that observation without reconstructing
private slots, reading backing length separately or dividing byte length
locally. The focused
[integer-index buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-integer-index-buffer-witness.md),
structural guard and expanded `Reflect.has` CLI fixture are written and
focused-verified: `cargo xc` is green, the structure target passes `2/2`, and
the exact CLI fixture passes `1/1`. The direct pinned Test262 leaf discovers
two variants but both stop at the harness's declared `resizable-arraybuffer`
feature gate, so no Test262 pass or unsupported-retirement claim is made.

The witness is still not the universal integer-indexed exotic protocol. Key
classification and each internal method's descriptor, prototype and result
policy remain separate owners, other binary-data consumers still use older
emitters, and no Test262 resizable-buffer rewrite has been retired. The
TypedArray iterator boundaries are migrated separately below; ordinary Array
iterators do not require a TypedArray backing-store witness.
Constructor/subclass and BigInt variants represented by those rewrites remain
separate closure work. The shared `at` emitter encodes its generic-array-like
versus validated-TypedArray receiver policy as a closed enum; the old raw
boolean can no longer route a new caller to the wrong incompatible-receiver
behavior.

ArrayBuffer slicing now has a closed late-source-observation seam. The three
builtin operations project exhaustively to detachable-bounded, shared-bounded,
or detachable-exact-final copy policy. The sole copy writer rechecks ordinary
detachment and reloads current source length and data after observable work.
Ordinary `slice` bounds the copy by the bytes still available from the initially
normalized start, so a species-provided target suffix remains untouched.
`sliceToImmutable` instead rejects a current length below the resolved final
bound before allocating its target, then copies the exact requested length.
Shared sources keep their distinct non-detachable bounded branch. The focused
[slice source re-observation contract](../docs/rust-rewrite/contracts/array-buffer-slice-source-reobservation.md)
and CLI fixture cover detachment during coercion/species, ordinary bounded
resizable shrinkage, and `sliceToImmutable` detach-versus-short-source error
precedence; this is not yet a claim of complete ArrayBuffer or shared memory
correctness.

The three `%TypedArray%.prototype` view accessors now share the same live
buffer-witness seam as the migrated Array/TypedArray consumers. A closed
`TypedArrayAccessorKind` makes `byteLength`, `byteOffset`, and `length` explicit
projections; each builtin delegates with one variant, and the accessor compiler
cannot directly read backing length, data, or the length-tracking slot. The
single witness therefore owns detached/out-of-bounds zeroing, fixed-view
regrowth, and whole-element flooring for odd-byte length-tracking buffers.
The focused
[accessor buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-accessor-buffer-witness.md)
and existing accessor fixture pin those rules. This closes the accessor
duplication, not the older shared indexed `Get`, constructor, or
remaining binary-data consumers, and it does not retire a Test262 rewrite.

TypedArray iterator creation and stepping now use that same live buffer witness
instead of reconstructing private view slots through the older raw validator.
Both boundaries select the closed `ValidatedMethodEntry` projection: creation
consumes validation, while `next` consumes the length derived from the one
cached backing-store observation. Detached and out-of-bounds errors route
through the current function Realm, including created-Realm TypedArray methods
and their Realm-owned `%ArrayIteratorPrototype%.next`. The focused
[iterator buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-iterator-buffer-witness.md)
and existing iterator fixture pin Realm identity, detach/shrink timing, current
resizable length, whole-element flooring and permanently-done behavior. Its
foreign buffers borrow the entry Realm's `resize`, so the proof does not claim
complete created-Realm ArrayBuffer prototype bootstrap. The focused structure
and CLI fixture pass on the current working tree. The remaining raw TypedArray
validators and full integer-indexed/iterator closure remain open; this does not
claim a new Test262 baseline pass.

`%TypedArray%.prototype.join` now uses the validated-method-entry projection of
that same buffer witness. Its compiler performs the receiver-brand check first,
loads one immutable view record, and consumes the witness's element length
directly instead of reconstructing private slots, calling the legacy raw
validator and dividing byte length itself. Detached and out-of-bounds failures
therefore use the executing builtin's Realm, including when a created Realm's
`join` is borrowed onto an entry-Realm receiver. Separator coercion remains
after the initially captured length, and later integer-indexed reads remain
live. The focused
[join buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-join-buffer-witness.md)
and CLI fixture pin Realm identity, fixed and tracking resize behavior, BigInt,
and whole-element flooring. Created-Realm `join` is installed through the
self-backed TypedArray method table; the foreign buffer borrows the entry
Realm's `resize`, so complete created-Realm ArrayBuffer surface parity remains
open. The focused structure and CLI fixture pass on the current working tree.
Remaining raw validators, the shared indexed `Get`, Test262 rewrites and full
binary-data closure remain separate work.

The `%TypedArray%.prototype.reverse` and `toReversed` compilers now use the
same validated-method-entry buffer witness. Each method brand-checks its
receiver, loads one immutable `TypedArrayViewLocals` record and consumes the
element length produced by one `ValidatedMethodEntry` projection instead of
calling the legacy raw validator and dividing byte length locally.
`toReversed` retains its separate element-kind load and intrinsic same-kind
allocation; both reversal loops and their indexed read/write order are
unchanged. The focused
[reverse-family buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-reverse-family-buffer-witness.md)
and bounded source-structure regression record that ownership. Under the shared
eight-core cap, `cargo xc` is green; the structural witness and the exact
`reverse` and `toReversed` CLI fixtures each pass `1/1`. The pinned
`reverse/resizable-buffer.js` and `toReversed/reverses.js` Test262 leaves each
pass `2/2` Wasm-AOT executions with every non-success bucket at zero.

The `%TypedArray%.prototype.sort` and `toSorted` compilers now carry the same
validated-method-entry ownership. Comparator admissibility remains before the
receiver check, and each compiler completes that brand guard before loading one
immutable `TypedArrayViewLocals` record and consuming one
`ValidatedMethodEntry` witness. Both retain one separate element-kind load.
`sort` still targets and returns its receiver; `toSorted` still performs
same-kind allocation, copies the complete captured range before sorting the
distinct result and returns that result. The shared stable-sort emitter is
unchanged. The focused
[sort-family buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-sort-family-buffer-witness.md)
and bounded source-structure regression record those invariants. The
implementation and guard are independently reviewed. Under the shared
eight-core cap, `cargo xc` is green, the structural guard passes `1/1`, and the
exact `sort` and `toSorted` CLI fixtures each pass `1/1`. The pinned
`sort/return-abrupt-from-this-out-of-bounds.js` and
`toSorted/length-property-ignored.js` leaves each pass `2/2` Wasm-AOT
executions with all non-success buckets at zero under `--jobs 1 --threads 1`.
The fixtures now separately preserve their own `length = 50` shadow and check
the six integer-indexed elements, removing a contradictory assertion found by
the focused run. No aggregate or published conformance-count change is claimed.

The four `%TypedArray%.prototype` find-family methods now have the same written
method-entry ownership. Their shared `FindViaPredicateKind` compiler completes
the receiver-brand check, loads one immutable `TypedArrayViewLocals` record and
consumes one `ValidatedMethodEntry` witness before predicate validation. That
witness produces the single snapshot length used by all four direction and
value/index projections; later indexed reads, Proxy-aware predicate calls,
abrupt routing and result policies remain in the existing shared algorithm. The
focused
[find-family buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-find-family-buffer-witness.md)
and hardened bounded source-structure regression record those invariants and
reject the raw validator, private-slot reconstruction, parallel backing-store
observation and local byte-length division. The guard also fixes all eight
Array/TypedArray builtin-to-kind mappings, the single brand-error owner, exact
callback receiver/argument wiring, and the live-read, abrupt-propagation,
truthiness and projection sequence. The implementation and guard are written
and independently reviewed. Under the shared eight-core cap, `cargo xc` is
green, the structural guard passes `4/4`, the exact
`wasm_typedarray_find.js` CLI fixture passes `1/1`, and the current-pin
`find/return-abrupt-from-this-out-of-bounds.js` and
`findLastIndex/detached-buffer.js` leaves each pass `2/2` Wasm-AOT executions
with `--jobs 1 --threads 1`. No new-pass, baseline or published-count change is
claimed.

The `%TypedArray%.prototype.every` and `some` quantifier family now uses one
validated-method-entry witness after its receiver-brand check and before callback
validation. The shared compiler consumes the witness-produced snapshot length
without a raw validator, private-slot reconstruction or local byte-length
division, while retaining live indexed reads, callback ordering and the closed
`Every`/`Some` short-circuit polarities. The focused
[quantifier-family buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-quantifier-family-buffer-witness.md)
and `3/3` structural guard are implemented, independently reviewed and
focused-verified as of 2026-08-23. Under the shared eight-core cap,
`cargo fmt --all -- --check` and `cargo xc` are green; the exact
`wasm_typedarray_every_some.js` CLI fixture passes `1/1`, and the exact
current-pin `every/return-abrupt-from-this-out-of-bounds.js` and
`some/detached-buffer.js` Test262 leaves each pass `2/2`, for `4/4` Wasm-AOT
executions with all failure buckets at zero under `--jobs 1 --threads 1`.

The direct `%TypedArray%.prototype.toLocaleString` entry now uses the same
validated-method-entry witness after its receiver-brand check. One cached
backing-store observation supplies the captured loop length, while the shared
loop retains live per-index reads; the generic
`Array.prototype.toLocaleString` branch keeps its distinct non-throwing
`LengthOfArrayLike` policy. The focused
[toLocaleString buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-to-locale-string-buffer-witness.md),
companion invocation guard and bounded witness guard are implemented,
independently reviewed and focused-verified as of 2026-08-23. Under the shared
eight-core cap, `cargo fmt --all -- --check`, `cargo xc` and `git diff --check`
are green; the companion structure suite passes `4/4`, the witness structure
suite passes `3/3`, and the exact core and invocation CLI fixtures each pass
`1/1`. The pinned out-of-bounds, detached-buffer, mid-invocation growth and
mid-invocation shrink Test262 leaves each pass `2/2`, for `8/8` Wasm-AOT
executions with all failure buckets at zero under `--jobs 1 --threads 1`.

The `%TypedArray%.prototype.map` and `filter` compilers now use that same
validated-method-entry witness after their receiver-brand guards and before
callback validation. Each loads one immutable `TypedArrayViewLocals` record
and consumes its witness-produced snapshot length without a raw validator,
private-slot reconstruction or local byte-length division. The migration keeps
the algorithms' distinct allocation order: `map` performs species construction
before its callback loop, while `filter` completes callback collection before
species construction and selected-value writes. Live per-index reads and the
existing callback `(value, index, receiver)` wiring remain unchanged.

The focused
[map/filter buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-map-filter-buffer-witness.md)
and bounded source guard are implemented, independently reviewed and
focused-verified as of 2026-08-23. Under the shared eight-core, 22 GB cap,
`cargo fmt --all -- --check`, `cargo xc` and `git diff --check` are green; the
structural guard passes `3/3`, and the exact `map` and `filter` CLI fixtures
each pass `1/1`, including detached and out-of-bounds entry controls that prove
the callback is not called. The eight pinned detached, out-of-bounds, growth
and shrink Test262 leaves each pass `2/2`, for `16/16` Wasm-AOT executions with
every failure bucket at zero under `--jobs 1 --threads 1`.

`%TypedArray%.prototype.copyWithin` now uses one immutable
`TypedArrayViewLocals` record and exactly two validated-method-entry witnesses:
the entry witness captures the range before coercion, while a second witness
inside the positive-count branch reobserves the buffer after target, start and
end coercion. The typed seam preserves fixed-view extent, whole-element
flooring, current-length truncation and the zero-count rule that skips the
second observation and all copy setup. Its structural guard pins coercion and
clamping order, both length snapshots, branch containment, current-length
caps, overlap direction and the byte-copy loop.

The implementation and guard were independently reviewed and focused-verified
on 2026-08-23. Under the shared eight-core, 22 GB cap, the structure suite
passes `3/3`, the exact CLI fixture passes `1/1`, and the six exact Test262
leaves pass `12/12` Wasm-AOT variants with every failure bucket at zero under
`--jobs 1 --threads 1`. The source of truth is
`docs/rust-rewrite/contracts/typed-array-copy-within-buffer-witness.md`.

`%TypedArray%.prototype.slice` now loads one immutable source view and consumes
an entry witness plus a conditional post-species witness. The late observation
caps copying after shrinkage while leaving the originally constructed target
length intact, preserves whole-element flooring and stays after target
validation and content-type checks. Its implementation, durable guard and
expanded CLI fixture are focused-verified on 2026-08-24: the structure target
passes `6/6`, the exact CLI fixture passes `1/1`, and the seven pinned leaves
pass all `14/14` Wasm-AOT variants with every failure bucket at zero. The source
of truth is
`docs/rust-rewrite/contracts/typed-array-slice-buffer-witness.md`.

The four Wasm-AOT Atomics access owners now load one immutable
`TypedArrayViewLocals` value and consume one `ValidatedMethodEntry` witness
before index coercion. `notify`, `waitAsync`, `wait` and the shared integer-
operation compiler use the witness-produced element length directly for their
post-`ToIndex` bound; they no longer reconstruct current byte length or admit a
trailing partial element. The validated projection also intentionally corrects
the old fixed-view behavior: an initially detached or out-of-bounds view throws
TypeError before a side-effecting index is coerced, while a valid zero-length
tracking view still coerces the index and then throws the operation-specific
RangeError against the captured length. The pre-coercion backing-pointer
snapshot remains separate for address formation, preserving the current
Atomics pointer timing. The focused
[Atomics buffer-witness contract](../docs/rust-rewrite/contracts/atomics-typed-array-buffer-witness.md),
bounded four-owner structural guard and CLI fixture are focused-verified. The
guard passes `3/3`, the CLI fixture passes `1/1`, and the four exact pinned
Test262 files pass `8/8` Wasm-AOT variants with every non-success bucket at
zero. This does not implement post-coercion `RevalidateAtomicAccess` or claim
complete Atomics semantics.

The shared TypedArray HasProperty predicate used by `Array.prototype.concat`
and the TypedArray receiver branch of `Array.prototype.slice` now consumes the
closed `IntegerIndexedProperty` witness. It keeps its non-throwing result
policy: detached, fixed/tracking out-of-bounds and index-at-or-above-current-
length states are absent, while fixed-view regrowth restores the stored index
extent. The witness also floors odd available byte lengths before comparing an
index, so concat cannot create an own `undefined` target property for a
trailing partial element that should remain a hole. Non-TypedArray receivers
still select the ordinary-object fallback through the separate classification
output. The focused
[concat TypedArray buffer-witness contract](../docs/rust-rewrite/contracts/array-concat-typed-array-buffer-witness.md),
bounded predicate/caller guard and CLI fixture are focused-verified: the
structure target passes `3/3`, the CLI fixture passes `1/1`, and the concat plus
Array-slice Test262 controls pass `4/4` Wasm-AOT variants with every non-success
bucket at zero. This closes one shared raw HasProperty owner, not concat, Array
slice or integer-indexed exotic semantics as a whole.

The direct TypedArray branch of `Object.getOwnPropertyNames` now loads one
immutable `TypedArrayViewLocals` value and consumes one non-throwing
`ArrayLikeLengthSnapshot` witness. The witness-produced element length owns the
ascending integer-key prefix, including detached/out-of-bounds zeroing,
fixed-view regrowth and whole-element flooring for odd-byte length-tracking
buffers. Ordinary String keys remain after that prefix and Symbol keys remain
excluded. Proxy `ownKeys` dispatch and every non-TypedArray fallback keep their
existing order and behavior. The focused
[`Object.getOwnPropertyNames` TypedArray buffer-witness contract](../docs/rust-rewrite/contracts/object-get-own-property-names-typed-array-buffer-witness.md),
bounded owner guard and CLI fixture are focused-verified: `cargo xc` is green,
the structure target passes `3/3`, and the exact CLI fixture passes `1/1`. The
pinned suite has no direct
`Object.getOwnPropertyNames` TypedArray leaf, so the contract inventories the
two smallest adjacent resizable-buffer `[[OwnPropertyKeys]]` controls. They
pass all `4/4` Wasm-AOT variants with every non-success bucket at zero, while
remaining adjacent rather than direct evidence for this compiler.

`%TypedArray%.prototype.subarray` now loads one immutable
`TypedArrayViewLocals` and consumes the non-throwing
`ArrayLikeLengthSnapshot` projection. Detached and initially out-of-bounds
sources therefore contribute a zero source-length snapshot without skipping
begin/end coercion or species construction. An explicitly selected constructor
still owns any later detached-buffer error and its Realm, and a custom species may
return a compatible result. The compiler retains the stored source byte offset,
floors available bytes to whole elements, selects the source element kind for
the intrinsic default constructor, and keeps the normative two-argument result
construction only when the source is length-tracking and `end` is omitted. The
focused
[subarray buffer-witness contract](../docs/rust-rewrite/contracts/typed-array-subarray-buffer-witness.md),
bounded owner guard and CLI fixture are focused-verified: `cargo xc` is green,
the structure target passes `3/3`, the exact CLI fixture passes `1/1`, and the
six direct pinned Test262 leaves pass all `12/12` Wasm-AOT variants with every
non-success bucket at zero. The adjacent custom-species-constructor invocation
leaf retains two `Runtime/Bug` failures already recorded in the pre-batch
current-pin baseline, so it is not included in the witness cohort's pass claim.

These migrations still do not cover `with`, `set`, constructor validation or
other remaining raw validators. They do not change key
classification, caller-specific integer-indexed descriptor/result policy,
result allocation, SharedArrayBuffer synchronization, Test262 rewrites or
published counts. The toLocaleString, map/filter and copyWithin fixtures do not
prove created-Realm buffer-error prototype identity at direct method entry;
only the shared witness's current-function-Realm route is structurally owned
for that case.
`subarray` additionally retains two adjacent semantic debts: its nullish-species
default constructor comes from entry globals rather than the executing Realm,
and its species result does not yet reject detached or out-of-bounds TypedArrays
through `ValidateTypedArray`.

## Objective

Implement the complete binary-data stack, integer-indexed exotic semantics and real agent/Atomics behavior. Replace rejection-only SharedArrayBuffer behavior and harness simulations with general backing-store and host concurrency support.

## Backing stores and ArrayBuffer

- Model detachable, resizable, growable/shared and fixed backing stores separately from view objects.
- Implement `ArrayBuffer` construction, `byteLength`, `maxByteLength`, `resizable`, `resize`, `slice`, `transfer`, `transferToFixedLength`, detachment and species behavior.
- Preserve backing-store identity across views and define safe host access during memory growth/detachment.
- Implement `SharedArrayBuffer` and growable shared buffers where present; they must not be detachable.

## DataView

Complete constructor validation and every get/set method, including:

- ToIndex/offset ordering;
- detached/out-of-bounds checks before and after observable coercion;
- endian handling;
- integer, Float16, Float32/64 and BigInt64/BigUint64 conversion;
- resizable/growable buffer behavior;
- realm/species/custom-new-target descriptors.

## Typed arrays

Implement all concrete typed-array constructors and `%TypedArray%` semantics:

- construction from length, buffer/offset/length, typed arrays and iterables/array-likes;
- integer-indexed exotic internal methods and canonical numeric index strings;
- fixed vs length-tracking views over resizable/growable buffers;
- BigInt/Number element-kind separation, clamping, Float16 and NaN/signed-zero rules;
- all static/prototype methods, iterators, species and subclassing;
- detachment/out-of-bounds validation at exact spec points;
- generic Array method borrowing where allowed and non-generic TypedArray methods where required.

## Atomics and agents

- Implement all Atomics operations with correct element-kind validation and sequentially consistent behavior required by ECMAScript.
- Provide host-managed shared backing stores and actual agent threads/workers for Test262.
- Implement wait queues, `wait`, `notify`, `waitAsync`, timeouts, `isLockFree`, blocking restrictions and monotonic timing.
- Integrate job completion for `waitAsync` with T14.
- Eliminate regex/source-pattern agent simulations from the embedded
  `lila-test262` local harness under T03.

### Resolved CLI hang and remaining concurrency debt

`binary_data::run_wasm_backend_succeeds_for_atomics_wait_core_fixture` used to
hang the CLI suite. The bounded known-failure machinery detected when it began
passing in batch 6, and its hang row, `should_panic` annotation and compile-time
ledger assertion were removed together. It is now an ordinary passing test and
the current CLI ledger contains no declared hang. The suite must run without an
`atomics_wait_core` skip.

That focused result proves only that the fixture's non-equal waits return; it
does not prove the real-agent acceptance criteria below. Host-managed agents,
wait queues, notifications, timeouts and `waitAsync` job integration remain
open until the real Test262 agent trees pass without source-pattern simulation.
The generic per-invocation timeout and watched-run safeguards remain useful for
detecting the next hang and are not evidence of an expected failure.

## Wasm/runtime strategy

The backend uses a hybrid design. Shared scalar memory operations use Wasm
shared memory and atomic instructions. Host-managed agent orchestration and
the cross-instance `waitAsync` waiter registry use the typed `agent_call`
import, because waiters and reports must cross independently instantiated Wasm
modules. The host operation is decoded into a closed Rust enum before semantic
dispatch; an unknown wire value is a visible host error. Both paths must still
preserve JavaScript object identity, detachment rules and agent
synchronization. Single-threaded scripted simulation is not concurrency
coverage.

## Acceptance criteria

- Complete pinned trees for ArrayBuffer, SharedArrayBuffer, DataView, TypedArray constructors/prototypes and Atomics are green.
- Integer-indexed exotic descriptor/key/proxy cases pass.
- Resizable/growable buffer tests pass before, during and after coercion/callback mutation.
- BigInt and Number typed arrays reject mixed values correctly.
- Real multi-agent wait/notify/report tests pass without source pattern matching.
- Detached/out-of-bounds checks occur at spec-required times.
- No data races or host panics under repeated agent stress tests.

## Required tests

```sh
cargo test -p lila-aot-wasm typed_array_ --quiet
cargo test -p lila-spec-exec agent_ --quiet
cargo test -p lila-test262 agent_ --quiet
cargo test -p lila-cli wasm_typed_array --quiet
./target/debug/lila test262 run built-ins/ArrayBuffer --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/lila test262 run built-ins/TypedArray --execution-backend wasm --timeout-ms 180000 --threads 4
./target/debug/lila test262 run built-ins/Atomics --execution-backend wasm --timeout-ms 180000 --threads 2
```

Run DataView and every concrete typed-array subtree separately during implementation, then execute shared-buffer/agent tests under repeated stress.
