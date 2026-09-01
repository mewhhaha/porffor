# RegExp matcher failure route

Status: implemented for the Wasm-AOT RegExp matcher failure boundary.

## Boundary

`RegExpMatcherFailureRoute::{GenericError,
CurrentFunctionRealmRangeError}` is the crate-private compile-time authority for
turning a typed matcher failure into a JavaScript throw. The matcher-status row
source maps `CorruptProgram` to the existing generic `Error` and
`ResourceExhausted` to a current-function-realm `RangeError`; the string builtin
wrapper is the sole product consumer and matches both routes exhaustively.

The route domain derives no cloning, copying, debugging, equality, ordering,
hashing or default-construction capability. Its status-row constructor is the
only production projection, and adding a route requires the wrapper to state a
throw policy before the crate builds. The owner unit also projects both rows
through an exhaustive route match instead of comparing route values.

## Durable evidence

`crates/lila-aot-wasm/tests/regexp_matcher_failure_route_structure.rs`
recursively pins all eight source mentions, the exact no-capability two-row
domain, both status-row mappings and messages, the macro-owned exhaustive
projection, the owner-unit projection, and the sole product consumer's error
constructors and return order.

The neighboring matcher-status unit remains the focused behavioral authority
for words, routes and messages. The separate runtime-entry-kind structure
target remains the neighboring RegExp wire-domain control.

The dedicated structure target passes `3/3`, the exact owner unit passes
`1/1`, the neighboring runtime-entry-kind structure target passes `3/3`, and
`cargo fmt --all -- --check` is green. Independent dry review found the
strengthened declaration, complete row table and route-bound throw guard clean.
`cargo xc`, the diff check and repository boundary checks are also green. CLI,
Test262, semantic-golden and broad-suite verification remain deferred to a
coordinated checkpoint.

## Nonclaims

This source-equivalent capability closure changes no matcher status word,
message, generated Wasm instruction, JavaScript error constructor, Realm,
scratch-arena rewind, `lastIndex` policy or RegExp semantics. It does not add a
step budget, make resource exhaustion reachable from a valid current program,
close arbitrary runtime compilation or claim broader Test262 progress.
