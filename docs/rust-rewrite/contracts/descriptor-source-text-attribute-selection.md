# Descriptor source-text attribute selection

`DescriptorSourceText` exposes each statically known property attribute through
a named selection: `writable`/`non_writable`,
`enumerable`/`non_enumerable`, and
`configurable`/`non_configurable`. None accepts a boolean parameter.

Each method writes exactly one `Presence::Present(true)` or
`Presence::Present(false)` value. Explicit false therefore remains distinct
from an absent field, while a call site cannot transpose an unlabelled boolean
or require a reader to remember which value was selected. The existing
`DataSide`/`AccessorSide` typestate remains the authority for which descriptor
fields may coexist: only the data builder exposes the writable pair.

Module namespace exports no longer pass through this builder: the namespace
cell is a linker-recognized export-reader table, and the backend module
namespace exotic object owns the writable, enumerable, non-configurable export
attributes as labelled `StoredPropertyAttributes` fields. `modules/namespace.rs`
no longer renders a `DescriptorSourceText` value: JavaScript Source Text Modules
reject source imports, and no ordinary tag object substitutes for a real module
source representation. See the [source-phase contract](source-text-module-source-phase-rejection.md).

```sh
cargo test -p lila-ir --test descriptor_source_text_attribute_selection_structure
cargo test -p lila-ir property_descriptor::tests::source_descriptor_attribute_methods_preserve_explicit_false_fields -- --exact
```

At the earlier descriptor checkpoint, the recursive structure target passed
`4/4`; the explicit-false rendering witness passed `1/1`, and the shared
`cargo xc`, formatting, diff, module-boundary and task-plan checks were green.
Those results retain their original source scope. The 2026-10-03 module-source
retirement maintains the affected assertions; current compilation and execution
remain pending.
