# Object.assign compiler owner

## Private ownership boundary

The complete `Object.assign` compiler lives in the private
`builtins/object/assign.rs` module. Its 262-line compiler family moved together:
target and source coercion, `Reflect.ownKeys`, per-key descriptor observation,
enumerability filtering, source `Get`, target `Set`, abrupt completion and
temporary-local release retain one implementation owner. The parent keeps only
the private module declaration, while standard dispatch keeps one fixed call.

The exact original body has SHA-256
`65680b329345d9833065718b97a828484f464208d19bc5ff09d7f6ad3a46f6cd`.
Normalizing only the child entry visibility from `pub(in crate::builtins)` to
the former `pub(super)` reproduces that hash exactly.

## Durable invariant

`object_assign_owner_structure.rs` requires one private module declaration,
one sibling-visible fixed entry, one standard-dispatch call and absence of the
compiler from the parent. It also pins the three Reflect dependencies and the
descriptor, key-conversion and source-read stages inside the child, so a
partial move or copied implementation fails cheaply.

## Verification and nonclaims

```sh
cargo test -p lila-aot-wasm --test object_assign_owner_structure
./target/debug/lila test262 run built-ins/Object/assign/Target-Object.js --threads 1 --execution-backend wasm --snapshot-dir <temporary-directory>
cargo xc
cargo fmt --all -- --check
git diff --check
```

At the 2026-08-28 Batch AR checkpoint, `cargo xc` is green, the owner structure
target passes `4/4`, and the exact `Target-Object.js` Wasm-AOT leaf passes `2/2`
with every failure bucket at zero. This source-equivalent extraction claims
no new Object behavior, broader Object conformance or published
conformance-count change.
