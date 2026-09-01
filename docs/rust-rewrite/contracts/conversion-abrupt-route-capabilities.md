# Conversion abrupt-route capabilities

## Closed ownership domains

The crate-visible `ToPrimitiveAbruptRoute`, `PrimitiveToStringAbruptRoute` and
`ToLengthAbruptRoute` domains derive no cloning, copying, debugging or equality
capability. Each conversion call constructs one named route and moves it into
the corresponding emitter. One private finisher consumes that route in an
exhaustive match immediately after the operation can throw.

The route itself cannot be duplicated or inspected into a parallel authority.
`IteratorCloseOnThrowLocals` remains independently copyable because the same
prepared iterator locals legitimately participate in distinct ToPrimitive and
ToString operations; each operation still constructs and consumes a separate
route value.

## Preserved behavior

Removing the incidental route capabilities changes no variant, caller,
projection or emitted Wasm. The three exhaustive finishers retain their exact
policies:

- ToPrimitive routes to an active handler, returns the current function, or
  closes an iterator and returns;
- primitive ToString owns the same three destinations for its Symbol error;
  and
- exceptional ToLength either routes to the active handler or rejects and
  returns the existing Array.fromAsync promise capability.

The focused structure regression pins the exact domains, rejects derived and
manual incidental capabilities across the source tree, and requires each route
to move into one exhaustive private finisher without a wildcard.

```sh
cargo test -p lila-aot-wasm --test conversion_abrupt_route_capability_structure
cargo test -p lila-aot-wasm --test conversion_error_realm_source_structure
cargo xc
git diff --check
```

The focused capability target passes `2/2`. The neighboring conversion-Realm
target passes `4/4` after its stale primitive-ToNumber marker was aligned with
the already-reviewed borrowed non-`Copy` route projection. The shared workspace
`cargo xc` checkpoint is green.

## Nonclaims

This ownership closure does not add a completion route, migrate another
operation, change iterator closing, alter error-Realm selection or claim broad
ECMAScript conformance progress.
