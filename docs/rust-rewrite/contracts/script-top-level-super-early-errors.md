# Script top-level `super` early errors

**Status:** Product condition and GeneratorExpression-updated shared producer
census focused-verified 2026-08-24

## Decision

A `ScriptBody` whose `StatementList Contains super` is one closed
pre-evaluation condition:

`EarlyErrorCode::ScriptTopLevelSuper`

Its sole wire spelling is `E_SCRIPT_TOP_LEVEL_SUPER`. “Top-level” names the
`ScriptBody` static-semantics boundary, not literal syntax depth. The
`Contains` operation follows ordinary and async arrow functions because they
inherit `super` lexically. It also follows class heritage and the computed
public names selected by `ComputedPropertyContains`, while method,
constructor, field-initializer and static-block bodies establish or use their
own class-owned boundaries.

This is distinct from the existing Module condition and code:

`EarlyErrorCode::ModuleTopLevelSuper` / `E_MODULE_TOP_LEVEL_SUPER`

The two source goals apply parallel specification rules, but pinned Boa emits
different messages from different goal-owned producers. They must not be
collapsed merely because both rules mention top-level `super`.

## Specification and goal boundary

The edition-pinned ECMA-262 2026
[16.1.1, Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-scripts-and-modules.html#sec-scripts-static-semantics-early-errors)
rule for

```text
ScriptBody : StatementList
```

requires a `SyntaxError` if `StatementList Contains super`, except when the
source containing `super` is eval code processed by a direct eval. That
exception does not make `super` generally valid in direct eval. `PerformEval`
separately rejects a body containing `SuperProperty` unless the calling
function environment supplies `inMethod`, and rejects a body containing
`SuperCall` unless it supplies `inDerivedConstructor`. The same Script section
separately applies the adjacent `Contains NewTarget` condition already owned by
`ScriptTopLevelNewTarget`. Clause 17 makes these listed conditions early errors
rather than ordinary parser failures.

The `Contains` rules deliberately traverse `ArrowFunction` and
`AsyncArrowFunction` for `super`. An arrow at Script-body scope has no enclosing
method state from which to inherit `super`, so both a reference in its
parameters and a reference in its body reach the Script rule. The same
operation traverses a class heritage expression and computed public method or
field names, and traverses a computed object-method name while excluding the
method body. Ordinary, generator, async and async-generator function
definitions instead have their own `SuperProperty` / `SuperCall` early errors;
their pinned Boa producers are not owned by this code.

Under Module goal, the corresponding ECMA-262 Module rule rejects a
`ModuleItemList` that `Contains super`. Pinned Boa's Module parser already emits

```text
module cannot contain `super` on the top-level
```

and the current classifier maps it to `ModuleTopLevelSuper`. That code and
message owner remain unchanged.

Direct eval is the explicit T13 dynamic-source boundary. Its `super` legality
depends on the `inMethod` and `inDerivedConstructor` flags computed by
`PerformEval`; it is not an unconditional exception. This contract does not
make eval source AOT-compilable, classify those eval-specific rules through the
static Script entry point, or change the architecture's dynamic-code generation
policy.

## Measured pinned-Boa producer boundary

Pinned `boa_parser-0.21.1` has exactly one semantic producer for the Script
condition. In `vendor/boa_parser-0.21.1/src/parser/mod.rs`, `ScriptBody::parse`
first parses the complete statement list and then, while outside direct eval,
applies:

```text
if contains(&body, ContainsSymbol::Super) {
    return Err(Error::general("invalid super usage", Position::new(1, 1)));
}
```

`Error::General` appends its position. The complete rendered message owned by
this condition is therefore exactly:

```text
invalid super usage at line 1, col 1
```

The fixed `Position::new(1, 1)` is part of the current pin's classification
boundary. It distinguishes this producer from the other reachable producers
that reuse the raw literal `invalid super usage`.

After the separately typed class-super-call, class-field-initializer, ordinary-
function expression/declaration, async-function-expression and generator-
expression repairs, across every Rust source in pinned `boa_parser-0.21.1` that
raw literal occurs exactly three times:

| Parser owner | Raw occurrences | Position source | New code owns it |
| --- | ---: | --- | --- |
| `parser/mod.rs` ScriptBody check | 1 | fixed `Position::new(1, 1)` | yes |
| shared hoistable-declaration default for generator/async forms | 1 | common branch retains `params_start_position` | no |
| async-generator expression parser | 1 | parameter-start position | no |

The other two positions occur only after a function head and cannot render
line 1, column 1. They cover distinct callable conditions and must remain
unclassified by this lane. A broad row for `invalid super usage at line` would
falsely merge all three raw-message owners. The base-constructor and static-block
conditions have unique messages and codes under
`class-super-call-early-errors.md`; the four field-initializer producers are
owned by `class-field-initializer-super-call-early-errors.md`; and the ordinary
function-expression producer is owned by
`function-expression-contains-super-early-errors.md`. The ordinary
function-declaration producer is separately owned by
`function-declaration-contains-super-early-errors.md`; the async-function-
expression producer is owned by
`async-function-expression-contains-super-early-errors.md`; the generator-
expression producer is owned by
`generator-expression-contains-super-early-errors.md`.

No vendor repair is required for the Script producer. Its existing
fixed-position message remains sufficient for an exact-message classifier row.
A future change that makes another producer render the same 1:1 message would
invalidate this contract and must fail the structural source guard before
classification is broadened.

Before this extension, the complete Script message matches no row in
`lila-front`'s classifier. The source is therefore reported as
`ParseCode::Malformed`, with phase `Parse` and native `SyntaxError`. The
extension changes only its typed identity and derived phase to
`ParseCode::Early(ScriptTopLevelSuper)` / `Early`; it does not create a new
syntax rejection or a Test262 pass by itself.

## Exact typed encoding

The pre-extension `EarlyErrorCode` domain and parse-failure table each contain
61 entries. This extension grows both array-typed counts to 62.

The classifier addition is exactly one code and one row:

```text
ScriptTopLevelSuper => "E_SCRIPT_TOP_LEVEL_SUPER";

const SCRIPT_TOP_LEVEL_SUPER_MESSAGE: &str =
    "invalid super usage at line 1, col 1";

ParseFailureRule {
    pattern: ParseFailurePattern::Exact(SCRIPT_TOP_LEVEL_SUPER_MESSAGE),
    code: EarlyErrorCode::ScriptTopLevelSuper,
    witnesses: &["invalid super usage at line 1, col 1"],
}
```

`Exact` is required because the reviewed text is the complete current message.
`StartsWith("invalid super usage at line 1, col 1")` is not exact: decimal
positions such as `col 10` through `col 19` share that byte prefix, and the
other raw-message producers can report those columns. A `ContainsAll` row would
also allow user-controlled text embedded later in another Boa diagnostic to
forge this condition. The row must be accompanied by:

- an evaluated `ParseClassified::from_parse_table` const assertion, so deleting
  the row while retaining the enum variant fails to build;
- an exact-single-owner const assertion that independently spells the complete
  reviewed message and requires the sole owner to use `Exact`;
- the existing table-wide witness-disjointness, wire-name and
  interpolation-safety assertions;
- an injection-safety const assertion using a user-controlled Module export
  name that contains the complete message; and
- an explicit arm in `lila-ir`'s exhaustive `EarlyErrorCode` mapping to
  `IrDiagnosticKind::EarlyError`, with no catch-all.

The injection witness must prove that a diagnostic such as

```text
exported name `invalid super usage at line 1, col 1` declared multiple times
```

retains `ModuleDuplicateExport` rather than selecting the new exact row. Direct
classifier witnesses must prove that both the adjacent message
`invalid super usage at line 1, col 2` and the decimal-prefix collision
`invalid super usage at line 1, col 10` do not acquire the new code.

## Direct source and precedence matrix

The permanent front-end matrix must establish the direct cases and one
representative of every distinct pinned `Contains` traversal boundary. Every
row reports `ScriptTopLevelSuper`, phase `Early`, native `SyntaxError` and a
nonempty span:

| Boundary | Source |
| --- | --- |
| direct SuperCall | `super();` |
| direct SuperProperty | `super.value;` |
| strict Script | `"use strict"; super.value;` |
| ordinary-arrow body | `() => { super(); };` |
| ordinary-arrow body property | `() => super.value;` |
| async-arrow body call | `async () => { super(); };` |
| async-arrow body property | `async () => super.value;` |
| async-arrow parameter call | `async (value = super()) => value;` |
| async-arrow parameter property | `async (value = super.value) => value;` |
| nested-arrow traversal | `() => () => super.value;` |
| ordinary-arrow parameter traversal | `(value = super.value) => value;` |
| class-declaration heritage | `class C extends super.value {}` |
| class-expression heritage | `(class extends super.value {});` |
| computed class method name | `class C { [super.value]() {} }` |
| computed instance-field name | `class C { [super.value]; }` |
| computed static-field name | `class C { static [super.value]; }` |
| computed object-method name | `({ [super.value]() {} });` |
| computed object-getter name | `({ get [super.value]() { return 0; } });` |
| computed ordinary-property name | `({ [super.value]: 0 });` |

The same sources parsed under Module goal must report the existing
`ModuleTopLevelSuper` code, not the new Script code.

Positive controls must preserve valid method-owned `super` forms under both
goals. Each line below is an independent source witness; the lines are not
concatenated into one program:

```text
class Base {}; class Derived extends Base { constructor() { super(); } }
class Base {}; class Derived extends Base { method() { return super.value; } }
class Base {}; class Derived extends Base { method() { return () => super.value; } }
class Base {}; class Derived extends Base { field = super.value; }
class Base {}; class Derived extends Base { static field = super.value; }
class Base {}; class Derived extends Base { static { void super.value; } }
({ method() { return super.value; } });
({ get value() { return super.value; } });
({ set value(input) { void super.value; } });
```

Pinned Boa's class-element visitor currently omits public auto-accessor names
from `ComputedPropertyContains`. A computed auto-accessor name is therefore not
part of the producer surface claimed here. The structural guard records that
omission so a vendor change cannot silently alter the cohort; repairing it is
adjacent parser/`Contains` debt, not part of this classifier-only lane.

Adjacent semantic owners must not be absorbed. An ordinary function containing
`super.value` remains outside this code, while base constructors, field
initializers and static blocks containing `super()` retain their distinct
class-owned codes.

One mixed Script source pins Boa's current semantic-check order:

```text
super.value; new.target;
```

The ScriptBody checks `Contains super` before `Contains NewTarget`, so this
source must report `ScriptTopLevelSuper`. This is a parser-order regression
witness, not a claim that ECMA-262 makes simultaneous early-error ordering
observable.

The fixed 1:1 position produces a nonempty source span at the compilation-unit
start. This lane does not claim token-precise location for the actual `super`.
Improving that span would require a separate Boa AST/location repair and would
change the message contract recorded here.

## IR and retained dependency graph boundary

The enum extension must make `lila-ir`'s rejection-kind match exhaustive and
map `ScriptTopLevelSuper` to `IrDiagnosticKind::EarlyError`. A classifier-to-IR
witness may project the complete rendered message and derive phase `Early` and
native `SyntaxError`, but it must not present that projection as a real Module
parse.

This condition is honestly Script-only in Lila's static product path. Loaded
dependencies are parsed as Module, so a real rejected `ModuleSourceIr` carrying
`ScriptTopLevelSuper` cannot exist. Tests must not construct such a rejection
and call it retained-graph evidence.

The real Module and graph controls are instead:

- parse a Module containing top-level `super` and require
  `ModuleTopLevelSuper`, phase `Early`, native `SyntaxError` and a nonempty
  span;
- retain a dependency with that Module failure in `ModuleSourceIr`, carry it
  through `build_graph`, and require the existing Module code to survive; and
- retain a valid Module containing a derived constructor or method-owned
  `super` form and require successful graph construction.

These controls prove the goal split and parse-once retention boundary. The
exhaustive IR match proves the new Script code has an IR owner without
inventing an unreachable Module producer.

## Vendored-source structural guard

A durable source guard must recursively inventory the pinned Boa parser and
prove all of the following:

- the raw literal `invalid super usage` occurs exactly three times across
  Rust sources;
- exactly one occurrence is the ScriptBody call with the complete
  `Error::general(..., Position::new(1, 1))` shape;
- that call remains under `if !self.direct_eval` and the exact
  `contains(&body, ContainsSymbol::Super)` condition;
- the Script super check remains before the adjacent NewTarget, private-name,
  label and cover-initialized-name checks;
- the shared declaration branch and async-generator-expression producer retain
  their reviewed parameter-start positions rather than acquiring the fixed 1:1
  coordinate;
- pinned `boa_ast` keeps ordinary callable bodies as stopping boundaries,
  traverses ordinary and async arrows, class heritage and computed public
  method/field names, traverses computed object-method names, and reaches a
  computed expression through `PropertyName::Computed`;
- pinned `boa_ast` continues to omit public auto-accessor names from the class
  element visitor until that adjacent debt is deliberately repaired;
- the Module parser retains exactly one separate raw message
  ``module cannot contain `super` on the top-level`` and its existing
  `ModuleTopLevelSuper` classifier owner; and
- the product prefix of `lila-front` retains exactly one Script parse, one
  Module parse and one classifier call, no other `lila-front` source owns a Boa
  parse route, and no other workspace crate declares Boa parser as a normal
  dependency.

The guard must inspect source structure as well as literal counts. A count of
ten alone would not detect moving the fixed position to the wrong producer
or broadening the Script check to direct eval, changing the traversal boundary
or adding a second product parse route.

## Complete current-pin Test262 cohort

At the repository's declared Test262 revision
`e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the complete static ScriptBody
producer cohort contains eight physical files. None declares `onlyStrict`,
`noStrict`, `raw` or `module`, so Lila's closed execution-plan expansion runs
each as sloppy Script and strict Script: exactly sixteen variants.

The four dedicated clause-16.1.1 global-code leaves are:

- `language/global-code/super-call.js`;
- `language/global-code/super-call-arrow.js`;
- `language/global-code/super-prop.js`; and
- `language/global-code/super-prop-arrow.js`.

They expand to eight variants. That `4/8` set is the dedicated global-code
cohort, not the complete producer cohort.

Four additional async-arrow leaves reach the same ScriptBody producer:

- `language/expressions/async-arrow-function/early-errors-arrow-body-contains-super-call.js`;
- `language/expressions/async-arrow-function/early-errors-arrow-body-contains-super-property.js`;
- `language/expressions/async-arrow-function/early-errors-arrow-formals-contains-super-call.js`; and
- `language/expressions/async-arrow-function/early-errors-arrow-formals-contains-super-property.js`.

They add eight variants. Pinned Boa's async-arrow parser checks duplicate
parameters, Yield/Await containment, `ContainsUseStrict` and lexical-name
intersections, then returns the arrow AST; it has no condition-specific `super`
producer. The completed ScriptBody's `ContainsSymbol::Super` check therefore
owns these four files. Calling only the global-code `4/8` set
producer-complete would be false.

Verification must invoke all eight complete suite-relative paths, require an
exact discovery total of sixteen, compare the exact completed execution-ID
set, and inspect every failure and non-success bucket.

Excluded adjacent Test262 families include direct/indirect eval (T13), Module
top-level `super` (the existing Module code), ordinary/async/generator function
and expression restrictions (their callable producers), class constructor,
field, method and static-block restrictions (their class producers), escaped
keyword syntax, private-name syntax and assignment/optional-chain grammar
errors.

The current-pin inventory contains no additional class-heritage or
computed-name parse-negative leaves that reach this ScriptBody producer. The
adjacent `language/module-code/early-super.js` leaf is rejected earlier by
member-expression parsing, so it contributes zero ModuleItemList-owned
variants and is not evidence for `ModuleTopLevelSuper`.

## Verification and nonclaims

Under the shared eight-core, 22 GB cap, the focused front group passes `4/4`,
the exact `Contains super` traversal guard passes `1/1`, and the complete
`lila-front` library passes `119/119`. The complete relevant IR groups pass
`44/44` for `modules::early::tests` and `41/41` for
`modules::graph::tests`; their focused classifier, rejected-graph and
method-owned graph witnesses each pass `1/1`. All eight Test262 files were run
separately and produced exactly `16/16` passing Wasm-AOT variants, with every
failure and non-success bucket at zero under `--jobs 1 --threads 1`.

A broader serialized `cargo test -p lila-ir` checkpoint was also attempted and
was stopped once two unrelated lowering tests were red. Exact reruns confirmed
the repeatable current-working-tree failures in
`boolean_method_fold_and_shape_are_invalidated_through_binding_aliases` and
`dynamic_json_parse_observes_reviver_holder_kinds`; neither exercises the
early-error or module-graph paths changed here. They remain explicit broad-suite
debt, so this lane does not claim the complete `lila-ir` crate is green.

The updated shared producer census was re-verified in the complete `129/129`
front suite and the `47/47` IR early plus `45/45` graph groups on 2026-08-24.
The subsequent GeneratorExpression checkpoint leaves it at `142/142` front
tests, `50/50` relevant IR early tests and `51/51` graph tests.

The lane classifies a rejection pinned Boa already produces. It does not:

- add or broaden `super` syntax;
- classify the remaining generic generator/async declarations or async-
  generator expression;
- classify the eleven `invalid super call usage` producers;
- repair token-level source location;
- support direct eval or Function-family dynamic source;
- repair pinned Boa's omitted public auto-accessor computed-name traversal;
- change valid constructor, method, field or static-block execution;
- refresh aggregate conformance status; or
- complete T07.
