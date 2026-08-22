# Contract: class-constructor generator-method early errors

**Status:** Normative T07 extension, 2026-08-20

## Spec invariant

The ClassElement early-error rules reject a non-static MethodDefinition whose
literal property name is `"constructor"` when its `SpecialMethod` result is
true. This bounded extension covers the generator and async-generator forms,
which pinned Boa reports through one message shape. The rejection occurs before
evaluation and is a `SyntaxError` under both Script and Module goals.

A static generator method named `constructor` is not a constructor definition.
A generator method with a computed name whose value is `"constructor"` also
has no syntactic `PropName` equal to `"constructor"`. Both remain valid and may
coexist with one ordinary constructor.

## Measured Boa boundary

Pinned `boa_parser-0.21.1` emits the exact, case-sensitive literal

`class constructor may not be a generator method`

at exactly two producer sites in
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs`:

- lines 786-792 reject a non-static `*constructor()`;
- lines 850-855 reject a non-static `async *constructor()`.

The complete literal occurs nowhere else in pinned Boa. Both producers inspect
the literal identifier before parsing the method. The static path and computed
property-name path bypass this rejection by construction.

Before this extension, the one classifier had no matching row. Entry parsing
therefore returned `P_PARSE_MALFORMED`; the same retained failure from a
dependency Module became an `Unsupported` IR diagnostic with no native error
type. Neither result represented the specified early `SyntaxError`.

Adjacent constructor restrictions remain separate conditions. Ordinary async,
getter, setter, private and duplicate constructors have distinct Boa literals
and must not be selected by this row.

## Encoding

- Add `EarlyErrorCode::ClassConstructorGeneratorMethod` with the sole wire
  spelling `E_CLASS_CONSTRUCTOR_GENERATOR_METHOD`.
- Add exactly one parse-failure row whose fragment and witness are the complete
  Boa literal. The shared wording deliberately unifies the generator and
  async-generator producers without matching any adjacent constructor message.
- Map the new variant through `lila-ir`'s exhaustive `rejection_kind` match to
  `IrDiagnosticKind::EarlyError`. Phase and error type remain derived as
  `Early` and `SyntaxError` at both front-end and retained-module boundaries.
- Preserve `ParseClassified` as the only parse-stage gate and retain the
  parse-once ownership path. No parser or classifier copy is introduced.

The const-sized closed domain grows from 23 to **24** variants and the one
parse-failure table grows from 21 to **22** rows. Existing const assertions
continue to encode table population, exact witness ownership, wire-name
closure, classifier reachability, interpolation-guard separation and
parse-to-IR phase consistency.

## Durable regressions

Front-end source tests require Script and Module goals to reject both a class
declaration with `*constructor()` and a class expression with
`async *constructor()`. Every rejection carries the new code, early phase,
`SyntaxError` and a source span.

A positive matrix under both goals keeps static generator and async-generator
methods plus computed generator and async-generator methods named
`constructor` valid beside one ordinary constructor. This fixes the syntactic
boundary of the condition rather than merely recording its error text.

The IR regression parses a real async-generator constructor Module and sends
its retained `ParseError` through `module_parse_failure_diagnostic`. The
message-boundary table test independently fixes the complete literal-to-code
mapping.

## Pinned conformance evidence

Pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
contains four direct generated negative witnesses, the generator and
async-generator cases for both class expressions and class declarations:

- `language/expressions/class/elements/syntax/early-errors/grammar-special-meth-ctor-gen.js`;
- `language/expressions/class/elements/syntax/early-errors/grammar-special-meth-ctor-async-gen.js`;
- `language/statements/class/elements/syntax/early-errors/grammar-special-meth-ctor-gen.js`;
- `language/statements/class/elements/syntax/early-errors/grammar-special-meth-ctor-async-gen.js`.

Each requires `phase: parse` and `type: SyntaxError`. The adjacent `syntax/valid`
directories contain the corresponding static generator and async-generator
forms for both class forms; the computed-name boundary is additionally pinned
by `language/computed-property-names/class/method/constructor-can-be-generator.js`
and `language/computed-property-names/class/static/generator-constructor.js`.
These fixtures establish the classification boundary, not a current Wasm-AOT
pass claim.

## Deliberate separations and nonclaims

This extension classifies a parser rejection Boa already produces. It does not
implement generator execution, class construction, method installation or
lowering. It does not combine ordinary async, getter, setter, private or
duplicate-constructor restrictions, close the class parser bucket, complete
T07, refresh a snapshot or change a published count.

No Test262 file or CLI fixture is added. Cargo, focused execution and
current-pin Test262 verification remain deferred to the shared verification
lane.
