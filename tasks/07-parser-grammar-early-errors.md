# T07 — Parser boundary, grammar coverage and early errors

**Status:** In progress — parse-once boundary plus ObjectLiteral CoverInitializedName, Script top-level `new.target`, top-level `super` and `using`, for-in and switch-clause `using` declarations, the callable-parameter `Contains YieldExpression`/`Contains AwaitExpression` matrix across declarations, expressions, methods and arrows, callable non-simple-parameter `ContainsUseStrict`, ordinary FunctionExpression/FunctionDeclaration, AsyncFunctionExpression and GeneratorExpression `Contains super`, duplicate formal/catch-parameter, catch-body conflict, duplicate-class-constructor/private-name, constructor method/private-name, public-static-method `prototype` and class-field literal-name restrictions, class static-block `ContainsAwait`, class static-block/field `ContainsArguments`, strict-mode `with`/delete, duplicate static import-attribute-key, optional-chain tagged-template, for-head/body declaration-conflict, duplicate `ForDeclaration` BoundNames, lexical bound-name `let` and `import.meta` outside Module classification implemented; class field-initializer `SuperCall` and all four expression/declaration `Contains super` classifications focused-verified while broader grammar/early-error closure remains

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

### Focused-verified 2026-08-23: Script top-level `super`

`ScriptTopLevelSuper` (`E_SCRIPT_TOP_LEVEL_SUPER`) now owns the exact pinned-Boa
ScriptBody condition whose `StatementList Contains super`. Its exact-message
classifier cannot absorb the eleven adjacent callable and class producers that
reuse the raw phrase, while the separate Module condition remains
`ModuleTopLevelSuper`. The permanent source matrix follows lexical arrow,
class-heritage and computed-name traversal and preserves method-owned stopping
boundaries. Source guards also pin the single product Script/Module parse route
and the workspace's sole normal Boa-parser dependency.

Under the shared eight-core, 22 GB cap, the focused front group passes `4/4`,
the traversal guard passes `1/1`, and the complete front library passes
`119/119`. The complete relevant IR early and graph groups pass `44/44` and
`41/41`, with their focused witnesses each passing `1/1`. The complete
eight-file cohort passes exactly `16/16` Wasm-AOT variants with every failure
bucket at zero under `--jobs 1 --threads 1`. A broader `lila-ir` run exposed two
unrelated lowerer-test failures, confirmed by exact reruns, so no complete-IR or
aggregate green claim is made. The source of truth is
`docs/rust-rewrite/contracts/script-top-level-super-early-errors.md`.

### Focused-verified 2026-08-24: class constructor and static-block `super()`

The current head contains the two condition-specific producer repairs,
closed codes, anchored classifier rows, exhaustive IR arms, direct Script and
Module matrices, precedence controls, retained dependency witnesses and shared
vendored-source guard for `ClassBaseConstructorHasDirectSuper` and
`ClassStaticBlockContainsSuperCall`. Their complete three-file Test262 cohort
passes all `6/6` Wasm-AOT variants with every non-success bucket at zero; the
current shared producer census also passes the complete `138/138` front gate
and relevant `49/49` IR early and `49/49` graph groups. The source of truth is
`docs/rust-rewrite/contracts/class-super-call-early-errors.md`.

### Focused-verified 2026-08-24: class field initializer `Contains SuperCall`

`ClassFieldInitializerContainsSuperCall`
(`E_CLASS_FIELD_INITIALIZER_CONTAINS_SUPER_CALL`) now owns the one class-field
condition shared by public/private, instance/static and auto-accessor
initializers. Pinned Boa's four exhaustive class-element arms keep their
existing optional-initializer and `ContainsSymbol::SuperCall` predicates but
now emit one condition-specific message instead of the generic text shared
with Script and callable producers. The front domain and parse table grow from
64 to 65 entries, and the exhaustive IR map derives `Early` / `SyntaxError`.

Direct tests cover all field shapes, both goals, base/derived and lexical-arrow
traversal, positive `SuperProperty` and nested-class boundaries, and precedence
against field `ContainsArguments` and the deferred base-constructor check. A
real failed Module parse and retained rejected dependency prove structured IR
and graph projection. The shared source guard now pins the current three generic
messages, the unique ordinary function expression/declaration, async-function-
expression and generator-expression producers, four field-specific branches,
unique constructor/static-block messages, traversal rules, and sole parse/
classifier boundary.

