# Contract: early-error taxonomy — one closed `EarlyErrorCode` domain and one fragment table

## 2026-08-20 normative remaining class-constructor restriction amendment

Four adjacent ClassElement early-error conditions are now distinct closed
codes: a non-static async method, getter or setter whose literal name is
`constructor`, and any private ClassElement named `#constructor`. Pinned
`boa_parser-0.21.1` emits four complete case-sensitive literals: the async,
getter and setter messages each have one producer, while the private-name
message has seven producers spanning private fields and method forms.

T07 therefore adds `ClassConstructorAsyncMethod`,
`ClassConstructorGetter`, `ClassConstructorSetter` and
`ClassPrivateConstructorName`, each with one exact classifier row. The
exhaustive `lila-ir` mapping derives `Early`, `SyntaxError` for all four. The
domain now has **28** variants and the one parse-failure table has **26** rows.

Front-end regressions cover declaration and expression sources under both
parse goals, retained Module diagnostics cover each code, and positive sources
preserve static and computed public names. The full theory, producer inventory,
18-case pinned Test262 matrix and nonclaims live in
`docs/rust-rewrite/contracts/class-constructor-nonordinary-method-early-errors.md`
and are normative.

This amendment classifies rejections Boa already produces. It does not
implement async/class/private-element execution, close the class parser bucket
or complete T07. Cargo, focused execution and pinned Test262 verification
remain deferred to the shared verification lane.

## 2026-08-20 normative class-constructor generator-method amendment

ClassElement early errors reject a non-static generator or async-generator
method whose literal property name is `"constructor"`. Pinned
`boa_parser-0.21.1` emits the same complete, case-sensitive literal from the two
corresponding branches:

`class constructor may not be a generator method`

T07 therefore extends the domain with
`EarlyErrorCode::ClassConstructorGeneratorMethod` /
`E_CLASS_CONSTRUCTOR_GENERATOR_METHOD` and one classifier row whose fragment
and witness are that exact literal. The exhaustive `lila-ir` mapping derives
`Early`, `SyntaxError` for the condition. At that amendment's checkpoint the
domain had **24** variants and the one parse-failure table had **22** rows.

Front-end regressions reject generator and async-generator constructors in
declaration and expression forms under both goals. Positive witnesses preserve
static and computed generator methods named `constructor`; a real Module parse
also crosses the retained front-end-to-IR diagnostic boundary. The full theory,
producer inventory, source matrix, pinned Test262 evidence and nonclaims live
in
`docs/rust-rewrite/contracts/class-constructor-generator-method-early-errors.md`
and are normative.

This amendment preserves the parse-once path and adds no second classifier. At
that amendment's checkpoint it did not cover adjacent async/accessor/private
constructor diagnostics. It does not close the class parser bucket or complete
T07. Cargo, focused execution and pinned Test262 verification remain deferred
to the shared verification lane.

## 2026-08-17 normative class-static-block `ContainsArguments` amendment

The ClassStaticBlockBody early-error rules reject a
`ClassStaticBlockStatementList` whose `ContainsArguments` result is true. The
condition is decided before evaluation and reported as a `SyntaxError` under
both Script and Module goals. The traversal follows lexical `arguments` use
through arrows and evaluated class-element names, but stops at ordinary,
generator and async functions and at method parameters and bodies, where
`arguments` belongs to the nested callable.

Pinned `boa_parser-0.21.1` has exactly one producer and one case-sensitive
literal for this condition. At
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs:740-745`,
it applies `contains_arguments` to the parsed static-block statement list and
reports `'arguments' not allowed in class static block`. The full literal
occurs nowhere else in pinned Boa. The visitor's function and method traversal
boundaries are explicit in
`vendor/boa_ast-0.21.1/src/operations/mod.rs:350-452`.

T07 therefore extends the domain with
`EarlyErrorCode::ClassStaticBlockContainsArguments` /
`E_CLASS_STATIC_BLOCK_CONTAINS_ARGUMENTS` and exactly one classifier row whose
fragment and witness are that full literal. `lila-ir` maps the new variant in
its exhaustive `rejection_kind` match to `IrDiagnosticKind::EarlyError`; phase
and error type remain derived as `Early` and `SyntaxError`. The closed domain
now has **23** variants and the one parse-failure table has **21** rows.

Front-end regressions require declaration and expression forms to reject under
both goals. They include pinned Test262's escaped `argument\u0073`
computed-name source and lexical use through an arrow, and preserve ordinary
function and method parameters and bodies as positive traversal boundaries. A
real Module parse crosses the retained front-end-to-IR diagnostic boundary, and
the message-table regression separately fixes the exact literal-to-code map.

At pinned Test262 revision
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, the bounded evidence is
`language/statements/class/static-init-invalid-arguments.js` (`phase: parse`,
`type: SyntaxError`) plus the positive
`static-init-arguments-functions.js` and `static-init-arguments-methods.js`
files in the same directory. They specify the classification and traversal
boundary; they are not a current Wasm-AOT pass claim.

This amendment classifies the rejection Boa already produces. It does not
implement static-block parsing, lowering, initialization or execution; cover
the separate `'arguments' not allowed in class field definition` condition,
direct eval, or adjacent `await`, `yield`, `super()`, `return`, lexical or label
rules; close T07; or change a snapshot or published count. Cargo, focused
execution and current-pin Test262 verification remain deferred to the shared
verification lane.

## 2026-08-13 normative duplicate-class-constructor amendment

T07 extends the closed taxonomy with
`EarlyErrorCode::DuplicateClassConstructor` /
`E_DUPLICATE_CLASS_CONSTRUCTOR` and exactly one classifier row for the sole
case-sensitive message emitted by pinned `boa_parser-0.21.1`. At that
amendment's checkpoint the domain had 22 variants and the parse-failure table
had 20 rows.

The theory, exact producer, declaration/expression and Script/Module source
matrix, positive static/computed-method boundaries, and nonclaims live in
`docs/rust-rewrite/contracts/duplicate-class-constructor-early-errors.md` and
are normative. The counts in older amendments and the historical measurement
below remain descriptions of their respective encoder checkpoints.

## 2026-08-13 normative catch-body-declaration-conflict amendment

T07 extends the closed taxonomy with
`EarlyErrorCode::CatchBodyDeclarationConflict` /
`E_CATCH_BODY_DECLARATION_CONFLICT` and exactly one classifier row for the
single case-sensitive message emitted by two reachable branches in pinned
`boa_parser-0.21.1`. After that amendment the domain had 21 variants and the
parse-failure table had 19 rows.

The theory, exact producer inventory, union-shaped encoding, source
regressions, simple-identifier exception, and nonclaims live in
`docs/rust-rewrite/contracts/catch-body-declaration-conflict-early-errors.md`
and are normative. The counts in older amendments and the historical
measurement below remain descriptions of their respective encoder checkpoints.

## 2026-08-13 normative duplicate-catch-parameter amendment

T07 extends the closed taxonomy with
`EarlyErrorCode::DuplicateCatchParameter` /
`E_DUPLICATE_CATCH_PARAMETER` and exactly one classifier row for the sole
case-sensitive message emitted by pinned `boa_parser-0.21.1`. After that
amendment the domain had 20 variants and the parse-failure table had 18 rows.

The theory, measured producer, encoding, regressions, and deliberate separation
from catch-body conflicts live in
`docs/rust-rewrite/contracts/duplicate-catch-parameter-early-errors.md` and are
normative. The counts in older amendments and the historical measurement below
remain descriptions of their respective encoder checkpoints.

## 2026-08-13 normative duplicate-formal-parameter amendment

T07 extends the closed taxonomy with
`EarlyErrorCode::DuplicateFormalParameter` /
`E_DUPLICATE_FORMAL_PARAMETER` and exactly two case-sensitive classifier rows
for the two messages emitted by pinned `boa_parser-0.21.1`. After that amendment
the domain had 19 variants and the parse-failure table had 17 rows.

The theory, measured producer inventory, encoding, regressions, and nonclaims
for this extension live in
`docs/rust-rewrite/contracts/duplicate-formal-parameter-early-errors.md` and are
normative. All 18-variant and 15-row counts in the historical measurement below
remain descriptions of the original encoder checkpoint, not current counts.

## 2026-08-12 normative parse-once amendment

T07 removed the second parser path described by the original measurement
below. `lila-front` now owns the sole product parse and returns goal-typed
`ParsedScript` / `ParsedModule` values holding the AST and its exact interner.
Loaded dependencies retain that result; request discovery and module-record
construction borrow the same parser session. Consequently
`reparse_module`, `MODULE_REPARSE_PREFIX`, and assertion P6 no longer exist.
All later references to them describe the historical baseline and discrepancy
ledger only; they are not current encoder requirements. The closed
`EarlyErrorCode` domain, the single classifier, and the remaining assertions
stay normative.

Area: *Early-error taxonomy: one closed `EarlyErrorCode` domain and one fragment
table, not two that have already drifted*
Stage: FORMALIZER. This document is normative for the encoder and is the oracle
the dry-runner checks against. **No source code is edited in this stage.**

Method: theory first. The domain is closed by ECMA-262 clause 17 and the
per-production *Static Semantics: Early Errors* clauses; the Rust types are then
chosen so that each way of violating that closure is a named compile error. The
only empirical inputs are (a) the exact byte-strings `boa_parser`/`boa_ast`
produce, which are upstream and not derivable from the spec, and (b) the
test262 `negative:` frontmatter, which is the conformance-visible shadow of the
same closed set. Both were measured, not recalled; every count and every
`file:line` below was read.

---

## 0. Corrections to the area brief, measured

The brief is wrong or imprecise in seven places. Each correction is load-bearing
for the encoding, so they come first.

### 0.1 `EarlyErrorCode` **cannot** live in `crates/lila-ir` — it would be a dependency cycle

The brief's scope item (a) puts the enum in `crates/lila-ir/src/early_error_code.rs`
and has `lila_front::parser_static_semantics_error_code` consume it. Measured
from `Cargo.toml`:

```
lila-front  deps: boa_ast, boa_interner, boa_parser        (no lila-ir)
lila-ir     deps: ..., lila-front                        (lila-ir → lila-front)
lila-engine deps: ..., lila-front, lila-ir
lila-test262 deps: lila-engine, lila-front, lila-ir
```

`lila-ir` depends on `lila-front` (`crates/lila-ir/src/diagnostics.rs:1`
is `use lila_front::SourceSpan;`). A type in `lila-ir` therefore cannot be
named by `lila-front`.

Both crates independently call boa's parser and both independently need the
classification: `lila_front::parse` (`crates/lila-front/src/lib.rs:166-173`)
for the entry file, and `lila_ir::modules::record::reparse_module`
(`crates/lila-ir/src/modules/record.rs:1331-1351`) for every dependency
module. The one table must therefore sit in the lower crate.

**Decision D0.1.** `EarlyErrorCode`, the fragment table and the classifier live in
a new module **`crates/lila-front/src/early_error_code.rs`**.
`crates/lila-ir/src/early_error_code.rs` still exists and is still this lane's
exclusive module, but it holds only what needs `lila-ir` types: the
re-export and the single `EarlyErrorCode → IrDiagnosticKind` map (§2.2). There
is no second copy of any table.

### 0.2 The domain has **18** inhabitants, not 20

20 distinct `"E_..."` string literals were measured (51 tokens, 7 files, 4
crates — the brief's counts are exact and were reproduced). Two of the 20 are
not early-error codes:

- `"E_IR_DIAGNOSTIC"` (`crates/lila-test262/src/lib.rs:21523`) is the display
  placeholder used when `diagnostic.code` is `None`, i.e. for `Unsupported` and
  `Lowering` diagnostics. It names the *absence* of a code. Making it an
  inhabitant would let `code: Some(EarlyErrorCode::IrDiagnostic)` mean "no code".
- `"E_TEST_EARLY"` (`crates/lila-engine/src/lib.rs:4112`) is a fabricated code
  in one `#[test]`. That a unit test could mint a code that no producer emits is
  itself an instance of MC2.

`EarlyErrorCode` therefore has **18** variants. `E_IR_DIAGNOSTIC` survives as a
`&'static str` literal at exactly one display site (§6.10); `E_TEST_EARLY` is
deleted and the test names a real code (§6.9).

