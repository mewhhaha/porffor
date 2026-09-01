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

The module-namespace source emitter uses `enumerable()` and
`non_configurable()` for its export accessors. Rendering order, descriptor
presence, completion defaults and generated JavaScript are unchanged.

```sh
cargo test -p lila-ir --test descriptor_source_text_attribute_selection_structure
cargo test -p lila-ir property_descriptor::tests::source_descriptor_attribute_methods_preserve_explicit_false_fields -- --exact
```

The recursive structure target passes `4/4`, the explicit-false rendering
witness passes `1/1`, and the shared `cargo xc`, formatting, diff,
module-boundary and task-plan checks are green.
