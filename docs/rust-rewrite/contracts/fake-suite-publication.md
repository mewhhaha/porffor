# Measured fake-suite publication evidence

Owner: **T03 — conformance harness integrity**.

The real-suite publisher accepts fake counts only through the private
`VerifiedFakeSuiteCounts` owner. Its sole product constructor performs a fresh,
non-resumed full Wasm-AOT fake-suite run, requires a nonempty passing verdict,
and proves that completed execution identities exactly equal the discovered
selection without duplicates. The wasm-safe selection must be a nonempty
subset of that same measured evidence. Both published passed/total pairs derive
from the run; discovery alone cannot construct the input to the artifact builder.
Any failing, empty or incomplete measurement returns before canonical JSON/TXT
or README writes. Fresh diagnostic namespaces belong under `target/`, outside
the source inventory used by the frozen publication supervisor.

Three focused library regressions pass: mode expansion and incomplete evidence,
a discovered failure outside the safe subset, and an empty selection. A separate
standalone CLI checkpoint passes the complete current fake suite at 191/191
executions from 190 physical files in 540.049 seconds. Those fixture results
remain separate from the full pinned real Test262 matrix.

The first workspace integration attempt put the full publication measurement
inside the CLI fixture target's shared process beside eight concurrent tests and
a separate 187-execution suite run. Its unchanged 900-second command bound fired
at 90/191, but preserved checkpoints continued to 120 passing executions before
the CLI target exited. Its slowest completed cases took 125.342, 102.999 and
89.548 seconds; the standalone full selection's slowest was 20.138 seconds.
That demonstrates continued progress under contention, rather than establishing
a stuck JavaScript case.

The end-to-end publication witness therefore belongs to the standalone
`fake_suite_publication` Cargo integration target. It runs the actual product
CLI with one compilation job, retains the publisher's complete canonical fake
measurement, and kills and waits for the child at the same 900-second deadline.
The moved test retains every real status/README assertion and also verifies the
measured full 191/191 and wasm-safe 187/187 pairs in JSON, text and README output.
The only tiny input is the requested real matrix fixture; fake coverage is
unchanged. The closed CLI target scanner registers this new integration target;
no failure-ledger row, expected failure, ignore or skip is introduced.

A `--jobs 1` argument inside the former shared-process helper would not provide
this control: the process-wide compilation pool has already been selected by
other concurrently executing tests, and changing it is rejected. A standalone
CLI process selects its own pool before compilation and gives the test a real
kill-and-wait boundary.

```sh
cargo test -p lila-cli --lib fake_suite_publication::tests -- --test-threads=2
cargo test -p lila-cli --test fake_suite_publication -- --test-threads=2
cargo test -p lila-cli --test cli known_failures:: -- --test-threads=8
```

The relocated witness and affected hygiene checks still require verification.
This contract does not refresh the canonical generated README block or establish
a complete pinned Test262 aggregate. The Rust publisher remains the only owner
of those artifacts after verified matrix completion.
