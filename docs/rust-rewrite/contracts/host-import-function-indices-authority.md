# Host import function indices authority

Status: implemented; includes the required typed runtime-semantics rejection capability.

## Closed host import roles

The optional Wasm function indices for `number_pow`, `wall_clock_millis`,
shared-memory allocation, `monotonic_clock_nanos`, `sleep_nanos`, `agent_call`,
`intl_call`, `system_time_zone`, `random_f64`, and the twenty `math_*` transcendentals (`math_acos`
through `math_tanh` plus `math_atan2`), plus the required
`reject_runtime_semantics` index, cross emission planning through one non-copyable
`HostImportFunctionIndices` authority. Each position accepts a distinct,
non-derived Rust role type.

Previously `emit.rs` passed eight adjacent `Option<u32>` values to
`FunctionMetaRegistry::new`. Transposing two values compiled and could route a
generated `call` to the wrong imported capability or Wasm signature. The typed
constructor now rejects such a transposition. There is one complete producer,
and `FunctionMetaRegistry` stores the authority intact rather than flattening it
back into raw fields.

The thirty named registry getters are the only raw-index projections.
They remain the semantic boundary at which a role-specific index becomes the
`u32` required by `wasm_encoder::Instruction::Call`; all downstream callers and
the twenty-nine optional import roles preserve their order. The configured-zone role is claimed after the existing optional math roles, so
existing optional indices retain their relative order. It reuses the registered
`(i64) -> i64` signature and has a consumed immutable Realm primitive caller.
The required rejection import follows them, takes one closed-domain `i64` reason code and returns no value. Its
index cannot be absent: any dynamically selected intrinsic can require this
capability, including one reached through a property or proxy. The wire domain
also carries explicitly owned compiler gaps such as incomplete Temporal
rounding-window operations; those gaps do not change the import's position or unary signature.

## Durable evidence

`host_import_function_indices_structure` uses a Rust lexical scanner that
excludes comments and all Rust string and character literal forms. It pins the
thirty exact role types (twenty-nine optional and one required), lack of derived capabilities, authority fields and
constructor, recursive source census, sole complete producer, intact registry
storage, and the thirty named sole projections.

On 2026-08-27, its five focused structure tests passed, as did
`cargo check -p lila-aot-wasm`. Six existing emission witnesses passed for the
Number power, Date clock, `Math.random`, Intl, Test262 agent, and Atomics
timeout import paths. Rustfmt's check mode and `git diff --check` also passed
for the scoped source, test, task, and contract files.

The role authority does not claim that every host capability or Test262 lane
is complete. The rejection import adds an execution capability boundary;
its typed error is not a JavaScript completion and remains non-passing in
Test262. See [dynamic-source-capability.md](dynamic-source-capability.md) and
[temporal-named-zone-authority.md](temporal-named-zone-authority.md).

The configured-zone extension and its updated closed-role structural census are
staged with the Date local-time batch. Their compile and runtime checks are
pending; the earlier dated checkpoint above describes the original authority.
