# Ordinary generator pattern initializers

An eager binding pattern can consume a complete suspended initializer in an
ordinary generator. The private `GeneratorPatternInitializerSource` accepts the
actual paired source pattern and initializer only after checking that the pattern
contains neither Yield nor Await and the initializer has a complete admitted
source plan. Declaration planning and actual lowering use that same owner.

Lexical source names belong to the enclosing declaration-instantiation owner and
stay uninitialized through every initializer suspension. The lowering declares
missing TDZ facts before staging the initializer, then consumes its retained
whole Normal value with the existing lexical pattern initialization owner. Var
names retain their existing enclosing variable-instantiation and hoisting rules.

Object patterns reuse the single semantic `ObjectDestructure` operation. Array
patterns reuse `ArrayDestructure` with `BindingInitialization`, including its
binding iterator closure rules. Keys, defaults, Gets, rest operations and source
binding publication happen after Normal initializer completion. Injected Throw
or Return skips those operations and uses the existing generator completion and
finally owners. No duplicate pattern interpreter, object model or resume driver
is introduced.

The source controls cover complete multi-yield and selected initializer regions,
paired received cells and source bindings. The strict/sloppy Wasm AOT fixture
covers escaped TDZ probes, one observable property Get, eager computed keys and
defaults, rest, IteratorClose, var hoisting, GC between resumes, and whole abrupt
values through a yielding finally. These controls are authored but have not run
for this source epoch; compilation and focused execution remain required.

Suspensions inside a binding pattern's defaults or computed keys still require
their own pattern continuation owner and are explicitly refused. This batch does
not claim those forms, assignment-pattern suspensions, iterator-owned regions or
mixed async-generator Await/Yield support.
