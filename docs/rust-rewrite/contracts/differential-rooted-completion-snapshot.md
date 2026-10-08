# Native rooted completion snapshots

Status: 2026-10-07 source batch. The shared graph domain, Engine and Boa adapters,
v7 replay joins and controls are authored and typechecked. Pure protocol controls
pass. The first actual worker checkpoint rejects an incorrect fixture URL and
times out cold native compilation at the original 120-second deadline. The URL
now uses the admitted `lila://` scheme. `tasks-scope-native5` passes both original
Module and Script v7 worker controls under their unchanged 120-second deadlines.
The Script control retains post-job cycles, Symbols, Array holes and noninvoked
accessors through both actual backends. This selected run does not establish
full T25 acceptance or a cold-compilation performance guarantee.

Schema v7 observes the actual Script or Module completion after the backend's
job checkpoint. `Engine::observe_script_graph` and `observe_module_graph` take
the original source, compile/run options and checked `SnapshotLimits`; they
return `GraphRunOutcome` with backend identity, `SnapshotCompletion`, captured
output and a diagnostic note. Capture happens while the actual heap root is
retained. It invokes no JavaScript coercion, getter, proxy trap or reflection
wrapper, and does not change the source goal or wrap Script source in a function.

`SnapshotCompletion.kind` preserves Normal or Throw independently of its
`Captured { graph }` or `Rejected { reason }` outcome. A matching rejection is
red. Parse, early-error, loading, lowering and backend failures retain the
existing Engine-failure roles; process failure and incomplete journals retain
the existing worker-failure roles.

## Native input and authority

V7 requires `observation_contract: "rooted_completion_graph_print_transcript"`,
`host_profile: "test262"`, eight checked `snapshot_limits`, a nonzero deadline
and a tagged `program`. No case name, filename or source text grants host
authority. The Script form retains a normalized filename and exact source;
the embedded-graph form reuses the existing validated entry, SourceText/JSON
records and exact resolution rows. It carries no independently mutable goal,
filename or entry source. Script source closure is admitted inside the worker;
the parent-side native replay input never invokes a JavaScript parser.

```json
{
  "schema_version": 7,
  "id": "t25/rooted/example",
  "observation_contract": "rooted_completion_graph_print_transcript",
  "host_profile": "test262",
  "timeout_ms": 120000,
  "snapshot_limits": {
    "nodes": 512, "symbols": 256, "realms": 16, "properties": 4096,
    "utf16_units": 32768, "bigint_digits": 4096, "work": 262144, "depth": 64
  },
  "program": {
    "kind": "script",
    "filename": "differential/rooted-example.js",
    "source": "const root = Object.create(null); root.self = root; root;"
  }
}
```

The other program form is `{ "kind": "embedded_graph", "module_graph": ... }`
with the existing strict graph wire. Both forms roundtrip through the normal
CLI replay and corpus paths. The fingerprint binds the v7 host domain, every
limit, deadline, source and locator, plus the full embedded graph digest when
present. V1–v6 wire formats, authority, fingerprints and observations retain
their existing meanings. Their constructors refuse a v7 request without its
explicit budget owner.

## Graph contract

The shared `lila-runtime::rooted_snapshot` driver assigns canonical breadth-first
encounter identities. It records primitive values, original UTF-16 strings,
canonical numeric bits/BigInt values and Symbol description plus local,
registry or well-known origin. Object/Symbol aliases and cycles refer only to
graph-local IDs; backend addresses and allocation order are absent.

The initial object domain is ordinary Objects, Arrays, and ordinary/native
Function property graphs. Each node retains its actual extensibility,
prototype, ordered own keys and complete data/accessor descriptors. Accessor
get/set functions are edges and are never invoked. Array holes remain absent
keys and differ from present `undefined` values. Function nodes retain
constructability and actual callable Realm identity; capturing their property
graph does not establish equivalent function behavior or closure state.

Realm zero identifies the retained entry Realm. Foreign identities follow
their first observed encounter. Intrinsic anchors come from retained Realm
tables, not mutable names, `constructor` properties or prototype guesses. The
closed initial anchor vocabulary covers Object, Function, Array and the
ordinary Error constructor/prototype families listed by `SnapshotIntrinsic`.
Anchors annotate nodes while their own properties remain traversed, so mutated
intrinsic properties are retained. Other intrinsic identities and internal
slots remain declared observation gaps.

Budgets cover nodes, Symbols, Realms, properties, UTF-16 units, BigInt digits,
work and traversal depth. Constructors and deserialization reject zero or
above-ceiling limits. Backends charge retained copies and traversal work before
allocation; graph admission checks canonical IDs, reachable references,
descriptors, ordering and the retained limits. Unsupported exotics, exhausted
budgets and adapter/graph invariants produce explicit rejected observations,
even when both engines encounter the same boundary.

Native graph decoding threads one local aggregate hard-ceiling budget through
serde's map, sequence and enum visitors before retaining graph elements,
including internal tagged-enum buffers. It bounds total UTF-16 units, decimal
digits, properties, nodes, Symbols, anchors and decoding work, suppresses
untrusted allocation hints, and limits wire nesting. The requested limits are
checked again by canonical admission. The outer snapshot outcome streams its
graph instead of first buffering it in a tagged-enum intermediate. Input field
order remains arbitrary. Buffers already created by a caller-provided
deserializer remain outside this graph owner; the worker additionally caps
request and journal bytes. Shared `ALL` inventories are generated from the same
intrinsic/Symbol enum declarations consumed by both adapters.

## Worker, comparison and evidence

The selected worker calls only the graph observation APIs for v7. It checks
the actual backend and print hook transcript before publishing. Terminal
deserialization admits the shared validated graph; the supervisor additionally
requires its exact limits to equal the request's limits. A legacy primitive or
unit-label completion cannot satisfy a v7 terminal. Earlier protocols reject
graph terminals and their preserving reducers cannot acquire graph witnesses.

A green result requires identical captured completion kind, canonical graph
and ordered print transcript. A captured difference produces a stable v7
mismatch signature; diagnostic notes and backend handles are excluded. Any
snapshot rejection stays `observation_contract_violated`, with no semantic
mismatch signature. Both Engine failures remain red. Reports expose the host
profile and limits and retain `semantic_equivalence: "not_established"`.

Pure controls live in `differential::rooted_snapshot::tests` and the worker's
`rooted_journal_requires_the_requested_limits_and_native_result_domain` control.
They cover exact native replay, every fingerprinted limit, typed embedded graph
ownership, descriptor/kind/output differences, red matching rejections,
malformed graphs, foreign terminal budgets and old-protocol exclusion.
The shared `rooted_snapshot::wire_bounds::tests` add aggregate-copy limits,
late tag/limit ordering and a three-element decoder tripwire that fails before
decoding the third value, without allocating a large test payload.

The `differential_rooted_snapshot_execution` target has two actual selected-worker
programs: a post-job cyclic Object with Symbol origins/aliases, Array holes,
signed zero, a noninvoked accessor and native Function property graph; and a
top-level-await Module whose original completion and output are retained. These
new inputs explicitly use 120,000 ms deadlines. The first script worker exceeds
that deadline before native output, while Boa completes its graph. The Module
fixture initially failed admission because it used `file://`; the written
repair uses `lila://`. Both await affected execution verification. Existing
five-second inputs and scenario deadlines are unchanged. No synthetic comparison
control claims native execution.
