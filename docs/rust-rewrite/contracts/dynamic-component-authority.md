# Dynamic-import component construction authority

`DynamicComponentIr` is one resolved dynamic-import occurrence: its host target
key, full phaseful request, referrer and target unit must all come from the same
graph-discovery decision. These four values are not an open record.

The fields are private. `discover_components` is their only constructor and
derives the target key from the unit selected by the graph's host-resolution
map. Public callers can inspect a component through `target_key`, `request`,
`referrer` and `target`, but cannot assemble values from unrelated graphs.

`ModuleGraphIr` likewise keeps its component vector private. Linking is the
only writer: it discovers the wider phaseful edge set, classifies runtime
participation, filters out components whose referrer cannot materialize, then
stores the result. The public `dynamic_components` method returns a read-only
slice; there is no mutable slice, vector accessor or public setter.

This boundary does not make dynamic target evaluation lazy, implement module
namespace exotic objects or close cyclic async-module semantics. Attributed
re-export retention is closed separately by the canonical module-request
identity seam. This boundary prevents already-linked component identity from
being invalidated by Rust callers while its semantic gaps remain explicit.

The source-structure regression is:

```sh
cargo test -p lila-ir --test dynamic_component_authority_structure
```
