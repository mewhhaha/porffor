# Proxy trap signature capability

`ProxyTrapSignature` is the private eight-state semantic argument-record
domain for the thirteen ECMA-262 Proxy traps. It is an authority consumed by
lowering, not a value that may be cloned, copied, compared or formatted.

## Producer boundary

The single `proxy_traps!` table owns the complete mapping:

- `getPrototypeOf`, `isExtensible`, `preventExtensions` and `ownKeys` receive
  only the target;
- `getOwnPropertyDescriptor`, `has` and `deleteProperty` receive target and
  property key;
- `setPrototypeOf`, `defineProperty`, `get`, `set`, `apply` and `construct`
  each select their distinct ordered semantic record.

The macro-generated `ProxyTrap::signature` projection is exhaustive. Adding a
trap or a signature therefore requires an explicit table row and an explicit
consumer arm.

## Consumer boundary

`proxy_trap_argument_infos` is the sole consumer. Its exhaustive match emits
the exact ordered argument facts for all eight records, including distinct
receiver, value, descriptor, prototype, this-argument, arguments-list and
new-target positions. There is no wildcard, equality test, Boolean policy or
default record.

The structure guard recursively fixes the twelve production mentions, exact
private declaration without derived or manual capabilities, complete ordered
thirteen-row table, generated projection and all eight ordered argument-vector
bodies. Its lexical route census admits exactly the one immediate method call
bound to that projection, including raw-identifier and empty-turbofish Rust
spellings. The dedicated structure target passes `3/3`. The existing
Set-focused omitted-formals unit passes `1/1`, showing that the four supplied
Set operands still join an omitted fifth formal with `undefined`. Independent
dry review is clean. The shared `cargo fmt --all -- --check`, `cargo xc`, diff,
module-boundary and task-plan checkpoint is green with the workspace's existing
warnings.

## Source equivalence and nonclaims

The production change removes only unused derives. The signature is freshly
created and immediately consumed once, so no emitted IR, argument fact,
evaluation order or runtime Proxy behavior changes.

This capability closure is not additional Proxy or Reflect conformance. It
does not complete trap lookup, invocation, fallback, invariant validation,
nested Proxy behavior or the remaining T11 Test262 trees.
