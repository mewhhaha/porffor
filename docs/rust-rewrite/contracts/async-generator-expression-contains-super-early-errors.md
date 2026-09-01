# AsyncGeneratorExpression `Contains super` early errors

**Status:** Focused-verified 2026-09-01

## Decision

The four `AsyncGeneratorExpression` super restrictions are one closed parser
condition:

`EarlyErrorCode::AsyncGeneratorExpressionContainsSuper`

Its wire spelling is `E_ASYNC_GENERATOR_EXPRESSION_CONTAINS_SUPER`. It derives
phase `Early` and native error type `SyntaxError` through the exhaustive
`lila-ir` rejection projection.

The condition is true when the expression's `FormalParameters` or
`AsyncGeneratorBody` `Contains SuperProperty` or `Contains SuperCall`.
Ordinary and async arrows remain lexical traversal paths. Nested ordinary
callables and classes retain their own static-semantics boundaries.

Async-generator declarations, synchronous generator expressions, ordinary
async functions and methods remain different production owners.

## Specification boundary

ECMA-262 2026
[Async Generator Function Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-async-generator-function-definitions-static-semantics-early-errors)
rejects an `AsyncGeneratorExpression` when its parameters or body contains
`SuperProperty` or `SuperCall`. This code combines those four bullets for this
grammar production only.

## Pinned-Boa producer

The sole producer is
`vendor/boa_parser-0.21.1/src/parser/expression/primary/async_generator_expression/mod.rs`.
It parses the optional binding name, parameters and complete body; applies the
parameter `Contains YieldExpression` and `Contains AwaitExpression` checks and
the remaining callable early errors; constructs the complete node; and then
runs:

```text
if contains(&function, ContainsSymbol::Super) {
    return Err(...);
}
```

The producer-only repair changes its generic message to:

```text
async generator expression cannot contain super
```

The existing `params_start_position` remains the source position. Boa appends
that coordinate, so the classifier owns the complete anchored prefix:

```text
async generator expression cannot contain super at line
```

No grammar, predicate, accepted source, check order or location behavior
changes. On current head, the generic `invalid super usage` literal has one
pinned Boa owner: the fixed ScriptBody check. The shared declaration hook is
required and has no default.

## Typed and retained boundaries

The closed front domain and parse table grow from 69 to 70 entries. One
`StartsWith` row maps the complete fixed prefix to
`AsyncGeneratorExpressionContainsSuper`. The subsequent
AsyncFunctionDeclaration, GeneratorDeclaration and AsyncGeneratorDeclaration
lanes bring the current table to 73 rows with the exact `54/18/1`
`ContainsAll` / `StartsWith` / `Exact`
population. An evaluated
parse-owner witness, exact single-owner assertion, table-wide disjointness and
wire-name proofs make the classification structural. The exhaustive
`EarlyErrorCode` projection in `lila-ir` must also assign each new variant a
rejection kind before the workspace builds.

Classifier checks keep the prefix distinct from the synchronous generator,
ordinary async-function, typed declaration and method-owned messages. A
duplicate-export diagnostic containing the complete new prefix remains
`ModuleDuplicateExport`, so user-controlled text cannot forge the code.

A real exported initializer failure crosses
`module_parse_failure_diagnostic`. A real dependency remains a `ModuleSourceIr`
whose retained parse is `ModuleParse::Rejected`, exposes no module requests and
crosses `build_graph` with the same code, phase, error type and span. A valid
parenthesized async-generator-expression dependency remains a parsed graph
node.

## Permanent behavior and precedence matrix

Every direct rejection source runs under Script and Module goals. The matrix
covers named and anonymous expressions, parameter and body positions,
`SuperProperty`, `SuperCall` and ordinary/async-arrow traversal.

Positive controls preserve empty and ordinary async-generator expressions,
body `await` and `yield`, nested ordinary functions, string text and a nested
derived class constructor containing its own valid `super()` call.

The parser's existing check order remains observable through typed
diagnostics:

- parameter `Contains YieldExpression` precedes the super check;
- parameter `Contains AwaitExpression` precedes the super check;
- duplicate non-simple parameters precede it;
- a Use Strict Directive with non-simple parameters precedes it; and
- a formal parameter/body lexical declaration conflict precedes it.

GeneratorDeclaration and AsyncGeneratorDeclaration have separate typed owners.
Method producers remain `ParseCode::Malformed` until their own conditions
receive typed owners.

## Durable source guard

The shared super-producer guard recursively requires:

- exactly one async-generator-expression-specific message and no generic
  message in that producer file;
- the exact completed-node `contains(&function, ContainsSymbol::Super)` branch,
  new message and retained `params_start_position` together;
- parameter-Yield before parameter-Await, both before complete-node
  construction, and the super branch after construction;
- exactly one generic raw message in ScriptBody;
- one required declaration-message hook and one production-owned override for
  each of the four declaration implementations;
- the existing declaration, expression, class and method message censuses;
- ordinary/async-arrow traversal and ordinary callable/nested-class stopping
  behavior in pinned `boa_ast`; and
- the sole parse/classifier product boundary.

Moving the unique message before complete-node construction or onto an
adjacent producer fails this source shape even if literal counts remain equal.

## Exact pinned Test262 cohort

The complete dedicated cohort at suite pin
`aa55200d1310384c5cf69ea95b2a2ecba457007b` is exactly four files:

- `language/expressions/async-generator/early-errors-expression-body-contains-super-call.js`;
- `language/expressions/async-generator/early-errors-expression-body-contains-super-property.js`;
- `language/expressions/async-generator/early-errors-expression-formals-contains-super-call.js`; and
- `language/expressions/async-generator/early-errors-expression-formals-contains-super-property.js`.

All four are unflagged parse-negative `SyntaxError` tests and expand to eight
sloppy/strict Wasm-AOT variants. Declaration, method and dynamic-eval files
belong to other producers and are excluded.

## Verification

The focused checkpoint ran:

```sh
cargo check -p lila-front
cargo check -p lila-ir
cargo check -p lila-aot-wasm
cargo test -p lila-front --lib async_generator_expression_super -- --test-threads=1
cargo test -p lila-front --lib tests::known_script_and_class_super_producers_stay_structurally_reviewed -- --exact --test-threads=1
cargo test -p lila-front --test parse_failure_pattern_structure -- --test-threads=1
cargo test -p lila-ir modules::early::tests::async_generator_expression_super_module_parse_maps_to_an_early_syntax_error -- --exact --test-threads=1
cargo test -p lila-ir modules::graph::tests::rejected_async_generator_expression_super_dependency_keeps_its_code_through_graph_build -- --exact --test-threads=1
cargo test -p lila-ir modules::graph::tests::retained_async_generator_expression_without_super_builds_a_real_module_graph -- --exact --test-threads=1
```

The package checks are green. The direct front filter passes `4/4`, the shared
producer guard passes `1/1`, and the parse-pattern structure target passes
`4/4`. The exact IR early witness and both graph witnesses are green. The four
exact Test262 paths pass all `8/8` applicable sloppy/strict Wasm-AOT
executions. This is focused evidence only; no broad aggregate or published-
status claim follows.

## Nonclaims

This lane does not classify generator or async-generator declarations,
method-owned super restrictions, change `Contains`, add syntax, support dynamic
source, alter async-generator execution, refresh aggregate status, close
callable grammar or complete T07.
