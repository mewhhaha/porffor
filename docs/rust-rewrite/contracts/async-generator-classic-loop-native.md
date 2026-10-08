# Native mixed async-generator classic loops

The product path parses JavaScript, lowers checked mixed source regions, and
emits Wasm GC. `AsyncGeneratorLoopIr` and `AsyncGeneratorIfIr` are separate from
the ordinary-generator and linear async carriers. A private closed native
adapter consumes their complete phase ranges in the original classic-loop
pipeline; it introduces no activation type, JavaScript tag or object model.

The original async-generator activation retains its request queue, awaited
yield adoption and exact resume kinds. Fresh and resumed loop entries rebuild
the real loop, label and finally destinations before any phase can inject an
abrupt completion. Initialization runs once. A `for` lexical record is cloned
before the first test and before each fresh update; resuming inside an update
does not clone it again. Continue reaches update/test and break reaches the
checked exit. The mixed If completes its entire condition region before
selecting exactly one branch.

The actual source-owned suspension point selects the entry environment.
`InvocationOuter` points reattach checked mixed records from the original
invocation; `SavedLexicalChain` points retain the existing linear/foreign
protocol. Selection compares the actual activation resume point with the
validated source point's resume state, including yield adoption. The opaque
function source certificate also identifies enclosing
Block/Try records at foreign Saved and implicit disposal resume states. States
outside both checked lists retain the original saved-chain entry.
A native private capability
minted from the certificate or a real checked mixed carrier permits the shared
lexical helper's async-generator path and restores its prior scope even when
emission rejects a child. Unscoped Saved catch resumes retain the original path;
checked source scopes reattach the original catch parameter before its body.
A captured for-await body resume uses the same saved-child-of-current-parent
search to select its original iteration record before the old one-parent detach.
Its entry preserves the original deepest saved child until that child's body
is reconstructed; fresh value entry still publishes the new iteration record.
Next/Close and resource boundaries retain their reconstructed parent.
Delegation and resource capability/request/pending-completion algorithms remain
the original owners.
No body-wide boolean, extra source
Await, public plan flag or GC field authorizes this lifetime.

The loop's validated original activation cell holds its whole StatementList
value. The body alone activates this context. Initialization, test, update and
mixed conditional operands suppress it; the preceding compiler context is
restored after each operand region. Source-owned recursive
`EmptyStatementCompletionIr` wrappers suppress all generated initializer
prefixes, preserving the previous value on a Normal empty completion. Yield
and Await checkpoint before evaluating their operands. Bare and delegated
source yields retain the received whole value in an active body context.

Suspended Identifier assignment uses the existing immutable native Reference
record in BindingCell field 10, anchored in the original invocation record.
Selection/Get occur before the RHS; resumption never repeats ResolveBinding or
HasBinding. Normal Await and Yield preserve that edge. Committed Return or
Throw retires it before a handler/finalizer receives the whole completion;
existing Take/Release handles normal Put and short circuit. Async-generator
request and delegate machinery is otherwise unchanged.

The artifact control emits both actual source-lane fixtures and validates the
Wasm GC/control types. Separate strict/sloppy Engine controls exercise queued
requests, original old-value/reference retention through Yield then Await and
GC, injected Return through an awaiting/yielding finally, injected Throw and
rejected Await through catch and a fresh assignment, and original captured
Block/catch cells across a linear Yield followed by mixed phases, captured
for-await head/nested-body cells, and whole Return through synchronous and
implicitly awaiting disposal under a captured parent. These controls are authored
source evidence; this batch has not compiled or run them. General foreign
iterator/resource bodies and other excluded source forms remain explicit debt.
