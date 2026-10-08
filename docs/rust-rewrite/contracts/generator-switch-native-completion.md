# Ordinary generator Switch native completion

The private `control_flow/generator_switch.rs` emitter consumes the checked
`OrdinaryGeneratorSwitchIr`. It reconstructs its break and label frames around
every fresh or resumed discriminant, selector and body region. Switch adds no
Continue destination. Its break target is outside the one CaseBlock lexical
record, so no-match, normal final fallthrough, explicit Break and finalizer
delivery use the existing environment unwind exactly once.

Discriminant evaluation and actual retained publication precede CaseBlock
instantiation. Fresh CaseBlock entry initializes all direct lexical bindings,
including the original statement inside an Empty completion wrapper, before
hoisted function initialization and selector evaluation. A selector uses the
existing whole-value strict equality operation. A match commits a body state;
default is selected after all real selectors fail. Normal body fallthrough
commits the next body directly and never repeats a selector or lexical TDZ.

`GeneratorStatementListValueContext` is constructed only from the checked
Switch's exact allocated discriminant and completion bindings in its ordinary
generator invocation. The private values use the original InvocationFrame's
invocation Environment and real BindingCells. Nested lexical records never
change the storage address. The context has no public constructor or alternate
runtime record.

While a CaseBlock body is active, sequence entry checkpoints a fresh normal
StatementList value or restores the retained value for an interior resume.
Before a direct or delegated Yield overwrites the current completion, the
existing preceding value is retained. A resumed bare Yield publishes its whole
received value; delegated normal completion does the same. The inactive and
async paths retain their existing completion behavior.

The opaque source-owned Empty wrapper suppresses checkpoints throughout the
original declaration, Var, Empty or Debugger item, including generated yielded
initializer prefixes. On Normal it restores the retained preceding value.
Abrupt completion is preserved. Recursive source wrapping supplies this same
rule beneath Block, If, Try and loops without inspecting the last emitted
statement or copying a body.

Normal suspension keeps both private cells rooted in the activation. A branch
leaving its Switch, or a committed Return or Throw leaving the invocation,
clears the private cells before environment unwind. Inner catch and finalizer
transfers retain them until the final destination is known. Nested Switch
restores its enclosing compile-time context; retiring its own cells does not
clear an enclosing Switch. All whole completions and yielding-finalizer routes
remain the existing native completion algorithms.

The joined source/IR controls validate independent regions, recursive Empty
evidence and actual allocation. The paired strict/sloppy Engine fixture
exercises discriminant scope, selector TDZ, hoisted functions, shared escaped
cells, fallthrough, GC, nested declarations, labelled loop transfers, getter
Throw and injected whole Return/Throw through yielding finally. It does not
claim an eval-based observation of the retained internal CaseBlock value.
These controls are authored and unrun. Compilation, focused execution and the
batched verification checkpoint remain required.
