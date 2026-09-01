# Host import function indices authority

Status: implemented as a source-equivalent Wasm-AOT compiler invariant.

## Closed host import roles

The optional Wasm function indices for `number_pow`, `wall_clock_millis`,
shared-memory allocation, `monotonic_clock_nanos`, `sleep_nanos`, `agent_call`,
`intl_call`, and `random_f64` cross emission planning through one non-copyable
`HostImportFunctionIndices` authority. Each position accepts a distinct,
non-derived Rust role type.

Previously `emit.rs` passed eight adjacent `Option<u32>` values to
`FunctionMetaRegistry::new`. Transposing two values compiled and could route a
generated `call` to the wrong imported capability or Wasm signature. The typed
constructor now rejects such a transposition. There is one complete producer,
and `FunctionMetaRegistry` stores the authority intact rather than flattening it
back into eight raw fields.

The eight existing named registry getters are the only raw-index projections.
They remain the semantic boundary at which a role-specific index becomes the
`u32` required by `wasm_encoder::Instruction::Call`; all downstream callers and
host import ordering are unchanged.

## Durable evidence

`host_import_function_indices_structure` uses a Rust lexical scanner that
excludes comments and all Rust string and character literal forms. It pins the
eight exact role types, lack of derived capabilities, authority fields and
constructor, recursive source census, sole complete producer, intact registry
storage, and the eight named sole projections.

On 2026-08-27, its five focused structure tests passed, as did
`cargo check -p lila-aot-wasm`. Six existing emission witnesses passed for the
Number power, Date clock, `Math.random`, Intl, Test262 agent, and Atomics
timeout import paths. Rustfmt's check mode and `git diff --check` also passed
for the scoped source, test, task, and contract files.

This boundary changes no host import condition, order, index arithmetic,
signature, emitted instruction, public API, or conformance count. It is not a
claim that every host capability or Test262 lane is complete.
