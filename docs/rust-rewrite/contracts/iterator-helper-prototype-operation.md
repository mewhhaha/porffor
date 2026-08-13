# Iterator Helper Prototype operation dispatch

## Status

Integrated as a T15 compile-enforced seam. Cargo and Test262 verification are
owned by the central verifier and were not run while this contract was written.

## Specification boundary

Every iterator-helper object has the shared `%IteratorHelperPrototype%` as its
prototype. That prototype supplies two callable operations:

- `next`, which advances the concrete helper; and
- `return`, which closes the concrete helper.

Lila implements the shared methods with one brand dispatcher. The dispatcher
recognizes seven concrete helper families: concat, zip, map, filter, flatMap,
take and drop. Each family has a distinct `next` builtin and `return` builtin.

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
wrong `StandardBuiltinId` was written. The seven-family behavioral matrix is the
durable oracle for those semantic associations. Claiming more would turn the
type into documentation-shaped evidence.

## Producer and consumer inventory

| Role | Site | Obligation |
|---|---|---|
| producer | `StandardBuiltinId::IteratorHelperNext` | construct `Next` |
| producer | `StandardBuiltinId::IteratorHelperReturn` | construct `Return` |
| consumer | `emit_iterator_helper_dispatch` | exhaustively choose `next_builtin` or `return_builtin` |
| family rows | concat, zip, map, filter, flatMap, take, drop | supply the two concrete targets |

The enum and dispatcher are private to `lila-aot-wasm`. There is no public Rust
API, re-export or migration surface.

## Runtime and byte contract

For the two existing variants, target selection is identical to the previous
boolean branch. Receiver validation, brand order, bootstrap filtering, direct
call ABI, completion propagation and emitted instruction order do not change.
Existing programs should therefore emit byte-identical Wasm.

The regression borrows the exact shared `next` and `return` functions and calls
both on fresh helpers from every family. It proves that:

1. all seven helper objects share the same prototype;
2. borrowed `next` yields the family's first expected value;
3. borrowed `return` returns a completed iterator result; and
4. `next` after `return` remains completed.

The existing helper-prototype fixture covered shared identity for map, filter,
flatMap, take, drop and zip, but only exercised borrowed `next` for map and zip
and borrowed `return` for map. A second fixture covered both operations only for
drop. Neither was a complete dispatch-table oracle.

## Owned files and concurrency

- `crates/lila-aot-wasm/src/builtins/standard.rs`
- `crates/lila-cli/tests/fixtures/wasm_iterator_helper_prototype_dispatch_matrix.js`
- `crates/lila-cli/tests/cli/iterator.rs`
- this contract
- `tasks/15-generators-iterators-resource-management.md`

This file set does not overlap the active T13 dynamic-source or T14 async-resume
implementation files. Test262 snapshots are explicitly outside the batch.

## Verification ladder

Static owner gates:

1. scoped `rustfmt --check`;
2. `git diff --check` over the five owned files;
3. a repository scan showing no `is_return: bool`, `if is_return` or boolean
   argument at `emit_iterator_helper_dispatch`; and
4. a source inventory showing exactly two named operation producers and one
   exhaustive consumer.

Deferred central gates:

1. `cargo check -p lila-aot-wasm`;
2. the new focused CLI matrix regression;
3. the existing helper-prototype and drop-dispatch CLI regressions;
4. pinned concat, zip, map, filter, flatMap, take and drop next/return filters;
5. T15's broader required iterator suites; and
6. emitted-Wasm comparison for an existing helper fixture, with no new snapshot.

## Nonclaims

This seam does not complete iterator-helper laziness, branding or close
precedence; `IteratorClose` or `AsyncIteratorClose`; async iterator helpers;
generator suspension; explicit resource management; a Test262 status milestone;
or any parser/VM-in-Wasm work.
