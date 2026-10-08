# Ordinary generator classic loop emission

Ordinary generator For, While and DoWhile loops consume the checked
`OrdinaryGeneratorLoopIr` phase graph in the private
`control_flow/generator_loop.rs` owner. The preceding single-yield ordinary
implementation is retired. The existing `GeneratorLoop` representation and
async compiler remain the async lane's representation and implementation.

Each complete phase owns a `BlockIr` and an inclusive range ending at its last
normal or resumed segment. Initializer, test, body and update boundaries are
checked against the source planner; DoWhile's body precedes its test. The backend
guards the complete phase and compiles its existing resumable statement/block
algorithm. A normal phase publishes the checked next entry. A Yield returns from
the actual emitted body; a later invocation resumes the same phase without
repeating its acquired prefix or preceding phases. Conditional regions select a
whole checked branch once and retain If's UpdateEmpty behavior.

Real Wasm break, iteration and Continue frames are reconstructed for fresh and
resumed entries. The body Continue block ends before For update or DoWhile test.
Label and Continue targets are retired before their Wasm labels close; subsequent
head code sees live iteration targets. Checked head regions reject direct source
branches into the body's Continue destination. Source Break, Continue, Throw and
Return use the existing environment unwind and whole pending-completion/finalizer
dispatcher. A yielding finalizer resumes under the same control identities.

Every loop retains its checked private owned-environment cell for its whole
normal value. The backend requires the exact owned binding and a resume frame,
then addresses the cell through that frame's invocation Environment, as the
shared statement-list owner does. The private cell has no source declaration
and does not require a source-name lookup. It restores the value after yielding
heads and updates. It records the
normal body or Continue value in that cell. No temporary Wasm local is relied on
across suspension. The cell is retired on loop exit while the current whole
completion remains rooted.

The 2026-10-07 `tasks-loop-name1` checkpoint passes the full workspace type check
and the original yielding phase/branch-order control. The second original
control exposes a For initializer's binding alias discarded before its later
phases. Initializers now compile in the existing loop scope; temporary test and
update operands still retire their own scopes after their terminal values are
consumed. The runtime cells and initialization guards are unchanged.
`tasks-scope-native5` passes both original native controls after this repair,
including labelled finalizers, iteration cells and whole abrupt identity.

For lexical initialization uses the existing record allocator and resumed record
reattachment. Per-iteration cells are copied before the first test and before a
fresh update. Resuming a suspended head, body or update reuses its existing cells;
closures from prior iterations retain their original records. Break unwinds the
loop's lexical record and the invocation frame publishes the resulting enclosing
Environment. This uses the existing GC activation, invocation frame, Environment,
binding cells and Completion representation.

Statement entry/exit projections, labelled dispatch, all four backend planner
walks and pooled literal collection visit the complete checked regions and
expressions. Async admission explicitly refuses these ordinary-only owners.
The physical module assembly and its twelve Intl image writes are unchanged from
the sealed T02 predecessor.

Two new Wasm Engine controls cover multiple yielding head/body/update positions,
conditional branches, While/DoWhile order, nested and labelled Continue/Break
through yielding finalizers, iteration closures across collection, and whole Return/Throw identity
through nested finalizers. They are fixtures, not a product execution strategy.
The separate IR lane supplies checked graph/source-edge controls. The workspace
type check and both focused native controls pass. Broad and pinned verification
remain open; these focused results establish no full conformance claim.

The multiple-yield head controls consume the existing staged call and Identifier
assignment expression owners. Yielded logical selectors and compound Identifier
updates with suspended operands remain separate refused expression foundations;
this loop graph does not admit them by changing their source restrictions.
