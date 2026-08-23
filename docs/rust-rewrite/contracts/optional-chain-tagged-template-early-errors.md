# Optional-chain tagged-template early errors

**Status:** Normative T07 extension; implementation independently reviewed and
focused-verified under the shared eight-core cap, 2026-08-23

## Decision

An optional chain followed directly by a TemplateLiteral is one closed
pre-evaluation condition:

`EarlyErrorCode::OptionalChainTaggedTemplate`

Its sole wire spelling is `E_OPTIONAL_CHAIN_TAGGED_TEMPLATE`. The code names
the static-semantics condition, not either parser branch: the same condition
owns both a template introduced immediately by `?.` and a template appended to
an already-started `OptionalChain`.

## Specification boundary

ECMA-262 2026 section
[13.3.1.1, Static Semantics: Early Errors](https://tc39.es/ecma262/2026/multipage/ecmascript-language-expressions.html#sec-left-hand-side-expressions-static-semantics-early-errors)
defines these two forbidden productions:

```text
OptionalChain : ?. TemplateLiteral
OptionalChain : OptionalChain TemplateLiteral
```

Any source text matched by either production is a `SyntaxError`. This is a
grammar boundary rather than a run-time optional-chain decision: it rejects
even when the base is non-nullish. The second production also deliberately
rejects the line-terminator form instead of allowing automatic semicolon
insertion to reinterpret the TemplateLiteral as a separate statement.

Ordinary tagged templates, ordinary optional property/call forms, and a
completed parenthesized optional expression used as a tag remain outside the
two productions. For example, `` tag`x` ``, `value?.property`, `callable?.()`
and `` (value?.tag)`x` `` are syntactically valid. The last form terminates the
`OptionalChain` inside parentheses before the tagged-template grammar is
entered; any later call failure is a run-time concern.

## Measured Boa boundary

Pinned `boa_parser-0.21.1` has exactly two producers of the same fixed,
case-sensitive raw message in
`vendor/boa_parser-0.21.1/src/parser/expression/left_hand_side/optional/mod.rs`:

```text
Invalid tagged template on optional chain
```

- the branch currently at line 130 sees a TemplateLiteral while extending an
  existing optional chain and owns `OptionalChain TemplateLiteral`;
- the branch currently at line 163 sees a TemplateLiteral immediately after
  consuming `?.` and owns `?. TemplateLiteral`.

`Error::general` appends the source position. One classifier row therefore
uses the complete rendered prefix
`Invalid tagged template on optional chain at line` through
`ParseFailurePattern::StartsWith`. An anywhere-substring rule is not an
acceptable encoding: Boa can interpolate a user-chosen Module export name
inside a different `Error::general` diagnostic, so source text containing this
fixed phrase could otherwise forge the optional-chain condition.

The typed contract requires a const ownership assertion proving that exactly
one row owns `OptionalChainTaggedTemplate` and uses this complete `StartsWith`
prefix, plus an adversarial source/message witness proving an exported name
containing the phrase cannot acquire this code. A duplicated forged export
name must retain `ModuleDuplicateExport` ownership. Exact anchoring is part of
the invariant, not merely a table-order convention.

## Goal and diagnostic boundary

Both forbidden productions are reachable under Script and Module goals. Before
this extension, a direct entry parse had no matching classifier row and
therefore reported `P_PARSE_MALFORMED`. A failed dependency parse was retained
by the module loader as the same unclassified parse failure, which
`module_parse_failure_diagnostic` conservatively projected to `Unsupported`
without a native error type.

The written extension makes both entry goals produce
`ParseCode::Early(OptionalChainTaggedTemplate)`, with phase `Early`, native
`SyntaxError` and a source span. A retained dependency is necessarily parsed
under the Module goal; the same typed parse rejection must cross the
front-to-IR boundary as `IrDiagnosticKind::EarlyError` with the identical code,
phase, error type and span. The retained path consumes the front-end
classification and must not introduce a second message table.

## Typed encoding

- The one `EarlyErrorCode` variant and its wire spelling extend the closed
  front-end domain.
- Exactly one anchored-prefix classifier row carries the complete rendered Boa
  message as its witness.
- An evaluated `ParseClassified` const assertion makes the row parse-owned;
  deleting the row while leaving the variant must fail `cargo check`.
- Evaluated const assertions encode one-row ownership by the exact reviewed
  prefix and the export-name anti-forgery boundary rather than leaving them as
  comments.
- The variant appears in `lila-ir`'s exhaustive rejection-kind mapping. No
  catch-all can absorb it.
- A real failed Module parse passes through
  `module_parse_failure_diagnostic`, and a rejected `ModuleSourceIr` dependency
  crosses `build_graph`; the hand-built table witness does not stand in for
  retained-dependency projection.

The pre-extension closed domain had 56 variants and the parse-failure table had
55 rows. The written extension grows them to 57 and 56 respectively; those
counts remain written into array types so an incomplete extension is a compile
error.

## Durable witnesses

Front-end witnesses cover both grammar productions under Script and Module
goals, both substitution-bearing and no-substitution TemplateLiterals, and
same-line and line-terminator forms. Positive controls preserve ordinary
substituted and unsubstituted tagged templates, optional property access,
optional call and a parenthesized completed optional expression used as a tag.
The adversarial Module control embeds the fixed diagnostic phrase in an export
name without allowing it to select this code.

The retained-module witness parses a real dependency source using one of the
forbidden productions into `ModuleSourceIr`, verifies that its rejection is
retained, and requires `build_graph` to return the typed code, `Early` phase,
`SyntaxError` constructor and a nonempty source span. The written source-level
structural witness pins exactly two reviewed message occurrences and two
`TemplateMiddle | TemplateNoSubstitution` producer pairs in the vendored
optional-chain parser. It is a drift alarm for the measured producer boundary,
not a substitute for compilation or behavior verification.

## Pinned Test262 cohort

Pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
contains exactly these eight direct `phase: parse`, `type: SyntaxError`
witnesses under `language/expressions/optional-chaining`:

- `early-errors-tail-position-null-op-template-string-esi.js`
- `early-errors-tail-position-null-op-template-string.js`
- `early-errors-tail-position-null-optchain-template-string-esi.js`
- `early-errors-tail-position-null-optchain-template-string.js`
- `early-errors-tail-position-op-template-string-esi.js`
- `early-errors-tail-position-op-template-string.js`
- `early-errors-tail-position-optchain-template-string-esi.js`
- `early-errors-tail-position-optchain-template-string.js`

The `*-op-*` files cover `?. TemplateLiteral`; the `*-optchain-*` files cover
`OptionalChain TemplateLiteral`. The four `*-esi.js` variants preserve the
line-terminator/automatic-semicolon-insertion boundary. Nullish and non-nullish
bases confirm that this is syntactic and does not depend on run-time
short-circuiting.

## Focused verification and nonclaims

The shared capped lane completed the promised evidence:

- `cargo xc` passed for the workspace;
- the full `lila-front` library suite passed `97/97`, including both forbidden
  productions, both goals, both template token shapes, line-terminator forms,
  positive controls, exact producer inventory and anti-forgery witnesses;
- `modules::early::tests` passed `40/40`, including the real Module parse
  projection;
- the exact retained-graph regression passed `1/1`, preserving the code,
  `Early` phase, `SyntaxError` type and nonempty span through `build_graph`; and
- the exact eight-file pinned cohort selected by
  `language/expressions/optional-chaining/early-errors-tail-position` passed
  `16/16` Wasm-AOT executions with every non-success bucket at zero, using
  `--jobs 1 --threads 1`.

No measured pass gain is claimed: direct negative Test262 cases may already be
counted as successful parse-phase `SyntaxError` tests while still carrying the
generic malformed taxonomy. The material change is the closed diagnostic
identity and the retained-dependency projection.

This lane does not implement optional-chain evaluation, tagged-template
evaluation or template-object caching. It does not claim all optional-chain
grammar, direct eval, T07 closure, a current aggregate Test262 result or any
published status change.
