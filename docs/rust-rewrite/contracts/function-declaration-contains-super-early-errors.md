# FunctionDeclaration `Contains super` early errors

**Status:** Product condition focused-verified 2026-08-24; current shared
producer census through GeneratorDeclaration focused-verified 2026-09-01

## Decision

The four ordinary `FunctionDeclaration` restrictions are one closed parser
condition:

`EarlyErrorCode::FunctionDeclarationContainsSuper`

Its wire spelling is `E_FUNCTION_DECLARATION_CONTAINS_SUPER`. It reports phase
`Early`, native error type `SyntaxError`, and a nonempty source span under both
Script and Module goals.

The condition is true when the declaration's `FormalParameters` or
`FunctionBody` `Contains SuperProperty` or `Contains SuperCall`. Ordinary and
async arrows remain lexical traversal paths. Nested ordinary callable bodies
and nested classes retain their own static-semantics boundaries.

Generator, async-function and async-generator declarations are distinct
productions. Async-function and generator declarations now have their own typed
conditions; AsyncGeneratorDeclaration now has its own distinct typed condition.
None acquires this code.

## Specification boundary

ECMA-262 2026
[15.2.1, Function Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-function-definitions-static-semantics-early-errors)
rejects a `FunctionDeclaration` for each of these conditions:

- `FormalParameters Contains SuperProperty`;
- `FunctionBody Contains SuperProperty`;
- `FormalParameters Contains SuperCall`; and
- `FunctionBody Contains SuperCall`.

The code combines only those four bullets for one grammar production. It does
not merge a generator, async, expression, method, class-element or whole-source
condition.

## Pinned-Boa producer and diagnostic repair

