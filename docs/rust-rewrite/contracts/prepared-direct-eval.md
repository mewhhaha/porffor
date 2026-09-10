# Prepared direct eval and caller records

A bare `eval` call preserves its evaluated Reference, callable and argument
list. The backend compares that callable to the current realm's immutable
original `%eval%` slot. A replacement follows ordinary call semantics; another
realm's intrinsic follows indirect eval semantics. Preparing source text never
authorizes changing that dispatch decision.

Known source candidates compile as independent Script units. Their registry
key includes inherited strictness, invocation grammar, private-name identities
and the derived constructor owner when applicable. Parsing uses Eval grammar,
including caller permissions for `new.target`, `super`, private names and class
initializer `arguments`. Syntax errors are deferred until the call executes.

Comma callee candidates such as `(effect(), eval)` and their forwarded
`call`/`apply` forms can discover optional indirect Script units after runtime
binding lookup erases static callable facts. Discovery never rewrites the
callee expression, omits its effects, or grants direct-eval context; runtime
callee identity and the actual source string remain authoritative.

A missing runtime source tuple stays a typed AOT dynamic-source limitation.

Direct calls pass the lexical and variable environment identities separately.
A sloppy eval unit owns a fresh lexical record while its var and function
declarations target the caller's variable record. A strict unit owns its vars.
Named environment entries point to the same cells used by ordinary closures;
source spelling and declaration authority survive physical binding aliases.
A variable record exists even when it has no statically declared bindings.

`EnvironmentIdentifierIr` owns one ResolveBinding lifecycle. It resolves before
RHS evaluation, iterator/default evaluation or arguments. GetValue, PutValue,
delete and WithBaseObject consume that selected record. PutValue re-finds the
name within the selected record after user code, because eval may replace its
binding table; it does not search the enclosing chain again. Compound operators
reuse ordinary coercive code generation on the saved value.

An invocation record retains the home object and shared derived-constructor
cells. Merely invoking eval does not read an uninitialized `this`. A compiled
`this` access performs GetThisBinding, and `super()` updates the original cell,
checks duplicate initialization after base construction, and initializes the
original constructor's instance elements. All execution remains ordinary
Script IR and Wasm emission, with no runtime parser or interpreter.

The ten-argument Script ABI retains the ordinary seven-argument prefix and
adds the variable environment, private environment and execution-context
record. Its 88-byte record retains the home object, original class context,
shared derived cells and ordinary raw `this`/`new.target` values. A direct
Script owns a source-unspellable context cell; arrows inherit it through the
existing captured-binding plan until an ordinary function boundary. Escaped
arrows consequently retain caller authority without using their own call
receiver or eagerly accessing an uninitialized derived `this`.

The native regression targets are `aot_direct_eval`,
`aot_direct_eval_call_identity` and `aot_direct_eval_escaped_arrows`. Its cases cover caller cells,
strict isolation, deletion and redeclaration, reference timing, with records,
caller grammar and constructor context. Completion of this implementation is
reported only after the coordinated native and original Test262 replays; this
contract is not a conformance-count claim.
