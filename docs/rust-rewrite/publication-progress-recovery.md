# Durable publication progress and restart recovery

The low-RAM Wasm-AOT publication driver now retains its last accepted matrix
observation in the publication manifest. This addresses the restart boundary of
T01's reproducible full-suite workflow; it does not close T01 or T26, add compiler
semantics, or establish a new Test262 result.

## Why process-local checks were insufficient

Previously, `MATRIX_COMPLETED`, `MATRIX_TOTAL`, and `REPORT_RAN` were initialized
again on each invocation. A resumed family could therefore accept a lower
completed-node count, accept a different total, or treat an unavailable checkpoint
as permission to bootstrap despite already having results. The within-process
checks did not protect observations made by an earlier supervisor.

Three end-to-end regressions exercise those cases against the original scripts.
They fail because the old driver resumes and eventually calls `publish-status`.
The patched driver stops before calling either `report-all` or `publish-status`.
These are control-flow findings, not proof that the Rust publisher would accept
invalid test artifacts; its independent validation remains necessary.

## Manifest contract

Schema 2 retains the existing source, executable, suite and configuration
identity. The identity is still immutable for a snapshot family. A new `progress`
field is initially `null`, then holds an exact `{completed, total}` record.

Every successful `progress-status` response is parsed and checked before the
driver decides to resume or publish. Counts must be canonical nonnegative decimal
integers of at most 18 digits, with a positive total and `completed <= total`.
Missing or duplicate fields, boolean JSON counts and malformed records fail.

After the first observation, the total cannot change and the completed count
cannot decrease, including after a restart. Equal observations are accepted
before running a new report, allowing ordinary resume and republication. After a
successful `report-all`, the count must strictly increase. Reaching the total
means that all matrix nodes were processed, NOT that every test passed.

The updated identity-plus-progress document is written to a temporary file,
flushed and fsynced, atomically replaced, and followed by a directory fsync.
Identity and progress are not independently replaceable sidecars. The existing
supervisor family lock remains held; a separate persistent observation lock
serializes the manifest read/modify/write operation. Lock files are not unlinked.

The driver does not mutate the Rust matrix checkpoints, rewrite test sources,
change denominators, skip tests, or synthesize native conformance results.

## Recovery and migration

When `progress-status` fails before the first successful report in a process,
bootstrap is allowed only if no progress has ever been recorded and the family
has no `.json`, `.jsonl`, or `.txt` result files. Both the exact snapshot filename
and the existing `name-...` family prefix are checked. An unrelated family does
not prevent a fresh run.

An interrupted session with no results may bootstrap again. A failed report that
left a readable partial checkpoint may resume, subject to the previously recorded
high-water mark. A missing/unreadable checkpoint after observed progress is an
error; resolve the checkpoint failure before retrying. Do not erase the manifest
to hide a regression. Pre-existing results without a manifest remain ineligible
for adoption.

Schema-1 manifests are deliberately not upgraded in place: they have no durable
observation history. Retain those results for triage and start a fresh snapshot
name with this driver. Changed source fingerprints already prevent treating an
old driver's run as the same observed build. No existing family is overwritten
by migration.

## Verification

Run the standalone standard-library regression target from the repository root:

```sh
python3 scripts/tests/test_publication_progress.py -v
bash -n scripts/publish-real-status-low-ram.sh scripts/lib/publish-real-status-driver.sh
```

At patch preparation on 2026-09-08, all 45 tests passed: 26 parser/manifest tests
and 19 end-to-end supervisor/driver tests. Integration tests use temporary Git
repositories and an explicitly fake CLI, with real subprocess exit handling,
source/suite hashing, locks, manifests and shell control flow. They cover fresh
runs, ordinary resume, partial checkpoints, restart rollback, denominator changes,
missing checkpoints, stalled reports, corrupt/legacy manifests, source changes,
republishing completed families and publisher exit propagation. Atomic-replace
failure is injected separately. The three restart regressions were also run
against the original source files and all three failed as expected.

Neither Cargo, the Rust CLI against real Test262, nor the full pinned publication
ladder was run in the authoring environment. Those remain required for a new
current-pin result. The generated README status block is unchanged.

## Evidence boundaries

This records observed node-count history. It does not authenticate snapshot
contents, attest that a compiler binary was built from the recorded source, or
guarantee that inputs cannot change during a native CLI invocation. The existing
identity checks and the Rust publisher's full artifact validation remain in
force. A user who deliberately edits the manifest can invalidate this evidence;
it is an accidental-corruption/recovery guard, not a security boundary.
