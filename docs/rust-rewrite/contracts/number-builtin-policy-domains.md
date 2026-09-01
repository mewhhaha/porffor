# Number builtin policy domains

Status: implemented and focused-verified for the capability-free Number
dispatch domains.

## Closed domains

`NumberBuiltin` is the exact eleven-member input to the Number builtin
dispatcher: the constructor, four static predicates and six prototype methods.
`NumberPrototypeOperation` is the restricted six-member input to shared
branded-receiver extraction. Both are capability-free: neither can be cloned,
copied, compared, hashed or formatted. Each selection must move through its
single exhaustive consumer instead of being retained or forked into another
policy decision.

The standard-builtin dispatcher constructs every `NumberBuiltin` variant once.
One exhaustive match selects each complete top-level algorithm. Its six
prototype arms construct each restricted operation once, and a second
exhaustive match selects the complete result algorithm after receiver
extraction. There is no Boolean policy carrier, wildcard arm, debug assertion
or unreachable invalid-operation path at either boundary.

The constructor's argument-presence local and the four predicates' Boolean
results are runtime ECMAScript state, not compiler policy. They remain outside
this invariant.

## Durable regression

`number_builtin_policy_domains_structure.rs` owns both exact variant sets, all
eleven read-only `standard.rs` producers, the exhaustive eleven-to-six routing,
the exhaustive restricted consumer and the absence of equality/default escape
hatches.

The existing `wasm_number_builtin_family.js` fixture is the finite runtime
witness. It covers call and construct behavior, all four predicates, all six
prototype operations, boxed receiver extraction and incompatible receiver
rejection. The structure target pins its exact CLI registration, so this
closure needs no duplicate fixture.

```sh
cargo test -p lila-aot-wasm --test number_builtin_policy_domains_structure --quiet
cargo test -p lila-cli --test cli language_numerics::run_wasm_backend_succeeds_for_number_builtin_family_fixture -- --exact --test-threads=1
```

This capability closure changes no Number semantics or generated Wasm. It does
not claim complete Number formatting, ECMA-402 locale behavior or the full
pinned Number tree.

On 2026-08-26, the structure target passed `4/4` while the exact CLI target
failed at its then-unimplemented aggregate Number prototype formatting
assertion. Restoring only the two original `Debug, PartialEq, Eq` derive lists
failed at the same assertion, establishing that the failure was independent of
this capability-only change. The later shared decimal-formatting repair closed
that semantic debt. Its coordinated 679-dump semantic golden passed `2/2` in
800.46 seconds with no retained structural change attributed to the Number
domain closure.

Batch AI removes the remaining `Clone` and `Copy` capabilities without changing
either owned parameter, exhaustive match or producer. The recursive structure
guard now pins the attribute-free declarations, absence of manual capability
implementations and exact 25/14 product mention census. Shared `cargo xc`
passes, the structure target passes `4/4`, and the existing aggregate runtime
witness passes `1/1`. This source-equivalent closure needs no Test262 cohort or
semantic golden and claims no new Number behavior. Final formatter, diff,
module-boundary, task-plan and 240-entry shortcut-inventory gates are green.

Batch AT makes the outer family a private `NumberBuiltin` and the raw emitter
private to `number.rs`. Standard dispatch can reach the family only through
eleven fixed Number entries and can neither construct nor pass its raw policy.
The frozen 160-line domain/emitter selection has SHA-256
`7465f52181186c7cd1dd4bb2be3fa2a124ac6794fe509c4d7a0e003984091e9a`.
Restoring the former enum/emitter visibility and constructor worker name
reproduces that source exactly. At the 2026-08-28 Batch AT checkpoint,
`cargo xc` is green, the strengthened structure target passes `4/4`, and the aggregate
Number runtime witness passes `1/1`. No Test262 leaf or semantic golden was
required for this dispatcher-only closure. This source-equivalent boundary
claims no new Number behavior, broader conformance or published
conformance-count change.
