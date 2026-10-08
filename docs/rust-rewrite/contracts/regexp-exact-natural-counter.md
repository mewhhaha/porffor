# Exact counted RegExp state

The version-three RegExp descriptor carries one source-ordered bounds record per
checked RepeatBegin/Guard slot. Finite minimum and maximum bounds use canonical
decimal bytes. Unbounded is a separate closed maximum kind. The static producer,
emitted constructor compiler and immutable GC-byte admission share the same
eighty-byte header, forty-eight-byte bound row and checked state extent. The full
descriptor enters the existing immutable static-data key; no additional GC object
model or mutable bound object exists.

Matcher materialization retains the admitted slot count, state byte length and
relocated bounds pointer before consuming the checked GC view. It copies the
complete immutable descriptor before acquiring the exclusive workspace. Each live
repeat region contains five metadata words followed by source-sized little-endian
u32 limbs in radix 1,000,000,000. Minimum and finite maximum capacities derive from
their exact decimal digit lengths. Begin parses those digits once into all limbs;
zero has used length zero. Decrement borrows through low zero limbs and reduces the
used length only when the high nonzero limb disappears.

Guard, End and Exit preserve the existing required/optional lifecycle. Required
empty iterations decrement exact remaining bounds. Optional empty attempts restore
their alternative before decrementing. A finite zero maximum permits exit; an
Unbounded maximum does not share that representation. Ordinary choices and positive
assertion completion copy the entire contiguous metadata and limb slab together
with their established capture snapshots, so nested backtracking cannot retain
another branch's counter values.

The source-sized live slab is allocated once. Choice frames grow on demand within
the existing 512 MiB matcher scratch ceiling; that ceiling is not increased. The
separate [required-empty replay proof](regexp-required-empty-replay.md) admits
acceleration only after a successful required iteration at the same cursor. Its
actual body may contain replay-safe capture writes, fixed-position assertions,
replay-safe empty references, input-only lookaround islands and positive exact
nested repeats. Typed maximum subtraction consumes the
exact remaining minimum before clearing it; optional attempts retain their original
fallback order. This proof is source-authored and remains unrun. General mandatory
empty bodies containing escaping choices, capture-dependent lookarounds, nonempty capture-dependent references or optional
nested repeats still need their own acceleration proof. Construction or a timeout cannot be reported
as a passing huge-empty match.

This is a source batch. Compilation, descriptor controls and literal/runtime
matching regressions remain unrun until the combined capped verification checkpoint.