The complete front library passes `125/125`, while the relevant IR early and
graph groups pass `46/46` and `43/43`. The exact pinned static-source cohort is
60 generated physical files, 30 each under class expressions and statements,
and passes all `120/120` sloppy/strict Wasm-AOT variants under `--jobs 1
--threads 1`, with every non-success bucket at zero. This is bounded diagnostic
closure, not a pass-gain, aggregate refresh or T07-closure claim. The source of truth is
`docs/rust-rewrite/contracts/class-field-initializer-super-call-early-errors.md`.

### Focused-verified 2026-08-24: FunctionExpression `Contains super`

`FunctionExpressionContainsSuper`
(`E_FUNCTION_EXPRESSION_CONTAINS_SUPER`) now owns the four ordinary
FunctionExpression conditions where `FormalParameters` or `FunctionBody`
`Contains SuperProperty` or `Contains SuperCall`. The pinned parser already
checks the completed function node; its sole producer now has a condition-
specific message instead of the raw text shared with Script and four remaining
callable producers. The closed front domain and parse table grow from 65 to 66
entries, and the exhaustive IR map projects the code as `Early` /
`SyntaxError`.

Direct tests cover parameter and body positions, both super forms, named and
anonymous expressions, lexical ordinary/async arrows and both parse goals.
Positive and precedence controls retain nested-class ownership and the
existing duplicate-parameter, non-simple Use Strict and parameter/body lexical
conflict owners. Real Module and retained rejected-dependency tests preserve
the typed code through IR and graph construction. The shared structural guard
pins the unique completed-node producer, the separately typed ordinary
declaration, async-function-expression and generator-expression messages,
three remaining generic messages, the adjacent producer census, `Contains`
traversal and the sole product classifier boundary.

The complete front library passes `129/129`; the relevant IR early and graph
groups pass `47/47` and `45/45`. The exact pinned cohort is four unflagged
parse-negative files under `language/expressions/function/`, and all eight
sloppy/strict Wasm-AOT variants pass with every non-success bucket at zero.
This is a typed-classification closure rather than a new pass-gain or aggregate
status claim. The source of truth is
`docs/rust-rewrite/contracts/function-expression-contains-super-early-errors.md`.

### Focused-verified 2026-08-24: FunctionDeclaration `Contains super`

`FunctionDeclarationContainsSuper`
(`E_FUNCTION_DECLARATION_CONTAINS_SUPER`) now owns the four ordinary
FunctionDeclaration conditions where `FormalParameters` or `FunctionBody`
`Contains SuperProperty` or `Contains SuperCall`. Pinned Boa keeps one shared
predicate for all hoistable callable declarations; a private production-owned
message hook retains the generic default for generator/async forms and selects
the new message only for the ordinary declaration implementation. The closed
front domain and parse table grow from 66 to 67 entries, and the exhaustive IR
map projects the code as `Early` / `SyntaxError`.

Direct tests cover both super forms, parameter and body positions, lexical
ordinary/async arrows and both parse goals. Positive and precedence controls
retain nested callable/class ownership and the existing duplicate-parameter,
non-simple Use Strict and parameter/body lexical-conflict owners. A real
exported declaration failure and retained rejected dependency preserve the
typed code through IR and graph construction; a valid exported function remains
a parsed graph node.

The shared source guard pins one generic declaration default, one ordinary
override, no override in the three adjacent declaration implementations, the
single body-or-parameters predicate and its early-error order. It also pins the
separately typed async-function-expression and generator-expression producers
and three remaining generic raw messages. The exact pinned
cohort is four unflagged parse-negative files under
`language/statements/function/`, expanding to exactly eight sloppy/strict
Wasm-AOT variants. The complete front library passes `134/134`; the relevant IR
early and graph groups pass `48/48` and `47/47`. The exact cohort passes `8/8`
with every non-success bucket at zero. This is typed-classification evidence,
not an aggregate-status refresh. The source of truth is
`docs/rust-rewrite/contracts/function-declaration-contains-super-early-errors.md`.

