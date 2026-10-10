# Full dry-source pass and bounded later verification

## Managed cloud exception — 2026-10-09

The environment owner permits verification in a managed cloud machine under
that machine's inherited finite cgroup memory cap. Invoke
`python3 scripts/limited_verification.py --cloud -- <command>`. This mode checks
the current cgroup and all visible ancestors, uses the tightest finite
`memory.max`, and refuses an unbounded or unreadable hierarchy before starting
the payload. It does not require a systemd user session or create another
4 GiB scope. Serial CPU/worker defaults remain in force. The opt-in
`--cloud-cpus auto` mode bounds process affinity by the inherited CPU set and
all visible CPU quotas, preserving stricter native-worker defaults. Cargo and
libtest remain serial; explicit CLI `--jobs` and Test262 `--threads` remain
separate compiler/case counts. Three real Wasm controls pass identically with
one and four compiler workers; all 28 launcher controls and watched live
readback pass. See the [CPU receipt](../cloud-continuation-20261009.cloud-cpu.json).
Following the owner's
iteration-time request, cloud retention allows two modules with compilation
images bounded to `min(256 MiB, max(1 byte, inherited cap / 8))`. Stricter
explicit limits survive. Local retention remains one entry/64 MiB.

The same three native controls pass with 1/64 MiB, 2/64 MiB and 2/256 MiB.
The 41 MB Wasm core has a 252 MB native bundle: the first two configurations
invoke its native factory six times, while 2/256 MiB invokes it once and warm
loads measure about 24 ms rather than 130 ms. These limited diagnostics do not
establish a general speedup. All fifteen launcher controls pass, including
unchanged local controls, tighter cloud ancestors and stricter supplied caches.
See the [exact receipt](../cloud-continuation-20261009.core-retention.json).
Future watched commands use `--poll 1`; test and stall deadlines stay unchanged.

This is a distinct cloud policy: it does not establish the local zero-swap or
grouped-OOM settings. Local verification still uses the scope policy described
below. Record the actual inherited cap with each cloud result; a cloud pass
does not imply a pass under the local 4 GiB limit.

## Local verification history

Source update — 2026-10-07: the shared launcher now also caps retained Wasmtime
modules at one entry and 64 MiB of compilation-image bytes. Stricter positive
caller image limits survive; larger values are clamped. Invalid values refuse
before the scope or payload can start, preventing the Engine's ordinary fallback
defaults from undoing the verification limit. Both the scope-manager and payload
invocations inherit the same limits. These constrain retained cache references;
the existing kernel cap still bounds active compilation and process-tree RSS.
Focused source controls are authored and unrun. Product cache defaults and all
worker/test deadlines remain unchanged.

Status — 2026-10-05: the complete source pass is integrated as MAIN119.
The first sandboxed launch could not access the user scope bus; the approved
launch then rejected the launcher's unsupported MemoryOOMGroup assignment.
Both attempts stopped before Cargo or any other verification payload started.
The fresh resource correction uses the documented scope OOMPolicy=kill setting.
The next combined type-check launch confirmed the actual 4096 MiB aggregate
kernel cap, zero swap, grouped OOM and one CPU before Cargo started. The complete
source and integration repairs subsequently passed the whole-workspace,
all-feature, all-target Rust type check in attempt29 on 2026-10-05, retaining
those confirmed limits. This proves Rust types and consts; emitted Wasm and
runtime acceptance remain pending. The first cheap checkpoint passes all 22
resource/process controls, including the five launcher tests for ineffective or
missing limits, excessive budgets, literal arguments and serial descendants.
All 13 cheap repository commands passed before the focused checkpoint, including
formatting, module
assertions and the 56-entry shortcut inventory. Emitted Wasm/runtime and pinned
conformance remain unverified for this source batch.

The first default-product focused compiler/GC checkpoint attempted 86 test
functions: 60 passed, 21 failed and five have no completed results because the
sparse Array target aborted with a stack overflow. No tests were ignored.
The shared emitter, planning and Realm failures are being repaired as one source
batch before affected regressions resume. Broad suites and pinned conformance
remain unverified; these focused results do not close task acceptance.

