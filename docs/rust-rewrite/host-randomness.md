# Host randomness contract

`Math.random` is an ECMAScript host hook, not a unary numeric function and not
a compile-time constant. Lila represents that fact at three boundaries.

## Runtime domain

`RandomUnitInterval` is the only value a host randomness provider can return to
JavaScript. Its private `f64` is finite and satisfies `0 <= value < 1`.
`HostRandom` is `Send + Sync` and returns that type or a typed host failure.
It is a realm capability separate from output hooks and clocks: replacing a
test printer must not replace entropy, and a deterministic clock must not make
randomness deterministic by accident. Realm clones and agent workers retain
the same provider.

The production provider obtains a fresh `u64` from the operating system and
uses its high 53 bits as a binary64 fraction. Every one of the `2^53` exactly
representable grid values in `[0, 1)` therefore has one source word bucket.
Failure to obtain host entropy is an engine failure; it is never represented as
a JavaScript number outside the closed domain.

## Artifact boundary

The standard-builtin catalog marks the sole random reader. Wasm codegen derives
an optional `lila_host.random_f64: () -> f64` import from that flag and gives
the resulting typed function index only to the Math emitter. The import is
appended after existing optional host calls, so adding it cannot renumber an
older ABI row. A module that does not retain `Math.random` has no randomness
import.

The engine binds the import to the current realm's `HostRandom`. A host value
has already crossed `RandomUnitInterval`, so the linker does not perform a
second ad-hoc range check.

## Observable ordering

`Math.random` has arity zero and ignores every supplied argument. Its emitter
must call the host directly: it never loads, coerces or invokes hooks on an
argument. This differs from every unary Math operation and is an exhaustive
`MathBuiltin::Random` arm, not a boolean exception inside the unary path.

## Durable evidence

- a runtime contract pins construction bounds and proves cloned realms share
  an injected provider;
- an AOT structural contract proves the import is present exactly when the
  retained builtin requires it;
- an engine contract injects a deterministic sequence, verifies exact values,
  and gives `Math.random` an argument whose coercion would throw if observed.

This contract does not prescribe a reproducible production seed or a
cryptographic JavaScript API. It supplies the implementation-defined,
approximately uniform source required by `Math.random` and keeps its host
capability explicit.
