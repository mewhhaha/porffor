# Contract: duplicate catch-parameter early errors

**Status:** Normative T07 extension, 2026-08-13

## Spec invariant

The TryStatement early-error rules reject a `CatchParameter` whose `BoundNames`
contains duplicate elements. This is one unconditional static-semantics
condition: unlike duplicate ordinary-function parameters, it has no sloppy
simple-list exception. In practice a duplicate requires an object or array
binding pattern because a bare `BindingIdentifier` contributes only one name.

The rejection is decided before evaluation and is a `SyntaxError`. Lila must
therefore represent it as one closed early-error condition, not as an ordinary
malformed parse or an unsupported dependency.

## Measured Boa boundary

Pinned `boa_parser-0.21.1` has exactly one producer and one case-sensitive
literal for this condition:

- `vendor/boa_parser-0.21.1/src/parser/statement/try_stm/catch.rs:69-85`
  computes `BoundNames` into a set and reports
  `duplicate catch parameter identifier` at line 78 when insertion finds a
  duplicate.

The source `try {} catch ({ a, b: a }) {}` reaches that producer. Boa itself
uses the same object-pattern shape in
`vendor/boa_parser-0.21.1/src/parser/statement/try_stm/tests.rs:410-413`.

Before this extension, the classifier had no catch-parameter row. An entry parse
consequently reported `P_PARSE_MALFORMED` instead of a named early-error
condition. The same source loaded as a dependency Module was retained as that
unclassified parse failure and `module_parse_failure_diagnostic` converted it
to `Unsupported`, losing both the spec rejection code and `SyntaxError` type.

## Encoding

- Add `EarlyErrorCode::DuplicateCatchParameter` with the sole wire spelling
  `E_DUPLICATE_CATCH_PARAMETER`.
- Add exactly one classifier row whose sole fragment and witness are the full
  case-sensitive Boa literal `duplicate catch parameter identifier`.
- Map the new variant through `lila-ir`'s exhaustive `rejection_kind` match to
  `IrDiagnosticKind::EarlyError`. Phase and error type remain derived as
  `Early` and `SyntaxError` on both entry and retained dependency paths.
- Keep `ParseClassified` as the parse-stage witness. The row makes this code
  constructible there, while the exhaustive IR match makes an omitted consumer
  a compile error.

At this extension's checkpoint the closed domain had 20 variants and the one
parse-failure table had 18 rows. The later catch-body-declaration-conflict
extension brings those counts to 21 and 19 without changing
`DuplicateCatchParameter`. Existing const gates prove row population, witness
disjointness, wire-name closure, classifier reachability, and parse-to-IR phase
consistency.

## Durable regressions

One repository fixture, independent of Test262 paths and source text, uses the
object-pattern source above. The front-end regression parses it under both
Script and Module goals and requires the same
`E_DUPLICATE_CATCH_PARAMETER`, early phase, `SyntaxError`, and source span.

The dependency regression parses real Module source through `lila-front` and
passes the resulting `ParseError` through
`module_parse_failure_diagnostic`. A message-only unit witness remains useful
for the table boundary, but cannot substitute for this source-to-IR route.

## Deliberate separations and nonclaims

`catch parameter identifier declared in catch body` is a different
TryStatement condition. Boa emits it from two branches at `catch.rs:91-113`:
one for overlap with `LexicallyDeclaredNames`, and one for overlap with
`VarDeclaredNames` when the catch parameter is a binding pattern. It must not
map to `DuplicateCatchParameter`. It is now classified separately as
`CatchBodyDeclarationConflict`; see
`catch-body-declaration-conflict-early-errors.md`.

`DuplicateFormalParameter` also remains distinct: it concerns
`FormalParameters`, has two pinned Boa wordings, and includes the sloppy
ordinary-function exception. This extension does not change formal-parameter
classification, catch binding initialization, destructuring evaluation, or
catch-environment lowering.

The pinned Test262 witness
`language/statements/try/early-catch-duplicates.js` specifies `phase: parse` and
`type: SyntaxError`, but no pass-count claim follows from this local closure.
Focused Cargo and current-pin Test262 verification remain deferred until the
shared verification lane is available.
