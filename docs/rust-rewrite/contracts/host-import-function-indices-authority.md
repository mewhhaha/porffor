# Host import function indices authority

Status: implemented; includes the required runtime dynamic-source rejection capability.

## Closed host import roles

The optional Wasm function indices for `number_pow`, `wall_clock_millis`,
shared-memory allocation, `monotonic_clock_nanos`, `sleep_nanos`, `agent_call`,
`intl_call`, and `random_f64`, plus the required `reject_dynamic_source` index,
cross emission planning through one non-copyable
`HostImportFunctionIndices` authority. Each position accepts a distinct,
non-derived Rust role type.

Previously `emit.rs` passed eight adjacent `Option<u32>` values to
`FunctionMetaRegistry::new`. Transposing two values compiled and could route a
generated `call` to the wrong imported capability or Wasm signature. The typed
constructor now rejects such a transposition. There is one complete producer,
and `FunctionMetaRegistry` stores the authority intact rather than flattening it
back into raw fields.

The nine named registry getters are the only raw-index projections.
They remain the semantic boundary at which a role-specific index becomes the
`u32` required by `wasm_encoder::Instruction::Call`; all downstream callers and
the eight optional import roles preserve their order. The required rejection
import follows them, takes one `i64` operation code and returns no value. Its
index cannot be absent: any dynamically selected intrinsic can require this
capability, including one reached through a property or proxy.

## Durable evidence

`host_import_function_indices_structure` uses a Rust lexical scanner that
excludes comments and all Rust string and character literal forms. It pins the
nine exact role types (eight optional and one required), lack of derived capabilities, authority fields and
constructor, recursive source census, sole complete producer, intact registry
storage, and the nine named sole projections.

On 2026-08-27, its five focused structure tests passed, as did
`cargo check -p lila-aot-wasm`. Six existing emission witnesses passed for the
Number power, Date clock, `Math.random`, Intl, Test262 agent, and Atomics
timeout import paths. Rustfmt's check mode and `git diff --check` also passed
for the scoped source, test, task, and contract files.

The role authority does not claim that every host capability or Test262 lane
is complete. The dynamic-source import adds an execution capability boundary;
its typed error is not a JavaScript completion and remains non-passing in
Test262. See [dynamic-source-capability.md](dynamic-source-capability.md).
