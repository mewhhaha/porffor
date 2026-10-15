# Typed ProgramInstance hook dispatch (ABI6 component)

Status: **UNAPPLIED, NON-MERGEABLE** until the complete entry, data, retained-job
and same-Store host integration is closed and natively verified. This component
is based on frozen `eb1ca63c9dbf09148703ccfbd0ca35d9629753e7` selected files and the
separate sealed ProgramInstance ABI6 type API. It does not belong to the ABI5
bug/unsupported checkpoint and makes no runtime repair or conformance claim.

## Dispatch invariant

A hook is selected from the actual frame's explicit `ProgramInstanceLocals`.
`emit_program_hook_dispatch::<Role>` copies that owner into a separate GC root,
reads `ProgramInstanceLocals::hook::<Role>` into a nullable typed function local,
and preserves the existing installed/absent control paths. Inside the installed
branch, `ref.as_non_null` initializes a new exact `FunctionLocal<Role>`.

`InstalledHook<'owner, Role>` owns that function local and borrows the same rooted
owner. Its fields are private to `program_hooks`; it has no enum constructor,
public mint function, raw index, clone implementation, or independent owner input.
The callback can mutably borrow its builder without borrowing the builder's
current-owner state. The non-null callee is cleared in the installed branch;
the nullable candidate and copied owner are cleared after the branch. A compiler
error abandons the unfinished generated function.

Reentry can publish or enter another ProgramInstance without replacing this
callee or its owner. There is no mutable shared hook global, save/restore bridge,
or last-installed/current-P process selector in this dispatch implementation.
The frame's selection policy remains the entry integration's responsibility:
source entry selects its captured defining P; native entry inherits its caller.
A resumed job must first bind its retained owner.

## Registered helper ABI

The registry's five `@ Role` annotations are the single argument/body projection
source. A compile-time assertion checks that the role's hook and concrete typed
signature name the same actual helper row. Its first ABI operand is non-null
`ProgramInstance`; acquired parameters retain it in `HookProgramParameter<Role>`.
`HelperParameters::caller_program()` exposes that rooted operand to the common
helper-entry binder. Existing semantic `HookArguments::new(...)` operands retain
their order and do not accept a caller-selected owner.

| Role | Registered result | Existing result witness |
| --- | --- | --- |
| PreparedDynamicFunctionHook | StatusCompletion | StatusCompletionCallResult |
| PreparedScriptHook | StatusCompletion | StatusCompletionCallResult |
| RealmModuleImportHook | RealmImport | RealmImportCallResult |
| ModuleBodyReactionHook | Void | () |
| RegExpProgramCandidateHook | RegExpCompile | RegExpCompileCallResult |

`HookArguments` is closed and sealed separately from `HelperArguments`.
`RuntimeSchema::call_hook<A>` requires `InstalledHook<A::Role>`, then emits the
owner from that token, the row's semantic operands, the token's exact typed
function reference, registered `call_ref`, and the registered result witness.
It never calls the ordinary Completion-only `call_reference` API. A mismatched
hook role cannot satisfy the token type. Hook argument types cannot satisfy
`call_helper` or `return_call_helper`.

Ordinary R helper rows that may call JavaScript need an explicit final
`caller_program:Program` field. Its argument projection is
`&ProgramInstanceLocals`, its acquired parameter is `GcLocal<ProgramInstance>`,
and its ABI operand is non-null. The transitive helper/call-site census is a
separate required join; no optional-owner fallback supplies missing edges.
`PooledStringsInitialize` has `{}` parameters and `Arguments::new()`: genuine R
pool initialization must run before a ProgramInstance exists and carries no P.

## Effects and integration joins

The four selected consumers only replace enum-valued dispatch with a generic
role. The separately owned prepared-Script consumer has a minimal join patch.
Ordered constructor ToString, exceptions, prepared catalogue misses, status and
completion stores, Realm/module behavior and RegExp fallback behavior remain
as before. A missing hook retains the existing absent path. This migration does
not implement general dynamic-source compilation or widen any prepared catalogue.

Root must remove ProgramHookGlobals and its accessor/initialization/counts,
replace old global registration assertions with immutable typed ProgramInstance
field assertions, and remove the old `emit_install_program_hooks` call in main.
The exact deletion/check deltas are supplied outside the candidate file tree.
`module_assembly` imports/exports globals through `RuntimeLayout`'s actual typed
list; removal of the globals shortens that list automatically. Hook helper bodies
and dense absent slots remain necessary for immutable P field construction.
Root must construct/publish the ProgramInstance before entering user code and
bind the explicit owner at every hook helper entry.

## Verification boundaries

The packet runs a structural source control with deliberate rejected mutations,
exact-toolchain rustfmt, whitespace/patch checks, selected-base hashes, and source
HEAD/status fences. Those checks do not expand Rust macros or validate the ABI.
The focused Rust registry tests are written but have not been compiled or run.
They assert all five exact role/result types, non-null first owner operands,
ordinary call rows' final owner operands, and owner-independent R string bootstrap.

Required later acceptance includes one complete native compile, actual Wasm
schema/type/link validation, wrong-role/direct-route compile rejection controls,
absent-hook behavior, two Ps retained across reentry and forced GC, defining versus
calling P selection, retained module/job ownership, literal/metadata collisions,
and old/current R/P admission. Preserve the exact observed conflict reproducer:
P2 installing Function('return 23') must not change P1's old Function('return 17')
result from 17 to 23. Keep the already passing same-Store typed-function transport
control. No formatting, source-control or ABI-number result proves that conflict
fixed.
