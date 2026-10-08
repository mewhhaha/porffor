# Ordinary generator Array patterns

Array patterns with source-owned Yield operands use the existing Wasm-GC
IteratorRecord. The compiler retains one actual record per active Array pattern
in a private activation binding; it does not expose an iterator representation
as a JavaScript value. Await, async generators and IteratorBody continuations
remain separate admission and lifecycle work.

The complete RHS runs first. Acquisition and caching the original next method
run once, outside that pattern's own close scope. The checked source plan gives
acquisition, complete body and terminal exit distinct states. The structural IR
constructor validates the retained raw value and iterator binding against the
actual allocation inventory, exact body/suspension ranges and nested ownership.
Its operation kind/state tape comes from the actual ordered pattern elements;
the constructor consumes the matching dedicated statements, excluding nested
iterator owners from their parent's tape.
Resumed entry uses the retained record, including its original next and Done.
Step and rest operations inhabit a dedicated opaque Statement, which publishes
into a validated, distinct owned result cell. The only subsequent expression
is a pure read of that cell; an operation cannot be buried in a generic Expr.

For each element, a SingleName or Member assignment target is selected before
IteratorStepValue. Member bases and raw keys are retained before the step;
nullish rejection and target key conversion stay in the existing PutValue
consumer after any default. A default is a complete guarded source region and
runs only when the stepped value is Undefined. Nested patterns consume that
value after its default. Rest selects its target before draining and retains
the actual resulting Array. Elision never reads the iterator result's value.
All target/default consumers are shared with the Object pattern owner.

The native close scope surrounds target acquisition, steps, defaults, nested
patterns and PutValue, including injected Return/Throw. Normal Yield and
delegated done:false retain the close obligation. Next/protocol/done/value
abrupts mark the original record Done under the existing step algorithm, so
those abrupts do not call return. Other abrupts and normal truncation use the
existing IteratorClose, preserving whole completions and Throw precedence.
Nested scopes close from inner to outer before an enclosing yielding finalizer.
IteratorClose first materializes its whole outcome, including close errors.
Each completed or abandoned scope retires its private edge before dispatching
that outcome to the enclosing handler or finalizer.

Eager nested Arrays still use the ordinary native destructuring consumer.
Yielding nested Arrays use distinct checked structural owners. Lexical targets
and classic For heads retain their original storage names and per-iteration
cells; assignment identifiers use the existing WriteOnly captured Reference.
Object/With selection is never repeated after a step or suspension.

The IR controls check allocation, complete lazy default states, target/step/Put
order, nested ownership and classic head bindings. Paired strict/sloppy Wasm
controls cover cached next, elision, GC, lazy/exhausted defaults, step errors,
normal/abrupt/injected close, nested close order, rest, RHS acquisition order,
TDZ, private targets and per-iteration closure cells. A separate sloppy fixture
covers original Object/Global References when next mutates a With environment.
These controls and this source batch remain unverified until the joined native
implementation, compilation and focused/broad verification checkpoint run.
