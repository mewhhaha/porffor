# Compact pure fixed Array literals

Large pure fixed Array literal trees use compiler-validated passive recipes
when their complete construction has at least 128 nodes. The threshold is
independent of source grammar, test identity, locale and conformance metadata.
A conservative complete IR walk admits only literal Undefined, Null, Boolean,
raw Number bits, String, holes and recursively fixed arrays. BigInt, Symbol,
source/property reads, calls, coercions, spread and suspension retain the
existing emitter. No effectful subtree is evaluated as recipe data.

The checked recipe has a closed opcode domain, balanced parent-first frames,
exact per-array u32 lengths, checked passive extent and maximum nesting depth.
Its 16-byte rows retain exact Number bits, including signed zero and NaN, and
immutable pooled String ordinals. Collection keeps every original String and
RegExp candidate; it does not prune input or raise a body/memory limit. An
iterative walk collects one covering recipe without repeatedly encoding its
children. The dedicated passive program segment3 follows the companion pooled
String descriptor segment2; existing UTF-16 segment0 and static segment1 stay
under their established owners.

One conditionally emitted program helper constructs the arrays in source
order. Private ByteArray recipe scratch and a GC-rooted array/index frame stack
keep incomplete parents and children alive until ordinary descriptors link
them. Every evaluation allocates fresh actual Array exotics and nested arrays;
there is no cached or frozen template. Present elements have writable,
enumerable and configurable own data descriptors. Holes advance the index
without publishing a descriptor, while each array retains its exact length.
The caller supplies the actual defining source Realm's Array prototype through
the existing literal-prototype owner, independently of the public Array
constructor. Internal descriptor publication bypasses inherited setters.

The existing helper-conditional control retains its execution ID and now
asserts both closed conditional helper rows; each fact still comes from the
emission context. The new helper is program-owned and uses existing GC types.
No array backing or alternative object representation is added.

Paired data/native controls cover exact scalar/String rows, holes, deep
iterative recipes, BigInt/Symbol/read rejection, fresh identity, independent
mutation, descriptors, inherited holes/setters, public constructor replacement,
foreign source Realm prototypes and effect/coercion order. An encoded-module
control validates actual Wasm and compares 256 against 4,096 nested primitive
pairs: complete data must grow while the helper and largest program body stay
bounded. These controls and the original unchanged 65,536-pair Test262 source
require native verification and replay before this prepared change is accepted.

The program helper has a dense unconditional declaration like module-operation
helpers. Its real body is emitted only when the completed pool owns a recipe;
otherwise its slot contains an unreachable stub. Function layout precedes pool
collection, so a pool-dependent conditional declaration cannot select indices.
The existing conditional-helper invariant and JsonStringify control stay intact.
