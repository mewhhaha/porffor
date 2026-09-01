# Test262 execution identity

Test262 counts executions, not source files. A source without an execution-mode
flag is two tests: its sloppy Script execution and its strict Script execution.
Every durable and in-memory identity below therefore has the form

```text
(physical test path, execution mode)
```

The physical path remains available for discovery filters, source-specific
diagnostics, ownership rules, module roots, and temporary file names. It is not
an execution identity and must never be used alone for completion, failure,
sharding, matrix, resume, journal, comparison, or backlog joins.

## Closed mode law

`TestExecutionMode` is the only parse-goal/strictness authority:

Discovery first constructs the private, capability-free
`TestExecutionPlan::{One, SloppyAndStrict}` domain. `One` owns an exact mode;
`SloppyAndStrict` owns the only two-execution expansion and emits sloppy before
strict. The plan is consumed once through an exhaustive match, so a future plan
cannot inherit an empty, one-mode or two-mode fallback. Its flag parser also
matches all six valid mode combinations explicitly after rejecting conflicting
strictness flags.

```console
cargo test -p lila-test262 --test execution_identity_structure
cargo test -p lila-test262 tests::execution_plan_exhaustively_maps_valid_flags_and_rejects_conflicts -- --exact --test-threads=1
```

The execution-identity structure target passes `4/4`, and the exact flag-plan
unit witness passes `1/1`. Independent review confirmed the private capability
closure, complete ownership census, exact flag bindings and tuple order, all six
rows and sloppy-before-strict projection. Coordinated workspace verification
passes `cargo fmt --all -- --check`, `cargo xc`, `git diff --check`, the module
boundary check and the task-plan check; the compile retains the repository's
existing warnings.

| Frontmatter flags | execution plan |
| --- | --- |
| none of the mode flags | `sloppy-script`, then `strict-script` |
| `onlyStrict` | `strict-script` |
| `noStrict` | `sloppy-script` |
| `raw` | `raw-script` |
| `module` | `module` |
| `raw`, `module` | `raw-module` |

The following combinations are invalid suite data and stop discovery:

- `onlyStrict` with `noStrict`;
- `raw` with either `onlyStrict` or `noStrict`;
- `module` with either `onlyStrict` or `noStrict`.

Module goal is derived exhaustively from the mode. There is no parallel
`is_module` bit. Strict Script materialization starts with the exact directive
`"use strict";\n`, including when a Wasm-AOT self-contained rewrite supplies
the remaining body. Raw modes are byte-identical to the vendored source: no
directive, harness prelude, or source rewrite is permitted.

## Discovery and source ownership

Discovery reads and parses each physical file once, validates its mode flags
once, and expands the closed plan into one or two `TestCase` executions. The
public `materialize_test` operation remains one execution in and one
materialized execution out. Sibling sloppy and strict executions share the
same immutable `Arc<str>` source allocation and, when present, the same
`Arc<NegativeExpectation>` metadata allocation; expansion does not duplicate
either payload.

An ordinary selector is a physical path or directory prefix and therefore may
not contain `:`. A selector containing `:` is an execution-id selector and must
parse as `<known-mode>:<non-empty-physical-path>`; malformed or unknown mode
prefixes are errors, never path fallbacks. An exact execution selector must
resolve to exactly one discovered execution.

## Durable identity

`TestExecutionId` is carried without loss through:

- manifest and matrix hashes;
- matrix leaves and deterministic sharding;
- resume checkpoints and completed sets;
- the pre-attempt crash journal;
- forced child-runner selection;
- results, failures, timeouts, and slow-case records;
- snapshot comparison and generated backlog joins.

Consequently one completed sibling can never suppress the other, and one
sibling's crash strike cannot quarantine its peer.

