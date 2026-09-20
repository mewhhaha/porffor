# Synchronous resource loops in async owners

A `for (using resource = value; test; update)` or `for (using resource of values)`
may run before or after an await in an async function or canonical async Module
activation when the entire eagerly evaluated loop region cannot suspend.
The source still belongs to its analyzed function or Module execution owner.
This does not turn an async function into an ordinary function and does not
change the owner of surrounding or nested statement-list resource declarations.

The previous admission required the containing owner to be Immediate, and the
backend separately rejected every resumable protocol. That rejected ordinary
resource loops in async Modules after their canonical activation was introduced.
The retained `module_instantiation::async_module_resources_have_a_canonical_execution_owner`
regression demonstrates the mismatch.

The private `SynchronousResourceLoop` source proof visits initializers, test,
update, iterable and body. It rejects await/yield, asynchronous iteration and
implicit await-using disposal. It visits eager class heritage, decorators,
computed names, static fields and static blocks. Function bodies, parameters
and instance-field initializer bodies belong to deferred executions and are
excluded; any computed field name remains eager. Generator resource-loop
admission remains a separate capability boundary.

Only a proven region is lowered without allocating async continuation states.
The outer resume state is saved and restored; the actual function protocol,
Module environment, lexical captures and execution Realm remain intact.
Consequently an ordinary try/catch/finally inside the loop does not reserve
clause states that an ordinary loop cannot dispatch. Adjacent surrounding awaits
remain adjacent in the outer state graph. Nested async functions are analyzed
and lowered under their own owners.

At the backend boundary, `SynchronousLoopBodyIr` is a checked borrowed view with
a private field. Its constructor rejects suspension and continuation-bearing IR,
including manually constructed async try plans with no explicit await. The
classic resource emitter requires that type; the for-of resource head carries
it to the iterator consumer. Ordinary assignment heads continue to use their
existing path. The source proof governs lowering state allocation, while this
IR proof governs the backend lifetime; neither substitutes for the other.

The existing disposal algorithms remain the owners of acquisition, reverse
order, suppressed errors and abrupt completion. Classic-loop resources stay
live across continue and dispose on final exit. For-of resources dispose once
per iteration and before IteratorClose. Return settlement, throw identity and
foreign-Realm values use the containing async function's existing completion
transport. A nested ordinary using scope in an async owner retains its
activation-backed capability even when this particular loop never suspends.

Required verification includes `lila-ir --test synchronous_resource_loops`, the
unchanged `--test module_instantiation`, `lila-engine --test aot_async_resource_loops`,
`--test aot_async_for_of_continuations`, and the existing synchronous loop
structure/CLI resource targets. The new controls cover ordinary completion,
continue/break/return, initializer/body/disposer/close errors, suppression,
lexical captures, foreign error identity, eager class evaluation, nested async
functions, composition with a suspending outer iterator, and async Module
entry/dependency completion. Negative source controls keep suspending resource
loops explicit unsupported cases. No whole-suite result follows from this batch.
