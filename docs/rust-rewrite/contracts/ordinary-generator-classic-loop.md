# Ordinary generator classic-loop control owner

The source batch admits classic `for`, `while` and `do-while` iteration through
`ClassicGeneratorLoopSource`. Its checked phase ranges are consumed by both the
complete generator source planner and actual lowering. Initialization, test,
body and update have separate inclusive ranges, with checked successors between
phases. Empty phases retain their own entry. The typed constructor compares the
entire lowered suspension inventory with the source inventory, checks every
region/branch/try-clause edge and rejects repeated resume identities, foreign
async continuations and direct loop branches in expression heads.

`StatementIr::OrdinaryGeneratorLoop` contains the only backend-facing loop plan.
For evaluates initialization once, then test/body/update; While starts with test;
DoWhile starts with body. Continue targets For update or the other loops' test.
The backend reconstructs loop, break, continue and directly enclosing label
frames around fresh and resumed execution. Existing pending whole Completion
Records preserve Return, Throw, Break and Continue through yielding finally
clauses. Synchronous for-of retains its independent acquisition and IteratorClose
owner. Existing `GeneratorLoop` remains the distinct async/async-generator path.

Full staged head expression prefixes reuse generator expression and captured
Reference owners. Loop bodies retain multiple yields, nested conditionals,
classic loops and try/catch/finally clauses. `OrdinaryGeneratorIf` owns both full
branch ranges, including yielded conditions staged before selection; it does not
re-evaluate selection during a branch resume. Captured block records remain in
each region's actual BlockIr. Captured For head cells retain the analyzed For
Environment Record and its selected per-iteration slots. The backend creates a
fresh iteration record before the first test and before update, and restores the
active record when update/body/head execution resumes. Every loop additionally
owns one distinct existing GC activation cell for its normal completion value;
a Wasm local does not retain that value across suspension.

IR controls cover three complete phase layouts, multiple/branched body yields,
yielding finalizers with labelled control, nested loops with retained captured
cells and refusal of orphan state edges, unused source suspensions, foreign head
branches and async continuations. Existing ordinary loop-control and capture
refusal controls are updated to assert the new consumed owners. Existing async
controls retain their original semantics. The backend lane owns the independent
Wasm execution controls for phase effects, completion routing and closure cells.

This packet is source-only. Isolated formatting and bounded source review are
the available evidence; compilation, focused controls and broad verification
must pass before runtime support is reported. Suspended pattern initialization,
mixed iterator/resource/with/switch control owners and expression shapes lacking
an existing staged generator owner remain explicit acceptance work. This batch
does not turn those refusals into passing conformance results.