### 0.3 `E_OBJECT_DUPLICATE_PROTO` drift affects one path, not two

The brief says paths 2 (module entry) and 3 (module dependency) both fall through.
Measured: `Engine::compile_on_current_thread`
(`crates/lila-engine/src/lib.rs:1907-1924`) calls `lila_front::parse` on the
**entry** for *both* goals before any lowering. So a module entry file is
classified by front (`lib.rs:213-215`) and already reports
`E_OBJECT_DUPLICATE_PROTO`. Only the **dependency** path — where
`parse_module_record` → `reparse_module` → `module_parse_failure_diagnostic` is
the only classifier — falls through to `IrDiagnostic::unsupported`.

The drift is real and conformance-visible; it is one path wide, not two.

### 0.4 The `E_DUPLICATE_LEXICAL_DECLARATION` drift is worse than stated, and runs the other way too

Measured: `boa` produces **five** distinct wordings for this rule.

| # | verbatim message | producer |
|---|---|---|
| W1 | `lexical name declared multiple times` | `boa_parser/src/parser/mod.rs:366,376`; `statement/block/mod.rs:109`; `statement/switch/mod.rs:88`; `statement/declaration/lexical.rs:239`; `statement/declaration/hoistable/class_decl/mod.rs:712` |
| W2 | ``lexical name `x` declared multiple times`` | `boa_parser/src/parser/mod.rs:512,526` (**module goal only**) |
| W3 | `lexical name declared in var names` | `statement/block/mod.rs:122`; `class_decl/mod.rs:724` |
| W4 | `lexical name declared in var declared names` | `statement/switch/mod.rs:101` |
| W5 | `invalid scope analysis: duplicate lexical declaration` | `boa_parser/src/parser/mod.rs:186-191` wrapping `boa_ast/src/scope_analyzer.rs:1783,1793` |

Front (`crates/lila-front/src/lib.rs:235-242`) matches W1 (literal), W5
(`"duplicate lexical declaration"`) and — via the loose
`"lexical" && "declared" && "names"` fallback — W3 and W4. It does **not** match
W2, because W2 contains `name \`x\`` and not the substring `names`.

`early.rs:126-129` (`["lexical name", "declared multiple times"]`) matches W1 and
W2. It does **not** match W3, W4 or W5.

So the drift is bidirectional and both directions are live:

- **A module-goal duplicate lexical declaration reports `P_PARSE_MALFORMED` as an
  entry file and `E_DUPLICATE_LEXICAL_DECLARATION` as a dependency.** This is the
  exact failure the front doc comment at `lib.rs:207-211` forbids in words.
  `test/language/module-code/early-dup-lex.js` and `early-lex-and-var.js` both go
  down this path.
- **A W3/W4/W5 lexical error in a dependency module reports `Unsupported`**, whose
  `error_type` is `None` and whose kind is not in
  `{EarlyError, LinkError}` — so `compile_negative_error_matches`
  (`crates/lila-test262/src/lib.rs:21501-21506`) rejects it and the case
  **fails**. This one is conformance-visible, not merely taxonomy-visible.

### 0.5 W5 is **not** reachable from the product path — corrected at DISCREPANCY-FIXER stage

> **This subsection said the opposite. DR-6 refuted it, and the refutation was
> re-verified against `vendor/` before this rewrite.**

`Parser::parse_script_with_source` (`boa_parser/src/parser/mod.rs:179-193`) and
`Parser::parse_module_with_source` (`:222-235`) both call `ast.analyze_scope(...)`
and wrap any failure as `format!("invalid scope analysis: {reason}")`. So far so
good. What is false is the claim that the wrapped failure is reachable.

Measured in `boa_ast-0.21.1/src/scope_analyzer.rs`:

- the only payload-carrying `ControlFlow::Break` is at **:1220**, in
  `visit_script_mut`, forwarding `global_declaration_instantiation`'s `Err` at
  **:1783** and **:1793**;
- both of those returns require `env.has_binding(name)` / `env.has_lex_binding(name)`
  on the `Scope` that was passed in;
- both callers in this workspace pass a **fresh** `Scope::new_global()` —
  `lila_front::parse` (`front/lib.rs:239`) and
  `lila_ir::modules::record::reparse_module` (`record.rs:1333`) — whose
  `bindings` are `RefCell::default()` (`boa_ast/src/scope.rs:115-128`);
- `visit_module_mut` (**:1202-1210**) contains no `Break` at all, so the module
  goal cannot reach it under any scope.

Row 6 is therefore **dead through this crate's entry points**, and is ledger
**L5**'s first confirmed instance. It is *kept*, because DR-6 forbids deleting a
row on a negative reachability result and because boa has a **third** producer of
the same wording at `scope_analyzer.rs:2364`
(`eval_declaration_instantiation_scope`, reachable only via `analyze_scope_eval`,
which this compiler never calls). That producer matches row 6 and only row 6, so
an eval path added later classifies correctly with no edit. The row's comment in
`early_error_code.rs` now carries the measured statement instead of the false one.

The contract's earlier five-wording enumeration for
`E_DUPLICATE_LEXICAL_DECLARATION` was also incomplete: `scope_analyzer.rs:2364`
is a sixth producer. No misclassification follows.

### 0.6 The `super` / `new.target` "prefix-overlap hazard" does not exist