### Focused-verified 2026-08-24: AsyncFunctionExpression `Contains super`

`AsyncFunctionExpressionContainsSuper`
(`E_ASYNC_FUNCTION_EXPRESSION_CONTAINS_SUPER`) now owns the four
AsyncFunctionExpression conditions where `FormalParameters` or
`AsyncFunctionBody` `Contains SuperProperty` or `Contains SuperCall`. Its sole
pinned producer already checks the completed node and now emits one production-
specific message instead of the generic text retained by adjacent generator and
async declaration/expression forms. The closed front domain and parse table grow
from 67 to 68 entries; exhaustive IR projection derives `Early` /
`SyntaxError`.

Direct tests cover named/anonymous expressions, both super forms, parameter and
body positions, lexical ordinary/async arrows and both goals. Positive and
precedence controls retain nested callable/class boundaries and the earlier
parameter `Contains AwaitExpression`, duplicate-parameter, non-simple Use Strict
and parameter/body lexical-conflict owners. Real Module and retained dependency
witnesses preserve the code through IR and graph construction.

The shared source guard pins the unique completed-node producer, the later
generator-expression-specific producer, exactly three remaining generic raw
messages, adjacent production boundaries and traversal.
The complete front library passes `138/138`; the relevant IR early and graph
groups pass `49/49` and `49/49`. The exact pinned cohort is four unflagged
parse-negative files under `language/expressions/async-function/`, and all
eight sloppy/strict Wasm-AOT variants pass with every non-success bucket at
zero. This is typed-classification evidence, not an aggregate-status refresh.
The source of truth is
`docs/rust-rewrite/contracts/async-function-expression-contains-super-early-errors.md`.

### Focused-verified 2026-08-24: GeneratorExpression `Contains super`

`GeneratorExpressionContainsSuper`
(`E_GENERATOR_EXPRESSION_CONTAINS_SUPER`) now owns the four
GeneratorExpression conditions where `FormalParameters` or `GeneratorBody`
`Contains SuperProperty` or `Contains SuperCall`. Its sole pinned producer
already checks the completed node and now emits one production-specific message
instead of the generic text retained by generator/async declarations and the
async-generator expression. The closed front domain and parse table grow from
68 to 69 entries; exhaustive IR projection derives `Early` / `SyntaxError`.

Direct tests cover named/anonymous expressions, both super forms, parameter and
body positions, lexical ordinary/async arrows and both goals. Positive and
precedence controls retain nested callable/class boundaries and the earlier
parameter `Contains YieldExpression`, duplicate-parameter, non-simple Use
Strict and parameter/body lexical-conflict owners. Real Module and retained
dependency witnesses preserve the code through IR and graph construction.

The shared source guard pins the unique completed-node producer, exactly three
remaining generic raw messages, adjacent production boundaries and traversal.
The exact pinned Test262 cohort is zero files: all 290 JavaScript files in the
complete `language/expressions/generators/` tree contain no `SuperProperty`,
`SuperCall`, or static `super` source. The four async-generator leaves belong
to a different producer and are excluded. The complete front library passes
`142/142`; the relevant IR early and graph groups pass `50/50` and `51/51`.
This zero-file inventory supports no Test262 pass or aggregate-status claim.
The source of truth is
`docs/rust-rewrite/contracts/generator-expression-contains-super-early-errors.md`.

### Focused-verified 2026-08-23: lexical bound-name `let`

`LexicalBoundNameLet` (`E_LEXICAL_BOUND_NAME_LET`) now owns the shared
ECMA-262 condition where a lexical declaration or iterable `ForDeclaration`
has the exact name `let` in its `BoundNames`. One shared validator owns
ordinary and classic lexical declarations, while the iterable tail retains its
condition-specific producer. A closed
`BindingIdentifierContext::{General, LexicalDeclaration}` lets only the exact
name `let` cross pinned Boa's generic strict-reserved check in the three lexical
root shapes; array/object recursion preserves that context until the completed
declaration reaches the semantic owner. Every other binding consumer remains
`General`, and other reserved names stay rejected by their existing owner.

