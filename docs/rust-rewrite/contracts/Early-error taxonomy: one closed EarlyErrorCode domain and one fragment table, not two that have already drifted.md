# Contract: early-error taxonomy — one closed `EarlyErrorCode` domain and one fragment table

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

### 0.1 `EarlyErrorCode` **cannot** live in `crates/porffor-ir` — it would be a dependency cycle

The brief's scope item (a) puts the enum in `crates/porffor-ir/src/early_error_code.rs`
and has `porffor_front::parser_static_semantics_error_code` consume it. Measured
from `Cargo.toml`:

```
porffor-front  deps: boa_ast, boa_interner, boa_parser        (no porffor-ir)
porffor-ir     deps: ..., porffor-front                        (porffor-ir → porffor-front)
porffor-engine deps: ..., porffor-front, porffor-ir
porffor-test262 deps: porffor-engine, porffor-front, porffor-ir
```

`porffor-ir` depends on `porffor-front` (`crates/porffor-ir/src/diagnostics.rs:1`
is `use porffor_front::SourceSpan;`). A type in `porffor-ir` therefore cannot be
named by `porffor-front`.

Both crates independently call boa's parser and both independently need the
classification: `porffor_front::parse` (`crates/porffor-front/src/lib.rs:166-173`)
for the entry file, and `porffor_ir::modules::record::reparse_module`
(`crates/porffor-ir/src/modules/record.rs:1331-1351`) for every dependency
module. The one table must therefore sit in the lower crate.

**Decision D0.1.** `EarlyErrorCode`, the fragment table and the classifier live in
a new module **`crates/porffor-front/src/early_error_code.rs`**.
`crates/porffor-ir/src/early_error_code.rs` still exists and is still this lane's
exclusive module, but it holds only what needs `porffor-ir` types: the
re-export and the single `EarlyErrorCode → IrDiagnosticKind` map (§2.2). There
is no second copy of any table.

### 0.2 The domain has **18** inhabitants, not 20

