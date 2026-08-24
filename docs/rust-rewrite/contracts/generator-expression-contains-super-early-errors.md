# GeneratorExpression `Contains super` early errors

**Status:** Focused-verified 2026-08-24

## Decision

The four `GeneratorExpression` super restrictions are one closed parser
condition:

`EarlyErrorCode::GeneratorExpressionContainsSuper`

Its wire spelling is `E_GENERATOR_EXPRESSION_CONTAINS_SUPER`. It reports phase
`Early`, native error type `SyntaxError`, and a nonempty source span under both
Script and Module goals.

The condition is true when the expression's `FormalParameters` or
`GeneratorBody` `Contains SuperProperty` or `Contains SuperCall`. Ordinary and
async arrows remain lexical traversal paths. Nested ordinary callables and
classes retain their own static-semantics boundaries.

Generator declarations, async-generator expressions/declarations, methods and
ordinary or async functions remain different production owners.

## Specification boundary

ECMA-262 2026
[Generator Function Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-generator-function-definitions-static-semantics-early-errors)
rejects a `GeneratorExpression` when its parameters or body contains
`SuperProperty` or `SuperCall`. The code combines those four bullets for this
grammar production only.

## Pinned-Boa producer and diagnostic repair

The sole producer is
`vendor/boa_parser-0.21.1/src/parser/expression/primary/generator_expression/mod.rs`.
It parses the optional binding name, parameters and complete body; applies the
earlier duplicate-parameter, `ContainsUseStrict`, strict binding-name and
parameter/body lexical-name checks; rejects a parameter list that `Contains
YieldExpression`; constructs the complete node; and then runs:

```text
if contains(&function, ContainsSymbol::Super) {
    return Err(...);
}
```

The producer-only repair changes its generic message to:

```text
generator expression cannot contain super
```

The existing `params_start_position` remains the source position. Boa appends
that coordinate, so the classifier owns the complete anchored prefix:

```text
generator expression cannot contain super at line
```

No grammar, predicate, check order, accepted source or location behavior
changes.

After the repair, the unique message occurs once across pinned Boa Rust
sources. `invalid super usage` drops from four to three raw literals: the fixed
ScriptBody producer, the generic declaration default used by generator/async
declarations, and the async-generator-expression producer. The ordinary and
async function messages remain unique, and the method-owned `invalid super
call usage` census remains eleven.

## Typed and retained boundaries

The closed front domain and parse table grow from 68 to 69 entries. One
`StartsWith` row maps the complete fixed prefix to
`GeneratorExpressionContainsSuper`. An evaluated parse-owner witness, exact
single-owner assertion, table-wide disjointness and wire-name proofs, and
`lila-ir`'s no-catch-all rejection-kind match make the addition structural.

Classifier checks keep the prefix distinct from the generic and method-owned
messages and from the separately typed function-expression prefixes. A
duplicate-export diagnostic containing the complete new prefix remains
`ModuleDuplicateExport`, proving that user-controlled text cannot forge the
code.

A real exported initializer failure crosses
`module_parse_failure_diagnostic`. A real dependency remains
`ModuleSourceIr::Rejected`, exposes no module requests, and crosses
`build_graph` with the same code, phase, error type and span. A valid
parenthesized generator-expression dependency remains a parsed graph node.

## Permanent behavior and precedence matrix

Every direct rejection source runs under Script and Module goals. The matrix
covers named and anonymous expressions, parameter and body positions,
`SuperProperty` and `SuperCall`, and ordinary/async-arrow traversal.

Positive controls preserve empty and ordinary generator expressions, body
`yield`, nested ordinary functions, string text, and a nested derived class
constructor containing its own valid `super()` call.

The parser's existing check order remains observable through typed diagnostics:

- parameter `Contains YieldExpression` precedes the super check;
- duplicate non-simple parameters precede it;
- a Use Strict Directive with non-simple parameters precedes it; and
- a formal parameter/body lexical declaration conflict precedes it.

Adjacent generator declarations, async/async-generator declarations,
async-generator expressions, and method producers remain
`ParseCode::Malformed`.

## Durable source guard

The shared super-producer guard recursively requires:

- exactly one generator-expression-specific message and no generic message in
  that producer file;
- the exact completed-node `contains(&function, ContainsSymbol::Super)` branch,
  new message and retained `params_start_position` together;
- the parameter `Contains YieldExpression` check before complete-node
  construction and the super branch after construction;
- exactly three remaining generic raw messages, including the declaration
  default and sole remaining expression producer;
- the existing ordinary/async function, class and method message censuses;
- the declaration default/override boundary remains unchanged;
- ordinary/async-arrow traversal and ordinary callable/nested-class stopping
  behavior in pinned `boa_ast`; and
- the sole parse/classifier product boundary.

Moving the unique message before complete-node construction or onto an
adjacent producer fails the bounded source shape even if literal counts remain
equal.

## Exact pinned Test262 cohort inventory

At revision `e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the dedicated ordinary
GeneratorExpression cohort is exactly zero files. A recursive inventory of all
290 JavaScript files under `language/expressions/generators/` finds no
`SuperProperty` or `SuperCall` metadata/text and no static source containing
`super(`, `super.`, or `super[`.

The four analogous files belong to `AsyncGeneratorExpression` and are excluded:

- `language/expressions/async-generator/early-errors-expression-body-contains-super-call.js`
- `language/expressions/async-generator/early-errors-expression-body-contains-super-property.js`
- `language/expressions/async-generator/early-errors-expression-formals-contains-super-call.js`
- `language/expressions/async-generator/early-errors-expression-formals-contains-super-property.js`

Therefore this lane has no honest Test262 execution variant to claim; its
direct Script/Module and retained-graph tests are the focused behavioral
evidence. This absence is an inventory result, not a skip or a passing Test262
cohort.

## Verification

The coordinated batch verifier ran:

```sh
cargo test -p lila-front --lib -- --test-threads=1
cargo test -p lila-ir modules::early -- --test-threads=1
cargo test -p lila-ir modules::graph -- --test-threads=1
```

The complete front library passes `142/142`; the relevant IR module-early and
graph groups pass `50/50` and `51/51`. `cargo xc`, source formatting, the exact
message/cohort scans and diff hygiene are green. The pinned ordinary
GeneratorExpression Test262 cohort remains zero files, so no Test262 pass is
claimed for this condition.

## Nonclaims

This lane does not classify generator declarations, async-generator
expressions/declarations, async declarations, method-owned super restrictions,
change `Contains`, add syntax, support dynamic source, alter generator
execution, refresh aggregate status, close callable grammar, or complete T07.