The closed front domain and classifier table now have `61/61` entries. Exactly
two anchored rows own the two reachable fixed-message producers, and const
assertions pin that ownership. The exhaustive IR map derives
`Early` / `SyntaxError`; real failed Module parses and retained dependency graph
nodes preserve the typed diagnostic. Direct tests cover both goals, nested and
rest patterns, classic and iterable loops, resource spellings, precedence,
positive boundaries, strict `var let` separation and diagnostic-text
injection. The final vendored inventory records three lexical root opt-ins,
five identifier-leaf propagations and six recursive pattern propagations.

Independent reviews of the vendored repair and front/IR closure were clean.
Under the shared eight-core, 22 GB cap, `cargo fmt --all -- --check`, `cargo xc`
and `git diff --check` are green. The focused front group passes `4/4`, the
exact source guard passes `1/1`, the complete front library passes `112/112`,
the focused real-Module/graph witnesses pass `2/2`, and the complete IR
module-early group passes `43/43`. The ten complete pinned Test262 leaves pass
exactly `14/14` Wasm-AOT variants with every failure and non-success bucket at
zero under `--jobs 1 --threads 1`. This is bounded diagnostic closure: no
aggregate refresh, measured new-pass gain, broader resource/iteration grammar
result, T07 closure or aggregate-conformance claim is made. The source of
truth is
`docs/rust-rewrite/contracts/lexical-bound-name-let-early-errors.md`.

### Focused-verified 2026-08-23: `import.meta` outside Module

`ImportMetaOutsideModule` (`E_IMPORT_META_OUTSIDE_MODULE`) now owns the closed
ECMA-262 goal condition for `import.meta` parsed outside Module. One anchored
classifier row owns pinned Boa's sole fixed producer; the closed front domain
and parse table have 60 and 59 entries, and the exhaustive IR map projects the
code as `Early` / `SyntaxError`. Script witnesses cover direct and nested forms,
while positive Module and retained-graph witnesses prove that direct and nested
`import.meta` remain valid under the Module goal. No vendor repair was needed.

The focused `lila-front` `import_meta_` group passed `5/5`, and the complete
`lila-front --lib` gate passed `108/108`. The exact `lila-ir` classifier test and
retained Module graph witness each passed `1/1`. `cargo xc`,
`cargo fmt --all -- --check`, and `git diff --check` were green. The source guard
was aligned during verification to the actual derived `ModuleParser`
declaration and parser signature; that was guard calibration, not a production
behavior change.

The complete pinned Test262 paths
`language/expressions/import.meta/syntax/goal-script.js`,
`language/expressions/import.meta/syntax/goal-module.js`, and
`language/expressions/import.meta/syntax/goal-module-nested-function.js` passed
exactly `2/2`, `1/1`, and `1/1` Wasm-AOT variants, respectively, with every
failure and non-success bucket at zero. This is bounded diagnostic and goal
closure only: it does not establish a new Test262 pass, broad-suite gain,
runtime `import.meta`, dynamic-source support, T07 closure, or aggregate
conformance. The source of truth is
`docs/rust-rewrite/contracts/import-meta-outside-module-early-errors.md`.

### Focused-verified 2026-08-23: duplicate `ForDeclaration` BoundNames

`ForDeclarationDuplicateBoundName`
(`E_FOR_DECLARATION_DUPLICATE_BOUND_NAME`) now owns the ECMA-262 14.7.5.1
condition where a `let`/`const` iterable-loop `ForDeclaration` binding pattern
has duplicate `BoundNames`. The previously unreachable pinned-Boa producer is
now reached through a bounded parser repair: one private exhaustive
`Statement | ForHead` context replaces both raw loop-initializer booleans, one
shared validator preserves the generic duplicate-lexical owner for ordinary and
classic declarations, and a typed deferred lexical initializer routes `in` /
`of` heads to the existing condition-specific producer while retaining the
lexical keyword position for classic heads.

