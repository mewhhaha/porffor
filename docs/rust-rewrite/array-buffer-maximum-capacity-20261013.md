# ArrayBuffer maximum-capacity admission — 2026-10-13

Status: repair authored after the verified `a4b69645d` RegExp/cache batch.
Native and pinned before/after acceptance are pending. The original complete
baseline remains frozen at `4e01400fc`.

The completed pinned ArrayBuffer node contains 442 execution IDs: 440 Success
and two Bugs, both modes of `options-maxbytelength-allocation-limit.js`.
Zero-length resizable buffers previously accepted 7 PiB or near-2^53 maxima
because emission checked only the current allocation and stored the maximum
as an I64.

Pinned ECMA262 `5345883164f463e87f8b40aca4956157ecba8783`,
`AllocateArrayBuffer`, first compares current length with the requested maximum,
then creates the object and current backing. Its resizable branch requires a
RangeError when a Data Block of the maximum extent cannot be created. The note
permits failure when the maximum cannot be reserved. The exact local source
has SHA256 `5194892f5a5f595962929b7099e028b1db44c8b68c44f4ce20f77b266507d57d`.

The target allocator accepts a nonnegative I32 GC byte-array length. Eight-byte
rounding therefore bounds both concrete and maximum extents at `i32::MAX - 7`.
One constant now owns that existing representation bound. Maximum admission
occurs after prototype selection, current backing allocation and ordinary
object creation, before the completed buffer is published. ToIndex stays wide;
the earlier length-versus-maximum comparison stays before prototype lookup.
No allocation of the entire maximum or alternate backing representation is added.

The computed native fixture covers impossible maxima with zero/nonzero current
lengths, exact conversion/prototype effects, prototype and conversion exception
identity, the early current-length comparison, representable-boundary acceptance,
resize, transfer and fixed-length transfer with retained data. Both strict and
sloppy modes use the real Wasm backend and original 30000ms native deadline.
Existing actual allocation-failure and transactional resize/transfer controls
remain required.

Acceptance requires a coherent default optimized build, full affected native
scopes, and all 442 original pinned IDs at unchanged 60000ms bounds. Compare
every prior Success and failure, retain all raw owners, and report actual
eliminated Bugs, regressions and missing IDs. Full-suite/task completion remains
a separate gate.
