# `import.meta` outside Module early errors

**Status:** Theory, integrated implementation and capped focused verification
complete, 2026-08-23; this bounded diagnostic lane is focused-verified

## Decision

An `ImportMeta` production parsed with any syntactic goal other than Module is
one closed pre-evaluation condition:

`EarlyErrorCode::ImportMetaOutsideModule`

Its sole wire spelling is `E_IMPORT_META_OUTSIDE_MODULE`. The code names the
specification's goal condition, not merely the top-level Script spelling that
reaches it in the product today. In particular, lexical nesting inside a
function, arrow, method, field initializer, or class static block does not turn
a Script parse into a Module parse.

## Specification and goal boundary

The edition-pinned ECMA-262 2026
[13.3.1.1, Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-left-hand-side-expressions-static-semantics-early-errors)
rule for

```text
ImportMeta : import . meta
```

requires a `SyntaxError` when the syntactic goal symbol is not Module. Clause
17 makes the listed condition an early error. Strictness is irrelevant; the
property belongs to the parse goal.

Lila's static product parser exposes exactly the closed `Script | Module` goal
choice. Under Script, every source occurrence of `import.meta` is therefore an
early error, including occurrences lexically nested in ordinary, generator,
async, arrow, method, field, and static-block bodies. Under Module, the same
forms are syntactically valid, including occurrences nested inside functions;
the Module goal is not replaced by a FunctionBody goal while parsing nested
source text.

Direct eval, indirect eval, `Function`, `GeneratorFunction`,
`AsyncFunction`, and `AsyncGeneratorFunction` constructors parse newly created
source text under their own specification goals. Those dynamic-source paths are
the explicit T13 / Wasm-AOT unsupported boundary and are not producers for this
static-source lane. This contract neither makes those APIs AOT-compilable nor
changes their expected `SyntaxError` semantics.

## Measured pinned-Boa boundary

Pinned `boa_parser-0.21.1` has exactly one Rust source occurrence of the fixed,
case-sensitive raw message:

```text
invalid `import.meta` expression outside a module
```

The sole producer is
`vendor/boa_parser-0.21.1/src/parser/expression/left_hand_side/member.rs`, in
`MemberExpression::parse`. The parser records `position` from the initial
`import` token, consumes the exact `import . meta` production, rejects escaped
`meta` through a separate earlier message, and then applies this branch:

```text
if !cursor.module() {
    return Err(Error::general(
        "invalid `import.meta` expression outside a module",
        position,
    ));
}
```

`Error::General` appends the source position, so the stable classifier prefix
is exactly:

```text
invalid `import.meta` expression outside a module at line
```

The prefix admits the parser-owned line and column without admitting the same
phrase later inside another diagnostic. It must be a
`ParseFailurePattern::StartsWith` row, not a `ContainsAll` row.

The source path is statically closed and currently reachable. A fresh Boa lexer
cursor starts with `module: false`; Lila's exhaustive `ParseGoal::Script` arm
calls `parse_script`, which does not change it. Lila's `ParseGoal::Module` arm
calls `parse_module`; `ModuleParser` sets module mode before parsing the
`ModuleItemList`. Pinned Boa has one `set_module()` call and no nested-parser
reset, so Module mode remains true through nested source constructs.

No vendor repair is required. Before this extension, the message matches no
row in `lila-front`'s one parse-failure table. The reachable Script rejection
therefore has `ParseCode::Malformed`, wire spelling `P_PARSE_MALFORMED`, phase
`Parse`, and native `SyntaxError`. The extension changes that result to
`ParseCode::Early(ImportMetaOutsideModule)`, phase `Early`, and native
`SyntaxError`, preserving a nonempty span at the `import` token.

## Exact typed encoding

The pre-extension closed domain has 59 variants and the parse-failure table has
58 rows. The extension grows those array-typed counts to 60 and 59.

The classifier addition is exactly one code and one row:

```text
ImportMetaOutsideModule => "E_IMPORT_META_OUTSIDE_MODULE";

const IMPORT_META_OUTSIDE_MODULE_PREFIX: &str =
    "invalid `import.meta` expression outside a module at line";

ParseFailureRule {
    pattern: ParseFailurePattern::StartsWith(IMPORT_META_OUTSIDE_MODULE_PREFIX),
    code: EarlyErrorCode::ImportMetaOutsideModule,
    witnesses: &[
        "invalid `import.meta` expression outside a module at line 1, col 1",
    ],
}
```

The row must be accompanied by:

- an evaluated `ParseClassified::from_parse_table` const assertion, so deleting
  the row while retaining the enum variant fails to build;
- an exact-single-owner const assertion that independently spells the complete
  reviewed prefix and requires the sole owner to use `StartsWith`;
- the existing table-wide witness-disjointness and interpolation-safety
  assertions; and
