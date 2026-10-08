# Resumable catch patterns

Iterator-body catch defaults also use the original Try/catch owner and owned
thrown-value cell. The Generator body validator consumes the checked Array and
default-branch owners, while the async body validator already consumes its Array
owner. Only BindingInitialization enters the pattern's complete branch context;
the surrounding iterator keeps its original branch targets and close scope.
The iterator-catch fixture checks inner pattern close before outer iterator close
on normal Return, rejected Await and injected Throw/Return. Existing ForAwait
materialized child-environment restrictions remain explicit. These new controls
have not been compiled or run.

Ordinary Generator and plain Async catch patterns consume the same original
element algorithms as the mixed protocol. Their source planners reserve
BindingInitialization before the catch body, retaining their original Try
clause states. The generated thrown-value cell belongs to the actual activation
inventory, and one opaque Empty prefix covers the complete initialization.
Foreign iterator source boundaries remain closed.

The existing Try/catch graph owns complete CatchParameter BindingInitialization
before entering the original catch body Block. Mixed computed keys and defaults
consume the sole source suspension allocator and existing Array/Object pattern
owners. The original thrown value occupies one actual invocation binding cell;
the original analyzed catch parameter and body environments remain distinct.

All parameter BoundNames are predeclared in their original storage before any
default runs. The complete prefix carries an AST-bound Empty-completion proof,
preserving statement values while allowing whole abrupt completions through the
original Try/finalizer route. Array parameter patterns keep the existing retained
IteratorRecord and close precedence through Yield, Await rejection and injection.

Two IR controls and the strict/sloppy JS-to-Wasm GC fixture are authored and unrun.
They cover original thrown storage, parameter TDZ, outer/body capture separation,
Get-once ordering, normal and abrupt IteratorClose, escaping closures and whole
Return/Throw through yielding finalizers. No verification has run in this batch.
