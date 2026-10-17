# Native global Script Engine integration

This is an unapplied, uncompiled Engine component layered after the ABI7 global
Script IR/AOT packet and typed RealmScriptInvocation successor. It has no native
acceptance receipt and does not claim product support or conformance completion.
The earlier sealed ABI6 foundation and its native results remain separate.

## One armed source-compilation authority

`NativeSourceCompilerContext` replaces the Function-only context. The Store owns
one Arc containing the actual already-linked linker, exact R Arc, selected Intl
profile, lowering HostSurfacePolicy, and original execution-control Arc. It is
armed at the existing post-R-link seam only when the actual executable retains
its known compilation policy and runtime. Function and global Script callbacks
clone this same Arc. No callback infers a policy, allocates a substitute Realm,
re-arms a deadline, or caches a collection of dynamic sources/modules.

The Store field is `native_source_compiler`; isolated/raw test Store builders
leave it explicitly None. Raw external artifacts without the known authority
fail closed. Function-specific request/factory/outcome algorithms remain in
their own module; their context type/observer access is the only integration
change, apart from the Store field rename.

## Exact global request and executable P ownership

The distinct `CompileGlobalScript` callback admits only kind 0 (indirect eval)
or 1 (Realm Script), a non-null canonical String, target RealmRecord and actual
calling ProgramInstance. Declaration-owned GC projections validate field counts,
storage and nullability; actual request operands and emitted factory signatures
must share the same canonical GC types. The source is read in its original
UTF-16 code-unit domain. The actual calling P additionally retains an OwnedRooted
reference for the whole request.

The callback performs the real role-aware front end and full IR lowering on
every request. Its exact source, role and actual root ScriptId are compared to
the private compiler witness before AOT emission/factory execution. Emission
retains the identical R Arc/key/bytes/profile; only fresh P bytes enter the
existing bounded native compiler. The original Engine/Store/linker instantiate
P2. A start section is rejected. Actual exported initializer and factory types
are checked before invocation; the initialized P2 must match the original core
schema. Neither main nor a new Realm is invoked.

The host calls the owned P2 PreparedScript factory with P2, original kind,
original GC String and actual target Realm. The first compiled source row and
factory registry witness ensure ordered dispatch selects the actual root.
Nested prepared/direct/Shadow entries remain available under their existing
owners. P2's existing prepared-Script emitter owns Realm entry/restoration,
strict/sloppy environment creation, declaration admission and whole Completion.
The host accepts Entered=2 even on JavaScript Throw, validates Normal/Throw,
zero local target and tagged value shape, and retains any outgoing reference
with OwnedRooted before dropping the temporary RootScope. It re-roots only in
the receiving Caller scope.

Only ParseDiagnosticKind::MalformedJavaScript can return SyntaxError=1 with a
non-null exact GC message. Parser capability errors, every lowering rejection,
invalid compiler output, bad ABI/status, resource and deadline failures trap.
The AOT boundary creates the actual error using the passed target Realm's
intrinsic SyntaxError prototype and unchanged message. No message-text heuristic
or user-mutable constructor supplies error classification or identity.

Direct eval still requires caller parse/environment authority and has no global
kind. ShadowRealm source misses retain their separate result/error-wrapping
contract. Prepared sources for both remain intact. Indirect non-String values
retain ordinary passthrough, and Realm Script ToString remains in the existing
actual callable before the host callback. A replaced evalScript method remains
an ordinary call with its original receiver and arguments.

## Verification prepared, not executed

Seven production Engine controls observe actual per-Engine global/Function
callback counts and assert that required native source is absent from the
finite prepared guard. They cover strict/sloppy global declarations and lexical
conflicts, target-Realm syntax after constructor poisoning, ordered/coercive
throws, replaced methods, retained closures with GC and reentry into older P
and native Function code, and 257 distinct UTF-16 sources with only two selected
native factories and retained loop effects. One inspected outer IR is emitted
once per observed execution. Six lower-level host controls cover closed kinds,
canonical signatures, exact code units/original budget, OwnedRooted replies,
protocol violations and missing-context rejection. All thirteen are pending
compilation/native execution.

Existing indirect-eval selectors retain their identities while assertions now
check native results and catchable JavaScript Throw. Worker failure-taxonomy
fixtures use genuine remaining direct-eval/Shadow misses and an unsupported
Realm Script dynamic import, rather than treating newly supported global source
as an error. The Function-positive expectation hunk is coordinated separately
with its current owner. No old selector, capability/error-category assertion,
fixture skip or source ceiling is removed to manufacture green results.

ABI7 raw-package tests remain envelope controls; genuine old-R/new-P typed-link
rejection, all-suite positive-ID retention, independent hosts, large-instance
code/data reuse, and full Test262 acceptance are separate pending gates. There
is no AST-shape cache, emitted interpreter/parser, VM, deadline reset, source
ceiling increase, mutable global current-P state or 65k-instance acceptance
claim in this component.

The component's composed source check preserves the existing module-assembly
raw-line cap: its unchanged optional host-function index allocator now lives
in a private import-index owner, reducing the assembly owner to 1,949 lines.
This is a fresh successor over the sealed AOT packet; old packet bytes are
unchanged. The full inherited boundary script is separately recorded because
its physical-file traversal cannot enumerate sparse source symlinks and several
existing guards still name earlier assembly/literal-root ownership. Those
results do not replace the passing new boundary fragment or native gates.

The final guard-only materializations copy only Rust owners and the boundary
script into private read-only regular files; vendor/data resources remain links.
They create no tracked hardlinks, Git metadata or build tree. The baseline and
component report the identical nine inherited assembly/literal-root ownership
failures. The source-symlink traversal failure disappears on the materialization.
Those nine failures remain explicit acceptance work, with no waiver. The root
owned test-fingerprint policy successor must also cover the new cfg(test) source
owners while preserving production source/data and Cargo rerun coverage.