The first repair batch through MAIN134 passes the whole-workspace,
all-feature, all-target type checkpoint and all four affected source checks.
The affected 28-test attempt then aborted its dense-storage target with a
compiler stack overflow. It was cancelled after source inspection proved the
remaining ObjectDelete/ProxyDelete emitter cycle: zero test results completed,
all 28 remain incomplete, and no tests are counted as skipped or passing.
The owned runner and descendants were cleaned up. The confirmed 4 GiB cap,
zero swap and one worker remained enforced. A complete source successor covers
the typed Delete helper, supervised Test262 compilation deadline, selected
object differential probes and Realm-owned RegExp legacy state before another
verification attempt. This successor has no inherited type/runtime/conformance
pass; task acceptance and canonical publication remain open.


The 58-path successor through MAIN135 passed the all-feature, all-target
workspace type check (attempt31) and all four affected source checks. Its complete
focused compiler/GC cohort contained 88 test functions: 45 passed, three failed
emitted-Wasm validation, 40 remained incomplete and none were ignored. The dense
test completed without the earlier native stack overflow; it and two reached
module controls rejected a missing i64 operand. Verification was cancelled with
exit 143 before repeating the shared invalid emission, and owned processes were
retired. Source inspection found four single-constant, double-store initializations
in Temporal and RegExp literals. The complete MAIN136 source batch repairs them and
adds whole-corpus differential reporting, a fresh-run conformance closure gate
and measured performance reports. Its current type check passed; emitted Wasm
and runtime acceptance remain pending.
Broader generation/reduction tooling, runtime verification, current pinned
conformance, idle-machine timing and task acceptance remain open. Generated
full-suite counts are unchanged. Every verification payload retains the confirmed
4096 MiB aggregate cap, zero swap and one CPU; scratch uses disk-backed
`target/verification-tmp` because shared `/tmp` is nearly full.

MAIN136 passed the all-feature, all-target workspace type check (attempt32)
and all four repository checks. Its focused compiler/GC attempt completed
45 passing controls and two failures, with 41 incomplete and none ignored.
The dense control reached a later Wasm validation error: two TypedArray
constructor alignment branches consumed an i64 remainder as an i32 condition.
The owned runner was cancelled with exit 143 and its reached processes were
confirmed retired. The next source repair normalizes both nonzero conditions
and strengthens the existing construction cohort before verification resumes.
Current runtime acceptance and broad/pinned verification remain open.

The MAIN137 alignment source repair passed all four repository checks and
compiled in the focused default-product run. That 92-control run stopped after
45 passes and one emitted-Wasm validation failure; 46 controls remained
incomplete and none were ignored. Validation reached a later RegExp finite-atom
branch that loaded a checked I64 Boolean word as an I32 condition. MAIN138 stores
that checked predicate in I32Local and requires explicit zero-extension for
numeric width arithmetic. It also connects property-definition and assignment
facades to their existing typed runtime helpers; their compilers retain private
physical bodies. This removes repeated source emission while preserving whole
completion, receiver, strictness and caller-environment ownership. The complete
code, semantic controls and contracts passed the all-feature, all-target type
check (attempt33) and all four repository checks. The 102-control focused run
stopped after 45 passes and one code-growth failure, leaving 56 incomplete and
none ignored. The dense-literal module now passes Wasm validation; its producer
added 259,840 bytes for 896 elements, exceeding the 229,376-byte bound. No later
target ran after this failure. MAIN139 replaces per-element numeric-key and
generic property-definition dispatch with complete standard descriptors
published directly into the existing Array indexed storage. Its fresh-literal
owner preserves holes, active-Realm prototypes and expression/abrupt order.
This source repair awaits focused verification; broad/pinned acceptance remains
open and published full-suite counts are unchanged.

