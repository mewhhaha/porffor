# Ordinary generator Switch regions

Ordinary generator Switch owns complete discriminant, ordered selector and
fallthrough body regions. The source planner and lowerer consume the same checked
plan. Every region includes its final normal state; selectors and bodies have
distinct physical ranges, and the opaque IR constructor checks each consumed
source suspension, each edge and the actual allocated activation bindings.

The discriminant retains its whole value before the single CaseBlock lexical
environment is instantiated. Tests use strict identity without coercing either
operand. A matching selector commits its body entry. Normal fallthrough then
visits later bodies without evaluating their selectors. A default in the middle
of the source is considered only after every real selector has failed. Eager
function declarations and lexical TDZ belong to that shared CaseBlock record.
These rules follow the primary [Switch evaluation algorithms](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-switch-statement-runtime-semantics-evaluation)
and [CaseBlock evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-runtime-semantics-caseblockevaluation).

The checked IR retains separate discriminant and CaseBlock completion-value
activation cells. Native fresh and resumed entry reconstruct the real lexical
environment and live break/label frames. Outer Continue keeps its actual loop
destination. Whole Return, Throw, Break and Continue traverse the existing
pending-completion and yielding-finalizer owners.

An opaque Empty-source-item wrapper preserves the preceding StatementList value
across a suspended declaration or variable statement. Its sole source proof is
built from actual Declaration or Var/Empty/Debugger syntax, and the exact lowered
item is attached once. The generic lowering hooks run only inside the actual
ordinary generator Switch context. Nested Block, If, Try and classic-loop items
use the same proof, without guessing from the last generated statement or
duplicating the body. Native checkpoints preserve the preceding value during
generated declaration prefixes; an actual bare Yield statement publishes its
resumed whole value. Nested Switch uses the enclosing persistent value context
after its own completion.

Only Yield-containing Switch statements select this owner. Synchronous Switch
keeps its existing source algorithm and consumes no new continuation states.
The complete iterator and With owners compose their own checked regions;
async/await use their separate protocol owners. Complete yielding Throw operands
use the subsequent [Throw value continuation](generator-throw-regions.md).

Captured With records belong to the function's defining environment. A yielding
Switch inside that function retains their original hidden binding cells and
inner-to-outer ResolveBinding order. The old current-With-depth admission guard
incorrectly rejected this composition after the generator escaped its defining
With statement; that guard and its unused counter are retired. The source plan
still checks every selector/body suspension, and the Switch adds only its actual
CaseBlock child. Fresh invocation starts from FunctionContext's lexical parent;
resume restores the same InvocationFrame and reattaches the saved CaseBlock.
Captured Get/Delete operations use the existing With Reference owner, while a
yielding assignment retains its selected Reference before the RHS. Mutable
unscopables affect later resolutions, not that already selected assignment.

The `escaped_switch_retains_captured_with_order_and_the_pre_rhs_reference` IR
control checks original slots, nearer parameter bindings and assignment order.
The shared `generator_switch_regions/captured_with.js` fixture adds GC, delayed
selector lookup, nested captured objects, live unscopables, writes/deletes,
unrelated caller With records and whole Throw through a yielding finalizer.
It extends only the sloppy variant of the existing native Switch cohort and
supplies one actual emitted-Wasm validation control. The joined workspace type
check, all seven focused generator/With IR controls and all three With artifact
controls pass on 2026-10-08. Native execution and conformance remain pending.

External IR controls cover independent source coordinates, allocated
bindings, recursive Empty evidence, shared cells and actual admission boundaries.
One paired strict/sloppy Wasm fixture covers complete discriminants and selectors,
default order, getter effects, identity, fallthrough, TDZ, function instantiation,
captured cells, nested declarations and loops, labels, GC, caught getter Throw,
and injected Return/Throw across yielding finally. These controls are authored
source; compilation and execution remain required before reporting acceptance.
