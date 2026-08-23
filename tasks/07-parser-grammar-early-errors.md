# T07 — Parser boundary, grammar coverage and early errors

**Status:** In progress — parse-once boundary plus ObjectLiteral CoverInitializedName, Script top-level `new.target` and `using`, for-in and switch-clause `using` declarations, generator declaration/expression and async-generator-expression parameter `Contains YieldExpression`, duplicate formal/catch-parameter, catch-body conflict, duplicate-class-constructor/private-name, constructor method/private-name, public-static-method `prototype` and class-field literal-name restrictions, class static-block `ContainsAwait`, class static-block/field `ContainsArguments` and strict-mode `with` classification implemented; grammar and early-error closure remain

**Parallel group:** Core foundations  
**Depends on:** T01, T02  
**Blocks:** T08, T09, T12, T24 and parser-failure closure

## Current repository state

The front end now returns a closed `ParsedSource` with goal-typed
`ParsedScript` and `ParsedModule` variants. Each variant owns Boa's AST and the
exact interner that produced it, while `lila-ir` can only borrow the pair
through a controlled compiler session. Raw `SourceUnit` metadata is a distinct
type that the lowerer does not accept.

Loaded modules retain either that parsed product or the structured parse
rejection in `ModuleSourceIr`; dependency discovery and module-record
construction therefore share one parse attempt. The engine similarly retains a
`PreparedCompilation` so cache-key graph hashing and lowering consume the same
graph. Linked module text is a new, generated Script compilation unit and is
parsed once before ordinary Script lowering. The old reparsing functions and
the public reparsing stage have been removed.

A Script that contains `import()` keeps its Script-goal parse: request discovery
walks that retained AST and synthesizes only the graph record the linker needs.
It is not reparsed under Module grammar, so sloppy Script syntax and top-level
semantics cannot drift merely because the Script performs a dynamic import.

This closes the architectural double-parse defect, not T07 as a whole.
Current-pin parser and early-error buckets still lack a complete verified
Wasm-AOT aggregate, and the remaining grammar/diagnostic cases below still need
inventory-driven closure.

ObjectLiteral `CoverInitializedName` now has one closed condition across Boa's
Script, function-body, Module-item and class-static-block producers. The typed
code and retained-module projection report `Early`/`SyntaxError`, while
assignment patterns, binding patterns, arrow parameters, shorthand and ordinary
data properties remain parse-valid. The boundary is recorded in
`docs/rust-rewrite/contracts/object-literal-cover-initialized-name-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `53/53`, the focused IR
early-error gate passes `3/3`, and the exact pinned Test262 witness passes `2/2`
sloppy/strict Wasm-AOT executions with zero failure or non-success outcomes.
This is bounded diagnostic classification, not general ObjectLiteral grammar,
destructuring execution, T07 or aggregate closure.

ScriptBody `Contains NewTarget` now has one closed condition for Boa's sole
fixed-position producer. Direct and top-level-arrow-carried `new.target` reject
with a typed `Early`/`SyntaxError` diagnostic, while ordinary functions, their
nested arrows, constructors, methods and class static blocks remain valid. The
code is deliberately Script-only: retained dependencies preserve the existing
distinct `ModuleTopLevelNewTarget` diagnostic and valid exported function
boundaries. The contract is recorded in
`docs/rust-rewrite/contracts/script-top-level-new-target-early-errors.md`. At
`2026-08-23`, the capped serial front gate passes `55/55`, the focused IR
early-error gate passes `3/3`, and the exact two-file pinned cohort passes `4/4`
sloppy/strict Wasm-AOT executions with zero failure or non-success outcomes.
This is bounded classification, not direct-eval, all-`new.target`, T07 or
aggregate closure.

ScriptBody top-level `using` now has one closed condition for Boa's sole
fixed-position post-parse producer. Nested blocks, functions, loop heads and
class static blocks remain valid Script boundaries, while retained Module
sources keep both top-level `using` and `await using` valid. Pinned Boa rejects
top-level Script `await using` earlier as an ordinary parse error, so that
Test262 case remains an honest untyped parse-phase `SyntaxError` rather than a
fabricated classifier match. The boundary is recorded in
`docs/rust-rewrite/contracts/script-top-level-using-declaration-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `57/57`, the focused IR
early-error gate passes `3/3`, and the exact two-file pinned cohort passes `4/4`
sloppy/strict Wasm-AOT executions with zero failure or non-success outcomes.
This is bounded classification, not parser-reachability repair, disposal
execution, direct eval, T07 or aggregate closure.

