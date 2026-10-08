# Native iterator flatMap lifecycle

The atomic GC draft replaces the manual helper layout with the concrete
IteratorFlatMapHelper owner. Its nullable INNER and INNER_ACTIVE fields record
actual inner installation; OUTER, MAPPER and Number INDEX retain whole values
and the cached direct protocol. STARTED, DONE and EXECUTING own resume lifecycle.
The old unused IteratorFlatMapInnerState domain and its spelling guard retire.

Failed mapper invocation or inner acquisition closes the outer record while
preserving the original whole Throw. Failed inner step also closes the outer,
but excludes the failed inner from Close. Active return closes the inner first
and then the outer, preserving the first abrupt completion. Suspended-start
return marks the helper completed before Close; active return protects the
running state during Close. Exhausted inners are cleared before the next mapper
invocation, and the Number counter advances once per mapper invocation.

Actual entry methods are private to the Standard builtin owner and are consumed
by the real dispatch arms. The seven-family shared dispatcher selects the
helper's creation Realm for active execution through the actual callable owner.
The invocation owner restores Realm on every completion.

The [native helper contract](gc-iterator-helper-execution.md) records the
source and finite semantic controls. The earlier manual-layout verification is
historical evidence for that earlier source; this successor is source-only.
Compilation, fixture parsing, emitted Wasm validation and runtime verification
remain pending. All-task source completion precedes verification; later checks
require a confirmed aggregate 4096 MiB cgroup budget and one worker.
