# Function prototype materialization policy

Status: implemented as a source-equivalent T09 invariant closure.

## Closed policy

`FunctionPrototypeMaterialization::{Automatic, BootstrapSupplied}` is the
complete allocation policy for a function object's own default `prototype`.
`Automatic` admits the existing constructable or generator prototype
allocation gate. `BootstrapSupplied` emits none because intrinsic bootstrap
owns the exact prototype object and descriptor publication.

The domain is `pub(crate)` only because bootstrap is a sibling module. It
derives no cloning, copying, debugging, equality or default capability and has
no manual implementation. Its exhaustive two-arm projection replaces the
former equality decision, so a future policy cannot silently inherit
bootstrap's no-allocation behavior.

## Producer and consumer boundary

The six producer sites remain fixed:

- ordinary function materialization and created-Realm builtin materialization
  select `Automatic`;
- the created-Realm Array constructor, hidden `%TypedArray%`,
  `%GeneratorFunction%`, and the shared `%AsyncFunction%` /
  `%AsyncGeneratorFunction%` constructor loop select `BootstrapSupplied`.

The single consumer retains the existing HTMLDDA exclusion and the exact
constructable-or-generator gate. Once admitted, prototype allocation, function
header stores, own-property publication and the non-generator constructor link
remain in their original order.

## Source equivalence and evidence

This closure changes no emitted instruction, local reservation, heap store,
descriptor flag or bootstrap order. The bounded structure guard recursively
pins all twelve production mentions, each of the six producers, the exhaustive
projection and the ordered allocation steps. Existing automatic and
bootstrap-supplied function-prototype CLI fixtures remain the focused runtime
witnesses. The structure target passes `4/4`; the automatic-prototype and
created-Realm bootstrap fixtures each pass `1/1`.

Independent review found that the first guard version ordered only broad
markers, so it was hardened to exact-normalize the complete allocation,
function-header stores, both property-publication rows and local-release
sequence. Final review is clean. The coordinated workspace checkpoint passes
`cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings.

This is not new function, constructor, Realm or class behavior and does not
close T09's broader call/construct and private-element acceptance criteria.