The closed domain and parse table now have 59 and 58 entries. One anchored
classifier row owns the complete fixed producer prefix, evaluated const
assertions make that ownership compile-time checked, and the exhaustive IR map
projects the new code. Direct Script and Module witnesses cover array/object
patterns across `for-in`, `for-of`, and async `for-await-of`, with permanent
`var`, classic-for, resource-like expression, forbidden-`let`, and mixed
head/body-conflict boundaries. A real failed Module parse and a retained
rejected dependency prove front-to-IR and graph projection.

Independent reviews of the vendored repair and the front/IR closure were clean.
Under the shared CPU cap, `cargo fmt --all -- --check` and `cargo xc` were green.
The first complete `lila-front` run exposed two text-guard count/indent
assumptions; after repair, the exact vendored-source guard passed `1/1` and the
complete library passed `103/103`. The focused `lila-ir` early-module group
passed `42/42`, and the exact retained graph witness passed `1/1`.

The four complete pinned paths
`language/statements/for-in/head-let-bound-names-dup.js`,
`language/statements/for-in/head-const-bound-names-dup.js`,
`language/statements/for-of/head-let-bound-names-dup.js`, and
`language/statements/for-of/head-const-bound-names-dup.js` each passed `2/2`
sloppy/strict Wasm-AOT variants, for exactly `8/8`, with every failure and
non-success bucket at zero under `--jobs 1 --threads 1`. This is bounded
diagnostic closure only: no aggregate refresh, measured new-pass gain, broader
iteration-grammar result, T07 closure, or aggregate-conformance claim is made.

### Focused-verified 2026-08-23: for-head/body declaration conflicts

`ForHeadBodyDeclarationConflict`
(`E_FOR_HEAD_BODY_DECLARATION_CONFLICT`) is the one closed condition shared by
the edition-pinned ECMA-262 2026 14.7.4.1 and 14.7.5.1 `let`/`const` base,
extended by the corresponding living-spec productions to `using` and
`await using`: a classic `for` head's `LexicalDeclaration` or an iterable loop's
`ForDeclaration` has a `BoundName` that also occurs in the body `Statement`'s
`VarDeclaredNames`. Across every Rust source in pinned Boa there are exactly
two producers with the fixed raw message
`For loop initializer declared in loop body`, one for each specification
production. The classifier has one `ParseFailurePattern::StartsWith` row for
the complete rendered prefix ending in `at line`.

The enum and parse-table counts are written as 58 and 57. Evaluated const
assertions make the code parse-owned and require exactly one owning row with
the complete independently spelled prefix. The exhaustive IR mapping owns the
new variant. Eleven direct source shapes run under both Script and Module
goals, including classic-`for` `using`, async classic-`for` `await using`, and
async `for await (let x of [])` conflicts. A real failed Module parse exercises
`module_parse_failure_diagnostic`, and a rejected dependency retained in
`ModuleSourceIr` crosses `build_graph` with the code, `Early` phase,
`SyntaxError` type and nonempty span.

Positive controls preserve `var` heads plus nested function-expression and
`FunctionDeclaration` `VarDeclaredNames` boundaries. A conflicting
`for-in using` source remains owned by the existing `ForInUsingDeclaration`
code. The vendored-source guard inventories the literal across every Rust
source in the pinned Boa package, requires both copies to remain in
`for_statement.rs`, and separately pins each surrounding `bound_names` /
`var_declared_names` intersection rather than trusting a literal count alone.
No vendor file changes.

The exact eight-file, fifteen-variant pinned Test262 cohort is recorded in
`docs/rust-rewrite/contracts/for-head-body-declaration-conflict-early-errors.md`.
Under the shared eight-core cap, `cargo xc` is green, the full front-end library
passes `101/101`, the focused IR early-module tests pass `41/41`, the exact
retained graph witness passes `1/1`, and the eight Test262 files pass exactly
`15/15` Wasm-AOT variants with `--jobs 1 --threads 1`. No new-pass, aggregate,
broader iteration-grammar or T07-closure claim is made.

### Written 2026-08-23: optional-chain tagged-template early errors