- an explicit arm in `lila-ir`'s exhaustive `EarlyErrorCode` mapping to
  `IrDiagnosticKind::EarlyError`, with no catch-all.

The new prefix does not overlap an existing classifier witness. Anchoring is
still required because Boa diagnostics can interpolate user-controlled token,
binding, or export-name text. A diagnostic that contains the phrase only after
its own parser-owned prefix must retain its existing owner or remain
unclassified.

## Direct source and precedence matrix

The permanent front-end matrix must establish the following rejection rows.
Every row reports the new code, phase `Early`, native `SyntaxError`, and a
nonempty source span:

| Goal | Boundary | Source |
| --- | --- | --- |
| Script | direct, sloppy | `import.meta;` |
| Script | direct, strict | `"use strict"; import.meta;` |
| Script | ordinary-function nesting | `function f() { return import.meta; }` |
| Script | generator-function nesting | `function* f() { return import.meta; }` |
| Script | async-function nesting | `async function f() { return import.meta; }` |
| Script | arrow nesting | `const f = () => import.meta;` |
| Script | class-method nesting | `class C { m() { return import.meta; } }` |
| Script | instance-field nesting | `class C { field = import.meta; }` |
| Script | static-field nesting | `class C { static field = import.meta; }` |
| Script | static-block nesting | `class C { static { void import.meta; } }` |
| Script | multiline source-position witness | `\n  import.meta;` |

The same direct, ordinary-function, generator-function, async-function, arrow,
class-method, field and static-block forms must parse under Module goal. At
minimum, the positive Module matrix includes:

```text
import.meta;
function f() { return import.meta; }
```

The following adjacent forms must not acquire the new code:

- Script `import("./dep.mjs")` is an `ImportCall`, not `ImportMeta`.
- An escaped `import` keyword remains owned by Boa's distinct
  `keyword must not contain escaped characters` rejection.
- `import.m\u0065ta` remains owned by the distinct
  `` `import.meta` cannot contain escaped characters`` rejection.
- `import.foo` fails the `meta` token expectation before the goal check.
- Assignment-target errors such as `import.meta = value` parse `ImportMeta`
  under Module first and belong to the existing assignment-target condition,
  not this goal condition.

One mixed-error witness pins the existing parser order without claiming that
ECMA-262 makes simultaneous early errors observably ordered:

```text
import.meta; let x; let x;
```

Under Script, the member-expression producer rejects while parsing
`import.meta`, before whole-Script declaration analysis, and must report
`ImportMetaOutsideModule`. Under Module, `import.meta` is valid and the later
module declaration analysis must retain `DuplicateLexicalDeclaration`.

A message-level injection witness must also place the full reviewed prefix
inside a user-controlled Module export name and prove that the new anchored row
does not select it. If the surrounding diagnostic already has a classified
owner, that owner must remain unchanged.

## IR and retained-graph projection

The enum extension must make `lila-ir`'s rejection-kind match exhaustive and
map the new code to `EarlyError`. The existing classifier-to-IR projection
matrix should include the complete rendered witness and derive phase `Early`
and native `SyntaxError` from the code.

This condition is honestly Script-only on Lila's static product path. Loaded
dependencies are always parsed as Module, so a real rejected
`ModuleSourceIr` carrying `ImportMetaOutsideModule` cannot exist. Tests must not
construct one and present it as retained-path evidence.

The graph invariant is instead positive and goal-preserving: retain a loaded
Module dependency containing direct and nested `import.meta` through
`ModuleSourceIr` and `build_graph`, and require successful Module parsing with
no rejection carrying the new code. Where the current graph API exposes the
module record's `import_meta_sites`, the direct occurrence should remain
recorded; otherwise graph success plus retention of the parsed Module is the
minimum honest witness. This complements, rather than replaces, the direct
Script rejection and exhaustive IR mapping.

## Vendored-source structural guard

A durable source guard must prove more than one local string count:

- recursively inventory every Rust file in pinned `boa_parser-0.21.1` and
  require exactly one occurrence of the complete raw message;
- require that occurrence to remain in
  `parser/expression/left_hand_side/member.rs`;
- tie it to one bounded snippet containing the exact `if !cursor.module()`
  branch, `Error::general`, the complete literal, and `position`;
- tie `position` to the initial `import` token's span rather than accepting an
  unrelated coordinate;
- retain the separate escaped-`meta` check before the goal branch;
- require the lexer cursor's initial `module: false` state and the sole
  `ModuleParser` transition to module mode; and
- reject any alternative producer or direct goal projection that could bypass
  the reviewed branch.

The classifier's evaluated parse-owner and exact-prefix const assertions are
the typed half of this guard. The source guard owns drift in the vendored
implementation; neither one substitutes for the other.

## Exact pinned Test262 cohort

At the harness-declared pinned Test262 tree
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, the exact static-source negative
cohort for this lane is one physical file:

- `language/expressions/import.meta/syntax/goal-script.js`

