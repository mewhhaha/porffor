# Class constructor and static-block `super()` early errors

**Status:** Product conditions and GeneratorExpression-updated shared producer
census focused-verified 2026-08-24

## Decision

Two class-owned `super()` restrictions are two distinct pre-evaluation
conditions. They must not be merged merely because pinned Boa formerly emitted
the same raw text for both:

| Specification condition | Code | Sole wire spelling |
| --- | --- | --- |
| A class has no `ClassHeritage`, has a constructor, and `HasDirectSuper` of that constructor is true | `EarlyErrorCode::ClassBaseConstructorHasDirectSuper` | `E_CLASS_BASE_CONSTRUCTOR_HAS_DIRECT_SUPER` |
| A class static block's statement list `Contains SuperCall` | `EarlyErrorCode::ClassStaticBlockContainsSuperCall` | `E_CLASS_STATIC_BLOCK_CONTAINS_SUPER_CALL` |

Both codes derive phase `Early` and native error type `SyntaxError`. A rejected
class never reaches IR lowering, class-definition evaluation, constructor
execution or static-block execution.

“Base” in `ClassBaseConstructorHasDirectSuper` means exactly that the
`ClassHeritage` grammar production is absent. A class written with
`extends null` has heritage and is therefore outside that early-error
condition, even though calling its `super` constructor can later fail at run
time.

The static-block condition is independent of heritage. It applies equally to
base and derived classes. It rejects `SuperCall`, not `SuperProperty`:
`super.value` remains valid static-block syntax.

## Edition-pinned specification boundary

The source of truth is ECMA-262 2026
[15.7.1, Class Definitions — Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-class-definitions-static-semantics-early-errors).

For

```text
ClassTail : ClassHeritageopt { ClassBody }
```

the specification rejects when `ClassHeritage` is absent and this algorithm
returns true:

1. obtain the `ConstructorMethod` of `ClassBody`;
2. return false when there is no constructor; and
3. otherwise return `HasDirectSuper` of that constructor.

