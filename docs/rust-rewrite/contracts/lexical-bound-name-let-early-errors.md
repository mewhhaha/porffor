# Lexical bound-name `let` early errors

Status: implemented, independently reviewed and focused-verified for the T07
diagnostic-closure lane on 2026-08-23.

## Decision

A lexical declaration or iterable `ForDeclaration` whose `BoundNames`
contains the exact name `"let"` is one closed pre-evaluation condition:

`EarlyErrorCode::LexicalBoundNameLet`

Its sole wire spelling is `E_LEXICAL_BOUND_NAME_LET`. The code names the
static-semantics condition, not a declaration keyword, binding-pattern shape,
loop production or parser wording.

## Specification boundary

The edition-pinned ECMA-262 2026 clauses for
[`LexicalDeclaration`](https://tc39.es/ecma262/2026/multipage/ecmascript-language-statements-and-declarations.html#sec-let-and-const-declarations-static-semantics-early-errors)
and
[`ForDeclaration`](https://tc39.es/ecma262/2026/multipage/ecmascript-language-statements-and-declarations.html#sec-for-in-and-for-of-statements-static-semantics-early-errors)
require a `SyntaxError` when the production's `BoundNames` contains `"let"`.
The condition is independent of strictness and applies under both Script and
Module goals.

In the frozen 2026 edition, the ordinary lexical production is `let` or
`const`, and `ForDeclaration` is `LetOrConst ForBinding`. The corresponding
living-specification clauses extend the same condition to resource
declarations, including `using` and `await using`. Pinned Boa already parses
those resource-declaration alternatives. This lane classifies the condition
on the actual pinned parser surface while stating this frozen/living boundary;
it does not claim that the 2026 grammar itself contains those alternatives.

The exact name matters. A property name `let` bound to another identifier, an
identifier such as `letter`, and sloppy-Script `var let` are outside this
condition. Strict Script or Module may reject `var let` for another reason,
but that rejection must not acquire this code.

## Pre-normalization Boa boundary

Before this lane, pinned `boa_parser-0.21.1` contained five physical producers
and two fixed messages:

- four occurrences of `'let' is disallowed as a lexically bound name` in
  `parser/statement/declaration/lexical.rs`; and
- one occurrence of `Cannot use 'let' as a lexically bound name` in
  `parser/statement/iteration/for_statement.rs`.

Only the object, array and identifier branches of `LexicalBinding` are
currently source-reachable for an ordinary lexical declaration. They reject
before the later declaration-level walk, making that fourth occurrence
shadowed. The same binding-level checks also reject an ambiguous lexical
for-head before its delimiter is known, shadowing the iterable-tail producer.
Neither fixed message is classified, so entry parsing reports generic
`P_PARSE_MALFORMED`; a rejected dependency becomes untyped `Unsupported`.

## Producer normalization

Use the existing closed
`LexicalDeclarationContext::{Statement, ForHead}` and retained
`ParsedForInitializer::DeferredLexical` state to make the grammar owner known
before this condition is emitted:

1. remove the three shape-specific `LexicalBinding` checks;
2. keep one shared forbidden-`let` validator over the completed lexical
   declaration;
3. invoke it immediately for `Statement` declarations;
4. defer it for an ambiguous `ForHead`;
5. in the classic deferred-lexical route, invoke it before the existing
   duplicate-bound-name validator; and
6. leave iterable heads to the existing condition-specific tail producer,
   before its head/body-conflict and duplicate-bound-name checks.

This produces exactly two source-reachable semantic owners: the shared
ordinary/classic lexical validator and the iterable `ForDeclaration` tail.
The context and delimiter matches must remain exhaustive. A new lexical
context or for-head route must fail to compile until it selects an owner.

Pinned Boa's generic `BindingIdentifier` parser has an earlier strict-mode
reserved-identifier rejection. Under Module or another strict context, that
generic check can reject the exact name `let` while parsing an identifier or a
nested array/object binding shape, before the completed declaration reaches
either semantic owner above. The binding parser therefore also needs one
closed grammar context:

```rust
BindingIdentifierContext = General | LexicalDeclaration
```

`General` remains the default for parameters, `var`, catch bindings and every
other consumer. Only the three `LexicalBinding` entry shapes select
`LexicalDeclaration`, and array/object pattern recursion must carry that same
context to every nested `BindingIdentifier`. In that one context, only the
exact symbol `let` may pass the generic strict reserved-name check so the
declaration-level `BoundNames` validator can reject it with the specification
condition. No other strict reserved word is admitted, and no caller may select
the context with a raw Boolean. The context projection and propagation must be
exhaustive so a new binding shape cannot silently restore the earlier generic
owner.

The normalization preserves current precedence where invalidities coexist.
For ordinary and classic declarations, forbidden `let` is checked before a
duplicate-bound-name condition. For iterable declarations, forbidden `let`
remains before head/body conflict and duplicate-name insertion. This is a
pinned diagnostic-ownership rule, not a claim that ECMAScript exposes a
general ordering among simultaneous early errors.

## Closed diagnostic projection

Add exactly two `ParseFailurePattern::StartsWith` rows, anchored to the
complete rendered prefixes produced after Boa appends a position:

```text
'let' is disallowed as a lexically bound name at line
Cannot use 'let' as a lexically bound name at line
```

Do not use `ContainsAll` or a shorter substring. A const ownership assertion
must require exactly those two independently spelled rows for the new code, so
deleting, broadening or transferring either producer fails to build. Add the
one closed `EarlyErrorCode` variant and exhaustive `lila-ir` early-error
projection; no catch-all may absorb it.

The pre-extension `EarlyErrorCode::ALL` contains 60 variants and the classifier
table contains 59 rows. This extension grows them to 61 and 61 respectively.

Direct entry parsing must project real parser failures to phase `Early`, native
`SyntaxError`, the new code and a nonempty source span under both goals. A
loaded dependency must retain its real failed Module parse in `ModuleSourceIr`,
cross `build_graph`, and surface the same identity through
`module_parse_failure_diagnostic`. Constructed diagnostics or direct table
lookups are insufficient substitutes.

## Required source matrix

Both Script and Module goals must select the new code for:

- identifier, array and object bindings in ordinary `let` and `const`
  declarations;
- identifier, array and object bindings in classic lexical `for` heads;
- `for-in`, `for-of` and async `for-await-of` lexical heads; and
- the resource-declaration spellings accepted by pinned Boa, including
  `using` and `await using` iterable heads.

Permanent precedence controls include an ordinary `let [let, let]`, a classic
`for (let [let, let] = []; ; )`, and an iterable
`for (let [let, let] of [])`; all must select this code before the applicable
duplicate-name or body-conflict code.

Positive boundaries include a property named `let` bound to `x`, ordinary
identifiers such as `letter`, valid lexical loop heads and sloppy-Script
`var let`. Strict-Script and Module `var let` rejections must retain their own
identity. A duplicate Module export whose source text contains either complete
fixed prefix must remain `ModuleDuplicateExport`, proving that parser-message
classification cannot be injected through source text.

## Durable source witness

The bounded front-end guard should recursively inventory pinned Boa and prove:

- exactly two fixed messages and exactly two reachable producer locations
  after normalization;
- the shared validator owns ordinary and classic lexical declarations;
- the closed binding-identifier context is selected only by the three lexical
  entry shapes, propagates through nested array/object patterns, and defers only
  the exact name `let` under strict parsing;
- `ForHead` defers until an exhaustive classic/iterable split;
- the classic arm checks forbidden `let` before duplicate names;
- the iterable tail checks forbidden `let` before head/body conflict and
  duplicate insertion;
- the two exact anchored classifier rows are the sole owners of the new code;
  and
- the direct matrix, precedence, positives and injection controls above.

The `lila-ir` witness must use a real failed Module parse and a retained rejected
dependency graph node. It must not construct the result it intends to prove.

## Exact pinned Test262 cohort

The current pin contains ten direct leaves, expanding to fourteen variants:

- `language/statements/let/syntax/let-let-declaration-split-across-two-lines.js`
  — 2;
- `language/statements/let/syntax/let-let-declaration-with-initializer-split-across-two-lines.js`
  — 2;
- `language/statements/const/syntax/const-declaring-let-split-across-two-lines.js`
  — 2;
- `language/statements/let/syntax/identifier-let-disallowed-as-boundname.js`
  — 2;
- `language/statements/for-in/head-let-bound-names-let.js` — 1;
- `language/statements/for-in/head-const-bound-names-let.js` — 1;
- `language/statements/for-of/head-let-bound-names-let.js` — 1;
- `language/statements/for-of/head-const-bound-names-let.js` — 1;
- `language/statements/for-of/head-using-bound-names-let.js` — 1; and
- `language/statements/for-of/head-await-using-bound-names-let.js` — 1.

The first four carry no execution-mode flag and expand to sloppy/strict
variants. The remaining six are `noStrict`. Each physical leaf must be invoked
by its complete suite-relative path, and verification must inspect the exact
discovery total and every non-success bucket.

## Verification evidence

The vendored parser repair and the front/IR diagnostic closure were
independently reviewed. The final source inventory has 61 closed diagnostic
variants, 61 classifier rows, exactly two reachable semantic producers, three
lexical root context opt-ins, five identifier-leaf propagations and six
recursive binding-pattern propagations.

Under the shared eight-core, 22 GB cap, `cargo fmt --all -- --check`, `cargo xc`
and `git diff --check` are green. The focused front filter passes `4/4`, the
exact vendored-source guard passes `1/1`, and the complete `lila-front` library
passes `112/112`. The exact IR classifier passes `1/1`, the two focused real
Module/retained-graph witnesses pass `2/2`, and the complete IR module-early
group passes `43/43`.

The ten complete Test262 leaves above discover and pass exactly `14/14`
Wasm-AOT variants. Every parser, early-error, lowering, runtime, Wasm-backend,
host-harness, unsupported, not-implemented, crash and bug bucket is zero under
`--jobs 1 --threads 1`.

## Explicit nonclaims

This lane closes one parser-owned diagnostic identity. It does not add a new
syntax rejection, implement lexical or iterable execution, broaden resource
declaration grammar, change `var let`, resolve every early error, prove a new
Test262 pass, refresh aggregate status, or complete T07. The negative cohort
may already pass as generic parse-phase `SyntaxError`; the material result is
the condition-specific type and honest retained-dependency projection.
