# Global and Error native caller effects

Status: dry source only. Compilation, emitted Wasm, runtime controls and
broad/pinned acceptance remain unverified. All remaining task source precedes
the next verification checkpoint under a confirmed 4096 MiB aggregate kernel
cap, zero swap and serial execution. Full T04, T24 and T26 remain open.

## Actual native effects

Escape and all four URI codecs share the live native ToString owner with
Unescape. A codec cannot make input conversion pure: an object's conversion
method can change captured values or throw before encoding/decoding starts.
The five missing synchronous-user-code flags are added; Unescape retains the
flag from the preceding invocation batch. The native codecs, allocation paths
and separate construction workload reproducer keep their existing owners.

The seven shared-message Error constructors observe NewTarget prototype,
message conversion and cause options. AggregateError additionally consumes an
errors iterator after its constructor prefix. SuppressedError converts its
third message operand and observes NewTarget, with no cause-options algorithm.
All nine constructor rows record synchronous user code. Error.prototype.toString
records the name and message Get/ToString phases in their actual order.

## Consumed effect authority

The real catalog metadata feeds direct and candidate call analysis. The
existing caller-effect owner invalidates captured kind, shape and element facts
when these native operations can call user code. A consumed const catalog rule
requires the reviewed family flags; deleting one becomes a compile error.
Native identities, ordinals, installers, acquired callees, raw receivers,
complete argument lists and normal result domains remain with their existing
owners.

Meaningful lowering controls and finite paired Engine cohorts cover callback
mutations, acquisition before argument-driven replacement, retained ignored and
spread operands, prefix/abrupt ordering and called-function Realms. Expected
results are authored controls and require execution before acceptance.

## Remaining boundary

This covers the reviewed family metadata gap. Numeric and other coercing native
families still require their own
effect audit. Native algorithm, URI allocation, general GC/weak facilities and
full pinned conformance work remain separate. No test result, task closure,
conformance number, skip or interpreter path is introduced.
