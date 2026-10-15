# Pinned conformance closure

`lila --jobs 1 test262 close-release [--snapshot-dir PATH] [--threads N] [--reuse-case-workers]` is the release verdict owner.
`scripts/check-test262-closure.sh` invokes it through the verification launcher.
The shell wrapper's local route retains the 4096 MiB aggregate kernel budget with
zero swap, grouped OOM, one CPU and one worker. Its explicit `--cloud` mode uses the cloud machine's
finite inherited cgroup cap; a separate systemd scope is not required there.
Both routes default to one case worker. An explicit `--cloud --threads N` uses
`limited_verification.py --cloud-worker-cpus auto` for closure, which bounds its
CPU set by inherited affinity and all visible CPU quotas while keeping native
compilation jobs serial. The `scripts/check-test262-closure.sh` local route rejects
counts above one and retains its 4 GiB, one-CPU policy. Select additional workers only on an otherwise idle
machine with sufficient inherited memory and storage. The dedicated `conformance-closure.yaml` workflow consumes its exit code
on release tags or explicit dispatch and retains both families on a red verdict.
CI's existing native release build and artifact upload require both ordinary
validation and this reusable closure job on main. The full vendored suite is
tracked in the checkout. The driver first runs the native `test262 sync` presence
and pin-report route under the same cap; that route does not fetch or invent a
suite. The closure owner then admits only its actual clean committed pin.
The wrapper's local self-hosted Linux route requires actual systemd user cgroup delegation;
missing capability rejects that local attempt. Managed-cloud admission checks
the finite inherited machine budget through the same launcher. Direct CLI and
library entries validate case scheduling but do not authenticate managed-cloud
ownership or establish that memory budget. Use the verification wrapper for the
repository's local/cloud resource policy.

The library accepts only the current canonical pinned suite, its clean committed
content, complete embedded Wasm-AOT harness, actual running CLI worker, explicitly bounded
case scheduling and the standard 60,000 ms execution timeout. Existing whole-case
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
selected worker count and 60,000 ms timeout, and applies identically to
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

`closure.json` is schema 3 and records both aggregate paths, full denominator,
nonpass and taxonomy counts, durations, current compiler identity, pins, manifest,
selected worker-reuse policy, configured case workers, admitted worker ceiling and
compiler jobs, exact-result agreement and the consumed typed verdict. Its constructor is private
and it has no release-admission decoder. JSON is retained evidence; the command's
successful return comes only from the actual two-run construction and validation.

Informational `report-all`, `compare` and `publish-status` keep complete red
evidence useful. This gate neither publishes a README status block nor claims
that its own creation proves T26 acceptance. T26 additionally requires integrity
and interpreter-quarantine audits, generated canonical publication, broad green
tests, differential/stress evidence and host, shard/thread and interruption/resume
equivalence. Those independent obligations remain open. General Wasm-AOT
dynamic-code and weak/ephemeron capability boundaries keep their explicit owners.

Library controls cover exact execution-mode sets, nonzero denominators,
every nonpass taxonomy domain, changed per-case results, exclusive fresh evidence,
fake-suite admission and clean Git pins in both repository-root and nested-suite
layouts. CLI controls exercise actual command rejection of
subset/oracle/resume/alternate-worker/time-budget overrides and invalid evidence-parent
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


## Explicit bounded case scheduling

The CLI and shell default to one worker; `--threads N` is a positive explicit
request. The library's existing `run_closure` entry remains serial. Its separate
`run_closure_with_workers` entry accepts a nonzero count and uses the same full
fresh-run owner. Native admission rejects a request above visible CPU affinity
or the tightest whole-CPU cgroup quota, and parallel cases require `--jobs 1`.
Unreadable or malformed quota data rejects parallel admission. The serial entry
keeps its existing one-worker ceiling without requiring a CPU controller. Capacity is checked again
between the two sequential fresh families and before final closure.

This option changes only the existing case scheduler's configured slot count.
It does not run the two families concurrently, alter discovery or the exact
execution-ID denominator, enable resume, widen fixture admission, change pins
or raise either the execution timeout or its existing compile allowance. Each
supervised child remains a single-case worker; optional reuse still gives each
request a fresh Engine, Realm, Store and GC state and retires a process after at
most 64 requests. Cache and memory policies remain owned by the same launcher.

Every fresh node must retain its existing timing sidecar, bound by SHA-256 to
that exact compiler/pin/manifest/node snapshot. Closure verifies its configured
workers, timeout, reuse flag and `resume_requested=false`; missing or inconsistent
scheduling evidence fails instead of guessing settings. The report records the
configured upper bound, not a claim that every small node used all those slots.
The existing scheduler still clamps slots to the number of actual cases.

Controls cover invalid/duplicate worker arguments, unchanged serial defaults,
explicit cloud delivery, the shell wrapper's local parallel rejection, quota/affinity bounds, exact
sidecar settings and missing/tampered sidecars. A native regression compares one
serial cohort with both genuine four-worker shards, including reuse, exact modes,
fresh Realm state and the same literal failure. These mechanics controls do not
establish either full pinned family or release acceptance by themselves.
