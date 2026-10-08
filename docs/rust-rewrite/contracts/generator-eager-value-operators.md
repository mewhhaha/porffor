# Generator eager value operators

The source planner admits complete nonlogical binary operands, unary values and
TemplateLiteral substitutions through the same recursive continuation plan as
their lowerer. Yield states belong to the actual operands; operators add no
synthetic resume states. Logical and conditional expressions retain their
separate selected-region owner.

The private `generator_eager_value` owner evaluates and roots each whole binary
operand in its real activation cell before invoking the existing arithmetic,
relational or bitwise semantic consumer. Both evaluations finish before either
conversion. Retained operands discard mutable heap-shape evidence, so caller
mutations while suspended cannot become a stale coercion proof. The private
`operator_values` module holds the shared typed unary and relational consumers;
ordinary AST lowering and staged lowering use those same bodies.

Unary values keep Number/BigInt, Symbol errors, typeof and void rules. Delete of
an ordinary property stages its raw base/key and supplies them to the existing
Delete Reference owner through the actual operand substitution map. It never
acquires a PropertyRead or invokes a getter. A checked source disposition shared
by planning and lowering admits only ordinary property References or syntax
that produces a value. Terminal optional chains, including parenthesized chains,
remain refused until a guarded Delete Reference owner exists; their Value owner
must never substitute Get for deletion. Non-Reference delete evaluates its value
and uses DeleteValue, including an optional read inside a comma value expression.
Unresolved Identifier typeof remains on the ordinary Reference route; private
deletion remains an early error and suspended Super
Reference forms remain outside this source admission.

Each template substitution completes ToString and appends into its retained
String accumulator before evaluating the next substitution. An abrupt conversion
cannot reach a later Yield. Substitution expressions may themselves own admitted
operators and selected regions. Whole values, native errors and Return/Throw
continue through the existing generator completion and GC roots.

The authored IR control checks actual value-context admission and distinct source
resumes across every operator family. The paired strict/sloppy real JS→Wasm
cohort observes operand evaluation versus coercion, mutation and GC, BigInt,
relational reversal, whole thrown identity, injected completions, property Delete
without Get, source strictness, template ToString timing and Symbol failure.

This packet is source-only. Compilation, focused controls and the affected broad
verification remain required; it claims no Test262 count or complete T09/T15
acceptance. Mixed Await/Yield state graphs and the separate iterator, resource,
pattern and foreign control owners remain independent work.
