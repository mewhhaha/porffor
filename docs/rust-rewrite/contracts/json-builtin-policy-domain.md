# JSON builtin policy domain

Status: implemented and privately bounded through Batch AO.

## Closed dispatch

The private, capability-free `JsonBuiltin` contains exactly `Parse`,
`Stringify`, `RawJson` and `IsRawJson`. It never leaves `builtins/json.rs`.
The standard-builtin dispatcher can call only four private fixed semantic
wrappers, each of which constructs one owned selection for its corresponding
JSON namespace member. The JSON emitter consumes that selection through one
exhaustive match, so a future namespace member must supply its complete emitter
route before the crate compiles.

The domain cannot be cloned, copied, formatted, defaulted, compared, ordered or
hashed. A selection therefore cannot be retained or forked across multiple
dispatch decisions. Hidden static-JSON lowering and the parse, reviver,
stringify and raw-JSON machinery remain implementation details rather than
members of this namespace policy.

## Durable regression

`json_builtin_policy_domain_structure.rs` pins the exact four variants, four
fixed wrapper producers, four wrapper-only standard routes, one owned
parameter, one exhaustive consumer, the recursive ten-mention product census
and the absence of derived or manual incidental capabilities. It also rejects
raw policy imports, construction or compiler calls from the standard
dispatcher.

```sh
cargo test -p lila-aot-wasm --test json_builtin_policy_domain_structure --quiet
```

Batch AJ changes no emitted instruction or operation ordering and claims no new
JSON behavior. It does not close JSON grammar, reviver, replacer, raw-JSON,
deep-input or full pinned Test262 conformance. Shared `cargo xc` passes, the
structure target passes `4/4`, and exact JSON parse and stringify CLI witnesses
pass `2/2`. No Test262 cohort or semantic golden was needed or run.

Batch AO makes the raw domain and compiler private to `builtins/json.rs` and
exposes only the four fixed semantic wrappers to the standard dispatcher. The
former ten-line raw selection has SHA-256
`c3276e86866cc00345ee4ad017e710465e8b9a4d9973ba045a7f576ffe7beee0`;
the four-line fixed-wrapper selection has SHA-256
`72b2e14d442b15efe1a15d2d6fc3755e96b49bf7c9d0c75bb8da719efb0dcf8d`.
This source-equivalent boundary changes no emitted instruction and claims no
new JSON behavior. At the Batch AO checkpoint, `cargo xc` is green, the
strengthened structure target passes `4/4`, and the exact parse/stringify
controls pass `2/2`. No Test262 leaf or semantic golden was required or run.
