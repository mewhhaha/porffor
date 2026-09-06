# Reproducible low-RAM publication driver

Owner: **T01 — baseline and generated failure backlog**. This is an orchestration
safety step, not completion of T01 or T26 and not a new conformance result.

`scripts/publish-real-status-low-ram.sh` keeps the existing Rust-owned protocol:
read `progress-status`, run a bounded `report-all --resume`, repeat, and finally
invoke `publish-status`. It does not decode, rewrite, classify or approve any
snapshot. The Rust publisher remains responsible for pins, execution identities,
complete node evidence, outcome accounting and the canonical status artifacts.

## Invariants

The driver accepts only the Wasm product backend (`wasm` is normalized to
`wasm-aot`). Jobs, threads and nodes per invocation must be positive decimal
integers. Isolation accepts only `0` or `1`; setting it to `1` retains the existing
forced case-runner behavior.

A fresh matrix may have no readable progress yet. The driver permits one initial
`report-all` attempt and keeps the diagnostic visible. Once that command succeeds,
progress must be readable. A successful report must increase the number of
completed nodes, and an established total must not change. Missing or duplicate
counter fields, zero totals, negative/noncanonical/overflowing counts, and
completed counts above the total fail before publication. Counts and resource
limits are bounded to 18 decimal digits before Bash arithmetic.

Publication requires exact equality with a positive, stable total, never `>=`.
Even at equality, only the existing Rust publisher may verify the aggregate and
write the README. Report and publisher errors retain their exit status. A driver
error leaves checkpoints in place for diagnosis; it does not delete evidence,
retry indefinitely, or turn an error into an empty matrix.

At startup the log records the script checkout's Git commit, executable SHA-256,
backend, snapshot name and suite/snapshot paths. The driver checks the checkout
commit and executable identity before each CLI invocation. Rebuilding, replacing
or removing the executable, or moving the checkout to another commit during the
loop, aborts before the next invocation rather than silently switching compilers.
Paths containing spaces remain single arguments, including `README_PATH`.

## Evidence boundaries

These are **between-invocation checks**, not an atomic executable lock or a build
attestation. The recorded checkout commit does not prove that the executable was
built from that commit, and does not describe uncommitted source edits. Keep the
build log and use a dedicated, unchanged checkout and binary for a publication.
Concurrent writers to the same snapshot namespace are not supported by this
wrapper; use one owner for the complete run.

No new optional snapshot metadata or sidecar authority is introduced. In
particular, this does **not** prove which compiler produced checkpoints from a
previous invocation. Preserve and compare the build/publication logs when
resuming. Mandatory cross-invocation source/binary provenance still requires the
separately designed Rust snapshot-schema migration described by T01. Do not
relabel old evidence as a new compiler's baseline.

A driver contract test uses a fake CLI to exercise process ordering and failure
handling. It is neither a compiler execution nor a real Test262 pass. A complete
current-pin Wasm-AOT matrix and the generated failure backlog still need to be
produced through the existing Rust commands. Unsupported outcomes remain
non-passing and in the denominator.

## Commands

Run the complete, nonempty driver contract inventory without building Rust:

```sh
bash -n scripts/publish-real-status-low-ram.sh
python3 scripts/test_publish_real_status_low_ram.py
```

The retained read-only `Publication driver contracts` workflow runs that inventory
and saves its exact source commit, input hashes and individual results. The
runner rejects missing executions and skipped tests as well as failures.

For an actual publication, build the CLI in the checkout to be measured, retain
that build log, and capture the complete publication transcript:

```sh
cargo build --release --locked -p lila-cli
set -o pipefail
LILA_BIN=./target/release/lila \
  ./scripts/publish-real-status-low-ram.sh wasm-aot current-pin-baseline \
  2>&1 | tee /tmp/lila-current-pin-publication.log
```

The wrapper requires Bash, Git, Awk and `sha256sum` in addition to the built CLI.
The printed identities belong beside the publication evidence, not in hand-edited
status counts. The snapshot schema, Test262 sources/pins, materializers, exclusions
and generated README status block are unchanged by this driver repair.
