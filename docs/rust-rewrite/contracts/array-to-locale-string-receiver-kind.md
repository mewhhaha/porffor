# Array `toLocaleString` receiver kind

Status: current Wasm-GC source authored on 2026-10-05. Compilation, runtime
regressions and conformance verification have not run.

Generic Arrays capture observable length; TypedArray admission captures the sole typed read-view length. Both use the same whole-value locale invocation owner.

The shared receiver policy is the private, non-derived two-case
`ToLocaleStringReceiverKind`. The four Array/TypedArray `join` and
`toLocaleString` entries each choose it with their operation, and the shared
operation projects it only through exhaustive matches. The structure guard is
`cargo test -p lila-aot-wasm --test structure_builtins -- to_locale_string_receiver_kind_structure::`.

Values and completions retain typed GC references. Arrays use the single sparse
indexed storage owner; observable length never sizes a hole allocation. Old raw
payload/local-offset and direct-dispatch source-string mirrors are retired.
Actual Engine and CLI behavior controls remain the verification evidence; their
authorship does not establish a passing result.

See [native Array GC methods](native-array-gc-methods.md) and
[Array indexed storage](gc-array-indexed-storage.md) for the complete current
owners, lifecycle rules and unrun finite controls. Historical beforeimages and
source-only receipts remain in the atomic migration packet.
