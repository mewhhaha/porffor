# Async-generator request Promise Realm

Status: implemented and focused-runtime-verified on 2026-08-26.

## Ownership boundary

`%AsyncGeneratorPrototype%.next`, `.return` and `.throw` execute
`NewPromiseCapability(%Promise%)` before validating their receiver. The
intrinsic Promise constructor belongs to the Realm of the executing request
method. The generator object's activation Realm and the Realm installed for a
Promise job are not constructor authority.

The three request methods share one dispatcher arm and one capability path.
Both valid requests and invalid-receiver rejection Promises therefore use the
same method-defining Realm.

## Catalog

The Realm intrinsic record stores a traced canonical `%Promise%` constructor
at offset 416 and occupies 424 bytes. Entry bootstrap writes the initialized
Promise constructor global. Created bootstrap writes the exact constructor
local that it later publishes as the Realm's `Promise` global. Realm record
allocation zeroes the slot with every other intrinsic entry before either
bootstrap populates it.

Property lookup on the Realm global or `%Promise.prototype%.constructor` is not
an acceptable substitute: both properties are mutable and observable.

## Typed capability

`CurrentFunctionRealmIntrinsicPromiseConstructor` is opaque, non-`Copy` and
must-use. Its only factory follows the executing function object's defining
Realm to the Realm intrinsic record and loads the required Promise constructor
slot. Missing function, Realm, intrinsic record or constructor is an internal
invariant failure. The factory has no entry-global or dynamic current-Realm
fallback.

The private
`builtins/promise/current_function_realm_intrinsic_promise_capability.rs`
module owns the proof type, its factory and its consuming capability adapter.
The Promise parent does not import or re-export the proof, and the grouped
request dispatcher can only pass its inferred value between those child-owned
methods. Raw constructor-payload construction and projection are therefore not
available to adjacent Promise algorithms or other builtin families.

The proof can only be consumed by the request-specific intrinsic capability
operation. That operation supplies the Function representation tag, invokes
the generic `NewPromiseCapability` implementation, and releases the tag and
constructor local in reverse reservation order. The general capability API
remains available for species and other arbitrary constructor inputs.

The constructor local is reserved before the factory's Realm and intrinsics
temporaries. Those temporaries are released intrinsics-first and Realm-second.
The consuming operation then reserves and releases its tag before releasing the
constructor proof. Rust emission errors are retained until those outer locals
have been released.

Entry bootstrap self-backs each request method's environment handle with its
own function identity before publishing it on `%AsyncGeneratorPrototype%`.
That identity is the call ABI's proof carrier for the factory above. Generic
builtin publication leaves the environment handle empty and is therefore not
valid for these three methods; a missing carrier traps at the factory's first
precondition rather than selecting another Realm.

## Request record

The 56-byte async-generator request record remains unchanged. Its existing
capability, Promise payload and Promise record edges retain all required
ownership. Adding a Realm slot would duplicate authority and permit the stored
capability and an unrelated Realm to disagree.

The bootstrap planner already roots `PromiseConstructor` and
`PromiseCapabilityExecutor` whenever any of the three request methods is
compiled. Removing the dispatcher global read does not remove that explicit
dependency.

## Focused evidence

The structure target pins the traced catalog slot, both bootstrap publications,
private child and absence of imports or re-exports, sole proof construction and
two consuming projections, reverse-order releases, planner dependency and the
absence of entry/current/activation Realm authority in the grouped request arm.

The source-equivalent extraction selected the exact 11-line proof block at
SHA-256
`ee77d3da82ce3ff12687a42d0ba048e6106f4b0274b275fee96600fd61284cda`,
54-line factory at SHA-256
`6d788c661b1e39cc835862440c3e8963fef107b1b416dae1bc87a3fa9e57ef23`
and 24-line consumer at SHA-256
`5d7d6497aee9052a765c8d9b558e86741ec9cbc114835efff31800332dc036f4`.
Their combined 89 selected lines retain SHA-256
`d8f7520291ba35c725b72ad5a36ebdd58565c1cf5e78b9c4eadfb6c94ef717dd`.
The 92-line child has SHA-256
`c83e3819f36ef479aefa13d18b040fe73349cab0af739337b3471450c8e75bd3`;
the concurrent Promise parent is 9,224 lines at this checkpoint. The recursive
owner target and both retargeted neighboring structures each pass `4/4`.

The finite CLI fixture invokes entry-defined `generator.next` through a
created-Realm bound Array method used as a Promise handler. It observes the
entry Promise prototype for valid and invalid requests and the entry TypeError
prototype for invalid receiver rejection. The fixture drains its finite chain
without polling, Atomics or `waitAsync`.

Created-Realm async-generator methods are not currently reachable from the
Wasm-AOT host surface because created-Realm generator/async/async-generator
function materialization remains unsupported. The source contract is therefore
load-bearing for created-method catalog selection; the runtime fixture proves
that a created Promise-job Realm cannot replace an entry-defined method Realm.

```sh
cargo test -p lila-aot-wasm --test async_generator_request_promise_realm_structure --quiet
cargo test -p lila-aot-wasm --test created_realm_promise_publication_structure --quiet
cargo test -p lila-aot-wasm --test async_execution_realm_structure --quiet
cargo test -p lila-cli --test cli run_wasm_backend_uses_async_function_realms_for_promises_and_reactions --quiet
```

The consolidated semantic golden passes `2/2` in 707.34 seconds and contains
664 dumps. Relative to the preceding checkpoint it adds only the Temporal
date-field-mode fixture and removes none. Of 663 retained dumps, 662 preserve
every non-accounting summary after emitted-byte normalization; the strengthened
async Realm fixture intentionally adds five internal and named functions for
its valid and invalid request paths.

## Shared extraction checkpoint

The existing finite CLI fixture passes `1/1` after the extraction, and the
shared `cargo xc`, formatting, diff, module-boundary and task-plan checkpoints
are green. The semantic golden remains deferred. Method and dispatcher bodies
are unchanged, so no new behavior or conformance claim is made for the
ownership move.

The nonescaping PromiseResolve functions and constructor selection are closed
independently by `promise-resolve-realm-context.md`. This boundary does not
change Promise allocation in Array.fromAsync or other async builtins,
async-generator iterator-result object ownership, dynamic created-Realm
async-generator source, or the request completion-kind and resume state domains
owned by their existing contracts.
