# Body compilation, entry roots and module assembly owners

## Current owners — 2026-10-08

The emitter delegates body compilation, entry lifecycle and physical module
assembly to private children. They share the existing `FunctionBuilder`
implementation and its typed owners. The 2026-10-08 batch gives pooled literal
initialization and completed-function metadata their own children within those
owners.

`emit/body_compilation.rs` owns source, prepared-Script, standard and host
body compilation. Callable compilation combines the actual planned entry
and body as a `CompletedCallableBody`, then returns an `EmittedFunction` carrying
that declaration identity. Main compilation uses
`MainFunctionCompilation::compile` with the finalized package's own globals.
`CompiledModulePackage::compile_main` retains the result privately and returns
only `Result<(), EmitError>`; final assembly publishes runtime bodies, retained
main and program bodies in their planned order. Static-data installation,
heap-cursor and current-Realm initialization stay with their body compiler.

`emit/body_entry.rs` owns registered helper entry, current Environment
initialization, derived activation and body-entry cleanup. Helper entry checks
the actual `RuntimeHelperId` supplied by the builder's declaration before
consuming its body. Current lexical and private Environments, direct-eval
context, callable entry and argument-list roots retain their typed GC lifetimes.
Its private `async_generator_structured_owner` child retains the existing
structured resume-scope operation.

`emit/body_entry/literal_roots.rs` owns `initialize_gc_literal_roots`, the private
`emit_pooled_string_slot`, and `compile_pooled_strings_initialize_helper`. The
runtime helper constructs the pooled-string table, runtime strings, well-known
symbols and registry. Program main fills the subsequent program string slots
from its passive data. The initializer is visible within `crate::emit` for its
body-compilation consumer; the slot writer remains private. These methods keep
the runtime/program pool boundary and typed root lifetime in one physical owner.

`emit/module_assembly.rs` owns physical assembly and its private optional
import-index allocator. The bounded builtin-emission loop in `emit.rs` calls it
directly. This owner chooses runtime/program imports and bodies, registers typed
helpers, finalizes globals and delegates final sections to the sealed package.
The program module imports the shared runtime-global prefix; the package emits
the remaining global definitions through the opaque borrowed view described in
[compiled module package ownership](compiled-module-package-ownership.md).

`emit/module_assembly/metadata.rs` borrows the completed `ModuleFunctionTable`.
`append_function_attribution` derives artifact summaries, debug attribution and
the optional size report from that table. `append_function_name_and_check_budget`
emits the custom name section and checks the optional function-body budget.
Their calls stay at the respective original positions around the supported-values
producer check and all twelve Intl data custom-section writes, which remain in
`module_assembly.rs`. This extraction preserves the intended diagnostic and
section order while bringing assembly below its existing 1,950-line budget.
The metadata child has its own 100-line ceiling.

## Source witnesses and verification scope

Compiler fingerprints capture the children; service provenance recipes bind
the physical assembly source. Package escape checks include the parent,
body-compilation and body-entry owners, literal roots, assembly and metadata.
The function-module-state witness includes literal roots and retains its closed
role and planned-root checks. The Get/Proxy Realm witness continues to follow
the caller Environment, actual error Realm resolution, validated Proxy slots
and whole Completion. Its existing created-Realm semantic fixture is unchanged.
Module guards follow these physical owners, retain their consumers and enforce
the existing measured budgets.

The integrated Rust type check and current source guards pass, as do all 24
focused GC, linked-runtime, artifact and ownership controls. The artifact checks
validate both linked modules, exact canonical import/export types and pooled
helper byte independence. Unicode parity checks pass 2/2; three native Unicode
and seven backreference-folding cohorts pass. Full IR passes 2,166 checks, with
one existing documentation example ignored. Whole-corpus byte identity and full
task acceptance remain open; see the [current task checkpoint](../../../tasks/README.md#closure-audit-and-verification--2026-10-08).

## Historical extraction evidence — 2026-10-06

The original T02 source batch moved body compilation, body entry and module
assembly out of `emit.rs` into three private children. At that checkpoint,
exported main used `MainFunctionCompilation::compile_into`, and pooled literal
roots were still physically in `body_entry.rs`. Those lifecycle and placement
facts are superseded by the current owners above.

Beforeimages retained the final Root-owned twelve-component emitter bytes.
Source comparison checked the complete moved bodies, native dispatch and
custom-section block against those bytes. Existing module guards also registered
the preceding standard helper owners and the concurrent optional-chain,
conditional-flow and object-literal lowering owners. The extraction introduced
no new semantic control.

That source batch ran no Cargo, tests, runtime, guards or data generators.
Isolated formatting and source receipts did not establish emitted-artifact
equivalence. Compilation, focused semantic regressions, representative artifact
comparison and the shared broad checkpoint were still required at that date;
no later verification result is attributed to that historical checkpoint.
