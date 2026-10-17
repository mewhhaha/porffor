# Native direct-eval host boundary

ABI8 direct eval uses a separate `compile_direct_eval` native import. Its fourteen
machine operands carry the exact String, immutable caller descriptor, Realm,
lexical/variable/private environments, this/new.target value words, actual
`DirectEvalExecutionContext` and calling ProgramInstance. The Wasm side still
owns callee evaluation, intrinsic-eval identity, argument effects, non-String
identity and prepared-source matching. Only an actual prepared direct-eval miss
enters this native compiler.

The Engine validates the canonical import types against declaration-owned GC
layout and field projections. It roots every incoming GC operand, decodes the
source without lossy UTF-16 conversion, and decodes the descriptor's canonical
compiler-fact codec. Malformed descriptor or GC protocol data is an invariant
error. It cannot produce a JavaScript SyntaxError response.

Private-environment scanning visits the complete actual immutable parent chain.
Each row contributes its scope and actual PrivateNameTable length, including
hidden ancestor and auto-accessor scopes absent from the visible spelling map.
Repeated scopes, out-of-domain identifiers, missing ordinals and exhausted
identity space fail closed. This validates `RuntimeDirectEvalCaller`, whose
complete identity includes the resulting allocation floor.

The host verifies this/new.target operands against their actual retained
invocation snapshots. Derived callers retain the original four binding cells;
the active constructor remains a genuine derived FunctionObject with its own
non-null defining ProgramInstance. That defining owner can precede the current
calling eval program. The host does not replace it with the calling program.

Compilation runs native Rust parsing, early errors, IR lowering and Wasm codegen.
`RuntimeDirectEvalProgram` and its artifact retain the complete caller witness,
exact source, prepared root identity and direct-eval role. Before entry, the host
checks that witness and the exact immutable runtime Arc, rejects a module start,
and validates the initializer and fifteen-operand factory against the actual
core types. P2 initializes and executes in the existing Store through the
existing linker and Intl profile. No fresh Engine, runtime R, Realm, execution
control or deadline is created.

The original execution control is checked before decoding, throughout the
private-chain walk, around source lowering/emission/native module compilation,
and around initialization and execution. A completion returns `Entered`,
including an ordinary JavaScript throw. Only a genuine malformed-JavaScript
frontend diagnostic returns `SyntaxError`; other compiler capability failures
retain their existing typed errors. Outgoing references become `OwnedRooted`
before the temporary RootScope exits.

An unbridged `ImportCallSourceOriginIr::ExternalCaller` remains on the real
import operation. Uncalled function bodies and unexecuted branches remain
compilable. Its current AOT boundary evaluates the original operands and then
reports the distinct runtime capability `ExternalCallerImport` only on actual
execution. It does not alias global import authority or become a SyntaxError.

NativeFunction and global-Script compiler behavior is unchanged. Their shared
context only gains a test observer join for this direct-eval route. Exact
statically cfg-gated compiler test and observer paths are excluded from the
production fingerprint; all production compiler paths retain invalidation and
Cargo rerun coverage, including the new direct-eval host and private-chain code.

The private component includes Engine and protocol regression controls. Their
source checks do not establish native acceptance: the coherent ABI8 batch still
needs Rust compilation, focused execution, historical passing-ID retention and
the full verification ladder. No code/data reuse or iteration-time speedup is
claimed here.
