# AsyncGeneratorDeclaration `Contains super` early errors

**Status:** Implemented 2026-09-01; focused verification pending

## Decision

The four `AsyncGeneratorDeclaration` super restrictions are one closed parser
condition:

`EarlyErrorCode::AsyncGeneratorDeclarationContainsSuper`

Its wire spelling is
`E_ASYNC_GENERATOR_DECLARATION_CONTAINS_SUPER`. The exhaustive `lila-ir`
rejection projection derives phase `Early` and native error type `SyntaxError`.

The condition is true when the declaration's `FormalParameters` or
`AsyncGeneratorBody` `Contains SuperProperty` or `Contains SuperCall`.
Ordinary and async arrows are lexical traversal paths. Nested ordinary
callables and classes retain their own static-semantics boundaries.

## Specification boundary

ECMA-262 2026
[Async Generator Function Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-async-generator-function-definitions-static-semantics-early-errors)
rejects an `AsyncGeneratorDeclaration` for each of these conditions:

- `FormalParameters Contains SuperProperty`;
- `AsyncGeneratorBody Contains SuperProperty`;
- `FormalParameters Contains SuperCall`; and
- `AsyncGeneratorBody Contains SuperCall`.

This code combines only those four bullets for this grammar production.

## Pinned-Boa producer

Pinned Boa's
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/mod.rs`
parses all four callable declaration forms through
`parse_callable_declaration`. Its one body-or-parameters `Contains Super`
predicate remains shared. `CallableDeclaration::contains_super_error_message`
is now required, so every declaration implementation must select its own
diagnostic before it can compile. The AsyncGeneratorDeclaration implementation
in `hoistable/async_generator_decl/mod.rs` selects:

```text
async generator declaration cannot contain super
```

The common branch retains `params_start_position`; Boa appends that coordinate,
so the classifier owns the complete prefix:

```text
async generator declaration cannot contain super at line
```

The message occurs once across pinned Boa Rust sources. The generic declaration
default and its raw literal are deleted. The only remaining raw
`invalid super usage` literal is the fixed ScriptBody producer.

The shared declaration order remains duplicate-parameter and non-simple
Use Strict checks, strict-name checks, parameter/body lexical-name conflict,
super, parameter Yield, then parameter Await. An async generator declaration
whose parameters contain YieldExpression or AwaitExpression while its body
contains super therefore receives
`AsyncGeneratorDeclarationContainsSuper`.

## Typed and retained boundaries

The closed front domain and parse table have 73 entries. One `StartsWith` row
maps the complete prefix to
`AsyncGeneratorDeclarationContainsSuper`; the table population is exactly
`54/18/1` `ContainsAll` / `StartsWith` / `Exact`. An evaluated witness, exact
single-owner assertion, eight-observation prefix-injection proof and exhaustive
wire-name round trip close the front mapping. The exhaustive `EarlyErrorCode`
match in `lila-ir` owns the variant explicitly.

Direct front tests cover named declarations under Script and Module goals,
parameters and body, both super forms, ordinary and async-arrow traversal, the
anonymous default-export form, positive boundaries, nested production
ownership, shared-check precedence through both Yield and Await, and
diagnostic-text injection.

A real failed Module parse crosses `module_parse_failure_diagnostic`. A
rejected dependency remains a `ModuleSourceIr` containing
`ModuleParse::Rejected`, exposes no module requests, and crosses `build_graph`
with the same code, kind, phase, error type and nonempty span. A valid exported
async generator declaration remains a parsed graph node.

## Durable source guard

The shared super-producer guard recursively requires:

- one exact async-generator-declaration message, absent from the shared parser;
- one required message method on `CallableDeclaration` and one override in
  each of its four implementations;
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
`test/language/statements/async-generator/` tree contains 301 JavaScript files.
An exact tree grep finds no `SuperProperty`, `SuperCall` or static `super`
source in that directory.

The dedicated cohort is therefore zero files and zero execution variants.
This lane makes no Test262 pass-count, pass-gain or aggregate-status claim.
The direct Script, Module, retained-module and graph witnesses are the semantic
evidence for the condition.

## Verification status and nonclaims

The source, tests, count guards and documentation are internally complete.
Cargo, rustfmt and focused test execution remain pending while the coordinated
workspace verification session owns the build artifacts. No green compile or
test result is claimed by this implementation pass.

This lane does not change `Contains` traversal, accepted syntax, source
positions, dynamic-source support, lowering or Wasm code generation, and does
not close T07 as a whole.
