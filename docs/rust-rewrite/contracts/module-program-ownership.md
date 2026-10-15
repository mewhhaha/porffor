# Module graphs retain their program

Each `ModuleGraph` retains its actual `ProgramInstance` in immutable, non-null
`PROGRAM`. A `ModuleRecord` retains its graph; a module-body Promise reaction
retains its record. This chain keeps the program's typed hooks, literal roots,
metadata and compiled source entries alive until the continuation is released.
The Realm used for execution and the program that drains the shared job FIFO
are separate authorities.

`RealmRecord.MODULES` is the head of a GC graph list. A graph's immutable `NEXT`
retains the previously published head. Lookup compares actual ProgramInstance
references, so a physical P2 installation cannot interpret P1 record ordinals,
replace P1's catalog, or suppress P2 initialization. The constructor rejects a
second graph for the same P and Realm; the initialization helper reuses the
existing graph. The cache never selects an ambient current P from a Realm.

Module initialization receives the explicitly calling P. The other seven
registered module helpers receive the retained record's graph P as their final
`caller_program` operand. Their helper entries bind that operand to their own
frame. A Realm import resolves the graph for its explicitly calling P, uses
that graph's retained owner when present, and uses the same calling P for its
first initialization. Both existing and first-load paths dispatch through an
immutable typed `RealmModuleImport` hook.

An old module-body reaction reads its record's graph owner and uses
`emit_owned_program_hook_dispatch`. Dispatch separately roots the selected P
and its actual typed callee. The private `InstalledHook` token borrows that
owner through the call and supplies the hook's first-P ABI operand; callers
cannot pair an unrelated function index with another owner. The ordinary
frame-based dispatch wrapper delegates to this same path. No global P or
save/restore of P is involved.

Default Promise handlers keep their defining P through `FunctionContext`.
Async and generator source resumption keeps the defining P in the retained
source callable. Thenable jobs invoke the retained method through that entry
path. ModuleJoin jobs only update their retained counter and settle their
retained promise; they read no program catalog. These paths preserve FIFO
ordering, runtime Gets/coercions/calls, execution Realm selection, literal
outcomes and unsupported dynamic-source boundaries. They do not add a generic
program capture to every PendingJob.

The existing internal `module_async_runtime_tests` seam links actual programs
against one R in one Wasmtime Store. The module ownership controls cover a
graphless P2 with an absent module hook, P2 main and explicit R job drainage,
fulfilled and rejected P1 bodies after GC, and two catalogs and graph registries
sharing a Realm. The tests also preserve the existing private lifecycle and
reentrant-evaluation controls with explicit record/P arguments. These are native
semantic controls; source inspection or formatting does not establish their
acceptance.
