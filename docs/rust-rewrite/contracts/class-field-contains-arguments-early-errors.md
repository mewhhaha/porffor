# Class-field `ContainsArguments` early errors

## Decision

The early error for an `arguments` reference in a class field initializer is
one closed condition:

`EarlyErrorCode::ClassFieldContainsArguments`

The condition applies to public and private fields, to instance and static
fields, and to public/private auto-accessor initializers. Those source forms do
not create separate diagnostics because they consume the same
`FieldDefinition` static-semantics rule and Boa emits the same exact message for
all of them:

```text
'arguments' not allowed in class field definition
```

The pinned producers are the exhaustive class-element match in
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs`.
The parser uses Boa AST's `contains_arguments` operation while it still owns the
initializer syntax. Lila classifies that rejection at the parse boundary; IR
lowering must never receive the rejected class.

## Traversal boundary

`ContainsArguments` follows lexical `arguments` capture. It therefore traverses
arrow functions nested in an initializer, but stops at ordinary, async,
generator, and async-generator function bodies. Method bodies are also
boundaries; computed method names remain traversed because they execute in the
surrounding class-definition context.

These positive boundaries are part of the contract. A classifier must not
replace the parser operation with source scanning for the token `arguments`.
String literals, property names, and nested ordinary function bodies are not
rejections.

Direct `eval` whose source later refers to `arguments` is separate T13 dynamic
source work. This code classifies only the pre-evaluation parser rejection
produced from the retained outer source AST.

## Diagnostic contract

Both Script and Module goals report:

- phase: `Early`;
- constructor: `SyntaxError`;
- wire code: `E_CLASS_FIELD_CONTAINS_ARGUMENTS`; and
- a source span from the parser rejection.

The one parse-failure table owns the exact Boa message. Adding the code extends
the exhaustive front-end domain and the IR rejection-stage projection, so an
unhandled consumer fails to compile.

## Verification boundary

Focused source tests cover class declarations and expressions, all four
public/private and instance/static field placements, public/private
auto-accessors, lexical arrow capture, and ordinary-function/method/string-name
positive controls. Retained dependency-module parsing must project the same
typed code.

The exact pinned Test262 checkpoint is the generated non-`eval` field
`*-init-err-contains-arguments.js` cohort under both
`language/expressions/class/elements` and
`language/statements/class/elements`. Direct- and indirect-`eval` cases remain
outside this bounded claim.

At `2026-08-22`, the full `lila-front` suite passes `40/40`, the focused
`lila-ir` early-error gate passes `3/3`, and all 60 exact pinned non-`eval`
files covering declaration/expression, public/private, instance/static, arrow
and nested forms pass `120/120` Wasm-AOT executions. Every parser, early-error,
lowering, runtime, backend, harness, unsupported, crash and bug bucket is zero.
