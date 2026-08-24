# Iterator Helper Prototype operation dispatch

## Status

Integrated and focused-verified as a T15 compile-enforced seam on 2026-08-24.

## Specification boundary

Every iterator-helper object has the shared `%IteratorHelperPrototype%` as its
prototype. That prototype supplies two callable operations:

- `next`, which advances the concrete helper; and
- `return`, which closes the concrete helper.

Lila implements the shared methods with one brand dispatcher. The dispatcher
recognizes seven concrete helper brands: concat, zip, map, filter, flatMap,
take and drop. The zip brand has two creation surfaces, `Iterator.zip` and
`Iterator.zipKeyed`, so the runtime matrix covers eight surfaces. Each brand
has a distinct `next` builtin and `return` builtin.

This contract is about choosing between those two operations after the helper
brand is known. It does not restate the internal semantics of any family.

## Pre-change hole

`FunctionBuilder::emit_iterator_helper_dispatch` accepted `is_return: bool`.
The `%IteratorHelperPrototype%.next` producer passed `false`; the `return`
producer passed `true`. For every recognized brand, one `if is_return` then
selected between two same-typed `StandardBuiltinId` values.

The booleans carried a closed specification domain without naming it. Adding an
operation could silently reuse one side of the boolean, and an inverted literal
at either producer compiled. The choice is load-bearing: it controls all seven
helper families through the shared prototype.

Current behavior was correct. This seam hardens an invariant; it is not a claim
to repair a known runtime failure.

## Closed representation

The backend owns one private type:

```rust
enum IteratorHelperPrototypeOperation {
    Next,
    Return,
}
```

There is no `Default` and no boolean conversion. The only producers are the two
`StandardBuiltinId` arms that compile the shared prototype methods. The one
consumer matches both variants exhaustively while selecting each family's
target builtin.

The compile-time guarantees are deliberately narrow:

- passing a boolean to the dispatcher is `E0308`;
- omitting the operation argument is `E0061`; and
- adding an operation without defining its target selection is `E0004`.

Rust does not prove that a deliberately wrong named variant or a deliberately
wrong `StandardBuiltinId` was written. The seven-brand, eight-surface behavioral
matrix is the durable oracle for those semantic associations. Claiming more
would turn the type into documentation-shaped evidence.

## Producer and consumer inventory

| Role | Site | Obligation |
|---|---|---|
| producer | `StandardBuiltinId::IteratorHelperNext` | construct `Next` |
| producer | `StandardBuiltinId::IteratorHelperReturn` | construct `Return` |
| consumer | `emit_iterator_helper_dispatch` | exhaustively choose `next_builtin` or `return_builtin` |
| family rows | concat, zip, map, filter, flatMap, take, drop | supply the two concrete targets |
| creation surfaces | concat, zip, zipKeyed, map, filter, flatMap, take, drop | construct every branded receiver |

The enum and dispatcher are private to `lila-aot-wasm`. There is no public Rust
API, re-export or migration surface.

## Runtime and byte contract

For the two existing variants, target selection is identical to the previous
boolean branch. Receiver validation, brand order, bootstrap filtering, direct
call ABI, completion propagation and emitted instruction order do not change.
The source transformation is therefore expected to be byte-neutral, but no
emitted-Wasm byte comparison was performed for this checkpoint.

The regression borrows the exact shared `next` and `return` functions and calls
both on fresh helpers from all eight creation surfaces. It proves that:

1. all eight helper objects share the same prototype;
2. borrowed `next` yields the family's first expected value;
3. borrowed `return` returns a completed iterator result; and
4. `next` after `return` remains completed.

The existing helper-prototype fixture covered shared identity for map, filter,
flatMap, take, drop and zip, but only exercised borrowed `next` for map and zip
and borrowed `return` for map. A second fixture covered both operations only for
drop. Neither was a complete dispatch-table oracle.

## Owned files and concurrency

- `crates/lila-aot-wasm/src/builtins/standard.rs`
- `crates/lila-aot-wasm/tests/iterator_helper_prototype_operation_structure.rs`
- `crates/lila-cli/tests/fixtures/wasm_iterator_helper_prototype_dispatch_matrix.js`
- `crates/lila-cli/tests/cli/iterator.rs`
- this contract
- `tasks/15-generators-iterators-resource-management.md`

This file set does not overlap the active T13 dynamic-source or T14 async-resume
implementation files. Test262 snapshots are explicitly outside the batch.

## Verification ladder

Static owner gates:

1. scoped `rustfmt --check`;
2. `git diff --check` over the six owned files;
3. a repository scan showing no `is_return: bool`, `if is_return` or boolean
   argument at `emit_iterator_helper_dispatch`; and
4. a source inventory showing exactly two named operation producers and one
   exhaustive consumer.

The central checkpoint passed `cargo check -p lila-aot-wasm` and `cargo xc`.
The executable structure target passed `4/4`; the eight-surface CLI matrix and
the existing helper-prototype and drop-dispatch CLI regressions each passed
`1/1`.

At Test262 pin `e9d582d6b8b13afc5ba9a676664741592b5c7f69`, four unrewritten leaves
provided the bounded runtime projection:

- `staging/sm/Iterator/prototype/lazy-methods-return-closes-iterator.js`;
- `built-ins/Iterator/concat/return-is-forwarded.js`;
- `built-ins/Iterator/zip/suspended-yield-iterator-close-calls-return.js`; and
- `built-ins/Iterator/zipKeyed/suspended-yield-iterator-close-calls-return.js`.

Each leaf produced two ordinary sloppy/strict executions. All `8/8` Wasm-AOT
executions passed with every failure bucket at zero. This cohort is a bounded
projection of the shared `next`/`return` dispatch: it is not exclusive
attribution for every behavior observed by those tests and is not broader
`IteratorClose` evidence.

## Nonclaims

This seam does not complete iterator-helper laziness, branding or close
precedence; `IteratorClose` or `AsyncIteratorClose`; async iterator helpers;
generator suspension; explicit resource management; a Test262 status milestone;
emitted-Wasm byte identity; or any parser/VM-in-Wasm work.