`for-in` heads whose lexical declaration is `using` or `await using` now share
one closed condition for Boa's sole fixed-message producer. Script, Module and
retained dependency parsing carry the same typed `Early`/`SyntaxError`
diagnostic. Positive boundaries preserve `for-of` for both declaration forms,
ordinary `let`/`const` `for-in`, and initialized `using` in classic `for`. The
contract is recorded in
`docs/rust-rewrite/contracts/for-in-using-declaration-early-errors.md`. At
`2026-08-23`, the capped serial front gate passes `59/59`, the focused IR
early-error gate passes `3/3`, and the exact two-file pinned cohort passes `4/4`
sloppy/strict Wasm-AOT executions with zero failure or non-success outcomes.
This is bounded classification, not disposal execution, direct eval, all
iterable-loop grammar, T07 or aggregate closure.

Switch CaseClause and DefaultClause StatementLists that directly contain a
`using` or `await using` declaration now share one closed condition for Boa's
sole fixed-message producer and exactly two disabling callers. Script, Module
and retained dependency parsing carry the same typed `Early`/`SyntaxError`
diagnostic. Nested blocks, classic `for`, `for-of`, nested functions and direct
`let`/`const` clause declarations remain valid. The contract is recorded in
`docs/rust-rewrite/contracts/switch-clause-using-declaration-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `61/61`, the focused IR
early-error gate passes `3/3`, and the exact four-file pinned cohort passes
`8/8` sloppy/strict Wasm-AOT executions with zero failure or non-success
outcomes. This is bounded classification, not disposal execution, direct eval,
all switch grammar, T07 or aggregate closure.

Ordinary and async generator declarations whose own FormalParameters contain a
`YieldExpression` now share one closed condition for Boa's sole fixed-message
declaration producer. Script, Module and retained dependency parsing carry the
same typed `Early`/`SyntaxError` diagnostic. Generator bodies and nested
generator initializers remain valid containment boundaries, while generator
expressions and methods keep their distinct pinned producers. The contract is
recorded in
`docs/rust-rewrite/contracts/generator-declaration-parameters-contain-yield-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `63/63`, the focused IR
early-error gate passes `3/3`, and the exact one-file pinned cohort passes `2/2`
sloppy/strict Wasm-AOT executions with zero failure or non-success outcomes.
This is bounded classification, not all generator grammar, direct eval, T07 or
aggregate closure.

Ordinary generator expressions whose own FormalParameters contain a
`YieldExpression` now have one closed condition for Boa's sole fixed-message
primary-expression producer. Anonymous and named forms reject under Script and
Module goals, and retained dependency parsing carries the same typed
`Early`/`SyntaxError` diagnostic. Generator bodies and nested generator
initializers remain valid containment boundaries. Generator declarations,
async-generator expressions and methods retain their distinct pinned
producers. The contract is recorded in
`docs/rust-rewrite/contracts/generator-expression-parameters-contain-yield-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `65/65`, the focused IR
early-error gate passes `3/3`, and the exact one-file pinned cohort passes `2/2`
sloppy/strict Wasm-AOT executions with zero failure or non-success outcomes.
This is bounded classification, not all generator grammar, direct eval, T07 or
aggregate closure.

Async-generator expressions whose own FormalParameters contain a
`YieldExpression` now have one closed condition for Boa's sole fixed-message
yield producer. Anonymous and named forms reject under Script and Module goals,
and retained dependency parsing carries the same typed `Early`/`SyntaxError`
diagnostic. Async-generator bodies and nested generator initializers remain
valid containment boundaries. The adjacent parameter `Contains
AwaitExpression` condition and other generator forms retain their distinct
pinned producers. The contract is recorded in
`docs/rust-rewrite/contracts/async-generator-expression-parameters-contain-yield-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `67/67`, the focused IR
early-error gate passes `3/3`, and the exact one-file pinned cohort passes `2/2`
sloppy/strict Wasm-AOT executions with zero failure or non-success outcomes.
This is bounded classification, not all async-generator grammar, direct eval,
T07 or aggregate closure.

Duplicate formal parameters now have one closed diagnostic condition across
entry and retained dependency parsing. The classifier follows pinned Boa's two
exact, case-sensitive wordings and preserves the spec exception for sloppy
ordinary functions with simple parameter lists. This closes that bounded
misclassification only; it does not claim the remaining formal-parameter early
errors or the current-pin parser bucket are complete. The focused Cargo and
Test262 verification is deferred to the shared verification lane.

Duplicate catch-parameter `BoundNames` now form a separate closed condition,
selected by pinned Boa's sole exact wording across both parse goals and retained
dependency failures. It remains distinct from the catch-body conflict
classifier and does not change catch binding initialization or lowering.

Catch-parameter/body declaration conflicts now have one closed condition for
Boa's one exact wording across both reachable branches: overlap with catch-body
lexical declarations, and overlap between a binding-pattern parameter and
catch-body `var` declarations. Script and Module tests preserve the specified
simple-`BindingIdentifier` `var` exception. This classification does not change
runtime catch environments, destructuring evaluation, or lowering.