Measured. `"module cannot contain \`new.target\` on the top-level"` does not
contain the substring `super`; `"module cannot contain \`super\` on the
top-level"` does not contain `new.target`. The two rules are disjoint under
`contains`, in both orders. The brief's ordering concern is unfounded *for these
two rows* — but the property was only ever a comment
(`early.rs:114`: "Order matters only in that the first match wins; the patterns
are disjoint"). §2.4 turns it into const assertion **P2**, which proves it for
every row and makes the table order-independent.

### 0.7 `E_MODULE_DUPLICATE_EXPORT` is already emitted under two phases

`crates/lila-ir/src/modules/early.rs:44-55` emits it through
`IrDiagnostic::early_error` (kind `EarlyError`, phase `Early`).
`crates/lila-ir/src/modules/graph.rs:783-788` pushes
`ModuleLinkErrorIr::DuplicateExport`, which becomes
`IrDiagnostic::link_error` (kind `LinkError`, phase `Resolution`) at
`graph.rs:226`. 16.2.3.1 makes it an **early** error and
`test/language/module-code/early-dup-export-id.js` is `phase: parse`, so the link
path is wrong.

It is currently unreachable — `parse_module_record`
(`record.rs:740-743`) returns `Err(early_errors)` before the graph is linked — so
this is latent MC4, not live MC4. §2.4's assertion **P7** makes it unrepresentable
rather than merely unreached.

---

## 1. Spec basis

### 1.1 What an early error is, formally

ECMA-262 **clause 17** (*Error Handling and Language Extensions*) fixes the whole
shape of this area:

- An **early error** is an error detectable and reportable *prior to the
  evaluation of any construct* in the Script or Module containing it.
- The presence of an early error **prevents evaluation** of the construct. It is
  not a value that flows; there is no Completion Record for it.
- An implementation **must** report early errors in a Script as part of parsing
  that Script in `ParseScript`, and in a Module as part of parsing that Module in
  `ParseModule`.
- An implementation **must** treat as an early error every occurrence of a
  condition listed in a *Static Semantics: Early Errors* subclause, and
  **must not** treat other kinds of error as early errors.

Three formal consequences, and they are what the types must carry:

1. **The set is closed.** "Every occurrence of a condition listed in a *Static
   Semantics: Early Errors* subclause" is a finite enumeration over productions.
   Closed set → Rust `enum`, exhaustive `match`, no catch-all (AGENTS.md,
   *Code Invariants Before Test Invariants*).
2. **The reporting time is fixed by the operation, not chosen per error.**
   `ParseScript` (16.1.4) and `ParseModule` (16.2.1.6.1) each return *a List of
   SyntaxError objects* on failure. The phase and the error type are properties
   of *where the rejection is produced*, not free parameters of the rejection.
   → phase and error type must be **derived**, never fields.
3. **A shared production is rejected by shared static semantics.** The Script
   early errors (16.1.1) and the Module early errors (16.2.1.2, cited as
   16.2.3.1 by the existing code — see §1.5.4) apply the *same*
   `ContainsDuplicateLabels`, `ContainsUndefinedBreakTarget`,
   `ContainsUndefinedContinueTarget` and `AllPrivateIdentifiersValid` operations
   to their respective item lists. A source that is an early error as a Script is
   an early error as a Module, under the same rule, with the same name.
   → **one** classification table, not one per parse path.

### 1.2 The conditions in scope, with citations

Citations follow the clause numbers already written into this repository's
source and into the test262 files' own `esid`/`es6id` frontmatter, both of which
were read. Where an edition renumbers a clause, the *operation name* is
authoritative and the number is a pointer.

| Condition | Clause | Abstract operation |
|---|---|---|
| Duplicate `__proto__` in an ObjectLiteral | **B.3.1** (`sec-__proto__-property-names-in-object-initializers`; amends the 13.2.5.1 ObjectLiteral early errors) | `PropertyNameList of PropertyDefinitionList` contains ≥2 duplicate `"__proto__"` entries from `PropertyDefinition : PropertyName : AssignmentExpression` |
| Duplicate lexical declaration, Script | **16.1.1** | `LexicallyDeclaredNames of ScriptBody` has duplicates; or intersects `VarDeclaredNames of ScriptBody` |
| Duplicate lexical declaration, Module | **16.2.1.2** | `LexicallyDeclaredNames of ModuleItemList` has duplicates; or intersects `VarDeclaredNames of ModuleItemList` |
| Duplicate lexical declaration, Block / Switch / Class body | **14.2.1**, **14.12.1**, **15.7.1** | same two conditions over the enclosing item list |
| Formal parameters vs. function body lexicals | **15.2.1** (and 15.1.1) | `BoundNames of FormalParameters` intersects `LexicallyDeclaredNames of FunctionBody` |
| Duplicate label | **14.13.1** | `ContainsDuplicateLabels` with argument « » is `true` |
| Undefined `break` target | **14.13.1**, applied by 16.1.1 / 16.2.1.2 | `ContainsUndefinedBreakTarget` with argument « » is `true` |
| Undefined `continue` target | **14.13.1**, applied by 16.1.1 / 16.2.1.2 | `ContainsUndefinedContinueTarget` with arguments « », « » is `true` |
| Illegal `break` | **14.9.1** | `BreakStatement` not nested within an `IterationStatement` or `SwitchStatement` |
| Illegal `continue` | **14.8.1** | `ContinueStatement` not nested within an `IterationStatement` |
| Invalid private identifier | **15.7.1**, applied by 16.1.1 / 16.2.1.2 | `AllPrivateIdentifiersValid` with argument « » is `false` |
| Duplicate `ExportedNames` | **16.2.3.1** / 16.2.1.2 | `ExportedNames of ModuleItemList` contains duplicates |
| `ExportedBindings` with no declaration | **16.2.3.1** / 16.2.1.2 | element of `ExportedBindings` not in `VarDeclaredNames` ∪ `LexicallyDeclaredNames` |
| Module top-level `super` | **16.2.1.2** | `ModuleItemList Contains super` |
| Module top-level `new.target` | **16.2.1.2** | `ModuleItemList Contains NewTarget` |

### 1.3 The link-phase conditions, and why they are in the same domain

Module linking is not clause-17 early-error territory — it happens after every
module in the closure has been parsed successfully. But its failures share every
property that matters here:

- They are decided **before evaluation of any construct** (16.2.1.5.1
  `InnerModuleLinking`, and `InitializeEnvironment` which throws before the
  module body runs).
- They are **`SyntaxError`s**: `InitializeEnvironment` step "If `resolution` is
  either `null` or `ambiguous`, throw a `SyntaxError` exception."
- test262 gives them their own `phase: resolution` but the *same*
  `type: SyntaxError` — measured, see §1.4.
- In an AOT compiler they are reported at compile time, exactly like an early
  error. `crates/lila-ir/src/diagnostics.rs:28-31` already says this.

They therefore belong in the same closed code domain, distinguished by a derived
phase, not by living in a separate enum. `ModuleLinkErrorIr::code()`
(`graph.rs:174-186`) is already that enumeration in `&'static str` form.

Two of its seven are **not** spec conditions at all —
`E_MODULE_UNSUPPORTED_PHASE` and `E_MODULE_TOO_MANY_UNITS` are implementation
limits (`graph.rs:141-168`), and their messages say so
("unsupported in lila wasm-aot: ..."). Their current classification as
`SyntaxError`/`Resolution` is a spec claim this compiler cannot support. That is
a **recorded defect, deliberately not fixed here** — see §4 B0 and ledger entry
L4.

### 1.4 The conformance shadow, measured

Every `negative: { phase, type }` pair in the pinned suite
(`test262/vendor/test262/test`), counted:

```
   4592  parse/SyntaxError
     34  resolution/SyntaxError
     14  runtime/Test262Error
     14  runtime/ReferenceError
      4  runtime/TypeError
      4  runtime/SyntaxError
      3  runtime/EvalError
      1  runtime/RangeError
```

**There is no `parse/<anything-but-SyntaxError>` and no
`resolution/<anything-but-SyntaxError>` in the entire pinned suite.** The
`(phase, type)` relation on the pre-evaluation phases is a constant function.
This is the empirical half of the argument that `error_type` is derived and not
stored; 16.1.4 and 16.2.1.6.1 ("a List of *SyntaxError* objects") are the
normative half.

Frontmatter verified individually for all ten named corpus files (§7); all ten
are `phase: parse, type: SyntaxError`.

### 1.5 Where the spec leaves latitude, and what this contract chooses

**1.5.1 Which mechanism detects the error.** Clause 17 fixes *that* the error is
reported at parse and *that* it is a `SyntaxError`; it says nothing about how an
implementation discovers it. This compiler discovers it by asking `boa` and
reading the message string. That is a fragile oracle and this contract does not
pretend otherwise: **the types buy single-sourcing and exhaustiveness, not oracle
robustness.** If boa rewords a message, exactly one row of one table becomes
dead, no compile error fires, and a test262 case silently reclassifies. §3.1
records that as ledger entry **L1**, and §2.1.2's `witnesses` column is the
mitigation (a reworded message makes the row's own witness stale and visible in
one place),
not a cure.

**1.5.2 Naming.** The `E_...` wire names are not spec artefacts; they are this
project's taxonomy. The spec constrains the *set*, not the spelling. This
contract fixes the spelling as "whatever is in the tree today", byte for byte,
so the retrofit is a pure single-sourcing change with no taxonomy churn — every
one of the 18 `wire_name()` strings is copied verbatim from the literal it
replaces.

**1.5.3 Granularity.** The spec does not require one code per condition. Where
boa emits several wordings for one spec rule (the five in §0.4), this contract
uses **several rows, one code**. Where one wording covers several spec rules
(W1 covers both 16.1.1 conditions), one row is enough. Rows are keyed by boa's
message shape; codes are keyed by the spec rule.

**1.5.4 Clause numbering.** The existing code cites 16.2.3.1 for the
`ExportedNames`/`ExportedBindings` rules (`early.rs:42,57`), which in current
editions sit in the `Module : ModuleBody` early-errors clause (16.2.1.2). This
contract does not renumber anything: it keeps the citations the code already
carries and names the operation alongside, so the reference survives renumbering.

---

## 2. The types

### 2.0 The two shapes being replaced

Today, three independent free fields carry what is one derived triple:

```rust
// crates/lila-ir/src/diagnostics.rs:35-43
pub struct IrDiagnostic {
    pub kind: IrDiagnosticKind,          // EarlyError | LinkError | Unsupported | Lowering
    pub phase: IrDiagnosticPhase,        // Early | Resolution | Lowering
    pub code: Option<&'static str>,
    pub error_type: Option<&'static str>,
    pub span: Option<SourceSpan>,
    pub message: String,
}
```

```rust
// crates/lila-front/src/lib.rs:57-65
pub struct ParseDiagnostic {
    pub kind: ParseDiagnosticKind,       // MalformedJavaScript | UnsupportedParserFeature
    pub phase: ParseDiagnosticPhase,     // Parse | Early
    pub code: &'static str,
    pub error_type: &'static str,
    pub span: Option<SourceSpan>,
    pub message: String,
}
```

Both collapse to **one closed field plus payload**. Everything else is a `const
fn` of it.

### 2.1 `crates/lila-front/src/early_error_code.rs` — NEW

The sole definition of the domain, the sole producer of every `E_...` string, and
the sole classifier of boa messages.

#### 2.1.1 The enum, generated from one row list

Follow the shape of `crates/lila-ir/src/native_error.rs` exactly — it is the
landed precedent from round 1's `closed-name-domains.md`, and consistency here is
worth more than novelty. One `macro_rules!` row list generates the enum, `ALL`,
`wire_name` and `from_wire_name`, so there is no second list to forget.

```rust
early_error_codes! {
    // ---- rejected during parse (clause 17; ParseScript 16.1.4 / ParseModule 16.2.1.6.1)
    /// B.3.1. Duplicate `__proto__` in an object literal.
    ObjectDuplicateProto         => "E_OBJECT_DUPLICATE_PROTO";
    /// 16.1.1 / 16.2.1.2 / 14.2.1 / 14.12.1 / 15.2.1. Any lexical redeclaration.
    DuplicateLexicalDeclaration  => "E_DUPLICATE_LEXICAL_DECLARATION";
    /// 14.13.1 ContainsDuplicateLabels.
    DuplicateLabel               => "E_DUPLICATE_LABEL";
    /// 14.13.1 ContainsUndefinedBreakTarget.
    UndefinedBreakTarget         => "E_UNDEFINED_BREAK_TARGET";
    /// 14.13.1 ContainsUndefinedContinueTarget.
    UndefinedContinueTarget      => "E_UNDEFINED_CONTINUE_TARGET";
    /// 14.9.1. `break` not nested in an iteration or switch statement.
    IllegalBreak                 => "E_ILLEGAL_BREAK";
    /// 14.8.1. `continue` not nested in an iteration statement.
    IllegalContinue              => "E_ILLEGAL_CONTINUE";
    /// 15.7.1 AllPrivateIdentifiersValid, applied by 16.1.1 / 16.2.1.2.
    InvalidPrivateIdentifier     => "E_INVALID_PRIVATE_IDENTIFIER";
    /// 16.2.3.1. Duplicate ExportedNames.
    ModuleDuplicateExport        => "E_MODULE_DUPLICATE_EXPORT";
    /// 16.2.3.1. ExportedBinding with no VarDeclared/LexicallyDeclared name.
    ModuleUndeclaredExport       => "E_MODULE_UNDECLARED_EXPORT";
    /// 16.2.1.2. ModuleItemList Contains super.
    ModuleTopLevelSuper          => "E_MODULE_TOP_LEVEL_SUPER";
    /// 16.2.1.2. ModuleItemList Contains NewTarget.
    ModuleTopLevelNewTarget      => "E_MODULE_TOP_LEVEL_NEW_TARGET";
    // ---- rejected during linking (16.2.1.5 ResolveExport / InitializeEnvironment)
    /// A requested specifier the host could not resolve.
    ModuleUnresolved             => "E_MODULE_UNRESOLVED";
    /// ResolveExport returned null.
    ModuleMissingExport          => "E_MODULE_MISSING_EXPORT";
    /// ResolveExport returned ambiguous.
    ModuleAmbiguousExport        => "E_MODULE_AMBIGUOUS_EXPORT";
    /// Host invariant: one key loaded twice with different source text.
    ModuleInconsistentLoad       => "E_MODULE_INCONSISTENT_LOAD";
    /// Implementation limit, not a spec condition. See ledger L4.
    ModuleUnsupportedPhase       => "E_MODULE_UNSUPPORTED_PHASE";
    /// Implementation limit, not a spec condition. See ledger L4.
    ModuleTooManyUnits           => "E_MODULE_TOO_MANY_UNITS";
}
```

Generated surface:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum EarlyErrorCode { /* 18 variants, declaration order */ }

impl EarlyErrorCode {
    /// Every code, in declaration order. The length is in the type: adding a
    /// row without updating it is `error[E0308]`.
    pub const ALL: [EarlyErrorCode; 18] = [ /* ... */ ];

    /// The single spelling authority. This is the only place in the workspace
    /// where an `E_...` literal exists.
    #[must_use]
    pub const fn wire_name(self) -> &'static str { /* 18-arm match */ }

    /// The only parse. Total on the domain, `None` off it.
    ///
    /// Its **sole consumer** is const assertion P3 below, which uses the round
    /// trip to prove `wire_name` is injective. It is `pub` only because the
    /// assertion and the type share a module boundary with the rest of the
    /// crate; if a product call site is ever added, delete this comment. A
    /// `pub` item with no consumer is the "survival by `pub`" shape AGENTS.md
    /// names, and `native_error.rs`'s module docs record what it cost here.
    #[must_use]
    pub const fn from_wire_name(name: &str) -> Option<Self> { /* ... */ }
}
```

**Deliberately absent, and never to be added** (same rule as `NativeErrorKind`):
`Display`, `AsRef<str>`, `Deref<Target = str>`, `FromStr`, `Default`,
`From<EarlyErrorCode> for String`. A stringification must name `wire_name()` at
the call site, so `format!("{code}")` cannot quietly resurrect the `&str` domain
this type replaces.

#### 2.1.2 The one fragment table

```rust
/// One boa static-semantics message shape, and the code it denotes.
struct ParseFailureRule {
    /// Every fragment that must appear in boa's message for this rule to fire.
    /// Chosen to be the invariant part of boa's `format!`, never an
    /// interpolated identifier. Never empty — const assertion P1.
    fragments: &'static [&'static str],
    /// The code this shape denotes.
    code: EarlyErrorCode,
    /// Every message boa actually produces that this row must classify, copied
    /// verbatim from the cited source. A **list**, because one fragment set
    /// legitimately covers several of boa's wordings. Non-empty — P1.
    /// Consumed by const assertions P2 and P6. It is not documentation: a row
    /// whose witnesses stop selecting it does not compile.
    witnesses: &'static [&'static str],
}
```

**2026-08-23 anchored-pattern amendment.** The current schema replaces the
bare `fragments` field above with one closed match domain:

```rust
enum ParseFailurePattern {
    ContainsAll(&'static [&'static str]),
    StartsWith(&'static str),
}

