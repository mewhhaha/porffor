# Compiled module package ownership

## Current lifecycle — 2026-10-08

`crates/lila-aot-wasm/src/module/compiled_module_package.rs` owns the transition
from open type/global construction to the completed module. The parent
`module.rs` re-exports only `ModuleTypeRegistry`, `ModuleGlobalSectionBuilder`
and `ModuleAssemblySections`. The intermediate `FinalizedModuleSections` and
`CompiledModulePackage` types have no re-export through that facade; their
runtime sections, code, retained main and program bodies are private.

The crate root re-exports `WasmArtifact`, the result already returned by the
public emitter. Callers can retain it and inspect both program bytes and its
linked `RuntimeArtifact`; internal package/global constructors stay private.
Artifact controls validate both modules and compare program imports with the
runtime's exports using one canonical Wasm type context.

`ModuleTypeRegistry::new()` registers the closed runtime type domain without a
policy argument. Its single `finalize_globals(self, globals, snapshot_roots,
module_guard_count)` transition consumes the registry and global builder. The
canonical GC finalizer declares semantic roots, optional snapshot roots and the
program's final once-guard range, then seals the ledger with its matching schema.
`FinalizedModuleSections::begin(self, first_wasm_index)` consumes that state and
starts package-owned code at the first function defined by this module.

`CompiledModulePackage::compile_main(&mut self, compilation)` supplies its own
sealed globals to `MainFunctionCompilation::compile`, retains the resulting
`EmittedFunction` and local count, and returns only `Result<(), EmitError>`.
The emitter cannot transfer that main into another package. Main is compiled
only when this module owns program bodies; the runtime-only module has no main.
A second successful main compilation into the same package is rejected.

`append_functions(&mut self, runtime_functions, program_functions)` publishes
runtime bodies into the package's `ModuleCode` and retains program bodies
privately. `append_to_module(self, ...)` consumes the complete package, publishes
its retained main if present, then publishes program bodies. This preserves the
planned **runtime → main → program** function order, including the split
runtime/program modules. `ModuleCode::push` consumes each emitted body and derives
its function declaration, code bytes and attribution together; its planned-entry
check binds publication to the declared index. Function-pointer gates pin the
consume-once begin/assembly transitions and the mutable-borrow main/body APIs.

## Sealed globals and canonical sections

`FinalizedModuleGlobals::defined_section(&self, imported)` returns an opaque
borrowed `impl Section + '_` view. The private view retains the exact ledger and
imported prefix; only its encoder constructs a raw `GlobalSection`, inside the
GC module. Assembly cannot obtain or clone that raw section, append globals to
it, or replace the view's ledger. The associated schema remains a borrow of the
same sealed owner.

A program module imports the runtime-global prefix and defines only the globals
following that prefix. `runtime_global_types()` derives the import types from
the sealed ledger up to the program once guards. The split preserves the shared
root indices while keeping program guards in the program module.

`ModuleAssemblySections` accepts a defined-function range. Its private
`CallableFunctionTableSections` owner builds the declarative element segment
for that range; actual calls use registered typed function references. The
current owner allocates no dispatch table. Assembly emits type, import,
function, optional memory, global, export, element, optional data-count, code
and optional data sections in that order. Function declarations and code arrive
from the same `ModuleCode`; the caller cannot supply independent sections.

## Current evidence and remaining verification

`compiled_module_package_structure` follows the private owner, narrow re-export,
canonical global finalizer, retained main, runtime/main/program publication
order, opaque imported-prefix global view and current lifecycle gates. Its
negative escape checks include the emitter's `literal_roots` and `metadata`
children. `check-module-boundaries.sh` enforces the same physical owners and
sealed-package invariants.

