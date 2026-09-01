# GeneratorDeclaration `Contains super` early errors

**Status:** Focused verified 2026-09-01

## Decision

The four `GeneratorDeclaration` super restrictions are one closed parser
condition:

`EarlyErrorCode::GeneratorDeclarationContainsSuper`

Its wire spelling is `E_GENERATOR_DECLARATION_CONTAINS_SUPER`. The exhaustive
`lila-ir` rejection projection derives phase `Early` and native error type
`SyntaxError`.

The condition is true when the declaration's `FormalParameters` or
`GeneratorBody` `Contains SuperProperty` or `Contains SuperCall`. Ordinary and
async arrows are lexical traversal paths. Nested ordinary callables and method
bodies retain their own static-semantics boundaries. Generator expressions and
async-generator declarations remain different production owners.

## Specification boundary

ECMA-262 2026
[Generator Function Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-generator-function-definitions-static-semantics-early-errors)
rejects a `GeneratorDeclaration` for each of these conditions:

- `FormalParameters Contains SuperProperty`;
- `GeneratorBody Contains SuperProperty`;
- `FormalParameters Contains SuperCall`; and
- `GeneratorBody Contains SuperCall`.

This code combines only those four bullets for this grammar production.

## Pinned-Boa producer

Pinned Boa's
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/mod.rs`
parses all four callable declaration forms through
`parse_callable_declaration`. Its one body-or-parameters `Contains Super`
predicate remains shared. The private
`CallableDeclaration::contains_super_error_message` hook lets
`hoistable/generator_decl/mod.rs` select:

```text
generator declaration cannot contain super
```

The common branch retains `params_start_position`; Boa appends that coordinate,
so the classifier owns the complete prefix:

```text
generator declaration cannot contain super at line
```

The message occurs once across pinned Boa Rust sources. On current head, the raw
`invalid super usage` literal occurs once at the fixed ScriptBody producer. The
shared declaration message hook is required, and AsyncGeneratorDeclaration has
its own production-owned override.

The shared declaration order remains duplicate-parameter and non-simple
Use Strict checks, strict-name checks, parameter/body lexical-name conflict,
super, parameter Yield, then parameter Await. A generator declaration whose
parameters contain both YieldExpression and super therefore receives
`GeneratorDeclarationContainsSuper`. This intentionally differs from the
separate GeneratorExpression parser, whose parameter-Yield check precedes its
completed-node super check. Every outcome is still a SyntaxError; this pins
which typed producer owns a multiply-invalid source.

## Typed and retained boundaries

The closed front domain and parse table now have 73 entries. One `StartsWith` row
maps the complete prefix to `GeneratorDeclarationContainsSuper`; the table
population is exactly `54/18/1` `ContainsAll` / `StartsWith` / `Exact`. An
evaluated witness, exact single-owner assertion, prefix-injection proof and
exhaustive wire-name round trip close the front mapping. The exhaustive
`EarlyErrorCode` match in `lila-ir` owns the variant explicitly.

Direct front tests cover named declarations under Script and Module goals,
parameters and body, both super forms, ordinary and async-arrow traversal, the
anonymous default-export form, positive boundaries, nested production
ownership, shared-check precedence and diagnostic-text injection.

A real failed Module parse crosses `module_parse_failure_diagnostic`. A
rejected dependency remains a `ModuleSourceIr` containing
`ModuleParse::Rejected`, exposes no module requests, and crosses `build_graph`
with the same code, kind, phase, error type and nonempty span. A valid exported
generator declaration remains a parsed graph node.

## Durable source guard

The shared super-producer guard recursively requires:

- one exact generator-declaration message, absent from the shared parser and
  other declaration implementations;
- one required shared message hook and one override for each of the four
  declaration implementations;
- the exact shared body-or-parameters predicate and parameter-start position;
- parameter/body lexical-name validation before super, then the parameter-
  Yield and parameter-Await checks;
- exactly one call from each declaration parser to the common parser;
- exactly one raw `invalid super usage` literal and the existing expression,
  class and method producer censuses; and
- ordinary/async-arrow traversal plus nested callable and class boundaries in
  pinned `boa_ast`.

The parse-pattern structure guard pins 73 rows, the `54/18/1` population, 91
recursive lexical mentions and all six exhaustive consumers. The retained
graph source guards pin 60 graph tests, 33 `build_graph(` calls, 35
`ModuleGraphSources` mentions, 67 `ModuleSourceIr` mentions and 67 `ModuleKey`
mentions.

## Exact pinned Test262 inventory

The repository Test262 revision
`e9d582d6b8b13afc5ba9a676664741592b5c7f69` resolves to vendored content tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`. The complete
`test/language/statements/generators/` tree contains 266 JavaScript files. An
exact tree grep finds no `SuperProperty`, `SuperCall` or static `super` source
in that directory. A wider language-tree cross-check likewise finds no
ordinary GeneratorDeclaration containing super.

The dedicated cohort is therefore zero files and zero execution variants.
This lane makes no Test262 pass-count, pass-gain or aggregate-status claim.
The direct Script, Module, retained-module and graph witnesses are the semantic
evidence for the condition.

## Verification status and nonclaims

At 2026-09-01, the full front library passes `158/158` and the parse-pattern
structure target passes `4/4`. The focused IR filter passes `7/7`, including
the three new Module and graph witnesses, with 1,101 unrelated tests filtered;
the four exact graph/source structure targets pass `13/13`. The product-path
`cargo check -p lila-aot-wasm` also compiles the live parser, front and IR
layers. The implementation-time standalone parse/graph checks remain green at
`17/17`. No broad IR or Test262 suite was rerun, and no broader green result is
claimed here.

This lane did not classify AsyncGeneratorDeclaration; that production is now
owned by its subsequent dedicated lane. It does not change `Contains`
traversal, alter accepted syntax or source positions, support dynamic source,
change lowering or Wasm code generation, or close T07 as a whole.