struct ParseFailureRule {
    pattern: ParseFailurePattern,
    code: EarlyErrorCode,
    witnesses: &'static [&'static str],
}
```

`ContainsAll` retains the original invariant-fragment behavior. `StartsWith`
is available when a producer audit establishes one complete fixed message
followed only by a source position; for that row it prevents user-controlled
text interpolated later inside a different `Error::general` diagnostic from
forging the condition. `rule_matches` and P1 consume the enum exhaustively; an
empty fragment list, empty fragment, or empty prefix remains a compile-time
assertion failure. The duplicate static import-attribute-key rule is the sole
anchored consumer in this amendment. Existing `ContainsAll` rows retain their
prior behavior and are not newly audited for General-message interpolation.
The 15-row table below records the original measured foundation and is
historical rather than a current row-count claim.

`witnesses` is a list and not a single string for a measured reason: boa emits
`lexical name declared multiple times` **and**
``lexical name `x` declared multiple times`` for one spec rule, and the fragment
set `["lexical name", "declared multiple times"]` covers both. A one-string
witness column would have forced a second, redundant row whose fragments are a
superset of the first's — which P2 would then have to be weakened to tolerate.
The list keeps P2 in its strong form: **exactly one row matches each witness.**

**The table has 15 rows.** Every witness is a byte-for-byte copy of a literal
read from `vendor/`; the cited `file:line` is where it was read.

| # | `fragments` | `code` | `witnesses` | boa source |
|---|---|---|---|---|
| 1 | `["Duplicate __proto__ fields"]` | `ObjectDuplicateProto` | `Duplicate __proto__ fields are not allowed in object literals.` | `boa_parser/src/parser/expression/primary/object_initializer/mod.rs:133` |
| 2 | `["exported name", "declared multiple times"]` | `ModuleDuplicateExport` | ``exported name `x` declared multiple times`` | `boa_parser/src/parser/mod.rs:541` |
| 3 | `["could not find the exported binding"]` | `ModuleUndeclaredExport` | ``could not find the exported binding `x` in the declared names of the module`` | `boa_parser/src/parser/mod.rs:556` |
| 4 | `["lexical name", "declared multiple times"]` | `DuplicateLexicalDeclaration` | **two:** ``lexical name `x` declared multiple times`` (W2) and `lexical name declared multiple times` (W1) | W2: `boa_parser/src/parser/mod.rs:512,526`. W1: `mod.rs:366,376`, `block/mod.rs:109`, `switch/mod.rs:88`, `lexical.rs:239`, `class_decl/mod.rs:712` |
| 5 | `["lexical name declared in var"]` | `DuplicateLexicalDeclaration` | **two:** `lexical name declared in var names` (W3) and `lexical name declared in var declared names` (W4) | W3: `block/mod.rs:122`, `class_decl/mod.rs:724`. W4: `switch/mod.rs:101` |
| 6 | `["duplicate lexical declaration"]` | `DuplicateLexicalDeclaration` | `invalid scope analysis: duplicate lexical declaration` (W5) | `boa_parser/src/parser/mod.rs:186-191` wrapping `boa_ast/src/scope_analyzer.rs:1783,1793` |
| 7 | `["formal parameter", "declared in lexically declared names"]` | `DuplicateLexicalDeclaration` | ``formal parameter `x` declared in lexically declared names`` | `boa_parser/src/parser/mod.rs:614` |
| 8 | `["module cannot contain", "super"]` | `ModuleTopLevelSuper` | ``module cannot contain `super` on the top-level`` | `boa_parser/src/parser/mod.rs:567` |
| 9 | `["module cannot contain", "new.target"]` | `ModuleTopLevelNewTarget` | ``module cannot contain `new.target` on the top-level`` | `boa_parser/src/parser/mod.rs:575` |
| 10 | `["invalid private identifier usage"]` | `InvalidPrivateIdentifier` | `invalid private identifier usage` | `boa_parser/src/parser/mod.rs:462,593`, `statement/mod.rs:1020` |
| 11 | `["duplicate label"]` | `DuplicateLabel` | `duplicate label: lbl` | `boa_ast/src/operations/mod.rs:1402` |
| 12 | `["undefined break target"]` | `UndefinedBreakTarget` | `undefined break target: lbl` | `boa_ast/src/operations/mod.rs:1406` |
| 13 | `["undefined continue target"]` | `UndefinedContinueTarget` | `undefined continue target: lbl` | `boa_ast/src/operations/mod.rs:1411` |
| 14 | `["illegal break statement"]` | `IllegalBreak` | `illegal break statement` | `boa_ast/src/operations/mod.rs:1414` |
| 15 | `["illegal continue statement"]` | `IllegalContinue` | `illegal continue statement` | `boa_ast/src/operations/mod.rs:1415` |

Four rows (4, 5, 6, 7) carry `DuplicateLexicalDeclaration`. That is intended and
is §1.5.3's granularity choice: rows are keyed by boa's message shape, codes by
the spec rule.

**Disjointness, checked by hand before it was written as P2.** Each of the 17
witness strings above was tested against all 15 fragment sets. Every one matches
exactly one row. The three cases worth naming because they look close:

- W5 (`invalid scope analysis: duplicate lexical declaration`) does **not** match
  row 4 — it contains `lexical declaration`, not `lexical name`.
- ``formal parameter `x` declared in lexically declared names`` does **not**
  match row 4 or row 5 — it contains `lexically declared`, not `lexical name`.
- Rows 8 and 9 are disjoint in both directions: the `super` message contains no
  `new.target`, and the `new.target` message contains no `super`.

Ordering: first match wins, as today. P2 proves the order is irrelevant, so a
future insertion cannot silently change an existing classification.

#### 2.1.3 The classifier

```rust
/// The prefix `lila_ir::modules::record::reparse_module` puts in front of
/// boa's message (`record.rs:1343`). Const assertion P6 proves it matches no
/// rule on its own, so classifying a prefixed message and a bare one give the
/// same answer.
pub const MODULE_REPARSE_PREFIX: &str = "lowering module reparse failed: ";

/// Classifies a boa parse failure. `None` means "a syntax error whose wording
/// we do not model", which must stay `Unsupported` / `MalformedJavaScript` —
/// claiming a spec rejection for a source we merely failed to read would dress
/// a compiler gap up as a spec claim.
#[must_use]
pub fn classify_parse_failure(message: &str) -> Option<EarlyErrorCode> {
    for rule in PARSE_FAILURE_RULES {
        if rule.fragments.iter().all(|f| message.contains(f)) {
            return Some(rule.code);
        }
    }
    None
}
```

There is exactly one such function in the workspace. `lila-ir` calls this one.

### 2.2 `crates/lila-ir/src/early_error_code.rs` — NEW

Everything that needs a `lila-ir` type, and nothing else. `EarlyErrorCode` is
foreign here, so this is a free `const fn`, not an inherent `impl`.

```rust
pub use lila_front::EarlyErrorCode;

/// Which stage rejects a program carrying this code.
///
/// The single `EarlyErrorCode → IrDiagnosticKind` map. Exhaustive with no
/// catch-all: a nineteenth code is `error[E0004]` here, which is the point.
/// Phase and error type are then functions of the *kind* (§2.3), so a code
/// determines all three and nothing can be paired inconsistently.
pub(crate) const fn rejection_kind(code: EarlyErrorCode) -> IrDiagnosticKind {
    match code {
        EarlyErrorCode::ObjectDuplicateProto
        | EarlyErrorCode::DuplicateLexicalDeclaration
        | EarlyErrorCode::DuplicateLabel
        | EarlyErrorCode::UndefinedBreakTarget
        | EarlyErrorCode::UndefinedContinueTarget
        | EarlyErrorCode::IllegalBreak
        | EarlyErrorCode::IllegalContinue
        | EarlyErrorCode::InvalidPrivateIdentifier
        | EarlyErrorCode::ModuleDuplicateExport
        | EarlyErrorCode::ModuleUndeclaredExport
        | EarlyErrorCode::ModuleTopLevelSuper
        | EarlyErrorCode::ModuleTopLevelNewTarget => IrDiagnosticKind::EarlyError,

        EarlyErrorCode::ModuleUnresolved
        | EarlyErrorCode::ModuleMissingExport
        | EarlyErrorCode::ModuleAmbiguousExport
        | EarlyErrorCode::ModuleInconsistentLoad
        | EarlyErrorCode::ModuleUnsupportedPhase
        | EarlyErrorCode::ModuleTooManyUnits => IrDiagnosticKind::LinkError,
    }
}
```

Note `ModuleDuplicateExport` is `EarlyError`. That is 16.2.3.1, and it is what
forces the fix in §0.7 / §4 B4: `ModuleLinkErrorIr::DuplicateExport` now yields
an `EarlyError`/`Early` diagnostic from *both* producers, and const assertion P7
makes any other answer fail to build.

### 2.3 `crates/lila-ir/src/diagnostics.rs` — rewritten

```rust
use lila_front::{EarlyErrorCode, SourceSpan};
use crate::early_error_code::rejection_kind;
use crate::NativeErrorKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrDiagnosticKind { EarlyError, LinkError, Unsupported, Lowering }
//  ^ unchanged, four unit variants — see §2.6 for why the payload does NOT move in here.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrDiagnosticPhase { Early, Resolution, Lowering }   // unchanged

impl IrDiagnosticKind {
    /// 16.1.4 / 16.2.1.6.1 fix the phase per producing operation; it is never a
    /// free choice at a call site.
    #[must_use]
    pub const fn phase(self) -> IrDiagnosticPhase {
        match self {
            Self::EarlyError => IrDiagnosticPhase::Early,
            Self::LinkError => IrDiagnosticPhase::Resolution,
            Self::Unsupported | Self::Lowering => IrDiagnosticPhase::Lowering,
        }
    }