The integrated Rust type check and current source guards pass. All 24 focused
GC, linked-runtime, artifact and ownership controls pass. These validate both
linked modules, exact canonical import/export types and pooled helper byte
independence. Separate Unicode parity checks pass 2/2, and the three native
Unicode plus seven backreference-folding cohorts pass. Full IR passes 2,166
checks, with one existing documentation example ignored. These results do not
establish whole-corpus byte identity or close T05, runtime capability gaps or
full conformance acceptance. See the [current task checkpoint](../../../tasks/README.md#closure-audit-and-verification--2026-10-08).

## Historical checkpoints

The following records retain their original evidence scopes. Their scalar ABI,
table layout, source-only status and verification counts describe those earlier
checkpoints, not the current implementation or a fresh verification run.

### Callable body and invocation roles — 2026-10-04 dry source

The actual function planner now owns callable origin, the existing source
protocol and both function-index spaces together. User, native and prepared
Script origins project their permitted body role from that one owner. The
builder retains the planned entry while emitting the body; a completed body
carries it into the joint declaration/code publisher. A runtime helper remains
an existing closed helper identity, rather than acquiring callable parameter
access merely because it returns multiple values.

Private ordinary, generator, async, async-generator and prepared-Script input
bundles drive the actual direct/indirect calls. Ordinary inputs contain semantic
this/new.target; each suspended family accepts only its own activation input.
Prepared Script inputs retain the complete lexical/variable/private environment
and direct-eval context lifecycle. The scalar encoder preserves the existing
seven- or ten-word shape and seventeen static signature ordinals. Internal
activation inputs cannot be passed as semantic new.target through these bundles.

Dynamic dispatch loads environment, family flags and table identity from one
validated Function object. A private branch encloses the matching role projection
and actual call. Resume calls consume the saved record's family-specific fields.
New.target first follows retained direct-eval or Arrow lexical ownership; an
activation-backed generator, async or async-generator body otherwise evaluates
it as undefined. Async Arrows therefore retain their ordinary lexical owner's
new.target across await rather than reading an activation word.

Paired strict/sloppy semantic controls cover ordinary calls/construction,
alternate newTarget, all suspended families, escaped lexical/direct-eval Arrows
and prepared created-Realm entries. Only source inspection and Rust formatting
are complete. Compilation, Wasm validation, execution and broad verification
remain pending. This joins the current scalar ABI's semantic role authority;
it does not move values, callable references, allocations, completions, frames,
jobs or host roots/decoding to Wasm GC. The atomic semantic migration and full
T05 acceptance remain open.

### Function declarations follow actual bodies — 2026-10-04 dry source

ModuleCode now privately owns both FunctionSection and CodeSection. Its sole
push consumes an actual EmittedFunction, derives the declaration from that
body's closed identity, and appends declaration, body and attribution together.
Prepared Script bodies carry a distinct identity with the existing ten-parameter
signature; ordinary user/builtin/host bodies retain their seven-parameter shape
and runtime helpers retain their existing exhaustive signature authority.
All seventeen static type ordinals and shapes are unchanged. Prepared Script
names and the script report category are preserved.

The final module package consumes both sections from this one body owner.
Assembly can no longer accept an independent function section. The positional
prepared-body index-range inference and second declaration list are removed,
along with the count assertion that checked those independent lists. Actual
main/source/prepared/builtin/shared-stub/host/helper body order and conditional
helper selection remain unchanged. Existing package guards and unit consumers
are maintained around the new joint lifecycle; no new test groups are added.

This earlier checkpoint closed the consumed declaration/body authority. It
left the seven-I64 semantic context/activation roles for the newer callable-role
source above. Migration of callable/value/completion, frame/job, host roots and
decoding to GC references remains open. Current product semantic
allocations remain linear. Compilation, Wasm validation, focused runtime and
complete phase3/atomic migration acceptance remain pending.

### Original ownership move and mandatory-table evidence

At the original ownership-move checkpoint, the child was 292 lines with SHA-256
`e6c8aab33f1e616bfbf9ae00a7a154226885c3b1520a77c3a45694b4b6e2aaef`.
The method bodies and section order moved without semantic changes; the only
caller edit removes an intra-doc link to the intentionally un-re-exported
intermediate type. Focused source checks and their results are recorded in T02.

At the coordinated mandatory-table checkpoint, `cargo check -p lila-aot-wasm`
is green. The package structure target passes `4/4`, the obsolete-planning
target passes `3/3`, and the exact module-assembly, emitted-module-validation,
and String memory/data controls are green.

At that checkpoint, the boundary changed no Wasm type, global, function body,
section ordering, import, export, public API, JavaScript behavior, or conformance
count. No Wasm golden or broad runtime suite is claimed by the mandatory-table
write-phase evidence.

### 2026-10-03 consumed static signature authority proposal

The queued source-only foundation replaces the module's seventeen handwritten
function-type declarations and parallel type-index constants with one closed
`StaticSignature` row domain. Each row supplies its existing ordinal, parameter
shape and result shape; it emits the ordered domain and exhaustive definition.
`ModuleTypeRegistry` registers those definitions before the existing runtime
GC anchor declarations. Main, prepared Script, helper and host-import function
sections and indirect calls consume the same domain. Function parameter counts
are derived from those definitions. All seventeen ordinals and shapes remain
unchanged, including equal shapes registered at distinct ordinals.

This is a consumed phase-3 foundation for the atomic semantic switch. It does
not migrate JavaScript values, closures, completions, suspended frames, jobs or
host decoding to GC references, and it does not reclaim the current linear
semantic heap. T05 and the URI allocation-pressure failure remain open. A new
signature row without a complete shape fails the macro contract, and an ordinal
that disagrees with declaration order fails the constant assertion. The
existing package-ownership and function-state structural assertions retain
their original lifecycle and capability purposes with signature expectations
updated to the new authority.

The proposal is authored against the approved batch-2 composition as a virtual
future base plus unchanged protected MAIN inputs. No actual future Source
manifest exists at preparation, and actual-base binding is deferred. Rustfmt
and source-only checks are preparation evidence; compilation, those existing
Rust tests, focused execution and fresh broad verification remain UNRUN.
There is no inherited compile or runtime pass and no change to published
conformance counts.
