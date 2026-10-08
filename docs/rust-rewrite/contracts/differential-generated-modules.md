# Generated complete module graphs

`module-graph-v1` is a checked source algebra consumed by
`lila differential campaign --grammar module-graph-v1 --modules N --edges N`.
It admits 1–16 declared Modules and between `modules-1` and
`min(64, modules²)` exact import edges. A deterministic spanning star makes every
source reachable from the Module entry; additional unique edges can form self
imports and arbitrary cycles. Every edge selects named or namespace imports,
optional reexports and an entry-side live-binding increment.

Module source owns mutable numeric exports, functions which read those live cells,
ordinary module initialization and optional `import.meta.url` trace events.
Dependency initialization never reads a cyclic lexical export. The entry reads
and mutates imports only after the static dependency evaluation completes. This
bounded algebra exercises cycles without introducing accidental initialization
TDZ reads or unbounded recursion. Numbers remain exact small integers.

`module-graph-v2` uses the same source, graph, replay and reduction owners. Its
closed edge kind selects static imports or dynamic imports with literal/computed
specifiers and absent/blue/red exact attributes. Modules can await a resolved
Promise before requesting dependencies; dynamic edges await the original import
Promise and print the completed namespace value. The entry then mutates and
reads the same exported live cells. Every nontrivial initial V2 graph contains a
computed attributed import; the reducer can simplify those independent features.

V2 admission requires every target to follow its referrer in the checked module
order, across both static and dynamic edges. This prevents a generated top-level
Await from waiting on its own or an ancestor's unfinished evaluation. Its maximum
is `min(64, modules × (modules-1) / 2)` edges. Pruning/reindexing preserves the
order. V1 retains its static cycle domain and original deterministic draw/source
order. Each grammar's private source constructor rejects the other's illegal
states before any source or loader rows can be published.

Rendering constructs the existing `EmbeddedModuleGraph` with the actual entry,
all dependency source/metadata bytes and exact module-origin resolution rows.
Every generated request has one declared target; a reexport shares its existing
edge rather than manufacturing a duplicate resolution row. Native graph validation
is the only loader authority. Both real workers consume this same immutable owner
through unchanged schema v4; ambient filesystem loading remains excluded.

Reduction removes import edges, prunes newly unreachable dependency modules,
reindexes actual targets and shrinks literals, metadata, namespace/reexport and
increment features. Original retained source identities and metadata URLs survive
pruning. Every candidate regenerates its complete source and resolution rows and
passes through the same constructor and `EmbeddedModuleGraph` admission. No row
is removed independently of the source request that it serves. Complexity decreases
strictly; grammar, seed, original budgets, timeout and protocol remain unchanged.
V2 also removes explicit awaits and attributes, replaces computed specifiers by
literals and simplifies dynamic requests into static namespace imports. The
selected source and its exact attribute row always change together. A plan for
another grammar cannot label the reduced graph.

Replay retains completion/print difference dimensions and the two completion
kinds, or the actual failing backend and phase. Source-dependent full graph
fingerprints and mismatch signatures belong to each evidence record and change
lawfully during reduction. Worker/shared/observation failures are never replacement
witnesses or accepted corpus cases. The existing campaign handles cancellation,
incomplete truth, output ownership and the explicit selected oracle.

Authored controls cover foreign/duplicate targets, cycles, pruning/reindexing,
exact source/resolution closure, deterministic wire round trips, fingerprint
changes with a retained print mismatch, worker-failure interruption, CLI budget
admission and actual selected-worker replay, including attributed dynamic imports
and top-level Await. They remain unrun. Import phase generation, arbitrary statement mutation, parse/error
phase generation and full T25 acceptance remain separate grammar work.
