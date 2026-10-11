# Iterator chunking — 2026-10-11

Iterator.prototype.chunks and windows implement the pinned
[stage-three proposal](https://github.com/tc39/proposal-iterator-chunking/blob/0fbb09fbf5b8afa47bf2397d6a0f37e6302fc623/spec.emu)
in Wasm AOT and the optional oracle. Both public methods have length one.
On-demand GC Lists grow with consumed values and preserve complete tagged
values; every yielded Array is fresh. Windows step the source before removing
the oldest value and never yield a partial suffix after a full window.

Strict size and mode admission performs no coercion. Argument errors close the
provisional receiver before acquiring next and preserve the original error.
Stepping errors complete the helper without closing; explicit return closes
once. Helpers preserve executing/completed state and captured Realm ownership,
including borrowed entries and unstarted return. The oracle's common helper
wrapper now completes after uncaught step failures, restores captured Realms,
and creates its own return result. See the [contract](contracts/iterator-chunking.md).

All 24 focused native controls pass with zero failures or ignores: four new
regression groups, seventeen existing helper groups and three constructor groups.
The new groups run both script modes and both backends (sixteen script
executions), with their original 30-second budgets. Native executables run on
one CPU with libtest serial; Cargo compilation uses the bounded cloud opt-in.

| Complete official selection | Wasm AOT | Optional oracle |
| --- | ---: | ---: |
| chunks (38 files) | 76/76 | 76/76 |
| windows (40 files) | 80/80 | 80/80 |

Each scope independently checks its full sloppy/strict execution-ID set against
the pinned source tree. All failures, unsupported outcomes, crashes and timeout
lists are empty. Each case retains the original 60-second deadline; compiler
jobs remain one, with four explicit AOT case workers and one oracle case worker.
The fresh version-eight snapshots bind current compiler and source identities.

Joined all-feature/all-target workspace types pass. The first check reached the
runtime-image producer but failed because the new mode strings were absent from
its pool; the corrected collector includes both literals. That failed attempt
is retained alongside its passing successor. All twelve repository guards pass.

The [machine-readable receipt](iterator-chunking-20261011.json) retains source,
command, executable and log digests, selected tests, full execution-ID maps and
proposal bytes. The private failed type attempt and disk-pruning receipts are
retained. Full workspace/default sweeps, unfiltered pinned-suite acceptance,
supported-host evidence and task closure remain open; canonical publication
counts are unchanged.