20 distinct `"E_..."` string literals were measured (51 tokens, 7 files, 4
crates — the brief's counts are exact and were reproduced). Two of the 20 are
not early-error codes:

- `"E_IR_DIAGNOSTIC"` (`crates/porffor-test262/src/lib.rs:21523`) is the display
  placeholder used when `diagnostic.code` is `None`, i.e. for `Unsupported` and
  `Lowering` diagnostics. It names the *absence* of a code. Making it an
  inhabitant would let `code: Some(EarlyErrorCode::IrDiagnostic)` mean "no code".
- `"E_TEST_EARLY"` (`crates/porffor-engine/src/lib.rs:4112`) is a fabricated code
  in one `#[test]`. That a unit test could mint a code that no producer emits is
  itself an instance of MC2.

`EarlyErrorCode` therefore has **18** variants. `E_IR_DIAGNOSTIC` survives as a
`&'static str` literal at exactly one display site (§6.10); `E_TEST_EARLY` is
deleted and the test names a real code (§6.9).

### 0.3 `E_OBJECT_DUPLICATE_PROTO` drift affects one path, not two

The brief says paths 2 (module entry) and 3 (module dependency) both fall through.
Measured: `Engine::compile_on_current_thread`
(`crates/porffor-engine/src/lib.rs:1907-1924`) calls `porffor_front::parse` on the
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

Front (`crates/porffor-front/src/lib.rs:235-242`) matches W1 (literal), W5
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
  (`crates/porffor-test262/src/lib.rs:21501-21506`) rejects it and the case
  **fails**. This one is conformance-visible, not merely taxonomy-visible.

### 0.5 W5 is reachable from the product path; the "duplicate lexical declaration" alternative is **not** dead

`Parser::parse_script_with_source` (`boa_parser/src/parser/mod.rs:179-193`) and
`Parser::parse_module_with_source` (`:222-235`) both call `ast.analyze_scope(...)`
and wrap any failure as `format!("invalid scope analysis: {reason}")`. The only
`ControlFlow::Break` payload in `boa_ast/src/scope_analyzer.rs` is line 1220,
carrying `global_declaration_instantiation`'s `Err("duplicate lexical declaration")`
(lines 1783, 1793). There are no other `Err("...")` string literals in that file.

Therefore `"invalid scope analysis: duplicate lexical declaration"` is the only
message that reaches either classifier through that wrapper, and it reaches
**both** `porffor_front::parse` goals. Keep the rule. Its reachability *frequency*
is an open question handed to the dry-runner as obligation **DR-6** (§7).

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

`crates/porffor-ir/src/modules/early.rs:44-55` emits it through
`IrDiagnostic::early_error` (kind `EarlyError`, phase `Early`).
`crates/porffor-ir/src/modules/graph.rs:783-788` pushes
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
  error. `crates/porffor-ir/src/diagnostics.rs:28-31` already says this.

They therefore belong in the same closed code domain, distinguished by a derived
phase, not by living in a separate enum. `ModuleLinkErrorIr::code()`
(`graph.rs:174-186`) is already that enumeration in `&'static str` form.

Two of its seven are **not** spec conditions at all —
`E_MODULE_UNSUPPORTED_PHASE` and `E_MODULE_TOO_MANY_UNITS` are implementation
limits (`graph.rs:141-168`), and their messages say so
("unsupported in porffor wasm-aot: ..."). Their current classification as
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
// crates/porffor-ir/src/diagnostics.rs:35-43
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
// crates/porffor-front/src/lib.rs:57-65
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

### 2.1 `crates/porffor-front/src/early_error_code.rs` — NEW

The sole definition of the domain, the sole producer of every `E_...` string, and
the sole classifier of boa messages.

#### 2.1.1 The enum, generated from one row list

Follow the shape of `crates/porffor-ir/src/native_error.rs` exactly — it is the
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
/// The prefix `porffor_ir::modules::record::reparse_module` puts in front of
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

There is exactly one such function in the workspace. `porffor-ir` calls this one.

### 2.2 `crates/porffor-ir/src/early_error_code.rs` — NEW

Everything that needs a `porffor-ir` type, and nothing else. `EarlyErrorCode` is
foreign here, so this is a free `const fn`, not an inherent `impl`.

```rust
pub use porffor_front::EarlyErrorCode;

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

### 2.3 `crates/porffor-ir/src/diagnostics.rs` — rewritten

```rust
use porffor_front::{EarlyErrorCode, SourceSpan};
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

### 2.4 `crates/porffor-front/src/lib.rs` — `ParseDiagnostic` collapses the same way

```rust
/// Everything `porffor_front::parse` can report, as one closed domain.
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
    Early(EarlyErrorCode),
}

impl ParseCode {
    #[must_use] pub const fn wire_name(self) -> &'static str { /* two literals + delegate */ }
    #[must_use] pub const fn kind(self) -> ParseDiagnosticKind { /* Malformed|Early → MalformedJavaScript */ }
    #[must_use] pub const fn phase(self) -> ParseDiagnosticPhase { /* Early(_) → Early, else Parse */ }
    /// The one `"SyntaxError"` literal in this crate. It cannot be
    /// `NativeErrorKind` — see ledger L2.
    #[must_use] pub const fn error_type(self) -> &'static str { "SyntaxError" }
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
parameters and takes an `EarlyErrorCode`.

### 2.5 The const assertions

All of P1–P6 live in `crates/porffor-front/src/early_error_code.rs`; P7–P9 live
in `crates/porffor-ir/src/early_error_code.rs`. They need a private
`const fn contains_sub(haystack: &str, needle: &str) -> bool` written as a byte
loop, in the shape of `native_error.rs`'s private `str_eq` (and, like it, private
on purpose: a `pub const fn contains_sub` would be workspace surface with no
product call site).

| id | statement | the mistake it makes fail to build |
|---|---|---|
| **P1** | every row has `fragments.len() >= 1` and `witnesses.len() >= 1` | `fragments: &[]` matches **every** message (`[].iter().all(_)` is `true`), silently swallowing all parse failures into one code |
| **P2** | for every row *i* and every `w` in `rows[i].witnesses`: exactly one row of `PARSE_FAILURE_RULES` matches `w`, and it is row *i* | a new row that shadows an existing one, or is shadowed by it; **and** it makes the table order-independent, upgrading `early.rs:114`'s comment to a checked fact |
| **P3** | `ALL[i] as u8 == i` for all *i*, and `from_wire_name(c.wire_name()) == Some(c)` for all `c` | `ALL` out of declaration order or incomplete; `wire_name`/`from_wire_name` diverging |
| **P4** | no two codes share a `wire_name()` | a duplicated spelling would make one code unreachable through `from_wire_name` and would collapse two taxonomy buckets |
| **P5** | every `wire_name()` begins `"E_"` and contains only `b'A'..=b'Z'` and `b'_'` | `"e_FOO"`, `"E_Foo"`, a stray space — a typo that a `&str` domain would carry all the way into the failure taxonomy |
| **P6** | `classify_parse_failure(MODULE_REPARSE_PREFIX) == None` | a future fragment (e.g. bare `"module"`) that the dependency-path wrapper prefix would match on its own, making every dependency parse failure classify as one code |
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

- `crates/porffor-aot-wasm/src/emit.rs:313-318`
- `crates/porffor-ir/src/ir.rs:2248-2255`
- `crates/porffor-ir/src/lowering.rs:391-400`

— and `porffor-aot-wasm` is batch 2's crate. Keeping the four unit variants means
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
- **L2 — `"SyntaxError"` in `porffor-front`.** `ParseCode::error_type` returns a
  `&'static str` literal because `NativeErrorKind` lives in `porffor-ir`, which
  `porffor-front` cannot name (§0.1). The literal count in `porffor-front` goes
  from 3 to 1 and it is no longer at any call site, but it is still a string.
  Closing this requires moving `NativeErrorKind` below `porffor-ir`, which is
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
  for unmodelled parse failures. **Recorded, not fixed:** the brief puts the
  semantics of link failures out of scope, and fixing it changes which
  diagnostics reach the backend (`emit.rs:313-318` treats `LinkError` and
  `Unsupported` alike, but `engine/lib.rs:1948-1952` does not). A future lane
  owning `graph.rs` and `emit.rs` should split them out.
- **L5 — reachability of a row.** Nothing makes a `PARSE_FAILURE_RULES` row with
  no living producer fail to build. Row 6 (`duplicate lexical declaration`) was
  suspected dead and proved live (§0.5); the next one may not be. Enforced by
  review of the `file:line` provenance column, not by the compiler.
- **L6 — the dependency-path message prefix exists twice.** `MODULE_REPARSE_PREFIX`
  and the literal at `crates/porffor-ir/src/modules/record.rs:1343` are two copies
  of `"lowering module reparse failed: "`, because `record.rs` is not in this
  lane's `files_owned`. P6 checks the inertness property for the constant as
  written, which is the property that matters; the copies drifting apart would
  make P6 check a string no producer emits. Stated in full at §6.15.

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
| MC1 | Add an early-error rule to one table and not the other | happens silently; **has happened twice and is live** (§0.3, §0.4) | there is no second table; `early.rs:107-170` is deleted and `porffor_ir` calls `porffor_front::classify_parse_failure` | unrepresentable — deletion, not a diagnostic |
| MC1′ | Add a row that shadows an existing row, or is shadowed by one | silent misclassification; the disjointness claim is a comment (`early.rs:114`) | `witnesses` + **P2** | `error: evaluation of constant value failed` at the `const _: () = assert!(witnesses_select_their_own_row(), …)` in `early_error_code.rs` |
| MC1″ | Add a row with an empty `fragments` list | matches every message; every parse failure collapses to one code | **P1** | same const-assert failure, distinct message |
| MC2 | Mint a code as a fresh string literal | compiles; 51 tokens over 4 crates; typo = unmatched arm = silent misclassification | there is no `&'static str` code anywhere; `EarlyErrorCode` has no `FromStr`/`Display` | `error[E0308]: expected \`EarlyErrorCode\`, found \`&str\`` at any construction site |
| MC2′ | Misspell an existing code at a comparison site (`test262/src/lib.rs:21523`, `engine/src/lib.rs:4131`) | compiles; silently never matches | variant path | `error[E0599]: no variant or associated item named \`ObjectDuplicateProtoo\` found for enum \`EarlyErrorCode\`` |
| MC2″ | Add a 19th code and forget to classify it | not possible today (no domain) | `rejection_kind`'s exhaustive match | `error[E0004]: non-exhaustive patterns: \`EarlyErrorCode::NewThing\` not covered` in `ir/early_error_code.rs`; and `error[E0308]` on `ALL: [EarlyErrorCode; 18]` |
| MC3 | Write `"SyntaxError"` — or a misspelling — at a diagnostic construction site | compiles; `NativeErrorKind` exists and is bypassed (measured: zero references in `diagnostics.rs` or `modules/*.rs`) | `error_type` is not a field and not a parameter; `IrDiagnosticKind::error_type()` is the sole producer, returning `NativeErrorKind` | `error[E0061]: this function takes 3 arguments but 4 arguments were supplied` on `IrDiagnostic::rejected`; and `error[E0599]` on a misspelled `NativeErrorKind::SyntaxErrror` |
| MC4 | Report one code under two phases from two paths | four independent fields; held only by convention (measured: zero `IrDiagnostic { … }` literals outside `diagnostics.rs`); **already latent for `E_MODULE_DUPLICATE_EXPORT`** (§0.7) | `phase` is not a field; `code` is private; `rejected` is the only coded constructor; **P7** ties the parse table to `rejection_kind` | `error[E0609]: no field \`phase\` on type \`IrDiagnostic\``; `error[E0451]: field \`code\` of struct \`IrDiagnostic\` is private` for a struct literal outside the module; const-assert failure for a P7 violation |
| MC4′ | A fifth constructor added *inside* `diagnostics.rs` pairing a `LinkError` kind with a parse-phase code | possible | still possible — **ledger L3** | none; review only |

---

## 6. Retrofit map

Strictly ordered. Each step leaves the tree in a state where the next step's
errors are attributable. `cargo check -p porffor-front` after step 3;
`cargo check -p porffor-ir` after step 8; `cargo check --workspace` after step 11.

### 6.1 NEW `crates/porffor-front/src/early_error_code.rs`

The whole of §2.1 plus const assertions P1–P6. Nothing else; this file must not
name any `porffor-ir` type.

### 6.2 `crates/porffor-front/src/lib.rs` — module wiring

Add `mod early_error_code;` and
`pub use early_error_code::{classify_parse_failure, EarlyErrorCode, MODULE_REPARSE_PREFIX};`.

### 6.3 `crates/porffor-front/src/lib.rs:40-65` — `ParseCode` and `ParseDiagnostic`

Add `enum ParseCode` (§2.4). Delete `ParseDiagnostic`'s `kind`, `phase` and
`error_type` fields; retype `code` to `ParseCode`; add the three accessors.
`ParseDiagnosticKind` and `ParseDiagnosticPhase` keep their definitions — they
are now return types, not fields.

### 6.4 `crates/porffor-front/src/lib.rs:73-128` — the three constructors

`malformed` → `code: ParseCode::Malformed`. `unsupported_parser_feature` →
`ParseCode::UnsupportedParserFeature`. `early_error(code, error_type, message,
span)` → `early_error(code: EarlyErrorCode, message, span)`, storing
`ParseCode::Early(code)`. The two `"P_PARSE_*"` literals and the three
`"SyntaxError"` literals leave the constructors and become `ParseCode`'s three
`const fn` bodies (2 + 1 literals).

### 6.5 `crates/porffor-front/src/lib.rs:181-185` and `:205-256` — the classifier

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

### 6.6 `crates/porffor-front/src/lib.rs` tests — 5 sites

`:356`, `:380`, `:448` → `ParseCode::Malformed`. `:396` →
`ParseCode::Early(EarlyErrorCode::ObjectDuplicateProto)`. `:408`, `:413`, `:421`,
`:429` → the corresponding `ParseCode::Early(...)`. Add one test asserting
**B3**: a module-goal `let x; const x;` classifies as
`ParseCode::Early(EarlyErrorCode::DuplicateLexicalDeclaration)` and no longer as
`Malformed`.

### 6.7 NEW `crates/porffor-ir/src/early_error_code.rs`

§2.2 plus const assertions P7–P9.

### 6.8 `crates/porffor-ir/src/lib.rs`

- Add `mod early_error_code;` beside `mod early_errors;` (line 60-ish) —
  **note the two names are unrelated**; `early_errors.rs` is derived-constructor
  validation over 76 `ExprIr::` arms and belongs to the Reference-Records lane.
  Consider a comment saying so at the `mod` lines, because the adjacency is a
  trap.
- Add `pub use early_error_code::EarlyErrorCode;` next to the existing
  `pub use native_error::NativeErrorKind;` (line 136).
- `:1185` → `.all(|d| d.code() != Some(EarlyErrorCode::ObjectDuplicateProto))`.

### 6.9 `crates/porffor-ir/src/diagnostics.rs`

The whole of §2.3. `early_error` and `link_error` are deleted; `rejected` replaces
both. The `"SyntaxError"` literal at `:68` disappears.

### 6.10 `crates/porffor-ir/src/modules/early.rs`

- Delete `struct ParseFailureRule` and `const PARSE_FAILURE_RULES`
  (`:107-170`, 64 lines, 11 `E_` tokens). Replace the doc comment at `:98-106`
  with a pointer to `porffor_front::early_error_code`.
- `:49-54` and `:83-92` → `IrDiagnostic::rejected(EarlyErrorCode::ModuleDuplicateExport, …)`
  and `IrDiagnostic::rejected(EarlyErrorCode::ModuleUndeclaredExport, …)`. The
  first currently reads `error.code()` from a `ModuleLinkErrorIr`; name the code
  directly — the round trip through a link error was never meaningful.
- `module_parse_failure_diagnostic` becomes:
  ```rust
  pub(crate) fn module_parse_failure_diagnostic(message: &str) -> IrDiagnostic {
      match porffor_front::classify_parse_failure(message) {
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

### 6.11 `crates/porffor-ir/src/modules/graph.rs`

- `:174-186` `pub const fn code(&self) -> &'static str` → `-> EarlyErrorCode`,
  seven arms returning variants. Keep the arm order.
- `:226` `IrDiagnostic::link_error(self.code(), self.message())` →
  `IrDiagnostic::rejected(self.code(), self.message(), None)`. This is where B4
  lands.
- `:1783` → `d.code() == Some(EarlyErrorCode::ModuleTooManyUnits)`.

### 6.12 `crates/porffor-engine/src/lib.rs` — tests only

- `:4103-4105` → `diagnostic.phase()`, `diagnostic.error_type()`,
  `diagnostic.code == ParseCode::Malformed`.
- `:4112` → `IrDiagnostic::rejected(EarlyErrorCode::ObjectDuplicateProto, "early error: test", None)`;
  `"E_TEST_EARLY"` is deleted (§0.2).
- `:4131-4133` → `ParseCode::Early(EarlyErrorCode::ObjectDuplicateProto)`,
  `diagnostic.phase()`, `diagnostic.error_type()`.
- `:1948-1952` (`kind == IrDiagnosticKind::EarlyError`) is **untouched** — unit
  variant, unchanged.

### 6.13 `crates/porffor-test262/src/lib.rs`

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

`crates/porffor-ir/src/lowering.rs:28135-28141`:

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

### 6.15 What stays untouched, verified by reading

| file:line | why it does not change |
|---|---|
| `crates/porffor-aot-wasm/src/emit.rs:313-318` | matches `IrDiagnosticKind` unit variants only |
| `crates/porffor-ir/src/ir.rs:2248-2255` | same |
| `crates/porffor-ir/src/lowering.rs:391-400` | same |
| `crates/porffor-engine/src/lib.rs:1948-1952` | same |
| `crates/porffor-ir/src/modules/record.rs` | calls `module_parse_failure_diagnostic` and `module_early_errors`; neither signature changes. Its `"lowering module reparse failed: "` literal at `:1343` is duplicated by `MODULE_REPARSE_PREFIX` — see **L6** below |
| `crates/porffor-ir/src/modules/{link,namespace,dynamic}.rs` | construct `IrDiagnostic` only through `unsupported`/`lowering` |
| `crates/porffor-ir/src/native_error.rs` | round 1's file; read, not edited |
| `crates/porffor-ir/src/early_errors.rs` | despite the name, derived-constructor validation over 76 `ExprIr::` arms; Reference-Records lane |
| `crates/porffor-cli/**` | measured: zero `"E_..."` and zero `IrDiagnostic` references |
| `crates/porffor-spec-exec/**` | measured: zero `IrDiagnostic` references |

**Ledger addition L6** — `MODULE_REPARSE_PREFIX` and the literal at
`record.rs:1343` are two copies of one string, because `record.rs` is not in this
lane's `files_owned`. P6 still checks the property for the constant as written.
The one-line single-sourcing (`format!("{MODULE_REPARSE_PREFIX}{err}")`) is
deferred to whoever owns `record.rs`.

### 6.16 Token accounting

| file | `"E_..."` tokens before | after |
|---|---|---|
| `porffor-ir/src/modules/early.rs` | 21 | 0 |
| `porffor-front/src/lib.rs` | 17 | 0 |
| `porffor-ir/src/modules/graph.rs` | 8 | 0 |
| `porffor-engine/src/lib.rs` | 2 | 0 |
| `porffor-test262/src/lib.rs` | 1 | 1 (`E_IR_DIAGNOSTIC`, the absence placeholder) |
| `porffor-ir/src/lib.rs` | 1 | 0 |
| `porffor-ir/src/lowering.rs` | 1 | 0 |
| `porffor-front/src/early_error_code.rs` (new) | — | 18 (`wire_name`, the sole producer) |
| **total** | **51 across 7 files** | **19 across 2 files** |

`"SyntaxError"` literals: `porffor-ir` goes from 12 (`diagnostics.rs:68`,
`early.rs` ×11) to **0**; `porffor-front` goes from 3 to **1** (ledger L2).
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
| **DR-1** | `language/expressions/object/__proto__-duplicate.js` | Row 1 fires. Then **the adversarial triple**: the same body as (i) a script, (ii) a module entry, (iii) `dep.js` imported by a trivial module entry. All three must reach `ObjectDuplicateProto`/`SyntaxError`/parse. Show that (i) and (ii) go through `porffor_front::parse` (`engine/lib.rs:1913-1923`) and (iii) through `record.rs:1343` → `module_parse_failure_diagnostic`, and that both now consult the same table. **This is the acceptance trace for B1.** Confirm the brief's claim about path (ii) is wrong (§0.3) or produce the counter-trace. |
| **DR-2** | `language/module-code/early-dup-lex.js` (`let x; const x;`) | As an entry: `ModuleParser::parse` (`boa_parser/src/parser/mod.rs:507-517`) emits W2. Show that under §6 it now hits row 4 and yields `ParseCode::Early(DuplicateLexicalDeclaration)`, where today it yields `Malformed`. **Acceptance trace for B3.** As a dependency: row 4 again, same code. State plainly that the two paths now agree, and that they did not before. |
| **DR-3** | `language/module-code/early-lex-and-var.js` (`let x; var x;`) | Which boa check fires for the *module* goal — `mod.rs:521-531` (W2) — and which for a *script* — `mod.rs:372-379` (W1) or the block/switch forms (W3/W4). Then show W1 and W2 both hit row 4 and W3/W4 both hit row 5, one code. This is the probe that decides whether the loose `"lexical"+"declared"+"names"` fallback was load-bearing; §4's non-change proof predicts **not** and must be checked, not assumed. |
| **DR-4** | `language/module-code/early-dup-export-id.js` | Row 2. Then follow the *other* producer: `modules::early::module_early_errors` `:44-55` for a dependency, and `graph.rs:783-788` for the (currently unreachable) link path. Show `rejection_kind(ModuleDuplicateExport) == EarlyError` makes both yield phase `Early`, and that P7 is what forces it. **Acceptance trace for B4.** Also show *why* the graph path is unreachable today (`record.rs:740-743` returns `Err` first) so the change is correctly labelled latent. |
| **DR-5** | `language/module-code/early-export-unresolvable.js` | Row 3 on the parse path. Then the distinct link-path code `ModuleMissingExport` (`graph.rs:836,873`) → `rejection_kind` → `LinkError` → `Resolution` → `SyntaxError`. Show the two codes are different codes for different conditions and that neither can borrow the other's phase. |
| **DR-6** | any script with a lexical redeclaration | **Settle L5/§0.5 as a refutable prediction.** `Parser::parse_script_with_source` runs `ScriptParser::parse`'s own duplicate check (`mod.rs:361-379`) *before* `analyze_scope` (`mod.rs:186`), so W1 should fire and W5 should never be reached for a plain top-level `let x; let x;`. Predict: no ordinary source reaches W5 through `porffor_front::parse`, and row 6 is retained on the strength of the *reachable path existing*, not of a witness case. If the trace finds a source that does reach W5, name it; if it finds W5 unreachable in every construction tried, say so and leave row 6 in with the finding recorded — do **not** delete a row on a negative result. |
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

1. `cargo check -p porffor-front` — after §6.6. All const assertions P1–P6 are
   evaluated here; a table defect is caught before `porffor-ir` is touched.
2. `cargo check -p porffor-ir` — after §6.11. P7–P9 evaluate here.
3. `cargo check --workspace` — after §6.14.
4. `cargo test -p porffor-front` then `cargo test -p porffor-ir` — the rewritten
   tests, including the new B1/B2/B3 cases.
5. `cargo test -p porffor-engine --lib` for the four diagnostic tests.
6. Rung 4 for this lane's family: `porf test262 run language/module-code` and
   `porf test262 run language/expressions/object`. B1 and B2 predict a pass-count
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
oracle (L1), `porffor-front` keeps one `"SyntaxError"` literal it cannot type
(L2), a fifth constructor inside `diagnostics.rs` can still pair badly (L3), two
implementation limits still claim `SyntaxError` (L4), and a dead table row still
compiles (L5). Those five are the whole of what tests remain load-bearing for in
this area.
