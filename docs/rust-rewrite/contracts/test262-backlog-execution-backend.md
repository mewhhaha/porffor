# Test262 backlog execution backend

`BacklogArtifact.execution_backend` stores the closed engine
`ExecutionBackend` domain. The backlog generator moves that typed backend into
the artifact once. File naming and human-readable output project its canonical
label with `as_str()`; they do not own another string copy of the backend.

The field-specific serde codec preserves the established JSON spellings
`spec-exec` and `wasm-aot`. Deserialization admits only those two rows. An
unknown spelling such as `future-backend` is schema corruption and returns an
error that includes the rejected spelling instead of entering backlog,
comparison, or reporting state.

The serializer matches `ExecutionBackend` exhaustively without a catch-all.
Adding an engine backend therefore fails compilation until its backlog wire
policy is explicit. The snapshot schema retains its separately versioned
legacy string field; this contract governs generated backlog artifacts only.

Focused verification:

```console
cargo test -p lila-test262 --test backlog_execution_backend_structure -- --test-threads=1
cargo test -p lila-test262 --lib tests::generate_backlog_writes_deterministic_failure_inventory -- --exact --test-threads=1
cargo check -p lila-test262 --quiet
```
