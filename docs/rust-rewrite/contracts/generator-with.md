# Ordinary-generator With continuation

Every With statement in an ordinary generator's complete function/classic region
has one consumed source plan, including statements without Yield. Iterator-body
LinearOnly and async/Await owners retain their existing admission boundaries.
Strict source still rejects With during frontend early errors.

The inclusive head region evaluates the complete source expression in the outer
environment and publishes the canonical ToObject result into one genuine owned
invocation cell. Body entry follows head end; cleanup exit follows body end. The
source planner and lowerer use the same complete expression and statement region
authorities, including nested With, If, classic loops, Switch, Try and labels.

The With Object Environment Record is the original analyzed child environment,
with its original hidden binding name and slot. Analysis reserves that row before
computing physical capture hops. Its WithObject role points to that same cell.
The generated head cell and the child With cell belong to separate record domains.
The checked carrier consumes both rows, the exact source regions and actual owned
binding inventory. Its outer body region has no lexical environment; genuine
nested source Blocks retain their separate lexical records.

Fresh body entry allocates the planned With record once and initializes its hidden
binding from the retained boxed head. Resumed body entry reattaches the same saved
record through the invocation frame's lexical environment, without evaluating the
head or allocating another record. Real ObjectEnvironment HasBinding,
Symbol.unscopables and Reference consumers use that same original cell.

The whole body cleanup target exists before resumed Yield injection. Ordinary
Yield and delegated done:false save the current chain. Normal completion and
Return/Throw/Break/Continue restore and save the original parent environment before
outward whole-completion dispatch. Inner finalizers and iterator close scopes run
before leaving their enclosing With record. Escaping closures retain their actual
captured With and lexical cells after cleanup; changing the saved invocation chain
does not invalidate those independent GC references.

The paired source controls cover yielded outer heads and primitive boxing, eager
With captures, nested original records and private hidden-row domains, dynamic
HasBinding/unscopables, original selected assignment after property deletion,
lexical TDZ, nested Switch and labelled/outer Continue through yielding finalizers,
whole injected Return/Throw and delegated suspension. The Engine fixture is sloppy
source because With is forbidden in strict source; a separate frontend control
retains that rejection. All controls are authored source-only until the joined
verification checkpoint; no runtime or conformance result is claimed here.
