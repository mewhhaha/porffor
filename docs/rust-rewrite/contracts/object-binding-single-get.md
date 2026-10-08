# One property read in object binding initialization

Status: source is authored; compilation and runtime acceptance remain pending.
All remaining task source precedes verification under a confirmed 4096 MiB
aggregate kernel cap, zero swap and serial defaults. T08, T15 and T26 stay open.

## Concrete trigger

The former simple lexical object-pattern path lowered a property to a
`PropertyRead`, then cloned that expression into the undefined condition and
the non-default result. A non-undefined getter therefore ran twice in
`let {value = fallback()} = source`. The shared initializer also reached
ordinary synchronous loop heads and the plain-async synchronous for-of head.
An undefined result selected the default after one read, hiding the error in
controls that covered only the default-taking path.

## Consumed owner

All object binding patterns now lower through the existing semantic
`ObjectDestructure` operation. The common private lowerer accepts the actual
binding mode, pattern, evaluated source and optional renamed storage map.
Ordinary lexical and var declarations and the general loop-pattern initializer
all consume it. The generator uses that same initializer after its checked
head/environment admission. The retired simple-name lexical and var statement
optimizers, cloned default helper and their private classifier are deleted.

This retains the existing backend algorithm: prepare the property key and
binding target, perform one GetV into an actual value/tag local pair, select the
default only for Undefined, then initialize or write the target once. Nested
patterns and computed keys keep that owner. Lookup may box a primitive, but the
getter receives the original primitive value rather than that temporary box.
Rest retains its separate CopyDataProperties operation and boxed-source rules.
Empty object patterns still require object coercibility, so null and undefined
throw. Array binding keeps
its existing IteratorRecord-based BindingInitialization owner.

The initializer carries the real source mode. Lexical targets retain
InitializeBinding and TDZ policy; var targets retain declaration/hoisting and
ordinary reference writes, including borrowed environment behavior. This change
does not turn the transient IteratorValue sink into a source binding or replace
the checked async/generator body, state, environment or close lifecycle.

## Source controls

Seven existing IR controls are maintained around the semantic operation:
ordinary RHS/order, var defaults, var loop hoisting/defaults, empty lexical and
var patterns, and a later-sibling async default that remains in TDZ. New IR
controls examine the actual semantic head initialization and real storage modes
in synchronous for-of/for-in and plain-async synchronous for-of.

Three finite Engine cohorts run paired strict/sloppy Test262-host Wasm AOT when
verification begins. They observe one getter or Proxy Get on both default
branches; prior initialized siblings and uninitialized later siblings; var
hoisting; primitive/empty patterns; fresh captured loop cells; interleaved async
head initialization without replay; abrupt getter/default cutoffs and original
throw identity; finally and IteratorClose precedence. Temporary prototype
descriptors are restored in finally. Each cohort requires Normal(Number(262))
and one exact print line, one compilation worker and a finite timeout.

The controls are authored and source-reviewed. No JavaScript fixture compiler,
parser, execution, test, guard or conformance run establishes acceptance yet.
The existing generated README status and historical proof remain unchanged.
