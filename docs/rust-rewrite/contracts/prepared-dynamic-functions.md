# Prepared Function constructor sources

Function-family calls with known source candidates register independent
compilation units. The original call or construction stays in the caller IR;
its callee and arguments are evaluated normally. This covers ordinary,
generator, async and async-generator Function constructors.

An unresolved member named `Function`, including a statically computed name,
also contributes exact string argument candidates. This discovery does not
assert the member's callable identity. Unsupported lowering capabilities in
such an optional candidate leave the runtime unsupported boundary available;
compiler invariant failures remain compilation failures. A confirmed intrinsic
call upgrades the same candidate to a required compilation unit.

Primitive arguments and known string bindings also contribute guarded tuple
candidates; these facts never replace runtime argument evaluation or coercion.
Literal arrays and records carry finite source candidates into callback
parameters. Constructor-family initializer hints survive later effects without
preserving callable identity: an overwritten alias still invokes its replacement.
Nested functions retain these optional hints for actual Script-global
declarations even though runtime reads use global properties. A scoped or
captured binding selects its own storage first, so local shadowing cannot
borrow a global declaration's hints. Runtime callable identity and exact source
equality remain authoritative.
Candidate expansion is bounded at 256 distinct values or argument tuples;
exceeding that bound leaves the runtime source limitation explicit. Getter
reads, callback dispatch and source coercion still occur at runtime.
Even when no candidate exists, intrinsic Function calls execute all argument
ToString operations before reporting the typed dynamic-source limitation. An
abrupt coercion remains its ordinary catchable JavaScript exception.

The frontend parses formal parameters and the body separately, then parses an
unnamed function expression for combined early errors. It retains the canonical
`function anonymous(...)` source for reflection without introducing an
`anonymous` lexical binding. A malformed source records a deferred SyntaxError;
a parser capability failure or compiler diagnostic remains a compilation
failure. The caller does not inherit that function body's parse error.

Each independent unit uses the ordinary parser, analysis, spec IR, lowering and
Wasm function emission. Shared allocation counters keep nested function,
environment and private-name identities distinct across units. The temporary
Script wrapper must have no declarations or captured root activation; lowering
rejects a violated wrapper invariant. Every emitted nested function, host
builtin requirement and capability counter is retained when units are merged.

The intrinsic constructor owns runtime dispatch. It converts the evaluated
arguments to strings in source order, compares the complete tuple against its
prepared registry, and either throws the deferred realm SyntaxError or
allocates a fresh function. Replacing the source callee still calls the
replacement. Calling a prepared constructor repeatedly shares only emitted
code: function objects, prototypes and invocation environments remain fresh.
The active constructor supplies the defining realm/global environment;
`newTarget` supplies the function's internal prototype through the existing
constructor protocol.

An unmatched tuple reaches the typed unsupported dynamic-source boundary. No
parser, source evaluator or interpreter is shipped in the artifact. Runtime
source generation outside the prepared registry remains unsupported. Direct
eval additionally requires its caller records and invocation context, described
by [the direct-eval contract](prepared-direct-eval.md).

The focused verification targets are `lila-ir --test prepared_dynamic_function`
and `lila-engine --test aot_prepared_dynamic_function`. They cover independent
scope, fresh functions, nested closures/source units, all four execution
protocols, deferred errors, argument coercion order and reflection. Their
execution results belong to the coordinated batch checkpoint; merely adding
them does not establish a conformance count.
