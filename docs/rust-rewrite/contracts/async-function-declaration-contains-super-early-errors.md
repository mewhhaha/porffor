# AsyncFunctionDeclaration `Contains super` early errors

**Status:** Focused-verified 2026-09-01

## Decision

The four `AsyncFunctionDeclaration` super restrictions are one closed parser
condition:

`EarlyErrorCode::AsyncFunctionDeclarationContainsSuper`

Its wire spelling is `E_ASYNC_FUNCTION_DECLARATION_CONTAINS_SUPER`. The
exhaustive `lila-ir` rejection projection derives phase `Early` and native
error type `SyntaxError`.

The condition is true when the declaration's `FormalParameters` or
`AsyncFunctionBody` `Contains SuperProperty` or `Contains SuperCall`. Ordinary
and async arrows are lexical traversal paths. Nested ordinary functions and
classes retain their own static-semantics boundaries.

Async-function expressions, generator declarations, async-generator
declarations and method definitions remain different production owners.

## Specification boundary

ECMA-262 2026
[Async Function Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-async-function-definitions-static-semantics-early-errors)
rejects an `AsyncFunctionDeclaration` for each of these conditions:

- `FormalParameters Contains SuperProperty`;
- `AsyncFunctionBody Contains SuperProperty`;
- `FormalParameters Contains SuperCall`; and
- `AsyncFunctionBody Contains SuperCall`.

This code combines only those four bullets for this grammar production.

## Pinned-Boa producer

Pinned Boa's
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/mod.rs`
parses ordinary, generator, async-function and async-generator declarations
through `parse_callable_declaration`. After the duplicate-parameter, non-simple
Use Strict and parameter/body lexical-name checks, its sole shared super
predicate runs:

```text
if contains(&body, ContainsSymbol::Super)
    || contains(&params, ContainsSymbol::Super)
{
    return Err(...);
}
```

The private `CallableDeclaration::contains_super_error_message` hook keeps the
predicate, position and order shared. Its default stays `invalid super usage`.
Only `hoistable/async_function_decl/mod.rs` selects:

```text
async function declaration cannot contain super
```

The common branch retains `params_start_position`; Boa appends that coordinate,
so the classifier owns the complete prefix:

```text
async function declaration cannot contain super at line
```

The shared super check remains before the declaration parameter-Yield and
parameter-Await checks. Therefore a source whose parameters contain both
`await` and `super` retains this production's super code. No grammar,
`Contains` traversal, accepted source or location behavior changes.

The specific message occurs once across pinned Boa Rust sources. On current
head, the generic literal remains only at ScriptBody. The declaration message
hook is required, with separate production-owned GeneratorDeclaration and
AsyncGeneratorDeclaration overrides.

## Typed and retained boundaries

The closed front domain and parse table grow from 70 to 71 entries. One
`StartsWith` row maps the complete prefix to
`AsyncFunctionDeclarationContainsSuper`. The subsequent GeneratorDeclaration
and AsyncGeneratorDeclaration lanes bring the current table to 73 entries with
the exact `54/18/1`
`ContainsAll` / `StartsWith` / `Exact` population. An evaluated table witness,
an exact single-owner assertion, disjointness proofs and exhaustive wire-name
round trips close the front mapping. The exhaustive `EarlyErrorCode` match in
`lila-ir` must include the new variant before the workspace builds.

Classifier proofs keep this prefix distinct from ordinary declarations,
async-function expressions, typed declarations and method-owned diagnostics.
A duplicate-export message containing the complete prefix remains
`ModuleDuplicateExport`, so user-controlled text cannot forge the code.

A real exported declaration failure crosses `module_parse_failure_diagnostic`.
A rejected dependency remains a `ModuleSourceIr` whose retained parse is
`ModuleParse::Rejected`, exposes no module requests, and crosses `build_graph`
with the same code, kind, phase, error type and nonempty span. A valid exported
async-function declaration remains a parsed graph node.

## Permanent behavior and precedence matrix

Every named direct source runs under Script and Module goals. The matrix covers
parameter and body positions, `SuperProperty`, `SuperCall`, and ordinary and
async-arrow traversal. A Module-only witness covers the anonymous default-
export form.

Positive controls preserve empty declarations, ordinary `await`, nested
ordinary functions, string text and a nested derived-class constructor that
owns its valid `super()` call.

Typed precedence witnesses preserve the shared declaration order:

- duplicate non-simple parameters precede the super check;
- a Use Strict Directive with non-simple parameters precedes it;
- a formal parameter/body lexical declaration conflict precedes it; and
- the super check precedes the async parameter `Contains AwaitExpression`
  check.

## Durable source guard

The shared super-producer guard recursively requires:

- one exact async-function-declaration message across pinned Boa Rust sources,
  absent from the shared parser and other declaration implementations;
- one required shared message hook and one production-owned override in each
  of the four declaration implementations;
- the exact shared body-or-parameters predicate, production-selected message
  and retained parameter-start position in one branch;
- parameter/body lexical-name validation before super, then the parameter-
  Yield and parameter-Await checks;
- exactly one call from each declaration parser to the common parser;
- exactly one generic raw message plus the expression, class and method
  producer censuses;
- ordinary/async-arrow traversal and ordinary callable/nested-class stopping
  behavior in pinned `boa_ast`; and
- the sole parse/classifier product boundary.

The parse-pattern structure guard pins 73 rows, the `54/18/1` population, 91
recursive lexical mentions and all six exhaustive observers.

## Exact pinned Test262 cohort

At suite pin `aa55200d1310384c5cf69ea95b2a2ecba457007b`, the complete dedicated
cohort is exactly four unflagged parse-negative `SyntaxError` files:

- `language/statements/async-function/early-errors-declaration-body-contains-super-call.js`;
- `language/statements/async-function/early-errors-declaration-body-contains-super-property.js`;
- `language/statements/async-function/early-errors-declaration-formals-contains-super-call.js`; and
- `language/statements/async-function/early-errors-declaration-formals-contains-super-property.js`.

They expand to eight sloppy/strict Wasm-AOT executions. Expression, generator,
method and dynamic-eval files belong to other producer boundaries.

## Verification

The focused checkpoint ran:

```sh
cargo test -p lila-front --lib async_function_declaration_super -- --test-threads=1
cargo test -p lila-front --lib tests::known_script_and_class_super_producers_stay_structurally_reviewed -- --exact --test-threads=1
cargo test -p lila-front --test parse_failure_pattern_structure -- --test-threads=1
cargo test -p lila-ir modules::early::tests::async_function_declaration_super_module_parse_maps_to_an_early_syntax_error -- --exact --test-threads=1
cargo test -p lila-ir modules::graph::tests::rejected_async_function_declaration_super_dependency_keeps_its_code_through_graph_build -- --exact --test-threads=1
cargo test -p lila-ir modules::graph::tests::retained_async_function_declaration_without_super_builds_a_real_module_graph -- --exact --test-threads=1
```

The direct front filter passes `6/6`, the shared producer guard passes `1/1`,
and the parse-pattern structure target passes `4/4`. The exact IR early witness
and both graph witnesses are green. The four exact Test262 paths pass all `8/8`
applicable sloppy/strict Wasm-AOT executions. This is focused evidence only; no
broad aggregate or published-status claim follows.

## Nonclaims

This lane does not classify generator or async-generator declarations, change
`Contains`, add syntax, support dynamic source, alter async execution, refresh
aggregate status, close callable grammar or complete T07.
