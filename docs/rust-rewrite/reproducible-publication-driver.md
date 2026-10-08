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

The dry schema-8 snapshot migration now makes native compiler provenance
mandatory. The schema-3 publication session queries `lila compiler-identity`,
checks its executing-image digest against before/after observed binary bytes,
and retains the embedded build fingerprint/revision separately from observed
checkout inputs. Every accepted progress response must carry the same native
identity, and the progress writer requires it before replacing the manifest.
The Rust harness independently rejects different-compiler resume and mixed-node
aggregate evidence. Older snapshots and schema-1/2 sidecars remain unbound history
and require a fresh family name; they cannot be relabelled as a current baseline.
See [the source contract](contracts/snapshot-compiler-provenance.md).

A driver contract test uses a fake CLI to exercise process ordering and failure
handling. It is neither a compiler execution nor a real Test262 pass. A complete
current-pin Wasm-AOT matrix and the generated failure backlog still need to be
produced through the existing Rust commands. Unsupported outcomes remain
non-passing and in the denominator.

## Commands

After all remaining task source is written and the kernel limiter is confirmed,
run the complete, nonempty driver contract inventories without building Rust.
The current migration is source only; none of these commands has run for it:

```sh
bash -n scripts/publish-real-status-low-ram.sh
python3 scripts/limited_verification.py -- python3 scripts/test_publish_real_status_low_ram.py
python3 scripts/limited_verification.py -- python3 scripts/tests/test_publication_progress.py -v
```

The retained read-only `Publication driver contracts` workflow runs that inventory
and saves its exact source commit, input hashes and individual results. The
runner rejects missing executions and skipped tests as well as failures.

For an actual publication, build the CLI in the checkout to be measured, retain
that build log, and capture the complete publication transcript:

```sh
python3 scripts/limited_verification.py -- cargo build --release --locked -p lila-cli
set -o pipefail
LILA_BIN=./target/release/lila \
  python3 scripts/limited_verification.py -- \
  ./scripts/publish-real-status-low-ram.sh wasm-aot current-pin-baseline \
  2>&1 | tee /tmp/lila-current-pin-publication.log
```

The wrapper requires Bash, Git, Awk and `sha256sum` in addition to the built CLI.
The printed identities belong beside the publication evidence, not in hand-edited
status counts. Native schema migration and its source-only controls establish
no fresh Test262 result. Suite sources/pins, materializers and exclusions retain
their owners, and the protected generated README block remains unchanged.
