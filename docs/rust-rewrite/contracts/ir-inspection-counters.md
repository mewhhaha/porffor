# IR inspection counters

`lila inspect` describes the emitted IR. Parameter and function counts include
materialized exact-context clones when lowering proves an exact call target;
source spelling alone does not authorize a clone. Executable parameter default
initializers participate in the expression census alongside function bodies.

`arguments_uses` counts `ExprIr::Arguments` object expressions.
`lexical_arguments_captures` counts the function metadata retaining the implicit
outer arguments cell. A named environment Reference can read that cell without
being an `ExprIr::Arguments` expression. The generic
`environment_identifier_reads` and `environment_identifier_calls` counters count
retained `EnvironmentIdentifierOperationIr::Read` and `Call` operations. They do
not infer an arguments object or builtin identity from an identifier name.

Builtin counters such as `aggregate_errors`, `function_proto_calls`, and
`function_proto_applies` describe intrinsic candidates recognized during
lowering. A live runtime Reference with no retained candidate contributes zero
to these counters, while the emitted call or constructor still contributes to
`calls`, `indirect_calls`, or `constructs`.

A source reference to `arguments` enables named environments for the current
whole Script. Consequently the unchanged parameter fixture has four functions,
two parameter defaults, one rest parameter, one lexical arguments capture, four
named reads, and three named calls. The unchanged function-builtin fixture keeps
its thirteen calls and three constructors; its AggregateError and call/apply
identities remain live runtime lookups. The CLI integration assertions and typed
IR controls check these facts separately from the existing Wasm execution tests.
