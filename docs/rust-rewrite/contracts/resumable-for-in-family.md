# Complete resumable ForIn initialization

Generator, plain Async and AsyncGenerator consume the same checked source-bound
enumeration carrier. Its closed execution tag selects original Yield, Await or
mixed expression and pattern lowering. The shared regions retain their original
protocol validators; source ranges and the exact suspension tape must match the
head, selected-key initialization and body before a carrier can be published.

The earlier per-protocol carriers and their dead branches have been retired;
see [the T02 reachability closure](obsolete-for-in-carrier-removal.md). The
common carrier retains the original Generator and Async control modules.

The four distinct original invocation cells retain the completed head, native
enumerator, selected String key and persistent statement value. A suspended
computed target or pattern default resumes that initialization against the same
selected key and original per-iteration Environment Record. It cannot advance
again, assign the head again or replay completed target operands. Initialization
is an operand context, so its generated prefixes do not replace the loop value.

The source-owned initializer proof is minted only by lowering the actual original
head against its allocated key cell. It preserves ignored immutable Identifier
Put evidence, With Reference selection, raw Member key ordering, private and Super
References, original binding storage and pattern IteratorClose. ForIn itself does
not use IteratorClose: its existing boxed-object cursor, lazy own-key snapshot,
visited-name set, descriptor checks and prototype traversal remain the native
semantic authority.

Annex B var initializers complete before the enumeration target is evaluated.
Their original resolved write is captured once across suspension. Lexical heads
keep their distinct head TDZ record; a closure over that record remains
uninitialized even when a later per-key record is initialized. Unsupported foreign
iterator or resource ancestry keeps its original owner rather than inheriting a
complete region by execution kind alone.

The authored IR controls cover exact protocol tapes, source identity, allocated
cells and guarded per-key ranges. Wasm-AOT fixtures cover captured TDZ cells,
raw target ordering, rejected/injected default completion, inner pattern close
before outer finally, GC and sloppy Annex B prefixes. The 2026-10-07 workspace
check and both mixed ForIn artifact controls pass. `tasks-iterator-intrinsics3`
passes Annex B; plain Async rejects a labelled complete iterator at an obsolete
admission guard, and Generator fails its first captured head TDZ assertion.
The joined source repair admits the existing checked labelled iterator owner
and always includes named-function self records in capture analysis, matching
their unconditional runtime allocation. The latter prevents a named head closure
from reading its own initialized function cell in place of the outer TDZ cell.
`tasks-for-in-capture2` passes all 2,096 IR controls, both labelled Wasm admission
controls and all five affected native tests. The original Generator and Async
fixtures pass both modes; named-function assignment and direct-eval self-binding
controls also pass. Broad and pinned conformance remain open.
