# `for await` identifier assignment heads

## Contract

`for await (identifier of iterable)` evaluates `identifier` as an assignment
target on every entered iteration. It does not declare an iteration binding.
The element value must therefore reach the same checked identifier Reference
write used by an ordinary assignment:

- a mutable resolved binding is updated;
- an immutable resolved binding throws `TypeError` after the element value has
  been obtained;
- a `with` object environment is selected ahead of the declarative/global
  fallback; and
- an unresolvable reference retains the ordinary strict/sloppy global-write
  policy.

`for await (var identifier of iterable)`, `let`, and `const` are declarations
and keep their existing binding and per-iteration-environment behavior.

## Lowering invariant

`ForOfBareIdentifierHead` is the private closed distinction between no bare
assignment target and `AssignmentTarget { source_name }`. The assignment form
uses a fresh synthetic `let` head slot. The iterator machinery writes each
element into that slot, then the loop body prefix consumes the slot through
`locate_identifier_reference`, with-environment selection, and
`lower_located_identifier_assign_value`.

The source identifier is never passed to the loop head's unconditional
`declare_binding`. Consequently it cannot shadow the binding that the source
Reference is required to resolve. The `var` AST arm remains distinct and keeps
its declared source name. The capture analyzer records a bare identifier head
as a Reference independently of body reads, so a nested async function owns the
correct outer-environment capture even when the loop is its only use.

## Regression evidence

`lila-ir` has three focused IR assertions: a mutable assignment head must
contain an `AssignIdentifier` from the synthetic slot into the existing lexical
storage; a write-only nested function must capture its outer target; and an
immutable target must contain the runtime `TypeError` while a `let` declaration
retains its own `$forof.lex.*` storage. The bounded structure guard pins the
closed domain, distinct bare/`var` AST arms, temporary allocation, both checked
Reference-write branches, and write-only capture registration.

The CLI fixture covers four observable cases in one async job:

- the pinned Test262 spelling `async` changes from `0` to the final element
  `7`, and the assigning function never reads it;
- assigning to an outer `const` throws `TypeError` and does not mutate it;
- a `let` loop declaration shadows without changing its outer namesake; and
- a `var` loop declaration still updates its declared binding.

The exact pinned leaf is
`language/statements/for-await-of/head-lhs-async.js`. This seam does not claim
completion of other assignment-target forms, iterator closing, async iterator
helpers, or the rest of T15.

## Verification

On 2026-08-27:

- `cargo test -p lila-ir --test for_await_identifier_assignment_head_structure
  --quiet` passed `4/4`;
- `cargo test -p lila-ir for_await_identifier_ --quiet` passed `3/3`;
- the focused CLI test passed `1/1`;
- the exact pinned Test262 leaf passed both sloppy/strict Wasm-AOT variants,
  `2/2`, with every failure bucket at zero;
- fixture `node --check`, `cargo xc`, formatting, diff hygiene, task-plan,
  module-boundary and shortcut-inventory gates passed; and
- the shared semantic golden passed `2/2` in 685.75 seconds with 682 dumps,
  adding this fixture and the independent created-Realm WeakRef witness,
  removing none and leaving all 680 retained dumps byte-identical.

No broad T15 filter was run for this focused seam.
