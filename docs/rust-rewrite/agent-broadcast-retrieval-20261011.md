# Native agent retrieval and deadline — 2026-10-11

Test262 requires the parent's broadcast to wait until every running agent has
retrieved the message. Previously the native owner queued each command and
returned immediately. Commands now carry separate retrieval acknowledgements.
All recipients are queued before waiting, and the worker registry is unlocked
during the wait. A recipient acknowledges after validating the shared backing
and rooting its local ExternRef. JavaScript SharedArrayBuffer wrappers remain
local to each Store. Callback completion is independent of retrieval.

The parent arms one monotonic execution budget immediately before calling its
Wasm entry. Native broadcast/command receives and worker readiness wait against
the remaining budget. Worker epochs use that remaining time, retaining the
existing tick rounding and phase guard. Native sleeps are interruptible by the
same budget and cleanup. Cleanup still cancels Atomics waiters, sends shutdown
and joins every worker. A dropped receipt does not remove its join handle or
discard the worker's failure. Synchronous host compilation checks expiry before
spawning; the outer case process supervisor still bounds that compiler work.

All 18 focused agent checks pass with zero failures or ignores in 46.71 test
seconds / 97 watched seconds, after a 49.734-second build. Cargo sees three CPUs;
the test executable uses the ordinary single-CPU cloud launcher. The controls
cover both queued recipients, an unlocked registry, disconnected receipts,
root/worker failure preservation, real shared-buffer delivery, waitAsync
notification/expiry and a ten-minute worker sleep under a ten-second parent
budget. Four native control checks cover expiry reuse, a connected pending
receiver, sleep expiry and cleanup wakeup. No existing count or deadline changes.

The preceding build failed during linking with SIGBUS under disk pressure and
supplies no test verdict. Its transcript is retained. Inactive generated
metadata-only caches were reclaimed before retry; source and check transcripts
remain. Exact source, executable and log hashes are in the
[receipt](agent-broadcast-retrieval-20261011.json).

The joined all-feature/all-target type check passes in 65 watched seconds.
All eleven repository/formatting guards pass. The identity guard first caught an
absolute checkout path in the public command receipt; the repository-relative
correction passes while the exact private argv and original log remain retained.

Arbitrary BigInt broadcast IDs,
complete current agent-tree/workspace/pinned acceptance and task closure remain
open. Canonical publisher totals and task states are unchanged.
