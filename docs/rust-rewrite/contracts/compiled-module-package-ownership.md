# Compiled module package ownership

Status: implemented as a source-equivalent Wasm-AOT compiler ownership boundary.

## Private lifecycle owner

`lila-aot-wasm/src/module/compiled_module_package.rs` owns the complete
consume-once transition from the type and global section builders, through the
finalized runtime sections and main compilation, to canonical Wasm section
assembly. The parent `module.rs` re-exports only the three construction surfaces
used by `emit.rs`: `ModuleTypeRegistry`, `ModuleGlobalSectionBuilder`, and
`ModuleAssemblySections`.

`FinalizedModuleSections`, `CompiledModulePackage`, and the raw type-section
builder have no re-export from the private child. Callers can advance the value
returned by the reviewed lifecycle but cannot name or construct an intermediate
package state through the module facade. Private fields prevent replacing the
types, globals, code, or main-local count independently.

The existing compile-time function-pointer gates remain beside the lifecycle.
They require global finalization and main compilation to consume their input
states, allow remaining function bodies to borrow only the one compiled
package, and require final module assembly to consume that package. The final
assembly retains the canonical type, import, function, table, memory, global,
export, element, code, and data section order.

## Mandatory callable-function table

Every emitted module uses the callable-function table. The obsolete planning
walk that once considered table absence is gone, and the sole emitter had
already selected the table with a literal `true`. `ModuleTypeRegistry::new`
therefore has no policy argument and always registers the JavaScript function
signature. `ModuleAssemblySections` requires a callable-function range rather
than raw encoder sections: its caller supplies the first callable function
index and count, and the private
`CallableFunctionTableSections` owner constructs exactly one `funcref` table
and its active element segment from that same range. Only the genuinely
conditional memory and data sections remain optional.

These signatures make the actual backend invariant structural. A caller can no
longer supply, omit, or independently mismatch the raw table and element
sections, choose a type registry whose indices exclude its function signature,
or construct the previously admitted table-without-elements and
elements-without-table states. Removing the dead branches and relocating the
paired section construction preserve the exact section contents and order
selected by the former literal-true path.

## Durable evidence

`compiled_module_package_structure` pins the private file owner, exact narrow
re-export, sole ownership of all seven lifecycle records/builders,
consume/borrow signatures, the mandatory callable range input, zero-policy
type-registry construction, the exact main/index-1 JavaScript/index-2
type-registration
order, private paired table construction, canonical section order, and the
single reviewed emitter call sequence. The function-pointer gates make both
strengthened construction signatures compile-time checked.
`check-module-boundaries.sh` enforces the same owner and retargets the
pre-existing runtime-root and sealed-package checks to the child.

At the original ownership-move checkpoint, the child was 292 lines with SHA-256
`e6c8aab33f1e616bfbf9ae00a7a154226885c3b1520a77c3a45694b4b6e2aaef`.
The method bodies and section order moved without semantic changes; the only
caller edit removes an intra-doc link to the intentionally un-re-exported
intermediate type. Focused source checks and their results are recorded in T02.

At the coordinated mandatory-table checkpoint, `cargo check -p lila-aot-wasm`
is green. The package structure target passes `4/4`, the obsolete-planning
target passes `3/3`, and the exact module-assembly, emitted-module-validation,
and String memory/data controls are green.

This boundary changes no Wasm type, global, function body, section ordering,
import, export, public API, JavaScript behavior, or conformance count. No Wasm
golden or broad runtime suite is claimed by the mandatory-table write-phase
evidence.
