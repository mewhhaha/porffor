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
