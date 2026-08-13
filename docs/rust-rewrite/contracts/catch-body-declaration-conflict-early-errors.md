# Contract: catch-body declaration-conflict early errors

**Status:** Normative T07 extension, 2026-08-13

## Spec invariant

The TryStatement early-error rules reject either of these intersections:

- `BoundNames` of `CatchParameter` with `LexicallyDeclaredNames` of the catch
  `Block`;
- `BoundNames` of a binding-pattern `CatchParameter` with `VarDeclaredNames` of
  the catch `Block`.

The second rule deliberately excludes `CatchParameter : BindingIdentifier`.
Consequently `try {} catch (a) { var a; }` remains valid, while
`try {} catch ({ a }) { var a; }` is an early error. This is the legacy simple
identifier exception specified by `sec-variablestatements-in-catch-blocks`, not
a parser tolerance.

Both intersections are one catch-parameter/body declaration-conflict union.
They reject before evaluation with `SyntaxError`; neither is a duplicate within
the catch parameter itself.

## Measured Boa boundary

Pinned `boa_parser-0.21.1` emits one exact, case-sensitive literal from two
reachable branches in
`vendor/boa_parser-0.21.1/src/parser/statement/try_stm/catch.rs:91-113`:

`catch parameter identifier declared in catch body`

- line 99 reports the `BoundNames` / `LexicallyDeclaredNames` intersection;
- line 108 reports the binding-pattern `BoundNames` / `VarDeclaredNames`
  intersection. The guard at line 104 preserves the simple
  `BindingIdentifier` exception.

Those are the only two occurrences of the literal in the pinned Boa source.
Boa's own `try_stm/tests.rs:416-418` covers the lexical branch. It has no local
source regression for the pattern/`var` branch, so Lila keeps both branches as
explicit source tests.

Before this extension, the one classifier had no row for the literal. Entry
parsing therefore returned `P_PARSE_MALFORMED`; the same failed parse retained
for a dependency Module became an `Unsupported` IR diagnostic with no native
error type. Neither result represented the specified early `SyntaxError`.

## Encoding

- Add the union-shaped `EarlyErrorCode::CatchBodyDeclarationConflict` with the
  sole wire spelling `E_CATCH_BODY_DECLARATION_CONFLICT`.
- Add exactly one parse-failure row. Its one fragment and one witness are the
  complete Boa literal above. Splitting the two source branches into separate
  codes would pretend that the parser reports information it does not carry.
- Map the new variant through `lila-ir`'s exhaustive `rejection_kind` match to
  `IrDiagnosticKind::EarlyError`. Phase and error type remain derived as
  `Early` and `SyntaxError` for entry and retained-module diagnostics.
- Keep `ParseClassified` as the parse-stage gate. Existing const assertions
  continue to prove row population, witness disjointness, wire-name closure,
  classifier reachability, interpolation-guard separation, and parse-to-IR
  phase consistency.

This extension brings the closed domain from 20 to **21** variants and the one
parse-failure table from 18 to **19** rows.

## Durable regressions

Front-end source tests require both Script and Module goals to reject:

- `try {} catch (a) { let a; }` for the lexical-declaration branch;
- `try {} catch ({ a }) { var a; }` for the pattern/`var` branch.

Each rejection must carry the new code, early phase, `SyntaxError`, and a source
span. A positive test under both goals requires
`try {} catch (a) { var a; }` to parse, preventing the union from erasing the
simple-identifier exception.

The IR regression parses both invalid sources as real Modules and sends the
result through `module_parse_failure_diagnostic`. The message-boundary table
test independently fixes the exact literal-to-code mapping.

## Pinned conformance evidence

Pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
contains 201 files under `language/statements/try`. Its negative lexical
witnesses `early-catch-lex.js` and `early-catch-function.js` require
`phase: parse` and `type: SyntaxError`.

Five positive Annex B witnesses preserve the simple-identifier `var`
exception: `catch-redeclared-for-in-var.js`, `catch-redeclared-for-of-var.js`,
`catch-redeclared-for-var.js`, `catch-redeclared-var-statement-captured.js`, and
`catch-redeclared-var-statement.js`.

These files establish the expected semantics, not a current Wasm-AOT pass
claim. This static-only extension consumes no current-pin run result and changes
no snapshot or published count.

## Deliberate separations and nonclaims

`DuplicateCatchParameter` remains the distinct condition where `BoundNames` of
the `CatchParameter` contains a duplicate. This extension also does not claim
strict `eval`/`arguments` catch-name restrictions, optional-catch-binding
grammar, runtime catch-environment construction, destructuring evaluation,
dynamic `eval`/`Function` support, strict-mode `with`, or broader T07 grammar
closure.

No fixture or Test262 file is added, and no snapshot is refreshed. Cargo and
focused/current-pin Test262 verification remain deferred to the serialized
verification lane.
