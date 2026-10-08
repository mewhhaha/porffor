# Complete ordinary-generator Throw operands

A synchronous generator Throw operand consumes the existing complete expression
source plan and staged value lowerer. The real `StatementIr::Throw` follows the
entire operand prefix once, so neither an intermediate Yield result nor a
temporary expression value becomes the thrown value. There is no separate
Throw evaluator, runtime graph or completion representation.

The function, structured Try, classic loop, Switch, Block and With source walkers
append the operand's actual suspension coordinates. Iterator bodies retain
their existing LinearOnly expression admission: a direct Yield operand is
admitted, while conditional or otherwise unowned suspension is refused.
Pattern-owned suspensions and mixed async-generator continuation boundaries
remain separate work.

Normal resumption finishes GetValue before source Throw. Injected Return/Throw
uses the existing suspension owner and bypasses the unfinished operand. Existing
IteratorClose, catch and pending-finalizer owners transport the whole completion
through cleanup and further Yield. No string conversion of a thrown value is
introduced.

Three IR controls cover exact source coordinates, complete operands across real
owners and retained refusal boundaries. A paired strict/sloppy Wasm fixture
covers ordered effects and receiver retention, conditional skipping, delegation,
cyclic values across GC, getter failure, injected Return/Throw, Switch and classic
loop exit, and IteratorClose followed by a yielding finalizer. These controls
are authored source only; compilation and execution remain deferred until the
requested whole source pass finishes.
