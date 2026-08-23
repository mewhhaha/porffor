# Iterator flatMap inner close state

Status: normative theory, integrated Wasm-AOT implementation, independent
review and capped focused verification complete for the
`Iterator.prototype.flatMap` abrupt-close invariant, 2026-08-23.

## Specification and lifecycle boundary

[ECMA-262 `Iterator.prototype.flatMap`](https://tc39.es/ecma262/2026/multipage/control-abstraction-objects.html#sec-iterator.prototype.flatmap)
has one relevant phase transition. It first obtains `innerIterator` with
`GetIteratorFlattenable` and applies `IfAbruptCloseIterator` to that result.
Only after that operation succeeds does it set `innerAlive` to true and begin
calling `IteratorStepValue(innerIterator)`. An abrupt completion on either
side closes the outer `iterated` iterator, but only the second side has a live
inner iterator.

The Wasm-AOT flatMap helper represents that transition with the private data
field `$IteratorFlatMapInnerActive`. Its shared abrupt helper,
`emit_iterator_flat_map_close_outer_after_throw`, is entered after the throw or
new TypeError has already been placed in the current completion tuple. The
helper closes the outer iterator while preserving that original throw, marks
the flatMap helper done, optionally clears the inner-active marker, and clears
the executing marker before its caller returns the completion.

Whether the inner marker must be cleared is not an independent Boolean policy.
It is fixed by which side of the specification's `innerAlive` transition the
failure occupies. The state described here also does not decide whether to
close an inner iterator: this helper closes the outer iterator only.

## Closed Rust state

The helper boundary carries exactly this crate-private domain:

```rust
pub(crate) enum IteratorFlatMapInnerState {
    NotInstalled,
    Active,
}
```

`NotInstalled` means that no inner iterator has yet been committed to the
helper's `$IteratorFlatMapInnerIterator` and `$IteratorFlatMapInnerNext` fields
and `$IteratorFlatMapInnerActive` has not been set to true for this mapped
value. Closing after such a failure must not manufacture a write that claims
an active inner iterator was cleared.

`Active` means that those two inner fields were committed, the active marker
was set to true, control branched back to the loop, and the active-inner branch
was entered. Closing after such a failure must set
`$IteratorFlatMapInnerActive` to false.

`emit_iterator_flat_map_close_outer_after_throw` accepts the state instead of
`clear_inner_active: bool`. It projects the two states through an exhaustive
match with no `_` arm:

- `NotInstalled` emits no inner-active write; and
- `Active` emits the existing false write exactly once.

The projection is an emitter-time Rust decision. It does not add a Wasm state
word or runtime branch, and it must not be hidden behind `is_active() -> bool`
or another Boolean-taking wrapper. A third state must fail to compile until
the helper body gives it explicit lifecycle semantics.

## Exact 4/4 caller ownership

The current tree has exactly eight calls to
`emit_iterator_flat_map_close_outer_after_throw`. All eight belong to the sole
`StandardBuiltinId::IteratorFlatMapNext` body in
`crates/lila-aot-wasm/src/builtins/standard.rs`; no return builtin, other
iterator-helper family, or generic control-flow owner calls it.

Exactly four calls select `Active`. They are inside the branch entered after a
runtime read confirms `$IteratorFlatMapInnerActive` and own these failures:

1. calling the installed inner `next` method completes abruptly;
2. the inner `next` result is not an object and creates a TypeError;
3. reading the inner result's `done` property completes abruptly; and
4. reading the inner result's `value` property completes abruptly.

Those four operations are the backend expansion of the active
`IteratorStepValue(innerIterator)` phase. Although this branch appears first
textually in the emitted loop, it is reachable only after an earlier loop
iteration stored both inner fields, set the active marker to true, and branched
back to the loop header.

Exactly four calls select `NotInstalled`. They are in the mapped-value branch
before the unique write that sets `$IteratorFlatMapInnerActive` to true and own
these failures:

1. reading the mapped value's `Symbol.iterator` property completes abruptly;
2. calling that iterator method completes abruptly;
3. the iterator method returns a non-object and creates a TypeError; and
4. reading the selected inner iterator's `next` property completes abruptly.

Only after those operations succeed does the body store the inner iterator and
next method, set `$IteratorFlatMapInnerActive` to true, and branch back to the
active-inner path. The 4/4 mapping is normative. A call-site inversion can
either leave stale active-inner state after a fatal inner step or clear a state
that was never installed.

The nearby non-callable iterator-method, non-callable inner-next, primitive
mapped-result, mapper-throw, and outer-step failure branches currently emit
their own close/finalization sequences. They are not hidden ninth callers and
are outside this bounded carrier migration.

## Preserved close and finalization order

The typed migration must preserve the helper's exact instruction order:

1. `emit_iterator_close_preserving_current_throw` closes the outer iterator;
2. `$IteratorFlatMapDone` is set to true;
3. the exhaustive state projection either does nothing for `NotInstalled` or
   sets `$IteratorFlatMapInnerActive` to false for `Active`;
4. `$IteratorFlatMapExecuting` is set to false; and
5. the caller returns the restored current completion.

No state-finalization write moves before the outer close. The outer iterator's
observable `return` call therefore still runs while the flatMap helper is
executing, and a throw from that close still loses to the already-current
throw. `Done` remains before the optional inner-active clear, and both remain
before clearing `Executing`. The migration does not clear or rewrite the stored
inner iterator or next-method values themselves.

## Bounded structural guard

`crates/lila-aot-wasm/tests/iterator_flat_map_inner_close_state_structure.rs`
must remain a source-bounded mutation guard rather than a second implementation
of flatMap. It must require:

- exactly `IteratorFlatMapInnerState::{NotInstalled, Active}` with no catch-all
  projection;
- one typed helper parameter and no `clear_inner_active: bool`, raw Boolean
  argument, Boolean projection, or alternate Boolean-taking wrapper;
- the exact global inventory of eight helper calls, all owned by the one
  `IteratorFlatMapNext` arm;
- exactly four `Active` calls tied to inner-next call, result-object validation,
  `done` read, and `value` read failures inside the active-inner branch;
- exactly four `NotInstalled` calls tied to `Symbol.iterator` read, iterator
  method call, iterator-result object validation, and inner-`next` read failures
  before installation;
- the unique installation sequence: store inner iterator, store inner next,
  set `$IteratorFlatMapInnerActive` true, then branch back to the loop;
- the helper-body order outer close, `Done`, state projection and optional
  `InnerActive` clear, then `Executing`; and
- no second helper definition, method-item escape, new caller, or direct helper
  bypass elsewhere in `lila-aot-wasm/src`.

The guard should extract only the enum, helper body, and bounded
`IteratorFlatMapNext` arm. It should use the semantic operation and field
anchors above rather than pinning temporary-local names, release-list shape,
indentation, or the whole large standard-builtin body. It must fail if all
eight variants are mechanically swapped even though the enum and counts remain
present.

## Focused witnesses and verified evidence

The existing exact CLI fixture
`crates/lila-cli/tests/fixtures/wasm_iterator_prototype_flat_map.js`, owned by
`iterator::run_wasm_backend_succeeds_for_iterator_prototype_flat_map_fixture`,
already exercises direct active-inner `next`, `done`, and `value` throws,
outer-close observation, primitive mapped-result failure before installation,
original mapper-throw preservation when outer `return` throws, explicit inner
then outer return, and reentrancy. The structural guard remains the direct
proof for all eight call-site state selections.

Two exact pinned Test262 controls bound the two lifecycle sides:

- `staging/sm/Iterator/prototype/flatMap/close-iterator-when-inner-next-throws.js`
  directly witnesses an `Active` inner-next throw and outer close; and
- `staging/sm/Iterator/prototype/flatMap/throw-when-inner-not-iterable.js`
  witnesses outer close when no valid inner iterator becomes active.

The second leaf is a lifecycle control, not proof of every one of the four
`NotInstalled` source calls. Exact per-call ownership belongs to the structural
guard.

The integrated implementation and independent source-level review are
complete. On 2026-08-23, the following focused gates ran under the repository's
`systemd-run` CPU and memory wrapper, which pins work to CPUs 0-7, applies
`CPUQuota=800%`, `MemoryHigh=20G`, `MemoryMax=22G`, and exports
`CARGO_BUILD_JOBS=8`. Cargo tests and Test262 remained single-test or
single-worker where applicable:

- `cargo fmt --all -- --check`, `git diff --check`, and `cargo xc` were green;
- `iterator_flat_map_inner_close_state_structure` passed all `3/3` structural
  tests;
- the exact
  `iterator::run_wasm_backend_succeeds_for_iterator_prototype_flat_map_fixture`
  CLI witness passed `1/1`; and
- the two exact Test262 leaves above passed `4/4` Wasm-AOT variants in total,
  with every reported failure bucket at zero under `--jobs 1 --threads 1`.

The focused commands were:

```sh
cargo fmt --all -- --check
cargo xc
cargo test -p lila-aot-wasm \
  --test iterator_flat_map_inner_close_state_structure -- \
  --test-threads=1
cargo test -p lila-cli --test cli -- \
  --exact iterator::run_wasm_backend_succeeds_for_iterator_prototype_flat_map_fixture
./target/debug/lila --jobs 1 test262 run \
  staging/sm/Iterator/prototype/flatMap/close-iterator-when-inner-next-throws.js \
  --suite-root test262/vendor/test262 --execution-backend wasm \
  --timeout-ms 180000 --threads 1
./target/debug/lila --jobs 1 test262 run \
  staging/sm/Iterator/prototype/flatMap/throw-when-inner-not-iterable.js \
  --suite-root test262/vendor/test262 --execution-backend wasm \
  --timeout-ms 180000 --threads 1
git diff --check
```

## Explicit nonclaims

This invariant-only migration does not change the flatMap algorithm, mapper or
outer-step evaluation order, `GetIteratorFlattenable`, `IteratorStepValue`,
IteratorClose precedence, error construction or Realm selection, helper
reentrancy behavior, runtime field representation, or completion tuple ABI. It
does not close the inner iterator on these throw paths or consolidate the
adjacent hand-emitted close sequences.

It does not generalize the state to map, filter, take, drop, zip, concat,
generators, or async iterators. It does not remove flatMap Test262 rewrites,
refresh a broad Iterator/Test262 cohort, update published status, establish a
conformance gain, or complete T04 or T15.