Pinned Boa's
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/mod.rs`
uses `parse_callable_declaration` for ordinary, generator, async-function and
async-generator declarations. After parsing the binding name, parameters and
body and applying the earlier callable checks, the shared producer runs:

```text
if contains(&body, ContainsSymbol::Super)
    || contains(&params, ContainsSymbol::Super)
{
    return Err(...);
}
```

One private `CallableDeclaration::contains_super_error_message` method keeps
that predicate and its order shared. Its default remains `invalid super usage`.
The ordinary implementation in `hoistable/function_decl/mod.rs` overrides it
with:

```text
function declaration cannot contain super
```

The common branch retains `params_start_position`. Boa appends that coordinate,
so the classifier owns the complete anchored prefix:

```text
function declaration cannot contain super at line
```

No grammar, `Contains` predicate, early-error order, accepted source or source
location changes. The seam selects a diagnostic by production; it does not
duplicate the semantic check.

The AsyncFunctionDeclaration implementation now independently overrides the
same hook with its own production-specific text. The ordinary message still
occurs exactly once across pinned Boa Rust sources. On current head, `invalid
super usage` occurs once at the fixed ScriptBody producer; the declaration hook
is required and has no default. GeneratorDeclaration and
AsyncGeneratorDeclaration have their own production-specific overrides. Each expression and typed declaration message
occurs once, and the method-owned `invalid super call usage` census remains
eleven.

## Typed and retained boundaries

The closed front domain and parse table grow from 66 to 67 entries. One
`StartsWith` row maps the complete fixed prefix to
`FunctionDeclarationContainsSuper`. An evaluated parse-owner witness, exact
single-owner assertion, disjoint table witnesses, exhaustive wire-name checks
and `lila-ir`'s no-catch-all rejection-kind match make the addition structural.

Classifier checks keep the new prefix distinct from the separately typed
FunctionExpression prefix, adjacent typed declarations/expressions and
method-owned text. A Module duplicate-export diagnostic containing the complete
new prefix remains `ModuleDuplicateExport`, so user-controlled text cannot
forge the code.

A real exported declaration failure crosses
`module_parse_failure_diagnostic`. A real rejected dependency remains a
`ModuleSourceIr` whose retained parse is `ModuleParse::Rejected`, exposes no
module requests, and crosses `build_graph` with the same code, phase, error
type and span. A valid exported ordinary function remains a parsed graph node.

## Permanent behavior and precedence matrix

Every direct rejection source runs under Script and Module goals. The matrix
covers parameter and body positions, `SuperProperty` and `SuperCall`, and
ordinary, async and nested arrow traversal.

One Module-only witness covers the anonymous default-export declaration form;
Script has no corresponding grammar production.

Positive controls preserve empty and ordinary declarations, nested ordinary
functions, string text, and a nested derived class constructor containing its
own valid `super()` call.

The parser's existing check order remains observable through typed diagnostics:

- duplicate non-simple parameters precede the super check;
- a Use Strict Directive with non-simple parameters precedes it; and
- a formal parameter/body lexical declaration conflict precedes it.

GeneratorDeclaration, AsyncFunctionDeclaration and the adjacent expression
producers have independently typed conditions. AsyncGeneratorDeclaration has
its own typed code; adjacent method owners remain `ParseCode::Malformed`.

## Durable source guard

The shared super-producer guard recursively requires:

- exactly one function-declaration-specific message across pinned Boa Rust
  sources and no generic message in `function_decl/mod.rs`;
- one required diagnostic hook on `CallableDeclaration`, plus one override for
  each of ordinary, async-function, generator and async-generator declarations;
- the exact shared body-or-parameters `Contains Super` predicate, production-
  selected message and retained `params_start_position` in one branch;
- the branch remains after parameter/body lexical-name validation and before
  the generator parameter checks;
- each declaration parser still calls the common parser once;
- exactly one generic raw message plus the existing declaration, expression,
  class and method message censuses;
- ordinary/async-arrow traversal and ordinary callable/nested-class stopping
  behavior in pinned `boa_ast`; and
- the sole parse/classifier product boundary.

Literal counts alone are insufficient: adding a shared default, removing an
override, or detaching message selection
from the common predicate fails the bounded source shape.

## Complete pinned Test262 cohort

The complete dedicated cohort at revision
`e9d582d6b8b13afc5ba9a676664741592b5c7f69` is exactly four files:

- `language/statements/function/early-body-super-call.js`;
- `language/statements/function/early-body-super-prop.js`;
- `language/statements/function/early-params-super-call.js`; and
- `language/statements/function/early-params-super-prop.js`.

All four are parse-negative `SyntaxError` tests and declare no execution-mode
flag. They expand to exactly eight sloppy/strict Wasm-AOT variants. Expression,
generator, async-function, method, class-field and eval files belong to other
producer boundaries and are excluded.

## Verification ladder

The coordinated batch ran the focused filters below before the broad groups:

```sh
cargo test -p lila-front function_declaration_super -- --test-threads=1
cargo test -p lila-front tests::known_script_and_class_super_producers_stay_structurally_reviewed -- --exact --test-threads=1
cargo test -p lila-front tests::pinned_contains_super_traversal_stays_structurally_reviewed -- --exact --test-threads=1
cargo test -p lila-ir modules::early::tests::function_declaration_super_module_parse_maps_to_an_early_syntax_error -- --exact --test-threads=1
cargo test -p lila-ir modules::graph::tests::rejected_function_declaration_super_dependency_keeps_its_code_through_graph_build -- --exact --test-threads=1
cargo test -p lila-ir modules::graph::tests::retained_function_declaration_without_super_builds_a_real_module_graph -- --exact --test-threads=1
```

Formatting, `cargo xc` and diff hygiene are green. The complete front library
passes `134/134`; the relevant IR early and graph groups pass `48/48` and
`47/47`. The four exact Test262 paths were run separately with `--jobs 1
--threads 1`; each passes `2/2`, for `8/8` completed Wasm-AOT variants with
every non-success bucket at zero.

The subsequent AsyncFunctionExpression lane updates the shared producer guard;
the complete `138/138` front gate and relevant `49/49` IR early and `49/49`
graph groups are green with that census.

The subsequent GeneratorExpression producer-census update passes the complete
`142/142` front gate and relevant `50/50` IR early and `51/51` graph groups.

The subsequent AsyncGeneratorExpression and AsyncFunctionDeclaration lanes
update the current shared producer census. Their focused checkpoint passes the
shared producer guard `1/1` and parse-pattern structure target `4/4`; this does
not refresh the historical broad-group counts above.

## Nonclaims

This lane does not classify generator or async-generator declarations, change
`Contains`, add syntax, support eval or Function-constructor dynamic source,
alter function lowering or execution, claim that typed classification caused a
new Test262 pass, refresh aggregate status, close callable grammar, or complete
T07.
