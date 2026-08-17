# T16 — Array exotic semantics and complete Array API

**Status:** In progress — many Array leaves are green; materialization-free full-tree closure remains

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10; iterator consumers use T15  
**Blocks:** Array-related T26 closure

## Current repository state

Array exotic storage, descriptors, species and most prototype families have
substantial implementations and many focused complete-leaf results recorded in
the README. `crates/lila-aot-wasm/src/builtins/array.rs` remains a very large
shared implementation file, and the Test262 harness still contains numerous
Array-specific path rewrites and source reductions. This task cannot close
until the full current-pin Array tree is green through general semantics.

The generic callback tranche (`map`, `every`, `some`, `filter`, `find*`,
`forEach`, `reduce` and `reduceRight`) and the search/access tranche now observe
borrowed TypedArrays through a closed view/witness API. The view carries the
immutable fixed extent, a length witness snapshots one backing-store length for
`LengthOfArrayLike`, and each live integer-indexed `HasProperty` or `Get` gate
takes a fresh witness. `at` selects generic length observation or validated
TypedArray entry through its closed receiver policy. Generic `includes`
continues to perform the observable `LengthOfArrayLike` and per-index `Get`
operations rather than borrowing the non-generic TypedArray entry rule. This
prevents an out-of-bounds observation from erasing the extent needed after a
later regrow, and length-tracking views floor odd backing-byte lengths to whole
elements.

The Array and TypedArray `find`, `findIndex`, `findLast` and `findLastIndex`
emitters now share one closed `FindViaPredicateKind`. Exhaustive projections
select the forward/reverse walk and value/index result; the old generic
booleans and unreachable TypedArray-only branch are gone. A private, non-Copy
predicate witness is constructible only through the general `IsCallable`
operation and has one ownership-consuming, Proxy-aware `Call` boundary. This
admits callable Proxy predicates while retaining receiver/length observation
before callability validation for both entry families. The exact boundary and
its nonclaims are recorded in
`docs/rust-rewrite/contracts/array-find-via-predicate.md`.

The distinct Array and TypedArray `toLocaleString` entry points now share one
element-invocation boundary. A private, non-`Copy` validation token pairs the
general-`IsCallable`-validated method with the exact original element receiver,
and its sole ownership-consuming call path is Proxy-aware and passes no
arguments. A non-callable element method now throws in the active built-in's
current-function realm, including when a created realm's Array or TypedArray
method is borrowed. The exact boundary, static evidence and baseline
nonclaims are recorded in
`docs/rust-rewrite/contracts/array-to-locale-string-invocation.md`.

The shared `at` emitter also receives a closed receiver policy rather than a
raw validation boolean. Generic `Array.prototype.at` and the validated
`%TypedArray%.prototype.at` path are the only inhabitants, so adding another
receiver policy cannot silently inherit either branch's error behavior.

Array-owned `Symbol.isConcatSpreadable` data properties now retain one exact
tagged JavaScript value instead of an eagerly coerced truthiness word. A closed
`ArrayConcatSpreadableSlotValue::{Data, Getter}` shape owns the sole occupied
slot writer, pairing every payload with its tag and exhaustively selecting the
data/accessor descriptor role. Reads distinguish absence from the two occupied
shapes, return data unchanged, and invoke callable getters; `concat` remains
the later boundary that applies `ToBoolean`. The shared tagged payload slot is
already a GC edge, so object identity is retained without growing the Array
record. The old pointer-free truthiness cell is no longer a behavioral source
or sink, but its allocator initialization and physical slot remain pending a
conflict-free record-layout cleanup. The storage, read-order and verification
boundaries for this seam are recorded in
`docs/rust-rewrite/contracts/array-concat-spreadable-tagged-slot.md`.

This is not yet a universal borrowed-Array seam. Indexed `Get` still belongs to
the general object/integer-indexed protocol, and Array iterators, getters and
other exotic consumers have not been migrated to the witness type. The
existing Test262 materializers also remain: several encode constructor/subclass
and BigInt breadth that these invariant migrations do not settle, so none can
be honestly deleted on their strength alone. The `@@isConcatSpreadable` seam
also does not claim complete descriptor attributes, deletion/redefinition,
inherited setters, Proxy traps or Array-record compaction.

