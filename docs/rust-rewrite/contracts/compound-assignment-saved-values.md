# Compound assignment saved-value domains

The eager identifier assignment lowering snapshots primitive value-kind facts
before lowering its right side. Runtime evaluation still reads the original
left value, evaluates the right side, performs ordered conversions, and writes
the result. A right-side assignment to the same binding cannot change the type
of that saved primitive or the type published for later operations.

Heap-bearing operands use a conservative dynamic value description. Their
identity survives the right side, but their conversion properties can change;
old heap shapes and function targets are not conversion proofs. This applies
to arithmetic, const-target conversion, and bitwise assignment.

The conservative representation is necessary because the existing
`object_to_primitive_kinds(None)` and `array_to_primitive_kinds(None)` rules infer
String from an absent shape. Absence of tracked properties cannot prove that a
mutable object lacks a custom conversion hook. Those rules are separate
structural debt; this repair does not rewrite global coercion inference.
It also leaves the previously documented definite-TDZ arithmetic completion
path unchanged.

Frozen batch5 returned `31` for a Number binding that was overwritten with a
String during the right side of `+=`, then read by `value + 1`; the correct
result is `4`. This stage builds on the frozen callable-harness assignment
repair, which already rejects false String admission for unknown operands.

Verification targets are `compound_assignment_saved_value` in lila-ir and
`aot_compound_assignment_saved_value` in lila-engine. Native controls include
Number, String, BigInt, later arithmetic, direct conversion-hook mutations,
replacement of the original binding, and abrupt completion ordering.