`OptionalChainTaggedTemplate` (`E_OPTIONAL_CHAIN_TAGGED_TEMPLATE`) is the one
closed condition for the two forbidden ECMA-262 productions
`?. TemplateLiteral` and `OptionalChain TemplateLiteral`. Pinned Boa has two
adjacent producer branches with the identical raw message
`Invalid tagged template on optional chain`. The written classifier adds one
`ParseFailurePattern::StartsWith` row for the complete rendered prefix ending
in `at line`, so both producers share one typed identity without accepting the
same phrase inside an interpolated Module export diagnostic.

The enum and parse-table counts are written as 57 and 56. Evaluated const
assertions make the new code parse-owned, require exactly one owning row with
the complete reviewed prefix, and preserve `ModuleDuplicateExport` when a
user-chosen export name contains the fixed phrase. The exhaustive IR mapping
owns the new variant. A real failed Module parse exercises
`module_parse_failure_diagnostic`, while a retained rejected
`ModuleSourceIr` dependency crosses `build_graph` rather than relying only on a
constructed message.

Front-end witnesses cover both productions under Script and Module goals,
substituted and unsubstituted templates, their line-terminator forms, and the
valid ordinary-tag, optional-access, optional-call and parenthesized
completed-chain boundaries. A source-count guard pins exactly two message
occurrences and both template-token alternatives at both vendored
optional-chain parser branches without changing vendor code. The exact
eight-file pinned Test262 cohort is recorded in
`docs/rust-rewrite/contracts/optional-chain-tagged-template-early-errors.md`.

Independent adversarial review accepted the typed ownership and strengthened
its exact-prefix, substituted-template and retained-graph witnesses. Under the
shared eight-core cap, `cargo xc` is green, the full front library passes
`97/97`, the focused retained-module group passes `40/40`, and the exact graph
projection passes `1/1`. The eight-file pinned Test262 cohort passes `16/16`
Wasm-AOT executions with all non-success buckets at zero under
`--jobs 1 --threads 1`. No measured pass gain or broader parser-conformance
result is claimed.

### Landed 2026-08-23: duplicate static import-attribute keys

