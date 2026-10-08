# Mixed async-generator classic regions

An async-generator FunctionBody uses one checked AST allocator for its complete function suspension plan and each classic For/While/DoWhile or If region. Every complete phase has a checked inclusive entry/end range and a separate successor boundary, including eager phases. Conditions finish before branch selection. For initialization, test, body and update retain their original evaluation order; the original lexical head and per-iteration environment owners remain authoritative.

The same complete value lowerer stages Await and Yield operands into actual invocation-owned cells. The original Await settlement, Yield adoption/delegation, implicit Return Await, operator coercion, invocation, property Reference and Identifier Reference algorithms supply the semantics. Both binary Values are acquired before either coercion. A retained property receiver loses stale heap-shape facts before a later key or RHS can run. Plain Identifier assignment captures a write-only Reference; compound/logical assignment reads the selected original Reference before its RHS and uses the same record for Put or release.

Each source suspension carries a closed resume-environment choice. Checked classic/If/expression-branch, explicit FunctionBody Block and Try regions use InvocationOuter. Original linear points, foreign iterator/resource bodies, ForAwaitNext/Close and resource-owned Return continuations use SavedLexicalChain. The checked whole-function source alone mints the environment certificate consumed by codegen; a function may contain both kinds. A second private certificate records actual enclosing Block/Try resume ancestry so those scopes reattach before entering a foreign boundary. Implicit resource-finalizer states have no source point; their enclosing-scope evidence comes from the same checked finalizer factory used by the original lowerer. The original foreign scalar If merge transition relocates both certificates through its checked plan method together with its actual source suffix.

Body StatementList V remains a whole Value in its actual activation cell. Recursive EmptyStatementCompletion evidence wraps the entire source declaration/Var/Empty/Debugger item, including staged initializer prefixes. Head/condition phases do not publish body V. Existing completion/finalizer owners preserve whole Return/Throw/Break/Continue while suspending in cleanup.

The 2026-10-07 native checkpoint exposes a mixed logical branch result whose
compiler alias was discarded with the selector's temporary scope. The selector
now compiles in its enclosing scope, preserving the original result and retained
Reference bindings for the selected arm and final value read. For initializers
use the same scope operation. It rejects a separate lexical Environment owner;
runtime cells, evaluation order and source phase guards remain authoritative.
The complete scope repair passes the workspace type check. Artifact admission
then exposes a separate stale rejection of Break through a catch region. The
preflight now retains checked iteration/CaseBlock targets and labels through
nested regions, while rejecting abrupt branches without a matching owner.
All four original strict/sloppy artifacts validate, and all eight dispatcher
controls pass. `tasks-scope-native5` passes both modes of the phase/Reference
fixture. Its completion fixture passes the whole Return/Throw and catch checks,
then fails at the first awaited iterator value after the checked loop in a
captured Block. Diagnostics confirm the array and its direct iterator both yield
`1`, while head initialization reads `undefined`: the private incoming cell had
no compiler alias and fell through to a global-property read. Complete ForOf and
ForIn now attach their exact owned input aliases before per-iteration scopes;
missing owned aliases fail emission. The workspace check and all four original
artifacts pass. The invariant also catches an uncaptured ForIn head alias lost
after initialization, so both iterator forms keep head declarations in their
iteration scope. `tasks-iterator-intrinsics3` passes all four original strict/
sloppy native fixtures, including the previously failing iterator transition.

ForOf, ForIn, resource, With and Switch owners retain their separate checked
continuation protocols. The source controls retain the positive multi-Await-
before-Yield loop cohort and the original iterator/resource cases.

The IR controls and paired strict/sloppy Engine fixtures cover exact tapes/ranges,
selected branch evaluation, per-iteration closures, GC, retained property and
Identifier References, coercion order, nested labelled loops, injected Return/
Throw and yielding/awaiting finalizers. Focused compilation, Wasm validation and
native execution pass; broad and pinned conformance remain open.
