# FunctionExpression `Contains super` early errors

**Status:** Focused-verified, 2026-08-24

## Decision

The four ordinary `FunctionExpression` restrictions are one closed parser
condition:

`EarlyErrorCode::FunctionExpressionContainsSuper`

Its wire spelling is `E_FUNCTION_EXPRESSION_CONTAINS_SUPER`. It reports phase
`Early`, native error type `SyntaxError`, and a nonempty source span under both
Script and Module goals.

The condition is true when the expression's `FormalParameters` or
`FunctionBody` `Contains SuperProperty` or `Contains SuperCall`. Ordinary and
async arrows remain lexical traversal paths. Nested ordinary callable bodies
and nested classes retain their own static-semantics boundaries.

This code deliberately does not include FunctionDeclaration, generator,
async-function, async-generator, method, field, static-block, or whole-source
conditions. They have distinct grammar productions and pinned parser
producers, even where Boa formerly reused one diagnostic string.

## Specification boundary

ECMA-262 2026
[15.2.1, Function Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-function-definitions-static-semantics-early-errors)
rejects a `FunctionExpression` for each of these conditions:

- `FormalParameters Contains SuperProperty`;
- `FunctionBody Contains SuperProperty`;
- `FormalParameters Contains SuperCall`; and
- `FunctionBody Contains SuperCall`.

Pinned Boa represents their common result with
`contains(&function, ContainsSymbol::Super)` after parsing the complete
`FunctionExpressionNode`. The combined code names that one production-owned
rejection family; it does not collapse a different callable production.

## Pinned-Boa producer and diagnostic repair

The sole producer is in
`vendor/boa_parser-0.21.1/src/parser/expression/primary/function_expression/mod.rs`.
It parses the optional binding identifier, parameters and complete body;
applies duplicate-parameter, `ContainsUseStrict`, strict binding-name and
parameter/body lexical-name checks; constructs the complete node; and then
runs:

```text
if contains(&function, ContainsSymbol::Super) {
    return Err(...);
}
```

Before this lane it emitted `invalid super usage`, shared with ScriptBody and
four other callable producers. The producer-only repair changes its message to:

```text
function expression cannot contain super
```

The existing `params_start_position` remains the source position. Boa appends
that coordinate, so the classifier owns the complete anchored prefix:

```text
function expression cannot contain super at line
```

No grammar, predicate, check order, accepted source, or location behavior is
changed.

After the repair, the unique message occurs once across pinned Boa Rust
sources. The old generic message occurs five times: one fixed ScriptBody
producer plus the shared hoistable declaration, generator-expression,
async-function-expression and async-generator-expression producers.

## Typed and retained boundaries

The closed front domain and parse table grow from 65 to 66 entries. One
`StartsWith` row maps the complete fixed prefix to
`FunctionExpressionContainsSuper`. An evaluated parse-owner witness, exact
single-owner assertion, disjoint table witnesses, exhaustive wire-name checks,
and `lila-ir`'s no-catch-all rejection-kind match make the addition structural.

Classifier checks keep the new prefix distinct from both generic
`invalid super usage` and method-owned `invalid super call usage`. A Module
duplicate-export diagnostic containing the complete new prefix remains
`ModuleDuplicateExport`, proving user-controlled text cannot forge the code.

A real failed Module parse crosses `module_parse_failure_diagnostic`. A real
dependency is retained as `ModuleSourceIr::Rejected`, exposes no module
requests, and crosses `build_graph` with the same code, phase, error type and
span. A valid parenthesized function-expression dependency remains a parsed
graph node.

## Permanent behavior and precedence matrix

Every direct rejection source runs under Script and Module goals. The matrix
covers anonymous and named expressions; parameter and body positions;
`SuperProperty` and `SuperCall`; and ordinary, async and nested arrow
traversal.

Positive controls preserve empty and ordinary expressions, string text, and a
nested derived class constructor containing its own valid `super()` call.

The parser's existing check order is observable through typed diagnostics
and remains pinned:

- duplicate non-simple parameters precede the super check;
- a Use Strict Directive with non-simple parameters precedes it; and
- a formal parameter/body lexical declaration conflict precedes it.

Adjacent FunctionDeclaration, generator-expression, async-function-expression,
async-generator-expression and method producers remain outside the new code.

## Durable source guard

The shared super-producer guard recursively requires:

- exactly one function-expression-specific message in all pinned Boa Rust
  sources and none of the old generic message in that producer file;
- the exact completed-node `contains(&function, ContainsSymbol::Super)` branch,
  new message and retained `params_start_position` together;
- exactly five remaining generic messages, with their fixed Script or
  parameter-start positions;
- the existing class constructor, static-block, field and method message
  censuses;
- ordinary/async-arrow traversal and ordinary callable/nested-class stopping
  behavior in pinned `boa_ast`; and
- the sole parse/classifier product boundary.

Literal counts alone are insufficient: moving the unique message before body
construction or onto an adjacent callable condition fails the bounded branch
shape.

## Complete pinned Test262 cohort

The complete dedicated cohort at revision
`e9d582d6b8b13afc5ba9a676664741592b5c7f69` is exactly four files:

- `language/expressions/function/early-body-super-call.js`;
- `language/expressions/function/early-body-super-prop.js`;
- `language/expressions/function/early-params-super-call.js`; and
- `language/expressions/function/early-params-super-prop.js`.

All four are parse-negative `SyntaxError` tests and declare no execution-mode
flag. They expand to exactly eight sloppy/strict Wasm-AOT variants. Generator,
async-function, method, class-field and eval files are different producers and
are not part of this cohort.

## Verification evidence

The coordinated batch ran:

```sh
cargo test -p lila-front function_expression_super -- --test-threads=1
cargo test -p lila-front tests::known_script_and_class_super_producers_stay_structurally_reviewed -- --exact --test-threads=1
cargo test -p lila-front tests::pinned_contains_super_traversal_stays_structurally_reviewed -- --exact --test-threads=1
cargo test -p lila-ir modules::early::tests::function_expression_super_module_parse_maps_to_an_early_syntax_error -- --exact --test-threads=1
cargo test -p lila-ir modules::graph::tests::rejected_function_expression_super_dependency_keeps_its_code_through_graph_build -- --exact --test-threads=1
cargo test -p lila-ir modules::graph::tests::retained_function_expression_without_super_builds_a_real_module_graph -- --exact --test-threads=1
```

All six focused commands pass. `cargo fmt --all -- --check`, `cargo xc` and
`git diff --check` are green. The complete front library passes `129/129`; the
relevant IR early and graph groups pass `47/47` and `45/45`.

The four exact Test262 paths were run separately with `--jobs 1 --threads 1`.
Each passes `2/2`, for exactly `8/8` completed Wasm-AOT variants with every
non-success bucket at zero.

## Nonclaims

This lane does not implement or classify other callable `super` restrictions,
change `Contains`, add syntax, support eval or Function-constructor dynamic
source, alter function lowering/execution, prove a new Test262 pass, refresh
aggregate status, close callable grammar, or complete T07.
