# AsyncFunctionExpression `Contains super` early errors

**Status:** Product condition and GeneratorExpression-updated shared producer
census focused-verified 2026-08-24

## Decision

The four `AsyncFunctionExpression` super restrictions are one closed parser
condition:

`EarlyErrorCode::AsyncFunctionExpressionContainsSuper`

Its wire spelling is `E_ASYNC_FUNCTION_EXPRESSION_CONTAINS_SUPER`. It reports
phase `Early`, native error type `SyntaxError`, and a nonempty source span under
both Script and Module goals.

The condition is true when the expression's `FormalParameters` or
`AsyncFunctionBody` `Contains SuperProperty` or `Contains SuperCall`. Ordinary
and async arrows remain lexical traversal paths. Nested ordinary callables and
classes retain their own static-semantics boundaries.

Async declarations, async-generator expressions/declarations, generator forms,
methods and ordinary functions remain different production owners.

## Specification boundary

ECMA-262 2026
[Async Function Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-async-function-definitions-static-semantics-early-errors)
rejects an `AsyncFunctionExpression` when its parameters or body contains
`SuperProperty` or `SuperCall`. The code combines those four bullets for this
grammar production only.

## Pinned-Boa producer and diagnostic repair

The sole producer is
`vendor/boa_parser-0.21.1/src/parser/expression/primary/async_function_expression/mod.rs`.
It parses the optional binding name and parameters, applies the parameter
`Contains AwaitExpression` check, parses the complete body, applies the earlier
duplicate-parameter, `ContainsUseStrict`, strict binding-name and parameter/body
lexical-name checks, constructs the complete node, and then runs:

```text
if contains(&function, ContainsSymbol::Super) {
    return Err(...);
}
```

The producer-only repair changes its generic message to:

```text
async function expression cannot contain super
```

The existing `params_start_position` remains the source position. Boa appends
that coordinate, so the classifier owns the complete anchored prefix:

```text
async function expression cannot contain super at line
```

No grammar, predicate, check order, accepted source or location behavior
changes.

After the repair, the unique message occurs once across pinned Boa Rust
sources. The later GeneratorExpression lane gives that production its own
message. On current head, `invalid super usage` occurs three times: the fixed
ScriptBody producer, the generic declaration default used by generator/async
declarations, and the async-generator-expression producer. All four typed
function messages remain unique, and the method-owned `invalid super call
usage` census remains eleven.

## Typed and retained boundaries

The closed front domain and parse table grow from 67 to 68 entries. One
`StartsWith` row maps the complete fixed prefix to
`AsyncFunctionExpressionContainsSuper`. An evaluated parse-owner witness,
exact single-owner assertion, table-wide disjointness/wire-name proofs and
`lila-ir`'s no-catch-all rejection-kind match make the addition structural.

Classifier checks keep the prefix distinct from the generic and method-owned
messages. A duplicate-export diagnostic containing the complete new prefix
remains `ModuleDuplicateExport`, proving that user-controlled text cannot forge
the code.

A real exported initializer failure crosses
`module_parse_failure_diagnostic`. A real dependency remains
`ModuleSourceIr::Rejected`, exposes no module requests, and crosses
`build_graph` with the same code, phase, error type and span. A valid
parenthesized async-function-expression dependency remains a parsed graph node.

## Permanent behavior and precedence matrix

Every direct rejection source runs under Script and Module goals. The matrix
covers named and anonymous expressions, parameter and body positions,
`SuperProperty` and `SuperCall`, and ordinary/async-arrow traversal.

Positive controls preserve empty and ordinary async expressions, body `await`,
nested ordinary functions, string text and a nested derived class constructor
containing its own valid `super()` call.

The parser's existing check order remains observable through typed diagnostics:

- parameter `Contains AwaitExpression` precedes the super check;
- duplicate non-simple parameters precede it;
- a Use Strict Directive with non-simple parameters precedes it; and
- a formal parameter/body lexical declaration conflict precedes it.

Adjacent async declarations, generator/async-generator expressions and
declarations, and method producers remain `ParseCode::Malformed`.

## Durable source guard

The shared super-producer guard recursively requires:

- exactly one async-function-expression-specific message and no generic
  message in that producer file;
- the exact completed-node `contains(&function, ContainsSymbol::Super)` branch,
  new message and retained `params_start_position` together;
- exactly one generator-expression-specific message on its completed-node
  `Contains Super` branch;
- exactly three remaining generic raw messages, including the declaration
  default and sole remaining expression producer;
- the existing ordinary function, class and method message censuses;
- the declaration default/override boundary remains unchanged;
- ordinary/async-arrow traversal and ordinary callable/nested-class stopping
  behavior in pinned `boa_ast`; and
- the sole parse/classifier product boundary.

Moving the unique message before complete node construction or onto an adjacent
producer fails the bounded source shape even if literal counts remain equal.

## Complete pinned Test262 cohort

The complete dedicated cohort at revision
`e9d582d6b8b13afc5ba9a676664741592b5c7f69` is exactly four files:

- `language/expressions/async-function/early-errors-expression-body-contains-super-call.js`;
- `language/expressions/async-function/early-errors-expression-body-contains-super-property.js`;
- `language/expressions/async-function/early-errors-expression-formals-contains-super-call.js`; and
- `language/expressions/async-function/early-errors-expression-formals-contains-super-property.js`.

All four are parse-negative `SyntaxError` tests and declare no execution-mode
flag. They expand to exactly eight sloppy/strict Wasm-AOT variants. Async
declaration, async-generator, method and dynamic-eval files belong to other
producers and are excluded.

## Verification

The coordinated batch verifier ran:

```sh
cargo test -p lila-front --lib -- --test-threads=1
cargo test -p lila-ir modules::early -- --test-threads=1
cargo test -p lila-ir modules::graph -- --test-threads=1
```

`cargo fmt --all -- --check`, `cargo xc`, `git diff --check` and the task-plan
check are green. The complete front library passes `138/138`; the relevant IR
module-early and graph groups pass `49/49` and `49/49`. The four exact Test262
paths were run separately through Wasm-AOT with `--jobs 1 --threads 1`; all
eight sloppy/strict variants pass with every failure and non-success bucket at
zero.

The subsequent GeneratorExpression producer-census update passes the complete
`142/142` front gate and relevant `50/50` IR early and `51/51` graph groups.

## Nonclaims

This lane does not classify async declarations, generator declarations,
async-generator expressions/declarations, method-owned super restrictions, change
`Contains`, add syntax, support dynamic source, alter async execution, prove a
new Test262 pass, refresh aggregate status, close callable grammar, or complete
T07.