It declares `phase: parse`, `type: SyntaxError`, and no execution-mode flag, so
the harness expands it to exactly two sloppy/strict variants. The CLI filter is
path-prefix oriented; focused verification must pass the complete
suite-relative path and require exactly one discovered physical file, rather
than using `goal-script.js` as a basename filter.

The exact positive goal controls are:

- `language/expressions/import.meta/syntax/goal-module.js`
- `language/expressions/import.meta/syntax/goal-module-nested-function.js`

Both declare the Module flag. They prove the goal boundary but are not members
of the negative classification cohort.

The same Test262 directory also contains runtime tests for direct eval and the
four dynamic function-constructor families. They exercise the same
specification sentence through newly parsed source strings, but they are not
static parser-producer witnesses for this lane:

- `language/expressions/import.meta/not-accessible-from-direct-eval.js`
- `language/expressions/import.meta/syntax/goal-function-params-or-body.js`
- `language/expressions/import.meta/syntax/goal-generator-params-or-body.js`
- `language/expressions/import.meta/syntax/goal-async-function-params-or-body.js`
- `language/expressions/import.meta/syntax/goal-async-generator-params-or-body.js`

Escaped-spelling and invalid-assignment-target files in the directory likewise
belong to different parser conditions and are excluded.

## Focused verification — 2026-08-23

The focused `lila-front` `import_meta_` group passed `5/5`, followed by the
complete `lila-front --lib` gate at `108/108`. The exact `lila-ir` classifier
projection passed `1/1`, and the exact retained positive Module graph witness
passed `1/1`. `cargo xc`, `cargo fmt --all -- --check`, and
`git diff --check` were green.

The three complete pinned Test262 paths passed their exact Wasm-AOT expansion:

- `language/expressions/import.meta/syntax/goal-script.js`: `2/2`
  sloppy/strict variants;
- `language/expressions/import.meta/syntax/goal-module.js`: `1/1` Module
  variant; and
- `language/expressions/import.meta/syntax/goal-module-nested-function.js`:
  `1/1` Module variant.

Every failure and non-success bucket was zero. During focused verification, the
vendored-source guard was aligned to Boa's actual derived `ModuleParser`
declaration and parser signature. That was a source-guard accuracy repair, not
a production parser behavior change.

This evidence closes only the typed static-source diagnostic and its positive
Module goal boundary. It is not an aggregate refresh, a broad-suite gain, or a
measured new Test262 pass.

## Diagnostic-only nature

This lane is expected to produce no new Test262 pass. The existing unclassified
Boa rejection is `ParseCode::Malformed`, whose derived phase/type pair is
`Parse` / `SyntaxError`; the harness already accepts that pair for a Test262
`phase: parse`, `type: SyntaxError` negative. The material change is the honest
closed condition identity and `Early` projection. Focused verification must
therefore report exact diagnostic evidence without claiming a measured pass
gain.

## Rejected adjacent lanes

At this contract's inventory point, class-static-block `SuperCall` shared
`invalid super usage` with eleven other pinned-Boa producers, so classifying
that text as one static-block code would have been false. The later
`class-super-call-early-errors.md` and
`class-field-initializer-super-call-early-errors.md` lanes supplied the required
condition-specific producer wording and taxonomy.

The formal-parameter/function-body lexical-name intersection has one shared
message constructor in `parser/mod.rs`, but thirteen parser call sites across
callable forms. Its message interpolates the binding name, and the current
classifier folds it into the already broad
`EarlyErrorCode::DuplicateLexicalDeclaration`. Splitting it is valid future
work, but it requires a larger callable matrix and a deliberate rewrite of the
existing taxonomy owner. It is less bounded than the fixed, unique ImportMeta
producer.

## Nonclaims and review points

This lane does not change vendored Boa, implement `import.meta` object creation
or host finalization, change Module rewriting/lowering, support dynamic source
generation, classify escaped spellings or assignment-target errors, close the
remaining `super` or duplicate-lexical families, close T07, refresh an
aggregate, or claim a Test262 pass gain. Its enum, classifier, direct goal
matrix, IR map, positive graph witness and source guard are integrated, but no
aggregate, broad-suite or measured new-pass result is claimed by the focused
verification above.

Implementation review should confirm two deliberately explicit choices:

- `ImportMetaOutsideModule` is the code name because it follows the spec goal
  condition and remains correct if another non-Module static goal is exposed;
  `ScriptImportMeta` would describe only today's product reachability.
- Retained graph evidence is positive Module-goal evidence. A rejected
  dependency carrying this code would contradict the parser-goal invariant and
  must not be manufactured for symmetry with both-goal early-error lanes.

The two-variant negative Test262 count is derived from the pinned file's absence
of a mode flag and the harness's established sloppy/strict expansion. The exact
capped verification results are recorded above; they do not close dynamic
source parsing, runtime `import.meta`, T07, or aggregate conformance.
