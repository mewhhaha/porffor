# Bounded case-worker reuse — 2026-10-11

`lila test262 run --reuse-case-workers` shares compiler caches across at most
64 sequential cases per supervised process. Each case creates a fresh Engine,
Realm, Store, GC state and agent ownership. The existing execution deadline and
cold compilation allowance remain unchanged. Exact execution IDs, compiler
identity and single-case snapshots are validated before accepting each result.
Timeouts, crashes, protocol errors and infrastructure failures retire the process
group. Ordinary JavaScript failures remain failures and permit another request.
Final process retirement must succeed before the run can succeed.

All 21 focused native controls pass with zero failures or ignores. They cover
bounded requests and replies, fresh deadlines, blocked pipes, descendant cleanup,
failure recovery, fresh globals/prototypes and jobs, exact snapshots, the
64-request replacement boundary, CLI policy, and real Engine image admission.
The direct server fixture sends a failing case and two successful cases through
the same process. Six IR documentation tests pass, including the private catalog
constructor rejection. The AOT documentation target contains zero executable
examples after the obsolete historical API snippet becomes plain text.

On this cloud machine, the same sixteen raw scripts with identical source pass
in 65.367 seconds with one-shot workers and 5.119 seconds with reused workers:
12.77 times faster for this synthetic selection. Both runs use one worker, one
compiler job, the same executable and IDs, a 60-second execution timeout and
the existing warm disk cache. This measurement does not predict full-suite speed.

The opt-in Linux x86_64 native-test linker also shares the verified immutable
runtime image through a DSO. It checks all three symbol arrays against the static
archive, keys the cache by archive/linker/builder/target identity, and seals each
linked test executable's digest. Engine admission and fresh-program controls
pass with the actual shared dependency. The ordinary CLI retains static linkage.
Completed test executables can be pruned after their receipts are sealed.

The tooling sweep initially reported six failures and one error because the
generated Intl identities predated the collector's Cargo.lock change. Normal
generators refresh the affected identities in dependency order; all eight
reproduction checks and all 443 tooling tests now pass. Joined workspace types
and all twelve repository guards pass. Original compiler, disk-exhaustion and
verification-driver failures remain recorded in the [receipt](case-worker-reuse-20261011.json).

The one-shot worker remains the default. Complete workspace, pinned Test262,
campaign, performance and publication acceptance remains open.
