# Plain async ForIn ownership

`AsyncGeneratorForInIr` with its checked Async execution tag consumes the actual
ForIn source plan, original head proof, four unique invocation cells, retained
raw head, complete per-key initialization,
original TDZ/per-iteration environments and complete body. A private AST identity
type ties both generator and async head evidence to the actual source node.
Foreign same-shaped source heads cannot mint the checked owner.

The complete head executes under the original head TDZ before creating the
native enumerator. Advance and body entry are separate phases. Head and body
consume exactly their reserved state ranges and actual ordered Await tape;
equal endpoints cannot hide missing source Await operations. The owner keeps
these phases even when the source has no Await. A matching Continue returns to
the original advance phase, retaining StatementList V and the visited-key set.

Every protocol uses the source-minted initializer proof and one physical
original-environment validator. Actual lowering supplies the retained String-key
write or original sloppy immutable-binding Ignore Reference outcome. The
constructor validates original Var and lexical names, source ranges, tape,
environments and invocation-cell aliases. A caller cannot substitute a bare
key read for the opaque initializer proof.

Analysis admits whole async enumeration in the current FunctionBody domain,
including nested whole With and ForIn. Existing foreign classic and ForOf loop
regions retain their separate boundaries. The head prefix does not move outside
the head TDZ. Empty declaration/Var completion evidence is consumed throughout
the body so awaited initialization cannot overwrite preceding StatementList V.
Global state, completion, early-error, storage and backend walks inspect the
actual head, initialization, original environments and body. The common carrier
retains the explicit foreign-owner boundary. The uncalled earlier per-protocol
carrier is [retired under T02](obsolete-for-in-carrier-removal.md).

Native execution shares the original four-field OwnKeys/VisitedKeys cursor,
BindingCell edge and physical create/advance/completion pipeline with generators.
ForIn performs no IteratorClose. Per-key environments remain fresh while normal
Await retains the original cursor, lexical chain and pending value. The existing
experimental Wasmtime GC/reference surface is required.

Private controls damage real source-produced carriers: missing or aliased cells,
incomplete head publication, missing key writes, foreign source identity,
displaced Await states and equal-extent source loss. Source, emitted-artifact and
engine cohorts cover head TDZ, lazy prototype/descriptor observations, original
Reference selection, per-key capture, Continue and whole abrupt completions.
Compilation and runtime verification remain deferred to the coherent source
batch checkpoint.