The available current-pin Array prototype baseline predates the T10
`Object.prototype.toLocaleString` repair and still records two primitive
`toLocaleString` failures caused by its former boxed getter and call receiver.
The Object path now statically preserves the original primitive through GetV
and Proxy-aware Call. The focused structure and CLI fixture pass on the current
working tree; pinned Test262 execution remains deferred, so neither seam
carries a current-SHA baseline-delta or full-subtree-green claim.

`Array.prototype.pop` now has one compiler algorithm owner. Statically named
method calls delegate to `StandardBuiltinId::ArrayPrototypePop` instead of
reading and shrinking the raw dense-Array heap record in `functions.rs`. The
canonical standard body therefore owns `ToObject`, `LengthOfArrayLike`, the
last-property `Get`, deletion, current-function-realm deletion errors, and the
strict `length` write in their observable order. The former direct path could
resurface an old dense slot after a later length regrowth and could not observe
accessors, descriptors or deletion failures. The ownership boundary and its
focused static evidence are recorded in
`docs/rust-rewrite/contracts/array-pop-algorithm-owner.md`.

The focused structure test and CLI fixture for this `pop` seam pass on the
current working tree. The pinned leaf and broader Array checkpoint remain
deferred. It changes no published count, removes no Test262 materializer,
carries no current-SHA snapshot delta, and does not claim the Array or Array
prototype tree is green.

## Objective

Complete Array exotic object behavior and every pinned Array constructor/prototype method using general internal operations. Retire focused static Test262 materializations as each family becomes fully semantic.

## Array exotic object

Implement and validate:

- `ArrayCreate`, initial prototype selection and maximum length handling;
- `ArraySetLength`, including descriptor validation, truncation order, deletion failures and rollback;
- `[[DefineOwnProperty]]` for canonical array indexes and `length`;
- dense-to-sparse transitions without changing key ordering or hole semantics;
- inherited indexes, accessors, non-writable length and non-extensible arrays;
- canonical index boundaries around `2^32 - 1` and large named numeric keys;
- `ownKeys` ordering and interaction with symbols/proxies.

Dense storage is an optimization only. Every observable operation must agree with the exotic protocol.

## Constructors and species

Complete `Array`, `Array.of`, `Array.from`, `Array.fromAsync` if present, `Array.isArray`, `@@species`, subclass construction and cross-realm behavior. Constructors must use iterator closing, mapping call order and custom `this` semantics.

## Prototype families

Implement the full pinned API, grouped so separate PRs can land within this task:

- mutators: `push`, `pop`, `shift`, `unshift`, `splice`, `copyWithin`, `fill`, `reverse`, `sort`;
- creators: `concat`, `slice`, `toSpliced`, `toReversed`, `toSorted`, `with`, `flat`, `flatMap`;
- search/access: `at`, `includes`, `indexOf`, `lastIndexOf`, `find*`;
- iteration/callback: `forEach`, `map`, `filter`, `every`, `some`, `reduce`, `reduceRight`;
- string/locale: `join`, `toString`, `toLocaleString`;
- iterators: `keys`, `values`, `entries`, `@@iterator`.

## Correctness matrix

Every method must be exercised against:

- ordinary arrays, sparse arrays and arrays with inherited indexes;
- generic array-like receivers and primitive receivers where allowed;
- proxies/accessors with observable operation order;
- subclasses, species constructors and cross-realm constructors;
- typed-array borrowed receivers where generic;
- mutation during iteration and length snapshots;
- abrupt callbacks/coercions and iterator closing.

Avoid method-specific duplicates of `LengthOfArrayLike`, `HasProperty`, `Get`, callback invocation or species logic.

## Acceptance criteria

- `built-ins/Array` and `built-ins/Array/prototype` are fully green for the pin.
- No path-specific materializer remains for covered Array tests.
- Array index/length descriptor edge cases pass.
- Sort is stable and obeys comparator/coercion/holes/undefined semantics.
- Sparse arrays do not cause loops proportional to impossible maximum lengths when the spec permits key-based optimization; observable access order remains correct.
- Species, proxy, inherited-index and cross-realm tests pass across creator methods.
- Adjacent TypedArray generic-borrow tests do not regress.

## Required tests

```sh
cargo test -p lila-aot-wasm array_ --quiet
cargo test -p lila-cli wasm_array --quiet
./target/debug/lila test262 run built-ins/Array --execution-backend wasm --timeout-ms 180000 --threads 8
```

During development use method-level filters and deterministic shards. Before closing, run the entire Array tree and all local `wasm_array_*` fixtures.
