# Logical-assignment reachability authority

`LogicalAssignmentReachability::{Definite, WithEnvironmentFallback}` is the
lowering-private authority for metadata after an identifier logical assignment
has either a definite Reference or a conditional fallback from a selected
`with` environment.

The domain derives no cloning, copying, debugging, equality, ordering, hashing
or default-construction capability. Its sole consumer borrows it in four
exhaustive decisions: conditional-global fallback routing, lhs metadata,
non-constant write metadata and final expression metadata. The definite row
retains proven information; the fallback row invalidates conditional global
facts and uses unknown runtime information. Adding a row therefore requires an
explicit decision at every semantic projection.

Exactly two assignment-lowering paths construct the domain. The selected
`with` chain produces `WithEnvironmentFallback`; the direct located Reference
produces `Definite`. The recursive structure guard pins the complete source
census, both producer contexts, all four exhaustive decisions and the absence
of equality, wildcard and default policy.

This closure is source-equivalent. It changes no Reference selection, RHS
order, short-circuit behavior, write placement, IR shape or metadata result.

Focused verification:

```console
cargo test -p lila-ir --test logical_assignment_reachability_structure
cargo test -p lila-ir object_environment_logical_assignment
cargo test -p lila-aot-wasm --test object_environment_logical_assignment_structure
```

The new structure target passes `3/3`, the focused lowering units pass `2/2`,
and the neighboring backend structure target passes `4/4`. Independent review
confirmed the complete capability and mention census, producer contexts, exact
branch bodies, preserved source order and the narrowly necessary neighboring
guard locator update. The coordinated `cargo xc`, full formatter, diff,
module-boundary and task-plan checks are green. Broad conformance verification
remains deferred.