The edition-pinned
[15.4.2 `HasDirectSuper`](https://tc39.es/ecma262/2026/multipage/ecmascript-language-functions-and-classes.html#sec-static-semantics-hasdirectsuper)
operation examines the constructor's parameter list and body for `SuperCall`.
It follows ordinary and async arrows because they preserve the surrounding
`super` binding, but it does not turn a nested non-arrow callable or a nested
class constructor into part of the outer constructor.

Separately, for

```text
ClassStaticBlockBody : ClassStaticBlockStatementList
```

15.7.1 rejects when `ClassStaticBlockStatementList Contains SuperCall` is
true. The edition-pinned
[`Contains` operation](https://tc39.es/ecma262/2026/multipage/syntax-directed-operations.html#sec-static-semantics-contains)
again follows lexical arrows and stops at nested non-arrow callable bodies.
General static semantics do not descend into nested class bodies or nested
static blocks, apart from the specific computed-property traversal selected by
the specification.

These class-owned rules are deliberately distinct from the whole-source rules
in ECMA-262 2026
[16.1.1 for Script](https://tc39.es/ecma262/2026/multipage/ecmascript-language-scripts-and-modules.html#sec-scripts-static-semantics-early-errors)
and
[16.2.1.1 for Module](https://tc39.es/ecma262/2026/multipage/ecmascript-language-scripts-and-modules.html#sec-module-semantics-static-semantics-early-errors).
They apply `Contains super` to a Script body or Module item list,
respectively. Those operations stop at class-owned bodies, allowing the two
conditions in this contract to report from their precise class producers
rather than being absorbed by `ScriptTopLevelSuper` or
`ModuleTopLevelSuper`.

Direct eval remains a separate T13 boundary. Its `super` legality depends on
the calling environment and cannot be represented by either source-static
class code.

## Measured pinned-Boa producer boundary

Pinned `boa_parser-0.21.1` implements both predicates in
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs`.
The predicates were already present and semantically separate. This lane
changed only their formerly shared raw diagnostic text.

### Base constructor producer

`ClassTail::parse` first parses the complete class body and obtains its optional
constructor. After the body closes, it applies exactly this conjunction:

```text
super_ref.is_none()
    && let Some(constructor) = &constructor
    && contains(constructor, ContainsSymbol::SuperCall)
```

Before the repair, the branch emitted:

```text
invalid super usage
```

at `body_start`. The implementation changed only that branch's raw message
to the exact unique text:

```text
base class constructor cannot contain direct super call
```

Boa's `Error::Lex(LexError::Syntax(...))` display appends the source position,
so the classifier owns the complete fixed prefix:

```text
base class constructor cannot contain direct super call at line
```

The predicate, position source and post-`ClassBody` placement remain unchanged.
This is a diagnostic-identity repair, not a grammar repair.

### Static-block producer

`ClassBody::parse` parses a static block's complete statement list and then
applies:

```text
if contains(&statement_list, ContainsSymbol::SuperCall) {
    return Err(Error::general("invalid super usage", position));
}
```

The implementation changed only this branch's raw message to the exact
unique text:

```text
class static block cannot contain super call
```

`Error::General` appends the source position, so the classifier owns the
complete fixed prefix:

```text
class static block cannot contain super call at line
```

The branch must remain after the adjacent `ContainsArguments` check and before
the adjacent `Contains AwaitExpression` and invalid-object-literal checks.

### Complete raw-message census

Before this repair, the literal `invalid super usage` occurs exactly twelve
times across every Rust source in pinned `boa_parser-0.21.1`:

| Parser owner | Source relative to `src/parser/` | Current occurrences | Position source |
| --- | --- | ---: | --- |
| `ScriptBody` top-level `Contains super` | `mod.rs:445` | 1 | fixed `Position::new(1, 1)` |
| shared hoistable declaration parser | `statement/declaration/hoistable/mod.rs:236` | 1 | `params_start_position` |
| ordinary function expression | `expression/primary/function_expression/mod.rs:151` | 1 | parameter-start position |
| generator expression | `expression/primary/generator_expression/mod.rs:167` | 1 | parameter-start position |
| async function expression | `expression/primary/async_function_expression/mod.rs:164` | 1 | parameter-start position |
| async-generator expression | `expression/primary/async_generator_expression/mod.rs:188` | 1 | parameter-start position |
| base-constructor/no-heritage check | `statement/declaration/hoistable/class_decl/mod.rs:208` | 1 | `body_start` |
| private, private-static, grouped public/accessor/static and static-accessor field-initializer checks | `statement/declaration/hoistable/class_decl/mod.rs:440,459,480,490` | 4 | class-element `position` |
| class static-block check | `statement/declaration/hoistable/class_decl/mod.rs:756` | 1 | static-block `position` |
| **Total** | | **12** | |

Immediately after both message-only repairs, the complete census was:

| Raw literal | Required occurrences | Owners |
| --- | ---: | --- |
| `invalid super usage` | 10 | ScriptBody, five callable producers and four class-field producers |
| `base class constructor cannot contain direct super call` | 1 | the no-heritage constructor conjunction only |
| `class static block cannot contain super call` | 1 | the static-block `Contains SuperCall` branch only |

Within `class_decl/mod.rs`, the old raw literal count falls from six to four;
the four survivors are precisely the field-initializer `SuperCall` checks.
The eleven separate `invalid super call usage` producers for method
`HasDirectSuper` remain untouched.

The later, separately owned class-field initializer lane gives those four
field producers their own message. The subsequent FunctionExpression and
FunctionDeclaration lanes also give the two ordinary function productions
their own messages. The AsyncFunctionExpression and GeneratorExpression lanes
give two more function productions their own messages. On current head,
`invalid super usage` occurs three times: the ScriptBody producer, the shared
default for the three remaining hoistable forms and the async-generator-
expression producer. The field message occurs four times and is owned by
`ClassFieldInitializerContainsSuperCall`; the four function messages occur
once each and are separately owned. See
`class-field-initializer-super-call-early-errors.md` and
`function-expression-contains-super-early-errors.md` plus
`function-declaration-contains-super-early-errors.md` and
`async-function-expression-contains-super-early-errors.md` plus
`generator-expression-contains-super-early-errors.md`. The Script producer
remains byte-for-byte unchanged and continues to be selected only by the exact
rendered message `invalid super usage at line 1, col 1`.

No classifier row may match the broad text `invalid super usage at line`.
Doing so would still merge callable and Script conditions; the class-field
condition now has its own anchored prefix for the same reason.

## Exact typed encoding

The implementation starts from 62 `EarlyErrorCode` variants and 62 parse-table
rows. Adding the two conditions grows both array-typed counts to 64.

The enum additions are exactly:

```text
ClassBaseConstructorHasDirectSuper
    => "E_CLASS_BASE_CONSTRUCTOR_HAS_DIRECT_SUPER";
ClassStaticBlockContainsSuperCall
    => "E_CLASS_STATIC_BLOCK_CONTAINS_SUPER_CALL";
```

The parse table receives two separately anchored rules:

```text
const CLASS_BASE_CONSTRUCTOR_DIRECT_SUPER_PREFIX: &str =
    "base class constructor cannot contain direct super call at line";
const CLASS_STATIC_BLOCK_SUPER_CALL_PREFIX: &str =
    "class static block cannot contain super call at line";

ParseFailureRule {
    pattern: ParseFailurePattern::StartsWith(
        CLASS_BASE_CONSTRUCTOR_DIRECT_SUPER_PREFIX,
    ),
    code: EarlyErrorCode::ClassBaseConstructorHasDirectSuper,
    witnesses: &[
        "base class constructor cannot contain direct super call at line 2, col 1",
    ],
}

ParseFailureRule {
    pattern: ParseFailurePattern::StartsWith(
        CLASS_STATIC_BLOCK_SUPER_CALL_PREFIX,
    ),
    code: EarlyErrorCode::ClassStaticBlockContainsSuperCall,
    witnesses: &[
        "class static block cannot contain super call at line 2, col 1",
    ],
}
```

These are real producer coordinates, not merely classifier-shaped strings. The
first is emitted for `class C {\nconstructor() { super(); }\n}` because
`body_start` is the constructor token; the second is emitted for
`class C { static {\nsuper();\n} }` because the static-block `position` is its
first statement token.

`StartsWith` is correct because the complete fixed producer-owned body ends at
`at line`, after which Boa appends only its decimal source coordinate. A bare
`ContainsAll` fragment is too broad: user-controlled text embedded later in a
different Boa diagnostic could forge a match. `Exact` is not correct because
both producers retain real, source-dependent coordinates.

Each code requires:

- an evaluated `ParseClassified::from_parse_table` const witness;
- an exact-single-owner const assertion that independently spells its complete
  prefix and requires the sole owning row to use `StartsWith`;
- the existing table-wide witness-disjointness, wire-name and parse-owner
  assertions;
- classifier witnesses proving the two new messages cannot select one
  another, `ScriptTopLevelSuper`, a callable `invalid super usage` message,
  the separately typed field message, or the method-owned
  `invalid super call usage` message;
- injection witnesses in which a user-controlled Module export name contains
  each complete new prefix but remains `ModuleDuplicateExport`; and
- explicit arms in `lila-ir`'s exhaustive `EarlyErrorCode` mapping to
  `IrDiagnosticKind::EarlyError`, with no catch-all.

The types must keep the two conditions separate. A union variant such as
`ClassInvalidSuperCall`, a shared message row, or a broad invalid-super
classifier is forbidden by this contract.

## Direct Script and Module matrix

Every rejection row below is parsed independently under both `ParseGoal::Script`
and `ParseGoal::Module`. Both goals must report phase `Early`, native
`SyntaxError`, the row's exact code and a nonempty source span. Neither goal may
report `ScriptTopLevelSuper`, `ModuleTopLevelSuper`, `Malformed`, or the other
new class code.

### `ClassBaseConstructorHasDirectSuper`

| Boundary | Exact source |
| --- | --- |
| class declaration, constructor body | `class C { constructor() { super(); } }` |
| class expression, constructor body | `(class { constructor() { super(); } });` |
| constructor parameter initializer | `class C { constructor(value = super()) {} }` |
| arrow in constructor body | `class C { constructor() { (() => super())(); } }` |
| arrow in constructor parameter | `class C { constructor(value = (() => super())()) {} }` |
| nested arrow traversal | `class C { constructor() { (() => () => super())()(); } }` |
| async-arrow traversal | `class C { constructor() { (async () => super())(); } }` |

The declaration/expression split proves that both `ClassDeclaration` and
`ClassExpression` feed the one `ClassTail` owner. Parameter and body rows prove
the two `HasDirectSuper` inputs, and the ordinary- and async-arrow rows prove
lexical traversal rather than token scanning.

### `ClassStaticBlockContainsSuperCall`

| Boundary | Exact source |
| --- | --- |
| class declaration | `class C { static { super(); } }` |
| class expression | `(class { static { super(); } });` |
| derived class, proving heritage is irrelevant | `class B {}; class C extends B { static { super(); } }` |
| nested block | `class C { static { { super(); } } }` |
| arrow body | `class C { static { (() => super())(); } }` |
| arrow parameter | `class C { static { ((value = super()) => value)(); } }` |
| nested arrow traversal | `class C { static { (() => () => super())()(); } }` |
| async-arrow traversal | `class C { static { (async () => super())(); } }` |

## Positive boundary matrix

Every row below must parse successfully under both Script and Module goals.
They control the exact negative predicates; they are not runtime-execution
claims.

| Boundary preserved | Exact source |
| --- | --- |
| absent constructor | `class C {}` |
| derived constructor direct call | `class B {}; class C extends B { constructor() { super(); } }` |
| `extends null` still has heritage | `class C extends null { constructor() { super(); } }` |
| base constructor `SuperProperty`, not `SuperCall` | `class C { constructor() { void super.value; } }` |
| nested derived constructor is not part of the outer base constructor | `class C { constructor() { class D extends C { constructor() { super(); } } } }` |
| empty static block | `class C { static {} }` |
| static-block `SuperProperty`, not `SuperCall` | `class B {}; class C extends B { static { void super.value; } }` |
| nested derived constructor is not part of the outer static block | `class C { static { class D extends C { constructor() { super(); } } } }` |
| string text is not syntax | `class C { static { const text = "super()"; } }` |

A nested ordinary function containing `super()` is not a positive traversal
control: that function has its own early error. The nested-class witnesses are
the honest positive boundaries for a syntactic `SuperCall` owned elsewhere.

## Precedence matrix

Pinned Boa checks static-block conditions while parsing `ClassBody`, but checks
the no-heritage constructor condition only after the complete class body has
returned to `ClassTail`. The implementation must preserve that observable
order.

Every row is tested under Script and Module goals:

| Simultaneous conditions | Exact source | Required owner |
| --- | --- | --- |
| base constructor direct `super()` and later static-block `super()` | `class C { constructor() { super(); } static { super(); } }` | `ClassStaticBlockContainsSuperCall` |
| static-block `super()` before base constructor direct `super()` | `class C { static { super(); } constructor() { super(); } }` | `ClassStaticBlockContainsSuperCall` |
| static-block `ContainsArguments` and `SuperCall` | `class C { static { arguments; super(); } }` | existing `ClassStaticBlockContainsArguments` |
| static-block `SuperCall` and `ContainsAwait` | `class C { static { super(); await 0; } }` | `ClassStaticBlockContainsSuperCall` |
| duplicate constructors where one has direct `super()` | `class C { constructor() { super(); } constructor() {} }` | existing `DuplicateClassConstructor` |

The first two rows prove that source element order does not move the deferred
ClassTail check ahead of a ClassBody rejection. The next two pin the existing
static-block check order: arguments, then `SuperCall`, then await. The final row
pins duplicate-constructor detection inside `ClassBody` before the deferred
base-constructor check.

Classifier-only precedence witnesses additionally require:

- `invalid super usage at line 1, col 1` remains `ScriptTopLevelSuper`;
- `invalid super call usage at line 1, col 1` matches neither new code;
- each new complete prefix matches only its own code; and
- an interpolated duplicate-export diagnostic containing either prefix remains
  `ModuleDuplicateExport`.

## IR and retained-module boundary

The public front-end result remains the sole parse owner. `lila-ir` must only
project the retained structured rejection; it must not inspect Boa messages or
reparse source.

For each code, focused IR witnesses must establish:

1. a real Module parse failure passed through
   `module_parse_failure_diagnostic` becomes
   `IrDiagnosticKind::EarlyError`, phase `Early`, native `SyntaxError`, the
   identical `EarlyErrorCode`, and a nonempty span;
2. a dependency containing the failing source is retained by `build_graph` as
   `ModuleSourceIr::Rejected` with that same diagnostic;
3. graph construction does not attempt lowering, request discovery or module
   record construction for the rejected node; and
4. positive dependency modules containing a valid derived constructor or
   `SuperProperty` static block remain parsed graph nodes.

An exhaustive `match` in `crates/lila-ir/src/early_error_code.rs` owns the two
new variants. No string table may be added to `lila-ir`, and no test may mint a
diagnostic without exercising a real front-end producer.

## Durable structural guards

The implementation extends one vendored-source guard that recursively
inventories the pinned Boa packages and proves all of the following:

- the current raw-message census is exactly `3 + 1 + 1 + 4 + 1 + 1 + 1 + 1`: three generic,
  one base-constructor, one static-block, four separately typed field messages
  and one message for each of the four typed function productions;
- the two new raw messages each occur exactly once, both in
  `class_decl/mod.rs`;
- the old raw `invalid super usage` no longer occurs in `class_decl/mod.rs`;
- the separately typed field message occurs exactly four times in its four
  reviewed initializer branches;
- the ordinary function expression/declaration, async-function-expression and
  generator-expression messages occur once each and remain attached to their
  distinct completed-node/shared-predicate owners;
- the base-constructor message is dominated by the complete three-part
  `super_ref.is_none` / optional-constructor / `ContainsSymbol::SuperCall`
  conjunction, retains `body_start`, and remains after the complete
  `ClassBody::parse` call;
- the static-block message is dominated by exactly
  `contains(&statement_list, ContainsSymbol::SuperCall)`, retains the
  static-block `position`, follows `contains_arguments`, and precedes the
  `AwaitExpression` and invalid-object-literal checks;
- the Script producer remains the sole fixed
  `Error::general("invalid super usage", Position::new(1, 1))` occurrence and
  remains inside the direct-eval exclusion;
- the eleven `invalid super call usage` method producers remain untouched;
- Boa AST `Contains` traverses ordinary and async arrows for `SuperCall`, stops
  at ordinary/generator/async/async-generator callable bodies, and does not
  descend into nested class bodies or nested static blocks beyond the
  specification's computed-property boundary; and
- the product parse prefix retains one Script parse, one Module parse and one
  classifier call, with no second parser or classifier owner in another crate.

Literal counts alone are insufficient. The guard must pin the surrounding
predicates and relative check order so moving either unique message to a
different branch fails even if the total counts remain unchanged.

The existing Script-top-level-super source guard must be revised in the same
implementation patch rather than weakened or deleted. Its exact fixed-message
and traversal assertions remain valid; only the explicitly reviewed producer
census changes.

## Complete current-pin Test262 cohort

At the repository's declared Test262 revision
`e9d582d6b8b13afc5ba9a676664741592b5c7f69`, the complete cohort for these two
producers contains three physical files:

### Base constructor without heritage: two files

- `language/expressions/class/elements/syntax/early-errors/grammar-ctor-super-no-heritage.js`
- `language/statements/class/elements/syntax/early-errors/grammar-ctor-super-no-heritage.js`

Both are generated parse-negative `SyntaxError` tests. Each expands to sloppy
and strict Script execution, for four variants.

### Static block: one file

- `language/statements/class/static-init-invalid-super-call.js`

It is an unflagged parse-negative `SyntaxError` test and expands to sloppy and
strict Script execution, for two variants.

None of the three files declares `onlyStrict`, `noStrict`, `raw` or `module`.
The complete cohort is therefore exactly three physical files and six
Wasm-AOT execution IDs. Verification must run all three exact suite-relative
paths, require discovery of exactly `6/6`, compare the exact completed-ID set,
and require every parser, early-error, lowering, runtime, Wasm-backend,
host-harness, unsupported, crash and bug bucket to be zero.

The direct Module and retained-graph witnesses are permanent Rust tests because
the pinned cohort contains no Module-goal leaf for either producer.

## Verification evidence

Implementation was batched before verification. Expensive commands ran
serially and did not overlap.

The verifier ran this ladder after the theory, vendor wording, typed codes,
classifier rows, IR arms, source guards, direct matrices, retained-graph
witnesses and task status were written:

1. read-only/adversarial source review of both predicates, all twelve old raw
   producers, the post-repair census, `Contains` traversal and classifier
   disjointness;
2. `cargo fmt --all -- --check` and `git diff --check`;
3. one capped `cargo xc` compile checkpoint;
4. focused `lila-front` exact tests for the two source matrices, classifier
   ownership/injection assertions and vendored-source guard;
5. the complete capped `lila-front --lib` gate;
6. focused real-Module diagnostic witnesses, then the complete relevant
   `lila-ir` `modules::early::tests` group;
7. focused retained rejected/positive dependency witnesses, then the complete
   relevant `lila-ir` `modules::graph::tests` group; and
8. the three exact Test262 paths with `--jobs 1`, `--threads 1` and an explicit
   timeout, requiring exactly six completed passing variants.

If a broader `lila-ir` or workspace suite is attempted afterward, report its
result separately from the green affected groups. Unrelated failures cannot be
hidden, but they also do not erase exact evidence for these parser/graph paths.
No full language tree or aggregate Test262 refresh is required for this bounded
diagnostic lane.

`cargo fmt --all -- --check`, `cargo xc` and `git diff --check` are green. The
complete front library passes `129/129`; the relevant IR early and graph groups
pass `47/47` and `45/45`. Each of the three exact Test262 files passes `2/2`,
for an aggregate `6/6` Wasm-AOT variants with every non-success bucket at zero.

The subsequent GeneratorExpression checkpoint leaves the verified shared
census at `142/142` front tests, `50/50` relevant IR early tests and `51/51`
graph tests.

## Explicit nonclaims

This contract does not:

- classify the eleven method-owned `invalid super call usage` producers;
- classify the remaining generic generator/async declarations or async-
  generator expression;
- own the separately implemented class-field-initializer `SuperCall`
  condition;
- broaden or merge `ScriptTopLevelSuper` or `ModuleTopLevelSuper`;
- implement direct eval, indirect eval, `Function` constructors or any dynamic
  source path;
- change `Contains`, `HasDirectSuper`, class grammar or accepted syntax;
- implement or validate constructor execution, `this` initialization,
  repeated `super()` calls, `extends null` invocation behavior, static-block
  execution or class-element lowering;
- improve token-level source locations beyond preserving Boa's current
  producer positions;
- prove a new Test262 pass, because the same negative files may already pass
  observably through the generic malformed-parse bucket;
- refresh README aggregate counts or claim a broad conformance gain; or
- close T07, the class grammar bucket, all `super` early errors, or aggregate
  ECMAScript/Test262 conformance.
