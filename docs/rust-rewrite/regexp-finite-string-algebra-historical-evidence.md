# Finite UnicodeSets string algebra: retained checkpoint

This is the recorded historical checkpoint migrated verbatim from T19.
It does not verify the current source or extend the recorded cohort.

The exact generated UnicodeSets `\q{…}` class-string batch is implemented and
verified. At clean pre-batch commit `f580b424d`, the three exact representatives
`built-ins/RegExp/unicodeSets/generated/string-literal-union-string-literal.js`,
`built-ins/RegExp/unicodeSets/generated/string-literal-intersection-string-literal.js`
and
`built-ins/RegExp/unicodeSets/generated/string-literal-difference-string-literal.js`
each reported `0/2` sloppy/strict Wasm-AOT executions. All six measured
executions were `Runtime/NotImplemented` with `RegExp.prototype.exec unsupported
pattern`, with zero unsupported, crash or bug verdicts. None of the three has
an exact rewrite, materializer or known-failure entry. The source-coherent
27-file/54-execution inventory has nine generated combinations for each of
union, intersection and subtraction where at least one operand is a string
literal, excluding the six combinations that also depend on Unicode properties
of strings. The compiler now retains a canonical range-and-string set, applies
exact sequence algebra, normalizes one-code-point members into ranges, and
emits multi-code-point alternatives longest-first before the singleton class
and empty member in both directions. Central verification passed
workspace/all-target checking, `cargo xc`, focused IR `1/1`, bounded structure
`7/7`, the source-free Wasm lifecycle fixture `1/1`, and the exact unmasked
cohort `54/54`, with zero parser, early-error, lowering, runtime, Wasm-backend,
harness, unsupported, crash or bug outcomes. The fixture found one integration
gap after the IR implementation: reverse lookbehind rejected the existing
code-point-literal and Unicode-range instructions. The matcher now admits those
instructions in reverse and shares one canonical range-membership emitter
between forward and reverse paths. At that checkpoint, other Unicode properties
of strings and direct class-string `/iv` folding remained distinct capability
boundaries; this historical evidence records no broader UnicodeSets closure.