`ModuleDuplicateImportAttributeKey`
(`E_MODULE_DUPLICATE_IMPORT_ATTRIBUTE_KEY`) now owns the one Module early-error
condition where `WithClauseToAttributes` produces two entries with the same
`[[Key]]`. Pinned Boa has exactly two fixed-message producers: its static import
and export-from parsers each emit `duplicate import attribute key`. One
`ParseFailurePattern::StartsWith` row requires Boa's rendered `duplicate import
attribute key at line` prefix and owns both sites without accepting the same
text inside an interpolated local-export diagnostic. A const ownership check
requires every row for this code to remain anchored. This brings the closed
domain to 56 variants and the parse table to 55 rows. The exhaustive IR map
derives `Early` and `SyntaxError`, and a real failed export-from parse exercises
the front-to-IR diagnostic projection.

The source witnesses cover both declaration forms, identifier/string key
equivalence, valid distinct keys and trailing commas, one reviewed occurrence
in each known vendored producer file, and adversarial export-name collisions.
The independent
`ModuleRequestAttributesIr::try_new` invariant remains intact: its
`DuplicateImportAttributeKeyIr` includes the duplicated key and is not a parse
classification.

Under the shared eight-core cap, `cargo xc` is green; the focused front and IR
projection filters pass `3/3` and `1/1`, the broad front and module-early
cohorts pass `93/93` and `39/39`, and the independent IR record invariant
passes `1/1`. The exact
`language/module-code/import-attributes/early-dup-attribute-key` Test262 filter
passes `3/3` Wasm-AOT executions with every non-success bucket at zero. This is
bounded classification evidence, not a measured pass gain or a claim about the
broader current-pin cohort, host import-attribute semantics, dynamic-import
option semantics, JSON-module behavior, T07 closure or aggregate conformance.
The source of truth is
`docs/rust-rewrite/contracts/duplicate-import-attribute-key-early-errors.md`.

### Landed 2026-08-23: delete-reference early errors

This fixed-message batch splits the strict-mode delete early error
by the two disjoint operand families detected by Boa's one `UnaryExpression`
delete branch:
`StrictModeDeleteIdentifierReference`
(`E_STRICT_MODE_DELETE_IDENTIFIER_REFERENCE`) and
`StrictModeDeletePrivateReference`
(`E_STRICT_MODE_DELETE_PRIVATE_REFERENCE`). The complete rendered prefixes are
`cannot delete variables in strict mode at line` and
`cannot delete private fields at line`, with exactly one adjacent producer each
in `vendor/boa_parser-0.21.1/src/parser/expression/unary.rs`.

Review found two parser defects before classification: the private arm omitted
the shared strictness guard and did not recognize a current-spec OptionalChain
whose final operation is private. The repaired branch retains recursive
parenthesis flattening, gates both operand families on strictness, and
exhaustively classifies the final optional operation. The implementation
has 55 closed variants and 54 parse-table rows, up from 53 and 52. The exact
current-pin cohorts are two `onlyStrict`
identifier files plus 192 generated class files with no execution-mode flag,
for 194 physical files and 386 Wasm-AOT executions.

The pinned private-delete cohort covers member and call-expression endings;
permanent source witnesses additionally own the current-spec optional-chain
forms and the sloppy-undeclared anti-conflation boundary, which must remain
`InvalidPrivateIdentifier` rather than a delete-specific code.

The typed rows, parser repair, durable source witnesses and retained dependency
graph projection are complete. The capped serial front, early-module,
retained-graph and focused IR gates pass `89/89`, `38/38`, `1/1` and `3/3`;
`cargo xc` and the release CLI build are green; and the exact cohort passes all
`386/386` Wasm-AOT executions with zero failure or non-success outcomes and an
exact completed-ID set match. There is no focused pre-change snapshot, so this
is typed diagnostic closure and bounded no-regression, not a measured pass
gain. The theory source of truth is
`docs/rust-rewrite/contracts/delete-reference-early-errors.md`.

### Landed 2026-08-23: callable non-simple parameters plus `ContainsUseStrict`

One new `EarlyErrorCode::CallableNonSimpleParametersContainUseStrict` owns
the common callable early error where the callable's own body contains a Use
Strict Directive and its own parameter list is non-simple. The one wire name is
`E_CALLABLE_NON_SIMPLE_PARAMETERS_CONTAIN_USE_STRICT`; the classifier uses
Boa's complete fixed rendered prefix and the exhaustive IR map derives
`Early`/`SyntaxError` for entry and retained dependency parsing. The closed
domain has 53 variants and the one parse table has 52 rows. A const ownership
witness makes deleting this parse-owned row while retaining the variant fail to
compile.

The pinned parser began with eighteen copies of the raw message, but those were
not eighteen honest producers. The direct binding-identifier arrow parser could
not receive a non-simple list, the private class-getter branch accepted
parameters that its grammar forbids, and the two class-setter branches accepted
unrestricted formal lists. Private getters now require `()`, class setters
parse exactly one non-rest formal parameter, and the impossible direct-arrow
check is gone. A source inventory pins the resulting sixteen executable,
spec-conforming producer sites and the direct-arrow count of zero. The
identical dynamic-`Function` engine message remains outside the AOT parser
classifier.

The durable source matrix exercises every remaining site under both goals,
plus the conjunction, containment and getter/setter grammar boundaries. A real
retained Module failure proves dependency projection. The exact pinned
cohort is the 110 language tests containing `FunctionBodyContainsUseStrict` or
`ContainsUseStrict`; with no mode flags they expand to 220 sloppy/strict
Wasm-AOT executions. The capped serial front, retained-module and focused IR
gates pass `85/85`, `38/38` and `3/3`; `cargo xc` is green; and the exact cohort
passes `220/220` with every failure and non-success bucket at zero. No focused
pre-change snapshot exists, so this is typed diagnostic closure and bounded
no-regression evidence, not a pass gain, dynamic source support, runtime
parameter semantics, T07 closure or aggregate conformance. The source of truth
is `docs/rust-rewrite/contracts/callable-non-simple-parameters-contain-use-strict-early-errors.md`.

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

Async-generator expressions whose own FormalParameters contain an
`AwaitExpression` now have one closed condition for Boa's sole fixed-message
await producer. Anonymous and named forms reject under Script and Module goals,
and retained dependency parsing carries the same typed `Early`/`SyntaxError`
diagnostic. Async-generator bodies and nested async-function initializers remain
valid containment boundaries. The yield sibling, declaration forms and methods
retain their distinct pinned producers. The contract is recorded in
`docs/rust-rewrite/contracts/async-generator-expression-parameters-contain-await-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `69/69`, the focused IR
early-error gate passes `3/3`, and the exact one-file pinned cohort passes `2/2`
sloppy/strict Wasm-AOT executions with zero failure or non-success outcomes.
This is bounded classification, not all async-generator grammar, direct eval,
T07 or aggregate closure.

