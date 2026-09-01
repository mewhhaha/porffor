# Prepared destructuring property-key ownership

Status: implemented for array and object destructuring assignment property targets.

## Boundary

`PreparedDestructuringPropertyKey` is the private closed carrier between
`prepare_destructuring_target` and `put_destructuring_target`. A static target
key owns its string and no temporary locals. A computed target key can be
constructed only after both its payload and tag locals have been reserved,
populated and checked for abrupt completion; that variant owns the raw key
shape and both locals together.

The write consumes the carrier. It installs a temporary key binding only for
the computed variant, exhaustively projects both variants to `PropertyKeyIr`,
then exhaustively releases both computed locals after PutValue. The previous
independent `Option<u32>` fields admitted a static key with locals, a computed
key without one local, or a key form that disagreed with its local pair.

The carrier derives no clone, copy, default, equality, ordering, hashing or
debug capability. It is not exported and has no fallback projection.

## Durable evidence

`crates/lila-aot-wasm/tests/prepared_destructuring_property_key_ownership_structure.rs`
Rust-lexically pins the private two-variant declaration, the nine production
mentions, computed-key construction after both locals are populated, the
computed-only scope binding, both exhaustive projections and tag-before-payload
release order.

The retained array-destructuring abrupt-completion fixture exercises the same
prepared-property-target path and its static-key row, including target
preparation before the iterator protocol fails. Focused verification commands:

```sh
cargo test -p lila-aot-wasm --test prepared_destructuring_property_key_ownership_structure -- --test-threads=1
cargo test -p lila-cli --test cli array::run_wasm_backend_preserves_array_destructuring_iterator_abrupt_completions -- --exact --test-threads=1
cargo check -p lila-aot-wasm --lib
```

## Nonclaims

This is source-equivalent Rust ownership hardening. It changes no evaluation
order, temporary-local count, IR, emitted Wasm or JavaScript behavior. The
retained semantic fixture does not directly witness the computed target-key
row; that row remains structurally closed here, while broader destructuring
conformance remains part of T08.