Duplicate ordinary class constructors now have one closed condition for Boa's
sole exact wording. Class declarations and expressions reject under both Script
and Module goals, while positive witnesses preserve `static constructor()` and
computed `["constructor"]()` methods beside one ordinary constructor. This is
classification only: it does not change class lowering or runtime constructor
semantics, close adjacent constructor restrictions, or complete the class
grammar bucket.

Non-static generator and async-generator methods named `constructor` now share
one closed condition for Boa's exact common wording. Declaration and expression
forms reject under both goals, while positive witnesses preserve static and
computed generator methods named `constructor` beside one ordinary
constructor. This is classification only: it does not implement generator or
class execution, combine adjacent constructor restrictions, or complete the
class grammar bucket. Focused Cargo and Test262 verification remains deferred
to the shared verification lane.

The remaining constructor-shaped ClassElement restrictions now have four
closed conditions. Non-static async methods, getters and setters named
`constructor` each follow their sole exact Boa wording, while the seven parser
branches that forbid the private name `#constructor` share one code. Script and
Module tests cover declaration and expression forms; positive boundaries retain
static and computed public methods/accessors plus a computed public
`"#constructor"` field. This is classification only: it does not implement
async execution, accessor/private-element installation, class lowering or the
remaining class grammar. The focused front and retained-module tests pass. The
complete adjacent expression and statement early-error subtrees each report
`444/444` under Wasm-AOT at the harness-declared
`aa55200d1310384c5cf69ea95b2a2ecba457007b` pin; this remains subtree evidence,
not T07 or aggregate closure.

Class static blocks whose statement lists have `ContainsArguments` now have one
closed condition for Boa's sole exact wording. Declaration and expression forms
reject under both Script and Module goals, including the pinned escaped
computed-name source and lexical use through an arrow. Positive witnesses keep
ordinary function and method parameters/bodies as traversal boundaries. This is
classification only: it does not implement static-block lowering or execution,
class-field `ContainsArguments`, or adjacent static-block early errors. Focused
Cargo and Test262 verification remains deferred to the shared verification
lane.

Class public/private, instance/static and auto-accessor field initializers whose
retained syntax has `ContainsArguments` now share one closed condition for
Boa's exact common wording. Script and Module witnesses preserve lexical
traversal through arrows and the stop at ordinary function and method bodies;
retained dependency failures use the same typed code. Direct-eval strings are
T13 dynamic-source debt rather than parser classifications. The boundary is
recorded in
`docs/rust-rewrite/contracts/class-field-contains-arguments-early-errors.md`.
At `2026-08-22`, the full front-end gate passes `40/40`, the focused IR early-
error gate passes `3/3`, and the exact 60-file pinned cohort passes `120/120`
Wasm-AOT executions with zero failure or non-success outcomes.

Class static blocks whose statement lists have `ContainsAwait` now have one
closed condition for Boa's sole exact producer. The classifier requires the
adjacent rendered fragment `invalid await usage at line`, so it cannot absorb
Boa's distinct longer generator-parameter message. Declaration and expression
forms reject under both Script and Module goals; positive witnesses preserve
`await` inside nested async ordinary and arrow function bodies. The boundary is
recorded in
`docs/rust-rewrite/contracts/class-static-block-contains-await-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `49/49`, the focused IR
early-error gate passes `3/3`, and the exact pinned Test262 witness passes `2/2`
Wasm-AOT executions with zero failure or non-success outcomes. This is typed
classification and retained-module repair, not static-block execution or broad
T07 closure.

Public static ordinary, generator, async, async-generator, getter and setter
methods whose literal property name is `prototype` now share one closed
condition for Boa's exact common wording across its six producer branches.
Script and Module declarations/expressions and retained dependency failures
carry the same typed `Early`/`SyntaxError` diagnostic. Positive witnesses keep
instance literal, public computed and private static names parse-valid; the
computed public run-time installation guard remains separate T09/T10 behavior.
The boundary is recorded in
`docs/rust-rewrite/contracts/class-static-method-prototype-name-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `51/51`, the focused IR
early-error gate passes `3/3`, and the exact twelve-file pinned cohort passes
`24/24` sloppy/strict Wasm-AOT executions with zero failure or non-success
outcomes. This is bounded diagnostic classification, not method execution,
class-grammar, T07 or aggregate closure.