Ordinary async and async-generator declarations whose own FormalParameters
contain an `AwaitExpression` now share one closed condition for Boa's sole
fixed-message callable-declaration producer. Script, Module and retained
dependency parsing carry the same typed `Early`/`SyntaxError` diagnostic, while
async bodies and nested async-function initializers remain valid containment
boundaries. The pinned suite has no source that reaches this exact producer, so
the condition relies on direct front-end and retained-module witnesses rather
than claiming nearby bare-`await` tests. The contract is recorded in
`docs/rust-rewrite/contracts/async-declaration-parameters-contain-await-early-errors.md`.
At `2026-08-23`, the capped serial front gate passes `75/75`, the retained-
module early suite passes `35/35`, the focused IR early-error gate passes
`3/3`, and `cargo xc` passes. This is bounded classification, not all async
grammar, direct eval, T07 or aggregate closure.

Generator methods whose own UniqueFormalParameters contain a
`YieldExpression`, and async-generator methods whose own parameters contain a
`YieldExpression` or `AwaitExpression`, now have three distinct closed
conditions for Boa's three fixed-message method producers. Object and class
methods share each producer, while method bodies and nested callable
initializers remain valid containment boundaries. The exact ordinary
generator-method cohort is five pinned files expanding to `9/9` passing Wasm-
AOT executions with every failure and non-success bucket at zero; the pinned
suite has no source reaching either async-generator-method producer. The
contract is recorded in
`docs/rust-rewrite/contracts/generator-method-parameters-contain-yield-await-early-errors.md`.
At `2026-08-23`, the same capped serial front, retained-module, focused IR and
workspace gates pass `75/75`, `35/35`, `3/3` and `cargo xc`, respectively. This
is bounded classification, not all generator grammar, direct eval, T07 or
aggregate closure.

Ordinary and async arrows whose own parameters contain a `YieldExpression` or
`AwaitExpression` now share two closed conditions. Pinned Boa emits two fixed
Yield wordings and one Await wording; the table maps that producer variation
to one code per static-semantics condition instead of splitting codes by arrow
form. A narrow vendored repair carries the enclosing `AllowYield` grammar
parameter into parenthesized async-arrow parameters, making the existing Yield
check reachable without admitting any new valid source. The exact pinned code
cohort is two files expanding to four sloppy/strict Wasm-AOT executions; both
reach the ordinary-arrow producer. No pinned source reaches the repaired
parenthesized async-arrow Yield producer, so its evidence is the direct front-
end and retained-module witnesses. The contract is recorded in
`docs/rust-rewrite/contracts/arrow-parameters-contain-yield-await-early-errors.md`.
At `2026-08-23`, the exact cohort passes `4/4` sloppy/strict Wasm-AOT
executions with every failure and non-success bucket at zero.

Async function expressions and ordinary async methods whose own parameters
contain an `AwaitExpression` now have two distinct closed conditions. Pinned
Boa parsed both Await-enabled parameter lists without applying their required
post-parameter containment checks; two vendored guards now reject before body
parsing with producer-specific fixed messages. Object and class methods share
the method producer, while named and anonymous function expressions share the
expression producer. The pinned suite has no complete AwaitExpression source
that reaches either repaired check, so direct front-end and retained-module
witnesses own these contracts. The boundary is recorded in
`docs/rust-rewrite/contracts/async-function-expression-and-method-parameters-contain-await-early-errors.md`.
At `2026-08-23`, the shared capped serial front, retained-module, focused IR
and workspace gates pass `81/81`, `37/37`, `3/3` and `cargo xc`, respectively.
No Test262 result is claimed for the repaired producers.

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
