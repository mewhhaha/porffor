# Differential embedded module graph replay

## Current replay execution owner — 2026-10-04 dry source

Parent JSON loading and `DifferentialReplayInput` construction validate native
graph metadata without parsing JavaScript. Each backend's selected worker
receives the same immutable graph, and its attempt deadline covers graph
admission, Realm construction, entry parsing/compilation and execution.
Completed observations require a validated graph-bound case identity and
worker retirement. The seven paired graph cases retain their exact fixture
bytes and expected observations in the named-worker integration target.
Current compilation and execution remain pending. See the
[worker lifecycle](differential-worker-lifecycle.md).

Schema v4 carries one validated immutable embedded host-source graph. Its sole
factory accepts that owner; goal, source and filename are projections of the
entry. The wire contains no independently mutable outer goal/source/filename.
Exact referrer roles distinguish Script, Module and Unlocated even when source
locators coincide. Complete dependency source, independent metadata URL, request
attribute and resolution rows are decoded by the runtime graph constructor once.

Dependency rows now carry a closed optional `kind`: `source_text` (also the
meaning of an absent field) or `json`. Serialization omits the source-text
default and retains JSON explicitly. Decoding remints the actual typed graph
owner, including record-kind/request-attribute agreement. The graph fingerprint
binds kind and exact bytes even for an unused dependency; JSON cannot become
JavaScript when a case crosses the selected-worker wire. A roundtrip control
checks the retained kind, fingerprint drift and unknown-kind rejection.

The serializer excludes the implicit Module entry from dependency rows. Canonical
node/edge order and UTF-16 attribute-key order give a reversible projection. No
path, URL, missing-row, attribute or phase fallback is part of this policy. The
same Arc feeds actual Engine compile options and the paired spec oracle; computed
requests may select only declared rows. Phases retain their existing ECMAScript
loading/activation semantics and do not choose different host source bytes.

The complete cached graph SHA-256 covers even unused nodes/edges and independently
declared URLs. V4 case identity hashes this full digest into its versioned FNV
framing and also retains the entire digest as a graph-sha256 component. V4 mismatch
signatures retain that full case identity. Legacy v1/v2/v3 wire serializers,
fingerprint domains, mismatch signatures, dependency-sealed Script rejection and
arithmetic generation defaults remain on their existing branches.

V4 reuses v3's primitive completion plus captured ordered print comparison. Its
report schema is 4; equal observations establish only those dimensions. Whole
semantic equivalence remains not_established, unsupported Symbol/Object values
remain contract violations, and two engine failures remain BothFailed. The
off-by-default spec-exec oracle feature and explicit replay capability are kept;
the default product build does not gain the oracle.

Seven authored finite corpus files cover cyclic/shared Module identity and
metadata, separate Script/self-imported Module records, computed exact attribute
rows and promise freshness, undeclared requests and unused malformed sources,
dynamic-only parse rejection, deferred activation and JavaScript source-phase
rejection. Unit controls exercise graph-owner handoff, reversible root projection,
uncoupled wire denial, complete unused-policy drift and the v4 report/mismatch
domain. Paired controls require the real requested backends, exact primitive
completion and exact ordered output.

These are source implementations and authored controls. Compilation, parser
execution, generated campaigns, runtime, paired replay and conformance
verification have not run for this batch.