Duplicate class private names now have one closed condition for Boa's exact
common wording across private fields, methods, accessors and static/instance
conflicts. Script and Module tests cover declarations and expressions, while
positive witnesses preserve the permitted getter/setter pair and independent
nested-class private-name domains. Retained dependency failures project the
same typed `Early`/`SyntaxError` diagnostic. The boundary is recorded in
`docs/rust-rewrite/contracts/class-duplicate-private-name-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `42/42`, the focused IR
early-error gate passes `3/3`, and the exact 32-file pinned cohort passes
`64/64` Wasm-AOT executions with zero failure or non-success outcomes. This is
bounded duplicate-private-name evidence, not class-grammar, T07 or aggregate
closure.

Public class-field literal-name restrictions now have two closed conditions.
Non-static fields and auto-accessors reject literal `constructor`; static forms
reject literal `constructor` or `prototype`. All eight Boa producer branches
map through the same typed Script, Module and retained-module boundary, while
computed names and ordinary constructor methods remain valid. The contract is
recorded in
`docs/rust-rewrite/contracts/class-field-literal-name-early-errors.md`. This is
classification plus a narrow vendored parser dispatch repair: non-static
identifier `constructor` enters constructor-method parsing only before `(`, so
field prefixes ending in `;` or followed by `=` reach the existing field
early-error producer. It does not change class-element lowering, private-name
rules, or the remaining class grammar bucket. The computed static `prototype`
positive syntax boundary exposed a separate Wasm class-definition bug: public
static fields, methods/accessors and auto-accessors now reject that run-time key
against the class constructor's non-configurable `prototype`, with field and
auto-accessor initializer order preserved. This adjacent T09/T10 repair does
not turn that run-time TypeError into an early error.

Strict-mode `WithStatement` parsing now has one closed condition for Boa's sole
exact wording. Strict Script directives, strict ordinary functions, class
methods and Module code reject with the same typed `Early`/`SyntaxError`
diagnostic, while sloppy Scripts and sloppy ordinary functions remain valid.
The boundary is recorded in
`docs/rust-rewrite/contracts/strict-mode-with-statement-early-errors.md`. This
is classification only: it does not change valid sloppy `with` lowering or
Object Environment Record semantics, close adjacent strict-mode rules, or
complete the statements grammar bucket. At `2026-08-23`, the capped serial
front gate passes `44/44`, the focused IR early-error gate passes `3/3`, and the
exact seven-file pinned cohort passes `7/7` Wasm-AOT executions with zero
failure or non-success outcomes.

## Objective

Make parsing and static-semantics classification complete, deterministic and source-located for the pinned ECMAScript grammar. Keep the parse-once ownership boundary intact while closing the remaining pinned-suite failures.

## Architecture

- `ParsedScript` and `ParsedModule` own the Boa AST/interner pair; access stays inside their non-escaping compiler-session callbacks.
- Parse exactly once per compilation unit and retain failed module attempts as structured rejections rather than retrying them.
- Preserve script vs module goal, filename, spans, strictness and source text in the parsed product.
- Convert parser panics into structured diagnostics without hiding compiler bugs. Known unsupported parser constructs must be distinguishable from malformed JavaScript.
- Keep Boa as an implementation dependency, not the public IR contract, so it can be upgraded or replaced deliberately.

## Grammar coverage

Drive parser work from T01's failure inventory and Test262 feature metadata. Include:

- scripts, modules, hashbang, directives and strict-mode transitions;
- all declaration/expression/statement forms in the pin;
- classes, private names, static blocks and current standardized syntax;
- async/generator syntax, `yield`, `await` and contextual-keyword restrictions;
- import/export forms, import attributes and dynamic import syntax present in the pin;
- optional chaining, nullish operators, logical assignment, numeric separators, BigInt and regexp literals;
- Annex B grammar extensions where enabled.

## Early errors

Implement explicit static-semantics checks with correct phase and error type, including:

- duplicate lexical/private/export names;
- binding-name restrictions and strict reserved words;
- invalid `break`/`continue` targets;
- illegal `return`, `super`, `new.target`, `yield` and `await` contexts;
- duplicate parameters under the correct strict/simple-list rules;
- class constructor/private-element restrictions;
- module import/export conflicts;
- destructuring and assignment-target validity;
- `__proto__` duplicate literal restrictions;
- Annex B exceptions.

Do not treat runtime errors as acceptable substitutes for parse/early errors.

## Diagnostics

Add stable diagnostic codes, phase (`parse` or `early`), error constructor and source span. Test262 negative tests should compare phase/type through structured data, not string fragments.

## Acceptance criteria

- Compilation parses each unit once.
- Parser panic cases become deterministic compiler failures and have minimized regression tests.
- All negative parse/early cases are classified at the required phase.
- Script/module goal differences are covered.
- Upgrading Boa does not require feature modules to depend directly on Boa AST internals.
- The parser/early-error buckets from T01 reach zero for the pinned suite, excluding only explicitly documented upstream parser defects with a vendored fix task.

## Required tests

```sh
cargo test -p lila-front --quiet
cargo test -p lila-ir early_error --quiet
cargo test -p lila-engine --quiet
./target/debug/lila test262 run language --execution-backend wasm
```

During development run focused `language/expressions`, `language/statements`, `language/declarations`, `language/module-code` and negative-phase shards rather than the full language tree on every edit.
