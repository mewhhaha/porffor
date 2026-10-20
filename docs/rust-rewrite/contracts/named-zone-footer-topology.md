# Shared named-zone footer topology

Every distinct TZif payload retains its own structural validation, complete
historical offset catalogue, explicit-transition proof and table/footer handoff
proof. Catalogue identifiers, actual record bytes and their digests are checked
before a payload is admitted.

One catalogue construction may share an immutable footer certificate only when
the complete parsed `PosixTzString` compares equal. The private footer projection
has no explicit transitions and retains a present footer. The pinned forward
selector therefore returns only that footer's standard/daylight offsets. These
offsets form a complete inverse catalogue for this projection; historical local
types remain in the actual payload's larger catalogue.

Each new footer certificate runs the original complete Gregorian-cycle and
relevant remote-phase proofs through the same forward selector. It is published
only after all checks succeed. Its cycle remains inside the retained certificate,
reachable through the actual record's `GapTopology`. Sharing a footer cannot
substitute for another record's historical or handoff proof.

The construction cache is local to the checked catalogue, whose input domain is
bounded by its record count. It introduces no process-wide mutable data cache or
change to the verifier's module-cache capacities, worker counts or deadlines.
The source identity includes nested production modules. Native verification,
component timing, exact frame/identity comparison and original worker budgets
are required before accepting a performance or timeout-repair claim.
