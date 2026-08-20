# Contract: duplicate class-constructor early errors

**Status:** Normative T07 extension, 2026-08-13

## Spec invariant

The ClassBody early-error rules reject a class whose
`PrototypePropertyNameList` contains more than one occurrence of
`"constructor"`. This is one closed condition for class declarations and class
expressions under both Script and Module goals. It is detected before
evaluation and reported as a `SyntaxError`.

Only ordinary, non-static constructor definitions contribute to that list. A
single ordinary constructor may coexist with a static method named
`constructor` and with a computed method named `["constructor"]`. Those are
positive boundaries of this condition, not alternative constructor
definitions.

## Measured Boa boundary

Pinned `boa_parser-0.21.1` emits exactly one case-sensitive literal for this
condition in
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs:319-324`:

`a class may only have one constructor`

The producer records the first parsed constructor and emits that literal when
another class element is returned as a constructor. This is the only occurrence
of the literal in the pinned Boa source.

Before this extension, the one classifier had no row for the literal. Entry
parsing therefore returned `P_PARSE_MALFORMED`; the same failed parse retained
for a dependency Module became an `Unsupported` IR diagnostic with no native
error type. Neither result represented the specified early `SyntaxError`.

Nearby constructor restrictions remain separate conditions. Private,
generator, accessor, async, or otherwise invalid constructor definitions have
their own grammar or early-error messages and must not be selected by this
literal.

## Encoding

- Add `EarlyErrorCode::DuplicateClassConstructor` with the sole wire spelling
  `E_DUPLICATE_CLASS_CONSTRUCTOR`.
- Add exactly one parse-failure row. Its one fragment and one witness are the
  complete Boa literal above; a broader fragment would turn an adjacent
  constructor restriction into an unearned duplicate-constructor claim.
- Map the new variant through `lila-ir`'s exhaustive `rejection_kind` match to
  `IrDiagnosticKind::EarlyError`. Phase and error type remain derived as
  `Early` and `SyntaxError` for entry and retained-module diagnostics.
- Keep `ParseClassified` as the parse-stage gate. Existing const assertions
  continue to prove row population, witness disjointness, wire-name closure,
  classifier reachability, interpolation-guard separation, and parse-to-IR
  phase consistency.

This extension brings the closed domain from 21 to **22** variants and the one
parse-failure table from 19 to **20** rows.

## Durable regressions

Front-end source tests require both Script and Module goals to reject both a
class declaration and a class expression with two ordinary constructors. Each
rejection carries the new code, early phase, `SyntaxError`, and a source span.

A positive matrix under both goals preserves the exact boundary for both class
forms: one ordinary constructor coexists with `static constructor() {}` and
`["constructor"]() {}`. This prevents the classifier regression from becoming
an over-broad source rule.

The IR regression parses a real duplicate-constructor Module and sends its
retained `ParseError` through `module_parse_failure_diagnostic`. The
message-boundary table test independently fixes the exact literal-to-code
mapping.

## Pinned conformance evidence

Pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
contains exactly two direct generated witnesses for this condition:

- `language/expressions/class/elements/syntax/early-errors/grammar-class-body-ctor-duplicate.js`;
- `language/statements/class/elements/syntax/early-errors/grammar-class-body-ctor-duplicate.js`.

Both require `phase: parse` and `type: SyntaxError`. They establish the expected
classification, not a current Wasm-AOT pass claim. This static-only extension
consumes no current aggregate result and changes no snapshot or published
count.

## Deliberate separations and nonclaims

This extension classifies the parser rejection; it does not implement class
construction, constructor invocation, derived-constructor semantics, private
elements, static blocks, or class-field initialization. It does not combine
adjacent private/generator/accessor/async constructor restrictions, claim the
class parser bucket is closed, or complete T07.

No Test262 file or CLI fixture is added, and no snapshot is refreshed. Cargo,
focused execution, and current-pin Test262 verification remain deferred to the
serialized verification lane.
