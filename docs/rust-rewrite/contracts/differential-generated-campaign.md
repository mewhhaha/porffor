# Deterministic generated campaigns

`lila differential campaign` consumes a checked nonwrapping sequence of 1–128
seeds, one of the actual V1/V2/V3 arithmetic grammars, `object-probe-v1`,
`object-mutations-v2`,
`module-graph-v1`, `module-graph-v2`, `control-flow-v1`, `negative-source-v1`,
`builtin-stateful-v1/v2` or `metamorphic-stateful-v1/v2`, bounded
grammar-specific generation inputs and the existing reduction budget. Its library
owner constructs the checked generator plans in seed order; it calls the same
selected-worker replay and each grammar's preserving reducer. The CLI requires the
explicit oracle and the oracle build feature. It defaults to V3, while the old
single-case command retains V2 and its original bytes/draw order.

A new output directory begins with an incomplete aggregate containing the
entire seed inventory and current compiler identity. Every initial and reduced
candidate writes its exact request before replay, then its actual observation
or rejection in `evidence/`. Files are create-new and synchronized; aggregate
updates use synchronized temporary files and atomic rename. A failed write or
process interruption cannot publish an all-matched aggregate. Only verified
or reduced mismatches enter `corpus/`, which the existing corpus replay consumes.
Rejected/shared/worker failures remain red evidence and do not become accepted
corpus cases. Worker failure stops the campaign with unattempted entries pending.

The generated-campaign integration tests retain any created output directory
when an assertion panics and report its path through best-effort stderr output.
Ordinary successful cleanup still removes it. This changes only failure evidence
retention; generated inputs, deadlines, assertions and campaign verdicts are unchanged.

With `LILA_WASM_TRACE` set, ordinary differential replay relays each retired
worker's already bounded stderr to the parent log with backend, case fingerprint
and evidence-completeness metadata. Writes are best effort and occur after the
original worker deadline and cleanup; diagnostics do not enter JavaScript print
observations, report comparison or verdicts. The existing Engine phase trace can
therefore locate an interrupted compilation without extending its budget.

The embedder cancellation token is checked before every initial/reduced replay.
An active bounded worker pair finishes the original retirement first. Cancellation
retains earlier attempt evidence, marks the active case cancelled and leaves the
remaining inventory pending. It never turns a partial campaign green. The CLI
returns success only after every case satisfies its typed observation contract;
semantic equivalence stays `not_established`. Negative-source cases explicitly
check their expected frontend phases while retaining the original `both_failed`
reports. They remain in evidence, outside the executable corpus. Metamorphic
cases retain both exact programs and reports plus the actual within-backend
relation; their checked pair artifact has a dedicated replay command.

Meaningful authored controls cover nonwrapping identity, real worker replay and
source-dependent fingerprints, fresh-output refusal, cancellation and a worker
which cannot publish its bound header. They remain unrun. The object grammar
adds actual descriptors, aliases/cycles and graph-preserving reduction through the
existing v5 observation. See [its contract](differential-generated-objects.md).
The V2 object grammar adds ordered mutations, receiver identities and actual
operation results under the same observer and preserving reducer; see the
[mutation contract](differential-object-mutations.md).
The module grammar adds complete exact v4 graphs and graph-preserving reduction;
see [its contract](differential-generated-modules.md). The V2 module grammar
adds actual dynamic imports, exact attributes and top-level Await through the
same complete graph owner. The control-flow grammar adds checked statement trees,
four actual function protocols, finalizers, labelled control and captured cells
under the v3 observer; see [its contract](differential-generated-control-flow.md).
The negative-source grammar retains actual frontend rejection provenance and
phase-preserving reduction; see [its contract](differential-generated-negative-source.md).
Stateful builtin scenarios and checked metamorphic transformations retain actual
coercion, iterator, species and job traces. Their v2 grammars add real cross-Realm
builtin observations under explicit v6 Test262 host authority and bounded ISO/fixed-offset
Temporal value observations under product v3; see
[their contract](differential-generated-scenarios.md).
Arbitrary mismatch preservation, broader AST algebra, subsystem
performance budgets and full T25 acceptance remain separate work.

The existing differential CI job now runs all eleven grammar families serially.
PR/default runs use two seeds per grammar with sixteen reduction attempts;
the two stateful rows use v2 seeds 6 and 7 to reach Realm and Temporal explicitly,
while other grammar rows retain initial seed 1.
nightly and reusable release validation use sixty-four seeds and attempts.
Nightly/release also build the optimized developer oracle and repeat the whole
corpus and grammar campaigns. Every build, test and campaign payload passes
through the confirmed one-CPU, 4096-MiB, zero-swap launcher. A failed grammar
does not suppress the other independent campaign evidence, and the runner
returns failure if any grammar fails. Artifact upload retains all complete/red
directories. The pinned release-closure job depends on this reusable validation
before its product compiler and two full pinned runs. These workflow changes
are authored and unexecuted; they do not establish sustained-campaign acceptance.