Every direct or sharded run hashes the exact selected execution set into its
manifest. Its checkpoint and attempt journal therefore belong to that set, not
to the unsharded discovery manifest. A partial checkpoint contains a unique
subset of the selected ids; a complete checkpoint or terminal direct-run
snapshot contains the exact set. The checkpoint identity also records the
caller's intended canonical terminal `run_kind` and exact matrix path. The
on-disk `resume-case-checkpoint` marker is never an identity by itself: its
paired checkpoint identity must match the receiving full, exact shard, or
matrix-leaf context. Ordinary terminal snapshots must omit checkpoint identity
and match their own canonical run kind and matrix path. Wire presence is
significant: legacy snapshots and terminal version-7 snapshots must omit the
`checkpoint_identity` key, while a version-7 `resume-case-checkpoint` must
contain a non-null canonical object. Explicit `null` is never absence. Failure,
timeout, and slow-case records must name unique completed ids, and a single-case
child snapshot must name exactly the requested id everywhere. Matrix-node
evidence is validated against the exact `case_ids`, recorded-pin manifest hash,
and actual aggregate entry before it is written or promoted and again whenever
it is loaded. A coherent strict-subset checkpoint remains resumable but is not
promotable as a completed node; malformed or foreign evidence is a hard error.
The top-level low-RAM resume flow may treat only a stale run envelope as absent
across reconstruction, the pending-node rescan, and the direct checkpoint load
immediately before execution: a different known backend, matrix strategy,
ECMA-262 pin, or non-content-equivalent Test262 pin. Every direct full, shard,
public matrix-node, child, and non-low-RAM load requires an exact envelope. An
unknown backend spelling is schema corruption, never stale evidence. Once an
envelope is accepted, filename/body hash disagreement, recorded-pin manifest
recomputation failure, and case-evidence corruption are hard integrity errors
rather than missing work.

### Case-set admission ownership

The private, capability-free
`CaseSetRequirement::{UniqueSubset, Exact}` authority carries that distinction
from each snapshot producer into the sole case-evidence validator. The
validator consumes it once in an exhaustive match that jointly selects exact
set enforcement and the corresponding `subset` or `exact copy` diagnostic.
The policy therefore cannot be copied into a second decision, and a future
variant cannot inherit either admission behavior or wording by default.

Removing the former equality observation reduces the exact ownership census
from 18 to 17 source mentions and from 14 to 13 production mentions. The 13
production mentions comprise the declaration, three typed signatures, seven
constructors and the two exhaustive projection arms. The seven constructors
are five `Exact` and two `UniqueSubset` rows delivered through six contexts:
completed shard, completed full run, resume-node identity, periodic resume
checkpoint, single-case child and completed matrix node. The change preserves
the existing set comparisons, error text and validation order.

```console
cargo test -p lila-test262 --test case_set_requirement_structure -- --test-threads=1
cargo test -p lila-test262 --lib tests::direct_terminal_and_checkpoint_identity_are_exactly_bound -- --exact --test-threads=1
cargo test -p lila-test262 --lib tests::case_evidence_contract_rejects_nested_failure_timeout_and_slow_id_drift -- --exact --test-threads=1
```

The focused structure target passes `4/4`, and both exact ownership and
case-evidence witnesses pass `1/1`. This is a source-equivalent accounting
invariant; it neither establishes complete aggregate evidence nor advances the
T26 release gate.

Attempt-journal schema 3 additionally requires unique worker slots, unique
in-flight execution ids, non-zero strike counts, execution-id wire keys, the
closed execution backend, the selected manifest hash, and an exact unique copy
of the selected execution set. In-flight and strike ids must belong to that
set. Any older, mixed, malformed, duplicate, foreign, or unreadable state is
discarded loudly and starts fresh; it never acquires a guessed mode or crosses
a backend.

Snapshot schema 7, matrix strategy 3, and attempt-journal schema 3 are the
first formats with execution identity. Snapshot execution backends are a
closed wire domain; an unknown spelling is invalid. Older envelopes may remain
readable only in explicitly metadata-only progress/report workflows. Both old
path fields and new typed case-evidence fields are invalid in a legacy v4-v6
artifact, even when the field is `[]` or `null`: mixed-version evidence is not
an upgrade path. Version 4 has no outcome counts, so an outcome-bearing progress
report refuses it rather than reconstructing outcomes from node records.
Resume, merge, complete verification, failure details, comparison, backlog,
publication, and matrix-node promotion require current version-7 aggregate and
node evidence. Legacy artifacts are never rewritten, promoted, compared to
current results, or reinterpreted as either execution mode.