    /// `ParseScript` and `ParseModule` return "a List of **SyntaxError**
    /// objects"; `InitializeEnvironment` throws a **SyntaxError**. Measured:
    /// zero `parse/` or `resolution/` negatives of any other type exist in the
    /// pinned suite (§1.4). `Unsupported` and `Lowering` are compiler gaps and
    /// must **not** claim a spec error type.
    #[must_use]
    pub const fn error_type(self) -> Option<NativeErrorKind> {
        match self {
            Self::EarlyError | Self::LinkError => Some(NativeErrorKind::SyntaxError),
            Self::Unsupported | Self::Lowering => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDiagnostic {
    pub kind: IrDiagnosticKind,
    /// PRIVATE, and that is the invariant. The only writer is
    /// `IrDiagnostic::rejected`, which derives `kind` from this value. A struct
    /// literal outside this module is `error[E0451]` (private field `code`) or
    /// `error[E0063]` (missing field `code`) — so a fifth constructor cannot be
    /// written anywhere except beside the other four, where the derivation is.
    code: Option<EarlyErrorCode>,
    pub span: Option<SourceSpan>,
    pub message: String,
}

impl IrDiagnostic {
    /// The **only** constructor that can produce a coded diagnostic: the
    /// program is rejected before evaluation, with this code. Replaces
    /// `early_error` and `link_error`, whose separation was exactly the
    /// opportunity to disagree.
    pub fn rejected(
        code: EarlyErrorCode,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self { kind: rejection_kind(code), code: Some(code), span, message: message.into() }
    }

    pub fn unsupported(message: impl Into<String>) -> Self { /* code: None */ }
    pub fn lowering(message: impl Into<String>) -> Self { /* code: None */ }

    #[must_use] pub const fn code(&self) -> Option<EarlyErrorCode> { self.code }
    #[must_use] pub const fn phase(&self) -> IrDiagnosticPhase { self.kind.phase() }
    #[must_use] pub const fn error_type(&self) -> Option<NativeErrorKind> { self.kind.error_type() }
}
```

`LoweringStage` is unchanged.

### 2.4 `crates/lila-front/src/lib.rs` — `ParseDiagnostic` collapses the same way

```rust
/// Everything `lila_front::parse` can report, as one closed domain.
///
/// The two `P_...` codes are compiler-gap codes, not spec rejections; keeping
/// them out of `EarlyErrorCode` is deliberate — an `EarlyErrorCode` must always
/// name a program the spec rejects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseCode {
    /// Boa rejected the source and we do not model the wording.
    Malformed,                    // "P_PARSE_MALFORMED"
    /// Boa's parser aborted (panic caught at `lib.rs:187-195`).
    UnsupportedParserFeature,     // "P_PARSE_UNSUPPORTED"
    /// A modelled spec rejection.
    ///
    /// DISCREPANCY-FIXER: the payload is `ParseClassified`, not a bare
    /// `EarlyErrorCode` — see the note below this block.
    Early(ParseClassified),
}

impl ParseCode {
    #[must_use] pub const fn wire_name(self) -> &'static str { /* two literals + delegate */ }
    #[must_use] pub const fn kind(self) -> ParseDiagnosticKind { /* Malformed|Early → MalformedJavaScript */ }
    #[must_use] pub const fn phase(self) -> ParseDiagnosticPhase { /* Early(_) → Early, else Parse */ }
    /// The one `"SyntaxError"` literal in this crate. It cannot be
    /// `NativeErrorKind` — see ledger L2.
    ///
    /// DISCREPANCY-FIXER: `Option`, and `None` for `UnsupportedParserFeature`.
    #[must_use] pub const fn error_type(self) -> Option<&'static str> { /* None for the abort */ }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub code: ParseCode,
    pub span: Option<SourceSpan>,
    pub message: String,
}
// kind(), phase(), error_type() become accessors delegating to `self.code`.
```

`ParseError::malformed` / `unsupported_parser_feature` / `early_error` keep their
names; `early_error` loses its `code: &'static str` and `error_type: &'static str`
parameters.

#### 2.4.1 `ParseClassified` — the witness `early_error` takes (DISCREPANCY-FIXER)

As first written, `early_error` took a bare `EarlyErrorCode`, so

```rust
ParseError::early_error(EarlyErrorCode::ModuleMissingExport, msg, None)
```

compiled — reporting a code whose `rejection_kind` is `LinkError`/`Resolution`
at `ParseDiagnosticPhase::Early`. That is **MC4 verbatim**: one condition under
two phases from two paths. P7 constrains what the *table* can yield; it says
nothing about what a *call site* can name, and `ParseDiagnosticPhase` and
`IrDiagnosticPhase` are unrelated types with nothing tying them. The mirror image
held in `lila-ir`: `IrDiagnostic::rejected(EarlyErrorCode::ModuleMissingExport, …)`
compiled inside `modules/early.rs`, a `ParseModule`-stage producer that names two
codes directly.

Both are now `error[E0308]`:

```rust
/// A code that the *parse* table can actually produce. No public constructor
/// beyond the two gated ones.
pub struct ParseClassified(EarlyErrorCode);

impl ParseClassified {
    /// Gated on `is_parse_classified`. `None` for a link-only condition.
    pub const fn from_early(code: EarlyErrorCode) -> Option<Self>;
    /// `from_early` with `None` as a **compile error**: the `None` arm is a
    /// `panic!`, so a `const` initializer built with it fails to build.
    pub const fn from_parse_table(code: EarlyErrorCode) -> Self;
    pub const fn code(self) -> EarlyErrorCode;
}
```

- `classify_parse_failure` returns `Option<ParseClassified>`;
- `ParseCode::Early` carries a `ParseClassified`;
- `ParseError::early_error` takes one;
- `IrDiagnostic::rejected_at_parse(ParseClassified, …)` is added beside
  `rejected`, and is what `modules/early.rs`'s three producers call. Its two
  direct namings are bound to `const` items built with `from_parse_table`, so
  rustc evaluates the gate.

The witness subsumes `is_parse_classified`'s previous single-consumer role: P7
still uses it, and now so does the constructor.

### 2.5 The const assertions

All of P1–P6 live in `crates/lila-front/src/early_error_code.rs`; P7–P9 live
in `crates/lila-ir/src/early_error_code.rs`. They need a private
`const fn contains_sub(haystack: &str, needle: &str) -> bool` written as a byte
loop, in the shape of `native_error.rs`'s private `str_eq` (and, like it, private
on purpose: a `pub const fn contains_sub` would be workspace surface with no
product call site).

| id | statement | the mistake it makes fail to build |
|---|---|---|
| **P1** | every row has `fragments.len() >= 1` and `witnesses.len() >= 1`, **and no fragment or witness is the empty string** (DISCREPANCY-FIXER) | `fragments: &[]` matches **every** message (`[].iter().all(_)` is `true`), silently swallowing all parse failures into one code. So does `fragments: &[""]`: `contains_sub` returns `true` for an empty needle, since its length guard only rejects a needle *longer* than the haystack. That second case used to be caught only as a side effect of P2 — i.e. only because other rows happened to have witnesses |
| **P2** | for every row *i* and every `w` in `rows[i].witnesses`: exactly one row of `PARSE_FAILURE_RULES` matches `w`, and it is row *i* | a new row that shadows an existing one, or is shadowed by it; **and** it makes the table order-independent, upgrading `early.rs:114`'s comment to a checked fact |
| **P3** | `ALL[i] as u8 == i` for all *i*, and `from_wire_name(c.wire_name()) == Some(c)` for all `c` | `ALL` out of declaration order or incomplete; `wire_name`/`from_wire_name` diverging |
| **P4** | no two codes share a `wire_name()` | a duplicated spelling would make one code unreachable through `from_wire_name` and would collapse two taxonomy buckets |
| **P5** | every `wire_name()` begins `"E_"` and contains only `b'A'..=b'Z'` and `b'_'` | `"e_FOO"`, `"E_Foo"`, a stray space — a typo that a `&str` domain would carry all the way into the failure taxonomy |
| **P5'** | no `wire_name()` equals `NO_EARLY_ERROR_CODE` (`"E_IR_DIAGNOSTIC"`, now spelled once in `lila-front` beside the codes) (DISCREPANCY-FIXER) | a nineteenth variant spelled `E_IR_DIAGNOSTIC` passes P4 (distinct) and P5 (well-formed) and is then indistinguishable from "no code" in every failure-detail string — the §0.2 confusion, reintroduced |
| **P6** | `classify_parse_failure(MODULE_REPARSE_PREFIX) == None` | a future fragment (e.g. bare `"module"`) that the dependency-path wrapper prefix would match on its own, making every dependency parse failure classify as one code |
| **P10** | for every row *i* and every `w` in `rows[i].witnesses`, `classify_parse_failure(w) == Some(rows[i].code)` — checked through the **whole** classifier, not row by row (DISCREPANCY-FIXER) | P2 checks `rule_matches`; the classifier now also refuses `INTERPOLATING_MESSAGE_SHAPES` (ledger L1). A guard shape that happens to occur inside a real boa wording would silently take a whole spec condition out of the taxonomy, and P2 would still pass |
| **P7** | for every row of `PARSE_FAILURE_RULES`, `rejection_kind(row.code) == IrDiagnosticKind::EarlyError`; and no code with `rejection_kind == LinkError` appears in any row | a boa **parse** failure classified as a link error, or a link-only code wired into the parse table — i.e. exactly the `E_MODULE_DUPLICATE_EXPORT` phase split of §0.7 |
| **P8** | `matches!(rejection_kind(c), EarlyError \| LinkError)` for all `c` in `ALL` | a code mapped to `Unsupported`/`Lowering`, which would give it `error_type() == None` and make it invisible to `compile_negative_error_matches` |
| **P9** | `rejection_kind(c).phase() != IrDiagnosticPhase::Lowering` and `rejection_kind(c).error_type().is_some()` for all `c` | the same failure as P8, caught on the derived side rather than the mapping side |

P7 spans the two crates and is the structural replacement for the doc comment at
`front/lib.rs:207-211`. It is the assertion that makes "the two tables agree" not
merely true but unstatable otherwise — because after this change there is one
table, and P7 checks the one remaining join.

### 2.6 Rejected alternative: moving the code *into* `IrDiagnosticKind`

The tightest possible shape is

```rust
pub enum IrDiagnosticKind { EarlyError(EarlyErrorCode), LinkError(EarlyErrorCode), Unsupported, Lowering }
```

which would make an inconsistent `(kind, code)` pair not merely unwritable but
non-existent. It is **rejected for this lane**, on measured grounds: the four
unit variants are matched by name in three files outside this lane's ownership —

- `crates/lila-aot-wasm/src/emit.rs:313-318`
- `crates/lila-ir/src/ir.rs:2248-2255`
- `crates/lila-ir/src/lowering.rs:391-400`

— and `lila-aot-wasm` is batch 2's crate. Keeping the four unit variants means
this lane's diff touches **only** its `files_owned` list plus the one routed
cross-lane line. The private-`code`-field design (§2.3) buys the same closure
against every constructor written outside `diagnostics.rs`; what it does not buy
is closure against a fifth constructor written *inside* `diagnostics.rs`, and
that residue is ledger entry **L3**.

---

## 3. Type mapping: invariant → construct

| # | Invariant | Construct | Where |
|---|---|---|---|
| I1 | The set of early-error conditions is closed (clause 17) | `enum EarlyErrorCode`, 18 variants, `#[repr(u8)]`, no catch-all at any consumer | `front/early_error_code.rs` |
| I2 | Each condition has exactly one wire name | `const fn wire_name` is the sole producer; P4 + P5 | same |
| I3 | Wire name and code are in bijection | `from_wire_name` + P3 round trip | same |
| I4 | One boa message shape maps to one code, on every parse path | `const PARSE_FAILURE_RULES`, 15 rows, one table, one `classify_parse_failure` | same |
| I5 | The table is order-independent and non-shadowing | `witnesses` column + P2 | same |
| I6 | A rule cannot match everything | P1 | same |
| I7 | The dependency-path message prefix is inert | `MODULE_REPARSE_PREFIX` + P6 | same |
| I8 | Every code names a rejection, never a compiler gap | `rejection_kind` returns only `EarlyError`/`LinkError`; P8, P9 | `ir/early_error_code.rs` |
| I9 | A parse-detected condition is reported at parse phase (16.1.4/16.2.1.6.1) | P7 | spans both crates |
| I10 | Phase is a function of the producing stage, not a field | `phase` field **deleted**; `IrDiagnosticKind::phase()` | `ir/diagnostics.rs` |
| I11 | A pre-evaluation rejection is a `SyntaxError` | `error_type` field **deleted**; `IrDiagnosticKind::error_type() -> Option<NativeErrorKind>` | same |
| I12 | `kind` and `code` cannot disagree | `code` is a private field; `rejected` is the only writer; struct literals elsewhere are E0451/E0063 | same |
| I13 | Front's code domain is closed too | `enum ParseCode` with `Early(EarlyErrorCode)`; `kind`/`phase`/`error_type` fields deleted | `front/lib.rs` |
| I14 | A test cannot mint a code no producer emits | `E_TEST_EARLY` deleted; the test names an `EarlyErrorCode` | `engine/lib.rs:4112` |

### 3.1 Runtime-checked ledger

These are the only places where a test, not the compiler, remains load-bearing.
Each entry states why a type cannot carry it.

- **L1 — boa's message wording.** `fragments` and `witnesses` are strings compared
  against another crate's `format!` output. No Rust type can require that
  `boa_parser` keeps emitting a given byte sequence. If boa rewords a message,
  the row goes dead silently and the affected test262 cases reclassify from
  `EarlyError` to `Unsupported`/`Malformed` — a taxonomy regression, and for the
  dependency path a conformance regression. **The type buys single-sourcing and
  exhaustiveness. It does not buy oracle robustness.** Mitigation, not cure: the
  `witnesses` column keeps the expected byte strings beside the fragments they are
  supposed to select, in one place, so a `vendor/` bump has one file to re-read.
  Detected only by running the corpus in §7.

  **Extended at DISCREPANCY-FIXER stage: the oracle is also *injectable by user
  source text*, which this entry did not cover.** boa renders a
  `TokenKind::StringLiteral` as its raw contents
  (`boa_parser/src/lexer/token.rs:313`), and `Error::Unexpected` /
  `Error::Expected` interpolate the found token into their `Display`
  (`boa_parser/src/error/mod.rs:182-220`). So

  ```js
  var x = "illegal break statement" "y";
  ```

  produces a message containing a row's whole fragment set verbatim, and the
  classifier answered `Some(IllegalBreak)` for an ordinary syntax error. On the
  entry path that is taxonomy-only (both report `parse`/`SyntaxError`); on the
  dependency path it converts an `IrDiagnostic::unsupported` into a spec
  rejection — a compiler gap wearing a spec claim — and, now that the engine
  short-circuits on any coded diagnostic, one that ends compilation earlier.

  **Mitigated, not cured:** `classify_parse_failure` returns `None` early for the
  two interpolating message shapes (`INTERPOLATING_MESSAGE_SHAPES`:
  `"unexpected token '"`, `"expected token '"`, `"expected one of "`), detected by
  substring because the dependency path prepends a prefix. Assertion **P10**
  proves the guard eats no witness of any row. An `Error::General` wording that
  interpolates user text reopens that channel for an anywhere-substring rule.

  **2026-08-23 closure for duplicate import-attribute keys:** Boa's
  local-export error interpolates the exported name, which can contain the
  complete fixed text of this condition. The message-pattern table now has a
  closed `StartsWith` variant alongside `ContainsAll`; the duplicate static
  import-attribute-key row uses it. Real-source witnesses pin both the forged
  local-export diagnostic and an overlapping duplicate-export name. This
  closes that one injection path only: current and future `ContainsAll` rows
  remain unaudited for arbitrary `Error::General` interpolation and each needs
  its own match-semantics review.

  Scanned at the same time, for completeness: all 1077 multi-word string literals
  in `boa_parser-0.21.1` and `boa_ast-0.21.1` against the 15 rows — exactly 28
  producer sites classify, every one to the intended code, and zero literals
  match two rows. The table has no false positive from boa's *own* corpus.
- **L2 — `"SyntaxError"` in `lila-front`.** `ParseCode::error_type` returns a
  `&'static str` literal because `NativeErrorKind` lives in `lila-ir`, which
  `lila-front` cannot name (§0.1). The literal count in `lila-front` goes
  from 3 to 1 and it is no longer at any call site, but it is still a string.
  Closing this requires moving `NativeErrorKind` below `lila-ir`, which is
  `closed-name-domains.md`'s file and another lane's decision.
- **L3 — a fifth constructor inside `diagnostics.rs`.** The private `code` field
  stops struct literals everywhere else in the workspace. Someone adding a fifth
  constructor *in that file* could still write
  `Self { kind: IrDiagnosticKind::LinkError, code: Some(EarlyErrorCode::DuplicateLabel), .. }`.
  Closing this needs the payload-carrying `IrDiagnosticKind` of §2.6, which is
  out of this lane's file ownership. The blast radius is one ~100-line file whose
  doc comment says so.
- **L4 — two implementation limits claiming `SyntaxError`.**
  `ModuleUnsupportedPhase` and `ModuleTooManyUnits` are not spec conditions
  (§1.3) yet map to `LinkError`, hence `SyntaxError`/`Resolution`. This is the
  same category error that `module_parse_failure_diagnostic` deliberately avoids
  for unmodelled parse failures. Note this entry is no longer moot: before
  DISCREPANCY-FIXER stage a `LinkError` never reached the conformance matcher at
  all (see L10), so nothing observed the claim. It does now.
  **Recorded, not fixed:** the brief puts the
  semantics of link failures out of scope, and fixing it changes which
  diagnostics reach the backend (`emit.rs:313-318` treats `LinkError` and
  `Unsupported` alike, but `engine/lib.rs:1948-1952` does not). A future lane
  owning `graph.rs` and `emit.rs` should split them out.
- **L5 — reachability of a row.** Nothing makes a `PARSE_FAILURE_RULES` row with
  no living producer fail to build. Enforced by review of the `file:line`
  provenance column, not by the compiler.

  **First confirmed instance, DISCREPANCY-FIXER stage: row 6**
  (`duplicate lexical declaration`). §0.5 originally claimed it was live from
  both parse goals; DR-6 measured it dead through both of this crate's entry
  points, and §0.5 now carries the measurement. The row is retained under DR-6's
  own rule and because boa's third producer of the wording
  (`boa_ast/src/scope_analyzer.rs:2364`) would classify correctly if an eval path
  is ever added. This is what the entry predicted, arriving on the row it named.
- **L7 — `IrDiagnostic::kind` is still a public field (added at ENCODER stage).**
  `code` became private, which closes every *construction* path: a struct literal
  outside `diagnostics.rs` is `error[E0451]`, so `kind` and `code` cannot be
  paired inconsistently at birth. It does **not** close *mutation*: given a
  `&mut IrDiagnostic`, `d.kind = IrDiagnosticKind::Unsupported;` compiles and
  desynchronises `kind` from `code`, and therefore `phase()`/`error_type()` from
  the condition. Making `kind` private needs a `kind()` accessor at four read
  sites, two of which — `crates/lila-aot-wasm/src/emit.rs:313-318` (batch 2's
  crate, excluded) and `crates/lila-ir/src/ir.rs:2248-2255` — are outside this
  lane's `files_owned`. Measured: zero `&mut IrDiagnostic` bindings exist in the
  workspace today, so nothing exploits it. Close it in the same patch as §2.6's
  payload-carrying `IrDiagnosticKind`, which removes the field entirely.
- **L8 — one item is private where §2.1 wrote `pub` (ENCODER deviation; halved at
  DISCREPANCY-FIXER stage).** `MODULE_REPARSE_PREFIX` is now `pub`, because L6's
  closure gave it the second real consumer this entry named as the condition for
  promotion. `EarlyErrorCode::from_wire_name` (§2.1.1) remains **private**.
  Original statement, for the item that is still open: Reason: neither has
  a product call site — `from_wire_name`'s only consumer is assertion P3 and
  `MODULE_REPARSE_PREFIX`'s only consumer is assertion P6, both in the same
  module. `native_error.rs`'s precedent does not carry over: its `from_str` is
  `pub` because `lowering.rs:34144` calls it. A `pub` item with no call site is
  the "survival by `pub`" shape AGENTS.md names, and `native_error.rs`'s own
  module docs record what it cost here. Both should become `pub` in the same
  patch that gives them a consumer, not before. `EarlyErrorCode::is_parse_classified`
  *is* `pub` — added at ENCODER stage — because P7 lives in the other crate and
  the table stays private; that is a real cross-crate consumer.
- **L6 — the dependency-path message prefix existed in three places. CLOSED at
  DISCREPANCY-FIXER stage.** `MODULE_REPARSE_PREFIX`, the live literal at
  `crates/lila-ir/src/modules/record.rs:1343`, and a *third* copy
  (`const PREFIX: &str = "lowering module reparse failed: ";`) inside the
  `modules/early.rs` test that is P6's runtime half — so a drift in `record.rs`
  would have left both checks guarding a string no producer emits, which is
  exactly the failure this entry was written to bound. `MODULE_REPARSE_PREFIX` is
  now `pub`, `record.rs` writes `format!("{MODULE_REPARSE_PREFIX}{err}")`, and
  the `early.rs` tests name the constant. One spelling, three consumers.

- **L10 — the derived `(LinkError, Resolution, SyntaxError)` triple reached
  nothing. CLOSED at DISCREPANCY-FIXER stage; behaviour change, needs a
  pass-count report.** `Engine::compile_on_current_thread` converted an
  `IrDiagnostic` into an `EngineError` carrying `ir_diagnostic` only when
  `diagnostic.kind == IrDiagnosticKind::EarlyError`. A `LinkError` fell through,
  reached `lila_aot_wasm::emit` (`emit.rs:312-321`), was flattened to
  `EmitError::unsupported(message)`, and arrived with `ir_diagnostic: None` —
  so `compile_negative_error_matches`'s `IrDiagnosticPhase::Resolution` **and**
  `IrDiagnosticPhase::Lowering` arms were both unreachable, and DR-5/DR-11 could
  not be discharged end to end. Compounding it,
  `case_has_compile_only_negative` returned true only for phase `parse`/`early`,
  so a `resolution` negative never entered the compile-only path and fell to the
  `detail.contains(&negative.error_type)` fallback against a message reading
  `module ./dep.js does not export nope`, which contains no error name at all.
  All 34 `resolution/SyntaxError` cases in the pinned suite failed on the
  wasm-aot backend.

  Two edits, both inside the retrofit map (§6.12, §6.13): the engine's find
  predicate is now `diagnostic.code().is_some()` — the same predicate
  `lila-test262` already used, and the one that survives §2.6's
  payload-carrying `IrDiagnosticKind` — and the negative-phase domain became a
  closed `NegativePhase` enum whose `is_compile_only()` is an exhaustive match
  admitting `Resolution`. Kept in the ledger rather than deleted because the
  *pass-count delta* is a runtime fact this contract cannot assert: report the
  `resolution` family separately from B1/B2 per §8 rung 6.

- **L11 — a compiler gap claimed a spec error type, and produced false test262
  passes. CLOSED at DISCREPANCY-FIXER stage; behaviour change outside B1-B4,
  needs its own B-row and a pass-count report.** `ParseCode::error_type` returned
  `"SyntaxError"` for `UnsupportedParserFeature`, which is the caught-panic case
  (`front/lib.rs`: "parser aborted while handling source"). Because
  `ParseDiagnosticPhase::Parse` satisfies `negative.phase == "parse"` and
  `"SyntaxError"` satisfied `negative.error_type`,
  `compile_negative_error_matches` scored a **PASS** for any `parse/SyntaxError`
  negative whose source merely crashed boa's parser — regardless of whether
  ECMAScript rejects the program. That is precisely the category error
  `IrDiagnosticKind::error_type` returns `None` for and that
  `module_parse_failure_diagnostic`'s doc comment forbids in words, and it is
  clause 17's "must not treat other kinds of error as early errors". The
  behaviour was pre-existing (verified at `84e782506`) and §2.4 preserved it
  under B0, but no ledger entry covered it and invariant I8 was enforced by
  P8/P9 for `EarlyErrorCode` and silently dropped for `ParseCode`.
  `ParseCode::error_type` now returns `Option<&'static str>` with `None` for that
  variant, and `compile_negative_error_matches` requires `Some`.

---

## 4. The measured drift and its closure

Exactly these behaviour changes are permitted, and they are required. Everything
else in the retrofit is behaviour-preserving by construction.

**B0 — not a change.** Nothing about *which programs are rejected* changes. No
new early error is detected. Every change below is about which `(code, kind,
phase, error_type)` a rejection is reported under.

| id | today | after | why |
|---|---|---|---|
| **B1** | `({__proto__: null, __proto__: {}})` inside a **dependency** module → `Unsupported`, `code: None`, `error_type: None` → test262 **fails** | `EarlyErrorCode::ObjectDuplicateProto`, `EarlyError`, `Early`, `SyntaxError` | row 1 of the one table; today `E_OBJECT_DUPLICATE_PROTO` has no rule in `PARSE_FAILURE_RULES` at all |
| **B2** | W3/W4/W5 lexical redeclaration in a **dependency** module → `Unsupported` → test262 **fails** | `DuplicateLexicalDeclaration`, `EarlyError`, `Early`, `SyntaxError` | rows 5 and 6 of the one table |
| **B3** | W2 (module-goal lexical redeclaration) in an **entry** file → `ParseCode::Malformed` (`P_PARSE_MALFORMED`) | `ParseCode::Early(DuplicateLexicalDeclaration)` | row 4 of the one table; front misses W2 today because W2 has `name \`x\`` and not `names` |
| **B4** | `ModuleLinkErrorIr::DuplicateExport` → `LinkError`, `Resolution` (latent; unreachable behind `record.rs:740`) | `EarlyError`, `Early` | forced by P7; 16.2.3.1 makes it an early error and `early-dup-export-id.js` is `phase: parse` |

B1 and B2 are conformance-visible (a failing case becomes passing). B3 and B4 are
taxonomy-visible only — the case passes either way because
`ParseDiagnosticPhase::Parse` and `ParseCode::Malformed`'s `"SyntaxError"` still
satisfy `phase: parse, type: SyntaxError`. They are still defects: the failure
taxonomy is the thing the backlog is built from.

**Non-change, proven.** Replacing front's loose
`"lexical" && "declared" && "names"` alternative (`front/lib.rs:237-239`) with
precise rows 5 and 7 classifies **exactly the same message set**. Measured: the
complete list of `boa_parser`/`boa_ast` message literals containing `lexical` is
20 strings; of those, the ones containing all three of `lexical`, `declared` and
`names` are exactly

- `lexical name declared in var names` → row 5
- `lexical name declared in var declared names` → row 5
- `` formal parameter `{}` declared in lexically declared names `` → row 7

All three keep the same code. (`'let' is disallowed as a lexically bound name`,
`a lexical declaration in the head of a {loop_type} loop can't have an
initializer` and `variable declaration {} in eval function already exists as a
lexical variable` fail the `declared` test and were never matched.) Row 7 is
therefore not a new rule: it makes today's accidental match deliberate, and it is
spec-defensible independently (15.2.1 is a lexical-redeclaration rule).

---

## 5. Mistake-class table

| id | Mistake | Today | After | Error |
|---|---|---|---|---|
| MC1 | Add an early-error rule to one table and not the other | happens silently; **has happened twice and is live** (§0.3, §0.4) | there is no second table; `early.rs:107-170` is deleted and `lila_ir` calls `lila_front::classify_parse_failure` | unrepresentable — deletion, not a diagnostic |
| MC1′ | Add a row that shadows an existing row, or is shadowed by one | silent misclassification; the disjointness claim is a comment (`early.rs:114`) | `witnesses` + **P2** | `error: evaluation of constant value failed` at the `const _: () = assert!(witnesses_select_their_own_row(), …)` in `early_error_code.rs` |
| MC1″ | Add a row with an empty `fragments` list | matches every message; every parse failure collapses to one code | **P1** | same const-assert failure, distinct message |
| MC2 | Mint a code as a fresh string literal | compiles; 51 tokens over 4 crates; typo = unmatched arm = silent misclassification | there is no `&'static str` code anywhere; `EarlyErrorCode` has no `FromStr`/`Display` | `error[E0308]: expected \`EarlyErrorCode\`, found \`&str\`` at any construction site |
| MC2′ | Misspell an existing code at a comparison site (`test262/src/lib.rs:21523`, `engine/src/lib.rs:4131`) | compiles; silently never matches | variant path | `error[E0599]: no variant or associated item named \`ObjectDuplicateProtoo\` found for enum \`EarlyErrorCode\`` |
| MC2″ | Add a 19th code and forget to classify it | not possible today (no domain) | `rejection_kind`'s exhaustive match | `error[E0004]: non-exhaustive patterns: \`EarlyErrorCode::NewThing\` not covered` in `ir/early_error_code.rs`; and `error[E0308]` on `ALL: [EarlyErrorCode; 18]` |
| MC3 | Write `"SyntaxError"` — or a misspelling — at a diagnostic construction site | compiles; `NativeErrorKind` exists and is bypassed (measured: zero references in `diagnostics.rs` or `modules/*.rs`) | `error_type` is not a field and not a parameter; `IrDiagnosticKind::error_type()` is the sole producer, returning `NativeErrorKind` | `error[E0061]: this function takes 3 arguments but 4 arguments were supplied` on `IrDiagnostic::rejected`; and `error[E0599]` on a misspelled `NativeErrorKind::SyntaxErrror` |
| MC4 | Report one code under two phases from two paths | four independent fields; held only by convention (measured: zero `IrDiagnostic { … }` literals outside `diagnostics.rs`); **already latent for `E_MODULE_DUPLICATE_EXPORT`** (§0.7) | `phase` is not a field; `code` is private; `rejected` is the only coded constructor; **P7** ties the parse table to `rejection_kind` | `error[E0609]: no field \`phase\` on type \`IrDiagnostic\``; `error[E0451]: field \`code\` of struct \`IrDiagnostic\` is private` for a struct literal outside the module; const-assert failure for a P7 violation |
| MC4′ | A fifth constructor added *inside* `diagnostics.rs` pairing a `LinkError` kind with a parse-phase code | possible | still possible — **ledger L3** | none; review only |
| MC4″ | Mutating `kind` on an existing `&mut IrDiagnostic` so it disagrees with `code` | possible | still possible — **ledger L7**, added at ENCODER stage | none; review only. Measured: zero `&mut IrDiagnostic` bindings in the workspace |

**ENCODER-stage status.** MC1, MC1′, MC1″, MC2, MC2′, MC2″, MC3 and MC4 are
discharged as written: each is now the compile error named in the last column.
MC4′ and MC4″ are the two residues, both recorded above and both needing §2.6's
payload-carrying `IrDiagnosticKind`, which is out of this lane's file ownership.
No type in this lane is decoration: every construct in §3's mapping table has at
least one mistake class that fails to build without it.

**DISCREPANCY-FIXER correction to that status.** MC4 was discharged on the
`lila-ir` side only. The *call-site* half was still convention in both
directions — `ParseError::early_error(EarlyErrorCode::ModuleMissingExport, …)`
and `IrDiagnostic::rejected(EarlyErrorCode::ModuleMissingExport, …)` inside
`modules/early.rs` both compiled, each reporting one code under the wrong phase
from a second path. `ParseClassified` (§2.4.1) makes both `error[E0308]`, and
`IrDiagnostic::rejected_at_parse` is the narrow door for parse-stage producers.
By AGENTS.md's own standard `is_parse_classified` + P7 were load-bearing for the
table and decorative for the call sites; they are now load-bearing for both.

Two further mistake classes were added to the table by the dry run and are now
closed: **MC5** (a compiler gap claiming a spec error type — ledger L11) and
**MC6** (user source text injected into the string oracle — ledger L1). Neither
is a *compile* error; both are recorded in the ledger with their mechanism, which
is the honest classification.

---

## 6. Retrofit map

Strictly ordered. Each step leaves the tree in a state where the next step's
errors are attributable. `cargo check -p lila-front` after step 3;
`cargo check -p lila-ir` after step 8; `cargo check --workspace` after step 11.

### 6.1 NEW `crates/lila-front/src/early_error_code.rs`

The whole of §2.1 plus const assertions P1–P6. Nothing else; this file must not
name any `lila-ir` type.

### 6.2 `crates/lila-front/src/lib.rs` — module wiring

Add `mod early_error_code;` and
`pub use early_error_code::{classify_parse_failure, EarlyErrorCode, MODULE_REPARSE_PREFIX};`.

### 6.3 `crates/lila-front/src/lib.rs:40-65` — `ParseCode` and `ParseDiagnostic`

Add `enum ParseCode` (§2.4). Delete `ParseDiagnostic`'s `kind`, `phase` and
`error_type` fields; retype `code` to `ParseCode`; add the three accessors.
`ParseDiagnosticKind` and `ParseDiagnosticPhase` keep their definitions — they
are now return types, not fields.

### 6.4 `crates/lila-front/src/lib.rs:73-128` — the three constructors

`malformed` → `code: ParseCode::Malformed`. `unsupported_parser_feature` →
`ParseCode::UnsupportedParserFeature`. `early_error(code, error_type, message,
span)` → `early_error(code: EarlyErrorCode, message, span)`, storing
`ParseCode::Early(code)`. The two `"P_PARSE_*"` literals and the three
`"SyntaxError"` literals leave the constructors and become `ParseCode`'s three
`const fn` bodies (2 + 1 literals).

### 6.5 `crates/lila-front/src/lib.rs:181-185` and `:205-256` — the classifier

`parser_static_semantics_error_code` is **deleted in full** (52 lines, 17 `E_`
tokens, including the loose alternative at `:237-239` and the doc comment at
`:205-211` whose requirement is now structural). Line 181 becomes:

```rust
return if let Some(code) = classify_parse_failure(&err) {
    Err(ParseError::early_error(code, message, span))
} else {
    Err(ParseError::malformed(message, span))
};
```

Note the argument is `&err`, the **bare** boa message — front does not apply the
`"parse error: "` prefix before classifying (`lib.rs:178-181`), and must not
start.

### 6.6 `crates/lila-front/src/lib.rs` tests — 5 sites

`:356`, `:380`, `:448` → `ParseCode::Malformed`. `:396` →
`ParseCode::Early(EarlyErrorCode::ObjectDuplicateProto)`. `:408`, `:413`, `:421`,
`:429` → the corresponding `ParseCode::Early(...)`. Add one test asserting
**B3**: a module-goal `let x; const x;` classifies as
`ParseCode::Early(EarlyErrorCode::DuplicateLexicalDeclaration)` and no longer as
`Malformed`.

### 6.7 NEW `crates/lila-ir/src/early_error_code.rs`

§2.2 plus const assertions P7–P9.

### 6.8 `crates/lila-ir/src/lib.rs`

- Add `mod early_error_code;` beside `mod early_errors;` (line 60-ish) —
  **note the two names are unrelated**; `early_errors.rs` is derived-constructor
  validation over 76 `ExprIr::` arms and belongs to the Reference-Records lane.
  Consider a comment saying so at the `mod` lines, because the adjacency is a
  trap.
- Add `pub use early_error_code::EarlyErrorCode;` next to the existing
  `pub use native_error::NativeErrorKind;` (line 136).
- `:1185` → `.all(|d| d.code() != Some(EarlyErrorCode::ObjectDuplicateProto))`.

### 6.9 `crates/lila-ir/src/diagnostics.rs`

The whole of §2.3. `early_error` and `link_error` are deleted; `rejected` replaces
both. The `"SyntaxError"` literal at `:68` disappears.

### 6.10 `crates/lila-ir/src/modules/early.rs`

- Delete `struct ParseFailureRule` and `const PARSE_FAILURE_RULES`
  (`:107-170`, 64 lines, 11 `E_` tokens). Replace the doc comment at `:98-106`
  with a pointer to `lila_front::early_error_code`.
- `:49-54` and `:83-92` → `IrDiagnostic::rejected(EarlyErrorCode::ModuleDuplicateExport, …)`
  and `IrDiagnostic::rejected(EarlyErrorCode::ModuleUndeclaredExport, …)`. The
  first currently reads `error.code()` from a `ModuleLinkErrorIr`; name the code
  directly — the round trip through a link error was never meaningful.
- `module_parse_failure_diagnostic` becomes:
  ```rust
  pub(crate) fn module_parse_failure_diagnostic(message: &str) -> IrDiagnostic {
      match lila_front::classify_parse_failure(message) {
          Some(code) => IrDiagnostic::rejected(code, message, None),
          None => IrDiagnostic::unsupported(message),
      }
  }
  ```
  Its doc comment keeps the "claiming `SyntaxError` for a source we simply failed
  to read would turn a compiler gap into a spec claim" paragraph verbatim; it is
  the reason `None` must not become a code.
- Tests `:260-263`, `:291-292`, `:328-367`, `:376-377` → `.code()`, `.phase()`,
  `.error_type()`, `EarlyErrorCode::*`, `Some(NativeErrorKind::SyntaxError)`.
  Extend the `boa_static_semantics_messages_classify_as_syntax_errors` case list
  to all 15 rows' witnesses under the `MODULE_REPARSE_PREFIX`. Add the **B1** and
  **B2** cases explicitly.

### 6.11 `crates/lila-ir/src/modules/graph.rs`

- `:174-186` `pub const fn code(&self) -> &'static str` → `-> EarlyErrorCode`,
  seven arms returning variants. Keep the arm order.
- `:226` `IrDiagnostic::link_error(self.code(), self.message())` →
  `IrDiagnostic::rejected(self.code(), self.message(), None)`. This is where B4
  lands.
- `:1783` → `d.code() == Some(EarlyErrorCode::ModuleTooManyUnits)`.

### 6.12 `crates/lila-engine/src/lib.rs` — tests only

- `:4103-4105` → `diagnostic.phase()`, `diagnostic.error_type()`,
  `diagnostic.code == ParseCode::Malformed`.
- `:4112` → `IrDiagnostic::rejected(EarlyErrorCode::ObjectDuplicateProto, "early error: test", None)`;
  `"E_TEST_EARLY"` is deleted (§0.2).
- `:4131-4133` → `ParseCode::Early(EarlyErrorCode::ObjectDuplicateProto)`,
  `diagnostic.phase()`, `diagnostic.error_type()`.
- `:1948-1952` (`kind == IrDiagnosticKind::EarlyError`) is **untouched** — unit
  variant, unchanged.

### 6.13 `crates/lila-test262/src/lib.rs`

- `:21477-21486` — `diagnostic.phase` → `diagnostic.phase()`; the match arms are
  unchanged (`ParseDiagnosticPhase` is unchanged).
- `:21491-21501` — `diagnostic.phase` → `diagnostic.phase()`.
- `:21501-21506` — `matches!(diagnostic.kind, EarlyError | LinkError)` may stay
  as-is (unit variants preserved). Preferred: `diagnostic.code().is_some()`,
  which is the same predicate stated once and survives §2.6's upgrade.
- `:21507-21509` — the error-type comparison becomes
  ```rust
  && (negative.error_type.is_empty()
      || diagnostic.error_type().is_some_and(|k| k.as_str() == negative.error_type))
  ```
  **Do not** write `NativeErrorKind::from_str(&negative.error_type) == diagnostic.error_type()`:
  a `negative.error_type` outside the nine-name domain (e.g. `Test262Error`)
  parses to `None`, which would then compare equal to a `None` `error_type` and
  turn an unmatched expectation into a match.
- `:21519` — `diagnostic.code` → `diagnostic.code.wire_name()`,
  `diagnostic.error_type` → `diagnostic.code.error_type()`.
- `:21523-21524` —
  ```rust
  let code = diagnostic.code().map_or("E_IR_DIAGNOSTIC", EarlyErrorCode::wire_name);
  let error_type = diagnostic.error_type().map_or("Error", NativeErrorKind::as_str);
  ```
  These two literals are the *absence* placeholders and stay literals (§0.2).
- `:25776` (`failure.detail.contains("P_PARSE_MALFORMED")`) — **untouched**; the
  detail is still built from `wire_name()`.

### 6.14 The one cross-lane edit, routed

`crates/lila-ir/src/lowering.rs:28135-28141`:

```rust
self.diagnostics.push(IrDiagnostic::early_error(
    "E_OBJECT_DUPLICATE_PROTO",
    "SyntaxError",
    "early error: duplicate __proto__ prototype setter in object literal",
    None,
));
```
becomes
```rust
self.diagnostics.push(IrDiagnostic::rejected(
    EarlyErrorCode::ObjectDuplicateProto,
    "early error: duplicate __proto__ prototype setter in object literal",
    None,
));
```

`lowering.rs` belongs to the Reference-Records lane. This is the single line
agreed in the lane note. Nothing else in `lowering.rs` is touched —
`:391-400` matches unit variants and is unaffected.

**Status: APPLIED at DISCREPANCY-FIXER stage.** It had *not* been applied at
ENCODER stage, and the omission was not cosmetic: `IrDiagnostic::early_error` no
longer exists, so `cargo check -p lila-ir` failed with
`error[E0599]: no function or associated item named \`early_error\` found for
struct \`IrDiagnostic\``. Consequently **assertions P7-P9 had never been
evaluated by rustc**, and §6.16's token accounting was wrong: `lila-ir`
carried 1 `"E_..."` literal and 1 production `"SyntaxError"` literal outside the
two spelling authorities, not 0. Both are now 0 (re-counted, §6.16).

### 6.15 What stays untouched, verified by reading

| file:line | why it does not change |
|---|---|
| `crates/lila-aot-wasm/src/emit.rs:313-318` | matches `IrDiagnosticKind` unit variants only |
| `crates/lila-ir/src/ir.rs:2248-2255` | same |
| `crates/lila-ir/src/lowering.rs:391-400` | same |
| `crates/lila-engine/src/lib.rs:1948-1952` | same |
| `crates/lila-ir/src/modules/record.rs` | calls `module_parse_failure_diagnostic` and `module_early_errors`; neither signature changes. Its `"lowering module reparse failed: "` literal at `:1343` is duplicated by `MODULE_REPARSE_PREFIX` — see **L6** below |
| `crates/lila-ir/src/modules/{link,namespace,dynamic}.rs` | construct `IrDiagnostic` only through `unsupported`/`lowering` |
| `crates/lila-ir/src/native_error.rs` | round 1's file; read, not edited |
| `crates/lila-ir/src/early_errors.rs` | despite the name, derived-constructor validation over 76 `ExprIr::` arms; Reference-Records lane |
| `crates/lila-cli/**` | measured: zero `"E_..."` and zero `IrDiagnostic` references |
| `crates/lila-spec-exec/**` | measured: zero `IrDiagnostic` references |

**Ledger addition L6** — `MODULE_REPARSE_PREFIX` and the literal at
`record.rs:1343` are two copies of one string, because `record.rs` is not in this
lane's `files_owned`. P6 still checks the property for the constant as written.
The one-line single-sourcing (`format!("{MODULE_REPARSE_PREFIX}{err}")`) is
deferred to whoever owns `record.rs`.

### 6.16 Token accounting

| file | `"E_..."` tokens before | after |
|---|---|---|
| `lila-ir/src/modules/early.rs` | 21 | 0 |
| `lila-front/src/lib.rs` | 17 | 0 |
| `lila-ir/src/modules/graph.rs` | 8 | 0 |
| `lila-engine/src/lib.rs` | 2 | 0 |
| `lila-test262/src/lib.rs` | 1 | 0 — the placeholder moved to `lila-front` as `NO_EARLY_ERROR_CODE` (P5'), named here, not spelled |
| `lila-ir/src/lib.rs` | 1 | 0 |
| `lila-ir/src/lowering.rs` | 1 | 0 |
| `lila-front/src/early_error_code.rs` (new) | — | 19 (18 in `wire_name`, the sole producer, + `NO_EARLY_ERROR_CODE`, the absence placeholder, proved distinct from all 18 by P5') |
| **total** | **51 across 7 files** | **19 in 1 file** |

**Re-counted at DISCREPANCY-FIXER stage**, after §6.14 was actually applied. The
ENCODER-stage figures counted the intended state, not the tree: `lowering.rs`
still held `"E_OBJECT_DUPLICATE_PROTO"` and `"SyntaxError"`, so the true "after"
was 20 tokens across 3 files. It is now 19 across 1.

`"SyntaxError"` literals: `lila-ir` goes from 12 (`diagnostics.rs:68`,
`early.rs` ×11) to **0**; `lila-front` goes from 3 to **1** (ledger L2).
`"P_PARSE_*"` literals go from 2 producers + 5 comparison sites to **2**, both
inside `ParseCode::wire_name`.

---

## 7. Dry-run corpus: what each trace must establish

Symbolic execution against the code as it will be after §6, on paper. Each trace
starts from the source file, goes through boa's actual message (cited above),
through `classify_parse_failure`, through `rejection_kind`, into
`(kind, phase, code, error_type)`, and ends at
`compile_negative_error_matches`. Every one of these ten files was opened and its
`negative:` frontmatter verified as `phase: parse, type: SyntaxError`.

| id | source | must establish |
|---|---|---|
| **DR-1** | `language/expressions/object/__proto__-duplicate.js` | Row 1 fires. Then **the adversarial triple**: the same body as (i) a script, (ii) a module entry, (iii) `dep.js` imported by a trivial module entry. All three must reach `ObjectDuplicateProto`/`SyntaxError`/parse. Show that (i) and (ii) go through `lila_front::parse` (`engine/lib.rs:1913-1923`) and (iii) through `record.rs:1343` → `module_parse_failure_diagnostic`, and that both now consult the same table. **This is the acceptance trace for B1.** Confirm the brief's claim about path (ii) is wrong (§0.3) or produce the counter-trace. |
| **DR-2** | `language/module-code/early-dup-lex.js` (`let x; const x;`) | As an entry: `ModuleParser::parse` (`boa_parser/src/parser/mod.rs:507-517`) emits W2. Show that under §6 it now hits row 4 and yields `ParseCode::Early(DuplicateLexicalDeclaration)`, where today it yields `Malformed`. **Acceptance trace for B3.** As a dependency: row 4 again, same code. State plainly that the two paths now agree, and that they did not before. |
| **DR-3** | `language/module-code/early-lex-and-var.js` (`let x; var x;`) | Which boa check fires for the *module* goal — `mod.rs:521-531` (W2) — and which for a *script* — `mod.rs:372-379` (W1) or the block/switch forms (W3/W4). Then show W1 and W2 both hit row 4 and W3/W4 both hit row 5, one code. This is the probe that decides whether the loose `"lexical"+"declared"+"names"` fallback was load-bearing; §4's non-change proof predicts **not** and must be checked, not assumed. |
| **DR-4** | `language/module-code/early-dup-export-id.js` | Row 2. Then follow the *other* producer: `modules::early::module_early_errors` `:44-55` for a dependency, and `graph.rs:783-788` for the (currently unreachable) link path. Show `rejection_kind(ModuleDuplicateExport) == EarlyError` makes both yield phase `Early`, and that P7 is what forces it. **Acceptance trace for B4.** Also show *why* the graph path is unreachable today (`record.rs:740-743` returns `Err` first) so the change is correctly labelled latent. |
| **DR-5** | `language/module-code/early-export-unresolvable.js` | Row 3 on the parse path. Then the distinct link-path code `ModuleMissingExport` (`graph.rs:836,873`) → `rejection_kind` → `LinkError` → `Resolution` → `SyntaxError`. Show the two codes are different codes for different conditions and that neither can borrow the other's phase. |
| **DR-6** | any script with a lexical redeclaration | **Settle L5/§0.5 as a refutable prediction.** `Parser::parse_script_with_source` runs `ScriptParser::parse`'s own duplicate check (`mod.rs:361-379`) *before* `analyze_scope` (`mod.rs:186`), so W1 should fire and W5 should never be reached for a plain top-level `let x; let x;`. Predict: no ordinary source reaches W5 through `lila_front::parse`, and row 6 is retained on the strength of the *reachable path existing*, not of a witness case. If the trace finds a source that does reach W5, name it; if it finds W5 unreachable in every construction tried, say so and leave row 6 in with the finding recorded — do **not** delete a row on a negative result. |
| **DR-7** | `language/module-code/early-super.js` + `early-new-target.js` | Rows 8 and 9. **Refute the brief's prefix-overlap hazard concretely**: show `"module cannot contain \`new.target\` on the top-level"` does not contain `super` and vice versa, and that P2 proves it for the whole table so first-match-wins ordering is no longer load-bearing anywhere. |
| **DR-8** | `early-undef-break.js`, `early-dup-lables.js` | Rows 12 and 11. These come from `CheckLabelsError::message` (`boa_ast/src/operations/mod.rs:1399-1417`) via `Error::lex(LexError::Syntax(...))`, a *different* boa error constructor from `Error::general`. Confirm the `Display` of both reaches the classifier as a plain `contains`-able string on both the script and module paths. |
| **DR-9** | `privatename-not-valid-earlyerr-module-1.js` | Row 10, and that `invalid private identifier usage` is emitted from three sites (`mod.rs:462`, `mod.rs:593`, `statement/mod.rs:1020`) covering script, module and statement contexts under one row. |
| **DR-10** | adversarial, MC2 | Write, on paper, `assert_eq!(diagnostic.code(), Some(EarlyErrorCode::ObjectDuplicateProtoo))` at `engine/lib.rs:4131` and a typo'd variant at `test262/src/lib.rs:21523`. Predict `error[E0599]`. Then write the same typo as a string against the *pre*-change code and confirm it compiles. A refutable prediction, not an assertion. |
| **DR-11** | adversarial, MC3/MC4 | `import { nope } from './dep.js'` where `dep.js` exports nothing named `nope`. Trace to `ModuleLinkErrorIr::MissingExport` → `rejected` → `(LinkError, Resolution, Some(NativeErrorKind::SyntaxError))`. Then attempt, on paper, to construct `(EarlyError, Resolution)` and `(LinkError, Early)`: show the first requires a `phase` field that no longer exists and the second requires a struct literal that is `error[E0451]`. Name the exact residue (L3) rather than claiming total closure. |
| **DR-12** | adversarial, table integrity | Add a 16th row with `fragments: &[]` and predict the P1 const-assert failure; add a row `{ fragments: ["module cannot contain"], code: ModuleTopLevelSuper, witnesses: [...] }` and predict the P2 failure against row 9's witness; add a row whose code is `ModuleMissingExport` and predict the P7 failure. Three distinct const-eval errors, three distinct messages. |

---

## 8. Verification ladder for the encoder

Rungs, per `docs/rust-rewrite/batch-workflow.md`. This lane is spec/IR only and
does not emit Wasm, so rung G does not apply.

1. `cargo check -p lila-front` — after §6.6. All const assertions P1–P6 are
   evaluated here; a table defect is caught before `lila-ir` is touched.
2. `cargo check -p lila-ir` — after §6.11. P7–P9 evaluate here.
3. `cargo check --workspace` — after §6.14.
4. `cargo test -p lila-front` then `cargo test -p lila-ir` — the rewritten
   tests, including the new B1/B2/B3 cases.
5. `cargo test -p lila-engine --lib` for the four diagnostic tests.
6. Rung 4 for this lane's family: `lila test262 run language/module-code` and
   `lila test262 run language/expressions/object`. B1 and B2 predict a pass-count
   *increase*; B3 and B4 predict no change in pass count and a change in the
   failure taxonomy only. Report both, separately.

Do not run rung 1c or the sweep from this lane.

---

## 9. Summary of the claim

Before: two fragment tables that had already drifted twice, 51 `E_...` string
tokens across 4 crates, a `NativeErrorKind` domain that `diagnostics.rs` did not
reference once, and a four-field `(kind, phase, code, error_type)` product held
consistent only by there being exactly four constructors.

After: one closed 18-variant `EarlyErrorCode` in the one crate both parse paths
can name, one 15-row fragment table whose disjointness and prefix-inertness are
const assertions rather than comments, `phase` and `error_type` deleted as fields
and derived from the producing stage as 16.1.4 and 16.2.1.6.1 require, `code`
private so that no fifth constructor can be written outside the file that owns
the derivation, and four measured drifts closed — two of them conformance-visible.

What it does not buy, stated plainly: boa's message wording stays a fragile
oracle (L1), `lila-front` keeps one `"SyntaxError"` literal it cannot type
(L2), a fifth constructor inside `diagnostics.rs` can still pair badly (L3), two
implementation limits still claim `SyntaxError` (L4), and a dead table row still
compiles (L5). Those five are the whole of what tests remain load-bearing for in
this area.
