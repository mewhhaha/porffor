# T16 — Array exotic semantics and complete Array API

**Status:** In progress — many Array leaves are green; materialization-free full-tree closure remains

**Parallel group:** Feature lane  
**Depends on:** T04, T05, T10; iterator consumers use T15  
**Blocks:** Array-related T26 closure

## Current repository state

Array exotic storage, descriptors, species and most prototype families have
substantial implementations and many focused complete-leaf results recorded in
the README. `crates/porffor-aot-wasm/src/builtins/array.rs` remains a very large
shared implementation file, and the Test262 harness still contains numerous
Array-specific path rewrites and source reductions. This task cannot close
until the full current-pin Array tree is green through general semantics.

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
cargo test -p porffor-aot-wasm array_ --quiet
cargo test -p porffor-cli wasm_array --quiet
./target/debug/porf test262 run built-ins/Array --execution-backend wasm --timeout-ms 180000 --threads 8
```

During development use method-level filters and deterministic shards. Before closing, run the entire Array tree and all local `wasm_array_*` fixtures.