The next T23 source batch carries real immutable ICU Locale, ListFormat and
Collator component images. Their operation consumers load admitted data; the
artifact carries matching payloads and the Engine checks them before cached or
fresh native execution. Source review also moved the 605 existing IR controls
out of the public facade without changing their names or source literals, and
replaced the retired no-collector CLI assertion with live closure/catch roots.
MAIN142 passed the whole-workspace, all-feature, all-target Rust type checkpoint
(attempt36, 96.545 seconds) under the confirmed memory/CPU limits after the exact
exporter build dependency and T02 iterator-family owner repairs. This establishes
the shared image SDK foundation, without runtime or conformance proof. The next
complete source batch adds the real native Number/Plural profile image and the
seven-marker ICU Segmenter image, followed by actual native DisplayNames,
RelativeTimeFormat and DurationFormat template images. Duration retains the
selected Number and List owners; RelativeTime retains selected Number/Plural
ownership, and DisplayNames retains selected Locale canonicalization data.
Artifact admission now covers all eight component images, with matching
profiles, foundation digests and actual dependent owners checked before use.
This successor is source-only and unverified; runtime tests remain deferred.
Whole Intl placement remains External because other service data still lives
in the host. Conformance and general filtered custom images remain open; current
Custom names group exact pinned components, and named-zone settings retain their
complete provider-identity check. Published full-suite counts are unchanged.

Recent staging directories and local receipts under `target` disappeared during
the interrupted turn. The integrated source and three watched logs survive.
The source successor is retained in the working tree with small recovery
notes outside `target`; missing receipts are not recreated as new proof.

## Implementation phase

The user requires all remaining tasks dry-coded first. Finish complete source,
types, meaningful controls and documentation concurrently. Use source inspection,
isolated formatting and cheap syntax parsing. Do not start Cargo, JS compilation,
focused tests, broad suites or conformance checkpoints until that pass is done.
Existing proof logs retain their original source and scope.

## Kernel memory ownership

`python3 scripts/limited_verification.py -- <command...>` requests a transient
user systemd scope with MemoryMax4096MiB, MemorySwapMax0 and OOMPolicy=kill.
The installed systemd262 scope documentation specifies that OOMPolicy=kill sets
memory.oom.group to1; scope support was added in version253. Manager acceptance
alone does not establish the cap: the launcher still requires the actual kernel
readback before starting the payload.
The command and its descendants share the kernel budget, including anonymous
memory and accounted file cache. It is an aggregate inherited cgroup cap rather
than a per-process address-space limit or a periodically sampled RSS warning.
Memory exhaustion terminates the owned group; it cannot be reported as a pass.

Before payload execution, the inner launcher reads its actual cgroup-v2
membership and memory.max, memory.swap.max and memory.oom.group. Unlimited or
oversized memory, enabled swap or missing group termination refuses execution.
A missing manager, unsupported host or inaccessible controller has no uncapped
fallback. The current sandbox denied user-bus access during read-only inspection;
the subsequent approved launch confirmed the actual cap. Resource bootstrap
remains distinct from accepted compiler or task verification.
The launcher rejects any --memory-mib value above 4096 before contacting the
scope manager. The option may lower the budget. Do not raise the budget after
a failure. This additional source control is authored and unexecuted.

The launcher preserves command arguments literally, disables manager environment
expansion, narrows the inherited CPU set to one member and sets Cargo, developer
wrapper, libtest and Rayon defaults to one. The Rust engine's default pool reads
that narrowed available parallelism. CLI conformance also uses --jobs1/--threads1.
The CPU-only wrapper preserves lower explicit Cargo caps and now defaults to
one; it does not claim to enforce memory on its own. Linker flags and Cargo
artifact fingerprints remain unchanged.

## Later verification

After the complete source pass, one integrator establishes the resource scope,
compiles once, runs focused regressions and then broader suites sequentially.
Use the memory launcher inside the existing stall/log supervisor:

```sh
./scripts/run-watched.sh --label source-pass-check --stall 900 -- \
  python3 scripts/limited_verification.py -- cargo xc --locked --offline
```

Use fresh processes for complete inventories when retained compiler memory grows.
Keep sweep cache ceilings within the budget; the current bounded examples use
256MiB function and128MiB each module/program caches. Historical large-machine
measurements retain their recorded values and do not set the current policy.
Preserve memory-limit failures as resource failures and report exactly what ran.

Authored resource controls cover a missing manager, absent/oversized kernel cap,
enabled swap, missing grouped OOM, original argv, serial inheritance and caller
job limits that cannot exceed inherited affinity. They do not allocate a large
workload or establish real kernel acceptance without execution. Full compiler,
semantic, timing, pinned conformance and task acceptance remain required.
