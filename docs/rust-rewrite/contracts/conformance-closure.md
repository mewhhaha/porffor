# Pinned conformance closure

`lila test262 close-release [--snapshot-dir PATH] [--reuse-case-workers]` is the release verdict owner.
`scripts/check-test262-closure.sh` invokes it through the verification launcher.
Local runs retain the 4096 MiB aggregate kernel budget with zero swap, grouped
OOM, one CPU and one worker. Its explicit `--cloud` mode uses the cloud machine's
finite inherited cgroup cap; a separate systemd scope is not required there.
Closure execution remains serial on both routes. The dedicated `conformance-closure.yaml` workflow consumes its exit code
on release tags or explicit dispatch and retains both families on a red verdict.
CI's existing native release build and artifact upload require both ordinary
validation and this reusable closure job on main. The full vendored suite is
tracked in the checkout. The driver first runs the native `test262 sync` presence
and pin-report route under the same cap; that route does not fetch or invent a
suite. The closure owner then admits only its actual clean committed pin.
The local self-hosted Linux route requires actual systemd user cgroup delegation;
missing capability rejects that local attempt. Managed-cloud admission checks
the finite inherited machine budget through the same launcher.

The library accepts only the current canonical pinned suite, its clean committed
content, complete embedded Wasm-AOT harness, actual running CLI worker, serial
case execution and the standard 60,000 ms execution timeout. Existing whole-case
supervision additionally bounds compilation and retires child process groups.
Admission and every run boundary also directly check Git status for the suite,
including a suite that is itself a repository; unchanged HEAD alone cannot
prove its working-tree bytes stayed clean. Modified, untracked and ignored suite
files reject admission, so ignored extra tests cannot change the denominator
while retaining an unchanged committed pin.
No fixture root, oracle backend, filter, shard, node limit, alternate worker,
custom harness, timeout override or resume option enters closure.

Each invocation exclusively creates a new workspace and two separate empty
family directories. It directly runs every current matrix node twice with
`resume=false`, through the ordinary JS-to-Wasm compiler and supervised worker.
Worker reuse defaults off. An explicit `--reuse-case-workers` retains the
standard serial execution policy and 60,000 ms timeout, and applies identically to
both fresh families. Each request creates a fresh Engine, Realm, Store and GC
state; only bounded immutable compiled modules remain in the worker process.
The first complete red family still permits the second fresh run. Names and
already loaded snapshots cannot construct a closure report. Incomplete runs,
crashes, invalid artifacts or changed pins return an error and retain evidence;
they never create a passing receipt.

After each actual run, the current complete-evidence validator verifies schema,
running compiler source/executable identity, exact pin, matrix strategy and
manifest, full fresh discovery, all matrix nodes, unique exact execution IDs,
sloppy/strict/module modes, failure taxonomy and complete node/aggregate joins.
Closure requires the exact requested family and compares the complete per-case
result maps from both validated node families, including failure kind, outcome,
origin and normalized detail hash. The denominator must be nonzero, both result
sets must agree exactly, and every case must pass. Unsupported dynamic-source
or runtime-capability cases, timeouts and unknown-origin failures remain red.
All taxonomy counts and exact node evidence remain available in the snapshots.

`closure.json` is schema 2 and records both aggregate paths, full denominator,
nonpass and taxonomy counts, durations, current compiler identity, pins, manifest,
selected worker-reuse policy,
exact-result agreement and the consumed typed verdict. Its constructor is private
and it has no release-admission decoder. JSON is retained evidence; the command's
successful return comes only from the actual two-run construction and validation.

Informational `report-all`, `compare` and `publish-status` keep complete red
evidence useful. This gate neither publishes a README status block nor claims
that its own creation proves T26 acceptance. T26 additionally requires integrity
and interpreter-quarantine audits, generated canonical publication, broad green
tests, differential/stress evidence and host, shard/thread and interruption/resume
equivalence. Those independent obligations remain open. General Wasm-AOT
dynamic-code and weak/ephemeron capability boundaries keep their explicit owners.

Six library controls cover exact execution-mode sets, nonzero denominators,
every nonpass taxonomy domain, changed per-case results, exclusive fresh evidence,
fake-suite admission and clean Git pins in both repository-root and nested-suite
layouts. Two CLI controls exercise actual command rejection of
subset/oracle/resume/worker/time-budget overrides and invalid evidence-parent
arguments. These are finite mechanics controls, not a fake full-pinned result.
Source authorship alone confers no compile, runtime, full-suite or release pass.

## Managed-cloud driver — 2026-10-11

`scripts/check-test262-closure.sh --cloud` uses the machine's actual finite
inherited cgroup cap through the existing verification launcher. The default
invocation retains its local 4096 MiB policy. Both routes synchronize first
and invoke the native two-family closure owner. The optional
`--reuse-case-workers` flag can accompany either route and is delivered only to
`close-release`, after synchronization succeeds. Repeated flags and unknown options
and failed synchronization prevent that owner from starting. See the
[driver contract](../cloud-verification-drivers-20261011.md) for verification
scope and the pending native acceptance.
