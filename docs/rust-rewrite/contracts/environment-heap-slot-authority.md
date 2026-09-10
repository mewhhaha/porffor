# Environment heap-slot identity authority

## Closed layout identities

The passive environment layout contains eight capability-free
`EnvironmentHeapSlot` identities in header and repeated binding order:

- `Parent`;
- `FunctionBody`;
- `NamedBindings`;
- `NamedBindingCount`;
- `WithObjectCell`;
- `RecordKind`;
- `BindingTag`;
- `BindingPayload`.

One private exhaustive `metadata()` projection is the sole authority for the
exact record names, slot names, offsets, widths and pointer classifications.
The environment parent remains a traced 8-byte word at `ENV_PARENT_OFFSET`.
The optional resumable function-body record at byte 8, named binding tables
and with-object cells are traced pointers; table counts
and the closed record kind are scalar words. Ordinary records reserve a
48-byte header before their tagged binding cells. Global Environment roots
have a separate layout with their defining realm and lexical table.
Each repeated binding retains a scalar tag at `ENV_SLOT_TAG_OFFSET` followed by
a traced payload at `ENV_SLOT_PAYLOAD_OFFSET`. An arbitrary row cannot mark the
parent or payload scalar, trace the tag, or exchange their identities.

The focused recursive structure regression pins the exact capability-free
domain, rejects derived and manual incidental capabilities, requires one
no-wildcard metadata projection, preserves typed registry order and verifies
that no second Rust source constructs free-form environment rows. The bounded
heap owner witness asserts every projected field and retains the existing
collision, record-size and pointer census checks.

## Verification

Runtime Environment Record allocation initializes this header before exposing
its named binding table. Named entries reference the existing tagged cells, so
closures and eval share binding identity.

Binding initialization writes the undefined or uninitialized tag and payload
directly into its selected cell. It must preserve the statement-completion
registers: entering a lexical block occurs before its declarations save the
preceding StatementList value. Reusing the result tag as initialization scratch
would expose the TDZ marker or replace a preceding value's type.

```sh
cargo test -p lila-aot-wasm --test environment_heap_slot_structure
cargo test -p lila-aot-wasm --lib heap::tests::environment_heap_slot_identities_own_layout_metadata -- --exact --test-threads=1
cargo test -p lila-aot-wasm --lib heap::tests::heap_layout_registry_ -- --test-threads=1
rustfmt --check crates/lila-aot-wasm/src/heap_environment_layout.rs crates/lila-aot-wasm/src/heap.rs crates/lila-aot-wasm/tests/environment_heap_slot_structure.rs
git diff --check
```
