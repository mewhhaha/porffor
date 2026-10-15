# Explicit ProgramInstance entry ownership

This is the later ABI6 entry/allocation component. It is unapplied and
non-mergeable until the typed hooks, literal and metadata readers, native host
compiler, retained jobs and modules close together. It does not change the
independent ABI5 repair checkpoint or establish native acceptance.

Every ordinary call or construct passes a non-null `ProgramInstance` after
its existing operands. The physical positions are ordinary 10, generator /
async / async-generator 7, and prepared Script 12. Existing receiver,
`new.target`, activation, argument-vector, Realm and eval-environment positions
remain unchanged. The semantic input constructors require that owner, so a
forwarder cannot omit it while preserving the old Rust call signature.

Callable entry materialization selects the captured `FunctionContext` defining
owner when present; native entries inherit the explicit actual caller operand.
Prepared Scripts bind the selected executing owner directly while retaining
their independent lexical, variable, private and direct-eval context. No Realm,
Store state, mutable current-program global, ordinal or source name selects P.

All FunctionContext allocations require `FunctionProgramOwner`. The allocator
checks it against the same planned executable kind used for code publication.
Source functions capture defining P, including actual empty bodies compiled by
Function; native builtins and native revokers use `InheritCaller`. The retired
synthetic-empty constructor metadata/entry alias is removed with the complete
native source compiler, rather than retained as a test-only product path.
Named-expression environment creation forwards the supplied owner. Class constructors and
methods capture their declaration's current owner. Delayed class element bodies
instead retain the defining owner from their actual class context. None of these
changes replaces the existing Realm, environment, home object, field, template
or ABI5 legacy-reflection captures and restoration.

Native call/apply/Reflect forwarding, bound and proxy call/construct dispatch,
native callbacks and proper tail calls transport the actual frame owner. Tail
calls root it before retiring the source activation. Saved resumptions read the
defining owner from the source callable retained in the invocation frame; they
do not require a live scheduling caller or infer an owner from a Realm.

`FunctionBuilder::current_program` borrows that materialized entry owner. Main
and registered helpers have a separate one-shot `bind_entry_program` seam.
Helper entry seeds it only from its declared `caller_program` parameter. Source
and native callable bodies cannot replace their selected owner. A cleared or
never-bound builder cannot silently select a default program.

Program bootstrap has a separate `ProgramInitialize` signature, no parameters
and one non-null ProgramInstance result. `begin_program_initialize_body` takes
an unexecuted zero-argument ProgramInitialize body with its distinct return/exit
state, without running the JavaScript main prologue, Realm initialization or
source instructions. The module owner declares it with
the distinct typed bootstrap emission token. Normal main binds the returned P
before Realm/source initialization. The native host may initialize a newly
compiled P in the same Store without executing its JavaScript main. Heap-free
Standalone and pure R bootstrap do not manufacture a fake P.

The same-Store host compilation import has seven input words: kind, the three
newTarget value words, converted source vector, active constructor context and
caller environment. Its six results are the five Completion words followed by
a nullable syntax-message String. A message selects genuine source SyntaxError
construction in the actual constructor Realm; its absence forwards the real
factory Completion. This static signature alone does not implement that host
compiler or grant semantic acceptance.

Required verification includes the exact unchanged retained-hook conflict
(old P must return 17 after P2 main, rather than the observed 23), its existing
transport control, genuine current runtime admission, reverse entry order,
two-P literal collisions, native ownership, constructor coercion/reentry and
new.target effects, callbacks, resumptions and jobs across GC. The added Engine
controls cover single-program ordinary/native/bound/proxy/construct/tail entry
and generator/async activation operands; they do not stand in for cross-program
acceptance. Rust ABI controls require non-null owner slots, distinct bootstrap
and host result domains, and the native versus source allocation policy.

Source/format checks are preparation evidence. Cargo compilation, complete
schema validation, native execution and all original retention obligations
remain pending. The separate retained-R bytegraph identifies 56 registered
helpers on actual direct paths to the three call/proxy seeds, including 20
omitted by the initial conservative source census. That graph does not close
new helper constructor operands, retained job/module owner selection, indirect
call semantics or remaining metadata consumers. This entry component updates
only its four direct FunctionCall/ProxyCall/ProxyConstruct constructor sites;
the remaining transitive constructor propagation belongs to the root join.
