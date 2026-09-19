# Array index storage and present-index outlining

Array literals previously emitted the complete present-index search, capacity
growth and record copy at every non-hole element. The unchanged computed RegExp
fixture contains 3,209 array elements; checkpoint thirteen rejected its
4,556,950-byte main during Wasmtime compilation. Checkpoint fourteen moved that
bookkeeping into one runtime helper without changing the fixture.

A separate frozen checkpoint-twelve measurement isolates a runtime cost:
preparing the generated Unicode input without any RegExp operations takes
68.229 seconds of Wasm execution. The timestamped control records 70.031 seconds
inside preparation. Increasing an array from 1,000 to 20,000 elements changes
fresh fill from 5 to 669 ms, reset from 5 to 1,814 ms, and refill from 4 to 673 ms;
overwriting existing elements changes from 2 to 32 ms. These are pre-repair
measurements, not a performance claim about the new implementation or matcher.

The duplicate index list was also incomplete: rest arrays and argument snapshots
write dense descriptors directly. Forward and reverse traversal checked just
one dense slot before consulting the list. Five frozen-compiler controls exposed
missed dense properties during shrinking length, non-configurable stop handling,
own-key enumeration and reverse search. Additional controls showed lost sparse
values on length growth and inconsistent writes inside an existing capacity
above the growth threshold.

The storage boundary now matches indexed reads and descriptors:

- Dense backing owns indexes below the allocated capacity. Literals, JSON arrays
  and ordinary dense writes do not copy those indexes into the sparse list.
- Inclusive traversal scans the bounded dense backing through holes. Beyond
  capacity it selects sparse records, avoiding iteration across huge gaps.
- Shrinking length decrements its cursor after each successful deletion and
  retains descending order, non-configurable stopping and requested writable
  changes even when the requested length cannot be reached.
- Indexed deletion uses the same descriptor lookup and tombstone operation for
  dense and sparse storage. It checks configurability before mutation and never
  addresses an index beyond capacity in the dense buffer. Tombstones clear value
  and accessor carriers for Arguments presence and GC tracing. Reusing a sparse
  tombstone publishes a fresh data descriptor without restoring a deleted setter.
- Growth copies at most the old allocated dense capacity, then transfers every
  newly covered sparse value and complete descriptor before publishing capacity.
  The former sparse descriptor becomes absent. Growth limits govern allocation;
  they do not override ownership of already allocated dense indexes.

`ArrayAppendPresentIndex` retains its seven-i64, zero-result internal ABI for
sparse writes: array pointer, exact index, value payload and value tag, followed
by three unused zero words. It invokes no JavaScript and does not overwrite
caller result or completion locals. The closed runtime-helper registry still
owns its signature, emission, index, name and Realm policy. No heap layout or
value representation changes are required. Sparse insertion remains a linear
search; this repair makes no general sparse-performance claim.

The emitted-code regression keeps the 128/1,024-element size and Wasm validation
checks, verifies that dense literals call no sparse bookkeeping, and verifies
that the single sparse helper remains called by the shared write path without
self-recursion. Native regressions cover rest-array holes, key/value enumeration,
forward and reverse searches, live callback/prototype behavior, dense refill,
sparse high indexes, length failure modes, descriptor transfer during growth,
JSON arrays and mapped/unmapped Arguments.

Frozen checkpoint fifteen passes the unchanged computed RegExp grammar suite
and the earlier rest-array and capacity-boundary probes. Nine of ten new array
tests pass; the descriptor-transfer test exposes an older sparse deletion bug,
whose correction has a separate revision. The same unchanged preparation probe
takes 3.674 seconds of Wasm execution, versus 68.229 seconds on checkpoint
twelve; the timestamped control records 3.266 seconds inside preparation.
These measurements isolate input construction. The historical timeout cohort
and full Array/JSON regressions remain separate verification requirements.
