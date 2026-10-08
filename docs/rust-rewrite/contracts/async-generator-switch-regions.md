# Mixed async-generator Switch regions

An actual FunctionBody Switch source owns complete mixed discriminant, selector
and case-body ranges. It reserves CaseBlock entry, a fallback state after every
selector, and the final exit even when the source has no Await or Yield. The
source factories and function census use the same physical allocator; internal
AST append does not recursively dry-run nested factories. A shared AST shape
gate checks complete With and Switch bodies without minting nested state plans.

The whole discriminant is retained before CaseBlock instantiation. One original
analyzed CaseBlock record supplies lexical TDZ and hoisted functions. Selectors
execute lazily in source order, including selectors after default. Default is
chosen only after all actual selectors fail. Bodies fall through in source order
without replaying selectors. Strict equality preserves object identity and never
coerces the retained discriminant. The original ordinary and mixed lowerers
consume one physical CaseBlock/fact/selection algorithm through a sealed protocol
whose associated ranges, regions, expressions and cases keep the domains paired.

The two original activation cells retain the discriminant and CaseBlock value.
Actual mixed source items with Empty normal completion wrap their entire lowered
prefix, including staged Var, lexical declarations, Empty and Debugger. Generated
Yield/Await values cannot overwrite an earlier statement-list value. Native
operand scopes preserve the terminal value lookup while suppressing body-value
checkpoints for discriminants and selectors. Case bodies use the original whole
Completion, break/label and finalizer routes; Switch adds no Continue target.

CaseBlock selector/body suspensions carry InvocationOuter ownership and actual
scope ancestry. Implicit Return Await, Yield adoption and delegation retain their
existing authorities. Captured Identifier References keep the original selected
record, old value and Put across Yield/Await and GC. Committed abrupt completion
retires the original capture; yielding finalizers preserve the whole pending
Return or Throw value. Sloppy With remains admitted through its actual analyzed
token; strict With remains an early error.

The complete region refuses for-await, suspended foreign ForOf/ForIn, resource
heads/suffixes and pattern-owned mixed suspension. Phase-free original eager
iterator children remain on their original physical paths. Foreign iterator and
resource bodies do not gain a mixed Switch owner through an execution-kind test.
There is one Wasm GC object/environment/Completion model and no interpreter or
alternate runtime representation.

Authored IR controls cover exact phases/tape/certificate, retained cells, hoisted
functions, recursive owners and genuine refusals. Engine cohorts consume the
actual selection and completion fixtures through JavaScript-to-Wasm compilation
and the Wasm-AOT backend. They cover selector laziness/default/fallthrough, outer
discriminant versus inner TDZ, captured cells across GC, labelled outer Continue,
getter/rejection identity, original With References and injected Return/Throw
through awaiting/yielding finalizers. These controls are source-only and have not
been compiled or executed in this batch.
